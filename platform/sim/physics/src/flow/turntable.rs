//! The 3D behaviour of the chain and roller turntables (S03, S04, T13 §1.6): the same cycle as the 2D turntable
//! (ADR-0008) on a kinematic platform that turns about its vertical axis. Its deck (chain strands or rollers) turns with
//! it; the pallet on it moves by friction alone, held during the turn with the deck stopped (rollers locked, S04 §3D).
//!
//! idle → [positioning: turn to line up with the entry] → receiving (the pallet stops centred) → turning to the exit and
//! settling → discharging → idle. The deck conveys both ways, so it never turns more than a quarter.

use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_2, PI};

use crate::bench::BodyId;
use intracraft_des::flow::network::{ExitRule, TurntableData, TurntableExit};
use intracraft_des::kinematics::{MoveProfile, profile_position_at, profile_time};
use intracraft_des::rng::Pcg32;
use intracraft_des::time::{SimTime, US_PER_S};
use intracraft_library::ports::rotate_y;

use crate::bench::{DT, PhysicsBench};
use crate::flow::deck::{Deck, Exit, Pallets};
use crate::frame::{Frame, V3};

const FULL_TURN: f64 = 2.0 * PI;
const ANGLE_EPS: f64 = 1e-6;
/// A centred pallet slower than this has come to rest on the deck, m/s.
const REST_SPEED: f64 = 0.005;
/// The pallet's rear may still hang this far over the entry edge when it is centred, m.
const ENTRY_TOLERANCE: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Positioning,
    Receiving,
    Turning,
    Discharging,
}

/// A turn: from the deck's heading, by a signed rotation, to convey toward `target` (the deck runs both ways: it may
/// end facing the other way), then the settling time; times in s.
#[derive(Clone, Copy, Debug)]
struct Turn {
    start: f64,
    from: f64,
    rotation: f64,
    target: f64,
    moving: f64,
    settled: f64,
}

#[derive(Clone, Debug, Default)]
struct Stats {
    busy: f64,
    total: f64,
    turns: u64,
    cycles: u64,
    cycle_sum: SimTime,
    cycle_start: Option<SimTime>,
}

pub struct Turntable3D {
    pub id: String,
    pub deck: usize,
    /// Where the exit ports lead, by port.
    pub exits: BTreeMap<String, Exit>,
    data: TurntableData,
    /// The platform's centre on the conveying surface, and the instance's turn about +y.
    centre: V3,
    yaw: f64,
    platform: BodyId,
    phase: Phase,
    /// The direction the deck conveys towards (rad, the turntable's frame).
    heading: f64,
    turn: Option<Turn>,
    exit: Option<TurntableExit>,
    entry_port: Option<String>,
    alternate: usize,
    rng: Pcg32,
    stats: Stats,
}

impl Turntable3D {
    pub fn new(id: &str, deck: usize, data: TurntableData, centre: V3, yaw: f64, platform: BodyId, rng: Pcg32) -> Turntable3D {
        Turntable3D {
            id: id.to_string(),
            deck,
            exits: BTreeMap::new(),
            data,
            centre,
            yaw,
            platform,
            phase: Phase::Idle,
            heading: 0.0,
            turn: None,
            exit: None,
            entry_port: None,
            alternate: 0,
            rng,
            stats: Stats::default(),
        }
    }

    /// The heading a deck frame of this turntable conveys toward, rad in its own frame.
    pub fn heading_of(&self, frame: &Frame) -> f64 {
        normalize(frame.yaw - self.yaw)
    }

