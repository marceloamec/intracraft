//! The 3D behaviour of the gravity roller line (S15; the reference's `gravity.ts`): pallets roll down free rollers on the
//! slope, brake rollers cap their speed, a separator holds the next pallet back from the pick-up position and an end stop
//! takes the impact. The rules are the 2D model's (ADR-0014), measured on the bodies.
//!
//! Free rollers are kinematic cylinders that follow the pallet above them: each step they turn at the speed the pallet
//! reaches rolling freely, v + g·(s − Crr)·dt, capped at the brake speed. Contacts with the separator, the end stop and
//! the other pallets stay physical (ADR-0016).

use std::collections::BTreeMap;

use intracraft_des::flow::network::GravityData;
use intracraft_des::time::{SimTime, US_PER_S};
use rapier3d::prelude::RigidBodyHandle;

use crate::bench::Bench;
use crate::flow::deck::Pallets;
use crate::flow::pallet::Extent;
use crate::frame::{Frame, scale};
use crate::mechanics::GRAVITY;

/// A pallet this close to what is ahead touches it, m.
const TOUCH: f64 = 0.01;
/// Below this speed a touching pallet counts as stopped, m/s.
const STOP_SPEED: f64 = 0.01;
/// The separator plate's centre above the rollers' surface when closed and when open, m.
pub const SEPARATOR_CLOSED_Y: f64 = 0.005;
pub const SEPARATOR_OPEN_Y: f64 = -0.085;
const SEPARATOR_SPEED: f64 = 0.5;
/// The separator plate's thickness along the travel; its upstream face is at the separator, m.
pub const SEPARATOR_THICKNESS: f64 = 0.02;

#[derive(Clone, Debug, Default)]
struct State {
    stopped: bool,
    /// Stopped at the closed separator: it starts as soon as it opens.
    at_separator: bool,
    /// A delayed start behind a pallet that started, s.
    start_at: Option<f64>,
}

#[derive(Clone, Debug, Default)]
struct Stats {
    delivered: u64,
    max_queue: usize,
    queue: f64,
    full: f64,
    pickup_wait: f64,
    total: f64,
    transit_sum: SimTime,
}

pub struct GravityLine3D {
    pub id: String,
    /// The pallets on the line, front first.
    pub pallets: Vec<u64>,
    /// Pallets taken and still pushed in by the feeder: the free rollers already carry their front.
    pub entering: Vec<u64>,
    /// The operator at OUT: a sink of the run.
    pub sink: Option<usize>,
    /// The length of the pallet a feeder is waiting to push in while the line is full, as in 2D.
    pub waiting_length: Option<f64>,
    pub status: &'static str,
    pub frame: Frame,
    data: GravityData,
    /// The free rollers and where they lie along the line (m from the entry).
    rollers: Vec<(RigidBodyHandle, f64)>,
    radius: f64,
    separator: RigidBodyHandle,
    separator_x: f64,
    accel: f64,
    state: BTreeMap<u64, State>,
    separator_closed: bool,
    holder: Option<u64>,
    /// When the closed separator opens again, s.
    open_at: Option<f64>,
    stats: Stats,
}

impl GravityLine3D {
    pub fn new(
        id: &str,
        data: GravityData,
        frame: Frame,
        rollers: Vec<(RigidBodyHandle, f64)>,
        radius: f64,
        separator: RigidBodyHandle,
    ) -> GravityLine3D {
        GravityLine3D {
            id: id.to_string(),
            pallets: Vec::new(),
            entering: Vec::new(),
            sink: None,
            waiting_length: None,
            status: "empty",
            frame,
            separator_x: data.length - data.pickup_length,
            accel: GRAVITY * (data.slope - data.rolling_resistance),
            data,
            rollers,
            radius,
            separator,
            state: BTreeMap::new(),
            separator_closed: false,
            holder: None,
            open_at: None,
            stats: Stats::default(),
        }
    }

    pub fn extent(&self, id: u64, pallets: &Pallets) -> Option<Extent> {
        pallets.get(&id).map(|p| p.extent_on(&self.frame, self.data.length, 1.0))
    }

    /// A whole pallet of `length` fits behind the last one and nothing else is entering. Unlike 2D the feeder's gap
    /// counts: the pallet is free of the feeder only once its front is that far in.
    pub fn can_accept(&self, length: f64, pallets: &Pallets) -> bool {
        if !self.entering.is_empty() {
            return false;
        }
        self.pallets.last().and_then(|&id| self.extent(id, pallets)).is_none_or(|e| e.rear >= length)
    }

