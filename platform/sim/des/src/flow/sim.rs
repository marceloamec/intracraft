//! The 2D run of a layout (A06, ADR-0007): sources, conveyors without accumulation, sinks, photoelectric sensors and the
//! controller. A port of the reference's `flow-simulation.ts`, kept line by line so both give the same event times.
//!
//! Every event settles the plant at the current time: pallets whose tail cleared a conveyor end move on; waiting sources
//! place their pallet; a front pallet at its conveyor's decision point asks the next object to take it, and if it can't,
//! the conveyor holds and brakes so the pallet stops with its front at the end. Conveyors linked by a crossing pallet
//! form a group that moves as one, at the pace of its slowest member. Then one wake-up is scheduled at the earliest
//! predicted change.

use std::collections::{BTreeMap, BTreeSet};

use intracraft_control::{Signals, evaluate_rules};
use intracraft_library::layout::Rule;
use intracraft_library::reference::Reference;

use crate::flow::gravity::GravityLine;
use crate::flow::network::{FlowIssue, FlowNetwork, FlowNode, Kind, LineData, Side};
use crate::flow::parts::{Photoeye, Sink, Source};
use crate::flow::station::{LiftStation, ProcessStation, Station, Transfer, Turntable};
use crate::kinematics::{Segment, time_to_reach};
use crate::queue::Simulator;
use crate::time::{SimTime, US_PER_S, from_ms};

const MAX_SETTLE_PASSES: usize = 10_000;
/// Rule evaluations in a row at one instant before the controller is taken to oscillate.
const MAX_CONTROL_PASSES: usize = 100;
/// Position tolerance for "has reached", m.
pub const EPS: f64 = 1e-6;
/// Speed tolerance for "has reached the target speed", m/s.
pub const SPEED_EPS: f64 = 1e-6;

/// A pallet in the 2D run: its front edge along the conveyor that carries it (the one under its tail), valid at that
/// conveyor's last update.
#[derive(Clone, Debug, PartialEq)]
pub struct Pallet {
    pub id: u64,
    pub type_id: String,
    pub along: Side,
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub load_mass: f64,
    pub load_height: f64,
    pub created_at: SimTime,
    pub entered_at: SimTime,
    pub front: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStatus {
    Empty,
    Working,
    Blocked,
    Stopped,
}

#[derive(Clone, Debug, Default)]
struct LineStats {
    empty: f64,
    working: f64,
    blocked: f64,
    stopped: f64,
    occupancy: f64,
    processed: u64,
    transit_sum: SimTime,
}

/// A straight conveyor without accumulation, or a station's deck: its pallets share one motion.
pub(crate) struct Line {
    pub(crate) id: String,
    name: String,
    /// The instance its pallets are drawn on: itself, or the station a deck belongs to.
    pub(crate) carrier: String,
    /// A station's deck rather than a conveyor of the layout.
    deck: bool,
    data: LineData,
    pub(crate) downstream: Option<String>,
    pub(crate) downstream_port: Option<String>,
    pub(crate) gap: f64,
    /// Pallets on it, front first.
    pub(crate) pallets: Vec<Pallet>,
    speed: f64,
    accel: f64,
    /// When `speed` and the fronts hold, s.
    t0: f64,
    /// The front pallet, taken by the next object and crossing to it.
    outgoing: Option<u64>,
    /// A pallet taken from the previous conveyor and crossing onto this one.
    pub(crate) incoming: Option<u64>,
    /// Its front pallet must stop at the end: the next object can't take it.
    pub(crate) holding: bool,
    /// Where a held front pallet stops, m: the end, or the centre of a turntable's deck.
    pub(crate) end: f64,
    status: LineStatus,
    stats: LineStats,
}

/// A station deck: a short conveyor with one ramp for starting and stopping.
fn deck_data(length: f64, speed: f64, accel: f64, side: Side) -> LineData {
    LineData { length, nominal_speed: speed, accel, decel: accel, min_gap: 0.0, pallet_side_along_travel: side }
}

impl Line {
    fn deck(id: &str, name: &str, carrier: &str, data: LineData) -> Line {
        Line {
            id: id.to_string(),
            name: name.to_string(),
            carrier: carrier.to_string(),
            deck: true,
            end: data.length,
            data,
            downstream: None,
            downstream_port: None,
            gap: 0.0,
            pallets: Vec::new(),
            speed: 0.0,
            accel: 0.0,
            t0: 0.0,
            outgoing: None,
            incoming: None,
            holding: false,
            status: LineStatus::Empty,
            stats: LineStats::default(),
        }
    }

    pub(crate) fn length(&self) -> f64 {
        self.data.length
    }

    pub(crate) fn is_blocked(&self) -> bool {
        self.status == LineStatus::Blocked
    }

    fn rear_tail(&self) -> Option<f64> {
        self.pallets.last().map(|p| p.front - p.length)
    }

    fn advance_to(&mut self, t: f64) {
        let dt = t - self.t0;
        if dt <= 0.0 {
            return;
        }
        let moved = travel(self.speed, self.accel, dt);
        for p in &mut self.pallets {
            p.front += moved;
        }
        self.speed = (self.speed + self.accel * dt).max(0.0);
        self.t0 = t;
    }

    fn front_at(&self, p: &Pallet, t: f64) -> f64 {
        p.front + travel(self.speed, self.accel, t - self.t0)
    }

    fn speed_at(&self, t: f64) -> f64 {
        (self.speed + self.accel * (t - self.t0).max(0.0)).max(0.0)
    }

