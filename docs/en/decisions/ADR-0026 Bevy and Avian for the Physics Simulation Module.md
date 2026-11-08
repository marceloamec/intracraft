---
title: ADR-0026 Bevy and Avian for the Physics Simulation Module
lang: en
tags: [decision, architecture, rust, physics, rendering, bevy, avian]
status: accepted
date: 2026-11-09
---

# ADR-0026 Bevy and Avian for the Physics Simulation Module

- **Status:** Accepted on 2026-11-09 by the owner. Two choices were made in this note: a separate Bevy window, and Avian
  for the physics. The Avian part is subject to the S1 gate below.
- **Date:** 2026-11-09
- **Replaces:**
  - decision 3 of [[ADR-0021 Game Engine Choice for the Rust Platform]] ("the simulation engines do not depend on Bevy");
  - decision 1 of [[ADR-0023 Rapier Passes the First Spike S1 Criteria]] ("Rapier stays the engine");
  - decision 4 of [[ADR-0021 Game Engine Choice for the Rust Platform]] and decision 1 of
    [[ADR-0022 Bevy Viewer for the Physics Simulation Module]], as far as they say how the viewer is fed.
- **Builds on:** [[ADR-0022 Bevy Viewer for the Physics Simulation Module]]. Bevy stays the viewer.

## Context

