//! IntraCraft's physics engine (A07, phases R5–R6 of A19 Rust Implementation Plan), the engine of the Physics simulation
//! module. It has no UI.
//!
//! - `mechanics`: the closed-form figures the tests compare with (ω = v / r, μ·g, ½·m·v²).
//! - `bench`: what the plant logic asks of a physics engine at a fixed step (1/120 s), `PhysicsBench`, with Rapier as the
//!   reference engine behind it (ADR-0023) and the S1 measurements every engine is held to (ADR-0026); Avian is in
//!   `intracraft-avian`. Chains carry pallets by a moving contact surface; contacts take the library's pair table (S16).
//! - `flow`: the 3D run of a layout, its decks driven by the 2D plant logic (ADR-0016): conveyors, lift and process
//!   stations, sources, sinks, photocells and the controller's rules.
//! - `frame`: where a deck's conveying surface lies in the plant.

pub mod bench;
pub mod flow;
pub mod frame;
pub mod mechanics;

pub use rapier3d;
