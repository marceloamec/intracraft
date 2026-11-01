//! What the Discrete simulation module draws on its plan, in plan metres (x east, y north): each object's footprint and
//! each pallet's outline at a snapshot. Pure geometry, so it is tested without a window.

use intracraft_des::flow::PalletView;
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

/// The plan's bounds, (min, max), with a margin; a unit square for an empty layout.
pub fn bounds(footprints: &[Footprint]) -> (Point, Point) {
    let mut lo = (f64::INFINITY, f64::INFINITY);
    let mut hi = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for c in footprints.iter().flat_map(|f| f.corners) {
        lo = (lo.0.min(c.0), lo.1.min(c.1));
        hi = (hi.0.max(c.0), hi.1.max(c.1));
    }
    if !lo.0.is_finite() {
        return ((-1.0, -1.0), (1.0, 1.0));
    }
    ((lo.0 - 1.0, lo.1 - 1.0), (hi.0 + 1.0, hi.1 + 1.0))
}

/// Whether a plan point lies in a footprint (a convex quadrilateral).
pub fn contains(f: &Footprint, p: Point) -> bool {
    let c = &f.corners;
    let side = |a: Point, b: Point| (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
    let s: Vec<f64> = (0..4).map(|k| side(c[k], c[(k + 1) % 4])).collect();
    s.iter().all(|v| *v >= 0.0) || s.iter().all(|v| *v <= 0.0)
}

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
        assert!(contains(f, (2.0, 2.5)) && !contains(f, (4.0, 2.5)));
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
    fn bounds_frame_every_footprint_with_a_margin() {
        let layout = Layout { instances: vec![instance("1", "conveyor.roller", Vec3::default(), 0.0)], ..Layout::default() };
        let ((x0, y0), (x1, y1)) = bounds(&footprints(&layout));
        assert!(x0 < 0.0 && y0 < -0.5 && x1 > 1.0 && y1 > 0.5);
        assert_eq!(bounds(&[]), ((-1.0, -1.0), (1.0, 1.0)));
    }
}
