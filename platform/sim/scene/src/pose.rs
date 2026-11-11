//! Where a run's pallets are, for a 3D view: the centre of each one's underside and its turn as a quaternion, its own x
//! along its pallet type's length, the same for the discrete run (from its snapshot) and the physics run (from the
//! bodies' axes).

use std::collections::BTreeMap;

use intracraft_des::flow::sim::PalletView;
use intracraft_des::flow::storage::{AisleView, STATION_X, StorageData};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::{Instance, Layout};
use intracraft_library::params::{instance_values, num};

/// A pallet as a 3D view draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct PalletShow {
    pub id: u64,
    pub type_id: String,
    /// The source that made it, which says its load.
    pub source: String,
    /// The centre of its underside (m, y up) and its turn (x, y, z, w).
    pub centre: [f64; 3],
    pub rotation: [f64; 4],
    /// Its own height, where its load stands, and the load's height (m).
    pub height: f64,
    pub load_height: f64,
    /// Its load is wrapped in film (S25): drawn in its wrapped look.
    pub wrapped: bool,
}

/// Where a pallet of the discrete run is: on its carrier's path, at the carrier's surface (a lift's deck height, a gravity
/// line's slope).
pub fn discrete_pallet(layout: &Layout, library: &Library, p: &PalletView, heights: &BTreeMap<String, f64>) -> Option<PalletShow> {
    let carrier = layout.instance(&p.carrier)?;
    let values = library.get(&carrier.definition_id).map(|d| instance_values(d, &carrier.params)).unwrap_or_default();
    let along = p.front - p.length / 2.0;
    let (c, s) = (p.heading.cos(), p.heading.sin());
    let local = (p.path_x + along * c, p.path_z - along * s);
    let surface = match heights.get(&p.carrier) {
        Some(h) => *h,
        None if p.slope != 0.0 => num(&values, "exitHeight", 0.1) + p.slope * (num(&values, "length", 0.0) - along),
        None => {
            ["surfaceHeight", "conveyorHeight", "entryHeight"].iter().find_map(|k| values.get(*k).and_then(Value::number)).unwrap_or(0.6)
        }
    };
    let centre = world(carrier, local, surface);
    let travel = mul(about_y(carrier.yaw + p.heading), about_z(-p.slope.atan()));
    let rotation = if along_width(library, &p.type_id, p.length) { mul(travel, about_y(std::f64::consts::FRAC_PI_2)) } else { travel };
    Some(PalletShow {
        id: p.id,
        type_id: p.type_id.clone(),
        source: p.source.clone(),
        centre,
        rotation,
        height: p.height,
        load_height: p.load_height,
        wrapped: p.wrapped,
    })
}

/// What the storage aisles of a run show (S26): their pallets, in the rack and on the forks, and how far each part of
/// their cranes has moved from where the scene draws it (`<aisle id>#<part>`, m, world).
pub fn storage_show(
    layout: &Layout,
    library: &Library,
    storage: &BTreeMap<String, AisleView>,
) -> (Vec<PalletShow>, BTreeMap<String, [f64; 3]>) {
    let (mut pallets, mut shifts) = (Vec::new(), BTreeMap::new());
    for (id, view) in storage {
        let Some(aisle) = layout.instance(id) else { continue };
        let values = library.get(&aisle.definition_id).map(|d| instance_values(d, &aisle.params)).unwrap_or_default();
        let rest = crate::crane_rest(&StorageData::read(&values));
        let (s, c) = aisle.yaw.sin_cos();
        let turned = |[x, y, z]: [f64; 3]| [x * c + z * s, y, -x * s + z * c];
        let [x, y, reach] = view.crane;
        let moved = [[x - STATION_X, 0.0, 0.0], [x - STATION_X, y - rest, 0.0], [x - STATION_X, y - rest, reach]];
        for (part, by) in crate::CRANE_PARTS.iter().zip(moved) {
            shifts.insert(format!("{id}#{part}"), turned(by));
        }
        for p in view.stored.iter().chain(&view.carried) {
            // Its side along the aisle is its type's width: the model, whose x is its length, lies across.
            let along = about_y(aisle.yaw);
            pallets.push(PalletShow {
                id: p.id,
                type_id: p.type_id.clone(),
                source: p.source.clone(),
                centre: world(aisle, (p.at[0], p.at[2]), p.at[1]),
                rotation: if along_width(library, &p.type_id, p.length) { mul(along, about_y(std::f64::consts::FRAC_PI_2)) } else { along },
                height: p.height,
                load_height: p.load_height,
                wrapped: p.wrapped,
            });
        }
    }
    (pallets, shifts)
}

