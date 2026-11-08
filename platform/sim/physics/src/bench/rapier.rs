//! The bench on Rapier 0.36 (ADR-0023), the reference kept frozen (ADR-0026): kinematic rollers turning at ω = v / r,
//! chain strips whose contact surface moves (the solver contacts' `tangent_velocity`), fixed stops and dynamic pallets.
//! Parts given a contact material (`body`) take the friction and restitution of the library's pair table (S16), exactly.

use std::collections::{HashMap, HashSet};

use intracraft_library::reference::ContactPair;
use rapier3d::prelude::*;

use super::{BodyId, DT, Material, Motion, Part, PartId, PhysicsBench, Shape};

/// What the contact hook needs: which colliders carry pallets with a moving surface (chain strips, a station's
/// platform) and at what velocity (m/s, world frame), and each collider's contact material with the pair table.
#[derive(Default)]
struct Hooks {
    surface_of: HashMap<ColliderHandle, usize>,
    surfaces: Vec<Vector>,
    material_of: HashMap<ColliderHandle, usize>,
    materials: Vec<String>,
    /// A friction a part sets itself, replacing the pair table's (S16: an instance's own roller or chain μ).
    friction_of: HashMap<ColliderHandle, Real>,
    /// Friction and restitution by pair of material indices (lower first).
    pairs: HashMap<(usize, usize), (Real, Real)>,
}

impl PhysicsHooks for Hooks {
    fn modify_solver_contacts(&self, context: &mut ContactModificationContext) {
        let (c1, c2) = (context.collider1, context.collider2);
        // `tangent_velocity` is the first surface's velocity relative to the second (measured on this bench: the other
        // way round carries the pallet backwards): a strip that is the first collider moves its own way; one that is the
        // second makes the other surface move the opposite way.
        let v = match (self.surface_of.get(&c1), self.surface_of.get(&c2)) {
            (Some(&s), _) => self.surfaces.get(s).copied(),
            (None, Some(&s)) => self.surfaces.get(s).map(|v| -v),
            _ => None,
        };
        let pair = match (self.material_of.get(&c1), self.material_of.get(&c2)) {
            (Some(&a), Some(&b)) => self.pairs.get(&(a.min(b), a.max(b))).copied(),
            _ => None,
        };
        let own = self.friction_of.get(&c1).or(self.friction_of.get(&c2)).copied();
        let pair = pair.map(|(friction, restitution)| (own.unwrap_or(friction), restitution));
        let Some(manifold) = context.rigid_mut() else { return };
        if let Some(v) = v {
            for contact in manifold.solver_contacts.iter_mut() {
                contact.tangent_velocity = v;
            }
        }
        if let Some((friction, restitution)) = pair {
            *manifold.friction = friction;
            *manifold.restitution = restitution;
        }
    }
}

pub struct RapierBench {
    pub world: PhysicsWorld,
    hooks: Hooks,
    steps: u64,
}

impl Default for RapierBench {
    fn default() -> Self {
        RapierBench::new()
    }
}

impl RapierBench {
    pub fn new() -> RapierBench {
        let mut world = PhysicsWorld::new();
        world.gravity = Vector::new(0.0, -crate::mechanics::GRAVITY as Real, 0.0);
        world.integration_parameters.dt = DT as Real;
        RapierBench { world, hooks: Hooks::default(), steps: 0 }
    }

    fn material(&mut self, name: &str) -> usize {
        match self.hooks.materials.iter().position(|m| m == name) {
            Some(k) => k,
            None => {
                self.hooks.materials.push(name.to_string());
                self.hooks.materials.len() - 1
            }
        }
    }

    fn collider(builder: ColliderBuilder, m: Material) -> ColliderBuilder {
        builder
            .friction(m.friction as Real)
            .friction_combine_rule(CoefficientCombineRule::Min)
            .restitution(m.restitution as Real)
            .restitution_combine_rule(CoefficientCombineRule::Max)
    }
}

impl PhysicsBench for RapierBench {
    fn use_pairs(&mut self, pairs: &[ContactPair]) {
        for p in pairs {
            let (a, b) = (self.material(&p.a), self.material(&p.b));
            self.hooks.pairs.insert((a.min(b), a.max(b)), (p.mu as Real, p.restitution as Real));
        }
    }

