//! IntraCraft's discrete-event engine (A06, phases R3–R4 of A19 Rust Implementation Plan), the engine of the Discrete
//! simulation module. It has no UI, draws its random numbers only from the project's seeded generator, and keeps time in
//! whole microseconds, so the same layout and seed give the same run.

pub mod kinematics;
pub mod queue;
pub mod rng;
pub mod time;

pub mod flow;
