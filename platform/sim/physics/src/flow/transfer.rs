//! The 3D behaviour of the roller-to-chain transfer table (S05, T13 §2): the same cycle as the 2D table (ADR-0009) with
//! two decks. Rollers along its length turn as a roller conveyor's; between them, chain strands across it on one
//! kinematic body that rises `liftStroke` above the rollers in `liftTime`, lifting the pallet by contact, carries it
//! sideways without turning it, and goes down again in `lowerTime`.
//!
//! From A: receive on the rollers, centre → lift → the chains carry it out through C1 or C2 → lower → idle.
//! From C1 or C2 (chains up): receive on the chains, centre → lower → the rollers carry it out through B → idle.

use std::collections::BTreeMap;
use std::f64::consts::FRAC_PI_2;

use intracraft_des::flow::network::{OperationMode, TransferData, TransferDirection};
use intracraft_des::time::{SimTime, US_PER_S};
use intracraft_library::ports::rotate_y;
use rapier3d::prelude::RigidBodyHandle;

use crate::bench::{Bench, DT};
use crate::flow::deck::{Deck, Exit, Pallets};
use crate::frame::{Frame, V3};

/// The chains' heading when they run toward C2 (+z) or C1 (−z); direction(h) = (cos h, −sin h).
const TOWARDS_C2: f64 = -FRAC_PI_2;
const TOWARDS_C1: f64 = FRAC_PI_2;
const CHAIN_PORTS: [&str; 2] = ["C1", "C2"];
/// A centred pallet slower than this has come to rest on a deck, m/s.
const REST_SPEED: f64 = 0.005;
/// The pallet's rear may still hang this far over the entry edge when it is centred, m.
const ENTRY_TOLERANCE: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Lifting,
    Lowering,
    Receiving,
    Discharging,
}

/// What happens when the chains' move ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum After {
    Ready,
    CarryOnChains,
    CarryOnRollers,
    CycleDone,
}

/// The chains' vertical move: from one level to the other at a steady speed; times in s.
#[derive(Clone, Copy, Debug)]
struct Move {
    from: f64,
    to: f64,
    start: f64,
    duration: f64,
}

#[derive(Clone, Debug, Default)]
struct Stats {
    busy: f64,
    total: f64,
    transfers: u64,
    cycles: u64,
    cycle_sum: SimTime,
    cycle_start: Option<SimTime>,
}

pub struct Transfer3D {
    pub id: String,
    /// Its two decks among the run's.
    pub rollers: usize,
    pub chains: usize,
    /// Where B, C1 and C2 lead.
    pub exits: BTreeMap<String, Exit>,
    data: TransferData,
    /// How far below the rollers' surface the lowered chains' tops lie, m (`lowerClearance`).
    clearance: f64,
    /// The table's middle on the rollers' conveying surface, and the instance's turn about +y.
    centre: V3,
    yaw: f64,
    chain_body: RigidBodyHandle,
    phase: Phase,
    after: After,
    chains_up: bool,
    /// The chains' tops above the rollers' conveying surface, m: −`lowerClearance` down, `liftStroke` up.
    level: f64,
    lift: Option<Move>,
    chain_heading: f64,
    exit_port: Option<String>,
    alternate: usize,
    stats: Stats,
}

impl Transfer3D {
    pub fn new(
        id: &str,
        decks: (usize, usize),
        data: TransferData,
        clearance: f64,
        centre: V3,
        yaw: f64,
        chain_body: RigidBodyHandle,
    ) -> Transfer3D {
        Transfer3D {
            id: id.to_string(),
            rollers: decks.0,
            chains: decks.1,
            exits: BTreeMap::new(),
            level: -clearance,
            data,
            clearance,
            centre,
            yaw,
            chain_body,
            phase: Phase::Idle,
            after: After::Ready,
            chains_up: false,
            lift: None,
            chain_heading: TOWARDS_C2,
            exit_port: None,
            alternate: 0,
            stats: Stats::default(),
        }
    }

    /// The rollers deck's frame: along the table from A, on the rollers' surface.
    pub fn roller_frame(&self) -> Frame {
        let (dx, dz) = rotate_y((-self.data.length / 2.0, 0.0), self.yaw);
        Frame { origin: [self.centre[0] + dx, self.centre[1], self.centre[2] + dz], yaw: self.yaw, slope: 0.0 }
    }

