//! IntraCraft's physics engine (A07, phases R5–R6 of A19 Rust Implementation Plan), the engine of the Physics simulation
//! module, on Rapier (ADR-0020, ADR-0021). It has no UI: the Bevy viewer (ADR-0022) only draws its snapshots.
//!
//! - `mechanics`: the closed-form figures the tests compare with (ω = v / r, μ·g, ½·m·v²).
//! - `bench`: a world of rollers, chain strips, stops and pallets at a fixed step (1/120 s), the base of spike S1 and of
//!   the world built from a layout. Chains carry pallets by a moving contact surface (Rapier's solver contacts'
//!   `tangent_velocity`), which the JavaScript bindings lacked.

pub mod bench;
pub mod mechanics;
pub mod world;

pub use rapier3d;
