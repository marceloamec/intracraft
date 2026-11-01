//! Parts of the plant whose logic the 2D run and the 3D run (`intracraft-physics`) share: a source's arrivals, a sink's
//! removal and a photoelectric sensor's output. Each run moves pallets its own way and asks these parts what to do.

use std::collections::BTreeMap;

use intracraft_library::reference::Reference;

use crate::flow::network::{Generation, SensorData, Side, SinkData, SourceData};
use crate::flow::sim::Pallet;
use crate::rng::Pcg32;
use crate::time::{SimTime, US_PER_S, from_seconds};

/// Seconds as the statistics give them.
fn seconds(t: SimTime) -> f64 {
    t as f64 / US_PER_S as f64
}

pub struct Source {
    pub id: String,
    pub data: SourceData,
    pub downstream: Option<String>,
    pub downstream_port: Option<String>,
    /// When the waiting pallet became due.
    pub due_since: Option<SimTime>,
    pub created: u64,
    pub rejected: u64,
    pub waiting: SimTime,
    pub rng: Pcg32,
}

impl Source {
    pub fn new(id: &str, data: SourceData, downstream: Option<String>, downstream_port: Option<String>, seed: u64, stream: u64) -> Source {
        Source {
            id: id.to_string(),
            data,
            downstream,
            downstream_port,
            due_since: None,
            created: 0,
            rejected: 0,
            waiting: 0,
            rng: Pcg32::new(seed, stream),
        }
    }

    pub fn active(&self) -> bool {
        self.data.max_pallets == 0.0 || ((self.created + self.rejected) as f64) < self.data.max_pallets
    }

    /// The delay to the next pallet: the interval, or an exponential draw with the interval as mean.
    pub fn next_delay(&mut self) -> SimTime {
        match self.data.generation {
            Generation::FixedInterval => from_seconds(self.data.interval),
            Generation::Exponential => from_seconds(-self.data.interval * (1.0 - self.rng.next_f64()).ln()),
        }
    }

    pub fn along_length(&self) -> f64 {
        if self.data.orientation == Side::Long { self.data.item.long } else { self.data.item.short }
    }

    /// Its waiting pallet was placed at `now`.
    pub fn placed(&mut self, now: SimTime) {
        self.created += 1;
        self.waiting += now - self.due_since.unwrap_or(now);
        self.due_since = None;
    }

    /// Its waiting pallet found the entry occupied and was dropped.
    pub fn reject(&mut self) {
        self.rejected += 1;
        self.due_since = None;
    }

    pub fn statistics(&self, now: SimTime) -> BTreeMap<&'static str, f64> {
        let waiting_now = self.due_since.map_or(0, |d| now - d);
        BTreeMap::from([
            ("palletsCreated", self.created as f64),
            ("palletsRejected", self.rejected as f64),
            ("waitingTime", seconds(self.waiting + waiting_now)),
        ])
    }
}

pub struct Sink {
    pub id: String,
    pub data: SinkData,
    pub incoming: i64,
    pub busy_until: SimTime,
    pub received: u64,
    pub cycle_sum: SimTime,
}

impl Sink {
    pub fn new(id: &str, data: SinkData) -> Sink {
        Sink { id: id.to_string(), data, incoming: 0, busy_until: 0, received: 0, cycle_sum: 0 }
    }

    /// Received, per hour over `hours` and mean cycle time.
    pub fn statistics(&self, hours: f64) -> BTreeMap<&'static str, f64> {
        BTreeMap::from([
            ("palletsReceived", self.received as f64),
            ("throughputPerHour", if hours > 0.0 { self.received as f64 / hours } else { 0.0 }),
            ("meanCycleTime", if self.received > 0 { seconds(self.cycle_sum) / self.received as f64 } else { 0.0 }),
        ])
    }

    /// With a removal time the sink takes one pallet at a time; without, it is always free.
    pub fn accepts(&self, now: SimTime) -> bool {
        self.data.removal_time == 0.0 || (self.incoming == 0 && now >= self.busy_until)
    }

    pub fn receive(&mut self, p: &Pallet, now: SimTime) {
        self.incoming -= 1;
        self.received += 1;
        self.cycle_sum += now - p.created_at;
        self.busy_until = now + from_seconds(self.data.removal_time);
    }
}

pub type Interval = (f64, f64);

