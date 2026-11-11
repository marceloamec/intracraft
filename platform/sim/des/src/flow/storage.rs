//! A storage aisle of a high-bay warehouse (S26, ADR-0033): two rack rows and the stacker crane between them. The crane
//! lifts a pallet with its forks off the end of an infeed chain conveyor, sets it down in a rack position, and later takes
//! it out and sets it down on the start of an outfeed chain conveyor.
//!
//! The aisle's logic is the same for the 2D run and the 3D run, as a source's or a sink's (`parts`): it keeps the rack,
//! the orders and the crane's jobs, and asks the run for the four things only the run can do — take the pallet resting at
//! an entry, say whether an exit has room, set a pallet down there, and wake it up at a time.
//!
//! Own frame (S26 §1): x along the aisle into the rack, y up, z across (the left row at −z). A crane place is the forks'
//! centre along the aisle, their height, and how far they reach across.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::sync::Arc;

use intracraft_library::definition::Value;
use intracraft_library::params::{Values, num};
use intracraft_library::reference::Reference;

use crate::flow::network::{Side, StationExit};
use crate::flow::sim::Pallet;
use crate::kinematics::{MoveProfile, profile_position_at, profile_time};
use crate::rng::Pcg32;
use crate::time::{SimTime, US_PER_S, from_seconds};

/// `behaviors.des` of a storage aisle.
pub const BEHAVIOR: &str = "des.storage_aisle";
/// Where the forks stand along the aisle to serve a station, m: the centre of a EUR pallet resting at the conveyor's end.
pub const STATION_X: f64 = -0.4;
/// A pallet with its load must be this much lower than the level pitch (clearance and beam), m.
pub const LOAD_CLEARANCE: f64 = 0.25;
/// The station heights an aisle can have.
pub const MAX_STATION_HEIGHTS: usize = 3;
/// The forks reach out during this share of a fork cycle, and come back during the same share at its end.
const FORK_REACH_SHARE: f64 = 0.4;
/// The ids of the pallets an aisle starts with begin here, far above those the sources give.
const STOCK_ID_BASE: u64 = 1 << 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Nearest,
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retrieval {
    None,
    Dwell,
    Interval,
}

/// A station port of an aisle and its conveying height, m.
#[derive(Clone, Debug, PartialEq)]
pub struct StationPort {
    pub port_id: String,
    pub height: f64,
}

/// A storage aisle as the layout describes it (S26 §2).
#[derive(Clone, Debug, PartialEq)]
pub struct StorageData {
    pub bays: usize,
    pub levels: usize,
    /// Pallets deep on each side: 1 or 2.
    pub deep: usize,
    pub bay_pitch: f64,
    pub level_pitch: f64,
    pub first_level_height: f64,
    pub aisle_width: f64,
    pub position_depth: f64,
    pub travel_speed: f64,
    pub travel_accel: f64,
    pub lift_speed: f64,
    pub lift_accel: f64,
    pub fork_time: f64,
    pub fork_time_double: f64,
    /// An idle crane goes back to the lowest station (`dwellPoint` = `front`).
    pub dwell_front: bool,
    /// The heights with stations, lowest index first (`pdHeight1` … up to `pdLevels`).
    pub station_heights: Vec<f64>,
    pub strategy: Strategy,
    pub retrieval: Retrieval,
    pub dwell_time: f64,
    pub retrieval_interval: f64,
    pub exponential: bool,
    pub fifo: bool,
    pub initial_fill: f64,
    pub initial_load_height: f64,
    pub initial_load_mass: f64,
    /// Its connected entry ports and the conveyor that ends at each.
    pub feeders: BTreeMap<String, String>,
    /// Its connected exit ports and where they lead.
    pub exits: BTreeMap<String, StationExit>,
}

