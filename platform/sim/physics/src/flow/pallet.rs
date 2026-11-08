//! Pallets of the 3D run (the reference's `pallet-body.ts`): the 2D pallet record plus its rigid body, pallet and load as
//! one body (ADR-0015), and the pose and velocity read after each step.

use crate::bench::BodyId;
use intracraft_des::flow::network::Side;
use intracraft_des::flow::sim::Pallet;
use intracraft_library::reference::PalletType;

use crate::bench::{Part, Shape};
use crate::frame::{Frame, V3, dot};

const PALLET_MATERIAL: &str = "wood";
/// A load travelling on a pallet as a block of cartons (S09 §1.1).
const LOAD_MATERIAL: &str = "cardboard";

pub struct WorldPallet {
    pub pallet: Pallet,
    pub body: BodyId,
    /// The object carrying it: the one under its tail, as in 2D.
    pub carrier: String,
    /// Its underside's centre, and its own x (along its travel when it was made) and z axes in the world.
    pub centre: V3,
    pub x_axis: V3,
    pub z_axis: V3,
    pub velocity: V3,
}

/// Where a pallet lies on a deck, m: its front and rear edges along the travel, the offset of its centre from the deck's
/// axis, the height of its underside above the conveying surface, and its speed along the travel (m/s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub front: f64,
    pub rear: f64,
    pub lateral: f64,
    pub lift: f64,
    pub speed: f64,
}

impl WorldPallet {
    /// Where it lies on a deck of `length`: `direction` +1 measures the travel from the deck's x = 0 end, −1 from its
    /// x = length end. The footprint is projected on the deck's axis, so a turned pallet reaches further.
    pub fn extent_on(&self, frame: &Frame, length: f64, direction: f64) -> Extent {
        let local = frame.to_local(self.centre);
        let along = frame.rotate([1.0, 0.0, 0.0]);
        let half = (dot(self.x_axis, along).abs() * self.pallet.length + dot(self.z_axis, along).abs() * self.pallet.width) / 2.0;
        let centre = if direction > 0.0 { local[0] } else { length - local[0] };
        Extent {
            front: centre + half,
            rear: centre - half,
            lateral: local[2],
            lift: local[1],
            speed: direction * dot(self.velocity, along),
        }
    }

    /// Its turn about +y, rad.
    pub fn yaw(&self) -> f64 {
        (-self.x_axis[2]).atan2(self.x_axis[0])
    }
}

/// The parts of a pallet's body, the centre of its underside at the origin and its front toward +x: the pallet type's
/// construction as the 2D sensor sees it (S06, T06), each band of height a row of boards or blocks across the whole
/// width, sharing the tare mass; and the load as a block of its mass on top.
pub fn pallet_parts(p: &Pallet, pallet_type: &PalletType) -> Vec<Part> {
    let bands = if p.along == Side::Long { &pallet_type.occupancy.along_length } else { &pallet_type.occupancy.along_width };
    let mut boxes: Vec<(V3, V3)> = Vec::new();
    for band in bands {
        let (from, to) = (band.from_height.max(0.0), band.to_height.min(p.height));
        for &[a, b] in &band.solid {
            let (a, b) = (a.max(0.0), b.min(p.length));
            if to > from && b > a {
                boxes.push(([(b - a) / 2.0, (to - from) / 2.0, p.width / 2.0], [p.length / 2.0 - (a + b) / 2.0, (from + to) / 2.0, 0.0]));
            }
        }
    }
    // Without a profile the pallet is solid.
    if boxes.is_empty() {
        boxes.push(([p.length / 2.0, p.height / 2.0, p.width / 2.0], [0.0, p.height / 2.0, 0.0]));
    }
    let volume: f64 = boxes.iter().map(|(h, _)| h[0] * h[1] * h[2]).sum();
    let mut parts: Vec<Part> = boxes
        .into_iter()
        .map(|(half, offset)| Part {
            shape: Shape::Cuboid(half),
            offset,
            material: PALLET_MATERIAL.into(),
            friction: None,
            mass: pallet_type.tare_mass * half[0] * half[1] * half[2] / volume,
        })
        .collect();
    if p.load_mass > 0.0 && p.load_height > 0.0 {
        parts.push(Part {
            shape: Shape::Cuboid([p.length / 2.0, p.load_height / 2.0, p.width / 2.0]),
            offset: [0.0, p.height + p.load_height / 2.0, 0.0],
            material: LOAD_MATERIAL.into(),
            friction: None,
            mass: p.load_mass,
        });
    }
    parts
}