/// A photoelectric sensor (S06, T06): the beam is the segment sensor → reflector in the host's frame; each solid part
/// of a pallet blocks it while the pallet's front lies in one interval, exactly, so every edge is predicted.
pub struct Photoeye {
    pub id: String,
    pub data: SensorData,
    pub raw: bool,
    pub detecting: bool,
    pub pending: Option<u64>,
    pub pulses: u64,
    pub occupied: f64,
    pub total: f64,
    pub cache: BTreeMap<String, Vec<Interval>>,
    pub signal: String,
}

impl Photoeye {
    pub fn new(id: &str, data: SensorData) -> Photoeye {
        Photoeye {
            id: id.to_string(),
            signal: if data.output_signal.is_empty() { format!("{id}.detected") } else { data.output_signal.clone() },
            data,
            raw: false,
            detecting: false,
            pending: None,
            pulses: 0,
            occupied: 0.0,
            total: 0.0,
            cache: BTreeMap::new(),
        }
    }

    pub fn accumulate(&mut self, seconds: f64) {
        self.total += seconds;
        if self.detecting {
            self.occupied += seconds;
        }
    }

    pub fn statistics(&self) -> BTreeMap<&'static str, f64> {
        BTreeMap::from([
            ("pulses", self.pulses as f64),
            ("occupiedTime", self.occupied),
            ("occupancy", if self.total > 0.0 { self.occupied / self.total } else { 0.0 }),
        ])
    }

    pub fn signal_value(&self) -> bool {
        self.detecting != self.data.active_on_dark
    }

    pub fn intervals(&mut self, p: &Pallet, reference: &Reference) -> Vec<Interval> {
        let key = format!("{}|{:?}|{}|{}|{}|{}", p.type_id, p.along, p.length, p.width, p.height, p.load_height);
        if let Some(c) = self.cache.get(&key) {
            return c.clone();
        }
        let result = match beam_span(&self.data, p.width / 2.0) {
            None => vec![],
            Some((from, to)) => merge(solid_parts(p, self.data.beam_height, reference).iter().map(|(a, b)| (from + a, to + b)).collect()),
        };
        self.cache.insert(key, result.clone());
        result
    }

    /// The output's delay after a beam change, µs: the response time, plus the filter when the beam frees.
    pub fn delay_for(&self, blocked: bool) -> SimTime {
        from_seconds(self.data.response_time + if blocked { 0.0 } else { self.data.filter })
    }

    pub fn commit(&mut self, state: bool) {
        if state == self.detecting {
            return;
        }
        if state {
            self.pulses += 1;
        }
        self.detecting = state;
    }
}

/// Positions along travel where the beam lies over the pallet's width (|lateral| ≤ half width); none when it passes
/// beside the pallet.
fn beam_span(s: &SensorData, half_width: f64) -> Option<Interval> {
    let (s1, z1, s2, z2) = (s.sensor_s, s.sensor_lateral, s.reflector_s, s.reflector_lateral);
    let (mut from, mut to) = (0.0, 1.0);
    if z1 == z2 {
        if z1.abs() > half_width {
            return None;
        }
    } else {
        let a = (-half_width - z1) / (z2 - z1);
        let b = (half_width - z1) / (z2 - z1);
        from = a.min(b).max(0.0);
        to = a.max(b).min(1.0);
        if from > to {
            return None;
        }
    }
    let (sf, st) = (s1 + from * (s2 - s1), s1 + to * (s2 - s1));
    Some((sf.min(st), sf.max(st)))
}

/// The solid parts (m from the pallet's front) a level beam at `height` meets.
fn solid_parts(p: &Pallet, height: f64, reference: &Reference) -> Vec<Interval> {
    if height < 0.0 {
        return vec![];
    }
    if height >= p.height {
        return if height < p.height + p.load_height { vec![(0.0, p.length)] } else { vec![] };
    }
    let bands =
        reference.pallet_type(&p.type_id).map(|t| if p.along == Side::Long { &t.occupancy.along_length } else { &t.occupancy.along_width });
    match bands.and_then(|b| b.iter().find(|b| height >= b.from_height && height < b.to_height)) {
        // Without a profile the pallet is solid.
        None => vec![(0.0, p.length)],
        Some(band) => band.solid.iter().map(|[a, b]| (*a, *b)).collect(),
    }
}

fn merge(mut intervals: Vec<Interval>) -> Vec<Interval> {
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<Interval> = Vec::new();
    for (from, to) in intervals {
        match out.last_mut() {
            Some(last) if from <= last.1 => last.1 = last.1.max(to),
            _ => out.push((from, to)),
        }
    }
    out
}
