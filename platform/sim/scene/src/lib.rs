//! The 3D scene of a layout, engine-free (ADR-0026): each object's parts as boxes and cylinders in the world (SI, y up),
//! fixed, or moving with its platform (a turntable turns, a lift table's deck and an elevator's carriage go up and down).
//! A viewer draws the parts and moves an object's moving parts by the run's turntable angles and deck heights.
//!
//! Our own geometry, after the families' forms (`intracraft-model::forms`): rollers on their pitch between steel side
//! frames, chain strands on their tracks, legs on feet at most 2 m apart with a cross beam, the geared motor; each
//! station's platform, carriage or gantry; the source's and sink's stands; a photocell's posts, heads and beam. The deck
//! sizes, roller positions and chain strands are the ones the physics run builds, so the picture and the bodies agree.

use intracraft_des::flow::storage::{STATION_X, StorageData};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::{Instance, Layout};
use intracraft_library::params::{Values, instance_values, num};

mod kit;
pub mod pose;

use kit::Kit;

/// The paints of the parts: the families' materials (`intracraft-model::forms::MATERIALS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Paint {
    Frame,
    Roller,
    Chain,
    Support,
    Base,
    Deck,
    Guard,
    Drive,
    Brake,
    Source,
    Sink,
    Beam,
    Gantry,
}

impl Paint {
    /// Its colour, sRGB.
    pub fn color(self) -> [u8; 3] {
        match self {
            Paint::Frame => [0x2f, 0x6f, 0xb3],
            Paint::Roller => [0x9a, 0xa3, 0xad],
            Paint::Chain => [0x3b, 0x3f, 0x45],
            Paint::Support => [0x5b, 0x64, 0x70],
            Paint::Base => [0x4a, 0x52, 0x5d],
            Paint::Deck => [0x6d, 0x77, 0x82],
            Paint::Guard => [0xd9, 0xa6, 0x2e],
            Paint::Drive => [0x3f, 0x5f, 0x7a],
            Paint::Brake => [0xc4, 0x47, 0x3a],
            Paint::Source => [0x3c, 0x9d, 0x5a],
            Paint::Sink => [0xc2, 0x45, 0x3d],
            Paint::Beam => [0x3d, 0xdc, 0x84],
            Paint::Gantry => [0xc9, 0x82, 0x2b],
        }
    }

    /// Bright steel or painted steel.
    pub fn metallic(self) -> f32 {
        match self {
            Paint::Roller => 0.85,
            Paint::Chain => 0.6,
            _ => 0.2,
        }
    }

    pub fn roughness(self) -> f32 {
        match self {
            Paint::Roller => 0.3,
            Paint::Chain => 0.45,
            _ => 0.6,
        }
    }
}

/// A part's shape in its own frame: a box by its sizes, a roller (a cylinder along its own z) or a post (a cylinder along
/// its own y).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Box([f64; 3]),
    Roller { radius: f64, length: f64 },
    Post { radius: f64, height: f64 },
}

/// A part in the world: its centre, its turn about +y (as an instance's yaw: +x toward −z) and then its tilt down along
/// its own x (a gravity line's slope, rad).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub centre: [f64; 3],
    pub yaw: f64,
    pub tilt: f64,
    pub paint: Paint,
}

/// How an object's moving parts move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mover {
    None,
    /// They turn about a vertical axis through `pivot` by the turntable's angle (rad, its own frame, 0 as drawn).
    Turn {
        pivot: [f64; 3],
    },
    /// They rise by the deck's height above the object's base less `rest`, the height they are drawn at.
    Lift {
        rest: f64,
    },
    /// They move by the shift the run gives this object (m, world): a part of a stacker crane.
    Shift,
}

/// An object of the layout as drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub fixed: Vec<Part>,
    pub moving: Vec<Part>,
    pub mover: Mover,
}

impl Object {
    /// The instance of the layout it belongs to: its own id, or for a part of an object (`<id>#<part>`) that object's.
    pub fn instance(&self) -> &str {
        self.id.split_once('#').map_or(self.id.as_str(), |(instance, _)| instance)
    }

    /// The corners of the box round all its parts as drawn.
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        self.fixed
            .iter()
            .chain(&self.moving)
            .map(part_bounds)
            .reduce(|(lo, hi), (a, b)| (std::array::from_fn(|i| lo[i].min(a[i])), std::array::from_fn(|i| hi[i].max(b[i]))))
    }
}

