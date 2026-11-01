//! The 3D run of a layout (A07 §3–4, ADR-0016, phase R6 of A19; the reference's `physics-flow.ts`). The layout is read
//! exactly as in the 2D run (`build_flow_network`); every object becomes bodies in one Rapier world and the plant advances
//! in fixed steps of 1/120 s:
//!   1. every 10 ms the controller's rules run on the current signals (A08);
//!   2. the plant logic settles on the measured pallet positions: crossings complete, stations move on, sources place
//!      pallets, front pallets at their decision point are taken downstream or hold their deck;
//!   3. decks ramp their speed and turn their rollers or move their surface, platforms follow their vertical profile;
//!   4. the engine steps; poses are read back and the photocells cast their beams.
//!
//! Arrivals, removal, sensor timing and the rules reuse the 2D logic, so both runs differ only in how pallets move: here
//! by friction and contact.

pub mod deck;
pub mod pallet;
pub mod sensor;
pub mod station;

use std::collections::{BTreeMap, HashSet};

use intracraft_control::{Signals, evaluate_rules};
use intracraft_des::flow::network::{Base, FlowIssue, FlowNode, Kind, LiftPort, LineData, Side};
use intracraft_des::flow::parts::{Photoeye, Sink, Source};
use intracraft_des::flow::sim::Pallet;
use intracraft_des::flow::{Statistics, build_flow_network};
use intracraft_des::rng::Pcg32;
use intracraft_des::time::{SimTime, US_PER_S, from_ms};
use intracraft_library::Library;
use intracraft_library::definition::{Definition, Value};
use intracraft_library::layout::{Layout, Rule};
use intracraft_library::params::{Values, defaults, instance_values, num};
use intracraft_library::snap::derive_connections;
use rapier3d::prelude::RigidBodyHandle;

use crate::bench::{Bench, DT, Motion, Part, Shape};
use crate::frame::Frame;
use deck::{Deck, DeckSpec, Drive, Exit, Pallets, Status};
use pallet::{WorldPallet, pallet_parts};
use sensor::{Sensor3D, SensorEdge};
use station::Station3D;

/// The controller runs every 10 ms (A07 §4).
const CONTROL_PERIOD: SimTime = 10_000;
/// Tolerance of measured positions, m.
const POS_EPS: f64 = 5e-4;
const SPEED_EPS: f64 = 1e-6;
/// A pallet whose underside drops this far below its deck has fallen off, m.
const FALL_DEPTH: f64 = 0.25;
const SINK_BED_LENGTH: f64 = 1.5;
const BED_THICKNESS: f64 = 0.03;
/// A sink's bed lies this far below the conveying surface, m: a pallet resting on the rollers at the solver's contact
/// depth catches the edge of a bed exactly level with them and loses about 40 mm; this way it steps onto it (ADR-0024).
const BED_DROP: f64 = 0.002;
const STRAND_HEIGHT: f64 = 0.03;
/// Lateral strand positions keep this far from the deck's edges, m (`chainLayout` of the reference's geometry).
const CHAIN_EDGE_INSET: f64 = 0.1;
/// A new pallet starts this far above the conveying surface, so it settles without overlapping, m.
const SPAWN_LIFT: f64 = 0.002;
const FLOOR_HALF: [f64; 3] = [250.0, 0.05, 250.0];

struct Source3D {
    logic: Source,
    next_arrival: Option<SimTime>,
}

/// A sink as the 3D run sees it: the 2D removal logic, and a receiving bed (a moving surface) per deck feeding it.
struct Sink3D {
    logic: Sink,
    beds: Vec<(usize, usize)>,
}

struct Write {
    due: SimTime,
    signal: String,
    value: bool,
}

/// A pallet as a viewer draws it and the tests read it.
#[derive(Clone, Debug, PartialEq)]
pub struct PalletPose {
    pub id: u64,
    pub carrier: String,
    /// Its front along its carrier's travel (m) and its speed along it (m/s).
    pub front: f64,
    pub speed: f64,
    /// The centre of its underside and its turn about +y.
    pub centre: [f64; 3],
    pub yaw: f64,
    /// Along its own x, its height with the load, across, m.
    pub size: [f64; 3],
}

pub struct PhysicsFlow {
    pub bench: Bench,
    pub issues: Vec<FlowIssue>,
    decks: Vec<Deck>,
    stations: Vec<Station3D>,
    sources: Vec<Source3D>,
    sinks: Vec<Sink3D>,
    sensors: Vec<Sensor3D>,
    pallets: Pallets,
    bodies: HashSet<RigidBodyHandle>,
    rules: Vec<Rule>,
    signals: Signals,
    evaluated: Signals,
    writes: Vec<Write>,
    next_control: SimTime,
    groups: Vec<Vec<usize>>,
    group_of: Vec<usize>,
    edges: Vec<SensorEdge>,
    fallen: u64,
    next_pallet_id: u64,
}

