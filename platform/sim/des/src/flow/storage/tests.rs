//! The storage aisle (S26 §8): the crane's times, where pallets go and in which order they leave, alone and in a plant.

use std::f64::consts::PI;

use intracraft_library::Library;
use intracraft_library::layout::{Instance, Layout, Vec3};
use intracraft_library::params::instance_values;

use super::*;
use crate::flow::{FlowSimulation, simulate};
use crate::time::to_seconds;

const AISLE: &str = "aisle1";
/// Where the rows' front positions are across the default aisle, m.
const ROW: f64 = 1.4;

fn n(x: f64) -> Value {
    Value::Number(x)
}

fn t(s: &str) -> Value {
    Value::Text(s.into())
}

mod crane {
    use super::*;

    #[test]
    fn t12_a_move_of_30_m_along_and_10_m_up_takes_the_longer_of_both_16_s() {
        let d = data(&[]);
        assert!((d.move_time([0.0, 0.0, 0.0], [30.0, 10.0, 0.0]) - 16.0).abs() < 1e-9);
        // Lifting decides a short, high move: 3 m along (4.9 s) and 10 m up (12 s).
        assert!((d.move_time([0.0, 0.0, 0.0], [3.0, 10.0, 0.0]) - 12.0).abs() < 1e-9);
    }

    #[test]
    fn t13_a_single_cycle_to_bay_30_level_6_and_back_takes_47_93_s() {
        let values = [("bays", n(30.0)), ("levels", n(6.0)), ("levelPitch", n(2.0)), ("initialFill", n(0.0)), ("dwellPoint", t("front"))];
        let mut b = Bench::new(&values, &["IN1"], &[]);
        let slot = (29 * 6 + 5) * 2;
        assert_eq!(b.aisle.data.place_of(slot), [29.5, 10.6, -ROW]);
        b.arrive(0, 0);
        b.aisle.see_entry(0, Some(1), 0);
        b.aisle.start_job(Job { kind: JobKind::Store, pick: Target::Entry(0), place: Target::Slot(slot), started: 0 }, 0);
        b.run_until(from_seconds(200.0));
        // 8 s at the station, max(29.9 / 3 + 6, 10 / 1 + 2) = 15.97 s there, 8 s at the position, 15.97 s back.
        let expected = 2.0 * (29.9 / 3.0 + 6.0) + 2.0 * 8.0;
        assert!((to_seconds(b.idle_since) - expected).abs() / expected < 0.01, "{} s", to_seconds(b.idle_since));
        assert!((expected - 47.93).abs() < 0.005);
        assert_eq!(b.aisle.stock(), 1);
        assert_eq!(b.aisle.view(b.now).crane, [STATION_X, 0.6, 0.0]);
    }

    #[test]
    fn a_rear_position_takes_the_longer_fork_cycle() {
        let d = data(&[("deep", n(2.0))]);
        assert_eq!((d.fork_cycle(Some(0)), d.fork_cycle(Some(1)), d.fork_cycle(None)), (8.0, 12.0, 8.0));
        assert_eq!(d.place_of(1), [0.5, 0.6, -(ROW + 1.3)]);
        assert_eq!(d.place_of(2), [0.5, 0.6, ROW]);
        assert_eq!(d.positions(), 640);
    }

    #[test]
    fn the_pallet_changes_hands_in_the_middle_of_a_fork_cycle_and_rides_the_forks_back() {
        let mut b = Bench::new(&[("initialFill", n(0.0))], &["IN1"], &[]);
        b.arrive(0, 0);
        b.run_until(from_seconds(3.9));
        assert!(b.at_entry[0].is_some() && b.aisle.view(b.now).carried.is_none());
        b.run_until(from_seconds(4.0));
        let view = b.aisle.view(b.now);
        assert!(b.at_entry[0].is_none());
        // Lifted off the conveyor where it stood: at the station, on the left row.
        assert_eq!(view.carried.map(|p| p.at), Some([STATION_X, 0.6, -ROW]));
        b.run_until(from_seconds(8.0));
        assert_eq!(b.aisle.view(b.now).crane, [STATION_X, 0.6, 0.0]);
    }
}

