---
title: ADR-0002 Physics Engine Candidate
lang: en
tags: [decision, physics]
status: proposed
date: 2026-10-28
---

🌐 **EN** · [[ADR-0002 Candidato a motor de física|ES]] · [[ADR-0002 Physik-Engine-Kandidat|DE]] · [[Home (EN)|Index]]

# ADR-0002 Physics Engine Candidate

- **Status:** Proposed — closes after spike S1
- **Date:** 2026-10-28

## Context

The biggest technical risk of the MVP is the **chain conveyor** in 3D: it needs either a "moving surface" contact or many small moving bodies ([[T12 Rollers and Chains in Physics]]). The plan lists Rapier (Apache-2.0) and Jolt (MIT) and says not to assume that the moving surface exists in the JS bindings.

## Findings (see [[T15 Physics Engine Research]])

- `jolt-physics` 1.1.1: `ContactListenerJS` + `ContactSettings.mRelativeLinearSurfaceVelocity` and `mCombinedFriction` are exposed → plan B and exact friction pairs possible.
- `@dimforge/rapier3d-compat` 0.21.0: `PhysicsHooks` only filters pairs; no solver-contact modification → plan A only.
- Measured defaults that must be overridden in either engine: penetration slop / allowed error (0.02 m Jolt, 0.005 m Rapier), Jolt restitution threshold 1 m/s.

## Decision (proposed)

Run spike S1 **first with Jolt**, using plan B for chains. Fall back to **Rapier with plan A** if Jolt fails S1 criteria (performance of JS contact callbacks, determinism or stability). Plan C (drag force) and D (own Rust module) remain last resorts.

## Spike S1 acceptance criteria

| #   | Test                                      | Criterion                                               |
| --- | ----------------------------------------- | ------------------------------------------------------- |
| a   | 1000 kg pallet on 10 kinematic rollers    | Nominal speed ±3 %; brakes without excessive slip       |
| b   | Chain (plan B, and plan A for comparison) | Pallet speed = chain speed ±3 %, no visible vibration   |
| c   | Pallet against a stop at 0.25 m/s         | Front at ±5 mm; rebound < 10 mm                         |
| d   | Guide with funnel                         | 50 mm and 5° misalignment → within ±10 mm and ±1°       |
| e   | 90° turn on a platform                    | Pallet moves < 5 mm relative to the platform            |
| f   | Performance                               | 20 conveyors and 10 pallets at 60 FPS, no instabilities |
| g   | Determinism                               | Same state hash after 60 s in two runs                  |

## Consequences

- All engine access stays behind `PhysicsBackend` ([[A07 Physics Engine Wrapper]]), so the fallback costs one adapter, not a rewrite.
- After S1 this ADR becomes *Accepted* (or *Rejected* with the alternative) and records measured results, limitations and the chain plan.