impl PhysicsFlow {
    /// The world of a layout as a project holds it (its connections come from the geometry, as in the editor).
    pub fn new(layout: &Layout, library: &Library, seed: u64) -> Result<PhysicsFlow, String> {
        let mut connected = layout.clone();
        connected.connections = derive_connections(layout, library)?;
        let network = build_flow_network(&connected, library);
        let mut f = PhysicsFlow {
            bench: Bench::new(),
            issues: network.issues,
            decks: Vec::new(),
            stations: Vec::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
            sensors: Vec::new(),
            pallets: Pallets::new(),
            bodies: HashSet::new(),
            rules: network.rules,
            signals: Signals::new(),
            evaluated: Signals::new(),
            writes: Vec::new(),
            next_control: 0,
            groups: Vec::new(),
            group_of: Vec::new(),
            edges: Vec::new(),
            fallen: 0,
            next_pallet_id: 1,
        };
        f.bench.use_pairs(&library.reference.contact_pairs);
        let floor = Part { shape: Shape::Cuboid(FLOOR_HALF), offset: [0.0; 3], material: "concrete".into(), friction: None, mass: 0.0 };
        f.bench.body(Motion::Fixed, [0.0, -FLOOR_HALF[1], 0.0], 0.0, 0.0, &[floor]);
        for (index, node) in network.nodes.iter().enumerate() {
            let b = &node.base;
            let Some(definition) = layout.instance(&b.id).and_then(|i| library.get(&i.definition_id).map(|d| (i, d))) else {
                continue;
            };
            let (instance, definition) = definition;
            let p = instance_values(definition, &instance.params);
            match &node.kind {
                Kind::Line(data) => f.add_line(b, data, &p, definition),
                Kind::Lift(data) => {
                    let kind =
                        station::Kind::Lift { up_speed: data.up_speed, down_speed: data.down_speed, vertical_accel: data.vertical_accel };
                    let entry = data.entry.clone();
                    f.add_station(b, (data.length, data.nominal_speed, data.accel), &p, kind, entry, data.exit_ports.clone());
                }
                Kind::Process(data) => {
                    let kind = station::Kind::Process { data: (**data).clone(), rng: Pcg32::new(seed, index as u64 + 1) };
                    let port = |id: &str, at_end| LiftPort { port_id: id.into(), at_end, height: b.surface_height };
                    let ports = (port("IN", false), vec![port("OUT", true)]);
                    f.add_station(b, (data.length, data.nominal_speed, data.accel), &p, kind, ports.0, ports.1);
                }
                Kind::Source(data) => f.sources.push(Source3D {
                    logic: Source::new(&b.id, (**data).clone(), b.downstream.clone(), b.downstream_port.clone(), seed, index as u64 + 1),
                    next_arrival: None,
                }),
                Kind::Sink(data) => f.sinks.push(Sink3D { logic: Sink::new(&b.id, data.clone()), beds: Vec::new() }),
                Kind::Sensor(_) => {}
                Kind::Turntable(_) | Kind::Transfer(_) | Kind::Gravity(_) | Kind::Inert { .. } => f.issues.push(FlowIssue {
                    instance_id: b.id.clone(),
                    rule: "behaviorNotAvailable3D",
                    message: format!(
                        "{}: has no 3D behaviour yet; pallets stop in front of it",
                        if b.name.is_empty() { &b.id } else { &b.name }
                    ),
                }),
            }
        }
        for node in &network.nodes {
            if let Kind::Sensor(data) = &node.kind
                && let Some(host) = f.decks.iter().find(|d| d.station.is_none() && d.spec.id == data.host)
            {
                f.sensors.push(Sensor3D::new(Photoeye::new(&node.base.id, data.clone()), &host.frame));
            }
        }
        f.connect(&network.nodes);
        for d in f.decks.iter().filter(|d| d.station.is_none()) {
            f.signals.insert(format!("{}.run", d.spec.id), true);
        }
        for s in &f.sources {
            f.signals.insert(format!("{}.enable", s.logic.id), true);
        }
        f.update_outputs();
        f.evaluated = f.signals.clone();
        for k in 0..f.sources.len() {
            if f.sources[k].logic.active() && f.entry_of(k).is_some() {
                f.sources[k].next_arrival = Some(0);
            }
        }
        f.regroup();
        Ok(f)
    }

    /// The run's time, µs: whole steps.
    pub fn now(&self) -> SimTime {
        (self.bench.time() * US_PER_S as f64).round() as SimTime
    }

