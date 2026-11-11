//! A storage aisle of a high-bay warehouse in the 3D run (S26, ADR-0033 §5). The aisle's logic is the 2D run's
//! (`intracraft_des::flow::storage`); the plant around it is physical: a pallet runs to the end of its infeed chains and
//! stops there by friction; when the forks lift it, it stops being a body and is carried guided (in the rack, on the
//! forks); when the forks set it down at the start of the outfeed chains, it is a body again, at rest on the moving
//! chains as a source's new pallet is.

use std::collections::{BTreeMap, BTreeSet};

use intracraft_des::flow::network::{Base, FlowIssue};
use intracraft_des::flow::sim::Pallet;
use intracraft_des::flow::storage::{AisleView, Request, StorageAisle, StorageData};
use intracraft_des::time::SimTime;
use intracraft_library::reference::{PalletType, Reference};

use crate::bench::Motion;
use crate::flow::pallet::{WorldPallet, pallet_parts};
use crate::flow::{PhysicsFlow, SPAWN_LIFT};

/// A pallet whose front is this close to its deck's end, m, and whose speed is under this, m/s, rests there: the
/// forks can take it. A pallet stopped by friction lands within a few millimetres of the end (T14).
const AT_END: f64 = 0.03;
const AT_REST: f64 = 0.01;

/// A storage aisle of the 3D run: its logic, and the deck at each of its connected entries and exits.
pub struct Storage3D {
    pub logic: StorageAisle,
    entries: Vec<Option<usize>>,
    exits: Vec<Option<usize>>,
    types: Vec<PalletType>,
    warned: BTreeSet<&'static str>,
}

impl PhysicsFlow {
    pub(super) fn add_storage(&mut self, b: &Base, data: &StorageData, reference: &Reference, seed: u64, index: u64) {
        self.storages.push(Storage3D {
            logic: StorageAisle::new(&b.id, data.clone(), reference, seed, index),
            entries: Vec::new(),
            exits: Vec::new(),
            types: reference.pallet_types.clone(),
            warned: BTreeSet::new(),
        });
    }

    /// Once the decks are made: the conveyor that ends at each entry, and the deck that starts at each exit.
    pub(super) fn connect_storages(&mut self) {
        for k in 0..self.storages.len() {
            let aisle = &self.storages[k].logic;
            let entries = aisle
                .entries
                .iter()
                .map(|p| {
                    let from = aisle.data.feeders.get(&p.port_id)?;
                    self.decks.iter().position(|d| d.station.is_none() && &d.spec.id == from)
                })
                .collect();
            let exits = aisle
                .exit_ports
                .iter()
                .map(|p| {
                    let to = aisle.data.exits.get(&p.port_id)?;
                    self.entry_deck_of(&to.target, Some(&to.port)).filter(|&d| self.decks[d].station.is_none())
                })
                .collect();
            self.storages[k].entries = entries;
            self.storages[k].exits = exits;
        }
    }

    /// Each aisle sees the pallets resting at its entries, then goes on as far as it can: the run lifts a pallet off an
    /// entry's chains, says whether an exit has room, and sets a pallet down there.
    pub(super) fn serve_storages(&mut self, now: SimTime) {
        for k in 0..self.storages.len() {
            for e in 0..self.storages[k].entries.len() {
                let resting = self.storages[k].entries[e].and_then(|d| self.resting_at_end(d));
                self.storages[k].logic.see_entry(e, resting, now);
            }
            loop {
                match self.storages[k].logic.next(now) {
                    Request::Nothing | Request::Wait(_) => break,
                    Request::Take { entry } => {
                        let deck = self.storages[k].entries.get(entry).copied().flatten();
                        let pallet = deck.and_then(|d| self.resting_at_end(d).map(|id| (d, id))).and_then(|(d, id)| {
                            let entered = self.pallets.get(&id).map_or(now, |p| p.pallet.entered_at);
                            self.decks[d].release(id, entered, now);
                            self.decks[d].holding = false;
                            self.remove_pallet(id).map(|p| p.pallet)
                        });
                        self.storages[k].logic.give(pallet, now);
                    }
                    Request::Room { exit, length } => {
                        let deck = self.storages[k].exits.get(exit).copied().flatten();
                        let free = deck.is_some_and(|d| self.can_enter(d, length + self.decks[d].spec.min_gap));
                        self.storages[k].logic.room(free, now);
                        if !free {
                            break;
                        }
                    }
                    Request::Put { exit } => {
                        let deck = self.storages[k].exits.get(exit).copied().flatten();
                        if let (Some(pallet), Some(d)) = (self.storages[k].logic.release(now), deck) {
                            let types = &self.storages[k].types;
                            let pallet_type = types.iter().find(|t| t.id == pallet.type_id).or(types.first()).cloned();
                            if let Some(pallet_type) = pallet_type {
                                self.set_down(d, pallet, &pallet_type, now);
                            }
                        }
                    }
                }
            }
            let notes = std::mem::take(&mut self.storages[k].logic.notes);
            for (rule, text) in notes {
                if self.storages[k].warned.insert(rule) {
                    let id = self.storages[k].logic.id.clone();
                    self.issues.push(FlowIssue { instance_id: id.clone(), rule, message: format!("{id}: {text}") });
                }
            }
        }
    }

