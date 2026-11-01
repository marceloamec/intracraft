//! Stations of the 2D run (turntables, and later transfer tables, process stations and lifts): one or two decks that
//! convey like short conveyors, and timed steps (turn, lift, lower) between them. A port of the reference's `station.ts`
//! and `turntable.ts` (S03, S04, T13, ADR-0008).
//!
//! The decks are lines of the run; a station keeps their indices and changes them (where a deck ends, where it leads).

use std::f64::consts::{FRAC_PI_2, PI};

use crate::flow::network::{
    ExitRule, LiftData, LiftPort, OperationMode, ProcessData, ProcessingMode, Side, TransferData, TransferDirection, TurntableData,
    TurntableExit,
};
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

/// The chains' heading when they run towards C2 (+z) or C1 (−z); direction(h) = (cos h, −sin h).
const TOWARDS_C2: f64 = -FRAC_PI_2;
const TOWARDS_C1: f64 = FRAC_PI_2;
const CHAIN_PORTS: [&str; 2] = ["C1", "C2"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferPhase {
    Idle,
    Lifting,
    Lowering,
    Receiving,
    Discharging,
}

/// What happens when the running lift or lowering ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum After {
    Ready,
    CarryOnChains,
    CarryOnRollers,
    CycleDone,
}

/// The roller-to-chain transfer table (S05, T13, ADR-0009): two decks, rollers along x (A → B) and liftable chains
/// across (C1 at −z, C2 at +z). The pallet changes direction without turning, so its side along travel swaps.
/// From A: receive on the rollers, centre → lift → the chains carry it out through C1/C2 → lower → idle.
/// From C1/C2 (chains up): receive on the chains, centre → lower → the rollers carry it out through B → idle.
pub struct Transfer {
    pub id: String,
    pub data: TransferData,
    pub rollers: usize,
    pub chains: usize,
    pub phase: TransferPhase,
    chains_up: bool,
    transfers: u64,
    chain_heading: f64,
    after: After,
    exit_port: Option<String>,
    alternate: usize,
    counters: Counters,
}

impl Transfer {
    pub fn new(id: &str, data: TransferData, rollers: usize, chains: usize) -> Transfer {
        Transfer {
            id: id.to_string(),
            data,
            rollers,
            chains,
            phase: TransferPhase::Idle,
            chains_up: false,
            transfers: 0,
            chain_heading: TOWARDS_C2,
            after: After::Ready,
            exit_port: None,
            alternate: 0,
            counters: Counters::default(),
        }
    }

