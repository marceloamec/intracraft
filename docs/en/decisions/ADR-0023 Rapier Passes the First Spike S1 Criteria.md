---
title: ADR-0023 Rapier Passes the First Spike S1 Criteria
lang: en
tags: [decision, physics, rapier, spike-s1, rust]
status: accepted
date: 2026-11-01
---

# ADR-0023 Rapier Passes the First Spike S1 Criteria

- **Status:** Accepted on 2026-11-01. Confirms decision 3 of [[ADR-0020 Rust Platform on BIMCraft with Three Modules]] (Rapier as the engine of the Physics simulation module) for the criteria measured so far; the rest of spike S1 follows in phase R5 of [[A19 Rust Implementation Plan]].
- **Date:** 2026-11-01

## Context

The TypeScript reference runs its 3D mode on Jolt (WASM), chosen by spike S1 ([[ADR-0002 Physics Engine Candidate]]) because the JavaScript binding of Rapier had no moving contact surface for chains. The Rust platform has no maintained Jolt binding, and native Rapier has that surface (`SolverContact::tangent_velocity`, set in `PhysicsHooks::modify_solver_contacts`). ADR-0020 chose Rapier subject to spike S1 being run again in Rust.

## What was measured

`platform/sim/physics` (`intracraft-physics`), Rapier 0.36.1 (`f32`, `enhanced-determinism`), vendored; Rapier's default solver settings, nothing tuned. A bench with a floor, 30 kinematic rollers Ø 89 mm at 206.3 mm pitch turning at ω = v / r, two chain strips whose contact surface moves, a fixed stop, and a EUR pallet of 631.4 kg as a dynamic box; a fixed step of 1/120 s. Friction: the lower of the two parts; restitution: the higher. Windows, Intel Core Ultra 9 275HX, release build.

| Criterion (reference scenario)                  | Rapier                         | Required       | Result   |
| ----------------------------------------------- | ------------------------------ | -------------- | -------- |
| P1 steady speed on rollers                      | 0.2509 m/s                     | 0.25 m/s ± 3 % | Pass     |
| P8 steady speed on chains (moving surface)      | 0.2500 m/s, floats 0.05 mm     | 0.25 m/s ± 3 % | Pass     |
| P3 front against a stop                         | 0.01 mm from the face          | ± 5 mm         | Pass     |
| P3-rebound                                      | 0.18 mm                        | < 10 mm        | Pass     |
| P7 determinism (same bench, same run)           | identical positions and speeds | identical      | Pass     |
| Friction μ = 0.5 stops a pallet at 1 m/s        | 0.1009 m (μ·g: 0.1019 m)       | ± 5 %          | Pass     |
| P11 step time (30 rollers, one pallet)          | 0.019 ms                       | ≤ 4.17 ms      | Pass     |
| P2 traction, wood on zinc-plated steel (μ 0.35) | 3.43 m/s² (μ·g 3.43)           | ± 10 %         | Pass     |
| P2 traction, plastic on steel (μ 0.25)          | within 10 % of 2.45 m/s²       | ± 10 %         | Pass     |
| P2 traction, wood on rubber (μ 0.6)             | about 3.7 m/s² (μ·g 5.89)      | ± 10 %         | **Open** |
| P5 restitution against steel, PU, rubber        | 0.1, 0.2, 0.35 within 10 %     | ± 10 %         | Pass     |

`cargo test -p intracraft-physics --release -- --ignored --nocapture s1_figures` prints these figures.

**Finding:** in Rapier 0.36, the solver contact's `tangent_velocity` is the **first** collider's surface velocity relative to the second (measured: the other reading carries the pallet backwards). The hook gives a strip's velocity when it is the first collider and its opposite when it is the second.

**Open criterion, P2 on rubber:** a pallet slipping on rubber-coated rollers (μ = 0.6) accelerates at about 3.7 m/s² instead of μ·g = 5.9 m/s², as if the friction were capped near 0.37; on steel (μ = 0.35) and in free sliding μ·g holds. Two changes were tried and reverted: the Coulomb friction model (one constraint per contact point) gave 3.05 m/s², and 8 solver iterations 3.66 m/s². Hypotheses to test next: the contact between a box and a cylinder (one point per roller) and the box pitching under a high friction force; the kinematic rollers' angular velocity in the friction target; a per-manifold friction limit. Until it is solved, rubber-coated rollers in the Physics simulation module drive pallets more gently than the material table says, which matters only for start-up times on such rollers. The test is kept, ignored, as `p2_rubber_rollers_open`.

## Decision

1. Rapier stays the engine of the Physics simulation module; no fallback to Jolt is needed for the criteria above.
2. The remaining S1 criteria are run on the same bench before the world is built from a layout (R6): P2 traction per material pair (S16), P4 impact energy, P5 restitution per material, P6 tip shift, P8-jitter, P9 guide funnel and yaw, P10 lift platform.
3. Physics settings keep Rapier's defaults until a criterion fails; any change is recorded here, as AGENTS.md asks.

## Related

- [[ADR-0020 Rust Platform on BIMCraft with Three Modules]] · [[ADR-0021 Game Engine Choice for the Rust Platform]] · [[ADR-0002 Physics Engine Candidate]] · [[T11 Contact, Friction and Restitution]] · [[T12 Rollers and Chains in Physics]] · [[A19 Rust Implementation Plan]]
