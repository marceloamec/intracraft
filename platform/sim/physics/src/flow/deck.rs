//! A driven deck of the 3D run (the reference's `deck.ts`): a straight conveyor (rollers or chains), or a station's deck.
//! Its control mirrors the 2D conveyor without accumulation (ADR-0007): the deck runs at one speed for all its pallets,
//! asks the next object to take its lead pallet at the decision point and otherwise brakes so that the pallet stops with
//! its front at `end`. The pallets themselves move by friction.

use std::collections::BTreeMap;

use crate::bench::BodyId;
use intracraft_des::time::SimTime;

use crate::bench::PhysicsBench;
use crate::flow::pallet::{Extent, WorldPallet};
use crate::frame::{Frame, V3, scale};
use crate::mechanics::roller_angular_speed;

/// The pallets of the run, by id.
pub type Pallets = BTreeMap<u64, WorldPallet>;

#[derive(Clone, Debug, PartialEq)]
pub struct DeckSpec {
    /// The conveyor's id, or the station's for a station's deck.
    pub id: String,
    pub length: f64,
    pub usable_width: f64,
    pub nominal_speed: f64,
    pub accel: f64,
    pub decel: f64,
    pub min_gap: f64,
}

/// How the deck moves pallets: kinematic rollers turning at ω = v / r (S01 §4), or parts whose contact surface moves
/// (chain strands, plan B of ADR-0002; a station's platform).
pub enum Drive {
    Rollers { bodies: Vec<BodyId>, radius: f64 },
    Surface(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Empty,
    Working,
    Blocked,
    Stopped,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Empty => "empty",
            Status::Working => "working",
            Status::Blocked => "blocked",
            Status::Stopped => "stopped",
        }
    }
}

/// Where a deck's pallets go: another deck (a conveyor or a station's entry), a sink with the bed it lays beyond the
/// deck's end (a moving surface), or a gravity line.
#[derive(Clone, Debug, PartialEq)]
pub enum Exit {
    Deck { deck: usize, port: Option<String>, gap: f64 },
    Sink { sink: usize, bed: usize },
    Gravity { line: usize, gap: f64 },
}

#[derive(Clone, Debug, Default)]
pub struct DeckStats {
    pub empty: f64,
    pub working: f64,
    pub blocked: f64,
    pub stopped: f64,
    pub occupancy: f64,
    pub processed: u64,
    pub transit_sum: SimTime,
}

pub struct Deck {
    pub spec: DeckSpec,
    pub frame: Frame,
    pub drive: Drive,
    /// The object its pallets are carried by: the conveyor, or the station whose deck it is.
    pub carrier: String,
    pub station: Option<usize>,
    /// +1: pallets travel toward local x = length; −1: toward x = 0 (a station leaving on its entry side).
    pub direction: f64,
    /// Where the lead pallet's front stops when nothing takes it, m along the travel.
    pub end: f64,
    pub exit: Option<Exit>,
    pub speed: f64,
    pub rate: f64,
    pub holding: bool,
    /// Without an exit, it runs its pallets off its end instead of stopping them there (`openEnd` = `runOff`).
    pub run_off: bool,
    pub status: Status,
    /// The pallets it carries (their tail on it), in arrival order.
    pub pallets: Vec<u64>,
    /// A pallet taken from the previous object and still crossing onto it.
    pub incoming: Option<u64>,
    /// A pallet handed to the exit and still crossing off it.
    pub outgoing: Option<u64>,
    pub stats: DeckStats,
}

impl Deck {
    pub fn new(spec: DeckSpec, frame: Frame, drive: Drive, carrier: &str, station: Option<usize>) -> Deck {
        Deck {
            end: spec.length,
            spec,
            frame,
            drive,
            carrier: carrier.to_string(),
            station,
            direction: 1.0,
            exit: None,
            speed: 0.0,
            rate: 0.0,
            holding: false,
            run_off: false,
            status: Status::Empty,
            pallets: Vec::new(),
            incoming: None,
            outgoing: None,
            stats: DeckStats::default(),
        }
    }

    pub fn extent(&self, p: &WorldPallet) -> Extent {
        p.extent_on(&self.frame, self.spec.length, self.direction)
    }

    /// The carried pallet furthest along that has not been handed over yet.
    pub fn lead(&self, pallets: &Pallets) -> Option<u64> {
        let mut lead = None;
        let mut front = f64::NEG_INFINITY;
        for id in self.pallets.iter().filter(|&&id| Some(id) != self.outgoing) {
            if let Some(p) = pallets.get(id) {
                let f = self.extent(p).front;
                if f > front {
                    (lead, front) = (Some(*id), f);
                }
            }
        }
        lead
    }

    /// The rear edge of its last pallet along the travel, or none when it is empty.
    pub fn rear_tail(&self, pallets: &Pallets) -> Option<f64> {
        self.pallets.iter().filter_map(|id| pallets.get(id)).map(|p| self.extent(p).rear).reduce(f64::min)
    }

    /// The world velocity of its conveying surface.
    pub fn surface_velocity(&self) -> V3 {
        scale(self.frame.rotate([1.0, 0.0, 0.0]), self.speed * self.direction)
    }

    /// Turns its rollers or moves its surface at the current speed.
    pub fn apply_drive(&self, bench: &mut dyn PhysicsBench) {
        match &self.drive {
            Drive::Surface(s) => bench.set_surface_velocity(*s, self.surface_velocity()),
            Drive::Rollers { bodies, radius } => {
                // A roller's top moves toward +x when it turns about −z.
                let omega = roller_angular_speed(self.speed, *radius) * self.direction;
                let spin = self.frame.rotate([0.0, 0.0, -omega]);
                for &b in bodies {
                    bench.set_velocity(b, [0.0; 3], spin);
                }
            }
        }
    }

    /// A pallet handed over: it leaves the deck and counts as processed.
    pub fn release(&mut self, id: u64, entered_at: SimTime, now: SimTime) {
        self.pallets.retain(|&p| p != id);
        if self.outgoing == Some(id) {
            self.outgoing = None;
        }
        self.stats.processed += 1;
        self.stats.transit_sum += now - entered_at;
    }

    pub fn accumulate(&mut self, seconds: f64) {
        match self.status {
            Status::Empty => self.stats.empty += seconds,
            Status::Working => self.stats.working += seconds,
            Status::Blocked => self.stats.blocked += seconds,
            Status::Stopped => self.stats.stopped += seconds,
        }
        self.stats.occupancy += self.pallets.len() as f64 * seconds;
    }
}
