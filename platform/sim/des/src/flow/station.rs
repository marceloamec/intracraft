//! Stations of the 2D run (turntables, and later transfer tables, process stations and lifts): one or two decks that
//! convey like short conveyors, and timed steps (turn, lift, lower) between them. A port of the reference's `station.ts`
//! and `turntable.ts` (S03, S04, T13, ADR-0008).
//!
//! The decks are lines of the run; a station keeps their indices and changes them (where a deck ends, where it leads).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::flow::network::{ExitRule, TurntableData, TurntableExit};
use crate::flow::sim::{EPS, Line};
use crate::kinematics::{MoveProfile, profile_position_at, profile_time};
use crate::rng::Pcg32;
use crate::time::{SimTime, US_PER_S, from_seconds};

const FULL_TURN: f64 = 2.0 * PI;
const ANGLE_EPS: f64 = 1e-6;

/// Where a deck's path starts and which way it runs, in the station's own frame (m, rad).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DeckPath {
    pub x: f64,
    pub z: f64,
    pub heading: f64,
}

/// What a station did when asked to move on: whether it changed, and a timed step to finish after `wait` µs (0: at once).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StationStep {
    pub changed: bool,
    pub wait: Option<SimTime>,
}

/// Busy time and cycles of a station; a cycle runs from taking a pallet to being free again.
#[derive(Clone, Debug, Default)]
pub struct Counters {
    busy: f64,
    total: f64,
    cycles: u64,
    cycle_sum: SimTime,
    cycle_start: Option<SimTime>,
}

impl Counters {
    fn accumulate(&mut self, seconds: f64, busy: bool) {
        self.total += seconds;
        if busy {
            self.busy += seconds;
        }
    }

    fn start_cycle(&mut self, now: SimTime) {
        self.cycle_start = Some(now);
    }

    fn end_cycle(&mut self, now: SimTime) {
        if let Some(start) = self.cycle_start.take() {
            self.cycles += 1;
            self.cycle_sum += now - start;
        }
    }

    fn utilization(&self) -> f64 {
        if self.total > 0.0 { self.busy / self.total } else { 0.0 }
    }

    fn mean_cycle(&self) -> f64 {
        if self.cycles > 0 { self.cycle_sum as f64 / self.cycles as f64 / US_PER_S as f64 } else { 0.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Positioning,
    Receiving,
    Turning,
    Discharging,
}

#[derive(Clone, Copy, Debug)]
struct Turn {
    start: SimTime,
    /// The platform's angle at the start, and the signed rotation (rad).
    from: f64,
    rotation: f64,
    /// The deck's heading once the turn is over.
    heading: f64,
}

/// A chain or roller turntable: its platform's chains or rollers are a deck whose direction turns with it.
/// idle → [positioning: turn to face the entry] → receiving (the pallet stops centred) → turning to the exit + settle →
/// discharging → idle. The deck conveys both ways, so a station pair (k, k + 180°) needs no turn between them.
pub struct Turntable {
    pub id: String,
    pub data: TurntableData,
    /// The deck's index among the run's lines.
    pub deck: usize,
    pub phase: Phase,
    /// The direction the deck conveys towards (rad, the turntable's frame).
    heading: f64,
    exit: Option<TurntableExit>,
    entry_port: Option<String>,
    turns: u64,
    turn: Option<Turn>,
    alternate: usize,
    rng: Pcg32,
    counters: Counters,
}

impl Turntable {
    pub fn new(id: &str, data: TurntableData, deck: usize, seed: u64, stream: u64) -> Turntable {
        Turntable {
            id: id.to_string(),
            data,
            deck,
            phase: Phase::Idle,
            heading: 0.0,
            exit: None,
            entry_port: None,
            turns: 0,
            turn: None,
            alternate: 0,
            rng: Pcg32::new(seed, stream),
            counters: Counters::default(),
        }
    }

    /// The station angle of a port the platform may stop at.
    fn station_angle(&self, port: Option<&str>) -> Option<f64> {
        let angle = match port? {
            "P0" => 0.0,
            "P90" => FRAC_PI_2,
            "P180" => PI,
            "P270" => 3.0 * FRAC_PI_2,
            _ => return None,
        };
        self.data.stations.iter().any(|s| same_angle(*s, angle)).then_some(angle)
    }

    pub fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        self.station_angle(port).map(|_| self.deck)
    }

    pub fn busy_status(&self) -> Option<&'static str> {
        matches!(self.phase, Phase::Positioning | Phase::Turning).then_some("turning")
    }

    /// Free, empty and lined up with the entry: a pallet coming through `port` can enter now.
    pub fn can_receive(&self, port: Option<&str>, lines: &[Line]) -> bool {
        let Some(angle) = self.station_angle(port) else { return false };
        self.phase == Phase::Idle && lines[self.deck].pallets.is_empty() && same_line(self.heading, angle)
    }