    pub fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        match port {
            Some("A") if self.data.operation_mode != OperationMode::ChainsToRollers => Some(self.rollers),
            Some(p) if CHAIN_PORTS.contains(&p) && self.data.operation_mode != OperationMode::RollersToChains => Some(self.chains),
            _ => None,
        }
    }

    fn busy_status(&self) -> Option<&'static str> {
        match self.phase {
            TransferPhase::Lifting => Some("lifting"),
            TransferPhase::Lowering => Some("lowering"),
            _ => None,
        }
    }

    fn is_free(&self, lines: &[Line]) -> bool {
        self.phase == TransferPhase::Idle && lines[self.rollers].pallets.is_empty() && lines[self.chains].pallets.is_empty()
    }

    fn can_receive(&self, port: Option<&str>, lines: &[Line]) -> bool {
        let Some(deck) = self.entry_deck(port) else { return false };
        self.is_free(lines) && (if deck == self.chains { self.chains_up } else { !self.chains_up })
    }

    /// Lifts the chains for a pallet coming in on them, or lowers them for one coming in on the rollers.
    fn prepare_for(&mut self, port: Option<&str>, lines: &[Line]) -> Option<SimTime> {
        let deck = self.entry_deck(port)?;
        if !self.is_free(lines) || (deck == self.chains) == self.chains_up {
            return None;
        }
        self.after = After::Ready;
        Some(self.start_lift(deck == self.chains))
    }

    fn receive(&mut self, port: Option<&str>, pallet_length: f64, now: SimTime, lines: &mut [Line]) {
        self.counters.start_cycle(now);
        self.phase = TransferPhase::Receiving;
        let deck = self.entry_deck(port).unwrap_or(self.rollers);
        if deck == self.chains {
            self.chain_heading = if port == Some("C1") { TOWARDS_C2 } else { TOWARDS_C1 };
        }
        let d = &mut lines[deck];
        d.end = (d.length() + pallet_length) / 2.0;
        d.downstream = None;
    }

    fn step(&mut self, now: SimTime, lines: &[Line]) -> StationStep {
        if self.phase == TransferPhase::Receiving {
            if is_centred(&lines[self.rollers]) {
                self.exit_port = self.choose_chain_exit();
                if self.exit_port.is_none() {
                    return StationStep::default();
                }
                self.after = After::CarryOnChains;
                return StationStep { changed: true, wait: Some(self.start_lift(true)) };
            }
            if is_centred(&lines[self.chains]) {
                if !self.data.exits.contains_key("B") {
                    return StationStep::default();
                }
                self.exit_port = Some("B".into());
                self.after = After::CarryOnRollers;
                return StationStep { changed: true, wait: Some(self.start_lift(false)) };
            }
        }
        if self.phase == TransferPhase::Discharging && lines[self.rollers].pallets.is_empty() && lines[self.chains].pallets.is_empty() {
            self.transfers += 1;
            if self.chains_up {
                // The chains stay up until the pallet has left them, then go down.
                self.after = After::CycleDone;
                return StationStep { changed: true, wait: Some(self.start_lift(false)) };
            }
            self.finish_cycle(now);
            return StationStep { changed: true, wait: None };
        }
        StationStep::default()
    }

    fn finish_timed_step(&mut self, now: SimTime, lines: &mut [Line]) {
        self.chains_up = self.phase == TransferPhase::Lifting;
        match self.after {
            After::Ready => self.phase = TransferPhase::Idle,
            After::CarryOnChains => {
                let port = self.exit_port.clone().unwrap_or_else(|| "C2".into());
                self.hand_over(self.rollers, self.chains, &port, lines);
            }
            After::CarryOnRollers => self.hand_over(self.chains, self.rollers, "B", lines),
            After::CycleDone => self.finish_cycle(now),
        }
    }

    fn path_of(&self, deck: usize) -> DeckPath {
        if deck == self.rollers {
            return DeckPath::default();
        }
        let half = self.data.usable_width / 2.0;
        let towards_c2 = self.chain_heading == TOWARDS_C2;
        DeckPath { x: self.data.length / 2.0, z: if towards_c2 { -half } else { half }, heading: self.chain_heading }
    }

    fn statistics(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("utilization", self.counters.utilization()),
            ("transfers", self.transfers as f64),
            ("meanCycleTime", self.counters.mean_cycle()),
            ("palletsProcessed", self.counters.cycles as f64),
        ]
    }

    /// Starts lifting (true) or lowering (false) the chains; how long it takes, µs.
    fn start_lift(&mut self, up: bool) -> SimTime {
        self.phase = if up { TransferPhase::Lifting } else { TransferPhase::Lowering };
        from_seconds(if up { self.data.lift_time } else { self.data.lower_time })
    }

    /// Moves the centred pallet to the other deck (its sides along travel swap) and lets it run out.
    fn hand_over(&mut self, from: usize, to: usize, exit_port: &str, lines: &mut [Line]) {
        let Some(exit) = self.data.exits.get(exit_port).cloned() else { return };
        if lines[from].pallets.is_empty() {
            return;
        }
        let mut p = lines[from].pallets.remove(0);
        if to == self.chains {
            self.chain_heading = if exit_port == "C1" { TOWARDS_C1 } else { TOWARDS_C2 };
        }
        p.along = if p.along == Side::Long { Side::Short } else { Side::Long };
        std::mem::swap(&mut p.length, &mut p.width);
        p.front = (lines[to].length() + p.length) / 2.0;
        let f = &mut lines[from];
        f.holding = false;
        f.end = f.length();
        let t = &mut lines[to];
        t.pallets.push(p);
        t.end = t.length();
        t.holding = false;
        t.downstream = Some(exit.target);
        t.downstream_port = None;
        t.gap = exit.gap;
        self.phase = TransferPhase::Discharging;
    }

    fn finish_cycle(&mut self, now: SimTime) {
        self.phase = TransferPhase::Idle;
        self.exit_port = None;
        self.counters.end_cycle(now);
    }

    /// right → C2 (+z), left → C1 (−z), bidirectional → the connected chain exits in turn.
    fn choose_chain_exit(&mut self) -> Option<String> {
        let connected: Vec<&str> = CHAIN_PORTS.iter().copied().filter(|p| self.data.exits.contains_key(*p)).collect();
        let wanted: Vec<&str> = match self.data.transfer_direction {
            TransferDirection::Right => vec!["C2"],
            TransferDirection::Left => vec!["C1"],
            TransferDirection::Bidirectional => connected.clone(),
        };
        let candidates: Vec<&str> = wanted.into_iter().filter(|p| connected.contains(p)).collect();
        if candidates.is_empty() {
            return None;
        }
        let k = self.alternate % candidates.len();
        self.alternate += 1;
        Some(candidates[k].to_string())
    }
}

