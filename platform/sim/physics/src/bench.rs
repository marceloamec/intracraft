//! A physics bench (spike S1, ADR-0002, T10–T12): what the plant logic asks of a physics engine, at a fixed step. SI units,
//! y up. Bodies are made of parts (boxes, rounded boxes, cylinders) with a contact material; a part can be a moving contact
//! surface (a chain strand, a station's platform). The plant logic of `flow` talks only to [`PhysicsBench`], so the engine
//! behind it can change (ADR-0026): [`RapierBench`] is the reference, kept frozen; the Avian bench lives in
//! `intracraft-avian`. The same S1 measurements ([`s1`]) run on both.

use std::collections::HashSet;

use intracraft_library::reference::ContactPair;

pub mod rapier;
pub mod s1;

pub use rapier::RapierBench;

/// The fixed physics step, s (A07 §4).
pub const DT: f64 = 1.0 / 120.0;

/// A body of a bench, as the bench numbers it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BodyId(pub u64);

/// A part (a collider) of a bench, as the bench numbers it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartId(pub u64);

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

/// A part of the S1 bench: its friction and restitution. A contact takes the lower friction and the higher restitution of
/// its two parts, as a contact pair table would give them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub friction: f64,
    pub restitution: f64,
}

/// What the plant logic and the S1 measurements ask of a physics engine.
pub trait PhysicsBench {
    /// The pair table the parts' contact materials look their friction and restitution up in (S16).
    fn use_pairs(&mut self, pairs: &[ContactPair]);

    /// A body of `parts` whose frame is at `origin`, turned `yaw` about +y and then tilted down along its x by `slope`
    /// (fall per metre). A dynamic body's mass and centre of mass come from its parts.
    fn body(&mut self, motion: Motion, origin: [f64; 3], yaw: f64, slope: f64, parts: &[Part]) -> (BodyId, Vec<PartId>);

    /// Makes `parts` a moving contact surface; its index for `set_surface_velocity`.
    fn surface(&mut self, parts: &[PartId]) -> usize;

    /// The velocity of a moving surface, m/s in the world.
    fn set_surface_velocity(&mut self, surface: usize, v: [f64; 3]);

    /// Sets a body's linear (m/s) and angular (rad/s) velocity.
    fn set_velocity(&mut self, body: BodyId, linear: [f64; 3], angular: [f64; 3]);

    /// A body's own x and z axes in the world.
    fn axes(&self, body: BodyId) -> ([f64; 3], [f64; 3]);

    /// Whether the segment from `from` along `span` meets a part of one of `bodies` (a photocell's beam).
    fn beam_blocked(&self, from: [f64; 3], span: [f64; 3], bodies: &HashSet<BodyId>) -> bool;

    /// Removes a body and its parts.
    fn remove(&mut self, body: BodyId);

    /// One fixed step of [`DT`].
    fn step(&mut self);

    /// The physics time, s.
    fn time(&self) -> f64;

    /// A body's centre, m.
    fn position(&self, body: BodyId) -> [f64; 3];

    /// A body's velocity, m/s.
    fn velocity(&self, body: BodyId) -> [f64; 3];

    /// Runs `seconds` of physics.
    fn run(&mut self, seconds: f64) {
        let n = (seconds / DT).round() as u64;
        for _ in 0..n {
            self.step();
        }
    }

    // The S1 bench's own pieces: travel along +x, on the world's axes, each part with its own friction and restitution.

    /// A level floor whose top is at y = 0.
    fn floor(&mut self, m: Material) -> PartId;

    /// A roller across the travel (its axis along z), its top at `top` m, centred at `x`, turning so its surface moves at
    /// `speed` m/s along +x: ω = v / r.
    fn roller(&mut self, x: f64, top: f64, radius: f64, length: f64, speed: f64, m: Material) -> BodyId;

    /// A fixed chain strip: a thin box from `along.0` to `along.1` along x, its top at `top`, `width` across, centred on
    /// z = `z`, whose contact surface moves at `speed` m/s along +x (chain plan B).
    fn strip(&mut self, along: (f64, f64), top: f64, z: f64, width: f64, speed: f64, m: Material) -> PartId;

    /// A fixed stop whose face, toward −x, is at `face` m, standing from the floor to `height`.
    fn stop(&mut self, face: f64, height: f64, m: Material) -> PartId;

    /// A pallet as a dynamic box of `size` (x along travel, y, z) and `mass`, its bottom at `bottom` and its front (+x)
    /// at `front`, moving at `speed` along +x.
    fn pallet(&mut self, front: f64, bottom: f64, size: [f64; 3], mass: f64, speed: f64, m: Material) -> BodyId;
}
