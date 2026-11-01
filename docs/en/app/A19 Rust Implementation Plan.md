---
title: A19 Rust Implementation Plan
lang: en
tags: [app, roadmap, rust, bim, pivot]
updated: 2026-10-31
---

# A19 Rust Implementation Plan

How to turn BIMCraft into the base of IntraCraft's simulation, in phases that each end with something the owner can run and check. Why: [[ADR-0020 Rust Platform on BIMCraft with Three Modules]]. What exists and the data model: [[A18 BIMCraft Integration Analysis]].

Every phase follows the IntraCraft workflow: spec first, failing test, implementation, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, a summary in simulation terms, one small commit per working step.

## 1. Phases at a glance

| Phase | Name                                    | The owner can …                                                                                           | Depends on  |
| ----- | --------------------------------------- | --------------------------------------------------------------------------------------------------------- | ----------- |
| R0    | Repository and ground rules             | Build BIMCraft from this repository, offline, with every library inside; read the merged agent rules      | ADR-0020    |
| R1    | Design module: library and attributes   | Place a roller conveyor from the library, see its attribute table with inherited flags, schedule it       | R0          |
| R2    | Design module: ports and the tree       | Snap conveyors together, see the Simulation tree, get validation errors (open port, wrong transport type) | R1          |
| R3    | DES engine and controller in Rust       | Run T1–T5 and the reference scenarios from the CLI with the same results as the TypeScript version        | R1          |
| R4    | Discrete simulation module              | Press Run in a plan view, watch pallets move, pause, change speed, see statistics sheets                  | R2, R3      |
| R5    | Physics spike S1 in Rust                | Run the stop, rebound and chain tests on Rapier; decide Rapier or Jolt                                    | R0          |
| R5b   | 3D viewer spike (Bevy or BIMCraft view) | See the same busy scene in both viewers and choose one by frame rate and effort                           | R0          |
| R6    | Physics simulation module               | Press Run in the 3D view, watch pallets driven by rollers and chains, compare 2D/3D sensor events         | R4, R5, R5b |
| R7    | Remaining objects and rules editor      | Edit if/then rules in a table; use lifts, elevator, gravity line, stations (S11–S15) in both runs         | R4          |
| R8    | Budgets and reports                     | Get a budget schedule with unit prices; export statistics and budget to XLSX                              | R1          |
| R9    | Agents and packaging                    | Drive simulations through the MCP server; install the app on Windows/macOS/Linux                          | R6, R7      |
| R10   | IntraScript                             | Write code blocks and information objects (S18–S22) that run in both simulation modules                   | R4, Q20     |

R5 and R5b can run in parallel with R1–R4; R8 in parallel with R3–R7. By module: **Design** is R1, R2 and R8; **Discrete simulation** is R3, R4 and R7; **Physics simulation** is R5, R5b, R6 and R7; R10 serves both simulation modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]] §7).

## 2. Phases in detail

### R0 – Repository and ground rules (done 2026-10-31)

