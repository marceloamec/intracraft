//! The 3D behaviour of the stations with one deck on a kinematic platform (the reference's `station.ts`): the manual lift
//! table, the scissor lift table and the pallet elevator (S12–S14) and the process station (S11). The same cycle as in 2D
//! (ADR-0013): receive and centre the pallet, lift or process it, discharge it through the exit port, return empty. The
//! platform carries the pallet up and down by contact. A stretch wrapper (S25) is a process station whose platform turns
//! about the table's centre while it wraps, along the 2D programme; the pallet turns with it by friction alone.

use std::collections::BTreeMap;

use crate::bench::BodyId;
use intracraft_des::flow::network::{LiftPort, ProcessData};
use intracraft_des::flow::processing_time;
use intracraft_des::flow::wrap::Programme;
use intracraft_des::kinematics::{MoveProfile, profile_position_at, profile_time};
use intracraft_des::rng::Pcg32;
use intracraft_des::time::{SimTime, US_PER_S, from_seconds};
use intracraft_library::ports::rotate_y;

use crate::bench::{DT, PhysicsBench};
use crate::flow::deck::{Deck, Exit, Pallets, Status};
use crate::frame::Frame;

/// A centred pallet slower than this has come to rest on the deck, m/s.
const REST_SPEED: f64 = 0.005;
/// The pallet's rear may still hang this far over the entry end when it is centred, m.
const ENTRY_TOLERANCE: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Receiving,
    Moving,
    Processing,
    Discharging,
    Returning,
}

pub enum Kind {
    Lift { up_speed: f64, down_speed: f64, vertical_accel: f64 },
    Process { data: ProcessData, rng: Pcg32 },
}

/// A vertical move with a trapezoidal profile that starts and ends at rest (ADR-0013 §3), s from the run's start.
struct VerticalMove {
    from: f64,
    to: f64,
    start: f64,
    duration: f64,
    profile: MoveProfile,
}

#[derive(Clone, Debug, Default)]
struct Stats {
    working: f64,
    blocked: f64,
    total: f64,
    cycles: u64,
    cycle_sum: SimTime,
    done: u64,
    processing_sum: SimTime,
    /// A stretch wrapper's pallets wrapped, turns, film laid (m) and loads too tall (S25 §5).
    wrapped: u64,
    turns: u64,
    film: f64,
    too_tall: u64,
}

/// A wrap in progress: when the processing started (s) and the programme.
#[derive(Clone, Copy, Debug)]
struct Wrapping {
    start: f64,
    programme: Programme,
}

pub struct Station3D {
    pub id: String,
    pub deck: usize,
    pub phase: Phase,
    /// The deck's surface height above the instance's base, m.
    pub height: f64,
    /// The connected exit ports and where they lead.
    pub exits: BTreeMap<String, Exit>,
    platform: BodyId,
    base_y: f64,
    kind: Kind,
    entry: LiftPort,
    exit_ports: Vec<LiftPort>,
    vertical: Option<VerticalMove>,
    wait_until: Option<SimTime>,
    cycle_start: SimTime,
    stats: Stats,
    wrapping: Option<Wrapping>,
    /// A wrapper's deck frame with the table in line (its angle 0).
    home: Option<Frame>,
}

impl Station3D {
    pub fn new(id: &str, deck: usize, platform: BodyId, base_y: f64, kind: Kind, entry: LiftPort, exit_ports: Vec<LiftPort>) -> Station3D {
        Station3D {
            id: id.to_string(),
            deck,
            phase: Phase::Idle,
            height: entry.height,
            exits: BTreeMap::new(),
            platform,
            base_y,
            kind,
            entry,
            exit_ports,
            vertical: None,
            wait_until: None,
            cycle_start: 0,
            stats: Stats::default(),
            wrapping: None,
            home: None,
        }
    }

    /// A stretch wrapper's table turns; its deck frame with the table in line is `home`.
    pub fn with_home(mut self, home: Frame) -> Station3D {
        self.home = Some(home);
        self
    }

    /// A stretch wrapper's table angle (rad, 0 in line), from its deck's frame.
    pub fn table_angle(&self, frame: &Frame) -> Option<f64> {
        let home = self.home.as_ref()?;
        let angle = (frame.yaw - home.yaw).rem_euclid(std::f64::consts::TAU);
        Some(if std::f64::consts::TAU - angle < 1e-9 { 0.0 } else { angle })
    }

    /// Its deck, if `port` is its entry.
    pub fn entry_deck(&self, port: Option<&str>) -> Option<usize> {
        (port == Some(self.entry.port_id.as_str())).then_some(self.deck)
    }

    /// The deck's travel starts at the entry port.
    pub fn entry_direction(&self) -> f64 {
        if self.entry.at_end { -1.0 } else { 1.0 }
    }