impl StorageData {
    /// The aisle's values from its parameters; its connections are added by the network.
    pub fn read(p: &Values) -> StorageData {
        let text = |k: &str| p.get(k).and_then(Value::text);
        let count = |k: &str, max: f64| num(p, k, 1.0).clamp(1.0, max).floor() as usize;
        let heights = count("pdLevels", MAX_STATION_HEIGHTS as f64);
        StorageData {
            bays: count("bays", 150.0),
            levels: count("levels", 25.0),
            deep: count("deep", 2.0),
            bay_pitch: num(p, "bayPitch", 1.0),
            level_pitch: num(p, "levelPitch", 1.8),
            first_level_height: num(p, "firstLevelHeight", 0.6),
            aisle_width: num(p, "aisleWidth", 1.5),
            position_depth: num(p, "positionDepth", 1.3),
            travel_speed: num(p, "travelSpeed", 3.0),
            travel_accel: num(p, "travelAccel", 0.5),
            lift_speed: num(p, "liftSpeed", 1.0),
            lift_accel: num(p, "liftAccel", 0.5),
            fork_time: num(p, "forkTime", 8.0).max(0.0),
            fork_time_double: num(p, "forkTimeDouble", 12.0).max(0.0),
            dwell_front: text("dwellPoint") == Some("front"),
            station_heights: (1..=heights).map(|n| num(p, &format!("pdHeight{n}"), 0.6)).collect(),
            strategy: if text("storageStrategy") == Some("random") { Strategy::Random } else { Strategy::Nearest },
            retrieval: match text("retrievalMode") {
                Some("none") => Retrieval::None,
                Some("interval") => Retrieval::Interval,
                _ => Retrieval::Dwell,
            },
            dwell_time: num(p, "dwellTime", 3600.0).max(0.0),
            retrieval_interval: num(p, "retrievalInterval", 120.0).max(1e-3),
            exponential: text("timeDistribution") == Some("exponential"),
            fifo: text("retrievalOrder") != Some("random"),
            initial_fill: num(p, "initialFill", 0.5).clamp(0.0, 1.0),
            initial_load_height: num(p, "initialLoadHeight", 1.2).max(0.0),
            initial_load_mass: num(p, "initialLoadMass", 600.0).max(0.0),
            feeders: BTreeMap::new(),
            exits: BTreeMap::new(),
        }
    }

    /// Its storage positions.
    pub fn positions(&self) -> usize {
        self.bays * self.levels * 2 * self.deep
    }

    /// The height of the rack's top, m.
    pub fn rack_height(&self) -> f64 {
        self.first_level_height + self.levels as f64 * self.level_pitch
    }

    /// The height of station port `port` (`IN2`, `OUT1`…), if its height is in use.
    pub fn station_height(&self, port: &str) -> Option<f64> {
        let n: usize = port.strip_prefix("IN").or_else(|| port.strip_prefix("OUT"))?.parse().ok()?;
        self.station_heights.get(n.checked_sub(1)?).copied()
    }

    /// How far the forks reach across the aisle to a station or a front position, m.
    pub fn front_reach(&self) -> f64 {
        (self.aisle_width + self.position_depth) / 2.0
    }

    /// Where storage position `slot` is: bay, level, side (0 left, 1 right) and depth (0 front, 1 rear).
    pub fn address(&self, slot: usize) -> (usize, usize, usize, usize) {
        let channel = slot / self.deep;
        (channel / (2 * self.levels), (channel / 2) % self.levels, channel % 2, slot % self.deep)
    }

    /// Where a pallet stands in position `slot`: the centre of its underside in the aisle's frame, m.
    pub fn place_of(&self, slot: usize) -> [f64; 3] {
        let (bay, level, side, deep) = self.address(slot);
        let reach = self.front_reach() + deep as f64 * self.position_depth;
        [
            (bay as f64 + 0.5) * self.bay_pitch,
            self.first_level_height + level as f64 * self.level_pitch,
            if side == 0 { -reach } else { reach },
        ]
    }

    /// The time of a crane move between two places, s: travelling and lifting at once, the longer of the two (S26 §3).
    pub fn move_time(&self, from: [f64; 3], to: [f64; 3]) -> f64 {
        let along = profile_time(&self.travel_profile((to[0] - from[0]).abs())).unwrap_or(0.0);
        let up = profile_time(&self.lift_profile((to[1] - from[1]).abs())).unwrap_or(0.0);
        along.max(up)
    }

    /// The time of one fork cycle at position `slot` (a rear position takes longer), or at a station, s.
    pub fn fork_cycle(&self, slot: Option<usize>) -> f64 {
        match slot {
            Some(s) if s % self.deep == 1 => self.fork_time_double,
            _ => self.fork_time,
        }
    }

    fn travel_profile(&self, distance: f64) -> MoveProfile {
        MoveProfile { distance, max_speed: self.travel_speed, accel: self.travel_accel, decel: Some(self.travel_accel) }
    }

    fn lift_profile(&self, distance: f64) -> MoveProfile {
        MoveProfile { distance, max_speed: self.lift_speed, accel: self.lift_accel, decel: Some(self.lift_accel) }
    }
}

/// What the aisle needs from the run before it can go on.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Nothing now, and no time to wake it at: a pallet at an entry or an order will.
    Nothing,
    /// Wake it at this time, µs.
    Wait(SimTime),
    /// Take the pallet resting at entry `entry` off its conveyor and `give` it.
    Take { entry: usize },
    /// Say with `room` whether the conveyor at exit `exit` can take a pallet `length` long at its start.
    Room { exit: usize, length: f64 },
    /// `release` the pallet on the forks and set it down at the start of the conveyor at exit `exit`.
    Put { exit: usize },
}

/// A pallet in the rack or on the forks as a viewer draws it: the centre of its underside in the aisle's frame.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredView {
    pub id: u64,
    pub type_id: String,
    pub source: String,
    /// Its side along the aisle and its side across, m.
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub load_height: f64,
    pub at: [f64; 3],
    /// Its load is wrapped in film (S25).
    pub wrapped: bool,
}

