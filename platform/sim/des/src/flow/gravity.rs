//! The gravity roller line in the 2D run (S15, ADR-0014), a port of the reference's `gravity-line.ts`. Pallets roll
//! down one by one: from rest a pallet accelerates with a = g·(slope − Crr) up to the brake speed, then keeps it. It stops
//! when it touches what is ahead (the pallet in front, the closed separator or the end stop); one that touches a moving
//! pallet goes on with it.
//! - Queue start: when a stopped pallet starts, the one stopped behind it starts `startDelay` later.
//! - Separator at x = length − pickupLength: open while the pick-up position is empty; it closes once the pallet that
//!   entered the position has passed it, and opens `separatorTime` after the operator took that pallet.
//!
//! Positions are the pallet fronts along the line (plan length, m), valid at `t0` (s). The line predicts its own next
//! change, which the run schedules.

use crate::flow::network::GravityData;
use crate::flow::sim::{EPS, Pallet, SPEED_EPS};
use crate::time::{SimTime, US_PER_S};

/// Times closer than this are one instant, s.
const TIME_EPS: f64 = 1e-9;
pub const GRAVITY: f64 = 9.81;

#[derive(Clone, Debug)]
pub struct GravityPallet {
    pub pallet: Pallet,
    pub speed: f64,
    pub accel: f64,
    pub stopped: bool,
    /// Stopped at the closed separator: it starts as soon as the separator opens.
    at_separator: bool,
    /// When a stopped pallet starts, s, if the pallet ahead has moved away.
    start_at: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LimitKind {
    End,
    Separator,
    Pallet,
}

pub struct GravityLine {
    pub id: String,
    pub data: GravityData,
    pub downstream: Option<String>,
    /// Pallets on the line, front (lowest) first.
    pub pallets: Vec<GravityPallet>,
    /// Pallets taken from the feeder and still crossing onto the line.
    pub incoming: u32,
    /// The room a waiting feeder needs behind the last pallet, m, while it waits.
    pub waiting_length: Option<f64>,
    pub ticket: Option<u64>,
    /// a = g·(slope − Crr) from rest, m/s²; ≤ 0: a pallet at rest doesn't start (S15 §4).
    pub accel: f64,
    separator_x: f64,
    t0: f64,
    separator_closed: bool,
    /// The pallet that entered the pick-up position and hasn't been taken yet.
    holder: Option<u64>,
    open_at: Option<f64>,
    delivered: u64,
    max_queue: usize,
    queue_time: f64,
    full_time: f64,
    pickup_wait_time: f64,
    total: f64,
    transit_sum: SimTime,
}

fn tail(p: &GravityPallet) -> f64 {
    p.pallet.front - p.pallet.length
}

/// The time, s, a pallet takes to cover `distance` at its speed and acceleration, if it does.
fn reach(p: &GravityPallet, distance: f64) -> Option<f64> {
    if distance <= 0.0 {
        return None;
    }
    let root = first_root(-distance, p.speed, p.accel)?;
    // A decelerating pallet stops before covering it.
    if p.accel < 0.0 && root > p.speed / -p.accel {
        return None;
    }
    Some(root)
}

/// The smallest t > 0 with c + b·t + ½·a·t² = 0 (c: the gap, b: the relative speed, a: the relative acceleration).
fn first_root(c: f64, b: f64, a: f64) -> Option<f64> {
    if a.abs() < 1e-12 {
        if b.abs() < 1e-12 {
            return None;
        }
        let t = -c / b;
        return (t > TIME_EPS).then_some(t);
    }
    let disc = b * b - 2.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let root = disc.sqrt();
    [(-b - root) / a, (-b + root) / a].into_iter().filter(|t| *t > TIME_EPS).reduce(f64::min)
}

impl GravityLine {
    pub fn new(id: &str, data: GravityData, downstream: Option<String>) -> GravityLine {
        GravityLine {
            id: id.to_string(),
            accel: GRAVITY * (data.slope - data.rolling_resistance),
            separator_x: data.length - data.pickup_length,
            data,
            downstream,
            pallets: Vec::new(),
            incoming: 0,
            waiting_length: None,
            ticket: None,
            t0: 0.0,
            separator_closed: false,
            holder: None,
            open_at: None,
            delivered: 0,
            max_queue: 0,
            queue_time: 0.0,
            full_time: 0.0,
            pickup_wait_time: 0.0,
            total: 0.0,
            transit_sum: 0,
        }
    }

    /// A whole pallet of `length` fits behind the last one and nothing else is entering.
    pub fn can_accept(&self, length: f64) -> bool {
        self.incoming == 0 && self.pallets.last().is_none_or(|rear| tail(rear) >= length - EPS)
    }

