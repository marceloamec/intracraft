//! Snapping in the Design module (ADR-0028): a dragged intralogistics object lines its in port up with a free out port
//! that faces it (or its out port with a free in port), 20 mm apart, within 300 mm; else its entry falls on the 100 mm
//! grid (`intracraft_library::snap`, A09). Only ports whose directions fit join, so a conveyor never snaps backwards.
//! Also where an object goes "next" after another's free out port: a copy, or a new object from the library.
//!
//! All in the model's frame (mm, x east, y north); the library's snapping works in the simulation's (`crate::layout`).

use bimcraft_core::engine::Session;
use bimcraft_core::geom::P2;
use bimcraft_core::model::{Element, Id, Model};
use intracraft_library::Library;
use intracraft_library::definition::Direction;
use intracraft_library::layout::{Layout, Vec3};
use intracraft_library::params::instance_values;
use intracraft_library::ports::{resolve_ports, rotate_y, world_ports};
use intracraft_library::snap::{Candidate, PORT_GAP, SNAP_RADIUS, snap_fitting};
use serde_json::json;

use crate::attributes::definition_of;
use crate::layout::{instance_id, layout, to_model, to_sim};

/// Where a drag puts the selection: how far it moves (mm), what it joins and what is wrong with the join.
#[derive(Clone, Debug, PartialEq)]
pub struct Moved {
    pub by: P2,
    /// "Roller 2.A", the port it lined up with.
    pub link: Option<String>,
    pub notes: Vec<String>,
}

impl Moved {
    fn free(by: P2) -> Moved {
        Moved { by, link: None, notes: Vec::new() }
    }

    /// What the status bar says of it, if anything.
    pub fn said(&self) -> Option<String> {
        let link = self.link.as_ref().map(|l| format!("Connected to {l}"));
        let all: Vec<String> = link.into_iter().chain(self.notes.iter().cloned()).collect();
        (!all.is_empty()).then(|| all.join(" · "))
    }
}

/// Where an object placed after another goes: its entry (mm, plan), its angle (degrees, counter-clockwise) and its offset
/// above the level of the one before (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Next {
    pub at: P2,
    pub angle: f64,
    pub offset: f64,
}

/// How far dragging `ids` by `by` (mm) moves them: snapped when they are one intralogistics object, else as dragged.
pub fn snap_move(model: &Model, library: &Library, ids: &[Id], by: P2) -> Moved {
    let &[id] = ids else { return Moved::free(by) };
    let Some(Element::Component(c)) = model.elements.get(id) else { return Moved::free(by) };
    let Some(definition) = definition_of(model, library, id) else { return Moved::free(by) };
    let (plant, _) = layout(model, library);
    let key = instance_id(id);
    let Some(instance) = plant.instances.iter().find(|i| i.id == key) else { return Moved::free(by) };
    let to = to_sim(c.at + by, 0.0);
    let candidate = Candidate {
        definition,
        params: instance.params.clone(),
        position: Vec3::new(to.x, instance.position.y, to.z),
        yaw: instance.yaw,
        instance_id: Some(&key),
    };
    let Ok(snap) = snap_fitting(&candidate, &plant, library, SNAP_RADIUS) else { return Moved::free(by) };
    let (at, _) = to_model(snap.position);
    Moved {
        by: micron(at - c.at),
        link: snap.link.map(|l| format!("{}.{}", name_in(&plant, &l.target.instance_id), l.target.port_id)),
        notes: snap.issues.into_iter().map(|i| i.message).collect(),
    }
}

/// Where an object of `definition` goes after object `after`: its in port 20 mm ahead of `after`'s first free out port,
/// facing it, at its height. None when `after` has no free out port or `definition` no in port.
pub fn next_after(model: &Model, library: &Library, after: Id, definition: &str) -> Option<Next> {
    let Some(Element::Component(c)) = model.elements.get(after) else { return None };
    let (plant, _) = layout(model, library);
    let key = instance_id(after);
    let instance = plant.instances.iter().find(|i| i.id == key)?;
    let busy = |port: &str| plant.connections.iter().any(|k| k.from.instance_id == key && k.from.port_id == port);
    let out = world_ports(instance, library.get(&instance.definition_id)?)
        .ok()?
        .into_iter()
        .find(|p| p.direction != Direction::In && !busy(&p.port_id))?;
    let d = library.get(definition)?;
    let ports = resolve_ports(d, &instance_values(d, &Default::default())).ok()?;
    let entry = ports.iter().find(|p| p.direction != Direction::Out)?;
    // The yaw that turns the entry's facing against the out port's: a yaw turns +x toward −z.
    let heading = |(x, z): (f64, f64)| (-z).atan2(x);
    let yaw = heading((-out.facing.0, -out.facing.1)) - heading(facing(entry.facing));
    let (lx, lz) = rotate_y((entry.local.x, entry.local.z), yaw);
    let x = out.position.x + out.facing.0 * PORT_GAP - lx;
    let z = out.position.z + out.facing.1 * PORT_GAP - lz;
    let level = model.level(c.level).map_or(0.0, |l| l.elevation);
    let (at, height) = to_model(Vec3::new(x, out.position.y - entry.local.y, z));
    Some(Next { at: micron(at), angle: tidy_angle(yaw.to_degrees()), offset: (height - level).max(0.0) })
}