    /// Runs whole steps until the time reaches `time` µs.
    pub fn run_until(&mut self, time: SimTime) {
        while self.now() < time {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let now = self.now();
        let t = now as f64 / US_PER_S as f64;
        self.control(now);
        self.settle(now);
        self.regroup();
        for g in 0..self.groups.len() {
            self.plan_group(g);
        }
        for d in &self.decks {
            d.apply_drive(&mut self.bench);
        }
        for s in &self.sinks {
            for &(deck, bed) in &s.beds {
                let d = &self.decks[deck];
                let feeding = matches!(d.exit, Some(Exit::Sink { bed: b, .. }) if b == bed);
                self.bench.set_surface_velocity(bed, if feeding { d.surface_velocity() } else { [0.0; 3] });
            }
        }
        for s in &self.stations {
            s.drive_vertical(&mut self.bench, t);
        }
        self.bench.step();
        let after = self.now();
        for s in &mut self.stations {
            s.sync_height(&self.bench, after as f64 / US_PER_S as f64, &mut self.decks[s.deck]);
        }
        self.observe(after);
        let stamp = now + (after - now + 1) / 2;
        for s in &mut self.sensors {
            s.sense(&self.bench, &self.bodies, stamp);
            if let Some(edge) = s.flush(after) {
                self.edges.push(edge);
            }
        }
        self.accumulate((after - now) as f64 / US_PER_S as f64);
    }

    pub fn pallets(&self) -> Vec<PalletPose> {
        self.pallets
            .values()
            .map(|p| {
                let e = self.carrier_deck(&p.carrier).map(|d| self.decks[d].extent(p));
                PalletPose {
                    id: p.pallet.id,
                    carrier: p.carrier.clone(),
                    front: e.map_or(0.0, |e| e.front),
                    speed: e.map_or(0.0, |e| e.speed),
                    centre: p.centre,
                    yaw: p.yaw(),
                    size: [p.pallet.length, p.pallet.height + p.pallet.load_height, p.pallet.width],
                }
            })
            .collect()
    }

    /// Each object's state: `empty`, `working`, `blocked`, `stopped`; `idle`/`waiting` (sources), `idle`/`busy` (sinks),
    /// `detecting`/`clear` (sensors); a station's `lifting`, `lowering` or `processing`, else its deck's.
    pub fn status(&self) -> BTreeMap<String, &'static str> {
        let mut out = BTreeMap::new();
        for d in self.decks.iter().filter(|d| d.station.is_none()) {
            out.insert(d.spec.id.clone(), d.status.name());
        }
        for s in &self.sources {
            out.insert(s.logic.id.clone(), if s.logic.due_since.is_none() { "idle" } else { "waiting" });
        }
        for s in &self.sinks {
            out.insert(s.logic.id.clone(), if self.now() < s.logic.busy_until { "busy" } else { "idle" });
        }
        for s in &self.sensors {
            out.insert(s.logic.id.clone(), if s.logic.detecting { "detecting" } else { "clear" });
        }
        for s in &self.stations {
            out.insert(s.id.clone(), s.busy_status().unwrap_or(self.decks[s.deck].status.name()));
        }
        out
    }

    /// The deck height of every lift station, m above its base.
    pub fn heights(&self) -> BTreeMap<String, f64> {
        self.stations.iter().map(|s| (s.id.clone(), s.height)).collect()
    }

    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    /// Statistics per instance, keyed as in the 2D run.
    pub fn statistics(&self) -> Statistics {
        let hours = self.bench.time() / 3600.0;
        let mut out = Statistics::new();
        for d in self.decks.iter().filter(|d| d.station.is_none()) {
            let s = &d.stats;
            let total = s.empty + s.working + s.blocked + s.stopped;
            let share = |v: f64| if total > 0.0 { v / total } else { 0.0 };
            let transit = if s.processed > 0 { s.transit_sum as f64 / s.processed as f64 / US_PER_S as f64 } else { 0.0 };
            out.insert(
                d.spec.id.clone(),
                BTreeMap::from([
                    ("utilization", share(s.working)),
                    ("blockedTime", s.blocked),
                    ("emptyTime", s.empty),
                    ("palletsProcessed", s.processed as f64),
                    ("throughputPerHour", if hours > 0.0 { s.processed as f64 / hours } else { 0.0 }),
                    ("meanTransitTime", transit),
                    ("meanOccupancy", share(s.occupancy)),
                ]),
            );
        }
        for s in &self.sources {
            out.insert(s.logic.id.clone(), s.logic.statistics(self.now()));
        }
        for s in &self.stations {
            out.insert(s.id.clone(), s.statistics());
        }
        for s in &self.sensors {
            out.insert(s.logic.id.clone(), s.logic.statistics());
        }
        for s in &self.sinks {
            out.insert(s.logic.id.clone(), s.logic.statistics(hours));
        }
        out
    }

    /// Sensor output changes since the last call, with their exact times.
    pub fn take_sensor_edges(&mut self) -> Vec<SensorEdge> {
        std::mem::take(&mut self.edges)
    }

    pub fn fallen_pallets(&self) -> u64 {
        self.fallen
    }

    // ── Building ─────────────────────────────────────────────────────────────────────────────────────────────────────