mod rack {
    use super::*;

    #[test]
    fn the_nearest_position_is_filled_first_rear_before_front_then_the_other_row() {
        let mut b = Bench::new(&[("deep", n(2.0)), ("initialFill", n(0.0)), ("retrievalMode", t("none"))], &["IN1"], &[]);
        for k in 0..3 {
            b.arrive(from_seconds(100.0 * f64::from(k)), 0);
        }
        b.run_until(from_seconds(400.0));
        let places: Vec<[f64; 3]> = b.aisle.view(b.now).stored.iter().map(|p| p.at).collect();
        assert_eq!(places, [[0.5, 0.6, -ROW], [0.5, 0.6, -(ROW + 1.3)], [0.5, 0.6, ROW + 1.3]]);
        let stats: BTreeMap<_, _> = b.aisle.statistics().into_iter().collect();
        assert_eq!((stats["palletsStored"], stats["stock"]), (3.0, 3.0));
    }

    #[test]
    fn the_initial_stock_fills_the_share_asked_for_nearest_first() {
        let b = Bench::new(&[("initialFill", n(0.25))], &["IN1"], &["OUT1"]);
        assert_eq!(b.aisle.stock(), 80);
        let stored = b.aisle.view(0).stored;
        assert!(stored.iter().any(|p| p.at == [0.5, 0.6, -ROW]) && stored.iter().any(|p| p.at == [0.5, 0.6, ROW]));
        // Nothing far away while nearer positions are free: the crane reaches bay 6 on level 1 in 5.7 s.
        assert!(stored.iter().all(|p| p.at[0] < 12.0), "{:?}", stored.iter().map(|p| p.at[0]).fold(0.0, f64::max));
    }

    #[test]
    fn a_full_aisle_leaves_the_pallet_at_its_entry() {
        let values = [("bays", n(1.0)), ("levels", n(1.0)), ("initialFill", n(0.0)), ("retrievalMode", t("none"))];
        let mut b = Bench::new(&values, &["IN1"], &[]);
        for k in 0..3 {
            b.arrive(from_seconds(60.0 * f64::from(k)), 0);
        }
        b.run_until(from_seconds(600.0));
        assert_eq!(b.aisle.stock(), 2);
        assert!(b.at_entry[0].is_some());
        assert_eq!(b.aisle.status(), "idle");
    }

    #[test]
    fn a_fixed_dwell_time_asks_each_pallet_out_that_long_after_it_was_set_down() {
        let values = [("initialFill", n(0.0)), ("dwellTime", n(300.0))];
        let mut b = Bench::new(&values, &["IN1"], &["OUT1"]);
        b.arrive(0, 0);
        // Set down in bay 1 at 8 s + 2.68 s (0.9 m: 2 × √(0.9 / 0.5)) + 4 s; on the exit 300 s later + 8 + 2.68 + 4 s.
        let set_down = 8.0 + 2.0 * (0.9_f64 / 0.5).sqrt() + 4.0;
        b.run_until(from_seconds(set_down + 300.0 + 4.0 + 2.6));
        assert!(b.out.is_empty());
        b.run_until(from_seconds(400.0));
        assert_eq!(b.out.len(), 1);
        let left = to_seconds(b.out[0].0);
        assert!((left - (set_down + 300.0 + 8.0 + 2.0 * (0.9_f64 / 0.5).sqrt() + 4.0)).abs() < 1e-3, "{left}");
    }

    #[test]
    fn orders_at_an_interval_take_the_pallets_in_the_order_they_came() {
        let values = [("initialFill", n(0.0)), ("retrievalMode", t("interval")), ("retrievalInterval", n(500.0))];
        let mut b = Bench::new(&values, &["IN1"], &["OUT1"]);
        for k in 0..3 {
            b.arrive(from_seconds(60.0 * f64::from(k)), 0);
        }
        b.run_until(from_seconds(2400.0));
        assert_eq!(b.out.iter().map(|(_, id)| *id).collect::<Vec<_>>(), [1, 2, 3]);
        // The fourth order found the rack empty.
        let stats: BTreeMap<_, _> = b.aisle.statistics().into_iter().collect();
        assert_eq!((stats["palletsRetrieved"], stats["ordersMissed"], stats["stock"]), (3.0, 1.0, 0.0));
    }