    /// The chains deck's frame when they run toward `heading`, on the chains' tops at `level`.
    pub fn chain_frame(&self, heading: f64, level: f64) -> Frame {
        let yaw = self.yaw + heading;
        let (dx, dz) = rotate_y((-self.data.usable_width / 2.0, 0.0), yaw);
        Frame { origin: [self.centre[0] + dx, self.centre[1] + level, self.centre[2] + dz], yaw, slope: 0.0 }
    }

    /// The chains' heading toward a chain port.
    pub fn heading_to(port: &str) -> f64 {
        if port == "C1" { TOWARDS_C1 } else { TOWARDS_C2 }
    }

    pub fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        match port {
            Some("A") if self.data.operation_mode != OperationMode::ChainsToRollers => Some(self.rollers),
            Some(p) if CHAIN_PORTS.contains(&p) && self.data.operation_mode != OperationMode::RollersToChains => Some(self.chains),
            _ => None,
        }
    }

    fn is_free(&self, decks: &[Deck]) -> bool {
        let empty = |d: &Deck| d.pallets.is_empty() && d.incoming.is_none();
        self.phase == Phase::Idle && self.lift.is_none() && empty(&decks[self.rollers]) && empty(&decks[self.chains])
    }

    /// Free, with its chains down for a pallet coming on the rollers or up for one coming on the chains.
    pub fn can_receive(&self, port: Option<&str>, decks: &[Deck]) -> bool {
        let Some(deck) = self.entry_deck(port) else { return false };
        self.is_free(decks) && (deck == self.chains) == self.chains_up
    }

    /// A pallet waits at `port`: lift the chains for one coming on them, lower them for one coming on the rollers.
    pub fn prepare_for(&mut self, port: Option<&str>, now: SimTime, decks: &[Deck]) {
        let Some(deck) = self.entry_deck(port) else { return };
        if !self.is_free(decks) || (deck == self.chains) == self.chains_up {
            return;
        }
        self.after = After::Ready;
        self.start_lift(deck == self.chains, seconds(now));
    }

    pub fn receive(&mut self, port: Option<&str>, length: f64, now: SimTime, decks: &mut [Deck]) {
        self.stats.cycle_start = Some(now);
        self.phase = Phase::Receiving;
        let deck = self.entry_deck(port).unwrap_or(self.rollers);
        if deck == self.chains {
            // A pallet from C1 runs toward C2, and the other way round.
            self.chain_heading = if port == Some("C1") { TOWARDS_C2 } else { TOWARDS_C1 };
            decks[self.chains].frame = self.chain_frame(self.chain_heading, self.level);
        }
        let d = &mut decks[deck];
        d.end = (d.spec.length + length) / 2.0;
        d.exit = None;
    }

    pub fn step(&mut self, now: SimTime, decks: &mut [Deck], pallets: &Pallets) {
        let t = seconds(now);
        match self.phase {
            Phase::Receiving if centred(&decks[self.rollers], pallets) => {
                self.exit_port = self.choose_chain_exit();
                if self.exit_port.is_some() {
                    self.after = After::CarryOnChains;
                    self.start_lift(true, t);
                }
            }
            Phase::Receiving if centred(&decks[self.chains], pallets) && self.exits.contains_key("B") => {
                self.exit_port = Some("B".into());
                self.after = After::CarryOnRollers;
                self.start_lift(false, t);
            }
            Phase::Lifting | Phase::Lowering if self.lift.is_none_or(|m| t >= m.start + m.duration - 1e-9) => {
                self.lift = None;
                self.chains_up = self.phase == Phase::Lifting;
                match self.after {
                    After::Ready => self.phase = Phase::Idle,
                    After::CarryOnChains => {
                        let port = self.exit_port.clone().unwrap_or_else(|| "C2".into());
                        self.hand_over(self.rollers, self.chains, &port, decks);
                    }
                    After::CarryOnRollers => self.hand_over(self.chains, self.rollers, "B", decks),
                    After::CycleDone => self.finish_cycle(now),
                }
            }
            Phase::Discharging => {
                let empty = |d: &Deck| d.pallets.is_empty() && d.outgoing.is_none();
                if empty(&decks[self.rollers]) && empty(&decks[self.chains]) {
                    self.stats.transfers += 1;
                    if self.chains_up {
                        // The chains stay up until the pallet has left them, then go down.
                        self.after = After::CycleDone;
                        self.start_lift(false, t);
                    } else {
                        self.finish_cycle(now);
                    }
                }
            }
            _ => {}
        }
    }

    /// Sets the chains' vertical speed so that they follow their move over the next step (`t` s).
    pub fn drive(&self, bench: &mut Bench, t: f64) {
        let vy = self.lift.map_or(0.0, |m| (level_on(&m, t + DT) - self.level) / DT);
        bench.set_velocity(self.chain_body, [0.0, vy, 0.0], [0.0; 3]);
    }

    /// Reads the chains' level after a step and moves their deck's frame with it.
    pub fn sync(&mut self, bench: &Bench, decks: &mut [Deck]) {
        self.level = bench.position(self.chain_body)[1] - self.centre[1];
        decks[self.chains].frame = self.chain_frame(self.chain_heading, self.level);
    }

    pub fn busy_status(&self) -> Option<&'static str> {
        match self.phase {
            Phase::Lifting => Some("lifting"),
            Phase::Lowering => Some("lowering"),
            _ => None,
        }
    }

    pub fn accumulate(&mut self, seconds: f64) {
        self.stats.total += seconds;
        if self.phase != Phase::Idle {
            self.stats.busy += seconds;
        }
    }

    pub fn statistics(&self) -> BTreeMap<&'static str, f64> {
        let s = &self.stats;
        BTreeMap::from([
            ("utilization", if s.total > 0.0 { s.busy / s.total } else { 0.0 }),
            ("transfers", s.transfers as f64),
            ("meanCycleTime", if s.cycles > 0 { s.cycle_sum as f64 / s.cycles as f64 / US_PER_S as f64 } else { 0.0 }),
            ("palletsProcessed", s.cycles as f64),
        ])
    }

    /// Starts lifting (true) or lowering (false) the chains.
    fn start_lift(&mut self, up: bool, t: f64) {
        self.phase = if up { Phase::Lifting } else { Phase::Lowering };
        let to = if up { self.data.lift_stroke } else { -self.clearance };
        let duration = if up { self.data.lift_time } else { self.data.lower_time };
        self.lift = (duration > 0.0).then_some(Move { from: self.level, to, start: t, duration });
    }

    /// The centred pallet now belongs to the other deck, which runs it out through `exit_port`.
    fn hand_over(&mut self, from: usize, to: usize, exit_port: &str, decks: &mut [Deck]) {
        if decks[from].pallets.is_empty() {
            return;
        }
        let id = decks[from].pallets.remove(0);
        if to == self.chains {
            self.chain_heading = Transfer3D::heading_to(exit_port);
            decks[self.chains].frame = self.chain_frame(self.chain_heading, self.level);
        }
        let f = &mut decks[from];
        f.holding = false;
        f.end = f.spec.length;
        let t = &mut decks[to];
        t.pallets.push(id);
        t.end = t.spec.length;
        t.holding = false;
        t.exit = self.exits.get(exit_port).cloned();
        self.phase = Phase::Discharging;
    }

    fn finish_cycle(&mut self, now: SimTime) {
        self.phase = Phase::Idle;
        self.exit_port = None;
        if let Some(start) = self.stats.cycle_start.take() {
            self.stats.cycles += 1;
            self.stats.cycle_sum += now - start;
        }
    }

    /// right → C2 (+z), left → C1 (−z), bidirectional → the connected chain exits in turn.
    fn choose_chain_exit(&mut self) -> Option<String> {
        let connected: Vec<&str> = CHAIN_PORTS.iter().copied().filter(|p| self.exits.contains_key(*p)).collect();
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

/// The chains' level `t` s into a move, held at its ends.
fn level_on(m: &Move, t: f64) -> f64 {
    let k = ((t - m.start) / m.duration).clamp(0.0, 1.0);
    m.from + (m.to - m.from) * k
}

/// The deck holds its only pallet at rest, centred.
fn centred(deck: &Deck, pallets: &Pallets) -> bool {
    let Some(p) = deck.pallets.first().and_then(|id| pallets.get(id)) else { return false };
    if deck.incoming.is_some() || !deck.holding || deck.speed > 0.0 {
        return false;
    }
    let e = deck.extent(p);
    e.rear >= -ENTRY_TOLERANCE && e.speed.abs() < REST_SPEED
}

fn seconds(t: SimTime) -> f64 {
    t as f64 / US_PER_S as f64
}
