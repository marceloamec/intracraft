//! The physics bench on Avian 0.8, Bevy's physics (ADR-0026): the engine of the Physics simulation module behind the
//! common [`PhysicsBench`] contract of `intracraft-physics`, held to the same S1 measurements and plant tests as the
//! Rapier reference.
//!
//! The bench is a Bevy app without a window: Avian runs in Bevy's fixed schedule, and each [`PhysicsBench::step`] feeds
//! the app exactly one fixed step of 1/120 s, on one thread, so the same layout and seed always give the same run.
//! Bodies are entities with their parts as child colliders; a part's contact material and a moving surface (a chain
//! strand, a station's platform) are components the contact hook reads. Positions are double precision (`parry-f64`).

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use avian3d::dynamics::solver::SolverConfig;
use avian3d::prelude::{
    ActiveCollisionHooks, AngularVelocity, CoefficientCombine, Collider, CollisionHooks, ContactPair, Friction, Gravity, LinearVelocity,
    Mass, Physics, PhysicsPlugins, Position, Restitution, RigidBody, Rotation, SleepingDisabled, SweptCcd,
};
use bevy::app::{App, TaskPoolOptions, TaskPoolPlugin};
use bevy::ecs::system::SystemParam;
use bevy::math::Vec3 as Vector;
use bevy::math::{DVec3, Quat, Vec3};
use bevy::prelude::{
    ChildOf, Commands, Component, Entity, Fixed, MinimalPlugins, PluginGroup, Query, Res, Resource, Time, Transform, TransformPlugin,
};
use bevy::time::TimeUpdateStrategy;
use intracraft_library::reference::ContactPair as MaterialPair;
use intracraft_physics::bench::{BodyId, DT, Material, Motion, Part, PartId, PhysicsBench, Shape};
use intracraft_physics::mechanics::{GRAVITY, roller_angular_speed};

/// The least speed of impact (m/s) at which a contact bounces as its restitution says. Avian's default, 1 m/s, is above
/// every conveyor speed, so a pallet never bounced off a stop (P5); ADR-0026 records the change.
const RESTITUTION_THRESHOLD: f32 = 0.05;

/// How long a fixed step is, to the nanosecond.
fn step_duration() -> Duration {
    Duration::from_secs_f64(DT)
}

/// What the contact hook needs to know of a part: its contact material (an index into the pair table), a friction of its
/// own (S16: an instance's own roller or chain μ), and the moving surface it belongs to.
#[derive(Component, Clone, Copy, Debug, Default)]
struct PartInfo {
    material: Option<usize>,
    friction: Option<f32>,
    surface: Option<usize>,
}

/// The moving surfaces' velocities (m/s, world) and the pair table's friction and restitution by pair of material
/// indices (lower first).
#[derive(Resource, Default)]
struct Contacts {
    surfaces: Vec<Vector>,
    pairs: HashMap<(usize, usize), (f32, f32)>,
}

#[derive(SystemParam)]
struct Hooks<'w, 's> {
    parts: Query<'w, 's, &'static PartInfo>,
    contacts: Res<'w, Contacts>,
}

impl CollisionHooks for Hooks<'_, '_> {
    fn modify_contacts(&self, contacts: &mut ContactPair, _commands: &mut Commands) -> bool {
        let info = |e: Entity| self.parts.get(e).ok().copied().unwrap_or_default();
        let (a, b) = (info(contacts.collider1), info(contacts.collider2));
        let velocity = |p: PartInfo| p.surface.and_then(|s| self.contacts.surfaces.get(s).copied());
        // Avian's `tangent_velocity` is the second surface's velocity less the first's: a surface that is the second
        // collider moves its own way; one that is the first makes the other surface move the opposite way.
        let surface = match (velocity(a), velocity(b)) {
            (_, Some(v)) => Some(v),
            (Some(v), None) => Some(-v),
            _ => None,
        };
        let pair = match (a.material, b.material) {
            (Some(x), Some(y)) => self.contacts.pairs.get(&(x.min(y), x.max(y))).copied(),
            _ => None,
        };
        let own = a.friction.or(b.friction);
        let pair = pair.map(|(friction, restitution)| (own.unwrap_or(friction), restitution));
        for manifold in &mut contacts.manifolds {
            if let Some(v) = surface {
                manifold.tangent_velocity = v;
            }
            if let Some((friction, restitution)) = pair {
                manifold.friction = friction;
                manifold.restitution = restitution;
            }
        }
        true
    }
}

