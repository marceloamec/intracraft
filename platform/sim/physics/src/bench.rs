//! A physics bench (spike S1, ADR-0002, T10–T12): a Rapier world at a fixed step with a floor, kinematic rollers turning
//! at ω = v / r, chain strips whose contact surface moves, fixed stops, and pallets as dynamic boxes with their mass. SI
//! units, y up, travel along +x. Each part's friction and restitution are its own; a contact takes the lower friction and
//! the higher restitution of its two parts, as a contact pair table would give them. Parts given a contact material
//! instead (`body`) take the friction and restitution of the library's pair table (S16), exactly.

use std::collections::HashMap;

use intracraft_library::reference::ContactPair;
use rapier3d::prelude::*;

/// The fixed physics step, s (A07 §4).
pub const DT: f64 = 1.0 / 120.0;

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

/// How a body moves: never, as the run drives it, or by the forces on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Fixed,
    Kinematic,
    Dynamic,
}

/// A shape in its part's frame: a box by its half sizes, the same with its edges rounded to `radius` (a chain strand,
/// whose ends wrap round their sprockets), or a cylinder lying along the part's z (a roller).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Cuboid([f64; 3]),
    Rounded { half: [f64; 3], radius: f64 },
    Cylinder { radius: f64, half_length: f64 },
}

/// A part of a body: its shape, where it sits in the body's frame, its contact material (and a friction of its own, if
/// set) and, on a dynamic body, its mass.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub offset: [f64; 3],
    pub material: String,
    pub friction: Option<f64>,
    pub mass: f64,
}

/// A part of the bench: its friction and restitution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub friction: f64,
    pub restitution: f64,
}

pub struct Bench {
    pub world: PhysicsWorld,
    hooks: Hooks,
    steps: u64,
}

impl Default for Bench {
    fn default() -> Self {
        Bench::new()
    }
}

impl Bench {
    pub fn new() -> Bench {
        let mut world = PhysicsWorld::new();
        world.gravity = Vector::new(0.0, -crate::mechanics::GRAVITY as Real, 0.0);
        world.integration_parameters.dt = DT as Real;
        Bench { world, hooks: Hooks::default(), steps: 0 }
    }

    /// The pair table the parts' contact materials look their friction and restitution up in.
    pub fn use_pairs(&mut self, pairs: &[ContactPair]) {
        for p in pairs {
            let (a, b) = (self.material(&p.a), self.material(&p.b));
            self.hooks.pairs.insert((a.min(b), a.max(b)), (p.mu as Real, p.restitution as Real));
        }
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

    /// A body of `parts` whose frame is at `origin`, turned `yaw` about +y and then tilted down along its x by `slope`
    /// (fall per metre). A dynamic body's mass and centre of mass come from its parts.
    pub fn body(
        &mut self,
        motion: Motion,
        origin: [f64; 3],
        yaw: f64,
        slope: f64,
        parts: &[Part],
    ) -> (RigidBodyHandle, Vec<ColliderHandle>) {
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
            colliders.push(collider);
        }
        (body, colliders)
    }

    /// Makes `colliders` a moving contact surface; its index for `set_surface_velocity`.
    pub fn surface(&mut self, colliders: &[ColliderHandle]) -> usize {
        let k = self.hooks.surfaces.len();
        self.hooks.surfaces.push(Vector::ZERO);
        for &c in colliders {
            self.hooks.surface_of.insert(c, k);
        }
        k
    }

    /// The velocity of a moving surface, m/s in the world.
    pub fn set_surface_velocity(&mut self, surface: usize, v: [f64; 3]) {
        if let Some(s) = self.hooks.surfaces.get_mut(surface) {
            *s = Vector::new(v[0] as Real, v[1] as Real, v[2] as Real);
        }
    }

    /// Sets a body's linear (m/s) and angular (rad/s) velocity.
    pub fn set_velocity(&mut self, body: RigidBodyHandle, linear: [f64; 3], angular: [f64; 3]) {
        if let Some(b) = self.world.bodies.get_mut(body) {
            b.set_linvel(Vector::new(linear[0] as Real, linear[1] as Real, linear[2] as Real), true);
            b.set_angvel(Vector::new(angular[0] as Real, angular[1] as Real, angular[2] as Real), true);
        }
    }