/// The scene of a layout: one object per instance the library knows, and the parts of a storage aisle's crane after it.
pub fn scene(layout: &Layout, library: &Library) -> Vec<Object> {
    let mut out = Vec::new();
    for i in &layout.instances {
        let Some(definition) = library.get(&i.definition_id) else { continue };
        let values = instance_values(definition, &i.params);
        out.push(object(layout, library, i, &values));
        if i.definition_id == "storage.aisle" {
            out.extend(crane(i, &values));
        }
    }
    out
}

/// The parts of a storage aisle's stacker crane that move on their own, each an object `<aisle id>#<part>`: the mast
/// travels, the carriage also lifts, the forks also reach across.
pub const CRANE_PARTS: [&str; 3] = ["mast", "carriage", "forks"];

/// The crane of storage aisle `i` as three objects, drawn at rest: its forks drawn in at the first station height.
fn crane(i: &Instance, v: &Values) -> Vec<Object> {
    let data = StorageData::read(v);
    let (x, y) = (STATION_X, crane_rest(&data));
    CRANE_PARTS
        .iter()
        .map(|part| {
            let mut k = Kit::new([i.position.x, i.position.y, i.position.z], i.yaw);
            match *part {
                "mast" => k.crane_mast(&data, x),
                "carriage" => k.crane_carriage(&data, x, y),
                _ => k.crane_forks(&data, x, y),
            }
            let (fixed, moving) = k.finish();
            Object {
                id: format!("{}#{part}", i.id),
                name: i.name.clone(),
                definition: i.definition_id.clone(),
                fixed,
                moving,
                mover: Mover::Shift,
            }
        })
        .collect()
}

/// The height a crane's forks are drawn at, m above the aisle's base: its first station height.
pub fn crane_rest(data: &StorageData) -> f64 {
    data.station_heights.first().copied().unwrap_or(0.6)
}

/// The corners of the box round a whole scene.
pub fn bounds(objects: &[Object]) -> Option<([f64; 3], [f64; 3])> {
    objects
        .iter()
        .filter_map(Object::bounds)
        .reduce(|(lo, hi), (a, b)| (std::array::from_fn(|i| lo[i].min(a[i])), std::array::from_fn(|i| hi[i].max(b[i]))))
}