pub struct AvianBench {
    app: App,
    /// Each body's parts, in the order they were made.
    parts: HashMap<Entity, Vec<Entity>>,
    materials: Vec<String>,
    steps: u64,
}

impl Default for AvianBench {
    fn default() -> Self {
        AvianBench::new()
    }
}

impl AvianBench {
    pub fn new() -> AvianBench {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins.set(TaskPoolPlugin { task_pool_options: TaskPoolOptions::with_num_threads(1) }),
            TransformPlugin,
            PhysicsPlugins::default().with_collision_hooks::<Hooks>(),
        ))
        .insert_resource(Time::<Fixed>::from_duration(step_duration()))
        .insert_resource(TimeUpdateStrategy::ManualDuration(step_duration()))
        .insert_resource(Gravity(Vector::NEG_Y * GRAVITY as f32))
        .insert_resource(SolverConfig { restitution_threshold: RESTITUTION_THRESHOLD, ..SolverConfig::default() })
        .init_resource::<Contacts>();
        app.finish();
        app.cleanup();
        AvianBench { app, parts: HashMap::new(), materials: Vec::new(), steps: 0 }
    }

    fn material(&mut self, name: &str) -> usize {
        match self.materials.iter().position(|m| m == name) {
            Some(k) => k,
            None => {
                self.materials.push(name.to_string());
                self.materials.len() - 1
            }
        }
    }

    /// A body without parts yet: its motion, its frame's place and turn.
    fn spawn_body(&mut self, motion: Motion, origin: [f64; 3], rotation: Quat) -> Entity {
        let rigid = match motion {
            Motion::Fixed => RigidBody::Static,
            Motion::Kinematic => RigidBody::Kinematic,
            Motion::Dynamic => RigidBody::Dynamic,
        };
        let at = DVec3::from_array(origin);
        let mut body = self.app.world_mut().spawn((
            rigid,
            Position(at),
            Rotation(rotation),
            Transform::from_translation(at.as_vec3()).with_rotation(rotation),
            SleepingDisabled,
        ));
        if motion == Motion::Dynamic {
            body.insert(SweptCcd::default());
        }
        let id = body.id();
        self.parts.insert(id, Vec::new());
        id
    }

    /// A part of `body`: its collider placed in the body's frame, with what the contact hook needs, a mass of its own (on
    /// a dynamic body) and its own friction and restitution (an S1 piece).
    fn spawn_part(
        &mut self,
        body: Entity,
        collider: Collider,
        frame: (Vec3, Quat),
        info: PartInfo,
        mass: Option<f32>,
        own: Option<Material>,
    ) -> Entity {
        let mut part = self.app.world_mut().spawn((
            collider,
            Transform::from_translation(frame.0).with_rotation(frame.1),
            ChildOf(body),
            ActiveCollisionHooks::MODIFY_CONTACTS,
            info,
        ));
        if let Some(m) = mass {
            part.insert(Mass(m));
        }
        if let Some(m) = own {
            part.insert((
                Friction::new(m.friction as f32).with_combine_rule(CoefficientCombine::Min),
                Restitution::new(m.restitution as f32).with_combine_rule(CoefficientCombine::Max),
            ));
        }
        let id = part.id();
        self.parts.entry(body).or_default().push(id);
        id
    }

    /// An S1 piece: a body of one part with its own friction and restitution (and a mass, if dynamic).
    fn piece(
        &mut self,
        motion: Motion,
        origin: [f64; 3],
        rotation: Quat,
        collider: Collider,
        m: Material,
        mass: Option<f32>,
    ) -> (Entity, Entity) {
        let body = self.spawn_body(motion, origin, rotation);
        let part = self.spawn_part(body, collider, (Vec3::ZERO, Quat::IDENTITY), PartInfo::default(), mass, Some(m));
        (body, part)
    }

    fn get<C: Component + Copy>(&self, e: Entity) -> Option<C> {
        self.app.world().get::<C>(e).copied()
    }
}

impl PhysicsBench for AvianBench {
    fn use_pairs(&mut self, pairs: &[MaterialPair]) {
        let mut table = HashMap::new();
        for p in pairs {
            let (a, b) = (self.material(&p.a), self.material(&p.b));
            table.insert((a.min(b), a.max(b)), (p.mu as f32, p.restitution as f32));
        }
        self.app.world_mut().resource_mut::<Contacts>().pairs.extend(table);
    }

