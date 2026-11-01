//! The future event list and the next-event clock (T02, T03). Events are ordered by (time, priority, sequence): at one
//! instant a lower priority goes first, and equal priorities in the order they were scheduled, so a run is the same
//! every time. A cancelled event is skipped when its turn comes.

use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap};

use crate::time::SimTime;

#[derive(Clone, Debug, PartialEq)]
pub struct Queued<T> {
    pub time: SimTime,
    pub priority: i32,
    pub seq: u64,
    pub payload: T,
}

/// Heap entry: the order key only, reversed for a min-heap; the payload lives beside it.
struct Entry<T>(Queued<T>);

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<T> Eq for Entry<T> {}
impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let key = |e: &Queued<T>| (e.time, e.priority, e.seq);
        key(&other.0).cmp(&key(&self.0))
    }
}

/// A next-event simulator: schedule events after a delay, cancel them by ticket, and take them in order.
pub struct Simulator<T> {
    heap: BinaryHeap<Entry<T>>,
    cancelled: BTreeSet<u64>,
    next_seq: u64,
    now: SimTime,
}

impl<T> Default for Simulator<T> {
    fn default() -> Self {
        Simulator { heap: BinaryHeap::new(), cancelled: BTreeSet::new(), next_seq: 0, now: 0 }
    }
}

impl<T> Simulator<T> {
    pub fn now(&self) -> SimTime {
        self.now
    }

    /// Schedules `payload` `delay` µs from now (a negative delay is now); its ticket.
    pub fn schedule(&mut self, delay: SimTime, payload: T, priority: i32) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.heap.push(Entry(Queued { time: self.now.saturating_add(delay.max(0)), priority, seq, payload }));
        seq
    }

    pub fn cancel(&mut self, ticket: u64) {
        self.cancelled.insert(ticket);
    }

    /// When the next event that still stands is due.
    pub fn next_event_time(&mut self) -> Option<SimTime> {
        while let Some(top) = self.heap.peek() {
            if !self.cancelled.contains(&top.0.seq) {
                return Some(top.0.time);
            }
            let seq = top.0.seq;
            self.heap.pop();
            self.cancelled.remove(&seq);
        }
        None
    }

    /// The next event due no later than `end`, the clock moved to it; none when there is no such event.
    pub fn pop_until(&mut self, end: SimTime) -> Option<Queued<T>> {
        loop {
            let due = self.heap.peek().is_some_and(|top| top.0.time <= end);
            if !due {
                return None;
            }
            let Entry(event) = self.heap.pop()?;
            if self.cancelled.remove(&event.seq) {
                continue;
            }
            self.now = event.time;
            return Some(event);
        }
    }

    /// Moves the clock to `end` once its events are done (never backwards).
    pub fn advance_to(&mut self, end: SimTime) {
        self.now = self.now.max(end);
    }

    pub fn pending(&self) -> usize {
        self.heap.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_events_by_time_then_priority_then_order() {
        let mut sim = Simulator::default();
        sim.schedule(10, "late", 0);
        sim.schedule(5, "b", 1);
        sim.schedule(5, "a", 0);
        sim.schedule(5, "c", 1);
        let mut order = Vec::new();
        while let Some(e) = sim.pop_until(100) {
            order.push((e.time, e.payload));
        }
        assert_eq!(order, [(5, "a"), (5, "b"), (5, "c"), (10, "late")]);
        sim.advance_to(100);
        assert_eq!(sim.now(), 100);
    }

    #[test]
    fn skips_cancelled_events() {
        let mut sim = Simulator::default();
        let t = sim.schedule(1, 1, 0);
        sim.schedule(2, 2, 0);
        sim.cancel(t);
        assert_eq!(sim.next_event_time(), Some(2));
        assert_eq!(sim.pop_until(10).map(|e| e.payload), Some(2));
        assert!(sim.pop_until(10).is_none());
    }

    #[test]
    fn stops_at_the_end_time_and_schedules_from_the_clock() {
        let mut sim = Simulator::default();
        sim.schedule(50, "x", 0);
        assert!(sim.pop_until(49).is_none());
        sim.advance_to(49);
        sim.schedule(1, "y", 0);
        // Both are due at 50 with the same priority: the one scheduled first goes first.
        assert_eq!(sim.pop_until(50).map(|e| e.payload), Some("x"));
        assert_eq!(sim.pop_until(50).map(|e| e.payload), Some("y"));
        assert_eq!(sim.now(), 50);
    }

    #[test]
    fn the_same_schedule_gives_the_same_order() {
        let run = || {
            let mut sim = Simulator::default();
            for i in 0..1000u64 {
                sim.schedule(((i * 7919) % 97) as i64, i, (i % 3) as i32);
            }
            let mut out = Vec::new();
            while let Some(e) = sim.pop_until(i64::MAX) {
                out.push(e.payload);
            }
            out
        };
        assert_eq!(run(), run());
    }
}