    pub fn can_receive(&self, port: Option<&str>, deck: &Deck) -> bool {
        self.entry_deck(port).is_some()
            && self.phase == Phase::Idle
            && self.vertical.is_none()
            && deck.pallets.is_empty()
            && deck.incoming.is_none()
    }

    /// A pallet of `length` is coming in: the deck runs it in and stops it centred.
    pub fn receive(&mut self, length: f64, now: SimTime, deck: &mut Deck) {
        self.cycle_start = now;
        self.phase = Phase::Receiving;
        deck.direction = self.entry_direction();
        deck.end = (deck.spec.length + length) / 2.0;
        deck.exit = None;
    }

    /// Moves the cycle on at `now`.
    pub fn step(&mut self, now: SimTime, deck: &mut Deck, pallets: &mut Pallets) {
        let t = now as f64 / US_PER_S as f64;
        match self.phase {
            Phase::Receiving => {
                if !centred(deck, pallets) {
                    return;
                }
                if let Kind::Process { data, rng } = &mut self.kind {
                    self.phase = Phase::Processing;
                    let time = match &data.wrap {
                        Some(wrap) => {
                            let pallet = deck.pallets.first().and_then(|id| pallets.get(id)).map(|p| &p.pallet);
                            let (load, perimeter) = pallet.map_or((0.0, 0.0), |p| (p.load_height, 2.0 * (p.length + p.width)));
                            let programme = wrap.programme(load);
                            if programme.turns > 0 {
                                self.stats.wrapped += 1;
                                self.stats.turns += u64::from(programme.turns);
                                self.stats.film += f64::from(programme.turns) * perimeter;
                            }
                            self.stats.too_tall += u64::from(programme.too_tall);
                            self.wrapping = Some(Wrapping { start: t, programme });
                            from_seconds(programme.processing_time)
                        }
                        None => processing_time(data, rng),
                    };
                    self.stats.processing_sum += time;
                    self.wait_until = Some(now + time);
                } else {
                    self.phase = Phase::Moving;
                    let to = self.exit_port().map_or(self.height, |p| p.height);
                    self.start_move(to, t);
                }
            }
            Phase::Processing => {
                if self.wait_until.is_none_or(|w| now < w) {
                    return;
                }
                self.wait_until = None;
                self.stats.done += 1;
                // The film is cut: the pallet leaves wrapped, the table in line.
                if let Some(w) = self.wrapping.take() {
                    if w.programme.turns > 0
                        && let Some(p) = deck.pallets.first().and_then(|id| pallets.get_mut(id))
                    {
                        p.pallet.wrapped = true;
                    }
                    if let Some(home) = &self.home {
                        deck.frame.yaw = home.yaw;
                        deck.frame.origin = [home.origin[0], deck.frame.origin[1], home.origin[2]];
                    }
                }
                self.discharge(deck);
            }
            Phase::Moving => {
                if self.vertical.is_none() {
                    self.discharge(deck);
                }
            }
            Phase::Discharging => {
                if !deck.pallets.is_empty() || deck.outgoing.is_some() {
                    return;
                }
                deck.end = deck.spec.length;
                if matches!(self.kind, Kind::Process { .. }) {
                    self.end_cycle(now);
                } else {
                    self.stats.done += 1;
                    self.phase = Phase::Returning;
                    self.start_move(self.entry.height, t);
                }
            }
            Phase::Returning => {
                if self.vertical.is_none() {
                    self.end_cycle(now);
                    deck.direction = self.entry_direction();
                }
            }
            Phase::Idle => {}
        }
    }

    /// Sets the platform's vertical speed, and a wrapper's turning speed, so that it follows its profile over the next
    /// step (`t` s).
    pub fn drive_vertical(&self, bench: &mut dyn PhysicsBench, t: f64) {
        let vy = self.vertical.as_ref().map_or(0.0, |m| (height_on(m, t + DT) - self.height) / DT);
        let omega = match (&self.kind, &self.wrapping) {
            (Kind::Process { data, .. }, Some(w)) => {
                data.wrap.as_ref().map_or(0.0, |wrap| wrap.angular_speed_at(&w.programme, t - w.start, DT))
            }
            _ => 0.0,
        };
        bench.set_velocity(self.platform, [0.0, vy, 0.0], [0.0, omega, 0.0]);
    }

    /// Reads the platform's height after a step (`t` s) and moves the deck's frame with it.
    pub fn sync_height(&mut self, bench: &dyn PhysicsBench, t: f64, deck: &mut Deck) {
        self.height = bench.position(self.platform)[1] - self.base_y;
        deck.frame.origin[1] = self.base_y + self.height;
        // A wrapper's deck turns with its table about the table's centre, its entry edge half a length behind it.
        if let (Kind::Process { data, .. }, Some(w), Some(home)) = (&self.kind, &self.wrapping, &self.home) {
            let angle = data.wrap.as_ref().map_or(0.0, |wrap| wrap.angle_at(&w.programme, t - w.start));
            let half = deck.spec.length / 2.0;
            let (cx, cz) = rotate_y((half, 0.0), home.yaw);
            let centre = (home.origin[0] + cx, home.origin[2] + cz);
            let yaw = home.yaw + angle;
            let (dx, dz) = rotate_y((-half, 0.0), yaw);
            deck.frame.yaw = yaw;
            deck.frame.origin = [centre.0 + dx, deck.frame.origin[1], centre.1 + dz];
        }
        if self.vertical.as_ref().is_some_and(|m| t >= m.start + m.duration - 1e-9) {
            self.vertical = None;
        }
    }

