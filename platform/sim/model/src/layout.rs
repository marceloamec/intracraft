//! The layout the two simulation modules run, read from the BIM model of the Design module: every component of an
//! intralogistics family becomes an instance, with its id (the element's six-digit id), its place converted to the
//! simulation's frame and its values in SI, and the connections are the ones the geometry implies.
//!
//! Frames: the model keeps millimetres with x east, y north, z up and angles in degrees counter-clockwise; the
//! simulation keeps metres with x east, y up, z south (right-handed) and a yaw about +y that turns +x toward north.
//! So a model point (x, y) at height h is the simulation point (x, h, −y), and the angle is the yaw.

use bimcraft_core::geom::P2;
use bimcraft_core::model::{Element, Id, Model};
use intracraft_library::Library;
use intracraft_library::layout::{Instance, Layout, Vec3};
use intracraft_library::snap::derive_connections;

use crate::attributes::{attributes, definition_of};

/// A model point (mm, plan) and height above the project's zero (mm) in the simulation's frame (m).
pub fn to_sim(at: P2, height: f64) -> Vec3 {
    Vec3::new(at.0 / 1000.0, height / 1000.0, -at.1 / 1000.0)
}

/// A simulation point as a model point in plan (mm) and its height (mm).
pub fn to_model(p: Vec3) -> (P2, f64) {
    (P2(p.x * 1000.0, -p.z * 1000.0), p.y * 1000.0)
}

/// The id an object goes by in a run: its element id.
pub fn instance_id(id: Id) -> String {
    id.to_string()
}

/// The model's intralogistics objects as a layout, ordered by id, with the connections their ports imply. Objects whose
/// values don't read are left out and named in the second list.
pub fn layout(model: &Model, library: &Library) -> (Layout, Vec<String>) {
    let mut out = Layout::default();
    let mut problems = Vec::new();
    for (&id, element) in model.elements.iter() {
        let Element::Component(c) = element else { continue };
        let Some(definition) = definition_of(model, library, id) else { continue };
        let attrs = match attributes(model, library, id) {
            Ok(a) => a,
            Err(e) => {
                problems.push(e);
                continue;
            }
        };
        let level = model.level(c.level).map_or(0.0, |l| l.elevation);
        out.instances.push(Instance {
            id: instance_id(id),
            definition_id: definition.id.clone(),
            definition_version: definition.version.clone(),
            name: format!("{} {id}", c.family),
            position: to_sim(c.at, level + c.offset),
            yaw: c.angle.to_radians(),
            params: attrs
                .into_iter()
                .filter(|a| a.source != crate::attributes::Source::Derived)
                .filter_map(|a| a.value.map(|v| (a.key, v)))
                .collect(),
            attributes: Default::default(),
        });
    }
    match derive_connections(&out, library) {
        Ok(c) => out.connections = c,
        Err(e) => problems.push(e),
    }
    (out, problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_between_the_model_and_the_simulation_frame() {
        let p = to_sim(P2(3000.0, 2000.0), 600.0);
        assert_eq!(p, Vec3::new(3.0, 0.6, -2.0));
        assert_eq!(to_model(p), (P2(3000.0, 2000.0), 600.0));
    }

    #[test]
    fn a_model_angle_turns_the_simulation_yaw_the_same_way() {
        // 90° counter-clockwise in plan takes +x to north (+y in the model), which is −z in the simulation, as a yaw of
        // +90° about +y does.
        let (x, z) = intracraft_library::ports::rotate_y((1.0, 0.0), 90f64.to_radians());
        let north = to_sim(P2(0.0, 1000.0), 0.0);
        assert!((x - north.x).abs() < 1e-12 && (z - north.z).abs() < 1e-12);
    }
}