- BIMCraft is a git subtree in `platform/` (ADR-0020 §7); upstream changes come in with `git subtree pull --prefix=platform https://github.com/IPedrax/BIMCraft.git main --squash`. Changed BIMCraft files are listed in `platform/INTRACRAFT-CHANGES.md`.
- Every crate is vendored in `platform/vendor/` (`cargo vendor --versioned-dirs vendor`) and `platform/.cargo/config.toml` builds from it, offline; a new dependency comes with its vendored sources in the same commit.
- IntraCraft's crates live in `platform/sim/` (workspace member `sim/*`), its apps in `platform/apps/intracraft*`; BIMCraft's own `crates/` keep their rules (agent-docs test, translation catalogue).
- The two rule sets are merged in `AGENTS.md`: BIMCraft's (never crash, everything is a command, the core without UI, file format and migrations, test first) plus IntraCraft's (SI units inside, seeded randomness, determinism first, Spanish explanations for the owner, ask before architecture changes, `test_data/` untouched).
- **Units:** the model keeps millimetres (BIMCraft's files and commands); `intracraft-model` converts once when it builds the run, and the engines stay in SI.
- The documentation stays in `docs/` (one vault); the TypeScript reference is frozen and its scenario files are the oracle.
- CI: `.github/workflows/platform.yml` runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on Windows and Linux, offline.
- **Exit:** BIMCraft builds and its whole test suite passes from this repository.

### R1 – Design module: library, families and the attribute table

- **Done 2026-10-31:** `intracraft-library` (`platform/sim/library`, 51 tests). It reads the same JSON definitions and reference data as the TypeScript reference (`include_str!`, one source), ports the derived values, the nine validation rules, world ports and geometric connections; its tests reproduce `library.test.ts`, `ports.test.ts` and `snap.test.ts`.
- **Done 2026-11-01:** `intracraft-model` (`platform/sim/model`, 13 tests): each definition as a BIMCraft family (instance parameters with the library's defaults in mm and degrees, presets as types, derived values as formulas, a deck-and-frame form, the definition's id in a text type parameter); the attribute table with its source (instance, type, library, derived) and inherited flag; set, reset and detach through BIMCraft's `modify`; the simulation tree; the layout read back from the model; the reference's `.intracraft.json` layouts imported with the same connections. BIMCraft gained text parameters and instance values inherited from the type (file format 12, `platform/INTRACRAFT-CHANGES.md`).
- `platform/apps/intracraft`: the BIMCraft window with a module selector (Design, Discrete simulation, Physics simulation).

- New family categories: conveyor, transfer, turntable, lift, sensor, stop, guide, source, sink, station, MTBH.
- `sim-library`: read IntraCraft's 17 JSON definitions (`packages/library/src/definitions`) and turn each into a BIMCraft family: parameters (with formulas for derived values), types from the presets, forms from the parametric geometry (deck, rollers as a repeated cylinder or a mesh, frame, legs), plan symbols.
- Extend family parameters with: description, more value kinds (A18 §4.1), display unit, min/max/list, group, flags (`advanced`, `affects`). File `format` bumped with a migration.
- Inheritance: commands `attribute_set`, `attribute_reset`, `detach_from_library`, `attribute_add` (user attribute), each with its test; `Model::check` validates ranges and lists.
- UI: an **Attribute table** panel (egui grid) generated from the schema: no hand-written forms. Columns: key, label, value, unit, inherited ✔/✖, source, description (tooltip and column). Reset button per row.
- A schedule category *Intralogistics* listing the simulation objects and their attributes.
- **Tests:** S01 defaults; inheritance chain (family → type → instance), override, reset, detach; library reload keeps overrides; hostile values never panic (BIMCraft's sweep).
- **Exit:** the owner places a roller conveyor of type RC-2000, changes its speed, sees ✖ on that row and ✔ on the rest, and sees it in a schedule.

### R2 – Ports, connections and the simulation tree

- `sim-model`: ports as family parts with id and GUID (`HasPart`); geometric snapping as in [[ADR-0006 Command Package and Geometric Port Connections]]; connections stored by GUID; validation (open ports, transport type mismatch, pallet orientation) in `Model::check`.
- Hosted objects: sensors, stops, guides placed on a conveyor, their position along it an instance attribute.
- `included` flag per element; command `sim_include`.
- Query `sim_tree` in the core; **Simulation** tab in the Project Browser (A18 §3.2), with search, grouping by kind or by line, check boxes, selection shared with plan, 3D view and schedules.
- **Exit:** the owner builds the reference layout of the TypeScript editor in the BIM app and the tree shows every element with its ID.

### R3 – DES engine and controller in Rust

- **Done 2026-11-01:** `intracraft-control` (3 tests) and the core of `intracraft-des` (seeded PCG32 identical bit for bit to the reference, event queue, kinematics: T1–T5 within 1 %), and the first part of the flow model: conveyors without accumulation, sources, sinks, photoeyes and the controller (T1-2D 12.25 s, T2-2D 666.7/h, determinism, no overlap). Turntables, transfer, process station, lifts and gravity line are still inert in the Rust engine.
- `intracraft-control` (`platform/sim/control`): signals and if/then rules (ADR-0010), a pure evaluator as in the reference; evaluated on events in 2D and every 10 ms in 3D.
- `intracraft-des` (`platform/sim/des`): event queue ordered by (time µs, priority, sequence); seeded random generator: the reference's PCG32 (XSH-RR) ported bit for bit, so the same seed gives the same arrivals as the TypeScript run; kinematics (trapezoidal profiles, T04); flow network built from the model; line conveyor without accumulation (ADR-0007); turntable (ADR-0008); transfer (ADR-0009); photoeye; source; sink.
- Port order: kinematics → event queue → network → line conveyor → source/sink → photoeye → turntable → transfer.
- Golden results: export the event logs of the TypeScript scenarios once (`pnpm run scenario all`) into `test_data/`; the Rust tests compare against them.
- `sim-scenarios` + CLI `scenario <id|all>`.
- **Tests:** T1 12.25 s, T2 ≈ 0.185 pallets/s, T3 6.5 s, T4 7.4 s, T5 32 J, all within 1 %; scenario event logs identical; determinism (same layout and seed → same log, run twice and on two threads).
- **Exit:** `bimcraft-cli scenario all` passes with the same numbers as `pnpm run scenario all`.

### R4 – Discrete simulation module

- A run happens on a background thread; the UI receives snapshots (positions, states) at the display rate, never touches the engine state.
- Run controls in the ribbon (Simulate tab): Run, Pause, Step, Reset, speed factor (×1 … as fast as possible), duration, seed, warm-up.
- Pallets drawn on the plan canvas over the layout; colour by state (moving, waiting, blocked).
- `sim-stats`: collectors per object (A18 §6), time-weighted, warm-up excluded; **Statistics sheet** panel per selected object; export CSV/XLSX.
- **Exit:** the owner runs the reference line for one hour of model time in seconds and reads throughput and utilisation per conveyor.

### R5 – Physics spike S1 in Rust (in parallel)

- `sim-physics` behind a `PhysicsBackend` trait (as in TypeScript); first backend Rapier 3D, `enhanced-determinism`, fixed step 1/120 s.
- Rollers: kinematic cylinders with ω = v / r. Chains: static strands with surface velocity through `modify_solver_contacts` / `tangent_velocity`.
- Re-run S1: stop ±5 mm, rebound < 10 mm (tune and document contact settings, T15); chain transport speed error; determinism of two runs.
- If Rapier fails a criterion, try Jolt through `joltc-sys`/`oxijolt-sys` behind the same trait, and record the result in an ADR.
- **Exit:** an ADR that confirms or replaces the engine choice of ADR-0020, with numbers.

### R5b – 3D viewer spike (done 2026-10-31: Bevy, [[ADR-0022 Bevy Viewer for the Physics Simulation Module]])

- Decide the viewer of the 3D run ([[ADR-0021 Game Engine Choice for the Rust Platform]]): BIMCraft's own wgpu view with instancing, or a separate Bevy viewer process fed by snapshots through the control port.
- Same scene in both: 200 pallets, 2 000 rollers, 20 conveyors, catalogue GLB models; measure frame rate on a laptop with integrated graphics, start-up time, binary size and work needed.
- The simulation engines never depend on the viewer: Rapier is used directly, not through a Bevy plugin.
- **Exit:** an ADR that confirms or rejects the Bevy viewer, with numbers.

### R6 – Physics simulation module

- Port `sim-physics/src/world` ([[ADR-0016 Physics World Built from the Layout]]): world from the model, decks, stations, gravity line, raycast sensors, MTBH bodies ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]), scenery ([[ADR-0017 Scenery Assets in the Layout and Live Spawn]]).
- Same plant logic as 2D, only the motion differs (ADR-0016 §1).
- The viewer chosen in R5b (BIMCraft's GPU view or the Bevy viewer) draws pallet bodies from snapshots each frame; the static scene is cached.
- **Tests:** 2D/3D sensor events within ±50 ms on the reference scenarios; stop accuracy; statistics sheet adds impact energy and stop error.
- **Exit:** the owner switches the same project between 2D and 3D runs and compares statistics side by side.

### R7 – Remaining objects and the rules editor

- Rules edited in a table panel of the Design module, one command per edit; live signal values during a run.
- S11 process station, S12 manual lift table, S13 scissor lift, S14 elevator (ADR-0013), S15 gravity line (ADR-0014), S16 contact materials.
- **Exit:** every object of the IntraCraft library passes its reference scenario in 2D and 3D.

### R8 – Budgets and reports

- Cost attributes on types and materials, price list import (CSV/XLSX), Budget schedule (A18 §5); intralogistics quantities (rollers, chain metres, motors, sensors).
- Report sheet: layout plan, statistics table, budget, on a BIMCraft sheet → PDF.
- Offer the generic budget part upstream to BIMCraft.
- **Exit:** the owner prints a sheet with the plan, the throughput results and the budget of the line.

### R9 – Agents and packaging

- Every simulation command is in `ALL`, so the CLI and the MCP server expose them (place, connect, set attribute, run, read statistics). Tokens never stored in the project (A15).
- Packages from BIMCraft's `packaging/` (Windows, macOS, Linux), with the new name and icon (`brand/`).
- **Exit:** an agent builds a small line, runs it and reports throughput through MCP; installers built by CI.

### R10 – IntraScript (after the owner answers Q20)

- `intracraft-script` (`platform/sim/script`): the interpreter of [[A17 IntraScript Language]] in Rust (no `eval`, sandboxed, seeded random stream per block), and the information objects S18–S22 as families placed in the Design module.
- **Exit:** the reference examples of A17 run with the same results in both simulation modules.

## 3. Extension points in BIMCraft's core

Changes inside `crates/core` are kept to a short list, so upstream merges stay easy:

1. New family categories (`family::Category`) and the IFC class for each.
2. Parameter schema: description, new kinds, display unit, limits, group, flags; `format` bump and migration.
3. A generic **extension block** on families and elements (`ext: BTreeMap<String, serde_json::Value>`, validated by registered checkers), so `sim-model` stores ports, `included`, connections and rules without new element variants. Alternative: a dedicated `Element::SimConnection`; to decide in R2.
4. A hook for `Model::check` and for `visit_refs!` so GUID references inside the extension block are renumbered on import and validated.
5. A way for external crates to register commands in the command list.

Each of these is proposed upstream as a small, generic pull request; if refused, the fork carries it.

## 4. Effort and order (rough)

| Phase | Relative size | Main risk                                                                       |
| ----- | ------------- | ------------------------------------------------------------------------------- |
| R0    | S             | Two rule sets that conflict (units, language of docs)                           |
| R1    | L             | Changing BIMCraft's parameter schema without breaking its files                 |
| R2    | M             | Where connections live in the model (extension block vs new element)            |
| R3    | L             | Reproducing the TypeScript event times exactly (floating point, event ordering) |
| R4    | M             | Smooth animation in egui while the engine runs on another thread                |
| R5    | M             | Rapier contact tuning for ±5 mm                                                 |
| R5b   | S             | Two processes (editor and viewer) kept in step                                  |
| R6    | L             | Performance with many roller bodies; determinism across platforms               |
| R7    | M             | Many objects, each with its own cycle                                           |
| R8    | M             | Scope creep (a full cost-estimating product)                                    |
| R9    | S             | Packaging signatures (inherited from BIMCraft)                                  |

## 5. What happens to the TypeScript repository

- Until R3 passes, it stays the **reference**: its scenario outputs are the golden results.
- New behaviour is specified in `docs/en/specs` first, then implemented only in Rust.
- After R6, the TypeScript editor is archived (kept read-only, not deleted).

## Related

- [[ADR-0020 Rust Platform on BIMCraft with Three Modules]] · [[ADR-0021 Game Engine Choice for the Rust Platform]] · [[A18 BIMCraft Integration Analysis]] · [[A12 Roadmap]]
- [[A06 DES Engine]] · [[A07 Physics Engine Wrapper]] · [[A08 Controller]] · [[A11 Testing and Validation]]