    /// The pallet's tail cleared the feeder: from now on it rolls on the line.
    pub fn receive(&mut self, id: u64) {
        self.entering.retain(|&e| e != id);
        self.pallets.push(id);
        self.state.insert(id, State::default());
        self.stats.max_queue = self.stats.max_queue.max(self.pallets.len());
    }

    /// The pallet at rest against the end stop, waiting for the operator.
    pub fn at_end_stop(&self, pallets: &Pallets) -> Option<u64> {
        let &first = self.pallets.first()?;
        if !self.state.get(&first).is_some_and(|s| s.stopped) {
            return None;
        }
        self.extent(first, pallets).filter(|e| e.front >= self.data.length - TOUCH).map(|_| first)
    }

    /// The operator takes the pallet at the end stop (it entered at `entered`); the separator opens `separatorTime` later.
    pub fn take(&mut self, id: u64, entered: SimTime, now: SimTime) {
        self.pallets.retain(|&p| p != id);
        self.state.remove(&id);
        self.stats.delivered += 1;
        self.stats.transit_sum += now - entered;
        if self.holder == Some(id) {
            self.holder = None;
        }
        self.open_at = Some(now as f64 / US_PER_S as f64 + self.data.separator_time);
    }

    /// A pallet gone from the run (fallen): it leaves the line.
    pub fn forget(&mut self, id: u64) {
        self.pallets.retain(|&p| p != id);
        self.entering.retain(|&p| p != id);
        self.state.remove(&id);
        if self.holder == Some(id) {
            self.holder = None;
        }
    }

    /// Stops, delayed starts and the separator at `t` s, from the measured positions.
    pub fn update(&mut self, t: f64, pallets: &Pallets) {
        if self.open_at.is_some_and(|at| at <= t) {
            self.open_at = None;
            self.separator_closed = false;
        }
        let extents: Vec<Option<Extent>> = self.pallets.iter().map(|&id| self.extent(id, pallets)).collect();
        let mut moving = false;
        for i in 0..self.pallets.len() {
            let id = self.pallets[i];
            let Some(e) = extents[i] else { continue };
            let ahead = i.checked_sub(1).map(|k| self.pallets[k]);
            let ahead_stopped = ahead.and_then(|a| self.state.get(&a)).map(|s| s.stopped);
            let ahead_rear = i.checked_sub(1).and_then(|k| extents[k]).map(|e| e.rear);
            let (mut limit, mut kind) = (self.data.length, "end");
            if self.separator_closed && e.front <= self.separator_x + TOUCH && self.separator_x < limit {
                (limit, kind) = (self.separator_x, "separator");
            }
            if let Some(rear) = ahead_rear.filter(|r| *r < limit) {
                (limit, kind) = (rear, "pallet");
            }
            let touching = limit - e.front <= TOUCH;
            // Touching a pallet that has started doesn't block: the two move off together.
            let leaving = kind == "pallet" && ahead_stopped == Some(false);
            let separator_closed = self.separator_closed;
            let start_delay = self.data.start_delay;
            let Some(st) = self.state.get_mut(&id) else { continue };
            let mut start = false;
            if !st.stopped {
                if e.rear >= 0.0 && touching && !leaving && e.speed.abs() < STOP_SPEED {
                    st.stopped = true;
                    st.at_separator = kind == "separator";
                    st.start_at = None;
                }
            } else if touching && !leaving {
                if kind == "pallet" && ahead_stopped == Some(true) {
                    st.start_at = None;
                }
            } else {
                let due = (st.at_separator && !separator_closed) || st.start_at.is_some_and(|at| at <= t);
                if !due && st.start_at.is_none() {
                    st.start_at = Some(t + start_delay);
                }
                start = due || st.start_at.is_some_and(|at| at <= t);
            }
            if start {
                self.start(i, t, pallets);
            }
            if self.state.get(&id).is_some_and(|s| !s.stopped) {
                moving = true;
            }
            if self.holder.is_none() && !self.separator_closed && e.front >= self.separator_x {
                self.holder = Some(id);
            }
            if self.holder == Some(id)
                && !self.separator_closed
                && e.rear >= self.separator_x + SEPARATOR_THICKNESS + TOUCH
                && !self.over_separator(pallets)
            {
                self.separator_closed = true;
            }
        }
        self.status = if self.pallets.is_empty() && self.entering.is_empty() {
            "empty"
        } else if self.waiting_length.is_some() {
            "blocked"
        } else if moving || !self.entering.is_empty() {
            "working"
        } else {
            "waiting"
        };
    }