    fn body(&mut self, motion: Motion, origin: [f64; 3], yaw: f64, slope: f64, parts: &[Part]) -> (BodyId, Vec<PartId>) {
        let [x, y, z] = origin.map(|v| v as Real);
        let rotation = Rotation::from_rotation_y(yaw as Real) * Rotation::from_rotation_z(-slope.atan() as Real);
        let builder = match motion {
            Motion::Fixed => RigidBodyBuilder::fixed(),
            Motion::Kinematic => RigidBodyBuilder::kinematic_velocity_based(),
            Motion::Dynamic => RigidBodyBuilder::dynamic().ccd_enabled(true),
        };
        let body = self.world.bodies.insert(builder.pose(Pose::from_parts(Vector::new(x, y, z), rotation)).can_sleep(false));
        let mut colliders = Vec::new();
        for p in parts {
            let [ox, oy, oz] = p.offset.map(|v| v as Real);
            let builder = match p.shape {
                Shape::Cuboid([hx, hy, hz]) => {
                    ColliderBuilder::cuboid(hx as Real, hy as Real, hz as Real).translation(Vector::new(ox, oy, oz))
                }
                // Its outer size is the box's: the rounding is taken from inside.
                Shape::Rounded { half, radius } => {
                    let [hx, hy, hz] = half.map(|h| (h - radius).max(1e-4) as Real);
                    ColliderBuilder::round_cuboid(hx, hy, hz, radius as Real).translation(Vector::new(ox, oy, oz))
                }
                // Rapier's cylinders stand along +y: lay it along z.
                Shape::Cylinder { radius, half_length } => ColliderBuilder::cylinder(half_length as Real, radius as Real)
                    .position(Pose::from_parts(Vector::new(ox, oy, oz), Rotation::from_rotation_x(std::f32::consts::FRAC_PI_2))),
            };
            let builder = if motion == Motion::Dynamic { builder.mass(p.mass as Real) } else { builder };
            let collider = self.world.colliders.insert_with_parent(
                builder.active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS),
                body,
                &mut self.world.bodies,
            );
            let m = self.material(&p.material);
            self.hooks.material_of.insert(collider, m);
            if let Some(f) = p.friction {
                self.hooks.friction_of.insert(collider, f as Real);
            }
            colliders.push(part_id(collider));
        }
        (body_id(body), colliders)
    }

    fn surface(&mut self, parts: &[PartId]) -> usize {
        let k = self.hooks.surfaces.len();
        self.hooks.surfaces.push(Vector::ZERO);
        for &c in parts {
            self.hooks.surface_of.insert(collider_handle(c), k);
        }
        k
    }

    fn set_surface_velocity(&mut self, surface: usize, v: [f64; 3]) {
        if let Some(s) = self.hooks.surfaces.get_mut(surface) {
            *s = Vector::new(v[0] as Real, v[1] as Real, v[2] as Real);
        }
    }

    fn set_velocity(&mut self, body: BodyId, linear: [f64; 3], angular: [f64; 3]) {
        if let Some(b) = self.world.bodies.get_mut(body_handle(body)) {
            b.set_linvel(Vector::new(linear[0] as Real, linear[1] as Real, linear[2] as Real), true);
            b.set_angvel(Vector::new(angular[0] as Real, angular[1] as Real, angular[2] as Real), true);
        }
    }

    fn axes(&self, body: BodyId) -> ([f64; 3], [f64; 3]) {
        let as_f64 = |v: Vector| [f64::from(v.x), f64::from(v.y), f64::from(v.z)];
        self.world
            .bodies
            .get(body_handle(body))
            .map_or(([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]), |b| (as_f64(b.rotation() * Vector::X), as_f64(b.rotation() * Vector::Z)))
    }

    fn beam_blocked(&self, from: [f64; 3], span: [f64; 3], bodies: &HashSet<BodyId>) -> bool {
        let length = (span[0] * span[0] + span[1] * span[1] + span[2] * span[2]).sqrt();
        if length <= 0.0 {
            return false;
        }
        let ray = Ray::new(
            Vector::new(from[0] as Real, from[1] as Real, from[2] as Real),
            Vector::new((span[0] / length) as Real, (span[1] / length) as Real, (span[2] / length) as Real),
        );
        let only = |_: ColliderHandle, c: &Collider| c.parent().is_some_and(|b| bodies.contains(&body_id(b)));
        let filter = QueryFilter { predicate: Some(&only), ..QueryFilter::default() };
        self.world.cast_ray(&ray, length as Real, true, filter).is_some()
    }

    fn remove(&mut self, body: BodyId) {
        let body = body_handle(body);
        if let Some(b) = self.world.bodies.get(body) {
            for c in b.colliders() {
                self.hooks.surface_of.remove(c);
                self.hooks.material_of.remove(c);
                self.hooks.friction_of.remove(c);
            }
        }
        self.world.remove_body(body);
    }

    fn step(&mut self) {
        self.world.step_with_events(&self.hooks, &());
        self.steps += 1;
    }

    fn time(&self) -> f64 {
        self.steps as f64 * DT
    }

    fn position(&self, body: BodyId) -> [f64; 3] {
        self.world.bodies.get(body_handle(body)).map_or([f64::NAN; 3], |b| {
            let t = b.translation();
            [f64::from(t.x), f64::from(t.y), f64::from(t.z)]
        })
    }

    fn velocity(&self, body: BodyId) -> [f64; 3] {
        self.world.bodies.get(body_handle(body)).map_or([f64::NAN; 3], |b| {
            let v = b.linvel();
            [f64::from(v.x), f64::from(v.y), f64::from(v.z)]
        })
    }

    fn floor(&mut self, m: Material) -> PartId {
        let c = Self::collider(ColliderBuilder::cuboid(100.0, 0.05, 100.0).translation(Vector::new(0.0, -0.05, 0.0)), m);
        part_id(self.world.colliders.insert(c))
    }

    fn roller(&mut self, x: f64, top: f64, radius: f64, length: f64, speed: f64, m: Material) -> BodyId {
        let omega = crate::mechanics::roller_angular_speed(speed, radius);
        // A point on top of a roller turning about +z moves along −x: turn it the other way.
        let body = RigidBodyBuilder::kinematic_velocity_based()
            .translation(Vector::new(x as Real, (top - radius) as Real, 0.0))
            .angvel(Vector::new(0.0, 0.0, -omega as Real))
            .can_sleep(false);
        // Rapier's cylinders stand along +y: lay it along +z.
        let shape =
            ColliderBuilder::cylinder((length / 2.0) as Real, radius as Real).rotation(Vector::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0));
        body_id(self.world.insert(body, Self::collider(shape, m)).0)
    }

    fn strip(&mut self, along: (f64, f64), top: f64, z: f64, width: f64, speed: f64, m: Material) -> PartId {
        let (x0, x1) = along;
        let h = 0.01;
        let c = Self::collider(
            ColliderBuilder::cuboid(((x1 - x0) / 2.0) as Real, h, (width / 2.0) as Real)
                .translation(Vector::new(((x0 + x1) / 2.0) as Real, (top as Real) - h, z as Real))
                .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS),
            m,
        );
        let part = part_id(self.world.colliders.insert(c));
        let k = self.surface(&[part]);
        self.set_surface_velocity(k, [speed, 0.0, 0.0]);
        part
    }

    fn stop(&mut self, face: f64, height: f64, m: Material) -> PartId {
        let half = 0.05;
        let c = Self::collider(
            ColliderBuilder::cuboid(half, (height / 2.0) as Real, 1.0).translation(Vector::new(
                (face as Real) + half,
                (height / 2.0) as Real,
                0.0,
            )),
            m,
        );
        part_id(self.world.colliders.insert(c))
    }

    fn pallet(&mut self, front: f64, bottom: f64, size: [f64; 3], mass: f64, speed: f64, m: Material) -> BodyId {
        let [sx, sy, sz] = size.map(|v| v / 2.0);
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new((front - sx) as Real, (bottom + sy) as Real, 0.0))
            .linvel(Vector::new(speed as Real, 0.0, 0.0))
            .ccd_enabled(true)
            .can_sleep(false);
        let volume = 8.0 * sx * sy * sz;
        let shape = Self::collider(ColliderBuilder::cuboid(sx as Real, sy as Real, sz as Real), m).density((mass / volume) as Real);
        body_id(self.world.insert(body, shape).0)
    }
}