    #[test]
    fn a_rear_pallet_behind_another_is_reached_by_relocating_the_front_one() {
        let values = [("deep", n(2.0)), ("initialFill", n(0.0)), ("retrievalMode", t("interval")), ("retrievalInterval", n(300.0))];
        let mut b = Bench::new(&values, &["IN1"], &["OUT1"]);
        b.arrive(0, 0);
        b.arrive(from_seconds(60.0), 0);
        b.run_until(from_seconds(500.0));
        // The first pallet stood at the rear; the second, in front of it, moved to the other row's rear position.
        assert_eq!(b.out.iter().map(|(_, id)| *id).collect::<Vec<_>>(), [1]);
        let stats: BTreeMap<_, _> = b.aisle.statistics().into_iter().collect();
        assert_eq!((stats["relocations"], stats["palletsStored"], stats["stock"]), (1.0, 2.0, 1.0));
        assert_eq!(b.aisle.view(b.now).stored.iter().map(|p| (p.id, p.at)).collect::<Vec<_>>(), [(2, [0.5, 0.6, ROW + 1.3])]);
        // The relocated pallet keeps its turn: the next order takes it.
        b.run_until(from_seconds(700.0));
        assert_eq!(b.out.iter().map(|(_, id)| *id).collect::<Vec<_>>(), [1, 2]);
    }