    /// A pallet waits at `port`: if idle, empty and not lined up, turn to it; the turn's time, µs.
    pub fn prepare_for(&mut self, port: Option<&str>, now: SimTime, lines: &[Line]) -> Option<SimTime> {
        let angle = self.station_angle(port)?;
        if self.phase != Phase::Idle || !lines[self.deck].pallets.is_empty() || same_line(self.heading, angle) {
            return None;
        }
        self.phase = Phase::Positioning;
        Some(self.start_turn(normalize(angle + PI), now))
    }

    /// The pallet was taken through `port`: it will stop with its centre on the platform's centre.
    pub fn receive(&mut self, port: Option<&str>, pallet_length: f64, now: SimTime, lines: &mut [Line]) {
        self.counters.start_cycle(now);
        if let Some(angle) = self.station_angle(port) {
            self.heading = normalize(angle + PI);
        }
        self.phase = Phase::Receiving;
        self.entry_port = port.map(str::to_string);
        let deck = &mut lines[self.deck];
        deck.end = (self.data.platform_length + pallet_length) / 2.0;
        deck.downstream = None;
    }

    /// A centred pallet: choose the exit and turn to it. An emptied deck: free again.
    pub fn step(&mut self, now: SimTime, lines: &[Line]) -> StationStep {
        let deck = &lines[self.deck];
        let centred = deck.pallets.first().is_some_and(|p| deck.incoming.is_none() && deck.holding && p.front >= deck.end - EPS);
        if self.phase == Phase::Receiving && centred {
            let Some(exit) = self.choose_exit() else { return StationStep::default() };
            let angle = exit.angle;
            self.exit = Some(exit);
            self.phase = Phase::Turning;
            return StationStep { changed: true, wait: Some(self.start_turn(angle, now)) };
        }
        if self.phase == Phase::Discharging && deck.pallets.is_empty() {
            self.phase = Phase::Idle;
            self.exit = None;
            self.entry_port = None;
            self.counters.end_cycle(now);
            return StationStep { changed: true, wait: None };
        }
        StationStep::default()
    }

    /// The platform reached its station and settled.
    pub fn finish_timed_step(&mut self, lines: &mut [Line]) {
        if let Some(t) = self.turn.take() {
            self.heading = t.heading;
        }
        match self.phase {
            Phase::Positioning => self.phase = Phase::Idle,
            Phase::Turning => self.start_discharge(lines),
            _ => {}
        }
    }

    /// The deck's heading at `t` µs, following the angular profile during a turn.
    pub fn angle_at(&self, t: SimTime) -> f64 {
        let Some(turn) = self.turn else { return self.heading };
        let elapsed = (t - turn.start) as f64 / US_PER_S as f64;
        let covered = profile_position_at(&self.turn_profile(turn.rotation.abs()), elapsed).unwrap_or(turn.rotation.abs());
        turn.from + turn.rotation.signum() * covered
    }

    pub fn path_at(&self, now: SimTime) -> DeckPath {
        let heading = self.angle_at(now);
        let half = self.data.platform_length / 2.0;
        // The entry edge: the centre − half · direction(heading), direction(h) = (cos h, −sin h).
        DeckPath { x: -half * heading.cos(), z: half * heading.sin(), heading }
    }

    pub fn accumulate(&mut self, seconds: f64) {
        self.counters.accumulate(seconds, self.phase != Phase::Idle);
    }