    fn body(&mut self, motion: Motion, origin: [f64; 3], yaw: f64, slope: f64, parts: &[Part]) -> (BodyId, Vec<PartId>) {
        let rotation = Quat::from_rotation_y(yaw as f32) * Quat::from_rotation_z(-slope.atan() as f32);
        let body = self.spawn_body(motion, origin, rotation);
        let mut ids = Vec::new();
        for p in parts {
            let material = self.material(&p.material);
            let info = PartInfo { material: Some(material), friction: p.friction.map(|f| f as f32), surface: None };
            let (collider, turn) = collider_of(p.shape);
            let mass = (motion == Motion::Dynamic).then_some(p.mass as f32);
            let offset = Vec3::new(p.offset[0] as f32, p.offset[1] as f32, p.offset[2] as f32);
            ids.push(part_id(self.spawn_part(body, collider, (offset, turn), info, mass, None)));
        }
        (body_id(body), ids)
    }

    fn surface(&mut self, parts: &[PartId]) -> usize {
        let world = self.app.world_mut();
        let k = {
            let mut contacts = world.resource_mut::<Contacts>();
            contacts.surfaces.push(Vector::ZERO);
            contacts.surfaces.len() - 1
        };
        for &p in parts {
            if let Some(mut info) = entity(p.0).and_then(|e| world.get_mut::<PartInfo>(e)) {
                info.surface = Some(k);
            }
        }
        k
    }

    fn set_surface_velocity(&mut self, surface: usize, v: [f64; 3]) {
        if let Some(s) = self.app.world_mut().resource_mut::<Contacts>().surfaces.get_mut(surface) {
            *s = Vector::new(v[0] as f32, v[1] as f32, v[2] as f32);
        }
    }

    fn set_velocity(&mut self, body: BodyId, linear: [f64; 3], angular: [f64; 3]) {
        let Some(e) = entity(body.0) else { return };
        if let Ok(mut b) = self.app.world_mut().get_entity_mut(e) {
            b.insert((
                LinearVelocity(Vector::new(linear[0] as f32, linear[1] as f32, linear[2] as f32)),
                AngularVelocity(Vector::new(angular[0] as f32, angular[1] as f32, angular[2] as f32)),
            ));
        }
    }

    fn axes(&self, body: BodyId) -> ([f64; 3], [f64; 3]) {
        let r = entity(body.0).and_then(|e| self.get::<Rotation>(e)).map_or(Quat::IDENTITY, |r| r.0);
        let as_f64 = |v: Vec3| [f64::from(v.x), f64::from(v.y), f64::from(v.z)];
        (as_f64(r * Vec3::X), as_f64(r * Vec3::Z))
    }

    fn beam_blocked(&self, from: [f64; 3], span: [f64; 3], bodies: &HashSet<BodyId>) -> bool {
        let length = (span[0] * span[0] + span[1] * span[1] + span[2] * span[2]).sqrt();
        if length <= 0.0 {
            return false;
        }
        let origin = DVec3::from_array(from);
        let direction = Vector::new((span[0] / length) as f32, (span[1] / length) as f32, (span[2] / length) as f32);
        let world = self.app.world();
        // Only pallets cut a beam, and there are few of them: each part of each is tested.
        bodies.iter().filter_map(|b| entity(b.0)).filter_map(|b| self.parts.get(&b)).flatten().any(|&part| {
            let (Some(at), Some(turn), Some(collider)) =
                (world.get::<Position>(part), world.get::<Rotation>(part), world.get::<Collider>(part))
            else {
                return false;
            };
            collider.cast_ray(at.0, *turn, origin, direction, length as f32, true).is_some()
        })
    }

    fn remove(&mut self, body: BodyId) {
        let Some(e) = entity(body.0) else { return };
        self.parts.remove(&e);
        if let Ok(b) = self.app.world_mut().get_entity_mut(e) {
            b.despawn();
        }
    }

    fn step(&mut self) {
        // One update is one fixed step; the very first update of a Bevy app has no time yet, so it gets another.
        let before = self.app.world().resource::<Time<Physics>>().elapsed();
        for _ in 0..3 {
            self.app.update();
            if self.app.world().resource::<Time<Physics>>().elapsed() > before {
                break;
            }
        }
        self.steps += 1;
    }

    fn time(&self) -> f64 {
        self.steps as f64 * DT
    }

    fn position(&self, body: BodyId) -> [f64; 3] {
        entity(body.0).and_then(|e| self.get::<Position>(e)).map_or([f64::NAN; 3], |p| p.0.to_array())
    }