    /// A body's own x and z axes in the world.
    pub fn axes(&self, body: RigidBodyHandle) -> ([f64; 3], [f64; 3]) {
        let as_f64 = |v: Vector| [f64::from(v.x), f64::from(v.y), f64::from(v.z)];
        self.world
            .bodies
            .get(body)
            .map_or(([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]), |b| (as_f64(b.rotation() * Vector::X), as_f64(b.rotation() * Vector::Z)))
    }

    /// Whether the segment from `from` along `span` meets a part of one of `bodies` (a photocell's beam).
    pub fn beam_blocked(&self, from: [f64; 3], span: [f64; 3], bodies: &std::collections::HashSet<RigidBodyHandle>) -> bool {
        let length = (span[0] * span[0] + span[1] * span[1] + span[2] * span[2]).sqrt();
        if length <= 0.0 {
            return false;
        }
        let ray = Ray::new(
            Vector::new(from[0] as Real, from[1] as Real, from[2] as Real),
            Vector::new((span[0] / length) as Real, (span[1] / length) as Real, (span[2] / length) as Real),
        );
        let only = |_: ColliderHandle, c: &Collider| c.parent().is_some_and(|b| bodies.contains(&b));
        let filter = QueryFilter { predicate: Some(&only), ..QueryFilter::default() };
        self.world.cast_ray(&ray, length as Real, true, filter).is_some()
    }

    fn collider(builder: ColliderBuilder, m: Material) -> ColliderBuilder {
        builder
            .friction(m.friction as Real)
            .friction_combine_rule(CoefficientCombineRule::Min)
            .restitution(m.restitution as Real)
            .restitution_combine_rule(CoefficientCombineRule::Max)
    }

    /// A level floor whose top is at y = 0.
    pub fn floor(&mut self, m: Material) -> ColliderHandle {
        let c = Self::collider(ColliderBuilder::cuboid(100.0, 0.05, 100.0).translation(Vector::new(0.0, -0.05, 0.0)), m);
        self.world.colliders.insert(c)
    }