    /// The deck's frame when it conveys toward `heading`: its x = 0 edge opposite that way, on the conveying surface.
    pub fn frame(&self, heading: f64) -> Frame {
        let yaw = self.yaw + heading;
        let (dx, dz) = rotate_y((-self.data.platform_length / 2.0, 0.0), yaw);
        Frame { origin: [self.centre[0] + dx, self.centre[1], self.centre[2] + dz], yaw, slope: 0.0 }
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

    /// Free, empty and lined up with the entry: a pallet coming through `port` can enter now.
    pub fn can_receive(&self, port: Option<&str>, deck: &Deck) -> bool {
        let Some(angle) = self.station_angle(port) else { return false };
        self.phase == Phase::Idle && deck.pallets.is_empty() && deck.incoming.is_none() && same_line(self.heading, angle)
    }

    /// A pallet waits at `port`: if idle and empty but not lined up, turn to it.
    pub fn prepare_for(&mut self, port: Option<&str>, now: SimTime, deck: &Deck) {
        let Some(angle) = self.station_angle(port) else { return };
        if self.phase != Phase::Idle || !deck.pallets.is_empty() || same_line(self.heading, angle) {
            return;
        }
        self.phase = Phase::Positioning;
        self.start_turn(normalize(angle + PI), seconds(now));
    }

    /// The pallet was taken through `port`: the deck runs from that side and stops it with its centre on the platform's.
    pub fn receive(&mut self, port: Option<&str>, length: f64, now: SimTime, deck: &mut Deck) {
        self.stats.cycle_start = Some(now);
        if let Some(angle) = self.station_angle(port) {
            self.heading = normalize(angle + PI);
        }
        self.phase = Phase::Receiving;
        self.entry_port = port.map(str::to_string);
        deck.frame = self.frame(self.heading);
        deck.direction = 1.0;
        deck.end = (self.data.platform_length + length) / 2.0;
        deck.exit = None;
    }

    /// Moves the cycle on at `now`.
    pub fn step(&mut self, now: SimTime, deck: &mut Deck, pallets: &Pallets) {
        let t = seconds(now);
        match self.phase {
            Phase::Receiving if centred(deck, pallets) => {
                let Some(exit) = self.choose_exit() else { return };
                let angle = exit.angle;
                self.exit = Some(exit);
                self.phase = Phase::Turning;
                self.start_turn(angle, t);
            }
            Phase::Positioning | Phase::Turning if self.turn.is_none_or(|turn| t >= turn.settled - 1e-9) => {
                if let Some(turn) = self.turn.take() {
                    self.heading = turn.target;
                }
                deck.frame = self.frame(self.heading);
                if self.phase == Phase::Positioning {
                    self.phase = Phase::Idle;
                } else {
                    self.start_discharge(deck);
                }
            }
            Phase::Discharging if deck.pallets.is_empty() && deck.outgoing.is_none() => {
                self.phase = Phase::Idle;
                self.exit = None;
                self.entry_port = None;
                if let Some(start) = self.stats.cycle_start.take() {
                    self.stats.cycles += 1;
                    self.stats.cycle_sum += now - start;
                }
            }
            _ => {}
        }
    }

    /// Sets the platform's turning speed so that it follows the angular profile over the next step (`t` s).
    pub fn drive(&self, bench: &mut dyn PhysicsBench, t: f64) {
        let omega = self.turn.map_or(0.0, |turn| (self.angle_on(&turn, t + DT) - self.angle_on(&turn, t)) / DT);
        bench.set_velocity(self.platform, [0.0; 3], [0.0, omega, 0.0]);
    }

    /// After a step (`t` s): the deck's frame follows the platform's angle.
    pub fn sync(&self, t: f64, deck: &mut Deck) {
        if let Some(turn) = &self.turn {
            deck.frame = self.frame(self.angle_on(turn, t));
        }
    }

    /// `turning` while the platform turns.
    pub fn busy_status(&self) -> Option<&'static str> {
        self.turn.is_some().then_some("turning")
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
            ("turns", s.turns as f64),
            ("meanCycleTime", if s.cycles > 0 { s.cycle_sum as f64 / s.cycles as f64 / US_PER_S as f64 } else { 0.0 }),
            ("palletsProcessed", s.cycles as f64),
        ])
    }

    fn start_discharge(&mut self, deck: &mut Deck) {
        let Some(exit) = self.exit.clone() else { return };
        self.phase = Phase::Discharging;
        deck.end = self.data.platform_length;
        deck.holding = false;
        deck.exit = self.exits.get(&exit.port_id).cloned();
    }

    /// Turns the shortest way to the line of `target` (the deck runs both ways), then settles.
    fn start_turn(&mut self, target: f64, t: f64) {
        let mut rotation = normalize(target - self.heading);
        if rotation > PI {
            rotation -= FULL_TURN;
        }
        if rotation.abs() > FRAC_PI_2 + ANGLE_EPS {
            rotation -= rotation.signum() * PI;
        }
        if rotation.abs() < ANGLE_EPS {
            self.heading = target;
            self.turn = None;
            return;
        }
        self.stats.turns += 1;
        let moving = profile_time(&self.profile(rotation.abs())).unwrap_or(0.0);
        self.turn = Some(Turn { start: t, from: self.heading, rotation, target, moving, settled: t + moving + self.data.settle_time });
    }

    /// The deck's heading `t` s into a turn, held at its ends.
    fn angle_on(&self, turn: &Turn, t: f64) -> f64 {
        let elapsed = (t - turn.start).clamp(0.0, turn.moving);
        let covered = profile_position_at(&self.profile(turn.rotation.abs()), elapsed).unwrap_or(turn.rotation.abs());
        turn.from + turn.rotation.signum() * covered
    }

    fn profile(&self, angle: f64) -> MoveProfile {
        MoveProfile {
            distance: angle,
            max_speed: self.data.max_angular_speed,
            accel: self.data.angular_accel,
            decel: Some(self.data.angular_decel),
        }
    }

    /// The exits other than the entry, clockwise seen from above from the entry on, picked by the exit rule.
    fn choose_exit(&mut self) -> Option<TurntableExit> {
        let entry = self.station_angle(self.entry_port.as_deref()).unwrap_or(0.0);
        let mut candidates: Vec<TurntableExit> = self
            .data
            .exits
            .iter()
            .filter(|e| Some(e.port_id.as_str()) != self.entry_port.as_deref() && self.station_angle(Some(&e.port_id)).is_some())
            .cloned()
            .collect();
        candidates.sort_by(|a, b| normalize(entry - a.angle).total_cmp(&normalize(entry - b.angle)));
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

/// The pallet is wholly on the deck and at rest, held at the centre.
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