    /// The pallet's tail cleared the feeder: it is wholly on the line, moving at min(the feeder's speed, the brake speed).
    pub fn receive(&mut self, mut pallet: Pallet, feeder_speed: f64, now: SimTime) {
        let t = now as f64 / US_PER_S as f64;
        self.advance_to(t);
        self.incoming = self.incoming.saturating_sub(1);
        pallet.front = pallet.length;
        pallet.entered_at = now;
        let speed = feeder_speed.min(self.data.brake_speed);
        let accel = if speed < self.data.brake_speed - SPEED_EPS { self.accel } else { 0.0 };
        self.pallets.push(GravityPallet { pallet, speed, accel, stopped: false, at_separator: false, start_at: None });
        self.max_queue = self.max_queue.max(self.pallets.len());
        self.resolve(t);
    }

    /// The pallet at rest against the end stop, waiting for the operator.
    pub fn at_end_stop(&self) -> bool {
        self.pallets.first().is_some_and(|p| p.stopped && p.pallet.front >= self.data.length - EPS)
    }

    /// The operator takes the pallet at the end stop; the separator opens `separatorTime` later.
    pub fn take(&mut self, now: SimTime) -> Option<Pallet> {
        if !self.at_end_stop() {
            return None;
        }
        let first = self.pallets.remove(0);
        self.delivered += 1;
        self.transit_sum += now - first.pallet.entered_at;
        if self.holder == Some(first.pallet.id) {
            self.holder = None;
        }
        let t = now as f64 / US_PER_S as f64;
        self.open_at = Some(t + self.data.separator_time);
        self.resolve(t);
        Some(first.pallet)
    }

    /// Moves every pallet to `t` (s) and settles contacts, starts and the separator then.
    pub fn advance_to(&mut self, t: f64) {
        let dt = t - self.t0;
        if dt > 0.0 {
            for p in self.pallets.iter_mut().filter(|p| !p.stopped) {
                let moving = if p.accel < 0.0 { dt.min(p.speed / -p.accel) } else { dt };
                p.pallet.front += p.speed * moving + 0.5 * p.accel * moving * moving;
                p.speed = (p.speed + p.accel * moving).max(0.0);
            }
            self.t0 = t;
        }
        self.resolve(t);
    }

    pub fn front_at(&self, p: &GravityPallet, t: f64) -> f64 {
        if p.stopped {
            return p.pallet.front;
        }
        let dt = (t - self.t0).max(0.0);
        let moving = if p.accel < 0.0 { dt.min(p.speed / -p.accel) } else { dt };
        p.pallet.front + p.speed * moving + 0.5 * p.accel * moving * moving
    }

    pub fn speed_at(&self, p: &GravityPallet, t: f64) -> f64 {
        if p.stopped {
            return 0.0;
        }
        (p.speed + p.accel * (t - self.t0).max(0.0)).min(self.data.brake_speed).max(0.0)
    }

    /// The earliest predicted change, s, if anything will move.
    pub fn next_change(&self) -> Option<f64> {
        let mut times: Vec<f64> = Vec::new();
        times.extend(self.open_at);
        for (i, p) in self.pallets.iter().enumerate() {
            if p.stopped {
                times.extend(p.start_at);
                continue;
            }
            let ahead = i.checked_sub(1).map(|k| &self.pallets[k]);
            if p.accel > 0.0 {
                times.push(self.t0 + (self.data.brake_speed - p.speed) / p.accel);
            }
            if p.accel < 0.0 {
                times.push(self.t0 + p.speed / -p.accel);
            }
            let (x, kind) = self.limit(p, ahead);
            let gap = x - p.pallet.front;
            let contact = match ahead {
                Some(a) if kind == LimitKind::Pallet && !a.stopped => first_root(gap, a.speed - p.speed, a.accel - p.accel),
                _ => reach(p, gap),
            };
            if let Some(c) = contact {
                times.push(self.t0 + c);
            }
            if self.holder.is_none()
                && !self.separator_closed
                && p.pallet.front < self.separator_x - EPS
                && let Some(r) = reach(p, self.separator_x - p.pallet.front)
            {
                times.push(self.t0 + r);
            }
            if self.holder == Some(p.pallet.id)
                && !self.separator_closed
                && let Some(r) = reach(p, self.separator_x - tail(p))
            {
                times.push(self.t0 + r);
            }
        }
        if let (Some(w), Some(rear)) = (self.waiting_length, self.pallets.last())
            && !rear.stopped
            && tail(rear) < w - EPS
            && let Some(r) = reach(rear, w - tail(rear))
        {
            times.push(self.t0 + r);
        }
        times.into_iter().filter(|t| *t > self.t0 + TIME_EPS).reduce(f64::min)
    }

    pub fn status(&self) -> &'static str {
        if self.pallets.is_empty() && self.incoming == 0 {
            "empty"
        } else if self.waiting_length.is_some() {
            "blocked"
        } else if self.pallets.iter().any(|p| !p.stopped) {
            "working"
        } else {
            "waiting"
        }
    }

    pub fn accumulate(&mut self, seconds: f64) {
        self.total += seconds;
        self.queue_time += self.pallets.len() as f64 * seconds;
        if self.waiting_length.is_some() {
            self.full_time += seconds;
        }
        if self.at_end_stop() {
            self.pickup_wait_time += seconds;
        }
    }

