---
title: ADR-0002 Physics Engine Candidate
lang: en
tags: [decision, physics]
status: accepted
date: 2026-10-28
updated: 2026-10-30
---

🌐 **EN** · [[ADR-0002 Candidato a motor de física|ES]] · [[ADR-0002 Physik-Engine-Kandidat|DE]] · [[Home (EN)|Index]]

# ADR-0002 Physics Engine Candidate

- **Status:** Accepted on 2026-10-30, after spike S1: Jolt with plan B for chains
- **Date:** 2026-10-28 (proposed), 2026-10-30 (accepted)

## Context

The biggest technical risk of the MVP is the **chain conveyor** in 3D: it needs either a "moving surface" contact or many small moving bodies ([[T12 Rollers and Chains in Physics]]). The plan lists Rapier (Apache-2.0) and Jolt (MIT) and says not to assume that the moving surface exists in the JS bindings.

## Findings (see [[T15 Physics Engine Research]])

- `jolt-physics` 1.1.1: `ContactListenerJS` + `ContactSettings.mRelativeLinearSurfaceVelocity` and `mCombinedFriction` are exposed → plan B and exact friction pairs possible.
- `@dimforge/rapier3d-compat` 0.21.0: `PhysicsHooks` only filters pairs; no solver-contact modification → plan A only.
- Measured defaults that must be overridden in either engine: penetration slop / allowed error (0.02 m Jolt, 0.005 m Rapier), Jolt restitution threshold 1 m/s.

## Decision

**Jolt** (`jolt-physics`, MIT) is the physics engine, behind `PhysicsBackend`. Chain conveyors use **plan B**: a static strip per strand with a surface velocity set in the contact callback. The spike was run first with Jolt, with Rapier and plan A as the fallback; Jolt met every criterion (results below), so the fallback was not needed.

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

## Spike S1 results (2026-10-30)

Measured with the physics bench and `jolt-physics` 1.1.1 at 1/120 s, with the engine settings of [[A07 Physics Engine Wrapper]] §5. Each line is a reference scenario ([[A10 CLI and Scenarios]]), run by `pnpm test` and `pnpm run scenario all`.

| #   | Scenario       | Measured                                                           | Criterion                                     | Result         |
| --- | -------------- | ------------------------------------------------------------------ | --------------------------------------------- | -------------- |
| a   | P1             | 0.2500 m/s with 1000 kg on 10 rollers                              | 0.25 m/s ±3 %                                 | ✅             |
| b   | P8, P8-jitter  | Plan B: 0.2500 m/s; height spread below 1 µm                       | ±3 %, no visible vibration                    | ✅             |
| b   | P8-planA       | Plan A (small driven rollers under each strand): 0.2502 m/s        | ±3 %                                          | ✅             |
| c   | P3, P3-rebound | Front error 0.03 mm; rebound 0.3 mm                                | ±5 mm; < 10 mm                                | ✅             |
| d   | P9, P9-yaw     | 50 mm and 5° → 0.2 mm and 0.89°, with polyethylene guide strips    | ±10 mm and ±1°                                | ✅ (see below) |
| e   | P10, P10-fast  | Shift on the platform 0.02 mm (15 °/s), 0.06 mm (45 °/s, 120 °/s²) | < 5 mm                                        | ✅             |
| f   | P11            | 0.54 ms per step: 20 conveyors (280 rollers, stops), 10 pallets    | ≤ 4.17 ms (half of the step budget at 60 FPS) | ✅             |
| g   | P7             | Same state hash after 60 s in two runs                             | Equal                                         | ✅             |

Further checks on the same bench:

- **P2:** traction limit a = μ·g within 1.3 %.
- **P4:** impact energy of 1025 kg at 0.25 m/s, 32.03 J.
- **P5:** restitution per stop material within 5 %.
- **P6:** tipping with an off-centre load.

**Findings that need a decision** ([[Open Questions]] Q18):

1. **Guide friction decides whether a turned pallet can be straightened (S1-d).**
   - A 1.2 m pallet turned 5° is 0.89 m wide seen from above. Guides with 10 mm clearance per side leave 0.82 m.
   - The funnel pushes one corner. The pallet slides sideways instead of turning and then **wedges diagonally** between the rails.
   - With painted steel guides (wood μ = 0.3) this happens at 5°. Up to 3° the pallet still leaves square (scenario P9-steel).
   - With low-friction polyethylene strips (μ = 0.2) the 5° case is corrected (P9).
   - No mass, friction or step value was changed to reach this. The guide material is an instance choice already in [[S16 Contact Materials]].
   - A 50 mm offset without rotation is corrected by either material (6.9 mm left).
2. **Guides must stay below an overhanging load.**
   - The loaded EUR model is 828 mm wide, 15 mm more per side than the pallet.
   - A guide 150 mm high (S08 default) reaches 9 mm above the pallet top, so the load hits the end of the rail.
   - The bench uses 100 mm guides for S1-d.
3. **Positions are single precision.** Jolt stores body positions as 32-bit floats: about 0.01 mm at 10 m. That is far below every criterion, but tests must not compare positions closer than 1e-6 m.

## Consequences

- All engine access stays behind `PhysicsBackend` ([[A07 Physics Engine Wrapper]]), so the fallback costs one adapter, not a rewrite.
- **Chains use plan B** (static strips with a surface velocity, one contact per strand). Plan A also meets the criterion, but needs dozens of bodies per metre, so it stays as a fallback.
- Rapier is not needed. Plans C and D are dropped.
- Next: the physics world built from a layout (phase 4 in [[A12 Roadmap]]): conveyors, sources, sinks, ray-cast photocells and stops, and the 2D/3D sensor consistency test (±50 ms).