    fn add_line(&mut self, b: &Base, data: &LineData, p: &Values, definition: &Definition) {
        let spec = DeckSpec {
            id: b.id.clone(),
            length: data.length,
            usable_width: num(p, "usableWidth", 1.0),
            nominal_speed: data.nominal_speed,
            accel: data.accel,
            decel: data.decel,
            min_gap: data.min_gap,
        };
        let frame = Frame { origin: [b.position.x, b.position.y + b.surface_height, b.position.z], yaw: b.yaw, slope: 0.0 };
        let drive = if definition.behaviors.des == "des.chain_conveyor" {
            let friction = own_friction(p, definition, "chainMu");
            let strand = num(p, "chainSupportWidth", 0.04);
            let parts: Vec<Part> = chain_layout(spec.usable_width, num(p, "chainCount", 3.0))
                .into_iter()
                .map(|z| Part {
                    shape: Shape::Cuboid([spec.length / 2.0, STRAND_HEIGHT / 2.0, strand / 2.0]),
                    offset: [spec.length / 2.0, -STRAND_HEIGHT / 2.0, z],
                    material: "steel-chain".into(),
                    friction,
                    mass: 0.0,
                })
                .collect();
            let (_, colliders) = self.bench.body(Motion::Fixed, frame.origin, frame.yaw, 0.0, &parts);
            Drive::Surface(self.bench.surface(&colliders))
        } else {
            let friction = own_friction(p, definition, "rollerMu");
            let diameter = num(p, "rollerDiameter", 0.089);
            let roller = Part {
                shape: Shape::Cylinder { radius: diameter / 2.0, half_length: spec.usable_width / 2.0 },
                offset: [0.0; 3],
                material: "steel-zinc".into(),
                friction,
                mass: 0.0,
            };
            let bodies = roller_positions(spec.length, num(p, "rollerPitch", 0.2063))
                .into_iter()
                .map(|x| {
                    self.bench
                        .body(Motion::Kinematic, frame.to_world([x, -diameter / 2.0, 0.0]), frame.yaw, 0.0, std::slice::from_ref(&roller))
                        .0
                })
                .collect();
            Drive::Rollers { bodies, radius: diameter / 2.0 }
        };
        self.decks.push(Deck::new(spec, frame, drive, &b.id, None));
    }

    /// Lift tables, the elevator and the process station: one deck on a kinematic platform (ADR-0016).
    fn add_station(
        &mut self,
        b: &Base,
        (length, speed, accel): (f64, f64, f64),
        p: &Values,
        kind: station::Kind,
        entry: LiftPort,
        exit_ports: Vec<LiftPort>,
    ) {
        let spec = DeckSpec {
            id: b.id.clone(),
            length,
            usable_width: num(p, "usableWidth", 1.0),
            nominal_speed: speed,
            accel,
            decel: accel,
            min_gap: 0.0,
        };
        let frame = Frame { origin: [b.position.x, b.position.y + entry.height, b.position.z], yaw: b.yaw, slope: 0.0 };
        let parts: Vec<Part> = if p.get("deckType").and_then(Value::text) == Some("chains") {
            let strand = num(p, "chainSupportWidth", 0.04);
            chain_layout(spec.usable_width, num(p, "chainCount", 3.0))
                .into_iter()
                .map(|z| Part {
                    shape: Shape::Cuboid([length / 2.0, STRAND_HEIGHT / 2.0, strand / 2.0]),
                    offset: [0.0, -STRAND_HEIGHT / 2.0, z],
                    material: "steel-chain".into(),
                    friction: None,
                    mass: 0.0,
                })
                .collect()
        } else {
            let diameter = num(p, "rollerDiameter", 0.089);
            roller_positions(length, num(p, "rollerPitch", 0.2063))
                .into_iter()
                .map(|x| Part {
                    shape: Shape::Cylinder { radius: diameter / 2.0, half_length: spec.usable_width / 2.0 },
                    offset: [x - length / 2.0, -diameter / 2.0, 0.0],
                    material: "steel-zinc".into(),
                    friction: None,
                    mass: 0.0,
                })
                .collect()
        };
        let (platform, colliders) = self.bench.body(Motion::Kinematic, frame.to_world([length / 2.0, 0.0, 0.0]), frame.yaw, 0.0, &parts);
        let surface = self.bench.surface(&colliders);
        let station = Station3D::new(&b.id, self.decks.len(), platform, b.position.y, kind, entry, exit_ports);
        let mut deck = Deck::new(spec, frame, Drive::Surface(surface), &b.id, Some(self.stations.len()));
        deck.direction = station.entry_direction();
        self.decks.push(deck);
        self.stations.push(station);
    }