    fn accumulate(&mut self, seconds: f64) {
        match self.status {
            LineStatus::Empty => self.stats.empty += seconds,
            LineStatus::Working => self.stats.working += seconds,
            LineStatus::Blocked => self.stats.blocked += seconds,
            LineStatus::Stopped => self.stats.stopped += seconds,
        }
        self.stats.occupancy += self.pallets.len() as f64 * seconds;
    }
}

/// Distance covered in `dt` s from `speed` at constant `accel`, never backwards.
fn travel(speed: f64, accel: f64, dt: f64) -> f64 {
    if dt <= 0.0 {
        return 0.0;
    }
    let moving = if accel < 0.0 { dt.min(speed / -accel) } else { dt };
    speed * moving + 0.5 * accel * moving * moving
}

#[derive(Clone, Debug, PartialEq)]
enum Event {
    SourceArrival(usize),
    SinkFree,
    SensorOutput(usize),
    SignalWrite(String, bool),
    FlowMotion,
    StationDone(usize),
    GravityMotion(usize),
}

/// A pallet as a viewer draws it at the snapshot's time.
#[derive(Clone, Debug, PartialEq)]
pub struct PalletView {
    pub id: u64,
    pub type_id: String,
    /// The conveyor or station carrying it.
    pub carrier: String,
    /// Its front edge along the carrier's path, m from its start.
    pub front: f64,
    /// Where the carrier's path starts and which way it runs, in the carrier's own frame (m, rad): zero for a conveyor;
    /// on a turntable the deck's entry edge and heading, which turn with the platform.
    pub path_x: f64,
    pub path_z: f64,
    pub heading: f64,
    /// The fall per metre along the path (a gravity line): the pallet sinks and tilts with it.
    pub slope: f64,
    pub speed: f64,
    pub accel: f64,
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub load_height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub time: SimTime,
    pub pallets: Vec<PalletView>,
    /// `empty`, `working`, `blocked`, `stopped`, `idle`, `waiting`, `busy`, `detecting`, `clear`.
    pub status: BTreeMap<String, &'static str>,
    pub signals: Signals,
    /// The deck heading of every turntable (rad, its own frame), following the platform while it turns.
    pub angles: BTreeMap<String, f64>,
    /// The deck height of every lift table and elevator (m above its base), following the vertical move.
    pub heights: BTreeMap<String, f64>,
}

/// Statistics per instance id, keyed as the definition's `statistics` list says (times in s).
pub type Statistics = BTreeMap<String, BTreeMap<&'static str, f64>>;

pub struct FlowSimulation {
    pub issues: Vec<FlowIssue>,
    sim: Simulator<Event>,
    nodes: Vec<FlowNode>,
    lines: Vec<Line>,
    sources: Vec<Source>,
    sinks: Vec<Sink>,
    sensors: Vec<Photoeye>,
    stations: Vec<Station>,
    gravity: Vec<GravityLine>,
    /// The station each line is a deck of.
    deck_owner: Vec<Option<usize>>,
    rules: Vec<Rule>,
    signals: Signals,
    /// The signals at the last rule evaluation: the "before" of edge conditions.
    evaluated: Signals,
    groups: Vec<Vec<usize>>,
    group_of: Vec<usize>,
    warned: BTreeSet<String>,
    motion_ticket: Option<u64>,
    next_pallet_id: u64,
    stats_time: SimTime,
    reference: Reference,
}

impl FlowSimulation {
    pub fn new(network: FlowNetwork, seed: u64, reference: &Reference) -> FlowSimulation {
        let mut f = FlowSimulation {
            issues: network.issues,
            sim: Simulator::default(),
            nodes: Vec::new(),
            lines: Vec::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
            sensors: Vec::new(),
            stations: Vec::new(),
            gravity: Vec::new(),
            deck_owner: Vec::new(),
            rules: network.rules,
            signals: Signals::new(),
            evaluated: Signals::new(),
            groups: Vec::new(),
            group_of: Vec::new(),
            warned: BTreeSet::new(),
            motion_ticket: None,
            next_pallet_id: 1,
            stats_time: 0,
            reference: reference.clone(),
        };
        for (index, node) in network.nodes.iter().enumerate() {
            let b = &node.base;
            match &node.kind {
                Kind::Line(data) => f.lines.push(Line {
                    id: b.id.clone(),
                    name: b.name.clone(),
                    carrier: b.id.clone(),
                    deck: false,
                    data: data.clone(),
                    downstream: b.downstream.clone(),
                    downstream_port: b.downstream_port.clone(),
                    gap: b.downstream_gap,
                    pallets: Vec::new(),
                    speed: 0.0,
                    accel: 0.0,
                    t0: 0.0,
                    outgoing: None,
                    incoming: None,
                    holding: false,
                    end: data.length,
                    status: LineStatus::Empty,
                    stats: LineStats::default(),
                }),
                Kind::Turntable(data) => {
                    let deck = f.lines.len();
                    let d = deck_data(data.platform_length, data.nominal_speed, data.accel, data.pallet_side_along_travel);
                    f.lines.push(Line::deck(&b.id, &b.name, &b.id, d));
                    f.stations.push(Station::Turntable(Box::new(Turntable::new(&b.id, (**data).clone(), deck, seed, index as u64 + 1))));
                }
                Kind::Transfer(data) => {
                    let (rollers, chains) = (f.lines.len(), f.lines.len() + 1);
                    let across = if data.pallet_side_along_travel == Side::Long { Side::Short } else { Side::Long };
                    let r = deck_data(data.length, data.nominal_speed, data.accel, data.pallet_side_along_travel);
                    let c = deck_data(data.usable_width, data.transfer_speed, data.transfer_accel, across);
                    f.lines.push(Line::deck(&format!("{}#rollers", b.id), &b.name, &b.id, r));
                    f.lines.push(Line::deck(&format!("{}#chains", b.id), &b.name, &b.id, c));
                    f.stations.push(Station::Transfer(Box::new(Transfer::new(&b.id, (**data).clone(), rollers, chains))));
                }
                Kind::Gravity(data) => f.gravity.push(GravityLine::new(&b.id, (**data).clone(), b.downstream.clone())),
                Kind::Process(data) => {
                    let deck = f.lines.len();
                    let d = deck_data(data.length, data.nominal_speed, data.accel, data.pallet_side_along_travel);
                    f.lines.push(Line::deck(&b.id, &b.name, &b.id, d));
                    let output = (b.downstream.clone(), b.downstream_gap, b.downstream_port.clone());
                    let station = ProcessStation::new(&b.id, (**data).clone(), deck, output, seed, index as u64 + 1);
                    f.stations.push(Station::Process(Box::new(station)));
                }
                Kind::Lift(data) => {
                    let deck = f.lines.len();
                    let d = deck_data(data.length, data.nominal_speed, data.accel, data.pallet_side_along_travel);
                    f.lines.push(Line::deck(&b.id, &b.name, &b.id, d));
                    f.stations.push(Station::Lift(Box::new(LiftStation::new(&b.id, (**data).clone(), deck))));
                }
                Kind::Source(data) => f.sources.push(Source::new(
                    &b.id,
                    (**data).clone(),
                    b.downstream.clone(),
                    b.downstream_port.clone(),
                    seed,
                    index as u64 + 1,
                )),
                Kind::Sink(data) => f.sinks.push(Sink::new(&b.id, data.clone())),
                Kind::Sensor(data) => f.sensors.push(Photoeye::new(&b.id, data.clone())),
                Kind::Inert { .. } => {}
            }
        }
        f.nodes = network.nodes;
        f.deck_owner = vec![None; f.lines.len()];
        for (k, st) in f.stations.iter().enumerate() {
            for d in st.decks() {
                f.deck_owner[d] = Some(k);
            }
        }
        // Inputs start switched on; outputs start from the empty plant.
        for l in f.lines.iter().filter(|l| !l.deck) {
            f.signals.insert(format!("{}.run", l.id), true);
        }
        for s in &f.sources {
            f.signals.insert(format!("{}.enable", s.id), true);
        }
        f.update_outputs();
        f.evaluated = f.signals.clone();
        for i in 0..f.sources.len() {
            if f.sources[i].active() && f.line_after(f.sources[i].downstream.as_deref(), f.sources[i].downstream_port.as_deref()).is_some()
            {
                f.sim.schedule(0, Event::SourceArrival(i), 0);
            }
        }
        f
    }

