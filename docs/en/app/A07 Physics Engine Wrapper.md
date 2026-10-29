---
title: A07 Physics Engine Wrapper
lang: en
tags: [app, physics, sim-physics]
---

🌐 **EN** · [[A07 Envoltorio del motor físico|ES]] · [[A07 Physik-Wrapper|DE]] · [[Home (EN)|Index]]

# A07 Physics Engine Wrapper (`packages/sim-physics`)

Theory: [[T09 Rigid Body Dynamics]] to [[T15 Physics Engine Research]].

## 1. Why a wrapper

All access to the physics engine goes through this package. Nothing else may import Jolt or Rapier. Benefits: the engine can be replaced after spike S1 (or later) without touching components; versions are pinned in one place; engine quirks (sleeping, slop, restitution threshold) are handled once.

## 2. What exists today

| Module          | Content                                                                                                                                                                                                                                                           | Tests         |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `backend.ts`    | `PhysicsBackend` interface: `addBody`, `removeBody`, `setKinematicVelocity`, `setSurfaceVelocity` (returns false if unsupported), `getPose`, `castRay`, `step`, `stateHash`; `BodyDesc` (static/kinematic/dynamic, shapes, mass, COM, friction, restitution, CCD) | — (interface) |
| `fixed-step.ts` | `FixedStepClock` (accumulator, max steps per frame, interpolation `alpha`), `DEFAULT_PHYSICS_DT` = 1/120 s                                                                                                                                                        | ✓             |
| `mechanics.ts`  | `rollerAngularVelocity`, `maxTractionAccel`, `turntableRequiredFriction`, `turntableRequiredTorque`, `sweptRadius`, `boxYawInertia`                                                                                                                               | ✓             |

## 3. Planned world building (phase 4)

| Library object  | Physics bodies                                                                                          |
| --------------- | ------------------------------------------------------------------------------------------------------- |
| Pallet          | Dynamic compound: 3 bottom-board boxes + load box; explicit mass and COM; CCD on                        |
| Roller conveyor | One kinematic cylinder per roller, grouped by zone; ω set every step from the zone profile              |
| Chain conveyor  | Plan B: one static box per strand + surface velocity (Jolt) · Plan A: strips of small kinematic rollers |
| Turntable       | Kinematic platform + kinematic children (rollers/chains) posed from the platform angle                  |
| Transfer table  | Rollers + kinematic chain strands moving vertically then horizontally                                   |
| Stop            | Kinematic box moved between down/up positions                                                           |
| Guide / funnel  | Static boxes                                                                                            |
| Photoeye        | Raycast (or shape-cast) every step; edge time = step midpoint                                           |

## 4. Step loop

```text
for each fixed step (dt = 1/120 s):
    if a 10 ms control boundary was crossed: run controller, apply writes
    for each component: update kinematic targets (roller ω, stop height, platform angle…)
    wake pallets touching conveyors whose speed changed        # T10 §7
    backend.step(dt)
    sensors: raycast, detect edges, timestamp
    record impacts (speed, energy), statistics
every N steps: hash state (determinism test)
publish snapshot to UI
```

## 5. Engine settings to override (from measured defaults)

| Setting                          | Default (engine)                 | Planned value    | Reason                                  |
| -------------------------------- | -------------------------------- | ---------------- | --------------------------------------- |
| Time step                        | 1/60 s (Rapier)                  | 1/120 s          | [[T10 Time Integration and Fixed Step]] |
| Penetration slop / allowed error | 0.02 m (Jolt) / 0.005 m (Rapier) | ≈ 0.001 m        | ±5 mm stop criterion                    |
| Min velocity for restitution     | 1 m/s (Jolt)                     | ≈ 0.01 m/s       | Impacts at 0.1–0.25 m/s                 |
| Friction combination             | geometric mean / average         | exact pair value | Friction table                          |

Every change of these values is recorded in `docs/*/decisions/` (AGENTS.md rule).

## Related

[[ADR-0002 Physics Engine Candidate]] · [[T12 Rollers and Chains in Physics]] · [[A02 Architecture]] · [[A14 API Reference]]