    /// A roller across the travel (its axis along z), its top at `top` m, centred at `x`, turning so its surface moves at
    /// `speed` m/s along +x: ω = v / r.
    pub fn roller(&mut self, x: f64, top: f64, radius: f64, length: f64, speed: f64, m: Material) -> RigidBodyHandle {
        let omega = crate::mechanics::roller_angular_speed(speed, radius);
        // A point on top of a roller turning about +z moves along −x: turn it the other way.
        let body = RigidBodyBuilder::kinematic_velocity_based()
            .translation(Vector::new(x as Real, (top - radius) as Real, 0.0))
            .angvel(Vector::new(0.0, 0.0, -omega as Real))
            .can_sleep(false);
        // Rapier's cylinders stand along +y: lay it along +z.
        let shape =
            ColliderBuilder::cylinder((length / 2.0) as Real, radius as Real).rotation(Vector::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0));
        self.world.insert(body, Self::collider(shape, m)).0
    }

    /// A fixed chain strip: a thin box from `along.0` to `along.1` along x, its top at `top`, `width` across, centred on
    /// z = `z`, whose contact surface moves at `speed` m/s along +x (chain plan B).
    pub fn strip(&mut self, along: (f64, f64), top: f64, z: f64, width: f64, speed: f64, m: Material) -> ColliderHandle {
        let (x0, x1) = along;
        let h = 0.01;
        let c = Self::collider(
            ColliderBuilder::cuboid(((x1 - x0) / 2.0) as Real, h, (width / 2.0) as Real)
                .translation(Vector::new(((x0 + x1) / 2.0) as Real, (top as Real) - h, z as Real))
                .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS),
            m,
        );
        let handle = self.world.colliders.insert(c);
        let k = self.surface(&[handle]);
        self.set_surface_velocity(k, [speed, 0.0, 0.0]);
        handle
    }

    /// A fixed stop whose face, toward −x, is at `face` m, standing from the floor to `height`.
    pub fn stop(&mut self, face: f64, height: f64, m: Material) -> ColliderHandle {
        let half = 0.05;
        let c = Self::collider(
            ColliderBuilder::cuboid(half, (height / 2.0) as Real, 1.0).translation(Vector::new(
                (face as Real) + half,
                (height / 2.0) as Real,
                0.0,
            )),
            m,
        );
        self.world.colliders.insert(c)
    }

    /// A pallet as a dynamic box of `size` (x along travel, y, z) and `mass`, its bottom at `bottom` and its front (+x)
    /// at `front`, moving at `speed` along +x.
    pub fn pallet(&mut self, front: f64, bottom: f64, size: [f64; 3], mass: f64, speed: f64, m: Material) -> RigidBodyHandle {
        let [sx, sy, sz] = size.map(|v| v / 2.0);
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new((front - sx) as Real, (bottom + sy) as Real, 0.0))
            .linvel(Vector::new(speed as Real, 0.0, 0.0))
            .ccd_enabled(true)
            .can_sleep(false);
        let volume = 8.0 * sx * sy * sz;
        let shape = Self::collider(ColliderBuilder::cuboid(sx as Real, sy as Real, sz as Real), m).density((mass / volume) as Real);
        self.world.insert(body, shape).0
    }

    /// A roller whose axis centre is at `centre`, laid across a conveyor turned `yaw` about +y (its axis along the
    /// conveyor's own z), at rest; `set_roller_speed` turns it.
    pub fn roller_at(&mut self, centre: [f64; 3], yaw: f64, radius: f64, length: f64, m: Material) -> RigidBodyHandle {
        let [x, y, z] = centre.map(|v| v as Real);
        let body = RigidBodyBuilder::kinematic_velocity_based().translation(Vector::new(x, y, z)).can_sleep(false);
        let rotation = Rotation::from_rotation_y(yaw as Real) * Rotation::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let shape = ColliderBuilder::cylinder((length / 2.0) as Real, radius as Real).position(Pose::from_parts(Vector::ZERO, rotation));
        self.world.insert(body, Self::collider(shape, m)).0
    }

    /// Turns a roller of a conveyor turned `yaw` so its top moves at `speed` m/s along the conveyor's travel: ω = v / r.
    pub fn set_roller_speed(&mut self, roller: RigidBodyHandle, yaw: f64, radius: f64, speed: f64) {
        let omega = crate::mechanics::roller_angular_speed(speed, radius);
        // The roller's axis, the conveyor's own +z turned by yaw; a top moving along +travel turns about −axis.
        let axis = Vector::new(yaw.sin() as Real, 0.0, yaw.cos() as Real);
        if let Some(b) = self.world.bodies.get_mut(roller) {
            b.set_angvel(axis * (-omega as Real), true);
        }
    }

    /// A fixed chain strip of a conveyor turned `yaw`: centred at `centre` (its top's middle), `length` along travel and
    /// `width` across; its index for `set_strip_speed`.
    pub fn strip_at(&mut self, centre: [f64; 3], yaw: f64, length: f64, width: f64, m: Material) -> usize {
        let h = 0.01;
        let [x, y, z] = centre.map(|v| v as Real);
        let c = Self::collider(
            ColliderBuilder::cuboid((length / 2.0) as Real, h, (width / 2.0) as Real)
                .position(Pose::from_parts(Vector::new(x, y - h, z), Rotation::from_rotation_y(yaw as Real)))
                .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS),
            m,
        );
        let handle = self.world.colliders.insert(c);
        self.surface(&[handle])
    }

    /// Sets the surface speed of a strip of a conveyor turned `yaw`, m/s along its travel.
    pub fn set_strip_speed(&mut self, strip: usize, yaw: f64, speed: f64) {
        self.set_surface_velocity(strip, [speed * yaw.cos(), 0.0, -speed * yaw.sin()]);
    }

    /// A pallet as a dynamic box: its centre, its turn about +y, its sizes (along its own x, y, z) and mass.
    pub fn pallet_at(&mut self, centre: [f64; 3], yaw: f64, size: [f64; 3], mass: f64, m: Material) -> RigidBodyHandle {
        let [x, y, z] = centre.map(|v| v as Real);
        let [sx, sy, sz] = size.map(|v| v / 2.0);
        let body = RigidBodyBuilder::dynamic()
            .pose(Pose::from_parts(Vector::new(x, y, z), Rotation::from_rotation_y(yaw as Real)))
            .ccd_enabled(true)
            .can_sleep(false);
        let shape =
            Self::collider(ColliderBuilder::cuboid(sx as Real, sy as Real, sz as Real), m).density((mass / (8.0 * sx * sy * sz)) as Real);
        self.world.insert(body, shape).0
    }

    /// A body's turn about +y, rad.
    pub fn yaw(&self, body: RigidBodyHandle) -> f64 {
        self.world.bodies.get(body).map_or(0.0, |b| {
            let forward = b.rotation() * Vector::X;
            f64::from((-forward.z).atan2(forward.x))
        })
    }

    /// Removes a body and its parts.
    pub fn remove(&mut self, body: RigidBodyHandle) {
        if let Some(b) = self.world.bodies.get(body) {
            for c in b.colliders() {
                self.hooks.surface_of.remove(c);
                self.hooks.material_of.remove(c);
                self.hooks.friction_of.remove(c);
            }
        }
        self.world.remove_body(body);
    }

    pub fn step(&mut self) {
        self.world.step_with_events(&self.hooks, &());
        self.steps += 1;
    }

    /// Runs `seconds` of physics.
    pub fn run(&mut self, seconds: f64) {
        let n = (seconds / DT).round() as u64;
        for _ in 0..n {
            self.step();
        }
    }

    pub fn time(&self) -> f64 {
        self.steps as f64 * DT
    }

    /// A body's centre, m.
    pub fn position(&self, body: RigidBodyHandle) -> [f64; 3] {
        self.world.bodies.get(body).map_or([f64::NAN; 3], |b| {
            let t = b.translation();
            [f64::from(t.x), f64::from(t.y), f64::from(t.z)]
        })
    }

    /// A body's velocity, m/s.
    pub fn velocity(&self, body: RigidBodyHandle) -> [f64; 3] {
        self.world.bodies.get(body).map_or([f64::NAN; 3], |b| {
            let v = b.linvel();
            [f64::from(v.x), f64::from(v.y), f64::from(v.z)]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::stopping_distance;

    const WOOD_ON_STEEL: Material = Material { friction: 0.35, restitution: 0.1 };
    const EUR: [f64; 3] = [1.2, 0.144, 0.8];
    /// A EUR pallet with a 600 kg load.
    const MASS: f64 = 631.4;

    #[test]
    fn a_pallet_rests_on_the_floor() {
        let mut b = Bench::new();
        b.floor(WOOD_ON_STEEL);
        let p = b.pallet(1.2, 0.0, EUR, MASS, 0.0, WOOD_ON_STEEL);
        b.run(2.0);
        let y = b.position(p)[1] - EUR[1] / 2.0;
        assert!(y.abs() < 0.002, "its bottom is {y} m off the floor");
    }

    #[test]
    fn friction_stops_a_sliding_pallet_where_mu_g_says() {
        // μ = 0.5 stops a pallet sliding at 1 m/s in 0.102 m (the reference's backend test).
        let mu = Material { friction: 0.5, restitution: 0.0 };
        let mut b = Bench::new();
        b.floor(mu);
        let p = b.pallet(1.2, 0.0, EUR, MASS, 1.0, mu);
        let x0 = b.position(p)[0];
        b.run(1.0);
        let slid = b.position(p)[0] - x0;
        let expected = stopping_distance(1.0, 0.5);
        assert!((slid - expected).abs() / expected < 0.05, "slid {slid} m, expected {expected} m");
    }

    /// Rollers Ø 89 mm at 206.3 mm pitch, top at 0.6 m, along 6 m.
    fn rollers(b: &mut Bench, speed: f64) {
        for k in 0..30 {
            b.roller(0.1 + 0.2063 * f64::from(k), 0.6, 0.0445, 1.0, speed, Material { friction: 0.35, restitution: 0.0 });
        }
    }

    #[test]
    fn p1_rollers_carry_a_pallet_at_their_surface_speed() {
        let mut b = Bench::new();
        b.floor(WOOD_ON_STEEL);
        rollers(&mut b, 0.25);
        let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
        b.run(4.0);
        let v = b.velocity(p)[0];
        assert!((v - 0.25).abs() / 0.25 < 0.03, "{v} m/s");
    }

    #[test]
    fn p8_a_chain_strip_carries_a_pallet_by_its_moving_surface() {
        let chain = Material { friction: 0.25, restitution: 0.0 };
        let mut b = Bench::new();
        b.floor(WOOD_ON_STEEL);
        for z in [-0.3, 0.3] {
            b.strip((0.0, 6.0), 0.6, z, 0.04, 0.25, chain);
        }
        let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
        b.run(4.0);
        let v = b.velocity(p)[0];
        assert!((v - 0.25).abs() / 0.25 < 0.03, "{v} m/s");
        let y = b.position(p)[1] - 0.6 - EUR[1] / 2.0;
        assert!(y.abs() < 0.001, "the pallet floats {y} m above the chains");
    }

    #[test]
    fn p3_a_pallet_stops_against_a_stop_within_5_mm_and_rebounds_under_10_mm() {
        let mut b = Bench::new();
        b.floor(WOOD_ON_STEEL);
        rollers(&mut b, 0.25);
        b.stop(4.0, 0.75, Material { friction: 0.3, restitution: 0.1 });
        let p = b.pallet(2.0, 0.6, EUR, MASS, 0.25, WOOD_ON_STEEL);
        let mut nearest = f64::NEG_INFINITY;
        let mut after_contact = Vec::new();
        for _ in 0..(12.0 / DT) as usize {
            b.step();
            let front = b.position(p)[0] + EUR[0] / 2.0;
            nearest = nearest.max(front);
            if nearest > 3.99 {
                after_contact.push(front);
            }
        }
        let front = b.position(p)[0] + EUR[0] / 2.0;
        assert!((front - 4.0).abs() <= 0.005, "front at {front} m");
        let rebound = after_contact.iter().fold(0.0f64, |m, f| m.max(nearest - f));
        assert!(rebound < 0.010, "rebound {rebound} m");
    }

    #[test]
    fn p7_the_same_bench_gives_the_same_run() {
        let run = || {
            let mut b = Bench::new();
            b.floor(WOOD_ON_STEEL);
            rollers(&mut b, 0.25);
            let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
            b.run(3.0);
            (b.position(p), b.velocity(p))
        };
        assert_eq!(run(), run());
    }

    /// The friction and restitution of a contact pair of the library (S16): both parts get the pair's values, so the
    /// contact takes them as they are.
    fn pair(a: &str, b: &str) -> Material {
        let lib = intracraft_library::Library::built_in().unwrap();
        let p = lib.reference.contact_pair(a, b).unwrap();
        Material { friction: p.mu, restitution: p.restitution }
    }

    /// The acceleration of a pallet at rest put on rollers already turning at 1 m/s: while it slips, μ·g.
    fn traction(pallet: &str, roller: &str) -> f64 {
        let m = pair(pallet, roller);
        let mut b = Bench::new();
        b.floor(m);
        for k in 0..30 {
            b.roller(0.1 + 0.2063 * f64::from(k), 0.6, 0.0445, 1.0, 1.0, m);
        }
        let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, m);
        b.run(0.05);
        let (t1, v1) = (b.time(), b.velocity(p)[0]);
        b.run(0.1);
        (b.velocity(p)[0] - v1) / (b.time() - t1)
    }

    #[test]
    fn p2_a_pallet_slipping_on_rollers_accelerates_at_mu_g_for_each_material() {
        for (pallet, roller, expected) in [("wood", "steel-zinc", 3.4335), ("plastic", "steel-zinc", 2.4525)] {
            let a = traction(pallet, roller);
            assert!((a - expected).abs() / expected < 0.10, "{pallet} on {roller}: {a} m/s², expected {expected}");
        }
    }

    /// Open (ADR-0023): on rubber-coated rollers (μ = 0.6) Rapier gives about 3.7 m/s², not μ·g = 5.9 m/s²; neither the
    /// Coulomb friction model nor more solver iterations change it.
    #[test]
    #[ignore]
    fn p2_rubber_rollers_open() {
        let a = traction("wood", "rubber");
        assert!((a - 5.886).abs() / 5.886 < 0.10, "wood on rubber: {a} m/s²");
    }

    /// The restitution against a stop: a pallet sliding without friction at 0.25 m/s hits the stop; its speed back over
    /// its speed in.
    fn restitution(stop: &str) -> f64 {
        let m = pair("wood", stop);
        let slide = Material { friction: 0.0, restitution: 0.0 };
        let mut b = Bench::new();
        b.floor(slide);
        b.stop(3.0, 0.141, m);
        let p = b.pallet(2.0, 0.0, EUR, MASS, 0.25, Material { friction: 0.0, restitution: m.restitution });
        let mut back: f64 = 0.0;
        for _ in 0..(8.0 / DT) as usize {
            b.step();
            back = back.max(-b.velocity(p)[0]);
        }
        back / 0.25
    }

    #[test]
    fn p5_a_pallet_bounces_off_a_stop_as_its_material_pair_says() {
        for (stop, expected) in [("steel-painted", 0.1), ("polyurethane", 0.2), ("rubber", 0.35)] {
            let e = restitution(stop);
            assert!((e - expected).abs() / expected < 0.10, "wood on {stop}: {e}, expected {expected}");
        }
    }

    /// The figures of spike S1 on Rapier, for the decision record: `cargo test -p intracraft-physics --release -- --ignored
    /// --nocapture s1_figures`.
    #[test]
    #[ignore]
    fn s1_figures() {
        let mut b = Bench::new();
        b.floor(WOOD_ON_STEEL);
        rollers(&mut b, 0.25);
        let p = b.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
        b.run(4.0);
        println!("P1 roller speed: {:.4} m/s", b.velocity(p)[0]);
        let mut c = Bench::new();
        c.floor(WOOD_ON_STEEL);
        for z in [-0.3, 0.3] {
            c.strip((0.0, 6.0), 0.6, z, 0.04, 0.25, Material { friction: 0.25, restitution: 0.0 });
        }
        let q = c.pallet(1.4, 0.6, EUR, MASS, 0.0, WOOD_ON_STEEL);
        c.run(4.0);
        println!("P8 chain speed: {:.4} m/s, height error {:.5} m", c.velocity(q)[0], c.position(q)[1] - 0.6 - EUR[1] / 2.0);
        let mut d = Bench::new();
        d.floor(WOOD_ON_STEEL);
        rollers(&mut d, 0.25);
        d.stop(4.0, 0.75, Material { friction: 0.3, restitution: 0.1 });
        let r = d.pallet(2.0, 0.6, EUR, MASS, 0.25, WOOD_ON_STEEL);
        let mut nearest = f64::NEG_INFINITY;
        let mut rebound = 0.0f64;
        let t = std::time::Instant::now();
        for _ in 0..(12.0 / DT) as usize {
            d.step();
            let front = d.position(r)[0] + EUR[0] / 2.0;
            nearest = nearest.max(front);
            if nearest > 3.99 {
                rebound = rebound.max(nearest - front);
            }
        }
        let per_step = t.elapsed().as_secs_f64() * 1000.0 / (12.0 / DT);
        println!(
            "P3 front error: {:.5} m, rebound {:.5} m; {:.4} ms a step with 30 rollers",
            d.position(r)[0] + EUR[0] / 2.0 - 4.0,
            rebound,
            per_step
        );
        let mu = Material { friction: 0.5, restitution: 0.0 };
        let mut e = Bench::new();
        e.floor(mu);
        let s = e.pallet(1.2, 0.0, EUR, MASS, 1.0, mu);
        let x0 = e.position(s)[0];
        e.run(1.0);
        println!("friction 0.5 at 1 m/s: slid {:.4} m (μ·g says {:.4})", e.position(s)[0] - x0, stopping_distance(1.0, 0.5));
    }
}