    pub fn now(&self) -> SimTime {
        self.sim.now()
    }

    /// Runs every event due up to `time` (µs); an error when the plant or the rules don't settle.
    pub fn run_until(&mut self, time: SimTime) -> Result<(), String> {
        while let Some(event) = self.sim.pop_until(time) {
            let now = self.sim.now();
            match event.payload {
                Event::SourceArrival(i) => {
                    if let Some(s) = self.sources.get_mut(i) {
                        s.due_since = Some(now);
                    }
                }
                Event::SinkFree => {}
                Event::SensorOutput(i) => {
                    self.accumulate(now);
                    if let Some(s) = self.sensors.get_mut(i) {
                        s.pending = None;
                        let raw = s.raw;
                        s.commit(raw);
                    }
                }
                Event::SignalWrite(signal, value) => {
                    self.signals.insert(signal, value);
                }
                Event::FlowMotion => self.motion_ticket = None,
                Event::GravityMotion(k) => {
                    if let Some(g) = self.gravity.get_mut(k) {
                        g.ticket = None;
                    }
                }
                Event::StationDone(k) => {
                    // The step's time belongs to the phase that ends now.
                    self.accumulate(now);
                    if let Some(st) = self.stations.get_mut(k) {
                        st.finish_timed_step(now, &mut self.lines);
                    }
                }
            }
            self.settle()?;
        }
        self.sim.advance_to(time);
        Ok(())
    }

    /// When the next event is due (the editor's "step"), if any.
    pub fn next_event_time(&mut self) -> Option<SimTime> {
        self.sim.next_event_time()
    }

