//! Where a placed component's ports are. Own frame: x along travel, y up, z across; an instance is placed by its
//! position and yaw (right-handed about +Y, so +x turns toward −z), as in the reference.

use crate::definition::{Coordinate, Definition, Direction, Facing, Term, Value};
use crate::layout::{Instance, Vec3};
use crate::params::{Values, derived, instance_values};

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPort {
    pub id: String,
    pub direction: Direction,
    pub facing: Facing,
    /// In the component's own frame, m.
    pub local: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldPort {
    pub instance_id: String,
    pub port_id: String,
    pub direction: Direction,
    pub position: Vec3,
    /// Outward unit direction in the floor plane (x, z).
    pub facing: (f64, f64),
}

impl WorldPort {
    pub fn key(&self) -> String {
        format!("{}.{}", self.instance_id, self.port_id)
    }
}

fn facing_vector(f: Facing) -> (f64, f64) {
    match f {
        Facing::PlusX => (1.0, 0.0),
        Facing::MinusX => (-1.0, 0.0),
        Facing::PlusZ => (0.0, 1.0),
        Facing::MinusZ => (0.0, -1.0),
    }
}

/// The definition's ports with their coordinates worked out for these values; a coordinate that names a parameter
/// without a number is an error. Derived values may place a port too (the gravity line's entry height).
pub fn resolve_ports(definition: &Definition, values: &Values) -> Result<Vec<ResolvedPort>, String> {
    let mut values = values.clone();
    values.extend(derived(definition, &values).into_iter().map(|(k, v)| (k, Value::Number(v))));
    definition
        .ports
        .iter()
        .map(|port| {
            let eval = |c: &Coordinate| evaluate(c, &values, &port.id);
            Ok(ResolvedPort {
                id: port.id.clone(),
                direction: port.direction,
                facing: port.facing,
                local: Vec3::new(eval(&port.local.x)?, eval(&port.local.y)?, eval(&port.local.z)?),
            })
        })
        .collect()
}

/// A placed instance's ports in world coordinates, its own values over the defaults.
pub fn world_ports(instance: &Instance, definition: &Definition) -> Result<Vec<WorldPort>, String> {
    Ok(resolve_ports(definition, &instance_values(definition, &instance.params))?
        .iter()
        .map(|p| place_port(p, &instance.id, instance.position, instance.yaw))
        .collect())
}

pub fn place_port(port: &ResolvedPort, instance_id: &str, position: Vec3, yaw: f64) -> WorldPort {
    let (ox, oz) = rotate_y((port.local.x, port.local.z), yaw);
    WorldPort {
        instance_id: instance_id.to_string(),
        port_id: port.id.clone(),
        direction: port.direction,
        position: Vec3::new(position.x + ox, position.y + port.local.y, position.z + oz),
        facing: rotate_y(facing_vector(port.facing), yaw),
    }
}

/// Turns a floor-plane vector (x, z) by `yaw` about +Y, right-handed: +x turns toward −z.
pub fn rotate_y((x, z): (f64, f64), yaw: f64) -> (f64, f64) {
    let (s, c) = yaw.sin_cos();
    (x * c + z * s, -x * s + z * c)
}

fn evaluate(coordinate: &Coordinate, values: &Values, port: &str) -> Result<f64, String> {
    let param =
        |key: &str| values.get(key).and_then(Value::number).ok_or_else(|| format!("port {port}: parameter {key:?} is not a number"));
    match coordinate {
        Coordinate::Number(n) => Ok(*n),
        Coordinate::Param(key) => param(key),
        Coordinate::Terms(terms) => terms.iter().try_fold(0.0, |sum, term| {
            Ok(sum
                + match term {
                    Term::Number(n) => *n,
                    Term::Param { param: key, factor } => factor * param(key)?,
                })
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Library;
    use crate::params::defaults;
    use std::f64::consts::FRAC_PI_2;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    pub(crate) fn instance(id: &str, definition: &str, position: Vec3, yaw: f64) -> Instance {
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

    #[test]
    fn places_the_roller_conveyor_ports_at_both_ends_on_the_support_plane() {
        let lib = Library::built_in().unwrap();
        let roller = lib.get("conveyor.roller").unwrap();
        let ports = resolve_ports(roller, &defaults(roller)).unwrap();
        assert_eq!((ports[0].id.as_str(), ports[0].facing, ports[0].local), ("A", Facing::MinusX, Vec3::new(0.0, 0.6, 0.0)));
        assert_eq!((ports[1].id.as_str(), ports[1].facing, ports[1].local), ("B", Facing::PlusX, Vec3::new(3.0, 0.6, 0.0)));
    }

    #[test]
    fn places_the_gravity_line_entry_at_its_derived_height() {
        let lib = Library::built_in().unwrap();
        let gravity = lib.get("conveyor.gravity").unwrap();
        let ports = resolve_ports(gravity, &defaults(gravity)).unwrap();
        assert!(close(ports[0].local.y, 0.34));
        assert_eq!((ports[1].id.as_str(), ports[1].local), ("OUT", Vec3::new(6.0, 0.1, 0.0)));
    }

    #[test]
    fn puts_the_transfer_chain_ports_at_roller_height_plus_lift_stroke_on_both_sides() {
        let lib = Library::built_in().unwrap();
        let transfer = lib.get("conveyor.transfer_rc").unwrap();
        let ports = resolve_ports(transfer, &defaults(transfer)).unwrap();
        let port = |id| ports.iter().find(|p| p.id == id).unwrap().local;
        let (c1, c2) = (port("C1"), port("C2"));
        assert!(close(c1.y, 0.64) && close(c1.x, 0.7) && close(c1.z, -0.5) && close(c2.z, 0.5));
    }

    #[test]
    fn gives_a_turntable_one_port_per_station_at_half_the_platform_length() {
        let lib = Library::built_in().unwrap();
        let turntable = lib.get("conveyor.roller_turntable").unwrap();
        let ports = resolve_ports(turntable, &defaults(turntable)).unwrap();
        assert_eq!(ports.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["P0", "P90", "P180", "P270"]);
        assert_eq!(ports[1].facing, Facing::MinusZ);
        assert!(close(ports[1].local.x, 0.0) && close(ports[1].local.z, -0.7));
    }

    #[test]
    fn a_port_on_a_parameter_without_a_number_is_an_error() {
        let lib = Library::built_in().unwrap();
        let roller = lib.get("conveyor.roller").unwrap();
        let mut values = defaults(roller);
        values.insert("length".into(), Value::Text("long".into()));
        assert!(resolve_ports(roller, &values).unwrap_err().contains("length"));
    }

    #[test]
    fn turns_plus_x_into_minus_z_for_a_quarter_turn() {
        let (x, z) = rotate_y((1.0, 0.0), FRAC_PI_2);
        assert!(close(x, 0.0) && close(z, -1.0));
    }

    #[test]
    fn moves_and_turns_the_ports_with_the_instance_and_uses_its_own_values() {
        let lib = Library::built_in().unwrap();
        let roller = lib.get("conveyor.roller").unwrap();
        let ports = world_ports(&instance("roller1", "conveyor.roller", Vec3::new(1.0, 0.0, 2.0), FRAC_PI_2), roller).unwrap();
        let b = &ports[1];
        assert!(close(ports[0].position.x, 1.0) && close(b.position.x, 1.0) && close(b.position.z, -1.0));
        assert!(close(b.position.y, 0.6) && close(b.facing.1, -1.0));
        let mut long = instance("roller1", "conveyor.roller", Vec3::default(), 0.0);
        long.params.insert("length".into(), Value::Number(5.0));
        assert!(close(world_ports(&long, roller).unwrap()[1].position.x, 5.0));
    }
}