/// A body's turn from its own x and z axes in the world; a pallet that travels short side first (its length along the
/// travel not its type's length) is turned a quarter, so the model's x lies along its type's length.
pub fn rotation_of(x: [f64; 3], z: [f64; 3], along_width: bool) -> [f64; 4] {
    let x = unit(x, [1.0, 0.0, 0.0]);
    let z = unit(z, [0.0, 0.0, 1.0]);
    let y = unit(cross(z, x), [0.0, 1.0, 0.0]);
    let z = cross(x, y);
    let travel = from_axes(x, y, z);
    if along_width { mul(travel, about_y(std::f64::consts::FRAC_PI_2)) } else { travel }
}

/// Whether a pallet of `type_id` whose length along its travel is `along` goes short side first.
pub fn along_width(library: &Library, type_id: &str, along: f64) -> bool {
    library.reference.pallet_type(type_id).is_some_and(|t| (t.length - along).abs() > 1e-3)
}

/// A point `(x, z)` of an instance's own frame at `height` above its base, in the world.
fn world(i: &Instance, (x, z): (f64, f64), height: f64) -> [f64; 3] {
    let (s, c) = i.yaw.sin_cos();
    [i.position.x + x * c + z * s, i.position.y + height, i.position.z - x * s + z * c]
}

/// The turn by `a` about +y (as an instance's yaw: +x toward −z).
pub fn about_y(a: f64) -> [f64; 4] {
    let (s, c) = (a / 2.0).sin_cos();
    [0.0, s, 0.0, c]
}

/// The turn by `a` about +z.
pub fn about_z(a: f64) -> [f64; 4] {
    let (s, c) = (a / 2.0).sin_cos();
    [0.0, 0.0, s, c]
}

/// The turn `a` after `b` (`a · b`).
pub fn mul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// A vector turned by `q`.
pub fn rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let u = [q[0], q[1], q[2]];
    let t = cross(u, v).map(|c| 2.0 * c);
    let w = cross(u, t);
    [v[0] + q[3] * t[0] + w[0], v[1] + q[3] * t[1] + w[1], v[2] + q[3] * t[2] + w[2]]
}

/// The turn whose columns are the unit axes `x`, `y`, `z`.
fn from_axes(x: [f64; 3], y: [f64; 3], z: [f64; 3]) -> [f64; 4] {
    let trace = x[0] + y[1] + z[2];
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(y[2] - z[1]) / s, (z[0] - x[2]) / s, (x[1] - y[0]) / s, 0.25 * s]
    } else if x[0] > y[1] && x[0] > z[2] {
        let s = (1.0 + x[0] - y[1] - z[2]).sqrt() * 2.0;
        [0.25 * s, (y[0] + x[1]) / s, (z[0] + x[2]) / s, (y[2] - z[1]) / s]
    } else if y[1] > z[2] {
        let s = (1.0 + y[1] - x[0] - z[2]).sqrt() * 2.0;
        [(y[0] + x[1]) / s, 0.25 * s, (z[1] + y[2]) / s, (z[0] - x[2]) / s]
    } else {
        let s = (1.0 + z[2] - x[0] - y[1]).sqrt() * 2.0;
        [(z[0] + x[2]) / s, (z[1] + y[2]) / s, 0.25 * s, (x[1] - y[0]) / s]
    };
    let n = q.iter().map(|c| c * c).sum::<f64>().sqrt();
    if n > 1e-12 { q.map(|c| c / n) } else { [0.0, 0.0, 0.0, 1.0] }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(v: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if n > 1e-12 { v.map(|c| c / n) } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
    }

    #[test]
    fn a_turn_from_axes_gives_the_axes_back_and_a_quarter_about_y_sends_x_to_minus_z() {
        let q = rotation_of([0.0, 0.0, -1.0], [1.0, 0.0, 0.0], false);
        assert!(close(rotate(q, [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
        assert!(close(rotate(q, [0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]));
        assert!(close(rotate(about_y(std::f64::consts::FRAC_PI_2), [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
        let nose_down = rotation_of([0.0, -1.0, 0.0], [0.0, 0.0, 1.0], false);
        assert!(close(rotate(nose_down, [1.0, 0.0, 0.0]), [0.0, -1.0, 0.0]));
    }

    #[test]
    fn a_short_side_first_pallet_turns_its_model_a_quarter() {
        let q = rotation_of([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], true);
        assert!(rotate(q, [1.0, 0.0, 0.0])[2].abs() > 0.99);
    }
}
