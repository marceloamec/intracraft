//! What the simulation modules draw over BIMCraft's views, in plan metres (x east, y north): each object's footprint and
//! each pallet's outline and height at a snapshot, and a pallet as a box of model points for the 3D view. Pure geometry,
//! so it is tested without a window.

use std::collections::BTreeMap;

use intracraft_des::flow::PalletView;
use intracraft_library::definition::Value;
use intracraft_library::layout::{Instance, Layout};
use intracraft_library::params::num;
use intracraft_library::ports::rotate_y;

/// A point of the plan, m: east and north.
pub type Point = (f64, f64);

#[derive(Clone, Debug, PartialEq)]
pub struct Footprint {
    pub id: String,
    pub definition: String,
    pub corners: [Point; 4],
}

/// Along-travel extent (from, to) and half width, m, of an object in its own frame (x along travel from its entry):
/// the same body the Design module's families draw.
fn extent(i: &Instance) -> (f64, f64, f64) {
    let p = &i.params;
    let half = num(p, "usableWidth", num(p, "platformWidth", 1.0)) / 2.0;
    match i.definition_id.as_str() {
        "conveyor.chain_turntable" | "conveyor.roller_turntable" => {
            let l = num(p, "platformLength", 1.4);
            (-l / 2.0, l / 2.0, half)
        }
        "flow.source" => (-0.8, 0.4, 0.5),
        "flow.sink" => (0.0, 1.2, 0.5),
        "sensor.photoeye" => (-0.05, 0.05, 0.05),
        "scenery.asset" => (-0.25, 0.25, 0.25),
        "flow.process_station" => (0.0, num(p, "length", 1.6), 0.5),
        _ => (0.0, num(p, "length", 1.0), half),
    }
}

/// A point of an instance's own frame (x along travel, z across) on the plan.
pub fn to_plan(i: &Instance, x: f64, z: f64) -> Point {
    let (dx, dz) = rotate_y((x, z), i.yaw);
    // The simulation's z points south.
    (i.position.x + dx, -(i.position.z + dz))
}

pub fn footprints(layout: &Layout) -> Vec<Footprint> {
    layout
        .instances
        .iter()
        .map(|i| {
            let (from, to, half) = extent(i);
            Footprint {
                id: i.id.clone(),
                definition: i.definition_id.clone(),
                corners: [to_plan(i, from, -half), to_plan(i, to, -half), to_plan(i, to, half), to_plan(i, from, half)],
            }
        })
        .collect()
}

/// A pallet's outline on the plan: its front at `front` along its carrier's path (which starts at the path's point and
/// runs along its heading: a turntable's deck turns), its length behind it.
pub fn pallet_outline(layout: &Layout, p: &PalletView) -> Option<[Point; 4]> {
    let i = layout.instance(&p.carrier)?;
    let (rear, half) = (p.front - p.length, p.width / 2.0);
    // Along the path: direction (cos h, −sin h) in the carrier's (x, z); across it: (sin h, cos h).
    let (c, s) = (p.heading.cos(), p.heading.sin());
    let at = |along: f64, across: f64| to_plan(i, p.path_x + along * c + across * s, p.path_z - along * s + across * c);
    Some([at(rear, -half), at(p.front, -half), at(p.front, half), at(rear, half)])
}

/// A pallet of the 3D run on the plan: its centre, its turn about +y and its sizes (along its own x, z).
pub fn body_outline(centre: [f64; 3], yaw: f64, size: [f64; 3]) -> [Point; 4] {
    let (hx, hz) = (size[0] / 2.0, size[2] / 2.0);
    let at = |x: f64, z: f64| {
        let (dx, dz) = rotate_y((x, z), yaw);
        (centre[0] + dx, -(centre[2] + dz))
    };
    [at(-hx, -hz), at(hx, -hz), at(hx, hz), at(-hx, hz)]
}

/// The height a pallet of the discrete run rides at, m above the floor: its carrier's conveying surface; a lift station's
/// deck at the height the run reports (`heights`), a gravity line's falling with its slope.
pub fn ride_height(layout: &Layout, p: &PalletView, heights: &BTreeMap<String, f64>) -> f64 {
    let Some(i) = layout.instance(&p.carrier) else { return 0.0 };
    let v = &i.params;
    let surface = match heights.get(&p.carrier) {
        Some(h) => *h,
        None if p.slope != 0.0 => num(v, "exitHeight", 0.1) + p.slope * (num(v, "length", 0.0) - (p.front - p.length / 2.0)),
        None => ["surfaceHeight", "conveyorHeight", "entryHeight"].iter().find_map(|k| v.get(*k).and_then(Value::number)).unwrap_or(0.6),
    };
    i.position.y + surface
}

/// A box standing on a plan outline (m) from `z0` to `z1` (m), as model points (mm): its bottom's corners counter-clockwise
/// seen from above, then its top's.
pub fn solid(outline: [Point; 4], z0: f64, z1: f64) -> [[f64; 3]; 8] {
    let area: f64 = (0..4).map(|k| outline[k].0 * outline[(k + 1) % 4].1 - outline[(k + 1) % 4].0 * outline[k].1).sum();
    let ring = if area < 0.0 { [outline[3], outline[2], outline[1], outline[0]] } else { outline };
    let at = |k: usize, z: f64| [ring[k].0 * 1000.0, ring[k].1 * 1000.0, z * 1000.0];
    [at(0, z0), at(1, z0), at(2, z0), at(3, z0), at(0, z1), at(1, z1), at(2, z1), at(3, z1)]
}

/// The faces of a [`solid`], each counter-clockwise seen from outside: the top, the bottom, the four sides.
pub const FACES: [[usize; 4]; 6] = [[4, 5, 6, 7], [3, 2, 1, 0], [0, 1, 5, 4], [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]];