/// The deck holds its only pallet stopped with its front at the stop position (centred).
fn is_centred(deck: &Line) -> bool {
    deck.pallets.first().is_some_and(|p| deck.incoming.is_none() && deck.holding && p.front >= deck.end - EPS)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessPhase {
    Idle,
    Receiving,
    Processing,
    Discharging,
}

/// The process station (S11): a short deck where one pallet at a time stops and is processed for a time.
/// idle → receiving (the pallet stops centred) → processing (a timed step) → discharging (the deck runs it out through
/// OUT; blocked while the next object is full) → idle.
pub struct ProcessStation {
    pub id: String,
    pub data: ProcessData,
    pub deck: usize,
    pub phase: ProcessPhase,
    /// Where OUT leads: the instance, the gap to it and its port.
    output: (Option<String>, f64, Option<String>),
    rng: Pcg32,
    counters: Counters,
    working: f64,
    blocked: f64,
    total: f64,
    processed: u64,
    processing_sum: SimTime,
}

impl ProcessStation {
    pub fn new(
        id: &str,
        data: ProcessData,
        deck: usize,
        output: (Option<String>, f64, Option<String>),
        seed: u64,
        stream: u64,
    ) -> ProcessStation {
        ProcessStation {
            id: id.to_string(),
            data,
            deck,
            phase: ProcessPhase::Idle,
            output,
            rng: Pcg32::new(seed, stream),
            counters: Counters::default(),
            working: 0.0,
            blocked: 0.0,
            total: 0.0,
            processed: 0,
            processing_sum: 0,
        }
    }

    fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        (port == Some("IN")).then_some(self.deck)
    }

    fn can_receive(&self, port: Option<&str>, lines: &[Line]) -> bool {
        self.entry_deck(port).is_some() && self.phase == ProcessPhase::Idle && lines[self.deck].pallets.is_empty()
    }

    fn receive(&mut self, pallet_length: f64, now: SimTime, lines: &mut [Line]) {
        self.counters.start_cycle(now);
        self.phase = ProcessPhase::Receiving;
        let d = &mut lines[self.deck];
        d.end = (self.data.length + pallet_length) / 2.0;
        d.downstream = None;
    }

    fn step(&mut self, now: SimTime, lines: &[Line]) -> StationStep {
        if self.phase == ProcessPhase::Receiving && is_centred(&lines[self.deck]) {
            self.phase = ProcessPhase::Processing;
            return StationStep { changed: true, wait: Some(self.draw_processing_time()) };
        }
        if self.phase == ProcessPhase::Discharging && lines[self.deck].pallets.is_empty() {
            self.phase = ProcessPhase::Idle;
            self.counters.end_cycle(now);
            return StationStep { changed: true, wait: None };
        }
        StationStep::default()
    }

    fn finish_timed_step(&mut self, lines: &mut [Line]) {
        self.processed += 1;
        // The processed pallet may now run out through OUT.
        let d = &mut lines[self.deck];
        d.end = self.data.length;
        d.holding = false;
        if let Some(target) = &self.output.0 {
            d.downstream = Some(target.clone());
            d.gap = self.output.1;
            if self.output.2.is_some() {
                d.downstream_port = self.output.2.clone();
            }
        }
        self.phase = ProcessPhase::Discharging;
    }

    fn accumulate(&mut self, seconds: f64, lines: &[Line]) {
        self.total += seconds;
        if self.phase == ProcessPhase::Processing {
            self.working += seconds;
        } else if self.phase == ProcessPhase::Discharging && lines[self.deck].is_blocked() {
            self.blocked += seconds;
        }
    }

    fn statistics(&self) -> Vec<(&'static str, f64)> {
        let share = |s: f64| if self.total > 0.0 { s / self.total } else { 0.0 };
        vec![
            ("utilization", share(self.working)),
            ("blockedShare", share(self.blocked)),
            ("idleShare", share(self.total - self.working - self.blocked)),
            ("palletsProcessed", self.processed as f64),
            (
                "meanProcessingTime",
                if self.processed > 0 { self.processing_sum as f64 / self.processed as f64 / US_PER_S as f64 } else { 0.0 },
            ),
            ("meanCycleTime", self.counters.mean_cycle()),
        ]
    }

    /// The next pallet's processing time, µs: fixed, normal (cut at 0) or exponential about the mean.
    fn draw_processing_time(&mut self) -> SimTime {
        let mean = self.data.processing_time;
        let seconds = match self.data.mode {
            ProcessingMode::Fixed => mean,
            // Box–Muller, drawing in the reference's order.
            ProcessingMode::Normal => {
                let r = (-2.0 * (1.0 - self.rng.next_f64()).ln()).sqrt();
                mean + self.data.processing_std_dev * r * (2.0 * PI * self.rng.next_f64()).cos()
            }
            ProcessingMode::Exponential => -mean * (1.0 - self.rng.next_f64()).ln(),
        };
        let time = from_seconds(seconds.max(0.0));
        self.processing_sum += time;
        time
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiftPhase {
    Idle,
    Receiving,
    Moving,
    Discharging,
    Returning,
}

#[derive(Clone, Copy, Debug)]
struct VerticalMove {
    start: SimTime,
    from: f64,
    to: f64,
    profile: MoveProfile,
}

/// The exit a lift uses: the first connected one in the definition's order of preference.
#[derive(Clone, Debug)]
struct LiftExit {
    port: LiftPort,
    target: String,
    gap: f64,
    target_port: String,
}

/// The manual lift table, the scissor lift table and the pallet elevator (S12–S14, ADR-0013): one deck takes a pallet
/// at the entry height, the station moves it to the exit height, the deck runs it out and the station goes back empty.
/// idle → receiving (the pallet stops centred) → moving (a timed step) → discharging (blocked while the next object is
/// full) → returning (a timed step) → idle. The deck conveys both ways: a pallet that leaves on the side it came in runs
/// out backwards without turning.
pub struct LiftStation {
    pub id: String,
    pub data: LiftData,
    pub deck: usize,
    pub phase: LiftPhase,
    height: f64,
    vertical: Option<VerticalMove>,
    exit: Option<LiftExit>,
    counters: Counters,
    busy: f64,
    blocked: f64,
    total: f64,
    moved: u64,
}

impl LiftStation {
    pub fn new(id: &str, data: LiftData, deck: usize) -> LiftStation {
        let exit = data.exit_ports.iter().find_map(|port| {
            data.exits.get(&port.port_id).map(|e| LiftExit {
                port: port.clone(),
                target: e.target.clone(),
                gap: e.gap,
                target_port: e.port.clone(),
            })
        });
        LiftStation {
            id: id.to_string(),
            height: data.entry.height,
            data,
            deck,
            phase: LiftPhase::Idle,
            vertical: None,
            exit,
            counters: Counters::default(),
            busy: 0.0,
            blocked: 0.0,
            total: 0.0,
            moved: 0,
        }
    }

    fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        (port == Some(self.data.entry.port_id.as_str())).then_some(self.deck)
    }

    fn busy_status(&self) -> Option<&'static str> {
        self.vertical.map(|m| if m.to > m.from { "lifting" } else { "lowering" })
    }

    fn can_receive(&self, port: Option<&str>, lines: &[Line]) -> bool {
        self.entry_deck(port).is_some() && self.phase == LiftPhase::Idle && lines[self.deck].pallets.is_empty()
    }

    fn receive(&mut self, pallet_length: f64, now: SimTime, lines: &mut [Line]) {
        self.counters.start_cycle(now);
        self.phase = LiftPhase::Receiving;
        let d = &mut lines[self.deck];
        d.end = (self.data.length + pallet_length) / 2.0;
        d.downstream = None;
    }

    /// A centred pallet: move to the exit height. An emptied deck: go back to the entry height.
    fn step(&mut self, now: SimTime, lines: &[Line]) -> StationStep {
        if self.phase == LiftPhase::Receiving && is_centred(&lines[self.deck]) {
            self.phase = LiftPhase::Moving;
            let to = self.exit.as_ref().map_or(self.height, |e| e.port.height);
            return StationStep { changed: true, wait: Some(self.start_move(to, now)) };
        }
        if self.phase == LiftPhase::Discharging && self.exit.is_some() && lines[self.deck].pallets.is_empty() {
            self.moved += 1;
            self.phase = LiftPhase::Returning;
            let to = self.data.entry.height;
            return StationStep { changed: true, wait: Some(self.start_move(to, now)) };
        }
        StationStep::default()
    }

    fn finish_timed_step(&mut self, now: SimTime, lines: &mut [Line]) {
        if let Some(m) = self.vertical.take() {
            self.height = m.to;
        }
        match self.phase {
            LiftPhase::Moving => {
                self.phase = LiftPhase::Discharging;
                // Seen from the exit side the centred pallet is still centred: only the direction changes.
                if let Some(exit) = &self.exit {
                    let d = &mut lines[self.deck];
                    d.end = self.data.length;
                    d.holding = false;
                    d.downstream = Some(exit.target.clone());
                    d.gap = exit.gap;
                    d.downstream_port = Some(exit.target_port.clone());
                }
            }
            LiftPhase::Returning => {
                self.phase = LiftPhase::Idle;
                self.counters.end_cycle(now);
            }
            _ => {}
        }
    }

    /// The deck's height at `t` µs, following the vertical profile during a move, m.
    pub fn height_at(&self, t: SimTime) -> f64 {
        let Some(m) = self.vertical else { return self.height };
        let covered = profile_position_at(&m.profile, (t - m.start) as f64 / US_PER_S as f64).unwrap_or(m.profile.distance);
        m.from + (m.to - m.from).signum() * covered
    }

    /// The deck runs from the entry end; while discharging, towards the exit end.
    fn path_of(&self) -> DeckPath {
        let from_end = match (&self.exit, self.phase) {
            (Some(e), LiftPhase::Discharging) => !e.port.at_end,
            _ => self.data.entry.at_end,
        };
        DeckPath { x: if from_end { self.data.length } else { 0.0 }, z: 0.0, heading: if from_end { PI } else { 0.0 } }
    }

    fn accumulate(&mut self, seconds: f64, lines: &[Line]) {
        self.total += seconds;
        if self.phase == LiftPhase::Idle {
            return;
        }
        if self.phase == LiftPhase::Discharging && lines[self.deck].is_blocked() {
            self.blocked += seconds;
        } else {
            self.busy += seconds;
        }
    }

    fn statistics(&self) -> Vec<(&'static str, f64)> {
        let share = |s: f64| if self.total > 0.0 { s / self.total } else { 0.0 };
        vec![
            ("utilization", share(self.busy)),
            ("blockedShare", share(self.blocked)),
            ("idleShare", share(self.total - self.busy - self.blocked)),
            ("palletsMoved", self.moved as f64),
            ("meanCycleTime", self.counters.mean_cycle()),
        ]
    }

    /// Starts a vertical move to `to`, m; its time, µs (0 when already there).
    fn start_move(&mut self, to: f64, now: SimTime) -> SimTime {
        let distance = (to - self.height).abs();
        if distance < EPS {
            return 0;
        }
        let up = to > self.height;
        let speed = if up { self.data.up_speed } else { self.data.down_speed };
        let profile = MoveProfile { distance, max_speed: speed, accel: self.data.vertical_accel, decel: Some(self.data.vertical_accel) };
        self.vertical = Some(VerticalMove { start: now, from: self.height, to, profile });
        from_seconds(profile_time(&profile).unwrap_or(0.0))
    }
}