    pub fn snapshot(&mut self) -> Snapshot {
        self.update_outputs();
        let t = self.sim.now() as f64 / US_PER_S as f64;
        let now = self.sim.now();
        let mut pallets = Vec::new();
        for (k, l) in self.lines.iter().enumerate() {
            let path = self.deck_owner[k].map(|s| self.stations[s].path_of(k, now)).unwrap_or_default();
            for p in &l.pallets {
                pallets.push(PalletView {
                    id: p.id,
                    type_id: p.type_id.clone(),
                    carrier: l.carrier.clone(),
                    front: l.front_at(p, t),
                    path_x: path.x,
                    path_z: path.z,
                    heading: path.heading,
                    slope: 0.0,
                    speed: l.speed_at(t),
                    accel: l.accel,
                    length: p.length,
                    width: p.width,
                    height: p.height,
                    load_height: p.load_height,
                });
            }
        }
        for g in &self.gravity {
            for p in &g.pallets {
                pallets.push(PalletView {
                    id: p.pallet.id,
                    type_id: p.pallet.type_id.clone(),
                    carrier: g.id.clone(),
                    front: g.front_at(p, t),
                    path_x: 0.0,
                    path_z: 0.0,
                    heading: 0.0,
                    slope: g.data.slope,
                    speed: g.speed_at(p, t),
                    accel: if p.stopped { 0.0 } else { p.accel },
                    length: p.pallet.length,
                    width: p.pallet.width,
                    height: p.pallet.height,
                    load_height: p.pallet.load_height,
                });
            }
        }
        let mut status = BTreeMap::new();
        for g in &self.gravity {
            status.insert(g.id.clone(), g.status());
        }
        for l in self.lines.iter().filter(|l| !l.deck) {
            status.insert(
                l.id.clone(),
                match l.status {
                    LineStatus::Empty => "empty",
                    LineStatus::Working => "working",
                    LineStatus::Blocked => "blocked",
                    LineStatus::Stopped => "stopped",
                },
            );
        }
        for s in &self.sources {
            status.insert(s.id.clone(), if s.due_since.is_none() { "idle" } else { "waiting" });
        }
        for s in &self.sinks {
            status.insert(s.id.clone(), if self.sim.now() < s.busy_until { "busy" } else { "idle" });
        }
        for s in &self.sensors {
            status.insert(s.id.clone(), if s.detecting { "detecting" } else { "clear" });
        }
        let mut angles = BTreeMap::new();
        let mut heights = BTreeMap::new();
        for st in &self.stations {
            let decks = st.decks();
            let deck_status = if decks.iter().any(|&d| self.lines[d].status == LineStatus::Blocked) {
                "blocked"
            } else if decks.iter().any(|&d| self.lines[d].status == LineStatus::Working) {
                "working"
            } else {
                "empty"
            };
            status.insert(st.id().to_string(), st.busy_status().unwrap_or(deck_status));
            if let Some(a) = st.angle_at(now) {
                angles.insert(st.id().to_string(), a);
            }
            if let Some(h) = st.height_at(now) {
                heights.insert(st.id().to_string(), h);
            }
        }
        Snapshot { time: now, pallets, status, signals: self.signals.clone(), angles, heights }
    }

    pub fn statistics(&mut self) -> Statistics {
        self.accumulate(self.sim.now());
        let hours = self.sim.now() as f64 / US_PER_S as f64 / 3600.0;
        let per_hour = |n: u64| if hours > 0.0 { n as f64 / hours } else { 0.0 };
        let now = self.sim.now();
        let mean = |sum: SimTime, n: u64| if n > 0 { sum as f64 / n as f64 / US_PER_S as f64 } else { 0.0 };
        let mut out = Statistics::new();
        for l in self.lines.iter().filter(|l| !l.deck) {
            let s = &l.stats;
            let total = s.empty + s.working + s.blocked;
            out.insert(
                l.id.clone(),
                BTreeMap::from([
                    ("utilization", if total > 0.0 { s.working / total } else { 0.0 }),
                    ("blockedTime", s.blocked),
                    ("emptyTime", s.empty),
                    ("palletsProcessed", s.processed as f64),
                    ("throughputPerHour", per_hour(s.processed)),
                    ("meanTransitTime", mean(s.transit_sum, s.processed)),
                    ("meanOccupancy", if total > 0.0 { s.occupancy / total } else { 0.0 }),
                ]),
            );
        }
        for s in &self.sources {
            out.insert(s.id.clone(), s.statistics(now));
        }
        for st in &self.stations {
            out.insert(st.id().to_string(), st.statistics().into_iter().collect());
        }
        for g in &self.gravity {
            out.insert(g.id.clone(), g.statistics().into_iter().collect());
        }
        for s in &self.sensors {
            out.insert(s.id.clone(), s.statistics());
        }
        for s in &self.sinks {
            out.insert(s.id.clone(), s.statistics(hours));
        }
        out
    }

    /// Settles the plant, then lets the controller react; its immediate writes settle the plant again.
    fn settle(&mut self) -> Result<(), String> {
        for pass in 0.. {
            if pass > MAX_CONTROL_PASSES {
                return Err(format!("The controller rules keep changing signals at {} µs", self.sim.now()));
            }
            self.settle_plant()?;
            if !self.control() {
                break;
            }
        }
        Ok(())
    }

    fn settle_plant(&mut self) -> Result<(), String> {
        let now = self.sim.now();
        self.accumulate(now);
        let t = now as f64 / US_PER_S as f64;
        for l in &mut self.lines {
            l.advance_to(t);
        }
        for g in &mut self.gravity {
            g.advance_to(t);
        }
        for pass in 0.. {
            if pass > MAX_SETTLE_PASSES {
                return Err(format!("Flow did not settle at {now} µs"));
            }
            self.regroup();
            let mut changed = self.complete_crossings(now)?;
            changed = self.feed_sources(now) || changed;
            changed = self.decide(now) || changed;
            changed = self.advance_stations(now) || changed;
            changed = self.serve_gravity_lines(now) || changed;
            if !changed {
                break;
            }
        }
        self.update_sensors();
        for g in 0..self.groups.len() {
            self.plan_group(g);
        }
        self.schedule_motion();
        self.schedule_gravity(now);
        Ok(())
    }

    /// The operator (a sink) takes the pallet resting at a gravity line's end stop when free (S15 §2.5).
    fn serve_gravity_lines(&mut self, now: SimTime) -> bool {
        let mut changed = false;
        for k in 0..self.gravity.len() {
            let g = &self.gravity[k];
            let Some(s) = g.downstream.as_deref().and_then(|d| self.sinks.iter().position(|s| s.id == d)) else { continue };
            if !g.at_end_stop() || !self.sinks[s].accepts(now) {
                continue;
            }
            let Some(pallet) = self.gravity[k].take(now) else { continue };
            let sink = &mut self.sinks[s];
            sink.incoming += 1;
            sink.receive(&pallet, now);
            if sink.busy_until > now {
                let delay = sink.busy_until - now;
                self.sim.schedule(delay, Event::SinkFree, 0);
            }
            changed = true;
        }
        changed
    }