/// Copies object `id` to where [`next_after`] puts one of its kind: joined to its out port. The copy's id; one undo step.
pub fn duplicate_after(session: &mut Session, library: &Library, id: Id) -> Result<Id, String> {
    let model = std::sync::Arc::clone(&session.model);
    let Some(Element::Component(c)) = model.elements.get(id) else { return Err(format!("{id} is not a placed object")) };
    let d = definition_of(&model, library, id).ok_or_else(|| format!("{id} is not an object of the IntraCraft library"))?;
    let next = next_after(&model, library, id, &d.id).ok_or_else(|| format!("{id} has no free exit to put a copy after"))?;
    session.transaction("Duplicate after", |s| {
        let made = s.execute("copy", &json!({"ids": [id], "by": [next.at.0 - c.at.0, next.at.1 - c.at.1]}))?;
        let copy = made["ids"].as_array().and_then(|a| a.first()).and_then(|v| v.as_u64()).ok_or("the copy made no id")?;
        let turn = tidy_angle(next.angle - c.angle);
        if turn != 0.0 {
            s.execute("rotate", &json!({"ids": [copy], "about": [next.at.0, next.at.1], "angle": turn}))?;
        }
        Ok(copy)
    })
}

/// Places a new object of `definition` after object `after`, joined to its out port. Its id; one undo step.
pub fn place_after(session: &mut Session, library: &Library, after: Id, definition: &str) -> Result<Id, String> {
    let next = next_after(&session.model, library, after, definition)
        .ok_or_else(|| format!("{after} has no free exit to put a {definition} after"))?;
    crate::place(session, library, definition, None, [next.at.0, next.at.1], next.angle, next.offset)
}

fn name_in(plant: &Layout, id: &str) -> String {
    plant.instances.iter().find(|i| i.id == id).map_or_else(|| id.to_string(), |i| i.name.clone())
}

fn facing(f: intracraft_library::definition::Facing) -> (f64, f64) {
    use intracraft_library::definition::Facing;
    match f {
        Facing::PlusX => (1.0, 0.0),
        Facing::MinusX => (-1.0, 0.0),
        Facing::PlusZ => (0.0, 1.0),
        Facing::MinusZ => (0.0, -1.0),
    }
}

/// To the micrometre, as BIMCraft keeps points.
fn micron(p: P2) -> P2 {
    P2((p.0 * 1000.0).round() / 1000.0, (p.1 * 1000.0).round() / 1000.0)
}