    #[test]
    fn storage_and_retrieval_take_turns_and_a_retrieval_after_a_storage_is_a_combined_cycle() {
        let values = [("initialFill", n(0.5)), ("retrievalMode", t("interval")), ("retrievalInterval", n(30.0))];
        let mut b = Bench::new(&values, &["IN1"], &["OUT1"]);
        for k in 0..40 {
            b.arrive(from_seconds(30.0 * f64::from(k)), 0);
        }
        b.run_until(from_seconds(1800.0));
        let stats: BTreeMap<_, _> = b.aisle.statistics().into_iter().collect();
        assert!(stats["palletsStored"] >= 30.0 && stats["palletsRetrieved"] >= 30.0, "{stats:?}");
        assert!(stats["combinedShare"] > 0.8, "{stats:?}");
        assert!((stats["utilization"] + stats["blockedShare"] + stats["idleShare"] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_full_exit_holds_the_crane_blocked_with_its_pallet() {
        let values = [("initialFill", n(0.5)), ("retrievalMode", t("interval")), ("retrievalInterval", n(60.0))];
        let mut b = Bench::new(&values, &[], &["OUT1"]);
        b.exit_free = false;
        b.run_until(from_seconds(600.0));
        assert_eq!(b.aisle.status(), "blocked");
        assert!(b.aisle.view(b.now).carried.is_some() && b.out.is_empty());
        let stats: BTreeMap<_, _> = b.aisle.statistics().into_iter().collect();
        assert!(stats["blockedShare"] > 0.8, "{stats:?}");
        b.exit_free = true;
        b.run_until(from_seconds(660.0));
        // Set down half a fork cycle after the exit freed; the orders that queued up meanwhile follow at once.
        assert_eq!(b.out.first().map(|(time, _)| to_seconds(*time)), Some(604.0));
        assert!(b.out.len() > 1);
    }

    #[test]
    fn a_random_strategy_and_exponential_times_give_the_same_run_for_the_same_seed() {
        let run = || {
            let values = [
                ("initialFill", n(0.3)),
                ("storageStrategy", t("random")),
                ("timeDistribution", t("exponential")),
                ("dwellTime", n(900.0)),
            ];
            let mut b = Bench::new(&values, &["IN1"], &["OUT1"]);
            for k in 0..20 {
                b.arrive(from_seconds(45.0 * f64::from(k)), 0);
            }
            b.run_until(from_seconds(1800.0));
            (b.out.clone(), b.aisle.view(b.now))
        };
        let (out, view) = run();
        assert!(out.len() > 5 && !view.stored.is_empty());
        assert_eq!(run(), (out, view));
    }
}

mod plant {
    use super::*;

    #[test]
    fn pallets_from_a_source_are_stored_and_come_back_out_to_a_sink() {
        let layout = plant(&[("initialFill", n(0.0)), ("dwellTime", n(120.0))], &[("maxPallets", n(3.0)), ("interval", n(40.0))], &[]);
        let mut sim = run(&layout, 900.0);
        assert!(sim.issues.is_empty(), "{:?}", sim.issues);
        let stats = sim.statistics();
        assert_eq!((stats[AISLE]["palletsStored"], stats[AISLE]["palletsRetrieved"]), (3.0, 3.0));
        assert_eq!(stats["sink1"]["palletsReceived"], 3.0);
        assert_eq!(stats["in1"]["palletsProcessed"], 3.0);
        assert_eq!(sim.snapshot().status[AISLE], "idle");
    }

    #[test]
    fn a_pallet_waits_at_the_end_of_the_chains_until_the_forks_lift_it() {
        let layout = plant(&[("initialFill", n(0.0)), ("retrievalMode", t("none"))], &[("maxPallets", n(1.0))], &[]);
        // 3 m of chains from rest at 0.2 m/s... the pallet is at the end well before 30 s; the crane, at the station, takes
        // 4 s to lift it.
        let lib = Library::built_in().unwrap();
        let mut sim = simulate(&layout, &lib, 1).unwrap();
        let mut picked = None;
        for step in 1..=600 {
            sim.run_until(from_seconds(f64::from(step) * 0.1)).unwrap();
            let snap = sim.snapshot();
            if snap.storage[AISLE].carried.is_some() {
                picked = Some((step, snap.pallets.len()));
                break;
            }
            assert!(snap.pallets.iter().all(|p| p.front <= 3.0 + 1e-6));
        }
        let (step, on_conveyors) = picked.unwrap();
        assert_eq!(on_conveyors, 0);
        assert!(step > 40, "picked after {} s", f64::from(step) * 0.1);
    }

    #[test]
    fn a_full_aisle_blocks_its_infeed_conveyor_and_the_source_behind_it() {
        let aisle = [("bays", n(1.0)), ("levels", n(1.0)), ("initialFill", n(0.0)), ("retrievalMode", t("none"))];
        let layout = plant(&aisle, &[("interval", n(20.0))], &["out1", "sink1"]);
        let mut sim = run(&layout, 1200.0);
        let stats = sim.statistics();
        assert_eq!((stats[AISLE]["palletsStored"], stats[AISLE]["fillLevel"]), (2.0, 1.0));
        assert_eq!(sim.snapshot().status["in1"], "blocked");
        // Two in the rack and what the 3 m of chains hold: the source stopped making pallets.
        assert!(stats["source1"]["palletsCreated"] <= 6.0, "{stats:?}");
    }

    #[test]
    fn a_sink_that_does_not_take_pallets_fills_the_outfeed_and_blocks_the_crane() {
        let aisle = [("initialFill", n(0.5)), ("retrievalMode", t("interval")), ("retrievalInterval", n(30.0))];
        let mut layout = plant(&aisle, &[], &["source1", "in1"]);
        for i in &mut layout.instances {
            if i.id == "sink1" {
                i.params.insert("removalMode".into(), t("afterTime"));
                i.params.insert("removalTime".into(), n(100_000.0));
            }
        }
        let mut sim = run(&layout, 1800.0);
        let stats = sim.statistics();
        assert_eq!(sim.snapshot().status[AISLE], "blocked", "{stats:?} {:?}", sim.issues);
        assert!(stats[AISLE]["blockedShare"] > 0.5, "{stats:?}");
        // One at the sink, and the pallets 3 m of chains hold.
        assert!((2.0..=5.0).contains(&stats[AISLE]["palletsRetrieved"]), "{stats:?}");
    }

    #[test]
    fn warns_about_an_aisle_without_stations_without_an_exit_or_fed_by_rollers() {
        let rules = |layout: &Layout| -> Vec<&'static str> {
            let lib = Library::built_in().unwrap();
            simulate(layout, &lib, 1).unwrap().issues.iter().filter(|i| i.instance_id == AISLE).map(|i| i.rule).collect()
        };
        assert_eq!(rules(&plant(&[], &[], &["source1", "in1", "out1", "sink1"])), ["noStation"]);
        assert_eq!(rules(&plant(&[], &[], &["out1", "sink1"])), ["retrievalWithoutExit"]);
        assert_eq!(rules(&plant(&[("retrievalMode", t("none"))], &[], &[])), ["exitWithoutRetrieval"]);
        let mut rollers = plant(&[], &[], &[]);
        for i in &mut rollers.instances {
            if i.id == "in1" {
                i.definition_id = "conveyor.roller".into();
            }
        }
        assert_eq!(rules(&rollers), ["stationNotOnChains"]);
    }

    #[test]
    fn stations_at_two_heights_take_pallets_in_below_and_give_them_out_above() {
        let aisle = [("pdLevels", n(2.0)), ("initialFill", n(0.0)), ("dwellTime", n(60.0))];
        let mut layout = plant(&aisle, &[("maxPallets", n(2.0)), ("interval", n(30.0))], &[]);
        // The outfeed and its sink go up to the second height: 4.6 m, on a base 4.0 m high.
        for i in &mut layout.instances {
            if i.id == "out1" || i.id == "sink1" {
                i.position.y = 4.0;
            }
        }
        let lib = Library::built_in().unwrap();
        let mut sim = simulate(&layout, &lib, 1).unwrap();
        assert!(sim.issues.is_empty(), "{:?}", sim.issues);
        let aisle = sim.nodes().iter().find(|n| n.base.id == AISLE).unwrap();
        let crate::flow::network::Kind::Storage(data) = &aisle.kind else { panic!() };
        assert_eq!(data.feeders.keys().map(String::as_str).collect::<Vec<_>>(), ["IN1"]);
        assert_eq!(data.exits.keys().map(String::as_str).collect::<Vec<_>>(), ["OUT2"]);
        sim.run_until(from_seconds(900.0)).unwrap();
        assert_eq!(sim.statistics()["sink1"]["palletsReceived"], 2.0);
    }

    #[test]
    fn the_same_plant_and_seed_give_the_same_run() {
        let aisle =
            [("initialFill", n(0.4)), ("storageStrategy", t("random")), ("timeDistribution", t("exponential")), ("dwellTime", n(600.0))];
        let layout = plant(&aisle, &[("generationMode", t("exponential")), ("interval", n(40.0))], &[]);
        let a = run(&layout, 3600.0).snapshot();
        assert_eq!(a, run(&layout, 3600.0).snapshot());
        assert!(a.storage[AISLE].stored.len() > 50);
    }

    fn run(layout: &Layout, seconds: f64) -> FlowSimulation {
        let lib = Library::built_in().unwrap();
        let mut sim = simulate(layout, &lib, 7).unwrap();
        sim.run_until(from_seconds(seconds)).unwrap();
        sim
    }

    /// source1 → in1 (chains) → aisle1 → out1 (chains) → sink1, without the instances named in `without`.
    fn plant(aisle: &[(&str, Value)], source: &[(&str, Value)], without: &[&str]) -> Layout {
        let mut source_values = vec![("orientation", t("short"))];
        source_values.extend(source.iter().cloned());
        let instances = vec![
            instance("source1", "flow.source", -3.44, -ROW, 0.0, &source_values),
            instance("in1", "conveyor.chain", -3.02, -ROW, 0.0, &[]),
            instance(AISLE, "storage.aisle", 0.0, 0.0, 0.0, aisle),
            instance("out1", "conveyor.chain", -0.02, ROW, PI, &[]),
            instance("sink1", "flow.sink", -3.04, ROW, PI, &[]),
        ];
        Layout { instances: instances.into_iter().filter(|i| !without.contains(&i.id.as_str())).collect(), ..Layout::default() }
    }

    fn instance(id: &str, definition: &str, x: f64, z: f64, yaw: f64, params: &[(&str, Value)]) -> Instance {
        Instance {
            id: id.into(),
            definition_id: definition.into(),
            definition_version: "0.1.0".into(),
            name: id.into(),
            position: Vec3::new(x, 0.0, z),
            yaw,
            params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            attributes: Default::default(),
        }
    }
}

fn data(own: &[(&str, Value)]) -> StorageData {
    let lib = Library::built_in().unwrap();
    let own: Values = own.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    StorageData::read(&instance_values(lib.get("storage.aisle").unwrap(), &own))
}

/// An aisle alone, with pallets put at its entries at given times and its exits free or not: what a run does for it.
struct Bench {
    aisle: StorageAisle,
    now: SimTime,
    /// When a pallet reaches each entry, and which.
    arrivals: Vec<(SimTime, usize)>,
    at_entry: Vec<Option<Pallet>>,
    /// When each pallet was set down on an exit, and its id.
    out: Vec<(SimTime, u64)>,
    exit_free: bool,
    next_id: u64,
    /// When the crane last had nothing left to do.
    idle_since: SimTime,
}

impl Bench {
    fn new(values: &[(&str, Value)], entries: &[&str], exits: &[&str]) -> Bench {
        let lib = Library::built_in().unwrap();
        let mut data = data(values);
        for port in entries {
            data.feeders.insert(port.to_string(), format!("conveyor {port}"));
        }
        for port in exits {
            data.exits.insert(port.to_string(), StationExit { target: format!("conveyor {port}"), gap: 0.02, port: "A".into() });
        }
        Bench {
            aisle: StorageAisle::new(AISLE, data, &lib.reference, 5, 1),
            now: 0,
            arrivals: Vec::new(),
            at_entry: vec![None; entries.len()],
            out: Vec::new(),
            exit_free: true,
            next_id: 1,
            idle_since: 0,
        }
    }

    /// A pallet reaches entry `entry` at `time` (or as soon as the one before it has gone).
    fn arrive(&mut self, time: SimTime, entry: usize) {
        self.arrivals.push((time, entry));
    }

    fn run_until(&mut self, end: SimTime) {
        loop {
            let mut k = 0;
            while k < self.arrivals.len() {
                let (time, entry) = self.arrivals[k];
                if time <= self.now && self.at_entry[entry].is_none() {
                    self.at_entry[entry] = Some(pallet(self.next_id, self.now));
                    self.next_id += 1;
                    self.arrivals.remove(k);
                } else {
                    k += 1;
                }
            }
            for (entry, p) in self.at_entry.iter().enumerate() {
                self.aisle.see_entry(entry, p.as_ref().map(|p| p.id), self.now);
            }
            let was_busy = self.aisle.status() != "idle";
            loop {
                match self.aisle.next(self.now) {
                    Request::Nothing | Request::Wait(_) => break,
                    Request::Take { entry } => {
                        let pallet = self.at_entry[entry].take();
                        self.aisle.give(pallet, self.now);
                    }
                    Request::Room { .. } => {
                        self.aisle.room(self.exit_free, self.now);
                        if !self.exit_free {
                            break;
                        }
                    }
                    Request::Put { .. } => {
                        if let Some(p) = self.aisle.release(self.now) {
                            self.out.push((self.now, p.id));
                        }
                    }
                }
            }
            if was_busy && self.aisle.status() == "idle" {
                self.idle_since = self.now;
            }
            let arrival = self.arrivals.iter().map(|(time, _)| *time).filter(|time| *time > self.now).min();
            let next = [self.aisle.next_wake(), arrival].into_iter().flatten().min().filter(|time| *time <= end).unwrap_or(end);
            self.aisle.accumulate((next - self.now) as f64 / US_PER_S as f64);
            if next <= self.now {
                break;
            }
            self.now = next;
        }
    }
}

/// A EUR pallet as a chain conveyor brings it: short side first.
fn pallet(id: u64, now: SimTime) -> Pallet {
    Pallet {
        id,
        type_id: "eur".into(),
        source: "source1".into(),
        along: Side::Short,
        length: 0.8,
        width: 1.2,
        height: 0.144,
        load_mass: 600.0,
        load_height: 1.2,
        created_at: now,
        entered_at: now,
        front: 3.0,
        wrapped: false,
    }
}
