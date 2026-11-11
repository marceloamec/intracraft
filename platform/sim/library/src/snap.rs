//! Port snapping and the connections the geometry implies. Two ports are connected when they face each other
//! `PORT_GAP` apart; there is no separate "connect" step, so placing objects is enough (ADR-0005, ADR-0006).

use crate::Library;
use crate::collate;
use crate::definition::{Definition, Direction};
use crate::layout::{Connection, Layout, PortRef, Vec3};
use crate::params::{Values, instance_values};
use crate::ports::{WorldPort, place_port, resolve_ports, world_ports};
use crate::rules::{Issue, Severity};

/// Gap between two connected ports, m.
pub const PORT_GAP: f64 = 0.02;
/// How far a dragged port reaches for a free port, m.
pub const SNAP_RADIUS: f64 = 0.3;
/// Placement grid when no port is within reach, m (A09).
pub const GRID_STEP: f64 = 0.1;
/// Allowed mismatch when deciding that two ports touch, m.
pub const CONNECT_TOLERANCE: f64 = 0.01;
/// Allowed difference of support heights between connected ports, m (A04).
pub const HEIGHT_TOLERANCE: f64 = 0.01;

/// Cosine of the angle between two ports that face each other.
const OPPOSITE: f64 = -0.999;

/// A component being dropped or dragged.
pub struct Candidate<'a> {
    pub definition: &'a Definition,
    pub params: Values,
    pub position: Vec3,
    pub yaw: f64,
    /// The instance being moved, never a target of its own.
    pub instance_id: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub moving_port: String,
    pub target: PortRef,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snap {
    pub position: Vec3,
    pub yaw: f64,
    pub link: Option<Link>,
    pub issues: Vec<Issue>,
}

struct SnapOption {
    distance: f64,
    moving: WorldPort,
    target: WorldPort,
    dx: f64,
    dz: f64,
}

/// Where a component dropped at `candidate` ends up: lined up with the nearest free port facing one of its ports
/// (within `radius`), else on the 100 mm grid. The yaw is kept; the user turns it.
pub fn snap_pose(candidate: &Candidate, layout: &Layout, library: &Library, radius: f64) -> Result<Snap, String> {
    snap_where(candidate, layout, library, radius, false)
}

/// As [`snap_pose`], but only an out port lines up with an in port (or an inout with either): what a drag in the
/// editor joins, so a conveyor never snaps backwards onto another.
pub fn snap_fitting(candidate: &Candidate, layout: &Layout, library: &Library, radius: f64) -> Result<Snap, String> {
    snap_where(candidate, layout, library, radius, true)
}

fn snap_where(candidate: &Candidate, layout: &Layout, library: &Library, radius: f64, fitting: bool) -> Result<Snap, String> {
    let values = instance_values(candidate.definition, &candidate.params);
    let moving: Vec<WorldPort> = resolve_ports(candidate.definition, &values)?
        .iter()
        .map(|p| place_port(p, candidate.instance_id.unwrap_or_default(), candidate.position, candidate.yaw))
        .collect();
    let targets = free_ports(layout, library, candidate.instance_id)?;
    let mut best: Option<SnapOption> = None;
    for m in &moving {
        for t in &targets {
            if !faces_each_other(m, t) || (fitting && !directions_compatible(m, t)) {
                continue;
            }
            let dx = t.position.x + t.facing.0 * PORT_GAP - m.position.x;
            let dz = t.position.z + t.facing.1 * PORT_GAP - m.position.z;
            let option = SnapOption { distance: dx.hypot(dz), moving: m.clone(), target: t.clone(), dx, dz };
            if option.distance <= radius && best.as_ref().is_none_or(|b| better(&option, b)) {
                best = Some(option);
            }
        }
    }
    let p = candidate.position;
    Ok(match best {
        None => Snap { position: Vec3::new(to_grid(p.x), p.y, to_grid(p.z)), yaw: candidate.yaw, link: None, issues: vec![] },
        Some(b) => Snap {
            position: Vec3::new(p.x + b.dx, p.y, p.z + b.dz),
            yaw: candidate.yaw,
            link: Some(Link {
                moving_port: b.moving.port_id.clone(),
                target: PortRef { instance_id: b.target.instance_id.clone(), port_id: b.target.port_id.clone() },
            }),
            issues: pair_issues(&b.moving, &b.target),
        },
    })
}