/// Rapier's handles are an index and a generation: a bench id holds both, so no table is needed either way.
fn body_id(h: RigidBodyHandle) -> BodyId {
    let (index, generation) = h.into_raw_parts();
    BodyId((u64::from(index) << 32) | u64::from(generation))
}

fn body_handle(id: BodyId) -> RigidBodyHandle {
    RigidBodyHandle::from_raw_parts((id.0 >> 32) as u32, (id.0 & 0xffff_ffff) as u32)
}

fn part_id(h: ColliderHandle) -> PartId {
    let (index, generation) = h.into_raw_parts();
    PartId((u64::from(index) << 32) | u64::from(generation))
}

fn collider_handle(id: PartId) -> ColliderHandle {
    ColliderHandle::from_raw_parts((id.0 >> 32) as u32, (id.0 & 0xffff_ffff) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::s1_tests!(RapierBench);

    #[test]
    fn a_bench_id_gives_back_the_handle_it_was_made_from() {
        let h = RigidBodyHandle::from_raw_parts(7, 3);
        assert_eq!(body_handle(body_id(h)), h);
        let c = ColliderHandle::from_raw_parts(u32::MAX - 1, 9);
        assert_eq!(collider_handle(part_id(c)), c);
    }

    /// Open (ADR-0023): on rubber-coated rollers (μ = 0.6) Rapier gives about 3.7 m/s², not μ·g = 5.9 m/s²; neither the
    /// Coulomb friction model nor more solver iterations change it.
    #[test]
    #[ignore]
    fn p2_rubber_rollers_open() {
        let a = crate::bench::s1::traction::<RapierBench>("wood", "rubber");
        assert!((a - 5.886).abs() / 5.886 < 0.10, "wood on rubber: {a} m/s²");
    }
}