    /// Free rollers follow their pallet; the separator plate moves toward its position. `feeding` gives the speed of the
    /// conveyor pushing each entering pallet in.
    pub fn drive(&self, bench: &mut Bench, pallets: &Pallets, dt: f64, feeding: &BTreeMap<u64, f64>) {
        let targets: Vec<(Extent, f64)> = self
            .pallets
            .iter()
            .chain(&self.entering)
            .filter_map(|&id| {
                let e = self.extent(id, pallets)?;
                let speed = if self.entering.contains(&id) {
                    // Still pushed by the feeder: the rollers under it turn at the feeder's speed. Following the pallet's
                    // own speed instead, they would speed up with it on the slope without end.
                    feeding.get(&id).copied().unwrap_or(0.0)
                } else if self.state.get(&id).is_none_or(|s| s.stopped) {
                    0.0
                } else {
                    (e.speed + self.accel * dt).max(0.0).min(self.data.brake_speed)
                };
                Some((e, speed))
            })
            .collect();
        let axis = self.frame.rotate([0.0, 0.0, 1.0]);
        for &(body, s) in &self.rollers {
            let speed = targets.iter().find(|(e, _)| e.rear <= s && s <= e.front).map_or(0.0, |(_, v)| *v);
            bench.set_velocity(body, [0.0; 3], scale(axis, -speed / self.radius));
        }
        let current = self.frame.to_local(bench.position(self.separator))[1];
        let target = if self.separator_closed { SEPARATOR_CLOSED_Y } else { SEPARATOR_OPEN_Y };
        let speed = ((target - current) / dt).clamp(-SEPARATOR_SPEED, SEPARATOR_SPEED);
        bench.set_velocity(self.separator, scale(self.frame.rotate([0.0, 1.0, 0.0]), speed), [0.0; 3]);
    }

    pub fn accumulate(&mut self, seconds: f64, pallets: &Pallets) {
        self.stats.total += seconds;
        self.stats.queue += self.pallets.len() as f64 * seconds;
        if self.waiting_length.is_some() {
            self.stats.full += seconds;
        }
        if self.at_end_stop(pallets).is_some() {
            self.stats.pickup_wait += seconds;
        }
    }

    pub fn statistics(&self) -> BTreeMap<&'static str, f64> {
        let s = &self.stats;
        let share = |v: f64| if s.total > 0.0 { v / s.total } else { 0.0 };
        BTreeMap::from([
            ("palletsDelivered", s.delivered as f64),
            ("meanQueue", share(s.queue)),
            ("maxQueue", s.max_queue as f64),
            ("fullShare", share(s.full)),
            ("pickupWaitShare", share(s.pickup_wait)),
            ("meanTransitTime", if s.delivered > 0 { s.transit_sum as f64 / s.delivered as f64 / US_PER_S as f64 } else { 0.0 }),
        ])
    }

    /// Starts the pallet at `index`; the one stopped right behind it follows `startDelay` later.
    fn start(&mut self, index: usize, t: f64, pallets: &Pallets) {
        let id = self.pallets[index];
        if let Some(st) = self.state.get_mut(&id) {
            st.stopped = false;
            st.at_separator = false;
            st.start_at = None;
        }
        let Some(&behind) = self.pallets.get(index + 1) else { return };
        let gap = match (self.extent(id, pallets), self.extent(behind, pallets)) {
            (Some(a), Some(b)) => a.rear - b.front,
            _ => return,
        };
        if let Some(st) = self.state.get_mut(&behind).filter(|s| s.stopped)
            && gap <= TOUCH
        {
            st.start_at = Some(t + self.data.start_delay);
        }
    }

    /// Some pallet lies over the separator plate, so it can't rise.
    fn over_separator(&self, pallets: &Pallets) -> bool {
        self.pallets
            .iter()
            .filter_map(|&id| self.extent(id, pallets))
            .any(|e| e.rear < self.separator_x + SEPARATOR_THICKNESS + TOUCH && e.front > self.separator_x - TOUCH)
    }
}