/// An aisle as a viewer draws it at one time.
#[derive(Clone, Debug, PartialEq)]
pub struct AisleView {
    /// The forks: their centre along the aisle, their height, and how far they reach across (− left, + right), m.
    pub crane: [f64; 3],
    pub carried: Option<StoredView>,
    /// The pallets in the rack; the same list until one comes or goes.
    pub stored: Arc<Vec<StoredView>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Entry(usize),
    Exit(usize),
    Slot(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JobKind {
    Store,
    Retrieve,
    Relocate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Job {
    kind: JobKind,
    pick: Target,
    place: Target,
    started: SimTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    ToPick,
    /// The forks reach out and lift: at its end the pallet is theirs.
    PickOut,
    PickBack,
    ToPlace,
    AwaitRoom,
    /// The forks reach out and lower: at its end the pallet stands.
    PlaceOut,
    PlaceBack,
}

#[derive(Clone, Copy, Debug)]
struct Motion {
    start: SimTime,
    end: SimTime,
    from: [f64; 3],
    to: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
struct Fork {
    start: SimTime,
    end: SimTime,
    reach: f64,
}

struct Stored {
    pallet: Pallet,
    /// The order in which pallets came: first in, first out.
    seq: u64,
}

#[derive(Clone, Debug, Default)]
struct Stats {
    total: f64,
    busy: f64,
    blocked: f64,
    stored: u64,
    retrieved: u64,
    relocations: u64,
    missed: u64,
    jobs: u64,
    job_time: SimTime,
    combined: u64,
    waits: u64,
    wait_time: SimTime,
}

/// The aisle during a run.
pub struct StorageAisle {
    pub id: String,
    pub data: StorageData,
    /// Its connected entries and exits, in port order.
    pub entries: Vec<StationPort>,
    pub exit_ports: Vec<StationPort>,
    /// What the run should warn about, once each: the rule and the text.
    pub notes: Vec<(&'static str, String)>,
    slots: Vec<Option<Stored>>,
    reserved: Vec<bool>,
    /// The position of every stored pallet.
    slot_of: BTreeMap<u64, usize>,
    /// Stored pallets no order has asked for yet, oldest first (`interval`).
    unasked: BTreeMap<u64, u64>,
    /// When each stored pallet asks to leave (`dwell`).
    due: BinaryHeap<Reverse<(SimTime, u64)>>,
    /// The pallets asked out, oldest request first.
    requests: VecDeque<u64>,
    next_order: Option<SimTime>,
    /// The pallet resting at each entry, and since when.
    waiting: Vec<Option<(u64, SimTime)>>,
    crane: [f64; 3],
    motion: Option<Motion>,
    fork: Option<Fork>,
    job: Option<Job>,
    step: Step,
    until: SimTime,
    asked: bool,
    blocked: bool,
    /// The job that must follow the current one: the retrieval a relocation was made for.
    forced: Option<Job>,
    carried: Option<Pallet>,
    /// The number of the pallet a relocation carries: it keeps its place in the order of leaving.
    seq_in_hand: Option<u64>,
    last_kind: Option<JobKind>,
    next_exit: usize,
    seq: u64,
    rng: Pcg32,
    stats: Stats,
    view: Arc<Vec<StoredView>>,
    view_stale: bool,
}

impl StorageAisle {
    /// The aisle at the start of a run, with its initial stock (S26 §4.5). `index` tells its pallets' ids and its random
    /// stream from those of the other aisles.
    pub fn new(id: &str, data: StorageData, reference: &Reference, seed: u64, index: u64) -> StorageAisle {
        let port = |p: &String| data.station_height(p).map(|height| StationPort { port_id: p.clone(), height });
        let entries: Vec<StationPort> = data.feeders.keys().filter_map(port).collect();
        let exit_ports: Vec<StationPort> = data.exits.keys().filter_map(port).collect();
        let positions = data.positions();
        let mut aisle = StorageAisle {
            id: id.to_string(),
            waiting: vec![None; entries.len()],
            entries,
            exit_ports,
            notes: Vec::new(),
            slots: (0..positions).map(|_| None).collect(),
            reserved: vec![false; positions],
            slot_of: BTreeMap::new(),
            unasked: BTreeMap::new(),
            due: BinaryHeap::new(),
            requests: VecDeque::new(),
            next_order: None,
            crane: [STATION_X, data.station_heights.first().copied().unwrap_or(0.6), 0.0],
            motion: None,
            fork: None,
            job: None,
            step: Step::ToPick,
            until: 0,
            asked: false,
            blocked: false,
            forced: None,
            carried: None,
            seq_in_hand: None,
            last_kind: None,
            next_exit: 0,
            seq: 0,
            rng: Pcg32::new(seed, index + 1),
            stats: Stats::default(),
            view: Arc::new(Vec::new()),
            view_stale: true,
            data,
        };
        aisle.crane = aisle.home();
        aisle.fill(reference, index);
        aisle.refresh_view();
        if aisle.retrieves() && aisle.data.retrieval == Retrieval::Interval {
            aisle.next_order = Some(aisle.draw(aisle.data.retrieval_interval));
        }
        aisle
    }

    /// The run saw the pallet `pallet` resting at the end of the conveyor of entry `entry`, or none there.
    pub fn see_entry(&mut self, entry: usize, pallet: Option<u64>, now: SimTime) {
        let Some(w) = self.waiting.get_mut(entry) else { return };
        match (pallet, *w) {
            (Some(id), Some((known, _))) if id == known => {}
            (Some(id), _) => *w = Some((id, now)),
            (None, _) => *w = None,
        }
    }

    /// Goes on as far as it can at `now`, then says what it needs from the run.
    pub fn next(&mut self, now: SimTime) -> Request {
        let request = self.go_on(now);
        self.refresh_view();
        request
    }

    fn go_on(&mut self, now: SimTime) -> Request {
        self.orders_due(now);
        loop {
            let Some(job) = self.job else {
                // Without a job the crane may still be on its way back to the front.
                if let Some(m) = self.motion {
                    if now < m.end {
                        return Request::Wait(m.end);
                    }
                    self.arrive();
                }
                if let Some(job) = self.choose_job(now) {
                    self.start_job(job, now);
                    continue;
                }
                let home = self.home();
                if self.data.dwell_front && !same_place(self.crane, home) {
                    self.start_move(home, now);
                    continue;
                }
                return Request::Nothing;
            };
            if self.step != Step::AwaitRoom && now < self.until {
                return Request::Wait(self.until);
            }
            match self.step {
                Step::ToPick => {
                    self.arrive();
                    self.start_fork(job.pick, now);
                    self.step = Step::PickOut;
                }
                Step::PickOut => match job.pick {
                    Target::Entry(entry) if !self.asked => {
                        self.asked = true;
                        return Request::Take { entry };
                    }
                    // The run had nothing to give: the job is dropped.
                    Target::Entry(_) | Target::Exit(_) => self.abandon(),
                    Target::Slot(slot) => {
                        self.carried = self.take_out(slot);
                        self.step = Step::PickBack;
                        self.until = self.fork.map_or(now, |f| f.end);
                    }
                },
                Step::PickBack => {
                    self.fork = None;
                    let to = self.place_for(job.place);
                    self.start_move(to, now);
                    self.step = Step::ToPlace;
                }
                Step::ToPlace => {
                    self.arrive();
                    if matches!(job.place, Target::Exit(_)) {
                        self.step = Step::AwaitRoom;
                    } else {
                        self.start_fork(job.place, now);
                        self.step = Step::PlaceOut;
                    }
                }
                Step::AwaitRoom => {
                    let (Target::Exit(exit), Some(p)) = (job.place, &self.carried) else {
                        self.abandon();
                        continue;
                    };
                    return Request::Room { exit, length: p.length };
                }
                Step::PlaceOut => match job.place {
                    Target::Exit(exit) if !self.asked => {
                        self.asked = true;
                        return Request::Put { exit };
                    }
                    Target::Exit(_) | Target::Entry(_) => self.abandon(),
                    Target::Slot(slot) => {
                        if let Some(pallet) = self.carried.take() {
                            self.put_in(slot, pallet, job.kind, now);
                        }
                        self.step = Step::PlaceBack;
                        self.until = self.fork.map_or(now, |f| f.end);
                    }
                },
                Step::PlaceBack => self.finish_job(job, now),
            }
        }
    }

    /// The run's answer to `Take`: the pallet it took off the entry's conveyor, or none if it wasn't there any more.
    pub fn give(&mut self, pallet: Option<Pallet>, now: SimTime) {
        let Some(Job { pick: Target::Entry(entry), .. }) = self.job else { return };
        let since = self.waiting.get_mut(entry).and_then(Option::take).map_or(now, |(_, since)| since);
        let Some(pallet) = pallet else {
            self.abandon();
            return;
        };
        self.stats.waits += 1;
        self.stats.wait_time += now - since;
        if pallet.height + pallet.load_height > self.data.level_pitch - LOAD_CLEARANCE + 1e-9 {
            let text = format!(
                "a pallet {:.0} mm high with its load is stored in levels {:.0} mm apart",
                (pallet.height + pallet.load_height) * 1000.0,
                self.data.level_pitch * 1000.0
            );
            self.notes.push(("loadTooTall", text));
        }
        if pallet.along == Side::Long {
            self.notes.push(("palletAcrossAisle", "a pallet arrives long side first; the forks can't take it that way".into()));
        }
        self.carried = Some(pallet);
        self.asked = false;
        self.step = Step::PickBack;
        self.until = self.fork.map_or(now, |f| f.end);
    }

    /// The run's answer to `Room`: with room the forks start setting the pallet down; without, the crane waits blocked.
    pub fn room(&mut self, free: bool, now: SimTime) {
        let Some(Job { place, .. }) = self.job else { return };
        if self.step != Step::AwaitRoom {
            return;
        }
        self.blocked = !free;
        if free {
            self.start_fork(place, now);
            self.step = Step::PlaceOut;
        }
    }

    /// The run's part of `Put`: the pallet leaves the forks; the run sets it down on the exit's conveyor.
    pub fn release(&mut self, now: SimTime) -> Option<Pallet> {
        let pallet = self.carried.take()?;
        self.stats.retrieved += 1;
        self.asked = false;
        self.step = Step::PlaceBack;
        self.until = self.fork.map_or(now, |f| f.end);
        Some(pallet)
    }

    /// The exit its crane waits at with a pallet for room, and the pallet's length along the conveyor, m.
    pub fn awaiting_room(&self) -> Option<(usize, f64)> {
        match (self.job, &self.carried) {
            (Some(Job { place: Target::Exit(exit), .. }), Some(p)) if self.step == Step::AwaitRoom => Some((exit, p.length)),
            _ => None,
        }
    }

    /// When the aisle must be looked at again without anything else happening, µs.
    pub fn next_wake(&self) -> Option<SimTime> {
        let timed = match (self.job, self.motion) {
            (Some(_), _) if self.step != Step::AwaitRoom => Some(self.until),
            (None, Some(m)) => Some(m.end),
            _ => None,
        };
        let due = if self.retrieves() { self.due.peek().map(|Reverse((t, _))| *t) } else { None };
        [timed, due, self.next_order].into_iter().flatten().min()
    }

    pub fn accumulate(&mut self, seconds: f64) {
        self.stats.total += seconds;
        if self.job.is_none() {
            return;
        }
        if self.blocked && self.step == Step::AwaitRoom {
            self.stats.blocked += seconds;
        } else {
            self.stats.busy += seconds;
        }
    }

    /// `idle`, `travelling`, `picking`, `placing` or `blocked`.
    pub fn status(&self) -> &'static str {
        match (self.job, self.step) {
            (None, _) if self.motion.is_some() => "travelling",
            (None, _) => "idle",
            (_, Step::ToPick | Step::ToPlace) => "travelling",
            (_, Step::PickOut | Step::PickBack) => "picking",
            (_, Step::AwaitRoom) => "blocked",
            (_, Step::PlaceOut | Step::PlaceBack) => "placing",
        }
    }

    /// The pallets in the rack now.
    pub fn stock(&self) -> usize {
        self.slot_of.len()
    }

    pub fn statistics(&self) -> Vec<(&'static str, f64)> {
        let s = &self.stats;
        let share = |x: f64| if s.total > 0.0 { x / s.total } else { 0.0 };
        let mean = |sum: SimTime, n: u64| if n > 0 { sum as f64 / n as f64 / US_PER_S as f64 } else { 0.0 };
        let positions = self.data.positions();
        vec![
            ("utilization", share(s.busy)),
            ("blockedShare", share(s.blocked)),
            ("idleShare", share(s.total - s.busy - s.blocked)),
            ("palletsStored", s.stored as f64),
            ("palletsRetrieved", s.retrieved as f64),
            ("relocations", s.relocations as f64),
            ("ordersMissed", s.missed as f64),
            ("meanCycleTime", mean(s.job_time, s.jobs)),
            ("combinedShare", if s.retrieved > 0 { s.combined as f64 / s.retrieved as f64 } else { 0.0 }),
            ("meanWaitAtEntry", mean(s.wait_time, s.waits)),
            ("stock", self.stock() as f64),
            ("fillLevel", if positions > 0 { self.stock() as f64 / positions as f64 } else { 0.0 }),
        ]
    }

    /// The list of the rack's pallets, made again when one came or went.
    fn refresh_view(&mut self) {
        if self.view_stale {
            let stored = self.slots.iter().enumerate().filter_map(|(slot, s)| s.as_ref().map(|s| (slot, s)));
            self.view = Arc::new(stored.map(|(slot, s)| stored_view(&s.pallet, self.data.place_of(slot))).collect());
            self.view_stale = false;
        }
    }

    /// The crane, the pallet on its forks and the rack's pallets at `now`.
    pub fn view(&self, now: SimTime) -> AisleView {
        let mut crane = self.crane;
        if let Some(m) = self.motion {
            let t = (now - m.start) as f64 / US_PER_S as f64;
            let along = (m.to[0] - m.from[0]).abs();
            let up = (m.to[1] - m.from[1]).abs();
            let gone_along = profile_position_at(&self.data.travel_profile(along), t).unwrap_or(along);
            let gone_up = profile_position_at(&self.data.lift_profile(up), t).unwrap_or(up);
            crane[0] = m.from[0] + (m.to[0] - m.from[0]).signum() * gone_along;
            crane[1] = m.from[1] + (m.to[1] - m.from[1]).signum() * gone_up;
        }
        if let Some(f) = self.fork {
            let span = (f.end - f.start).max(1) as f64;
            let share = ((now - f.start) as f64 / span).clamp(0.0, 1.0);
            let out = (share / FORK_REACH_SHARE).min((1.0 - share) / FORK_REACH_SHARE).clamp(0.0, 1.0);
            crane[2] = f.reach * out;
        }
        AisleView { crane, carried: self.carried.as_ref().map(|p| stored_view(p, crane)), stored: Arc::clone(&self.view) }
    }

    /// The lowest connected station, where the crane starts and an idle one may wait.
    fn home(&self) -> [f64; 3] {
        let lowest = self.entries.iter().chain(&self.exit_ports).map(|p| p.height).fold(f64::INFINITY, f64::min);
        let height = if lowest.is_finite() { lowest } else { self.data.station_heights.first().copied().unwrap_or(0.6) };
        [STATION_X, height, 0.0]
    }

    /// Pallets can leave: a retrieval mode is set and an exit is connected.
    fn retrieves(&self) -> bool {
        self.data.retrieval != Retrieval::None && !self.exit_ports.is_empty()
    }

    /// A time about `mean` s as `timeDistribution` says, µs.
    fn draw(&mut self, mean: f64) -> SimTime {
        let seconds = if self.data.exponential { -mean * (1.0 - self.rng.next_f64()).ln() } else { mean };
        from_seconds(seconds).max(1)
    }

    /// The initial stock: `initialFill` of the positions, placed as the storage strategy says from the home station.
    fn fill(&mut self, reference: &Reference, index: u64) {
        let count = (self.data.initial_fill * self.data.positions() as f64 + 1e-9).floor() as usize;
        if count == 0 {
            return;
        }
        let (type_id, long, short, height) =
            reference.pallet_type("eur").map_or(("eur".to_string(), 1.2, 0.8, 0.144), |t| (t.id.clone(), t.length, t.width, t.height));
        let channels = self.data.positions() / self.data.deep;
        let mut order: Vec<usize> = (0..channels).collect();
        let home = self.home();
        match self.data.strategy {
            Strategy::Nearest => {
                let time = |c: usize| self.data.move_time(home, self.data.place_of(c * self.data.deep));
                order.sort_by(|a, b| time(*a).total_cmp(&time(*b)).then(a.cmp(b)));
                order = order.iter().flat_map(|c| std::iter::repeat_n(*c, self.data.deep)).collect();
            }
            Strategy::Random => {
                order = order.iter().flat_map(|c| std::iter::repeat_n(*c, self.data.deep)).collect();
                for k in (1..order.len()).rev() {
                    let j = (self.rng.next_f64() * (k + 1) as f64) as usize;
                    order.swap(k, j.min(k));
                }
            }
        }
        for (n, channel) in order.into_iter().take(count).enumerate() {
            let Some(slot) = self.free_in(channel) else { continue };
            let pallet = Pallet {
                id: STOCK_ID_BASE * (index + 1) + n as u64,
                type_id: type_id.clone(),
                source: self.id.clone(),
                along: Side::Short,
                length: short,
                width: long,
                height,
                load_mass: self.data.initial_load_mass,
                load_height: self.data.initial_load_height,
                created_at: 0,
                entered_at: 0,
                front: 0.0,
                wrapped: false,
            };
            self.keep(slot, pallet, None, 0);
        }
    }

    /// Orders whose time has come: pallets whose dwell time is over, and the orders of the interval.
    fn orders_due(&mut self, now: SimTime) {
        if !self.retrieves() {
            return;
        }
        while let Some(Reverse((t, id))) = self.due.peek().copied() {
            if t > now {
                break;
            }
            self.due.pop();
            if self.slot_of.contains_key(&id) {
                self.requests.push_back(id);
            }
        }
        while let Some(t) = self.next_order.filter(|t| *t <= now) {
            let pick = if self.data.fifo || self.unasked.is_empty() {
                self.unasked.first_key_value().map(|(seq, id)| (*seq, *id))
            } else {
                let n = ((self.rng.next_f64() * self.unasked.len() as f64) as usize).min(self.unasked.len() - 1);
                self.unasked.iter().nth(n).map(|(seq, id)| (*seq, *id))
            };
            match pick {
                Some((seq, id)) => {
                    self.unasked.remove(&seq);
                    self.requests.push_back(id);
                }
                None => self.stats.missed += 1,
            }
            self.next_order = Some(t + self.draw(self.data.retrieval_interval));
        }
    }

    /// The next job (S26 §4.1): the one a relocation was made for; else storage and retrieval in turn, oldest first.
    fn choose_job(&mut self, now: SimTime) -> Option<Job> {
        if let Some(job) = self.forced.take()
            && matches!(job.pick, Target::Slot(s) if self.slots.get(s).is_some_and(Option::is_some))
        {
            return Some(Job { started: now, ..job });
        }
        let store = self.storage_candidate(now);
        let retrieve_first = store.is_none() || self.last_kind == Some(JobKind::Store);
        if retrieve_first && let Some(job) = self.retrieval_candidate(now) {
            return Some(job);
        }
        let job = store?;
        if let Target::Slot(slot) = job.place {
            self.reserved[slot] = true;
        }
        Some(job)
    }

    /// The pallet that has waited longest at an entry, with the position it gets; none while the aisle is full.
    fn storage_candidate(&mut self, now: SimTime) -> Option<Job> {
        let (entry, _) =
            self.waiting.iter().enumerate().filter_map(|(e, w)| w.map(|(_, since)| (e, since))).min_by_key(|(e, since)| (*since, *e))?;
        let from = [STATION_X, self.entries.get(entry)?.height, 0.0];
        let slot = match self.data.strategy {
            Strategy::Nearest => self.nearest_free(from, None),
            Strategy::Random => self.random_free(),
        }?;
        Some(Job { kind: JobKind::Store, pick: Target::Entry(entry), place: Target::Slot(slot), started: now })
    }

    /// The oldest request as a job: the retrieval itself, or first the relocation of the pallet in front (S26 §4.4).
    fn retrieval_candidate(&mut self, now: SimTime) -> Option<Job> {
        if self.exit_ports.is_empty() {
            return None;
        }
        let slot = loop {
            let id = *self.requests.front()?;
            match self.slot_of.get(&id) {
                Some(slot) => break *slot,
                None => _ = self.requests.pop_front(),
            }
        };
        let exit = self.next_exit % self.exit_ports.len();
        let retrieve = |slot: usize| Job { kind: JobKind::Retrieve, pick: Target::Slot(slot), place: Target::Exit(exit), started: now };
        let front = slot - slot % self.data.deep;
        if front != slot && self.slots[front].is_some() {
            let Some(to) = self.nearest_free(self.data.place_of(front), Some(front / self.data.deep)) else {
                // Nowhere to put the front pallet: it leaves instead, and the request stays with the rear one.
                self.next_exit += 1;
                return Some(retrieve(front));
            };
            self.requests.pop_front();
            self.next_exit += 1;
            self.forced = Some(retrieve(slot));
            self.reserved[to] = true;
            return Some(Job { kind: JobKind::Relocate, pick: Target::Slot(front), place: Target::Slot(to), started: now });
        }
        self.requests.pop_front();
        self.next_exit += 1;
        Some(retrieve(slot))
    }

    /// The position of `channel` a pallet can be set down in: the rear one first; none when its front is taken.
    fn free_in(&self, channel: usize) -> Option<usize> {
        let front = channel * self.data.deep;
        let free = |s: usize| self.slots.get(s).is_some_and(Option::is_none) && !self.reserved[s];
        if !free(front) {
            return None;
        }
        Some(if self.data.deep == 2 && free(front + 1) { front + 1 } else { front })
    }

    /// The free position the crane reaches soonest from `from`; ties: lower bay, then lower level, then the left row.
    fn nearest_free(&self, from: [f64; 3], except_channel: Option<usize>) -> Option<usize> {
        let mut best: Option<(f64, usize)> = None;
        for channel in 0..self.data.positions() / self.data.deep {
            if Some(channel) == except_channel {
                continue;
            }
            let Some(slot) = self.free_in(channel) else { continue };
            let time = self.data.move_time(from, self.data.place_of(slot));
            if best.is_none_or(|(t, _)| time < t - 1e-9) {
                best = Some((time, slot));
            }
        }
        best.map(|(_, slot)| slot)
    }

    fn random_free(&mut self) -> Option<usize> {
        let free: Vec<usize> = (0..self.data.positions() / self.data.deep).filter_map(|c| self.free_in(c)).collect();
        if free.is_empty() {
            return None;
        }
        let n = ((self.rng.next_f64() * free.len() as f64) as usize).min(free.len() - 1);
        free.get(n).copied()
    }

    fn place_for(&self, target: Target) -> [f64; 3] {
        match target {
            Target::Entry(e) => [STATION_X, self.entries.get(e).map_or(self.crane[1], |p| p.height), 0.0],
            Target::Exit(e) => [STATION_X, self.exit_ports.get(e).map_or(self.crane[1], |p| p.height), 0.0],
            Target::Slot(s) => {
                let at = self.data.place_of(s);
                [at[0], at[1], 0.0]
            }
        }
    }

    /// How far across the forks reach at `target`, m: entries are on the left row, exits on the right.
    fn reach_for(&self, target: Target) -> f64 {
        match target {
            Target::Entry(_) => -self.data.front_reach(),
            Target::Exit(_) => self.data.front_reach(),
            Target::Slot(s) => self.data.place_of(s)[2],
        }
    }

    fn start_job(&mut self, job: Job, now: SimTime) {
        self.job = Some(job);
        self.asked = false;
        self.blocked = false;
        let to = self.place_for(job.pick);
        self.start_move(to, now);
        self.step = Step::ToPick;
    }

    fn start_move(&mut self, to: [f64; 3], now: SimTime) {
        let end = now + from_seconds(self.data.move_time(self.crane, to));
        self.motion = Some(Motion { start: now, end, from: self.crane, to });
        self.until = end;
    }

    /// The crane is where its move went.
    fn arrive(&mut self) {
        if let Some(m) = self.motion.take() {
            self.crane = m.to;
        }
    }

    /// One fork cycle at `target`: the pallet changes hands in its middle.
    fn start_fork(&mut self, target: Target, now: SimTime) {
        let slot = if let Target::Slot(s) = target { Some(s) } else { None };
        let cycle = from_seconds(self.data.fork_cycle(slot));
        self.fork = Some(Fork { start: now, end: now + cycle, reach: self.reach_for(target) });
        self.until = now + cycle / 2;
        self.asked = false;
    }

    /// Takes the pallet out of position `slot`.
    fn take_out(&mut self, slot: usize) -> Option<Pallet> {
        let stored = self.slots.get_mut(slot)?.take()?;
        self.slot_of.remove(&stored.pallet.id);
        self.unasked.remove(&stored.seq);
        self.view_stale = true;
        if matches!(self.job, Some(Job { kind: JobKind::Relocate, .. })) {
            self.seq_in_hand = Some(stored.seq);
        }
        Some(stored.pallet)
    }

    /// Sets the pallet on the forks down in position `slot`.
    fn put_in(&mut self, slot: usize, pallet: Pallet, kind: JobKind, now: SimTime) {
        self.reserved[slot] = false;
        let moved = if kind == JobKind::Relocate { self.seq_in_hand.take() } else { None };
        match kind {
            JobKind::Relocate => self.stats.relocations += 1,
            _ => self.stats.stored += 1,
        }
        self.keep(slot, pallet, moved, now);
    }

    /// The pallet stands in `slot`: a new one gets its number and its order to leave; a relocated one keeps both.
    fn keep(&mut self, slot: usize, pallet: Pallet, moved: Option<u64>, now: SimTime) {
        let id = pallet.id;
        let seq = moved.unwrap_or_else(|| {
            self.seq += 1;
            self.seq
        });
        if moved.is_none() && self.data.retrieval == Retrieval::Dwell {
            // The initial stock has been there for a while: what is left of its time is spread evenly.
            let stay = if now == 0 && !self.data.exponential {
                from_seconds(self.rng.next_f64() * self.data.dwell_time).max(1)
            } else {
                self.draw(self.data.dwell_time)
            };
            self.due.push(Reverse((now + stay, id)));
        }
        if self.data.retrieval == Retrieval::Interval && !self.requests.contains(&id) {
            self.unasked.insert(seq, id);
        }
        if let Some(place) = self.slots.get_mut(slot) {
            *place = Some(Stored { pallet, seq });
            self.slot_of.insert(id, slot);
        }
        self.view_stale = true;
    }

    fn finish_job(&mut self, job: Job, now: SimTime) {
        self.fork = None;
        self.job = None;
        self.blocked = false;
        self.stats.jobs += 1;
        self.stats.job_time += now - job.started;
        match job.kind {
            JobKind::Retrieve => {
                if self.last_kind == Some(JobKind::Store) {
                    self.stats.combined += 1;
                }
                self.last_kind = Some(JobKind::Retrieve);
            }
            JobKind::Store => self.last_kind = Some(JobKind::Store),
            JobKind::Relocate => {}
        }
    }

    /// Drops a job that can't go on (its pallet is gone); what it had reserved is free again.
    fn abandon(&mut self) {
        if let Some(Job { place: Target::Slot(slot), .. }) = self.job {
            self.reserved[slot] = false;
        }
        self.job = None;
        self.fork = None;
        self.asked = false;
        self.blocked = false;
    }
}

fn same_place(a: [f64; 3], b: [f64; 3]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

fn stored_view(p: &Pallet, at: [f64; 3]) -> StoredView {
    StoredView {
        id: p.id,
        type_id: p.type_id.clone(),
        source: p.source.clone(),
        length: p.length,
        width: p.width,
        height: p.height,
        load_height: p.load_height,
        at,
        wrapped: p.wrapped,
    }
}

#[cfg(test)]
mod tests;