/// The connections the geometry implies: facing ports `PORT_GAP` apart, from an out port to an in port. When a port
/// has several candidates at one place seen from above (the C elevator, ADR-0013 §7), the closest height wins.
pub fn derive_connections(layout: &Layout, library: &Library) -> Result<Vec<Connection>, String> {
    let mut ports = all_ports(layout, library)?;
    ports.sort_by(|a, b| collate::compare(&a.key(), &b.key()));
    struct Pair<'a> {
        a: &'a WorldPort,
        b: &'a WorldPort,
        order: usize,
        dy: f64,
    }
    let mut candidates: Vec<Pair> = Vec::new();
    for (i, a) in ports.iter().enumerate() {
        for b in &ports[i + 1..] {
            if a.instance_id == b.instance_id || !faces_each_other(a, b) || !directions_compatible(a, b) {
                continue;
            }
            let dx = a.position.x + a.facing.0 * PORT_GAP - b.position.x;
            let dz = a.position.z + a.facing.1 * PORT_GAP - b.position.z;
            if dx.hypot(dz) > CONNECT_TOLERANCE {
                continue;
            }
            candidates.push(Pair { a, b, order: candidates.len(), dy: (a.position.y - b.position.y).abs() });
        }
    }
    candidates.sort_by(|p, q| p.dy.total_cmp(&q.dy).then(p.order.cmp(&q.order)));
    let mut used = std::collections::BTreeSet::new();
    let mut chosen: Vec<&Pair> = Vec::new();
    for c in &candidates {
        if used.contains(&c.a.key()) || used.contains(&c.b.key()) {
            continue;
        }
        used.insert(c.a.key());
        used.insert(c.b.key());
        chosen.push(c);
    }
    chosen.sort_by_key(|c| c.order);
    Ok(chosen
        .iter()
        .map(|c| {
            let (from, to) = if c.a.direction == Direction::In || c.b.direction == Direction::Out { (c.b, c.a) } else { (c.a, c.b) };
            Connection {
                from: PortRef { instance_id: from.instance_id.clone(), port_id: from.port_id.clone() },
                to: PortRef { instance_id: to.instance_id.clone(), port_id: to.port_id.clone() },
                gap: (c.a.position.x - c.b.position.x).hypot(c.a.position.z - c.b.position.z),
            }
        })
        .collect())
}

/// Warnings about the layout's connections (support heights that do not match), of one instance if given.
pub fn connection_issues(layout: &Layout, library: &Library, instance_id: Option<&str>) -> Result<Vec<Issue>, String> {
    let ports = all_ports(layout, library)?;
    let find = |r: &PortRef| ports.iter().find(|p| p.instance_id == r.instance_id && p.port_id == r.port_id);
    Ok(layout
        .connections
        .iter()
        .filter(|c| instance_id.is_none_or(|id| c.from.instance_id == id || c.to.instance_id == id))
        .flat_map(|c| match (find(&c.from), find(&c.to)) {
            (Some(a), Some(b)) => pair_issues(a, b),
            _ => vec![],
        })
        .collect())
}

fn pair_issues(a: &WorldPort, b: &WorldPort) -> Vec<Issue> {
    let mut issues = Vec::new();
    let dy = (a.position.y - b.position.y).abs();
    if dy > HEIGHT_TOLERANCE {
        issues.push(Issue {
            rule: "portHeightMismatch",
            severity: Severity::Warning,
            message: format!("Support heights of {} and {} differ by {:.0} mm", label(a), label(b), dy * 1000.0),
        });
    }
    if !directions_compatible(a, b) {
        let dir = match a.direction {
            Direction::In => "in",
            Direction::Out => "out",
            Direction::Inout => "inout",
        };
        issues.push(Issue {
            rule: "portDirectionMismatch",
            severity: Severity::Warning,
            message: format!("{} and {} are both {dir} ports", label(a), label(b)),
        });
    }
    issues
}