#[cfg(test)]
mod tests {
    use super::*;
    use intracraft_library::layout::Vec3;

    fn instance(id: &str, definition: &str, position: Vec3, yaw: f64) -> Instance {
        Instance {
            id: id.into(),
            definition_id: definition.into(),
            definition_version: "0.1.0".into(),
            name: id.into(),
            position,
            yaw,
            params: Default::default(),
            attributes: Default::default(),
        }
    }

    fn close(a: Point, b: Point) -> bool {
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
    }

    #[test]
    fn a_conveyor_turned_a_quarter_runs_north() {
        let mut i = instance("1", "conveyor.roller", Vec3::new(2.0, 0.0, -1.0), std::f64::consts::FRAC_PI_2);
        i.params.insert("length".into(), intracraft_library::definition::Value::Number(3.0));
        let layout = Layout { instances: vec![i], ..Layout::default() };
        let f = &footprints(&layout)[0];
        // Its entry is at (2, 1) on the plan, and it runs 3 m north.
        assert!(close(to_plan(&layout.instances[0], 0.0, 0.0), (2.0, 1.0)));
        assert!(close(to_plan(&layout.instances[0], 3.0, 0.0), (2.0, 4.0)));
        assert!(f.corners.iter().all(|c| (c.0 - 2.0).abs() <= 0.5 + 1e-9 && c.1 >= 1.0 - 1e-9 && c.1 <= 4.0 + 1e-9));
    }

    #[test]
    fn a_pallet_sits_behind_its_front_on_its_carrier() {
        let layout = Layout { instances: vec![instance("7", "conveyor.roller", Vec3::default(), 0.0)], ..Layout::default() };
        let p = PalletView {
            id: 1,
            type_id: "eur".into(),
            carrier: "7".into(),
            front: 2.0,
            path_x: 0.0,
            path_z: 0.0,
            heading: 0.0,
            slope: 0.0,
            speed: 0.25,
            accel: 0.0,
            length: 1.2,
            width: 0.8,
            height: 0.144,
            load_height: 1.0,
        };
        let o = pallet_outline(&layout, &p).unwrap();
        assert!(close(o[0], (0.8, 0.4)) && close(o[2], (2.0, -0.4)));
        assert!(pallet_outline(&Layout::default(), &p).is_none());
    }

    #[test]
    fn a_pallet_on_a_turned_deck_turns_with_it() {
        let layout = Layout { instances: vec![instance("9", "conveyor.roller_turntable", Vec3::default(), 0.0)], ..Layout::default() };
        let h = std::f64::consts::FRAC_PI_2;
        // A deck heading +90° (north) with its entry edge 0.7 m south of the centre: a centred pallet sits on the centre.
        let p = PalletView {
            id: 1,
            type_id: "eur".into(),
            carrier: "9".into(),
            front: 1.3,
            path_x: -0.7 * h.cos(),
            path_z: 0.7 * h.sin(),
            heading: h,
            slope: 0.0,
            speed: 0.0,
            accel: 0.0,
            length: 1.2,
            width: 0.8,
            height: 0.144,
            load_height: 1.0,
        };
        let o = pallet_outline(&layout, &p).unwrap();
        let centre = ((o[0].0 + o[2].0) / 2.0, (o[0].1 + o[2].1) / 2.0);
        assert!(close(centre, (0.0, 0.0)), "{centre:?}");
        // It runs north: its front edge is 0.6 m north of the centre.
        assert!((o[1].1 - 0.6).abs() < 1e-9 && (o[2].1 - 0.6).abs() < 1e-9);
    }

    #[test]
    fn a_body_turned_a_quarter_lies_north_south() {
        let o = body_outline([1.0, 0.7, -2.0], std::f64::consts::FRAC_PI_2, [1.2, 0.2, 0.8]);
        // Its centre is at (1, 2) on the plan and its long side runs north.
        let centre = ((o[0].0 + o[2].0) / 2.0, (o[0].1 + o[2].1) / 2.0);
        assert!(close(centre, (1.0, 2.0)));
        assert!(((o[1].1 - o[0].1).abs() - 1.2).abs() < 1e-9);
    }

    #[test]
    fn a_box_on_a_clockwise_outline_turns_its_bottom_counter_clockwise() {
        let cw = [(0.0, 0.0), (0.0, 1.0), (2.0, 1.0), (2.0, 0.0)];
        let b = solid(cw, 0.6, 0.744);
        let area: f64 = (0..4).map(|k| b[k][0] * b[(k + 1) % 4][1] - b[(k + 1) % 4][0] * b[k][1]).sum();
        assert!(area > 0.0);
        assert_eq!((b[0][2], b[4][2]), (600.0, 744.0));
    }

    #[test]
    fn a_pallet_rides_on_its_carriers_surface_or_its_lift_deck() {
        let mut i = instance("7", "conveyor.roller", Vec3::new(0.0, 4.0, 0.0), 0.0);
        i.params.insert("surfaceHeight".into(), Value::Number(0.8));
        let layout = Layout { instances: vec![i], ..Layout::default() };
        let p = PalletView {
            id: 1,
            type_id: "eur".into(),
            carrier: "7".into(),
            front: 2.0,
            path_x: 0.0,
            path_z: 0.0,
            heading: 0.0,
            slope: 0.0,
            speed: 0.0,
            accel: 0.0,
            length: 1.2,
            width: 0.8,
            height: 0.144,
            load_height: 1.0,
        };
        assert!((ride_height(&layout, &p, &BTreeMap::new()) - 4.8).abs() < 1e-12);
        assert!((ride_height(&layout, &p, &BTreeMap::from([("7".to_string(), 2.0)])) - 6.0).abs() < 1e-12);
    }
}