/// A station of the run.
pub enum Station {
    Turntable(Box<Turntable>),
    Transfer(Box<Transfer>),
    Process(Box<ProcessStation>),
    Lift(Box<LiftStation>),
}

impl Station {
    pub fn id(&self) -> &str {
        match self {
            Station::Turntable(t) => &t.id,
            Station::Transfer(t) => &t.id,
            Station::Process(t) => &t.id,
            Station::Lift(t) => &t.id,
        }
    }

    pub fn decks(&self) -> Vec<usize> {
        match self {
            Station::Turntable(t) => vec![t.deck],
            Station::Transfer(t) => vec![t.rollers, t.chains],
            Station::Process(t) => vec![t.deck],
            Station::Lift(t) => vec![t.deck],
        }
    }

    pub fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        match self {
            Station::Turntable(t) => t.entry_deck(port),
            Station::Transfer(t) => t.entry_deck(port),
            Station::Process(t) => t.entry_deck(port),
            Station::Lift(t) => t.entry_deck(port),
        }
    }

    pub fn can_receive(&self, port: Option<&str>, lines: &[Line]) -> bool {
        match self {
            Station::Turntable(t) => t.can_receive(port, lines),
            Station::Transfer(t) => t.can_receive(port, lines),
            Station::Process(t) => t.can_receive(port, lines),
            Station::Lift(t) => t.can_receive(port, lines),
        }
    }

    pub fn prepare_for(&mut self, port: Option<&str>, now: SimTime, lines: &[Line]) -> Option<SimTime> {
        match self {
            Station::Turntable(t) => t.prepare_for(port, now, lines),
            Station::Transfer(t) => t.prepare_for(port, lines),
            Station::Process(_) | Station::Lift(_) => None,
        }
    }

    pub fn receive(&mut self, port: Option<&str>, pallet_length: f64, now: SimTime, lines: &mut [Line]) {
        match self {
            Station::Turntable(t) => t.receive(port, pallet_length, now, lines),
            Station::Transfer(t) => t.receive(port, pallet_length, now, lines),
            Station::Process(t) => t.receive(pallet_length, now, lines),
            Station::Lift(t) => t.receive(pallet_length, now, lines),
        }
    }

    pub fn step(&mut self, now: SimTime, lines: &[Line]) -> StationStep {
        match self {
            Station::Turntable(t) => t.step(now, lines),
            Station::Transfer(t) => t.step(now, lines),
            Station::Process(t) => t.step(now, lines),
            Station::Lift(t) => t.step(now, lines),
        }
    }

    pub fn finish_timed_step(&mut self, now: SimTime, lines: &mut [Line]) {
        match self {
            Station::Turntable(t) => t.finish_timed_step(lines),
            Station::Transfer(t) => t.finish_timed_step(now, lines),
            Station::Process(t) => t.finish_timed_step(lines),
            Station::Lift(t) => t.finish_timed_step(now, lines),
        }
    }

    pub fn busy_status(&self) -> Option<&'static str> {
        match self {
            Station::Turntable(t) => t.busy_status(),
            Station::Transfer(t) => t.busy_status(),
            Station::Process(t) => (t.phase == ProcessPhase::Processing).then_some("processing"),
            Station::Lift(t) => t.busy_status(),
        }
    }

    pub fn path_of(&self, _deck: usize, now: SimTime) -> DeckPath {
        match self {
            Station::Turntable(t) => t.path_at(now),
            Station::Transfer(t) => t.path_of(_deck),
            Station::Process(_) => DeckPath::default(),
            Station::Lift(t) => t.path_of(),
        }
    }

    /// The deck's height above its base, where it moves up and down (a lift), for the snapshot, m.
    pub fn height_at(&self, now: SimTime) -> Option<f64> {
        match self {
            Station::Lift(t) => Some(t.height_at(now)),
            _ => None,
        }
    }

    /// The deck's heading, where it turns (a turntable's platform), for the snapshot.
    pub fn angle_at(&self, now: SimTime) -> Option<f64> {
        match self {
            Station::Turntable(t) => Some(t.angle_at(now)),
            Station::Transfer(_) => None,
            Station::Process(_) | Station::Lift(_) => None,
        }
    }

    pub fn accumulate(&mut self, seconds: f64, lines: &[Line]) {
        match self {
            Station::Turntable(t) => t.accumulate(seconds),
            Station::Transfer(t) => t.counters.accumulate(seconds, t.phase != TransferPhase::Idle),
            Station::Process(t) => t.accumulate(seconds, lines),
            Station::Lift(t) => t.accumulate(seconds, lines),
        }
    }

    pub fn statistics(&self) -> Vec<(&'static str, f64)> {
        match self {
            Station::Turntable(t) => t.statistics(),
            Station::Transfer(t) => t.statistics(),
            Station::Process(t) => t.statistics(),
            Station::Lift(t) => t.statistics(),
        }
    }
}