    /// Statistics as the definition's `statistics` list names them (S15 §3).
    pub fn statistics(&self) -> Vec<(&'static str, f64)> {
        let share = |s: f64| if self.total > 0.0 { s / self.total } else { 0.0 };
        vec![
            ("palletsDelivered", self.delivered as f64),
            ("meanQueue", share(self.queue_time)),
            ("maxQueue", self.max_queue as f64),
            ("fullShare", share(self.full_time)),
            ("pickupWaitShare", share(self.pickup_wait_time)),
            ("meanTransitTime", if self.delivered > 0 { self.transit_sum as f64 / self.delivered as f64 / US_PER_S as f64 } else { 0.0 }),
        ]
    }

    /// Settles the line at `t`: the separator, delayed starts, the brake speed, contacts, front to back.
    fn resolve(&mut self, t: f64) {
        if self.open_at.is_some_and(|o| o <= t + TIME_EPS) {
            self.open_at = None;
            self.separator_closed = false;
        }
        for i in 0..self.pallets.len() {
            if self.pallets[i].stopped {
                self.try_start(i, t);
            }
            if !self.pallets[i].stopped {
                let brake = self.data.brake_speed;
                let p = &mut self.pallets[i];
                if p.accel > 0.0 && p.speed >= brake - SPEED_EPS {
                    p.speed = brake;
                    p.accel = 0.0;
                }
                let (x, kind) = self.limit(&self.pallets[i], i.checked_sub(1).map(|k| &self.pallets[k]));
                let ahead = i.checked_sub(1).map(|k| (self.pallets[k].stopped, self.pallets[k].speed, self.pallets[k].accel));
                let p = &mut self.pallets[i];
                if p.pallet.front >= x - EPS {
                    p.pallet.front = x;
                    match ahead {
                        // Touching a moving pallet: carry on with it.
                        Some((false, speed, accel)) if kind == LimitKind::Pallet => {
                            p.speed = speed;
                            p.accel = accel;
                        }
                        _ => stop(p, kind),
                    }
                } else if p.accel <= 0.0 && p.speed <= SPEED_EPS {
                    stop(p, kind);
                }
            }
            let p = &self.pallets[i];
            if self.holder.is_none() && !self.separator_closed && p.pallet.front >= self.separator_x - EPS {
                self.holder = Some(p.pallet.id);
            }
            if self.holder == Some(p.pallet.id) && !self.separator_closed && tail(p) >= self.separator_x - EPS {
                self.separator_closed = true;
            }
        }
    }

    /// Starts a stopped pallet whose way is free: at once after the separator, else at its `start_at`.
    fn try_start(&mut self, i: usize, t: f64) {
        let ahead = i.checked_sub(1).map(|k| &self.pallets[k]);
        let (x, kind) = self.limit(&self.pallets[i], ahead);
        // Touching a pallet that has started doesn't block: the two move off together.
        let leaving = kind == LimitKind::Pallet && ahead.is_some_and(|a| !a.stopped);
        let ahead_stopped = ahead.is_some_and(|a| a.stopped);
        let delay = self.data.start_delay;
        let p = &mut self.pallets[i];
        if x - p.pallet.front <= EPS && !leaving {
            // Still blocked: a pending start waits for the pallet ahead to move again.
            if kind == LimitKind::Pallet && ahead_stopped {
                p.start_at = None;
            }
            return;
        }
        let due = p.at_separator || p.start_at.is_some_and(|s| s <= t + TIME_EPS);
        if !due {
            let start = *p.start_at.get_or_insert(t + delay);
            if start > t + TIME_EPS {
                return;
            }
        }
        if self.accel <= 0.0 {
            return;
        }
        p.stopped = false;
        p.at_separator = false;
        p.start_at = None;
        p.speed = 0.0;
        p.accel = self.accel;
        let started_tail = tail(p);
        // The pallet stopped behind this one follows `startDelay` later.
        if let Some(behind) = self.pallets.get_mut(i + 1)
            && behind.stopped
            && behind.pallet.front >= started_tail - EPS
        {
            behind.start_at = Some(t + delay);
        }
    }

    fn limit(&self, p: &GravityPallet, ahead: Option<&GravityPallet>) -> (f64, LimitKind) {
        let mut limit = (self.data.length, LimitKind::End);
        if self.separator_closed && p.pallet.front <= self.separator_x + EPS && self.separator_x < limit.0 {
            limit = (self.separator_x, LimitKind::Separator);
        }
        if let Some(a) = ahead
            && tail(a) < limit.0
        {
            limit = (tail(a), LimitKind::Pallet);
        }
        limit
    }
}

fn stop(p: &mut GravityPallet, kind: LimitKind) {
    p.stopped = true;
    p.speed = 0.0;
    p.accel = 0.0;
    p.at_separator = kind == LimitKind::Separator;
    p.start_at = None;
}
