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

## S1 on Avian (2026-11-09)

The setup:

- `intracraft-avian` (`AvianBench`) runs `avian3d` 0.8.0 (`parry-f64`, `enhanced-determinism`) in a Bevy 0.20.1 app
  without a window.
- Each step is one update of 1/120 s, on one thread.
- The same scenes and tests as Rapier (`s1_tests!`) run on the owner's laptop, as a release build.

| Criterion                                     | Rapier 0.36 (reference) | Avian 0.8             | Required        | Avian    |
| --------------------------------------------- | ----------------------- | --------------------- | --------------- | -------- |
| P1 steady speed on rollers                    | 0.2509 m/s              | 0.2499 m/s            | 0.25 m/s ± 3 %  | Pass     |
| P8 steady speed on chains, float              | 0.2500 m/s, 0.05 mm     | 0.2500 m/s, 0.01 mm   | ± 3 %, < 1 mm   | Pass     |
| P3 front against the stop, rebound            | 0.01 mm, 0.18 mm        | 0.06 mm, 0.06 mm      | ± 5 mm, < 10 mm | Pass     |
| P7 the same run twice                         | identical               | identical             | identical       | Pass     |
| Friction μ = 0.5 stops a pallet at 1 m/s      | 0.1009 m                | 0.1012 m (μ·g 0.1019) | ± 5 %           | Pass     |
| P2 traction, wood on steel-zinc (μ·g 3.43)    | 3.427 m/s²              | 3.436 m/s²            | ± 10 %          | Pass     |
| P2 traction, plastic on steel-zinc (μ·g 2.45) | 2.477 m/s²              | 2.453 m/s²            | ± 10 %          | Pass     |
| P2 traction, wood on rubber (μ·g 5.89)        | 3.800 m/s² (open)       | 5.889 m/s²            | ± 10 %          | **Pass** |
| P5 restitution, steel-painted / PU / rubber   | 0.102 / 0.207 / 0.365   | 0.104 / 0.208 / 0.364 | ± 10 %          | Pass     |
| P11 time a step, 30 rollers and a pallet      | 0.020 ms                | 0.064 ms              | ≤ 4.17 ms       | Pass     |

Notes:

- **One solver setting changed** (decision 4 of the owner: solver settings only, never masses or frictions).
  - Avian's `SolverConfig::restitution_threshold` is 1 m/s by default: a contact closing more slowly does not bounce.
  - At conveyor speeds (0.25 m/s), every stop then gave the same 0.041, whatever its material.
  - It is set to **0.05 m/s**. No other figure moved.
- **Avian closes P2 on rubber**, the criterion Rapier left open in ADR-0023.
- **Moving surface.** Avian's `ContactManifold::tangent_velocity` is the second surface's velocity less the first's,
  the opposite of Rapier's convention. The contact hook gives a strip's velocity when it is the second collider and its
  opposite when it is the first.

## The plant on Avian (2026-11-09)