/// An angle in degrees within (−180, 180], without the dust of a turn worked out in radians.
fn tidy_angle(a: f64) -> f64 {
    let a = (a * 1e6).round() / 1e6;
    let a = a.rem_euclid(360.0);
    if a > 180.0 { a - 360.0 } else { a }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intracraft_library::definition::Value;

    fn session() -> (Session, Library) {
        (Session::new(), Library::built_in().unwrap())
    }

    fn roller(s: &mut Session, lib: &Library, x: f64, y: f64, angle: f64) -> Id {
        crate::place(s, lib, "conveyor.roller", None, [x, y], angle, 0.0).unwrap()
    }

    fn connections(s: &Session, lib: &Library) -> Vec<String> {
        layout(&s.model, lib)
            .0
            .connections
            .iter()
            .map(|c| format!("{}.{}→{}.{}", c.from.instance_id, c.from.port_id, c.to.instance_id, c.to.port_id))
            .collect()
    }

    fn close(a: P2, b: P2) -> bool {
        (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
    }

    #[test]
    fn a_roller_dropped_near_anothers_exit_lines_up_20_mm_ahead_of_it() {
        // The first roller is 3000 mm long: its exit is at x = 3000. The second is dragged to 3200, 150 north.
        let (mut s, lib) = session();
        let first = roller(&mut s, &lib, 0.0, 0.0, 0.0);
        let second = roller(&mut s, &lib, 10_000.0, 0.0, 0.0);
        let m = snap_move(&s.model, &lib, &[second], P2(3200.0 - 10_000.0, 150.0));
        assert!(close(m.by, P2(3020.0 - 10_000.0, 0.0)), "{:?}", m.by);
        assert!(m.link.as_ref().is_some_and(|l| l.contains(&first.to_string()) && l.ends_with(".B")), "{:?}", m.link);
        assert!(m.notes.is_empty());
        s.execute("move", &json!({"ids": [second], "by": [m.by.0, m.by.1]})).unwrap();
        assert_eq!(connections(&s, &lib), [format!("{first}.B→{second}.A")]);
    }

    #[test]
    fn out_of_reach_it_falls_on_the_100_mm_grid() {
        let (mut s, lib) = session();
        roller(&mut s, &lib, 0.0, 0.0, 0.0);
        let second = roller(&mut s, &lib, 10_000.0, 0.0, 0.0);
        let m = snap_move(&s.model, &lib, &[second], P2(3520.0 - 10_000.0, 1234.0));
        assert!(close(m.by, P2(3500.0 - 10_000.0, 1200.0)), "{:?}", m.by);
        assert_eq!(m.link, None);
    }

    #[test]
    fn a_roller_turned_across_or_onto_a_busy_exit_does_not_snap() {
        let (mut s, lib) = session();
        roller(&mut s, &lib, 0.0, 0.0, 0.0);
        let across = roller(&mut s, &lib, 10_000.0, 0.0, 90.0);
        assert_eq!(snap_move(&s.model, &lib, &[across], P2(3020.0 - 10_000.0, 0.0)).link, None);
        // A third roller joined to the exit takes it: the next one dragged there finds it busy.
        let (mut s, lib) = session();
        let first = roller(&mut s, &lib, 0.0, 0.0, 0.0);
        roller(&mut s, &lib, 3020.0, 0.0, 0.0);
        let late = roller(&mut s, &lib, 10_000.0, 5000.0, 0.0);
        let m = snap_move(&s.model, &lib, &[late], P2(3020.0 - 10_000.0, -5000.0));
        assert_eq!(m.link, None, "{first}");
    }

    #[test]
    fn a_join_at_another_height_snaps_with_a_note() {
        let (mut s, lib) = session();
        roller(&mut s, &lib, 0.0, 0.0, 0.0);
        let second = roller(&mut s, &lib, 10_000.0, 0.0, 0.0);
        crate::set_attribute(&mut s, &lib, second, "surfaceHeight", &Value::Number(0.5)).unwrap();
        let m = snap_move(&s.model, &lib, &[second], P2(3100.0 - 10_000.0, 0.0));
        assert!(m.link.is_some());
        assert_eq!(m.notes.len(), 1);
        assert!(m.said().unwrap().contains("differ by 100 mm"));
    }

    #[test]
    fn two_objects_or_a_wall_move_as_dragged() {
        let (mut s, lib) = session();
        let a = roller(&mut s, &lib, 0.0, 0.0, 0.0);
        let b = roller(&mut s, &lib, 5000.0, 0.0, 0.0);
        assert_eq!(snap_move(&s.model, &lib, &[a, b], P2(123.0, 45.0)), Moved::free(P2(123.0, 45.0)));
        assert_eq!(snap_move(&s.model, &lib, &[999_999], P2(1.0, 2.0)).by, P2(1.0, 2.0));
    }

    #[test]
    fn a_copy_after_a_turned_roller_joins_its_exit() {
        // Turned 90° counter-clockwise, the roller runs north: its exit is at (0, 3000).
        let (mut s, lib) = session();
        let first = roller(&mut s, &lib, 0.0, 0.0, 90.0);
        let next = next_after(&s.model, &lib, first, "conveyor.roller").unwrap();
        assert!(close(next.at, P2(0.0, 3020.0)), "{next:?}");
        assert_eq!(next.angle, 90.0);
        let copy = duplicate_after(&mut s, &lib, first).unwrap();
        assert_eq!(connections(&s, &lib), [format!("{first}.B→{copy}.A")]);
        // One undo takes the copy away.
        s.execute("undo", &json!({})).unwrap();
        assert!(s.model.elements.get(copy).is_none());
    }

    #[test]
    fn a_sink_placed_after_a_roller_joins_it_and_a_busy_exit_takes_nothing_more() {
        let (mut s, lib) = session();
        let first = roller(&mut s, &lib, 0.0, 0.0, 0.0);
        let sink = place_after(&mut s, &lib, first, "flow.sink").unwrap();
        assert_eq!(connections(&s, &lib), [format!("{first}.B→{sink}.{}", in_port(&lib, "flow.sink"))]);
        assert!(next_after(&s.model, &lib, first, "conveyor.roller").is_none());
        assert!(place_after(&mut s, &lib, first, "flow.sink").is_err());
    }

    fn in_port(lib: &Library, definition: &str) -> String {
        let d = lib.get(definition).unwrap();
        resolve_ports(d, &instance_values(d, &Default::default())).unwrap().into_iter().find(|p| p.direction != Direction::Out).unwrap().id
    }
}