    /// One wake-up per gravity line at its next predicted change.
    fn schedule_gravity(&mut self, now: SimTime) {
        for k in 0..self.gravity.len() {
            if let Some(t) = self.gravity[k].ticket.take() {
                self.sim.cancel(t);
            }
            let Some(next) = self.gravity[k].next_change() else { continue };
            let delay = (((next * US_PER_S as f64 + 0.5).floor() as SimTime) - now).max(1);
            self.gravity[k].ticket = Some(self.sim.schedule(delay, Event::GravityMotion(k), 0));
        }
    }

    /// Pallets whose tail cleared the conveyor's end now belong to the next object.
    fn complete_crossings(&mut self, now: SimTime) -> Result<bool, String> {
        let mut changed = false;
        for i in 0..self.lines.len() {
            let l = &self.lines[i];
            let Some(out) = l.outgoing else { continue };
            let Some(p) = l.pallets.first().filter(|p| p.id == out) else {
                return Err(format!("{}: no outgoing pallet to release", l.id));
            };
            if p.front - p.length < l.data.length - EPS {
                continue;
            }
            // Released.
            let l = &mut self.lines[i];
            let mut pallet = l.pallets.remove(0);
            l.outgoing = None;
            l.stats.processed += 1;
            l.stats.transit_sum += now - pallet.entered_at;
            let (length, gap, downstream, port) = (l.data.length, l.gap, l.downstream.clone(), l.downstream_port.clone());
            let speed = self.lines[i].speed;
            if let Some(k) = downstream.as_deref().and_then(|d| self.gravity.iter().position(|g| g.id == d)) {
                self.gravity[k].receive(pallet, speed, now);
            } else if let Some(k) = downstream.as_deref().and_then(|d| self.sinks.iter().position(|s| s.id == d)) {
                let sink = &mut self.sinks[k];
                sink.receive(&pallet, now);
                if sink.busy_until > now {
                    let delay = sink.busy_until - now;
                    self.sim.schedule(delay, Event::SinkFree, 0);
                }
            } else {
                let target = self
                    .line_after(downstream.as_deref(), port.as_deref())
                    .ok_or_else(|| format!("{}: a crossing pallet has no destination", self.lines[i].id))?;
                pallet.front -= length + gap;
                pallet.entered_at = now;
                let t = &mut self.lines[target];
                t.pallets.push(pallet);
                t.incoming = None;
            }
            changed = true;
        }
        Ok(changed)
    }

    fn feed_sources(&mut self, now: SimTime) -> bool {
        let mut changed = false;
        for i in 0..self.sources.len() {
            let s = &self.sources[i];
            let Some(line) = self.line_after(s.downstream.as_deref(), s.downstream_port.as_deref()) else { continue };
            if s.due_since.is_none() || self.signals.get(&format!("{}.enable", s.id)) == Some(&false) {
                continue;
            }
            let length = s.along_length();
            let port = s.downstream_port.clone();
            // A source may feed a station directly (the operator at a manual lift table, S12).
            let station = self.deck_owner[line];
            let accepts = station.is_none_or(|k| self.stations[k].can_receive(port.as_deref(), &self.lines));
            if accepts && self.can_enter(line, length + self.lines[line].data.min_gap) {
                if let Some(k) = station {
                    self.stations[k].receive(port.as_deref(), length, now, &mut self.lines);
                }
                let s = &self.sources[i];
                let item = &s.data.item;
                let pallet = Pallet {
                    id: self.next_pallet_id,
                    type_id: s.data.pallet_type.id.clone(),
                    along: s.data.orientation,
                    length,
                    width: if s.data.orientation == Side::Long { item.short } else { item.long },
                    height: item.pallet_height,
                    load_mass: s.data.load_mass,
                    load_height: item.load_height,
                    created_at: now,
                    entered_at: now,
                    front: length,
                };
                self.next_pallet_id += 1;
                self.check_orientation(line, &pallet);
                self.lines[line].pallets.push(pallet);
                self.sources[i].placed(now);
            } else if !self.sources[i].data.block_if_occupied {
                self.sources[i].reject();
            } else {
                continue;
            }
            if self.sources[i].active() {
                let delay = self.sources[i].next_delay();
                self.sim.schedule(delay, Event::SourceArrival(i), 0);
            }
            changed = true;
        }
        changed
    }

    /// Front pallets at their decision point are taken downstream, or make their conveyor hold.
    fn decide(&mut self, now: SimTime) -> bool {
        let mut changed = false;
        for i in 0..self.lines.len() {
            let l = &self.lines[i];
            let Some(front) = l.pallets.first() else { continue };
            if l.outgoing.is_some() {
                continue;
            }
            if !l.holding && front.front < self.decision_point(i) - EPS {
                continue;
            }
            if self.reserve(i, now) {
                self.lines[i].holding = false;
                // The groups changed: everything is looked at again.
                return true;
            }
            if !self.lines[i].holding {
                self.lines[i].holding = true;
                changed = true;
            }
        }
        changed
    }