    /// Wires every object's output to what it feeds, and lays a receiving bed in front of each sink.
    fn connect(&mut self, nodes: &[FlowNode]) {
        for node in nodes {
            let b = &node.base;
            match &node.kind {
                Kind::Line(_) => {
                    let Some(deck) = self.decks.iter().position(|d| d.station.is_none() && d.spec.id == b.id) else { continue };
                    if let Some(target) = &b.downstream {
                        let to = (target.as_str(), b.downstream_port.clone(), b.downstream_gap);
                        self.decks[deck].exit = self.exit_to(deck, to, true, b.position.y + b.surface_height);
                    }
                }
                Kind::Lift(data) => {
                    let Some(k) = self.stations.iter().position(|s| s.id == b.id) else { continue };
                    for port in &data.exit_ports {
                        let Some(link) = data.exits.get(&port.port_id) else { continue };
                        let to = (link.target.as_str(), Some(link.port.clone()), link.gap);
                        if let Some(exit) = self.exit_to(self.stations[k].deck, to, port.at_end, b.position.y + port.height) {
                            self.stations[k].exits.insert(port.port_id.clone(), exit);
                        }
                    }
                }
                Kind::Process(_) => {
                    let Some(k) = self.stations.iter().position(|s| s.id == b.id) else { continue };
                    if let Some(target) = &b.downstream {
                        let to = (target.as_str(), b.downstream_port.clone(), b.downstream_gap);
                        if let Some(exit) = self.exit_to(self.stations[k].deck, to, true, b.position.y + b.surface_height) {
                            self.stations[k].exits.insert("OUT".into(), exit);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// The exit from `deck` to (target, its port, the gap). A sink gets a receiving bed beyond the deck's end, at the
    /// port's height `y` (world), on the `at_end` side, moving with the deck while it hands a pallet over.
    fn exit_to(&mut self, deck: usize, (target, port, gap): (&str, Option<String>, f64), at_end: bool, y: f64) -> Option<Exit> {
        if let Some(next) = self.entry_deck_of(target, port.as_deref()) {
            return Some(Exit::Deck { deck: next, port, gap });
        }
        let sink = self.sinks.iter().position(|s| s.logic.id == target)?;
        let d = &self.decks[deck];
        let frame = Frame { origin: [d.frame.origin[0], y, d.frame.origin[2]], ..d.frame };
        let x = if at_end { d.spec.length + gap + SINK_BED_LENGTH / 2.0 } else { -gap - SINK_BED_LENGTH / 2.0 };
        let bed = Part {
            shape: Shape::Cuboid([SINK_BED_LENGTH / 2.0, BED_THICKNESS / 2.0, d.spec.usable_width / 2.0]),
            offset: [0.0; 3],
            material: "steel-zinc".into(),
            friction: None,
            mass: 0.0,
        };
        let (_, colliders) =
            self.bench.body(Motion::Fixed, frame.to_world([x, -BED_DROP - BED_THICKNESS / 2.0, 0.0]), frame.yaw, 0.0, &[bed]);
        let surface = self.bench.surface(&colliders);
        self.sinks[sink].beds.push((deck, surface));
        Some(Exit::Sink { sink, bed: surface })
    }

    /// The deck receiving what reaches `target` at `port`: a conveyor, or a station's entry.
    fn entry_deck_of(&self, target: &str, port: Option<&str>) -> Option<usize> {
        self.decks
            .iter()
            .position(|d| d.station.is_none() && d.spec.id == target)
            .or_else(|| self.stations.iter().find(|s| s.id == target).and_then(|s| s.entry_deck(port)))
    }

    /// The deck (and its station) receiving what source `k` creates.
    fn entry_of(&self, k: usize) -> Option<(usize, Option<usize>)> {
        let s = &self.sources[k].logic;
        let deck = self.entry_deck_of(s.downstream.as_deref()?, s.downstream_port.as_deref())?;
        Some((deck, self.decks[deck].station))
    }

    fn carrier_deck(&self, carrier: &str) -> Option<usize> {
        self.decks.iter().position(|d| d.carrier == carrier)
    }

    // ── Plant logic ──────────────────────────────────────────────────────────────────────────────────────────────────

    fn settle(&mut self, now: SimTime) {
        self.complete_crossings(now);
        for s in &mut self.stations {
            s.step(now, &mut self.decks[s.deck], &self.pallets);
        }
        self.feed_sources(now);
        self.decide(now);
    }

    /// Pallets whose tail cleared their deck's end now belong to the next object.
    fn complete_crossings(&mut self, now: SimTime) {
        for i in 0..self.decks.len() {
            let (Some(id), Some(exit)) = (self.decks[i].outgoing, self.decks[i].exit.clone()) else { continue };
            let Some(p) = self.pallets.get(&id) else { continue };
            if self.decks[i].extent(p).rear < self.decks[i].spec.length {
                continue;
            }
            let entered = p.pallet.entered_at;
            self.decks[i].release(id, entered, now);
            match exit {
                Exit::Deck { deck, .. } => {
                    let carrier = self.decks[deck].carrier.clone();
                    if let Some(p) = self.pallets.get_mut(&id) {
                        p.carrier = carrier;
                        p.pallet.entered_at = now;
                    }
                    self.decks[deck].pallets.push(id);
                    self.decks[deck].incoming = None;
                }
                Exit::Sink { sink, .. } => {
                    if let Some(p) = self.remove_pallet(id) {
                        self.sinks[sink].logic.receive(&p.pallet, now);
                    }
                }
            }
        }
    }

    fn feed_sources(&mut self, now: SimTime) {
        for k in 0..self.sources.len() {
            let s = &mut self.sources[k];
            if s.logic.due_since.is_none() && s.next_arrival.is_some_and(|a| a <= now) {
                s.logic.due_since = s.next_arrival.take();
            }
            let Some((deck, station)) = self.entry_of(k) else { continue };
            let s = &self.sources[k].logic;
            if s.due_since.is_none() || self.signals.get(&format!("{}.enable", s.id)) == Some(&false) {
                continue;
            }
            let length = s.along_length();
            let port = s.downstream_port.clone();
            let accepts = station.is_none_or(|st| self.stations[st].can_receive(port.as_deref(), &self.decks[deck]));
            if accepts && self.can_enter(deck, length + self.decks[deck].spec.min_gap) {
                if let Some(st) = station {
                    self.stations[st].receive(length, now, &mut self.decks[deck]);
                }
                self.spawn(k, deck, length, now);
                self.sources[k].logic.placed(now);
            } else if !self.sources[k].logic.data.block_if_occupied {
                self.sources[k].logic.reject();
            } else {
                continue;
            }
            if self.sources[k].logic.active() {
                let delay = self.sources[k].logic.next_delay();
                self.sources[k].next_arrival = Some(now + delay);
            }
        }
    }

    /// A new pallet at rest on the entry deck, its rear at the entry end, aligned with the travel (S09 §3D).
    fn spawn(&mut self, k: usize, deck: usize, length: f64, now: SimTime) {
        let data = &self.sources[k].logic.data;
        let item = &data.item;
        let pallet = Pallet {
            id: self.next_pallet_id,
            type_id: data.pallet_type.id.clone(),
            along: data.orientation,
            length,
            width: if data.orientation == Side::Long { item.short } else { item.long },
            height: item.pallet_height,
            load_mass: data.load_mass,
            load_height: item.load_height,
            created_at: now,
            entered_at: now,
            front: length,
        };
        let d = &self.decks[deck];
        let x = if d.direction > 0.0 { length / 2.0 } else { d.spec.length - length / 2.0 };
        let centre = d.frame.to_world([x, SPAWN_LIFT, 0.0]);
        let yaw = d.frame.yaw + if d.direction < 0.0 { std::f64::consts::PI } else { 0.0 };
        let parts = pallet_parts(&pallet, &data.pallet_type);
        let (body, _) = self.bench.body(Motion::Dynamic, centre, yaw, 0.0, &parts);
        // At rest on the deck: it moves with the conveying surface from the start, as in 2D.
        let velocity = d.surface_velocity();
        self.bench.set_velocity(body, velocity, [0.0; 3]);
        let (x_axis, z_axis) = self.bench.axes(body);
        let id = pallet.id;
        self.pallets.insert(id, WorldPallet { pallet, body, carrier: d.carrier.clone(), centre, x_axis, z_axis, velocity });
        self.bodies.insert(body);
        self.decks[deck].pallets.push(id);
        self.next_pallet_id += 1;
    }

    /// Front pallets at their decision point are either taken downstream or make their deck hold.
    fn decide(&mut self, now: SimTime) {
        for i in 0..self.decks.len() {
            if self.decks[i].outgoing.is_some() {
                continue;
            }
            let Some(lead) = self.decks[i].lead(&self.pallets) else {
                self.decks[i].holding = false;
                continue;
            };
            let front = self.pallets.get(&lead).map_or(0.0, |p| self.decks[i].extent(p).front);
            if !self.decks[i].holding && front < self.decision_point(i) - POS_EPS {
                continue;
            }
            let accepted = self.reserve(i, lead, now);
            self.decks[i].holding = !accepted;
            if accepted {
                self.regroup();
            }
        }
    }

    fn reserve(&mut self, i: usize, lead: u64, now: SimTime) -> bool {
        let Some(exit) = self.decks[i].exit.clone() else { return false };
        match exit {
            Exit::Sink { sink, .. } => {
                if !self.sinks[sink].logic.accepts(now) {
                    return false;
                }
                self.sinks[sink].logic.incoming += 1;
                self.decks[i].outgoing = Some(lead);
                true
            }
            Exit::Deck { deck: target, port, gap } => {
                let Some(p) = self.pallets.get(&lead) else { return false };
                let length = p.pallet.length;
                let travel = self.decks[i].spec.length - self.decks[i].extent(p).rear + gap;
                if !self.can_enter(target, 0.0) || !self.has_room_for(target, travel) {
                    return false;
                }
                // A conveyor switched off can't pull a pallet across.
                if !self.group(target).iter().all(|&d| self.runs(d)) {
                    return false;
                }
                if let Some(st) = self.decks[target].station {
                    if !self.stations[st].can_receive(port.as_deref(), &self.decks[target]) {
                        return false;
                    }
                    self.stations[st].receive(length, now, &mut self.decks[target]);
                }
                self.decks[target].incoming = Some(lead);
                self.decks[i].outgoing = Some(lead);
                true
            }
        }
    }

    /// A deck can take a new pallet when nothing else is entering, it isn't held and its last pallet has moved
    /// `clearance` beyond the entry (ADR-0007 §5).
    fn can_enter(&self, deck: usize, clearance: f64) -> bool {
        if self.decks[deck].incoming.is_some() {
            return false;
        }
        let Some(rear) = self.decks[deck].rear_tail(&self.pallets) else { return true };
        if self.group(deck).iter().any(|&d| self.decks[d].holding) {
            return false;
        }
        rear >= clearance - POS_EPS
    }

    /// A deck takes a pallet only if it gets wholly aboard before the deck's own lead pallet reaches the end.
    fn has_room_for(&self, target: usize, travel: f64) -> bool {
        let t = &self.decks[target];
        t.lead(&self.pallets).and_then(|id| self.pallets.get(&id)).is_none_or(|p| t.extent(p).front + travel <= t.spec.length + POS_EPS)
    }

    fn decision_point(&self, deck: usize) -> f64 {
        let (max_speed, _, decel) = self.limits(&self.group(deck));
        self.decks[deck].end - max_speed * max_speed / (2.0 * decel)
    }

    /// The controller hasn't switched the deck off (`<id>.run`; a station's deck always runs).
    fn runs(&self, deck: usize) -> bool {
        let d = &self.decks[deck];
        d.station.is_some() || self.signals.get(&format!("{}.run", d.spec.id)) != Some(&false)
    }

    fn group(&self, deck: usize) -> Vec<usize> {
        self.group_of.get(deck).and_then(|&g| self.groups.get(g)).cloned().unwrap_or_else(|| vec![deck])
    }

    /// The speed and ramps of a group: the most restrictive member decides.
    fn limits(&self, group: &[usize]) -> (f64, f64, f64) {
        let min = |f: fn(&DeckSpec) -> f64| group.iter().map(|&d| f(&self.decks[d].spec)).fold(f64::INFINITY, f64::min);
        (min(|s| s.nominal_speed), min(|s| s.accel), min(|s| s.decel))
    }

    /// Decks linked by a pallet crossing from one to the next move as one (ADR-0007 §3).
    fn regroup(&mut self) {
        let n = self.decks.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn root(parent: &[usize], mut i: usize) -> usize {
            while parent[i] != i {
                i = parent[i];
            }
            i
        }
        for i in 0..n {
            if let (Some(_), Some(Exit::Deck { deck, .. })) = (self.decks[i].outgoing, &self.decks[i].exit) {
                let (a, b) = (root(&parent, *deck), root(&parent, i));
                parent[a] = b;
            }
        }
        let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for i in 0..n {
            by_root.entry(root(&parent, i)).or_default().push(i);
        }
        self.groups = by_root.into_values().collect();
        self.group_of = vec![0; n];
        for (g, members) in self.groups.iter().enumerate() {
            for &d in members {
                self.group_of[d] = g;
            }
        }
    }

    /// A group's shared speed for the next step: ramp up, brake to stop at the end, or stop (ADR-0007 §4).
    fn plan_group(&mut self, g: usize) {
        let group = self.groups[g].clone();
        let carriers: Vec<usize> = group.iter().copied().filter(|&d| !self.decks[d].pallets.is_empty()).collect();
        let held = group.iter().any(|&d| self.decks[d].holding);
        let switched_off = !group.iter().all(|&d| self.runs(d));
        for &d in &group {
            self.decks[d].status = if self.decks[d].pallets.is_empty() {
                Status::Empty
            } else if held {
                Status::Blocked
            } else if switched_off {
                Status::Stopped
            } else {
                Status::Working
            };
        }
        let active = !carriers.is_empty() || group.iter().any(|&d| self.decks[d].incoming.is_some() || self.decks[d].outgoing.is_some());
        if !active {
            for &d in &group {
                (self.decks[d].speed, self.decks[d].rate) = (0.0, 0.0);
            }
            return;
        }
        let (max_speed, accel, decel) = self.limits(&group);
        let moving = if carriers.is_empty() { &group } else { &carriers };
        let mut speed = moving.iter().map(|&d| self.decks[d].speed).fold(f64::INFINITY, f64::min);
        let rate = if held {
            let room = group
                .iter()
                .filter(|&&d| self.decks[d].holding)
                .map(|&d| {
                    let deck = &self.decks[d];
                    deck.lead(&self.pallets).and_then(|id| self.pallets.get(&id)).map_or(f64::INFINITY, |p| deck.end - deck.extent(p).front)
                })
                .fold(f64::INFINITY, f64::min);
            if speed <= SPEED_EPS || room <= POS_EPS {
                speed = 0.0;
                0.0
            } else {
                -decel.max(speed * speed / (2.0 * room))
            }
        } else if switched_off {
            if speed <= SPEED_EPS { 0.0 } else { -decel }
        } else if speed < max_speed - SPEED_EPS {
            accel
        } else if speed > max_speed + SPEED_EPS {
            -decel
        } else {
            speed = max_speed;
            0.0
        };
        let mut next = speed + rate * DT;
        if rate > 0.0 {
            next = next.min(max_speed);
        } else if rate < 0.0 {
            next = next.max(if held || switched_off { 0.0 } else { max_speed });
        }
        for &d in &group {
            (self.decks[d].speed, self.decks[d].rate) = (next, rate);
        }
    }

    // ── Controller (A08) ─────────────────────────────────────────────────────────────────────────────────────────────

    /// Applies the delayed writes that are due and, on each 10 ms boundary, evaluates the rules on changed signals.
    fn control(&mut self, now: SimTime) {
        let (due, waiting): (Vec<Write>, Vec<Write>) = std::mem::take(&mut self.writes).into_iter().partition(|w| w.due <= now);
        self.writes = waiting;
        for w in due {
            self.signals.insert(w.signal, w.value);
        }
        if now < self.next_control {
            return;
        }
        while self.next_control <= now {
            self.next_control += CONTROL_PERIOD;
        }
        if self.rules.is_empty() {
            return;
        }
        self.update_outputs();
        if self.signals.iter().all(|(k, v)| self.evaluated.get(k) == Some(v)) {
            return;
        }
        let writes = evaluate_rules(&self.rules, &self.evaluated, &self.signals);
        self.evaluated = self.signals.clone();
        for w in writes {
            if w.delay_ms > 0.0 {
                self.writes.push(Write { due: now + from_ms(w.delay_ms), signal: w.signal, value: w.value });
            } else {
                self.signals.insert(w.signal, w.value);
            }
        }
    }

    fn update_outputs(&mut self) {
        for d in self.decks.iter().filter(|d| d.station.is_none()) {
            self.signals.insert(format!("{}.running", d.spec.id), d.speed > 0.0 || d.rate > 0.0);
            self.signals.insert(format!("{}.occupied", d.spec.id), !d.pallets.is_empty());
        }
        for s in &self.sensors {
            self.signals.insert(s.logic.signal.clone(), s.logic.signal_value());
        }
    }

    // ── Observation and bookkeeping ──────────────────────────────────────────────────────────────────────────────────

    /// Reads every pallet's pose and velocity; pallets that dropped off their deck are taken out.
    fn observe(&mut self, now: SimTime) {
        for p in self.pallets.values_mut() {
            p.centre = self.bench.position(p.body);
            (p.x_axis, p.z_axis) = self.bench.axes(p.body);
            p.velocity = self.bench.velocity(p.body);
        }
        let fallen: Vec<(u64, String)> = self
            .pallets
            .values()
            .filter(|p| self.carrier_deck(&p.carrier).is_some_and(|d| self.decks[d].extent(p).lift <= -FALL_DEPTH))
            .map(|p| (p.pallet.id, p.carrier.clone()))
            .collect();
        for (id, carrier) in fallen {
            self.fallen += 1;
            self.issues.push(FlowIssue {
                instance_id: carrier.clone(),
                rule: "palletFell",
                message: format!("pallet {id} fell off {carrier} at {:.2} s", now as f64 / US_PER_S as f64),
            });
            for d in &mut self.decks {
                d.pallets.retain(|&p| p != id);
                if d.incoming == Some(id) {
                    d.incoming = None;
                }
                if d.outgoing == Some(id) {
                    d.outgoing = None;
                }
            }
            self.remove_pallet(id);
        }
    }

    fn remove_pallet(&mut self, id: u64) -> Option<WorldPallet> {
        let p = self.pallets.remove(&id)?;
        self.bench.remove(p.body);
        self.bodies.remove(&p.body);
        Some(p)
    }

    fn accumulate(&mut self, seconds: f64) {
        for d in self.decks.iter_mut().filter(|d| d.station.is_none()) {
            d.accumulate(seconds);
        }
        for s in &mut self.stations {
            s.accumulate(seconds, self.decks[s.deck].status);
        }
        for s in &mut self.sensors {
            s.logic.accumulate(seconds);
        }
    }
}

/// A friction the instance changed from the definition's default replaces the pair table for this conveyor (S16).
fn own_friction(p: &Values, definition: &Definition, key: &str) -> Option<f64> {
    let value = p.get(key).and_then(Value::number)?;
    (defaults(definition).get(key).and_then(Value::number) != Some(value)).then_some(value)
}

/// Roller axes at the pitch, the first half a pitch from the entry (as the bench and the 3D model).
fn roller_positions(length: f64, pitch: f64) -> Vec<f64> {
    let count = (length / pitch + 1e-9).floor() as usize;
    (0..count).map(|i| pitch / 2.0 + i as f64 * pitch).collect()
}

/// The lateral positions of `count` chain strands across `width`, evenly between the edge insets.
fn chain_layout(width: f64, count: f64) -> Vec<f64> {
    let count = count.round().max(2.0) as usize;
    let span = (width - 2.0 * CHAIN_EDGE_INSET).max(0.0);
    (0..count).map(|i| -span / 2.0 + i as f64 * span / (count - 1) as f64).collect()
}

#[cfg(test)]
mod tests;