fn object(layout: &Layout, library: &Library, i: &Instance, v: &Values) -> Object {
    let mut k = Kit::new([i.position.x, i.position.y, i.position.z], i.yaw);
    let mover = match i.definition_id.as_str() {
        "conveyor.roller" => {
            let (length, width, top) = (num(v, "length", 3.0), num(v, "usableWidth", 1.0), num(v, "surfaceHeight", 0.6));
            k.roller_deck(0.0, length, width, top, v, false);
            k.side_frames(0.0, length, width, top - kit::FRAME_DROP, false);
            k.legs(0.0, length, width, top - kit::FRAME_DROP - kit::FRAME_H);
            k.drive([length / 2.0, top - 0.3, width / 2.0 + 0.2]);
            Mover::None
        }
        "conveyor.chain" => {
            let (length, width, top) = (num(v, "length", 3.0), num(v, "usableWidth", 1.3), num(v, "surfaceHeight", 0.6));
            k.strands(0.0, length, width, top, v, false);
            k.side_frames(0.0, length, width, top - kit::CHAIN_H - kit::TRACK_H, false);
            k.legs(0.0, length, width, top - kit::CHAIN_H - kit::TRACK_H - kit::FRAME_H);
            k.drive([length - 0.15, top - 0.35, width / 2.0 + 0.2]);
            Mover::None
        }
        "conveyor.roller_turntable" | "conveyor.chain_turntable" => {
            let chains = i.definition_id == "conveyor.chain_turntable";
            let (length, width, top) = (num(v, "platformLength", 1.4), num(v, "platformWidth", 1.2), num(v, "surfaceHeight", 0.6));
            let pivot = [num(v, "pivotOffsetX", 0.0), 0.0, num(v, "pivotOffsetZ", 0.0)];
            k.turntable_base(pivot, top);
            if chains {
                k.strands(-length / 2.0, length, width, top, v, true);
            } else {
                k.roller_deck(-length / 2.0, length, width - 2.0 * kit::TURNTABLE_EDGE, top, v, true);
            }
            k.side_frames(-length / 2.0, length, width - 2.0 * kit::TURNTABLE_EDGE, top - kit::FRAME_DROP, true);
            k.moving(Shape::Box([length, 0.04, width]), [0.0, top - kit::FRAME_DROP - kit::FRAME_H - 0.02, 0.0], Paint::Deck);
            Mover::Turn { pivot: k.world(pivot) }
        }
        "conveyor.transfer_rc" => {
            let (length, width, top) = (num(v, "length", 1.4), num(v, "usableWidth", 1.0), num(v, "surfaceHeight", 0.6));
            k.roller_deck(0.0, length, width, top, v, false);
            k.side_frames(0.0, length, width, top - kit::FRAME_DROP, false);
            k.transfer_chains(length, width, top, v);
            k.legs(0.0, length, width, top - kit::FRAME_DROP - kit::FRAME_H);
            Mover::None
        }
        "flow.process_station" => {
            let (length, top) = (num(v, "length", 1.6), num(v, "surfaceHeight", 0.6));
            let width = num(v, "usableWidth", 1.0);
            k.roller_deck(0.0, length, width, top, v, false);
            k.side_frames(0.0, length, width, top - kit::FRAME_DROP, false);
            k.legs(0.0, length, width, top - kit::FRAME_DROP - kit::FRAME_H);
            k.gantry(length, width, top);
            Mover::None
        }
        "flow.stretch_wrapper" => {
            k.wrapper(v);
            Mover::Turn { pivot: k.world([num(v, "length", 1.8) / 2.0, 0.0, 0.0]) }
        }
        "conveyor.lift_table_manual" | "conveyor.scissor_lift" | "conveyor.elevator" => {
            let (length, width) = (num(v, "length", 1.6), num(v, "usableWidth", 1.0));
            let rest = lift_rest(i, v);
            if i.definition_id == "conveyor.elevator" {
                let top = num(v, "entryHeight", 0.6).max(num(v, "exitHeight", 4.6));
                k.elevator_masts(length, width, top);
            } else {
                k.fixed(Shape::Box([length, 0.06, width + 0.2]), [length / 2.0, 0.03, 0.0], Paint::Base);
            }
            let chains = v.get("deckType").and_then(Value::text) == Some("chains");
            if chains {
                k.strands(0.0, length, width, rest, v, true);
            } else {
                k.roller_deck(0.0, length, width, rest, v, true);
            }
            k.side_frames(0.0, length, width, rest - kit::FRAME_DROP, true);
            k.moving(Shape::Box([length, 0.03, width + 0.12]), [length / 2.0, rest - kit::FRAME_DROP - kit::FRAME_H, 0.0], Paint::Deck);
            Mover::Lift { rest }
        }
        "conveyor.gravity" => {
            k.gravity_line(v);
            Mover::None
        }
        "storage.aisle" => {
            k.rack(&StorageData::read(v));
            Mover::None
        }
        "flow.source" | "flow.sink" => {
            let source = i.definition_id == "flow.source";
            let (from, to) = if source { (-0.8, 0.4) } else { (0.0, 1.2) };
            k.stand(from, to, 1.0, num(v, "surfaceHeight", 0.6), if source { Paint::Source } else { Paint::Sink });
            Mover::None
        }
        "sensor.photoeye" => {
            if let Some(host) = v.get("host").and_then(Value::text).and_then(|h| layout.instance(h)) {
                let host_values = library.get(&host.definition_id).map(|d| instance_values(d, &host.params)).unwrap_or_default();
                let mut on_host = Kit::new([host.position.x, host.position.y, host.position.z], host.yaw);
                on_host.photoeye(v, deck_top(host, &host_values));
                k = on_host;
            }
            Mover::None
        }
        _ => Mover::None,
    };
    let (fixed, moving) = k.finish();
    Object { id: i.id.clone(), name: i.name.clone(), definition: i.definition_id.clone(), fixed, moving, mover }
}

/// The height a lift station's deck is drawn at (its rest, m above its base): where it takes pallets in.
fn lift_rest(i: &Instance, v: &Values) -> f64 {
    match i.definition_id.as_str() {
        "conveyor.lift_table_manual" => {
            if v.get("flowDirection").and_then(Value::text) == Some("out") {
                num(v, "conveyorHeight", 0.6)
            } else {
                num(v, "lowHeight", 0.1)
            }
        }
        _ => num(v, "entryHeight", 0.6),
    }
}

/// The height of an object's conveying surface above its base, m.
fn deck_top(i: &Instance, v: &Values) -> f64 {
    match i.definition_id.as_str() {
        "conveyor.lift_table_manual" | "conveyor.scissor_lift" | "conveyor.elevator" => lift_rest(i, v),
        "conveyor.gravity" => num(v, "exitHeight", 0.1),
        _ => num(v, "surfaceHeight", 0.6),
    }
}

