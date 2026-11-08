//! The 3D behaviour of the photoelectric sensor (S06 §3D; the reference's `sensor.ts`): after every physics step a ray from
//! the sensor to the reflector that only pallets interrupt. A beam change is stamped at the step's midpoint; the output
//! follows after the response time (and the filter when the beam frees), with the 2D sensor's own logic.

use std::collections::HashSet;

use crate::bench::BodyId;
use intracraft_des::flow::parts::Photoeye;
use intracraft_des::time::SimTime;

use crate::bench::PhysicsBench;
use crate::frame::{Frame, V3};

/// An output change of a sensor and when it happened.
#[derive(Clone, Debug, PartialEq)]
pub struct SensorEdge {
    pub sensor: String,
    pub time: SimTime,
    pub detecting: bool,
}

pub struct Sensor3D {
    pub logic: Photoeye,
    from: V3,
    span: V3,
    /// An output change waiting for the response time or the filter.
    pending: Option<(bool, SimTime)>,
}

impl Sensor3D {
    /// The sensor on its host's deck: positions along it, across it and above its conveying surface.
    pub fn new(logic: Photoeye, host: &Frame) -> Sensor3D {
        let d = &logic.data;
        let from = host.to_world([d.sensor_s, d.sensor_height, d.sensor_lateral]);
        let to = host.to_world([d.reflector_s, d.reflector_height, d.reflector_lateral]);
        Sensor3D { from, span: [to[0] - from[0], to[1] - from[1], to[2] - from[2]], logic, pending: None }
    }

    /// Casts the beam after a step; a change starts the delay from the step's midpoint `stamp`.
    pub fn sense(&mut self, bench: &dyn PhysicsBench, pallets: &HashSet<BodyId>, stamp: SimTime) {
        // A beam longer than the range is already reported by the network's check; as in 2D it still detects.
        let blocked = bench.beam_blocked(self.from, self.span, pallets);
        if blocked == self.logic.raw {
            return;
        }
        self.logic.raw = blocked;
        self.pending = Some((blocked, stamp + self.logic.delay_for(blocked)));
    }

    /// Commits a pending output change due at `now`; the edge, if the output changed.
    pub fn flush(&mut self, now: SimTime) -> Option<SensorEdge> {
        let (state, due) = self.pending?;
        if due > now {
            return None;
        }
        self.pending = None;
        if state == self.logic.detecting {
            return None;
        }
        self.logic.commit(state);
        Some(SensorEdge { sensor: self.logic.id.clone(), time: due, detecting: state })
    }
}