    fn velocity(&self, body: BodyId) -> [f64; 3] {
        entity(body.0)
            .and_then(|e| self.get::<LinearVelocity>(e))
            .map_or([f64::NAN; 3], |v| [f64::from(v.0.x), f64::from(v.0.y), f64::from(v.0.z)])
    }

    fn floor(&mut self, m: Material) -> PartId {
        let (_, part) = self.piece(Motion::Fixed, [0.0, -0.05, 0.0], Quat::IDENTITY, Collider::cuboid(200.0, 0.1, 200.0), m, None);
        part_id(part)
    }

    fn roller(&mut self, x: f64, top: f64, radius: f64, length: f64, speed: f64, m: Material) -> BodyId {
        let omega = roller_angular_speed(speed, radius);
        // Avian's cylinders stand along +y: the body lies it along +z. A point on top of a roller turning about +z moves
        // along −x: turn it the other way.
        let lay = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let (body, _) =
            self.piece(Motion::Kinematic, [x, top - radius, 0.0], lay, Collider::cylinder(radius as f32, length as f32), m, None);
        if let Ok(mut b) = self.app.world_mut().get_entity_mut(body) {
            b.insert(AngularVelocity(Vector::new(0.0, 0.0, -omega as f32)));
        }
        body_id(body)
    }

    fn strip(&mut self, along: (f64, f64), top: f64, z: f64, width: f64, speed: f64, m: Material) -> PartId {
        let (x0, x1) = along;
        let h = 0.01;
        let collider = Collider::cuboid((x1 - x0) as f32, (2.0 * h) as f32, width as f32);
        let (_, part) = self.piece(Motion::Fixed, [(x0 + x1) / 2.0, top - h, z], Quat::IDENTITY, collider, m, None);
        let id = part_id(part);
        let k = self.surface(&[id]);
        self.set_surface_velocity(k, [speed, 0.0, 0.0]);
        id
    }

    fn stop(&mut self, face: f64, height: f64, m: Material) -> PartId {
        let half = 0.05;
        let collider = Collider::cuboid((2.0 * half) as f32, height as f32, 2.0);
        let (_, part) = self.piece(Motion::Fixed, [face + half, height / 2.0, 0.0], Quat::IDENTITY, collider, m, None);
        part_id(part)
    }

    fn pallet(&mut self, front: f64, bottom: f64, size: [f64; 3], mass: f64, speed: f64, m: Material) -> BodyId {
        let [sx, sy, sz] = size;
        let collider = Collider::cuboid(sx as f32, sy as f32, sz as f32);
        let (body, _) =
            self.piece(Motion::Dynamic, [front - sx / 2.0, bottom + sy / 2.0, 0.0], Quat::IDENTITY, collider, m, Some(mass as f32));
        if let Ok(mut b) = self.app.world_mut().get_entity_mut(body) {
            b.insert(LinearVelocity(Vector::new(speed as f32, 0.0, 0.0)));
        }
        body_id(body)
    }
}

/// A part's shape as an Avian collider, and how it turns in the part's frame (a cylinder lies along z).
fn collider_of(shape: Shape) -> (Collider, Quat) {
    match shape {
        Shape::Cuboid(h) => (Collider::cuboid((2.0 * h[0]) as f32, (2.0 * h[1]) as f32, (2.0 * h[2]) as f32), Quat::IDENTITY),
        // Its outer size is the box's: the rounding is taken from inside.
        Shape::Rounded { half, radius } => {
            let inner = half.map(|h| (2.0 * (h - radius)).max(2e-4) as f32);
            (Collider::round_cuboid(inner[0], inner[1], inner[2], radius as f32), Quat::IDENTITY)
        }
        Shape::Cylinder { radius, half_length } => {
            (Collider::cylinder(radius as f32, (2.0 * half_length) as f32), Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
        }
    }
}

/// A bench id is an entity's bits, so no table is needed either way.
fn body_id(e: Entity) -> BodyId {
    BodyId(e.to_bits())
}

fn part_id(e: Entity) -> PartId {
    PartId(e.to_bits())
}

fn entity(bits: u64) -> Option<Entity> {
    Entity::try_from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    intracraft_physics::s1_tests!(AvianBench);

    #[test]
    fn a_bench_id_gives_back_its_entity() {
        let mut b = AvianBench::new();
        let id = b.pallet(1.2, 0.0, [1.2, 0.144, 0.8], 631.4, 0.0, Material { friction: 0.5, restitution: 0.0 });
        assert!(b.position(id)[0].is_finite());
        b.remove(id);
        assert!(b.position(id)[0].is_nan());
    }
}