/// The box round a part, whatever its turn (a box round its turned box).
fn part_bounds(p: &Part) -> ([f64; 3], [f64; 3]) {
    let half = match p.shape {
        Shape::Box(s) => [s[0] / 2.0, s[1] / 2.0, s[2] / 2.0],
        Shape::Roller { radius, length } => [radius, radius, length / 2.0],
        Shape::Post { radius, height } => [radius, height / 2.0, radius],
    };
    let (s, c) = (p.yaw.sin().abs(), p.yaw.cos().abs());
    let reach = [half[0] * c + half[2] * s, half[1] + half[0] * p.tilt.sin().abs(), half[0] * s + half[2] * c];
    (std::array::from_fn(|i| p.centre[i] - reach[i]), std::array::from_fn(|i| p.centre[i] + reach[i]))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use intracraft_library::definition::{ParamType, Value};
    use intracraft_library::layout::{Instance, Layout, Vec3};

    use super::*;

    fn library() -> Library {
        Library::built_in().unwrap()
    }

    fn one(definition: &str, params: &[(&str, Value)]) -> Layout {
        Layout { instances: vec![instance("a1", definition, params)], connections: Vec::new(), rules: Vec::new() }
    }

    fn instance(id: &str, definition: &str, params: &[(&str, Value)]) -> Instance {
        Instance {
            id: id.into(),
            definition_id: definition.into(),
            definition_version: String::new(),
            name: String::new(),
            position: Vec3::new(2.0, 0.0, -1.0),
            yaw: 0.3,
            params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
            attributes: BTreeMap::new(),
        }
    }

    fn rollers(o: &Object) -> usize {
        o.fixed.iter().chain(&o.moving).filter(|p| matches!(p.shape, Shape::Roller { .. }) && p.paint == Paint::Roller).count()
    }

    #[test]
    fn a_3_m_roller_conveyor_has_its_14_rollers_and_legs_at_most_2_m_apart() {
        let o = &scene(&one("conveyor.roller", &[]), &library())[0];
        assert_eq!(rollers(o), (3.0_f64 / 0.2063).floor() as usize);
        let mut xs: Vec<f64> = o
            .fixed
            .iter()
            .filter(|p| p.paint == Paint::Support && matches!(p.shape, Shape::Box(s) if s[1] > 0.3))
            .map(|p| (p.centre[0] - 2.0) * 0.3_f64.cos() - (p.centre[2] + 1.0) * 0.3_f64.sin())
            .collect();
        xs.sort_by(f64::total_cmp);
        xs.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        assert!(xs.len() >= 2, "{xs:?}");
        assert!(xs.windows(2).all(|w| w[1] - w[0] <= 2.0 + 1e-9), "{xs:?}");
        assert_eq!(o.mover, Mover::None);
    }

    #[test]
    fn a_turntable_turns_its_platform_about_its_pivot_and_keeps_its_base() {
        let o = &scene(&one("conveyor.roller_turntable", &[]), &library())[0];
        assert!(rollers(o) > 0 && o.moving.iter().filter(|p| p.paint == Paint::Roller).count() == rollers(o));
        assert!(o.fixed.iter().any(|p| p.paint == Paint::Base));
        let Mover::Turn { pivot } = o.mover else { panic!("{:?}", o.mover) };
        assert!((pivot[0] - 2.0).abs() < 1e-9 && (pivot[2] + 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_stretch_wrapper_turns_its_round_table_about_its_centre_with_the_rollers_inside_the_circle() {
        let o = &scene(&one("flow.stretch_wrapper", &[]), &library())[0];
        let Mover::Turn { pivot } = o.mover else { panic!("{:?}", o.mover) };
        // 0.9 m along the instance's x, turned by its yaw 0.3 (+x toward −z).
        assert!((pivot[0] - (2.0 + 0.9 * 0.3f64.cos())).abs() < 1e-9 && (pivot[2] - (-1.0 - 0.9 * 0.3f64.sin())).abs() < 1e-9);
        let turning: Vec<&Part> = o.moving.iter().filter(|p| p.paint == Paint::Roller).collect();
        assert!(turning.len() >= 7 && turning.len() == rollers(o), "every roller turns with the table: {}", turning.len());
        for p in turning {
            let Shape::Roller { length, .. } = p.shape else { continue };
            let (s, c) = p.yaw.sin_cos();
            for end in [1.0, -1.0] {
                let (x, z) = (p.centre[0] + end * length / 2.0 * s, p.centre[2] + end * length / 2.0 * c);
                assert!((x - pivot[0]).hypot(z - pivot[2]) <= 0.9 + 1e-9);
            }
        }
        assert!(o.fixed.iter().any(|p| p.paint == Paint::Guard), "the mast");
    }

    #[test]
    fn an_elevator_s_carriage_rests_at_its_entry_height_and_rises_with_it() {
        let o = &scene(&one("conveyor.elevator", &[("entryHeight", Value::Number(0.6))]), &library())[0];
        assert_eq!(o.mover, Mover::Lift { rest: 0.6 });
        let top = o.moving.iter().filter(|p| p.paint == Paint::Roller).map(|p| part_bounds(p).1[1]).fold(f64::MIN, f64::max);
        assert!((top - 0.6).abs() < 1e-9, "{top}");
    }

    #[test]
    fn a_photocell_s_beam_spans_its_host_from_the_sensor_to_the_reflector() {
        let mut layout = one("conveyor.roller", &[]);
        let host = Value::Text("a1".into());
        layout.instances.push(instance(
            "pe1",
            "sensor.photoeye",
            &[("host", host), ("sensorS", Value::Number(1.5)), ("reflectorS", Value::Number(1.5))],
        ));
        let objects = scene(&layout, &library());
        let beam = objects[1].fixed.iter().find(|p| p.paint == Paint::Beam).unwrap();
        let Shape::Box(size) = beam.shape else { panic!() };
        assert!((size[0] - 1.6).abs() < 1e-9, "{size:?}");
        assert!((beam.centre[1] - 0.66).abs() < 1e-9);
    }

    #[test]
    fn a_storage_aisle_has_its_uprights_and_beams_and_a_crane_in_three_moving_parts() {
        let objects = scene(&one("storage.aisle", &[("bays", Value::Number(4.0)), ("levels", Value::Number(3.0))]), &library());
        let ids: Vec<&str> = objects.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(ids, ["a1", "a1#mast", "a1#carriage", "a1#forks"]);
        assert!(objects.iter().all(|o| o.instance() == "a1"));
        let rack = &objects[0];
        // 5 bay lines × 2 upright lines × 2 rows; 3 levels × 2 beams × 2 rows; nothing of it moves.
        assert_eq!(rack.fixed.iter().filter(|p| p.paint == Paint::Frame).count(), 20);
        assert_eq!(rack.fixed.iter().filter(|p| p.paint == Paint::Guard).count(), 12);
        assert!(rack.moving.is_empty() && rack.mover == Mover::None);
        // 0.6 m + 3 × 1.8 m of rack, and the crane's guide rail above it.
        let top = rack.bounds().unwrap().1[1];
        assert!((6.0..7.0).contains(&top), "{top}");
        for part in &objects[1..] {
            assert!(part.fixed.is_empty() && !part.moving.is_empty() && part.mover == Mover::Shift, "{}", part.id);
        }
        // The forks' tops are just under the pallet at the first station height.
        let forks = objects[3].bounds().unwrap().1[1];
        assert!((forks - 0.595).abs() < 1e-9, "{forks}");
        // Double deep: a third line of uprights and twice the beams.
        let double = [("bays", Value::Number(4.0)), ("levels", Value::Number(3.0)), ("deep", Value::Number(2.0))];
        let rack = &scene(&one("storage.aisle", &double), &library())[0];
        assert_eq!(rack.fixed.iter().filter(|p| p.paint == Paint::Frame).count(), 30);
        assert_eq!(rack.fixed.iter().filter(|p| p.paint == Paint::Guard).count(), 24);
    }

    /// Every object of the library builds at its defaults, and with every number at its least and at its greatest value.
    #[test]
    fn every_object_of_the_library_builds_at_its_defaults_and_at_its_limits() {
        let library = library();
        for d in library.list() {
            for pick in [None, Some(false), Some(true)] {
                let params: Vec<(&str, Value)> = d
                    .parameters
                    .iter()
                    .filter(|p| matches!(p.kind, ParamType::Number | ParamType::Integer))
                    .filter_map(|p| match pick {
                        None => None,
                        Some(false) => p.min.map(|m| (p.key.as_str(), Value::Number(m))),
                        Some(true) => p.max.map(|m| (p.key.as_str(), Value::Number(m))),
                    })
                    .collect();
                let objects = scene(&one(&d.id, &params), &library);
                for part in objects.iter().flat_map(|o| o.fixed.iter().chain(&o.moving)) {
                    let (lo, hi) = part_bounds(part);
                    assert!(lo.iter().chain(&hi).all(|v| v.is_finite()), "{} {pick:?}: {part:?}", d.id);
                }
                let drawn = d.id.starts_with("conveyor.") || d.id.starts_with("flow.") || d.id.starts_with("storage.");
                if drawn && pick.is_none() {
                    assert!(objects.first().is_some_and(|o| !o.fixed.is_empty()), "{} draws nothing", d.id);
                }
            }
        }
    }
}