    fn reserve(&mut self, i: usize, now: SimTime) -> bool {
        let (front_id, front_tail, front_len) = match self.lines[i].pallets.first() {
            Some(p) => (p.id, p.front - p.length, p.length),
            None => return false,
        };
        let downstream = self.lines[i].downstream.clone();
        let port = self.lines[i].downstream_port.clone();
        if let Some(k) = downstream.as_deref().and_then(|d| self.gravity.iter().position(|g| g.id == d)) {
            // As a sink: the conveyor pushes the pallet in once a whole pallet fits behind the last one.
            let g = &mut self.gravity[k];
            if !g.can_accept(front_len) {
                g.waiting_length = Some(front_len);
                return false;
            }
            g.waiting_length = None;
            g.incoming += 1;
            self.lines[i].outgoing = Some(front_id);
            if let Some(p) = self.lines[i].pallets.first().cloned() {
                self.check_orientation(i, &p);
            }
            return true;
        }
        if let Some(k) = downstream.as_deref().and_then(|d| self.sinks.iter().position(|s| s.id == d)) {
            if !self.sinks[k].accepts(now) {
                return false;
            }
            self.sinks[k].incoming += 1;
            self.lines[i].outgoing = Some(front_id);
            return true;
        }
        let Some(target) = self.line_after(downstream.as_deref(), port.as_deref()) else { return false };
        let travel = self.lines[i].data.length - front_tail + self.lines[i].gap;
        if !self.can_enter(target, 0.0) || !has_room_for(&self.lines[target], travel) {
            return false;
        }
        // A conveyor switched off can't pull a pallet across.
        if !self.group(target).iter().all(|&m| self.runs(m)) {
            return false;
        }
        if let Some(k) = self.deck_owner[target] {
            if !self.stations[k].can_receive(port.as_deref(), &self.lines) {
                let wait = self.stations[k].prepare_for(port.as_deref(), now, &self.lines);
                self.schedule_step(k, wait, now);
                return false;
            }
            self.stations[k].receive(port.as_deref(), front_len, now, &mut self.lines);
        }
        self.lines[target].incoming = Some(front_id);
        self.lines[i].outgoing = Some(front_id);
        if let Some(p) = self.lines[i].pallets.first().cloned() {
            self.check_orientation(target, &p);
        }
        true
    }

    /// A conveyor without accumulation takes a new pallet when nothing else is entering, it isn't held, and its last
    /// pallet has moved `clearance` beyond the entry.
    fn can_enter(&self, line: usize, clearance: f64) -> bool {
        let l = &self.lines[line];
        if l.incoming.is_some() {
            return false;
        }
        let Some(rear) = l.rear_tail() else { return true };
        if self.is_held(&self.group(line)) {
            return false;
        }
        rear >= clearance - EPS
    }

    /// Where the front pallet must be taken downstream, or braking starts to stop it at the end.
    fn decision_point(&self, line: usize) -> f64 {
        let (max_speed, _, decel) = self.limits(&self.group(line));
        self.lines[line].end - max_speed * max_speed / (2.0 * decel)
    }

    fn group(&self, line: usize) -> Vec<usize> {
        self.group_of.get(line).and_then(|g| self.groups.get(*g)).cloned().unwrap_or_else(|| vec![line])
    }

    fn is_held(&self, group: &[usize]) -> bool {
        group.iter().any(|&l| self.lines[l].holding)
    }

    /// The controller hasn't switched the conveyor off (`<id>.run`, true unless a rule wrote false).
    fn runs(&self, line: usize) -> bool {
        self.signals.get(&format!("{}.run", self.lines[line].id)) != Some(&false)
    }

    /// The most restrictive member decides a group's speed and ramps.
    fn limits(&self, group: &[usize]) -> (f64, f64, f64) {
        let min = |f: fn(&LineData) -> f64| group.iter().map(|&l| f(&self.lines[l].data)).fold(f64::INFINITY, f64::min);
        (min(|d| d.nominal_speed), min(|d| d.accel), min(|d| d.decel))
    }

    /// Groups of conveyors linked by a pallet crossing from one to the next.
    fn regroup(&mut self) {
        let n = self.lines.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn root(parent: &[usize], i: usize) -> usize {
            let mut r = i;
            while parent[r] != r {
                r = parent[r];
            }
            r
        }
        for i in 0..n {
            if self.lines[i].outgoing.is_none() {
                continue;
            }
            let Some(j) = self.line_after(self.lines[i].downstream.as_deref(), self.lines[i].downstream_port.as_deref()) else { continue };
            let (ri, rj) = (root(&parent, i), root(&parent, j));
            parent[rj] = ri;
        }
        let mut order: Vec<usize> = Vec::new();
        let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for i in 0..n {
            let r = root(&parent, i);
            if !by_root.contains_key(&r) {
                order.push(r);
            }
            by_root.entry(r).or_default().push(i);
        }
        self.groups = order.iter().filter_map(|r| by_root.remove(r)).collect();
        self.group_of = vec![0; n];
        for (g, members) in self.groups.iter().enumerate() {
            for &l in members {
                self.group_of[l] = g;
            }
        }
    }