On 2026-11-08 the owner said they did not like how the 3D view of either simulation module is drawn. Both modules paint
the pallets as flat boxes over BIMCraft's 3D picture, after it and without its depth
([[ADR-0025 Simulation Modules on BIMCraft's Ribbon and Detailed Families]], consequences). So:

- a pallet shows in front of a mast that should hide it;
- rollers do not turn;
- a pallet never tilts.

The Bevy viewer accepted in ADR-0022 was never built.

The owner asked for the physics simulation to be made with a game engine. The candidates were Bevy, Fyrox and Godot;
development stays in Rust.

## Evaluation (versions checked 2026-11-08)

| Engine           | Version on 2026-11-08         | For IntraCraft                                                                                                                                                                                                                                 |
| ---------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Bevy**         | 0.20.1 (0.20.0 on 2026-10-08) | Pure Rust. Uses the **same wgpu (30.0.1)** and the same egui as BIMCraft's window (checked in the spike R5b lock file). Runs headless. Its ECS suits many similar bodies. Measured on the owner's laptop: 498 frames/s with shadows (ADR-0022) |
| **Fyrox**        | 1.0.1 (2026-03-28)            | Stable API and a full editor, but its own UI toolkit and renderer. Its editor would duplicate the BIM editor. Small community                                                                                                                  |
| **Godot + Rust** | Godot 4.7.x; godot-rust gdext | Mature renderer and Jolt built in, but a C++ core reached through FFI (`unsafe`, two runtimes). Its editor and UI would compete with the BIM editor                                                                                            |

The scores of [[ADR-0021 Game Engine Choice for the Rust Platform]] still hold: Bevy 35, Fyrox 31, Godot 28 of 45.

**Bevy is the engine.**

### Avian, Bevy's physics

| Item                    | Finding                                                                                                                                 |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Version                 | `avian3d` 0.8.0, 2026-10-20, requires Bevy 0.20 (`bevy ^0.20`, `parry3d ^0.31`)                                                         |
| Moving surface (chains) | `ContactManifold::tangent_velocity`, set in `CollisionHooks::modify_contacts`                                                           |
| Determinism             | Feature `enhanced-determinism` (libm math). Avian lists cross-platform determinism as future work: same result on the same machine only |
| Precision               | Feature `f64` (positions and solver in double precision), with `parry-f64`                                                              |
| Licence                 | MIT or Apache-2.0                                                                                                                       |

## Decision

1. **The 3D run is drawn in a separate Bevy window** (`platform/apps/intracraft-viewer`), opened by a "View in 3D"
   button of the Discrete simulation and Physics simulation tabs.
   - The main window saves the current model to a temporary file and starts the viewer with the module.
   - The viewer runs its own simulation from the file with the same seed; the runs are deterministic, so it shows the
     same run as the main window.
   - The static scene (conveyors with rollers, frames, legs) comes from BIMCraft's own triangles
     (`bimcraft_core::view3d::gpu::Mesh`), converted from mm with z up to m with y up.
   - The flat pallet boxes painted over BIMCraft's 3D view are removed; the floor plan keeps its state colours and
     pallets.
2. **The physics of the Physics simulation module moves from Rapier to Avian 0.8.**
   - The plant logic of `intracraft_physics::flow` ([[ADR-0024 Plant Logic on the 3D Decks in Rust]]) is not
     rewritten. It talks to the physics through a small common contract (`PhysicsBench`: bodies, surfaces, speeds,
     poses, a beam test, a step), with one Rapier version and one Avian version (crate `intracraft-avian`).
   - The Avian bench is a Bevy app without a window, stepped by hand at the fixed 1/120 s, with an explicit system
     order and a single-threaded schedule.
   - The same tests run on both benches. **Rapier's results become the expected values for Avian**, as the TypeScript
     reference's are for Rust.
   - Rapier is kept, frozen, as the reference until Avian matches it; removing it needs its own ADR.
3. **S1 gate.** Avian replaces Rapier in the app only if it passes, on the same bench and with Avian's default solver
   settings:
   - P1: steady speed on rollers;
   - P3: stop within ±5 mm, rebound under 10 mm;
   - P7: the same run twice gives identical results;
   - P8: chains carry the pallet by their moving surface.

   Then it must pass the plant tests of `flow` (T1 and T2 in 3D, hand-over, sensors within ±50 ms of 2D, stations).
   If a criterion fails, work stops and the figures go to the owner. Masses, frictions and steps are not changed to
   pass a test without recording it here.
4. **Versions are pinned:** `bevy` 0.20.1, `avian3d` 0.8.0, `bevy_egui` 0.43.0, vendored into `platform/vendor/`.
   Upgrades follow ADR-0022 decision 2: at most twice a year, in their own change, and Bevy and Avian together.

## Consequences

- The physics engine now depends on Bevy's ECS; a Bevy upgrade carries the physics with it. The plant logic and the
  DES engine do not depend on Bevy.
- The Rapier results measured in ADR-0023 and ADR-0024 must be measured again with Avian. A difference is recorded
  here before the work goes on.
- Determinism holds on one machine. Results saved in `test_data/` that come from a 3D run name the machine they were
  made on.
- Build: about 360 more vendored crates. The first build takes about 4 more minutes.
- Clippy's lints on Bevy systems (`too_many_arguments`, `type_complexity`) are allowed in the viewer only, never in the
  engines.
- Still open, as before:
  - P2 on rubber-coated rollers (ADR-0023), measured again on Avian;
  - the catalogue's meshopt-compressed GLB models (ADR-0022 decision 3). Until they are solved, the viewer draws the
    families' parametric forms.

## Measurement fix found on the way (2026-11-09)

The S1 tests became measurements on any bench (`intracraft_physics::bench::s1`) and the test macro `s1_tests!`.

Doing so showed that the P3 test measured the rebound from the furthest point the front **ever** reached. That counted
every sample from 3.99 m on, so the last 10 mm of the **approach** read as rebound: 9.62 mm on Rapier, just under the
10 mm limit.

The rebound is now measured from the furthest point reached **so far**, as the figure in
[[ADR-0023 Rapier Passes the First Spike S1 Criteria]] already was (0.18 mm). The limit (< 10 mm) is unchanged. Every
other S1 figure on Rapier is unchanged by the move to the common bench.

## Alternatives considered

| Option                               | Why not                                                                                                                       |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| Bevy draws inside BIMCraft's window  | Possible, since both use wgpu 30.0.1, but riskier (two frame loops sharing one device). The owner preferred a separate window |
| Bevy draws, Rapier keeps the physics | Recommended for risk, since Rapier's results are already measured. The owner preferred the engine's own physics               |
| Fyrox or Godot                       | See the evaluation above                                                                                                      |

## Related

- [[ADR-0021 Game Engine Choice for the Rust Platform]] · [[ADR-0022 Bevy Viewer for the Physics Simulation Module]] ·
  [[ADR-0023 Rapier Passes the First Spike S1 Criteria]] · [[ADR-0024 Plant Logic on the 3D Decks in Rust]] ·
  [[A19 Rust Implementation Plan]] · [[T12 Rollers and Chains in Physics]]
- Sources: [Bevy 0.20](https://bevy.org/news/bevy-0-20/) · [bevy 0.20.1 on docs.rs](https://docs.rs/crate/bevy/0.20.1) ·
  [avian3d versions](https://crates.io/crates/avian3d/versions) ·
  [Avian `ContactManifold` 0.8.0](https://docs.rs/avian3d/0.8.0/avian3d/collision/contact_types/struct.ContactManifold.html) ·
  [Avian features](https://lib.rs/crates/avian3d/features) ·
  [Avian README](https://github.com/janhohenheim/avian/blob/main/README.md) ·
  [Fyrox blog](https://fyrox.rs/blog/) ·
  [Fyrox 1.0 (i-programmer)](https://www.i-programmer.info/news/144-graphics-and-games/19015-fyrox-game-engine-reaches-version-100.html) ·
  [Godot 4.6 released](https://gamefromscratch.com/godot-4-6-released/) ·
  [Godot maintenance releases](https://forum.godotengine.org/c/announcements/23?page=1)
