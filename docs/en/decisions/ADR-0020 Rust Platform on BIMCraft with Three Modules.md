---
title: ADR-0020 Rust Platform on BIMCraft with Three Modules
lang: en
tags: [decision, architecture, rust, bim, pivot]
status: accepted
date: 2026-10-31
---

# ADR-0020 Rust Platform on BIMCraft with Three Modules

- **Status:** Accepted on 2026-10-31. Proposed earlier the same day; the owner then confirmed the direction and answered §6 points 1, 3 and 6 (see §7, which amends the decision). Points 2, 4 and 5 stay open.
- **Date:** 2026-10-31
- **Plan:** [[A18 BIMCraft Integration Analysis]] (what exists, what maps to what) and [[A19 Rust Implementation Plan]] (phases and exit criteria).

## Context

The owner wants to change direction. Instead of growing IntraCraft as a stand-alone TypeScript application, the intralogistics simulation becomes **one extra module of a BIM platform**, starting from [BIMCraft](https://github.com/IPedrax/BIMCraft) (pure Rust, egui desktop, MIT OR Apache-2.0, version 0.9.0 on 2026-10-31).

What the owner asked for:

1. Keep the BIM functions: schedules (tables), quantities and budgets, plans, sections, the 3D view and sheets.
2. Add the two simulation modes of IntraCraft: **2D discrete-event (DES)** and **3D physics in real time**.
3. Conveyors (and the other library objects) behave like BIM **families**: parametric, with types and instances.
4. A **simulation tree** that lists every element included in the simulation.
5. Every instance (conveyor, sensor, MTBH/pallet…) has a **unique ID**.
6. An object may be born from a library object and **inherit** its properties, or not.
7. Every object has an **attribute table**: typed values, a text description and an *inherited* flag per attribute.
8. Every object has a **basic statistics sheet**.
9. The application is written in **Rust**, like BIMCraft.
10. (Added on confirmation) The program has **three modules**: **Design** (the BIM editor: families, attributes, schedules, budgets, plans, 3D), **Discrete simulation** (2D, event-driven) and **Physics simulation** (3D, real time).
11. (Added on confirmation) **Everything lives in this repository**: BIMCraft and every Rust library the build needs.

What we found (details in [[A18 BIMCraft Integration Analysis]]):

- BIMCraft already has most of the platform the owner wants: element ids and GUIDs, a library of types and families with ids, families with type and instance parameters and formulas, project parameters, schedules with calculated fields and XLSX export, a command layer with undo and rollback, a CLI, an MCP server, IFC import and export, a software and GPU 3D view, plans, sections and sheets.
- It has **no simulation, no time loop, no ports or connections between elements**, and **no budget** yet (cost take-off is only an idea, `CON-01` in its `IDEAS.md`).
- IntraCraft's value is in its **specifications, theory, decisions, analytical tests and reference scenarios** (S01–S17, T01–T15, ADR-0007–0017, T1–T5), and in ~10 000 lines of TypeScript simulation code (`sim-des`, `sim-physics`) whose behaviour is fixed by those documents and tests.

## Decision

1. **Fork BIMCraft** into a new repository (working name *IntraCraft BIM*) and keep it close to upstream, so its BIM work keeps flowing in. The simulation lives in **new crates** beside BIMCraft's, never inside its core files, except for a small set of extension points (§3 of [[A19 Rust Implementation Plan]]).
2. **Port, do not translate line by line.** The TypeScript engines are rewritten in Rust from the specifications, with the existing tests and saved results as the oracle: the Rust 2D run must reproduce the TypeScript event times (T1–T5 within 1 %, scenario sensor events identical).
3. **Physics engine: Rapier (native Rust)**, replacing Jolt. Reasons:
   - It is pure Rust, so the workspace keeps `unsafe_code = "forbid"` and needs no C++ toolchain on Windows, macOS, Linux or riscv64.
   - Native Rapier exposes the **surface velocity** that chain conveyors need: `SolverContact::tangent_velocity`, set in a `PhysicsHooks::modify_solver_contacts` hook (Rapier's "conveyor belt" recipe). The JS binding lacked it, which is why IntraCraft chose Jolt ([[ADR-0002 Physics Engine Candidate]]).
   - It has an `enhanced-determinism` feature for bit-identical runs.
   - Jolt stays the fallback through Rust bindings (`joltc-sys`/`rolt`, early and not clearly maintained; `oxijolt-sys` for Jolt 5.6), behind the same `PhysicsBackend` trait.
   - Spike S1 must be **re-run in Rust** (stop ±5 mm, rebound < 10 mm, 2D/3D sensor events within ±50 ms) before this point is accepted.
4. **Library objects become families of a new category family**, *intralogistics* (conveyor, transfer, turntable, lift, sensor, stop, guide, source, sink, station, MTBH). Their geometry is BIMCraft family forms; their simulation behaviour is a Rust behaviour registered by id, as today.
5. **Attributes with inheritance** follow BIMCraft's family model (family → type → instance), extended with a per-attribute *description*, more value kinds and an explicit *inherited / overridden* state (§4 of [[A18 BIMCraft Integration Analysis]]).
6. **Unique IDs** are BIMCraft's: a six-digit `Id` people and schedules use, plus a `Guid` that never changes. MTBH created during a run get a run-scoped id (`MTBH-000123`) and a GUID only if they are saved.
7. **Simulation results stay out of the project file** (as today): statistics sheets are computed from the last run and can be exported or written to a separate results file.
8. **The TypeScript repository is frozen** after this ADR is accepted: it stays as the reference implementation and the oracle for the port; new features go to the Rust repository.

## Alternatives considered

| Option                                                            | Consequences                                                                                                                                                                        |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A. Fork BIMCraft, port the simulation to Rust (proposed)**      | One language and one app; BIM functions for free; cost: rewriting ~10 000 lines of engine code and the editor's simulation views; the ported engines are re-validated against T1–T5 |
| B. Keep IntraCraft in TypeScript and exchange files with BIMCraft | No rewrite; two apps, two installs, no shared tree, attributes or schedules; the "one module of a BIM platform" goal is not met                                                     |
| C. Embed the TypeScript engines in the Rust app (JS runtime)      | Reuses the engines quickly; adds a JavaScript runtime, breaks "pure Rust", Jolt WASM inside a desktop app, two debugging worlds; a dead end for performance and determinism         |
| D. Start a new Rust app and copy BIMCraft pieces                  | Clean design; loses BIMCraft's future work and doubles the maintenance                                                                                                              |

## Consequences

- **Kept:** all specifications S01–S17, theory T01–T15, the decisions on behaviour (ADR-0007–0010, 0012–0017), the analytical tests T1–T5, the scenarios and `test_data/`. They become the specification and test oracle of the Rust code.
- **Superseded once accepted:** [[ADR-0001 Toolchain and Versions]] (TypeScript, pnpm, Vite, React) by Rust/Cargo/egui; [[ADR-0002 Physics Engine Candidate]] by Rapier, pending the Rust spike; parts of [[ADR-0006 Command Package and Geometric Port Connections]] (commands become BIMCraft commands; ports stay).
- **Licences:** BIMCraft is MIT OR Apache-2.0, compatible with a fork. Its `crates/rvt` is Apache-2.0 only and `crates/dwg` builds on acadrust (MPL-2.0); both are optional features and can stay so. Rapier is Apache-2.0.
- **Clean-room:** BIMCraft follows the same clean-room rule as IntraCraft (no vendor content). The rule in `AGENTS.md` carries over unchanged.
- **Language of the interface:** BIMCraft's UI uses PO catalogues (`translations/ui/es.po` exists); the simulation texts follow the same mechanism, English first.
- **Risk:** the port is the largest piece of work; it is split so that each phase ends with something the owner can run (see [[A19 Rust Implementation Plan]]).

## 6. Questions for the owner

1. **Fork or contribution?** Do we fork BIMCraft into our own repository, or propose the simulation module upstream to its author? (Fork assumed.)
2. **Name.** Does the product keep the name IntraCraft?
3. **Physics engine.** Accept Rapier subject to the Rust spike, or keep Jolt through bindings?
4. **Budget.** BIMCraft has no budget yet. Do we build the cost take-off (unit prices per type and per material, sums in a schedule) as part of this plan, or wait for upstream?
5. **Inheritance default.** When an instance is placed from a library object, are its attributes *inherited* (they follow later changes to the library) by default? (Yes assumed.)
6. **TypeScript editor.** Freeze it now, or keep it running until the Rust app reaches the same features (phase R4)?

## 7. Amendment on confirmation (2026-10-31)

The owner confirmed the direction and changed three points of the decision above. Where they differ, this section wins.

1. **No separate fork repository.** BIMCraft is brought **into this repository** as a git subtree in `platform/` (`git subtree add --prefix=platform … --squash`, BIMCraft commit `0733fd4`). Upstream changes come in with `git subtree pull --prefix=platform https://github.com/IPedrax/BIMCraft.git main --squash`. Files of BIMCraft that IntraCraft changes are listed in `platform/INTRACRAFT-CHANGES.md` (Apache-2.0 §4(b) asks modified files to say so, and the list keeps upstream merges easy).
2. **Every Rust library is vendored.** `cargo vendor` keeps the sources of all crates in `platform/vendor/`, and `platform/.cargo/config.toml` builds from them, offline. A new dependency is added in the same change as its vendored sources. `.gitattributes` marks `platform/vendor/**` as binary so Git never rewrites line endings (the crates' checksums would break).
3. **Three modules in one desktop application** (`platform/apps/intracraft`):

| Module              | What it is                                                                                                                                                                      | Engine                                                                                          |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Design              | BIMCraft's editor plus the intralogistics families: place and connect conveyors, attribute table with inherited flags, simulation tree, schedules, budgets, plans, sections, 3D | `bimcraft-core`, `intracraft-model`                                                             |
| Discrete simulation | The 2D event-driven run of the same layout, much faster than real time; animated plan, run controls, statistics sheets                                                          | `intracraft-des`, `intracraft-control`                                                          |
| Physics simulation  | The 3D run in real time: friction, contacts, stops, sensors with real geometry                                                                                                  | `intracraft-physics` (Rapier), viewer per [[ADR-0021 Game Engine Choice for the Rust Platform]] |

4. **The TypeScript implementation is kept as the reference**, frozen: no new features. Its tests and scenario files (`packages/scenarios/scenarios/*.json`, the ten layouts) are the oracle the Rust port must reproduce. It is archived once the Rust modules match it.
5. **Physics engine.** The owner allows any Rust game or physics engine. Rapier is used directly for the physics (decision 3 above); whether a game engine (Bevy) draws the 3D run is decided by spike R5b ([[ADR-0021 Game Engine Choice for the Rust Platform]]).
6. **Shared data, one source.** While the TypeScript reference exists, the library definitions and reference data stay where they are (`packages/library/src/definitions/*.json`, `packages/library/src/reference/*.json`); the Rust crates embed the same files at build time instead of copying them.
7. **Own crates live in `platform/sim/`**, not in BIMCraft's `crates/`, so BIMCraft's own checks (its agent-docs test and translation catalogue) stay untouched; the workspace lists `sim/*` as members.

Answers to §6: 1 → inside this repository (point 1 above); 3 → Rapier, any engine allowed (point 5); 6 → TypeScript frozen as the reference (point 4). Still open: 2 (product name; IntraCraft is assumed), 4 (budget in scope) and 5 (inheritance by default; assumed yes).

## Related

- [[A18 BIMCraft Integration Analysis]] · [[A19 Rust Implementation Plan]] · [[ADR-0021 Game Engine Choice for the Rust Platform]] (game engine for the 3D viewer)
- [[ADR-0001 Toolchain and Versions]] · [[ADR-0002 Physics Engine Candidate]] · [[ADR-0006 Command Package and Geometric Port Connections]] · [[ADR-0016 Physics World Built from the Layout]]
- Sources: [Rapier common recipes (conveyor belt)](https://www.rapier.rs/docs/user_guides/rust/common_recipes) · [`SolverContact` API](https://docs.rs/crate/rapier3d/latest/target-redirect/x86_64-unknown-linux-gnu/rapier3d/geometry/struct.SolverContact.html) · [jolt-rust bindings](https://github.com/Bantarus/jolt-rust) · [oxijolt-sys](https://docs.rs/crate/oxijolt-sys/latest/source/build/bindgen_options.rs)