    pub fn statistics(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("utilization", self.counters.utilization()),
            ("turns", self.turns as f64),
            ("meanCycleTime", self.counters.mean_cycle()),
            ("palletsProcessed", self.counters.cycles as f64),
        ]
    }

    fn start_discharge(&mut self, lines: &mut [Line]) {
        let Some(exit) = self.exit.clone() else { return };
        let deck = &mut lines[self.deck];
        let length = self.data.platform_length;
        let Some(p) = deck.pallets.first_mut() else { return };
        self.phase = Phase::Discharging;
        // Seen from the new heading the centred pallet is centred on the deck again.
        p.front = (length + p.length) / 2.0;
        deck.end = length;
        deck.holding = false;
        deck.downstream = Some(exit.target);
        // As the reference's deck, it names no port of the next object.
        deck.downstream_port = None;
        deck.gap = exit.gap;
    }

    /// Turns the shortest way to the line of `target` (the deck runs both ways); the time with settling, µs.
    fn start_turn(&mut self, target: f64, now: SimTime) -> SimTime {
        let mut rotation = normalize(target - self.heading);
        if rotation > PI {
            rotation -= FULL_TURN;
        }
        if rotation.abs() > FRAC_PI_2 + ANGLE_EPS {
            rotation -= rotation.signum() * PI;
        }
        if rotation.abs() < ANGLE_EPS {
            self.heading = target;
            return 0;
        }
        self.turns += 1;
        self.turn = Some(Turn { start: now, from: self.heading, rotation, heading: target });
        from_seconds(profile_time(&self.turn_profile(rotation.abs())).unwrap_or(0.0) + self.data.settle_time)
    }

    fn turn_profile(&self, angle: f64) -> MoveProfile {
        MoveProfile {
            distance: angle,
            max_speed: self.data.max_angular_speed,
            accel: self.data.angular_accel,
            decel: Some(self.data.angular_decel),
        }
    }

    /// The exits other than the entry, clockwise seen from above from the entry on.
    fn choose_exit(&mut self) -> Option<TurntableExit> {
        let entry = self.station_angle(self.entry_port.as_deref()).unwrap_or(0.0);
        let mut candidates: Vec<TurntableExit> = self
            .data
            .exits
            .iter()
            .filter(|e| Some(e.port_id.as_str()) != self.entry_port.as_deref() && self.station_angle(Some(&e.port_id)).is_some())
            .cloned()
            .collect();
        // A stable sort, as the reference's.
        candidates.sort_by(|a, b| clockwise_from(entry, a.angle).total_cmp(&clockwise_from(entry, b.angle)));
        if candidates.is_empty() {
            return None;
        }
        let k = match self.data.exit_rule {
            ExitRule::FirstExit => 0,
            ExitRule::Random => ((self.rng.next_f64() * candidates.len() as f64).floor() as usize).min(candidates.len() - 1),
            ExitRule::Alternate => {
                let k = self.alternate % candidates.len();
                self.alternate += 1;
                k
            }
        };
        Some(candidates.swap_remove(k))
    }
}

fn normalize(angle: f64) -> f64 {
    ((angle % FULL_TURN) + FULL_TURN) % FULL_TURN
}

/// Both angles lie on one line through the platform's centre (equal or opposite).
fn same_line(a: f64, b: f64) -> bool {
    same_angle(a, b) || same_angle(a, b + PI)
}

fn same_angle(a: f64, b: f64) -> bool {
    let d = normalize(a - b);
    d < ANGLE_EPS || FULL_TURN - d < ANGLE_EPS
}

/// The clockwise angle from `from` to `to` seen from above (positive angles turn counter-clockwise).
fn clockwise_from(from: f64, to: f64) -> f64 {
    normalize(from - to)
}

/// A station of the run.
pub enum Station {
    Turntable(Box<Turntable>),
}

impl Station {
    pub fn id(&self) -> &str {
        match self {
            Station::Turntable(t) => &t.id,
        }
    }

    pub fn decks(&self) -> Vec<usize> {
        match self {
            Station::Turntable(t) => vec![t.deck],
        }
    }

    pub fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        match self {
            Station::Turntable(t) => t.entry_deck(port),
        }
    }

    pub fn can_receive(&self, port: Option<&str>, lines: &[Line]) -> bool {
        match self {
            Station::Turntable(t) => t.can_receive(port, lines),
        }
    }

    pub fn prepare_for(&mut self, port: Option<&str>, now: SimTime, lines: &[Line]) -> Option<SimTime> {
        match self {
            Station::Turntable(t) => t.prepare_for(port, now, lines),
        }
    }

    pub fn receive(&mut self, port: Option<&str>, pallet_length: f64, now: SimTime, lines: &mut [Line]) {
        match self {
            Station::Turntable(t) => t.receive(port, pallet_length, now, lines),
        }
    }

    pub fn step(&mut self, now: SimTime, lines: &[Line]) -> StationStep {
        match self {
            Station::Turntable(t) => t.step(now, lines),
        }
    }

    pub fn finish_timed_step(&mut self, lines: &mut [Line]) {
        match self {
            Station::Turntable(t) => t.finish_timed_step(lines),
        }
    }

    pub fn busy_status(&self) -> Option<&'static str> {
        match self {
            Station::Turntable(t) => t.busy_status(),
        }
    }

    pub fn path_of(&self, _deck: usize, now: SimTime) -> DeckPath {
        match self {
            Station::Turntable(t) => t.path_at(now),
        }
    }

    /// The deck's heading, where it turns (a turntable's platform), for the snapshot.
    pub fn angle_at(&self, now: SimTime) -> Option<f64> {
        match self {
            Station::Turntable(t) => Some(t.angle_at(now)),
        }
    }

    pub fn accumulate(&mut self, seconds: f64) {
        match self {
            Station::Turntable(t) => t.accumulate(seconds),
        }
    }

    pub fn statistics(&self) -> Vec<(&'static str, f64)> {
        match self {
            Station::Turntable(t) => t.statistics(),
        }
    }
}