/// The nearer option wins. At practically the same distance (two ports at one place seen from above) a pair whose
/// directions fit wins, then the one with the closest height.
fn better(option: &SnapOption, best: &SnapOption) -> bool {
    if (option.distance - best.distance).abs() > CONNECT_TOLERANCE {
        return option.distance < best.distance;
    }
    let fits = |o: &SnapOption| directions_compatible(&o.moving, &o.target);
    if fits(option) != fits(best) {
        return fits(option);
    }
    let dy = |o: &SnapOption| (o.moving.position.y - o.target.position.y).abs();
    if dy(option) != dy(best) {
        return dy(option) < dy(best);
    }
    option.distance < best.distance
}

fn free_ports(layout: &Layout, library: &Library, moving: Option<&str>) -> Result<Vec<WorldPort>, String> {
    let mut busy = std::collections::BTreeSet::new();
    for c in &layout.connections {
        if Some(c.from.instance_id.as_str()) == moving || Some(c.to.instance_id.as_str()) == moving {
            continue;
        }
        busy.insert(format!("{}.{}", c.from.instance_id, c.from.port_id));
        busy.insert(format!("{}.{}", c.to.instance_id, c.to.port_id));
    }
    Ok(all_ports(layout, library)?.into_iter().filter(|p| Some(p.instance_id.as_str()) != moving && !busy.contains(&p.key())).collect())
}

fn all_ports(layout: &Layout, library: &Library) -> Result<Vec<WorldPort>, String> {
    let mut ports = Vec::new();
    for instance in &layout.instances {
        let definition = library
            .get(&instance.definition_id)
            .ok_or_else(|| format!("{}: unknown definition {}", instance.id, instance.definition_id))?;
        ports.extend(world_ports(instance, definition)?);
    }
    Ok(ports)
}

fn faces_each_other(a: &WorldPort, b: &WorldPort) -> bool {
    a.facing.0 * b.facing.0 + a.facing.1 * b.facing.1 < OPPOSITE
}

fn directions_compatible(a: &WorldPort, b: &WorldPort) -> bool {
    !(a.direction == b.direction && a.direction != Direction::Inout)
}

fn to_grid(value: f64) -> f64 {
    // As JavaScript's Math.round: halves go up.
    (value / GRID_STEP + 0.5).floor() * GRID_STEP
}