    /// The shared speed and acceleration of a group, from its pallets, holds and limits.
    fn plan_group(&mut self, g: usize) {
        let group = self.groups[g].clone();
        let carriers: Vec<usize> = group.iter().copied().filter(|&l| !self.lines[l].pallets.is_empty()).collect();
        let held = self.is_held(&group);
        let switched_off = !group.iter().all(|&l| self.runs(l));
        for &l in &group {
            let line = &mut self.lines[l];
            line.status = if line.pallets.is_empty() {
                LineStatus::Empty
            } else if held {
                LineStatus::Blocked
            } else if switched_off {
                LineStatus::Stopped
            } else {
                LineStatus::Working
            };
        }
        let set = |s: &mut Self, speed: f64, rate: f64| {
            for &l in &group {
                s.lines[l].speed = speed;
                s.lines[l].accel = rate;
            }
        };
        if carriers.is_empty() {
            set(self, 0.0, 0.0);
            return;
        }
        let (max_speed, accel, decel) = self.limits(&group);
        let mut speed = carriers.iter().map(|&l| self.lines[l].speed).fold(f64::INFINITY, f64::min);
        let rate;
        if held {
            let room = group
                .iter()
                .filter(|&&l| self.lines[l].holding)
                .map(|&l| self.lines[l].end - self.lines[l].pallets.first().map_or(0.0, |p| p.front))
                .fold(f64::INFINITY, f64::min);
            if speed <= SPEED_EPS || room <= EPS {
                speed = 0.0;
                rate = 0.0;
            } else {
                rate = -decel.max(speed * speed / (2.0 * room));
            }
        } else if switched_off {
            if speed <= SPEED_EPS {
                speed = 0.0;
            }
            rate = if speed == 0.0 { 0.0 } else { -decel };
        } else if speed < max_speed - SPEED_EPS {
            rate = accel;
        } else if speed > max_speed + SPEED_EPS {
            rate = -decel;
        } else {
            speed = max_speed;
            rate = 0.0;
        }
        set(self, speed, rate);
    }

    /// One wake-up at the earliest predicted change of any group.
    fn schedule_motion(&mut self) {
        let earliest = (0..self.groups.len()).map(|g| self.next_change(g)).fold(f64::INFINITY, f64::min);
        if let Some(t) = self.motion_ticket.take() {
            self.sim.cancel(t);
        }
        if !earliest.is_finite() {
            return;
        }
        let delay = ((earliest * US_PER_S as f64 + 0.5).floor() as SimTime).max(1);
        self.motion_ticket = Some(self.sim.schedule(delay, Event::FlowMotion, 0));
    }

    /// Seconds until the group's next predicted change, or ∞.
    fn next_change(&mut self, g: usize) -> f64 {
        let group = self.groups[g].clone();
        let Some(&first) = group.first() else { return f64::INFINITY };
        let (speed, accel) = (self.lines[first].speed, self.lines[first].accel);
        if speed == 0.0 && accel == 0.0 {
            return f64::INFINITY;
        }
        let (max_speed, _, _) = self.limits(&group);
        let segment = Segment { t0: 0.0, s0: 0.0, v0: speed, a: accel };
        let mut times: Vec<f64> = Vec::new();
        if accel > 0.0 {
            times.push((max_speed - speed) / accel);
        }
        let to_rest = self.is_held(&group) || !group.iter().all(|&l| self.runs(l));
        if accel < 0.0 {
            times.push(if to_rest { speed / -accel } else { (speed - max_speed) / -accel });
        }
        for &l in &group {
            let line = &self.lines[l];
            let Some(p) = line.pallets.first() else { continue };
            let distance = if line.outgoing.is_some() {
                Some(line.data.length - (p.front - p.length))
            } else if line.holding {
                None
            } else {
                Some(self.decision_point(l) - p.front)
            };
            if let Some(d) = distance.filter(|d| *d > 0.0)
                && let Some(t) = time_to_reach(&segment, d)
            {
                times.push(t);
            }
        }
        for k in 0..self.sensors.len() {
            let Some(host) = self.lines.iter().position(|l| l.id == self.sensors[k].data.host) else { continue };
            if !group.contains(&host) {
                continue;
            }
            for (pallet, front) in self.pallets_over(host) {
                let intervals = self.sensors[k].intervals(&pallet, &self.reference);
                let next = intervals.iter().flat_map(|(a, b)| [*a, *b]).find(|b| *b > front + EPS);
                if let Some(t) = next.and_then(|n| time_to_reach(&segment, n - front)) {
                    times.push(t);
                }
            }
        }
        for &l in &group {
            let (Some(needed), Some(rear)) = (self.entry_waiter_clearance(l), self.lines[l].rear_tail()) else { continue };
            if rear >= needed - EPS {
                continue;
            }
            if let Some(t) = time_to_reach(&segment, needed - rear) {
                times.push(t);
            }
        }
        times.into_iter().filter(|t| *t >= 0.0).fold(f64::INFINITY, f64::min)
    }

    /// The clearance behind its last pallet that a waiting feeder (a source or a held conveyor) needs.
    fn entry_waiter_clearance(&self, line: usize) -> Option<f64> {
        let id = &self.lines[line].id;
        for s in &self.sources {
            if s.downstream.as_ref() == Some(id) && s.due_since.is_some() {
                return Some(s.along_length() + self.lines[line].data.min_gap);
            }
        }
        let feeder = (0..self.lines.len()).find(|&l| {
            self.lines[l].outgoing.is_none()
                && self.line_after(self.lines[l].downstream.as_deref(), self.lines[l].downstream_port.as_deref()) == Some(line)
        });
        feeder.filter(|&f| self.lines[f].holding).map(|_| 0.0)
    }

    /// The conveyors' and sensors' output signals from the current state.
    fn update_outputs(&mut self) {
        for l in self.lines.iter().filter(|l| !l.deck) {
            self.signals.insert(format!("{}.running", l.id), l.speed > 0.0 || l.accel > 0.0);
            self.signals.insert(format!("{}.occupied", l.id), !l.pallets.is_empty());
        }
        for s in &self.sensors {
            self.signals.insert(s.signal.clone(), s.signal_value());
        }
    }