    /// `lifting`, `lowering` or `processing` while busy.
    pub fn busy_status(&self) -> Option<&'static str> {
        if self.phase == Phase::Processing {
            return Some("processing");
        }
        self.vertical.as_ref().map(|m| if m.to > m.from { "lifting" } else { "lowering" })
    }

    /// Adds `seconds` in the current phase. As in 2D, a lift station works in every phase but idle; a process station only
    /// while processing. Discharging into a full next object counts as blocked.
    pub fn accumulate(&mut self, seconds: f64, deck: Status) {
        self.stats.total += seconds;
        let working = match self.kind {
            Kind::Process { .. } => self.phase == Phase::Processing,
            Kind::Lift { .. } => self.phase != Phase::Idle,
        };
        if self.phase == Phase::Discharging && deck == Status::Blocked {
            self.stats.blocked += seconds;
        } else if working {
            self.stats.working += seconds;
        }
    }

    pub fn statistics(&self) -> BTreeMap<&'static str, f64> {
        let s = &self.stats;
        let share = |v: f64| if s.total > 0.0 { v / s.total } else { 0.0 };
        let seconds = |t: SimTime, n: u64| if n > 0 { t as f64 / n as f64 / US_PER_S as f64 } else { 0.0 };
        let mut out = BTreeMap::from([
            ("utilization", share(s.working)),
            ("blockedShare", share(s.blocked)),
            ("idleShare", share(s.total - s.working - s.blocked)),
            ("meanCycleTime", seconds(s.cycle_sum, s.cycles)),
        ]);
        if matches!(self.kind, Kind::Process { .. }) {
            out.insert("palletsProcessed", s.done as f64);
            out.insert("meanProcessingTime", seconds(s.processing_sum, s.done));
            if self.home.is_some() {
                out.insert("meanTurns", if s.wrapped > 0 { s.turns as f64 / s.wrapped as f64 } else { 0.0 });
                out.insert("filmUsed", s.film);
                out.insert("tooTallPallets", s.too_tall as f64);
            }
        } else {
            out.insert("palletsMoved", s.done as f64);
        }
        out
    }

    /// A cycle runs from accepting a pallet to being free again.
    fn end_cycle(&mut self, now: SimTime) {
        self.phase = Phase::Idle;
        self.stats.cycles += 1;
        self.stats.cycle_sum += now - self.cycle_start;
    }

    /// The first connected exit port (the elevator prefers OUT_NEAR, S14 §1), else the first one.
    fn exit_port(&self) -> Option<&LiftPort> {
        self.exit_ports.iter().find(|p| self.exits.contains_key(&p.port_id)).or(self.exit_ports.first())
    }

    fn discharge(&mut self, deck: &mut Deck) {
        self.phase = Phase::Discharging;
        let Some(port) = self.exit_port() else { return };
        deck.direction = if port.at_end { 1.0 } else { -1.0 };
        deck.exit = self.exits.get(&port.port_id).cloned();
        deck.end = deck.spec.length;
        deck.holding = false;
    }

    fn start_move(&mut self, to: f64, t: f64) {
        let Kind::Lift { up_speed, down_speed, vertical_accel } = self.kind else { return };
        let distance = (to - self.height).abs();
        if distance < 1e-6 {
            return;
        }
        let speed = if to > self.height { up_speed } else { down_speed };
        let profile = MoveProfile { distance, max_speed: speed, accel: vertical_accel, decel: Some(vertical_accel) };
        let duration = profile_time(&profile).unwrap_or(0.0);
        self.vertical = Some(VerticalMove { from: self.height, to, start: t, duration, profile });
    }
}

/// The pallet is wholly on the deck and at rest, held at the centre.
fn centred(deck: &Deck, pallets: &Pallets) -> bool {
    let Some(p) = deck.pallets.first().and_then(|id| pallets.get(id)) else { return false };
    if !deck.holding || deck.speed > 0.0 {
        return false;
    }
    let e = deck.extent(p);
    e.rear >= -ENTRY_TOLERANCE && e.speed.abs() < REST_SPEED
}

/// The height along a move at `t` s, held at its ends (T04 §2).
fn height_on(m: &VerticalMove, t: f64) -> f64 {
    let covered = profile_position_at(&m.profile, t - m.start).unwrap_or(m.profile.distance);
    m.from + (m.to - m.from).signum() * covered
}