    /// The pallet standing still at the end of `deck`, held there: what the forks can take.
    fn resting_at_end(&self, deck: usize) -> Option<u64> {
        let d = &self.decks[deck];
        if !d.holding || d.outgoing.is_some() || d.speed > AT_REST {
            return None;
        }
        let lead = d.lead(&self.pallets)?;
        let e = d.extent(self.pallets.get(&lead)?);
        (e.front >= d.end - AT_END && e.speed.abs() < AT_REST).then_some(lead)
    }

    /// The forks set `pallet` down at the start of `deck`: a body again, at rest on its moving surface.
    fn set_down(&mut self, deck: usize, mut pallet: Pallet, pallet_type: &PalletType, now: SimTime) {
        let d = &self.decks[deck];
        let length = pallet.length;
        let x = if d.direction > 0.0 { length / 2.0 } else { d.spec.length - length / 2.0 };
        let centre = d.frame.to_world([x, SPAWN_LIFT, 0.0]);
        let yaw = d.frame.yaw + if d.direction < 0.0 { std::f64::consts::PI } else { 0.0 };
        let (velocity, carrier) = (d.surface_velocity(), d.carrier.clone());
        pallet.front = length;
        pallet.entered_at = now;
        let parts = pallet_parts(&pallet, pallet_type);
        let (body, _) = self.bench.body(Motion::Dynamic, centre, yaw, 0.0, &parts);
        self.bench.set_velocity(body, velocity, [0.0; 3]);
        let (x_axis, z_axis) = self.bench.axes(body);
        let id = pallet.id;
        self.pallets.insert(id, WorldPallet { pallet, body, carrier, centre, x_axis, z_axis, velocity });
        self.bodies.insert(body);
        self.decks[deck].pallets.push(id);
    }

    /// Every storage aisle's crane, the pallet on its forks and the pallets in its rack, now.
    pub fn storage(&self) -> BTreeMap<String, AisleView> {
        let now = self.now();
        self.storages.iter().map(|s| (s.logic.id.clone(), s.logic.view(now))).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use intracraft_des::time::from_seconds;
    use intracraft_library::Library;
    use intracraft_library::definition::Value;
    use intracraft_library::layout::{Instance, Layout, Vec3};

    use crate::flow::PhysicsFlow;

    const ROW: f64 = 1.4;

    #[test]
    fn a_pallet_rolls_to_the_end_of_the_chains_is_stored_by_the_forks_and_comes_back_out_as_a_body() {
        let instance = |id: &str, definition: &str, x: f64, z: f64, yaw: f64, params: &[(&str, Value)]| Instance {
            id: id.into(),
            definition_id: definition.into(),
            definition_version: "0.1.0".into(),
            name: id.into(),
            position: Vec3::new(x, 0.0, z),
            yaw,
            params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            attributes: Default::default(),
        };
        let source = [("orientation", Value::Text("short".into())), ("maxPallets", Value::Number(2.0)), ("interval", Value::Number(20.0))];
        let aisle = [
            ("bays", Value::Number(6.0)),
            ("levels", Value::Number(3.0)),
            ("initialFill", Value::Number(0.0)),
            ("dwellTime", Value::Number(30.0)),
        ];
        let layout = Layout {
            instances: vec![
                instance("source1", "flow.source", -3.44, -ROW, 0.0, &source),
                instance("in1", "conveyor.chain", -3.02, -ROW, 0.0, &[]),
                instance("aisle1", "storage.aisle", 0.0, 0.0, 0.0, &aisle),
                instance("out1", "conveyor.chain", -0.02, ROW, PI, &[]),
                instance("sink1", "flow.sink", -3.04, ROW, PI, &[]),
            ],
            ..Layout::default()
        };
        let lib = Library::built_in().unwrap();
        let mut flow = PhysicsFlow::new(&layout, &lib, 1).unwrap();
        assert!(flow.issues.is_empty(), "{:?}", flow.issues);
        let mut carried = false;
        for s in 1..=240 {
            flow.run_until(from_seconds(f64::from(s)));
            let view = &flow.storage()["aisle1"];
            carried |= view.carried.is_some();
            // A pallet stored or on the forks is no body: it is in one place only.
            let bodies = flow.pallets().len();
            assert!(bodies + view.stored.len() + usize::from(view.carried.is_some()) <= 2, "at {s} s");
        }
        assert!(carried);
        let stats = flow.statistics();
        assert_eq!((stats["aisle1"]["palletsStored"], stats["aisle1"]["palletsRetrieved"]), (2.0, 2.0), "{stats:?}");
        assert_eq!(stats["sink1"]["palletsReceived"], 2.0);
        assert_eq!(flow.fallen_pallets(), 0);
    }
}