The plant tests of `flow` (the reference's `physics-flow.test.ts`) became a macro, `flow_tests!`. Rapier and Avian run
the same tests with the same expected values.

At first, 14 of the 24 passed on Avian. Two settings of Avian's contacts (`NarrowPhaseConfig`) were the cause, and both
are now changed:

| Setting             | Avian's default | Now  | Why                                                                                                                                                                                                                                                                                                                                                                                                             |
| ------------------- | --------------- | ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `contact_tolerance` | 20 mm           | 1 mm | Contacts are kept up to this distance. The front edge of a sink's bed, 2 mm below the rollers (ADR-0024), became a contact with a slanted normal and stopped the pallet on it, as Rapier's bed at the rollers' level had done. Every run that ended in a sink failed.                                                                                                                                           |
| `recycle_distance`  | 50 mm           | 0    | Contact points are kept from the step before while the bodies have moved less than this. The contact hook is then not asked again, so a chain that started under a pallet kept its speed of the third step (12.5 mm/s) until the pallet had crept 50 mm, about 3.5 s. T1 on chains took 15.8 s. Stations whose deck is a moving surface held their pallet the same way. Contacts are now worked out every step. |

With the two settings changed, the results are:

- **24 of 24 plant tests pass on Avian:**
  - T1 on rollers and on chains (12.25 s ±1 %);
  - T2 (about 667 pallets/h);
  - hand-over, the open end (±5 mm), blocking and accumulation;
  - the controller's rule at a photocell, the beam through the pallet's voids, determinism;
  - the C elevator (30.03 s), the manual table, the process station (42.03 s), the gravity line;
  - the whole reference layout, as in 2D.
- **P6 passes on Avian.** Every photocell edge of the 3D run lies within 50 ms of the 2D edge; on Rapier the falling edge
  is 79 ms late (ADR-0024). Avian runs it with `flow_tests!(AvianBench, p6)`; it stays ignored on Rapier.
- **The S1 figures are unchanged** by the two settings, except the P3 rebound, which is now 0.85 mm (limit 10 mm).

## What was built (2026-11-09)

Each item below has its own tests. The owner can open the plant three ways:

- the floor plan alone;
- **2D + 3D** in the main window;
- **View in 3D** in IntraCraft's 3D window.

**The 3D scene, engine-free (`intracraft-scene`).**

- Every object of the library is drawn with our own parts, after the families' forms:
  - rollers on the physics run's pitch, steel side frames, chain strands on their tracks;
  - legs on feet at most 2 m apart, with a cross beam;
  - drives, turntable bases, transfer chains, elevator masts, the process gantry;
  - gravity lines with their brake rollers, stands, photocell beams.
- A turntable's platform turns about its pivot, and a lift deck or an elevator carriage rises, by the run's angles and
  heights.
- This departs from decision 1 above (BIMCraft's own triangles), because BIMCraft's mesh does not separate the moving
  parts.
- `intracraft_scene::pose` places a run's pallets for any 3D view: the discrete run from its snapshot, the physics run
  from the bodies' axes. It turns the model a quarter for a pallet that travels short side first.

**Pallet and load models (`intracraft-assets`).** All our own geometry, with no third-party models.

- Each pallet type of the library is built **from its own occupancy bands**, so the picture and what a photocell sees
  are one construction.
  - EUR: 3 bottom boards, 9 blocks, 3 stringers, 5 top boards.
  - A closed deck is split into boards for the picture only.
- Four loads, built at the source's load height:
  - cartons in layers;
  - the same in stretch film;
  - steel drums out to the pallet's corners;
  - plastic crates.
- The source's choice decides the load:
  - `block` gives the wrapped cartons;
  - an asset gives the load of its category;
  - each pallet carries its source's id, so two sources of one pallet type can carry different loads.
- A glTF 2.0 binary writer of our own writes 15 `.glb` files into `apps/intracraft-viewer/assets/pallets`. A test keeps
  them up to date.

**IntraCraft's 3D window (`apps/intracraft-viewer`, Bevy 0.20).**

- What it draws:
  - the discrete run or the physics run on Avian;
  - the scene with physically based materials, cascaded shadows and 4× MSAA;
  - the pallets from the glTF files, and their loads at their height;
  - a plate in the state's colour under each object.
- How it is used:
  - an orbit camera; clicking an object picks it;
  - a panel with Run/Pause, speed, clock, pallets in the plant, delivered and fallen, warnings, and the picked
    object's statistics.
- Options: `--module`, `--play`, `--speed`, and `--frames N --screenshot FILE` (a picture of its own window, then it
  closes).

**The 3D view in the main window (2D + 3D).**

- A generic BIMCraft hook (`Extension::splits_view` and `split_view`, listed in `INTRACRAFT-CHANGES.md`) splits the view
  area: the floor plan stays on the left, IntraCraft's 3D view is on the right, and the split is resizable.
- The view is drawn on the graphics card the BIMCraft way:
  - its own wgpu pipeline into its own targets, shown by egui as a texture;
  - instancing and 4× MSAA;
  - the same scene, poses and models as the 3D window, without shadows.
- **View in 3D** opens the 3D window on the design as it is now. The window is started beside the program.
- BIMCraft's own 3D view no longer has flat pallet boxes painted over it.
- `--split` opens the main window with the split. `--screenshot FILE [--frames N]` saves a picture of the window.

**BIMCraft's MEP tabs are hidden** (owner's decision): Plumbing, Electrical, Lightning Protection and Data.

- This uses a generic hook (`Extension::hides_tab`).
- BIMCraft's code is untouched, so its tests and upstream updates are unaffected.

**Open end: stop or run off** (owner's decision, 2026-11-09). This changes specified behaviour, so it is recorded here
and in [[S01 Roller Conveyor]] and [[S02 Chain Conveyor]].

- Roller and chain conveyors have a new parameter, `openEnd`:
  - `stop` is the default; it is the behaviour of the reference, whose tests are unchanged;
  - `runOff` keeps the conveyor running, and pallets run off the end and fall.
- The parameter lives in the Rust library only (`sim/library/definitions/open-end.json`); the TypeScript reference is
  frozen.
- In 2D, the pallet leaves the plant when its rear passes the end (T1: 12.25 s from rest on 3 m) and counts as fallen
  (`fallen_pallets`).
- In 3D, the pallet tips over the end and falls. Pallets that fall, by design or not, now lie where they come to rest,
  as bodies (the last 12), instead of vanishing.
  - At 0.6 m a loaded EUR pallet noses down and stands against the conveyor's end, and the next one runs into it.
  - At 1.5 m pallets fall clear and tumble.
- Tests on both engines; `examples/layouts/pallet-drop` shows both cases.

**Seven example installations (`examples/layouts`):**

1. a roller line with zone control at a photocell;
2. rollers to chains through a transfer table;
3. a turntable feeding three lines;
4. a chain line with a chain turntable (GMA pallets of drums);
5. an elevator to a 4 m mezzanine and a gravity buffer;
6. a mixed plant of three lines;
7. pallets running off open ends.

A test runs all seven in both modules. Every sink receives pallets in 2D, and every one but the last carries pallets on
Avian with no fall.

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