    /// Evaluates the rules when a signal changed since the last evaluation (A08 §2): delayed writes become events;
    /// true when an immediate write changed an input, so the plant must settle again.
    fn control(&mut self) -> bool {
        if self.rules.is_empty() {
            return false;
        }
        self.update_outputs();
        let current = self.signals.clone();
        if current.iter().all(|(k, v)| self.evaluated.get(k) == Some(v)) {
            return false;
        }
        let writes = evaluate_rules(&self.rules, &self.evaluated, &current);
        self.evaluated = current;
        let mut changed = false;
        for w in writes {
            if w.delay_ms > 0.0 {
                self.sim.schedule(from_ms(w.delay_ms), Event::SignalWrite(w.signal, w.value), 0);
            } else if self.signals.get(&w.signal) != Some(&w.value) {
                self.signals.insert(w.signal, w.value);
                changed = true;
            }
        }
        changed
    }

    /// A beam change starts the sensor's response time (and filter when it frees); the output follows when it ends.
    fn update_sensors(&mut self) {
        for k in 0..self.sensors.len() {
            let host = self.lines.iter().position(|l| l.id == self.sensors[k].data.host);
            let over = host.map(|h| self.pallets_over(h)).unwrap_or_default();
            let mut raw = false;
            for (p, front) in &over {
                if self.sensors[k].intervals(p, &self.reference).iter().any(|(a, b)| *front >= a - EPS && *front < b - EPS) {
                    raw = true;
                    break;
                }
            }
            let s = &mut self.sensors[k];
            if raw == s.raw {
                continue;
            }
            s.raw = raw;
            if let Some(t) = s.pending.take() {
                self.sim.cancel(t);
            }
            let delay = self.sensors[k].delay_for(raw);
            if delay == 0 {
                self.sensors[k].commit(raw);
            } else {
                self.sensors[k].pending = Some(self.sim.schedule(delay, Event::SensorOutput(k), 0));
            }
        }
    }

    /// Pallets over a conveyor with their fronts in its frame: its own, and the one crossing onto it.
    fn pallets_over(&self, line: usize) -> Vec<(Pallet, f64)> {
        let l = &self.lines[line];
        let mut over: Vec<(Pallet, f64)> = l.pallets.iter().map(|p| (p.clone(), p.front)).collect();
        if let Some(incoming) = l.incoming
            && let Some(feeder) = self.lines.iter().find(|f| f.outgoing == Some(incoming))
            && let Some(p) = feeder.pallets.iter().find(|p| p.id == incoming)
        {
            over.push((p.clone(), p.front - feeder.data.length - feeder.gap));
        }
        over
    }

    fn accumulate(&mut self, now: SimTime) {
        let seconds = (now - self.stats_time) as f64 / US_PER_S as f64;
        if seconds <= 0.0 {
            return;
        }
        for l in &mut self.lines {
            l.accumulate(seconds);
        }
        for st in &mut self.stations {
            st.accumulate(seconds, &self.lines);
        }
        for g in &mut self.gravity {
            g.accumulate(seconds);
        }
        for s in &mut self.sensors {
            s.accumulate(seconds);
        }
        self.stats_time = now;
    }

    fn check_orientation(&mut self, line: usize, p: &Pallet) {
        let l = &self.lines[line];
        if p.along == l.data.pallet_side_along_travel || self.warned.contains(&l.id) {
            return;
        }
        self.warned.insert(l.id.clone());
        let side = |s: Side| if s == Side::Long { "long" } else { "short" };
        let who = if l.name.is_empty() { &l.id } else { &l.name };
        self.issues.push(FlowIssue {
            instance_id: l.id.clone(),
            rule: "palletOrientationMismatch",
            message: format!(
                "{who}: receives pallets with the {} side along travel, but is set up for the {} side",
                side(p.along),
                side(l.data.pallet_side_along_travel)
            ),
        });
    }

    /// The conveyor or station deck that takes what a node with this downstream (and port) delivers.
    fn line_after(&self, downstream: Option<&str>, port: Option<&str>) -> Option<usize> {
        let d = downstream?;
        if let Some(st) = self.stations.iter().find(|s| s.id() == d) {
            return st.entry_deck(port);
        }
        self.lines.iter().position(|l| !l.deck && l.id == d)
    }

    /// Stations move on: a centred pallet starts a turn, an emptied deck frees the station.
    fn advance_stations(&mut self, now: SimTime) -> bool {
        let mut changed = false;
        for k in 0..self.stations.len() {
            let step = self.stations[k].step(now, &self.lines);
            self.schedule_step(k, step.wait, now);
            changed = step.changed || changed;
        }
        changed
    }

    /// Ends a station's timed step after `duration` µs, or at once when it takes no time.
    fn schedule_step(&mut self, k: usize, duration: Option<SimTime>, now: SimTime) {
        match duration {
            None => {}
            Some(0) => self.stations[k].finish_timed_step(now, &mut self.lines),
            Some(d) => {
                self.sim.schedule(d, Event::StationDone(k), 0);
            }
        }
    }

    /// The nodes of the network the run was built from.
    pub fn nodes(&self) -> &[FlowNode] {
        &self.nodes
    }
}

/// A conveyor takes a pallet from the previous one only if it can get wholly aboard before the conveyor's own leading
/// pallet reaches the end: it never stops with a pallet half on.
fn has_room_for(target: &Line, travel: f64) -> bool {
    match target.pallets.iter().find(|p| Some(p.id) != target.outgoing) {
        None => true,
        Some(lead) => lead.front + travel <= target.data.length + EPS,
    }
}