fn label(p: &WorldPort) -> String {
    if p.instance_id.is_empty() { p.port_id.clone() } else { p.key() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Value;
    use crate::layout::Instance;
    use std::f64::consts::{FRAC_PI_2, PI};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    fn at(x: f64, z: f64) -> Vec3 {
        Vec3::new(x, 0.0, z)
    }

    fn instance(id: &str, definition: &str, position: Vec3) -> Instance {
        Instance {
            id: id.into(),
            definition_id: definition.into(),
            definition_version: "0.1.0".into(),
            name: id.into(),
            position,
            yaw: 0.0,
            params: Default::default(),
            attributes: Default::default(),
        }
    }

    fn layout_of(instances: Vec<Instance>) -> Layout {
        Layout { instances, ..Layout::default() }
    }

    fn with_connections(mut layout: Layout, lib: &Library) -> Layout {
        layout.connections = derive_connections(&layout, lib).unwrap();
        layout
    }

    fn snap(lib: &Library, definition: &str, params: &[(&str, f64)], position: Vec3, yaw: f64, layout: &Layout) -> Snap {
        let candidate = Candidate {
            definition: lib.get(definition).unwrap(),
            params: params.iter().map(|(k, v)| (k.to_string(), Value::Number(*v))).collect(),
            position,
            yaw,
            instance_id: None,
        };
        snap_pose(&candidate, layout, lib, SNAP_RADIUS).unwrap()
    }

    fn link(instance: &str, moving: &str, port: &str) -> Option<Link> {
        Some(Link { moving_port: moving.into(), target: PortRef { instance_id: instance.into(), port_id: port.into() } })
    }

    #[test]
    fn puts_the_new_conveyors_a_port_20_mm_in_front_of_the_existing_b_port() {
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("roller1", "conveyor.roller", at(0.0, 0.0))]);
        let s = snap(&lib, "conveyor.roller", &[], at(3.15, 0.1), 0.0, &layout);
        assert_eq!(s.link, link("roller1", "A", "B"));
        assert!(close(s.position.x, 3.0 + PORT_GAP) && close(s.position.z, 0.0) && s.issues.is_empty());
    }

    #[test]
    fn also_snaps_with_the_b_end_of_the_moving_conveyor() {
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("roller1", "conveyor.roller", at(5.0, 0.0))]);
        let s = snap(&lib, "conveyor.roller", &[], at(1.9, 0.0), 0.0, &layout);
        assert_eq!(s.link, link("roller1", "B", "A"));
        assert!(close(s.position.x, 5.0 - PORT_GAP - 3.0));
    }

    #[test]
    fn does_not_snap_ports_that_do_not_face_each_other_and_falls_back_to_the_grid() {
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("roller1", "conveyor.roller", at(0.0, 0.0))]);
        assert_eq!(snap(&lib, "conveyor.roller", &[], at(3.1, 0.0), FRAC_PI_2, &layout).link, None);
        let s = snap(&lib, "conveyor.roller", &[], at(3.6, 1.234), 0.0, &layout);
        assert!(s.link.is_none() && close(s.position.x, 3.6) && (s.position.z - 1.2).abs() < 1e-12);
    }

    #[test]
    fn ignores_ports_already_connected() {
        let lib = Library::built_in().unwrap();
        let layout = with_connections(
            layout_of(vec![instance("roller1", "conveyor.roller", at(0.0, 0.0)), instance("roller2", "conveyor.roller", at(3.02, 0.0))]),
            &lib,
        );
        let s = snap(&lib, "conveyor.roller", &[], at(3.1, 0.0), 0.0, &layout);
        assert_ne!(s.link.map(|l| l.target.instance_id), Some("roller1".to_string()));
    }

    #[test]
    fn picks_the_port_at_its_own_height_when_two_share_a_place_seen_from_above() {
        // The C elevator's IN (0.6 m) and OUT_NEAR (4.6 m) are both at x = 0 facing −x. A conveyor on a 4.0 m
        // mezzanine, turned 180°, brings its A port (4.6 m) next to them.
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("elevator1", "conveyor.elevator", at(0.0, 0.0))]);
        let s = snap(&lib, "conveyor.roller", &[], Vec3::new(-0.05, 4.0, 0.0), PI, &layout);
        assert_eq!(s.link, link("elevator1", "A", "OUT_NEAR"));
        assert!(s.issues.is_empty());
    }

    #[test]
    fn warns_when_support_heights_differ_by_more_than_10_mm() {
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("roller1", "conveyor.roller", at(0.0, 0.0))]);
        let s = snap(&lib, "conveyor.roller", &[("surfaceHeight", 0.65)], at(3.1, 0.0), 0.0, &layout);
        assert!(s.link.is_some());
        assert_eq!(s.issues.iter().map(|i| i.rule).collect::<Vec<_>>(), ["portHeightMismatch"]);
    }

    #[test]
    fn turns_the_chain_port_of_a_transfer_onto_a_chain_conveyor() {
        // C2 sits at (0.7, 0.64, 0.5) facing +z; a chain conveyor at yaw −90° runs along +z.
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("transfer1", "conveyor.transfer_rc", at(0.0, 0.0))]);
        let s = snap(&lib, "conveyor.chain", &[("surfaceHeight", 0.64)], at(0.75, 0.6), -FRAC_PI_2, &layout);
        assert_eq!(s.link, link("transfer1", "A", "C2"));
        assert!(close(s.position.x, 0.7) && close(s.position.z, 0.5 + PORT_GAP));
    }

    #[test]
    fn connects_two_conveyors_end_to_end_from_out_to_in() {
        let lib = Library::built_in().unwrap();
        let layout =
            layout_of(vec![instance("roller2", "conveyor.roller", at(3.02, 0.0)), instance("roller1", "conveyor.roller", at(0.0, 0.0))]);
        let c = derive_connections(&layout, &lib).unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!((c[0].from.instance_id.as_str(), c[0].from.port_id.as_str()), ("roller1", "B"));
        assert_eq!((c[0].to.instance_id.as_str(), c[0].to.port_id.as_str()), ("roller2", "A"));
        assert!(close(c[0].gap, 0.02));
    }

    #[test]
    fn does_not_connect_conveyors_too_far_apart_or_side_by_side() {
        let lib = Library::built_in().unwrap();
        let r1 = instance("roller1", "conveyor.roller", at(0.0, 0.0));
        let apart = layout_of(vec![r1.clone(), instance("roller2", "conveyor.roller", at(3.2, 0.0))]);
        let side = layout_of(vec![r1, instance("roller2", "conveyor.roller", at(0.0, 1.5))]);
        assert!(derive_connections(&apart, &lib).unwrap().is_empty());
        assert!(derive_connections(&side, &lib).unwrap().is_empty());
    }

    #[test]
    fn pairs_the_port_with_the_closest_height_when_two_candidates_share_a_place() {
        // Two conveyors end at the elevator's IN: one at floor level (0.6 m), one on a mezzanine (4.6 m).
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![
            instance("elevator1", "conveyor.elevator", at(0.0, 0.0)),
            instance("roller1", "conveyor.roller", Vec3::new(-3.02, 4.0, 0.0)),
            instance("roller2", "conveyor.roller", at(-3.02, 0.0)),
        ]);
        let c = derive_connections(&layout, &lib).unwrap();
        let names: Vec<String> = c.iter().map(|c| format!("{}.{}→{}", c.from.instance_id, c.from.port_id, c.to.port_id)).collect();
        assert_eq!(names, ["roller2.B→IN"]);
    }

    #[test]
    fn connects_a_conveyor_to_a_turntable_station_port() {
        // Turntable centre at x = 3.02 + 0.7: its P180 port is at x = 3.02, 20 mm after the conveyor's end.
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![
            instance("roller1", "conveyor.roller", at(0.0, 0.0)),
            instance("roller_turntable1", "conveyor.roller_turntable", at(3.72, 0.0)),
        ]);
        let c = derive_connections(&layout, &lib).unwrap();
        let names: Vec<String> = c.iter().map(|c| format!("{}→{}.{}", c.from.port_id, c.to.instance_id, c.to.port_id)).collect();
        assert_eq!(names, ["B→roller_turntable1.P180"]);
    }

    #[test]
    fn reports_connected_ports_at_different_heights() {
        let lib = Library::built_in().unwrap();
        let mut lower = instance("roller2", "conveyor.roller", at(3.02, 0.0));
        lower.params.insert("surfaceHeight".into(), Value::Number(0.5));
        let layout = with_connections(layout_of(vec![instance("roller1", "conveyor.roller", at(0.0, 0.0)), lower]), &lib);
        let rules = |id| connection_issues(&layout, &lib, id).unwrap().iter().map(|i| i.rule).collect::<Vec<_>>();
        assert_eq!(rules(None), ["portHeightMismatch"]);
        assert_eq!(rules(Some("roller2")).len(), 1);
        assert!(rules(Some("roller9")).is_empty());
    }

    #[test]
    fn a_fitting_snap_never_joins_two_out_ports() {
        // Turned round, the dropped roller's out port B faces roller1's out port B, 0.2 m off.
        let lib = Library::built_in().unwrap();
        let layout = with_connections(layout_of(vec![instance("roller1", "conveyor.roller", at(0.0, 0.0))]), &lib);
        let candidate = Candidate {
            definition: lib.get("conveyor.roller").unwrap(),
            params: Default::default(),
            position: at(6.2, 0.0),
            yaw: PI,
            instance_id: None,
        };
        let any = snap_pose(&candidate, &layout, &lib, SNAP_RADIUS).unwrap();
        assert_eq!(any.issues.iter().map(|i| i.rule).collect::<Vec<_>>(), ["portDirectionMismatch"]);
        let fitting = snap_fitting(&candidate, &layout, &lib, SNAP_RADIUS).unwrap();
        assert_eq!(fitting.link, None);
        // Facing the right way, it joins as the plain snap does.
        let ahead = Candidate { position: at(3.2, 0.0), yaw: 0.0, ..candidate };
        assert_eq!(snap_fitting(&ahead, &layout, &lib, SNAP_RADIUS).unwrap().link, link("roller1", "A", "B"));
    }

    #[test]
    fn an_unknown_definition_is_an_error() {
        let lib = Library::built_in().unwrap();
        let layout = layout_of(vec![instance("x1", "conveyor.flying", at(0.0, 0.0))]);
        assert!(derive_connections(&layout, &lib).unwrap_err().contains("conveyor.flying"));
    }
}
