---
title: A18 Rust Implementation Plan
lang: en
tags: [app, roadmap, rust, bim, pivot]
updated: 2026-10-31
---

# A18 Rust Implementation Plan

How to turn BIMCraft into the base of IntraCraft's simulation, in phases that each end with something the owner can run and check. Why: [[ADR-0018 Pivot to a Simulation Module on a Rust BIM Platform]]. What exists and the data model: [[A17 BIMCraft Integration Analysis]].

Every phase follows the IntraCraft workflow: spec first, failing test, implementation, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, a summary in simulation terms, one small commit per working step.

## 1. Phases at a glance

| Phase | Name                                   | The owner can …                                                                                           | Depends on |
| ----- | -------------------------------------- | --------------------------------------------------------------------------------------------------------- | ---------- |
| R0    | Fork and ground rules                  | Open BIMCraft built from our repository; read the merged agent rules                                      | ADR-0018   |
| R1    | Simulation families and attributes     | Place a roller conveyor from the library, see its attribute table with inherited flags, schedule it       | R0         |
| R2    | Ports, connections and simulation tree | Snap conveyors together, see the Simulation tree, get validation errors (open port, wrong transport type) | R1         |
| R3    | DES engine in Rust                     | Run T1–T5 and the reference scenarios from the CLI with the same results as the TypeScript version        | R2         |
| R4    | 2D run in the desktop                  | Press Run in a plan view, watch pallets move, pause, change speed, see statistics sheets                  | R3         |
| R5    | Physics spike S1 in Rust               | Run the stop, rebound and chain tests on Rapier; decide Rapier or Jolt                                    | R0         |
| R6    | 3D physics run                         | Press Run in the 3D view, watch pallets driven by rollers and chains, compare 2D/3D sensor events         | R4, R5     |
| R7    | Controller and remaining objects       | Write if/then rules; use lifts, elevator, gravity line, stations (S11–S15)                                | R4         |
| R8    | Budgets and reports                    | Get a budget schedule with unit prices; export statistics and budget to XLSX                              | R1         |
| R9    | Agents and packaging                   | Drive simulations through the MCP server; install the app on Windows/macOS/Linux                          | R6, R7     |

R5 can run in parallel with R1–R4; R8 in parallel with R3–R7.

## 2. Phases in detail

### R0 – Fork and ground rules

- Fork `IPedrax/BIMCraft` into the owner's account (name to confirm, ADR-0018 §6); keep an `upstream` remote and a monthly merge.
- Merge the two rule sets into one `AGENTS.md`/`CLAUDE.md`: BIMCraft's (never crash, everything is a command, core without UI, file format and migrations, test first, layered agent docs) plus IntraCraft's (SI units inside, seeded randomness, determinism tests as top priority, Spanish explanations for the owner, ask before architecture changes, no change to `test_data/` to make a test pass).
- **Units:** BIMCraft stores millimetres; IntraCraft's engines use SI. Decision proposed: the model keeps millimetres (BIMCraft's files and commands), the `sim-model` crate converts once when it builds the flow network, the engines stay in SI.
- Copy `docs/en/specs`, `theory`, `decisions` and `test_data/` into the new repository (`docs/simulation/`), unchanged.
- CI: BIMCraft's `cargo fmt`, `clippy` and `test` on every pull request.
- **Exit:** the forked app builds and its whole test suite passes on CI.

### R1 – Simulation families and the attribute table

- New family categories: conveyor, transfer, turntable, lift, sensor, stop, guide, source, sink, station, MTBH.
- `sim-library`: read IntraCraft's 17 JSON definitions (`packages/library/src/definitions`) and turn each into a BIMCraft family: parameters (with formulas for derived values), types from the presets, forms from the parametric geometry (deck, rollers as a repeated cylinder or a mesh, frame, legs), plan symbols.
- Extend family parameters with: description, more value kinds (A17 §4.1), display unit, min/max/list, group, flags (`advanced`, `affects`). File `format` bumped with a migration.
- Inheritance: commands `attribute_set`, `attribute_reset`, `detach_from_library`, `attribute_add` (user attribute), each with its test; `Model::check` validates ranges and lists.
- UI: an **Attribute table** panel (egui grid) generated from the schema: no hand-written forms. Columns: key, label, value, unit, inherited ✔/✖, source, description (tooltip and column). Reset button per row.
- A schedule category *Intralogistics* listing the simulation objects and their attributes.
- **Tests:** S01 defaults; inheritance chain (family → type → instance), override, reset, detach; library reload keeps overrides; hostile values never panic (BIMCraft's sweep).
- **Exit:** the owner places a roller conveyor of type RC-2000, changes its speed, sees ✖ on that row and ✔ on the rest, and sees it in a schedule.

### R2 – Ports, connections and the simulation tree

- `sim-model`: ports as family parts with id and GUID (`HasPart`); geometric snapping as in [[ADR-0006 Command Package and Geometric Port Connections]]; connections stored by GUID; validation (open ports, transport type mismatch, pallet orientation) in `Model::check`.
- Hosted objects: sensors, stops, guides placed on a conveyor, their position along it an instance attribute.
- `included` flag per element; command `sim_include`.
- Query `sim_tree` in the core; **Simulation** tab in the Project Browser (A17 §3.2), with search, grouping by kind or by line, check boxes, selection shared with plan, 3D view and schedules.
- **Exit:** the owner builds the reference layout of the TypeScript editor in the BIM app and the tree shows every element with its ID.

### R3 – DES engine in Rust

- `sim-des`: event queue ordered by (time µs, priority, sequence); seeded random generator (e.g. `rand_chacha`, licence checked); kinematics (trapezoidal profiles, T04); flow network built from the model; line conveyor without accumulation (ADR-0007); turntable (ADR-0008); transfer (ADR-0009); photoeye; source; sink.
- Port order: kinematics → event queue → network → line conveyor → source/sink → photoeye → turntable → transfer.
- Golden results: export the event logs of the TypeScript scenarios once (`pnpm run scenario all`) into `test_data/`; the Rust tests compare against them.
- `sim-scenarios` + CLI `scenario <id|all>`.
- **Tests:** T1 12.25 s, T2 ≈ 0.185 pallets/s, T3 6.5 s, T4 7.4 s, T5 32 J, all within 1 %; scenario event logs identical; determinism (same layout and seed → same log, run twice and on two threads).
- **Exit:** `bimcraft-cli scenario all` passes with the same numbers as `pnpm run scenario all`.

### R4 – 2D run in the desktop

- A run happens on a background thread; the UI receives snapshots (positions, states) at the display rate, never touches the engine state.
- Run controls in the ribbon (Simulate tab): Run, Pause, Step, Reset, speed factor (×1 … as fast as possible), duration, seed, warm-up.
- Pallets drawn on the plan canvas over the layout; colour by state (moving, waiting, blocked).
- `sim-stats`: collectors per object (A17 §6), time-weighted, warm-up excluded; **Statistics sheet** panel per selected object; export CSV/XLSX.
- **Exit:** the owner runs the reference line for one hour of model time in seconds and reads throughput and utilisation per conveyor.

### R5 – Physics spike S1 in Rust (in parallel)

- `sim-physics` behind a `PhysicsBackend` trait (as in TypeScript); first backend Rapier 3D, `enhanced-determinism`, fixed step 1/120 s.
- Rollers: kinematic cylinders with ω = v / r. Chains: static strands with surface velocity through `modify_solver_contacts` / `tangent_velocity`.
- Re-run S1: stop ±5 mm, rebound < 10 mm (tune and document contact settings, T15); chain transport speed error; determinism of two runs.
- If Rapier fails a criterion, try Jolt through `joltc-sys`/`oxijolt-sys` behind the same trait, and record the result in an ADR.
- **Exit:** an ADR that confirms or replaces the engine choice of ADR-0018, with numbers.

### R6 – 3D physics run

- Port `sim-physics/src/world` ([[ADR-0016 Physics World Built from the Layout]]): world from the model, decks, stations, gravity line, raycast sensors, MTBH bodies ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]), scenery ([[ADR-0017 Scenery Assets in the Layout and Live Spawn]]).
- Same plant logic as 2D, only the motion differs (ADR-0016 §1).
- The 3D view (BIMCraft's GPU path) draws pallet bodies from snapshots each frame; the static scene is cached.
- **Tests:** 2D/3D sensor events within ±50 ms on the reference scenarios; stop accuracy; statistics sheet adds impact energy and stop error.
- **Exit:** the owner switches the same project between 2D and 3D runs and compares statistics side by side.

### R7 – Controller and remaining objects

- `sim-control`: signals and if/then rules (ADR-0010), evaluated every 10 ms in 3D and on events in 2D; rules edited in a table panel; commands for each edit.
- S11 process station, S12 manual lift table, S13 scissor lift, S14 elevator (ADR-0013), S15 gravity line (ADR-0014), S16 contact materials.
- **Exit:** every object of the IntraCraft library passes its reference scenario in 2D and 3D.

### R8 – Budgets and reports

- Cost attributes on types and materials, price list import (CSV/XLSX), Budget schedule (A17 §5); intralogistics quantities (rollers, chain metres, motors, sensors).
- Report sheet: layout plan, statistics table, budget, on a BIMCraft sheet → PDF.
- Offer the generic budget part upstream to BIMCraft.
- **Exit:** the owner prints a sheet with the plan, the throughput results and the budget of the line.

### R9 – Agents and packaging

- Every simulation command is in `ALL`, so the CLI and the MCP server expose them (place, connect, set attribute, run, read statistics). Tokens never stored in the project (A15).
- Packages from BIMCraft's `packaging/` (Windows, macOS, Linux), with the new name and icon (`brand/`).
- **Exit:** an agent builds a small line, runs it and reports throughput through MCP; installers built by CI.

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
| R6    | L             | Performance with many roller bodies; determinism across platforms               |
| R7    | M             | Many objects, each with its own cycle                                           |
| R8    | M             | Scope creep (a full cost-estimating product)                                    |
| R9    | S             | Packaging signatures (inherited from BIMCraft)                                  |

## 5. What happens to the TypeScript repository

- Until R3 passes, it stays the **reference**: its scenario outputs are the golden results.
- New behaviour is specified in `docs/en/specs` first, then implemented only in Rust.
- After R6, the TypeScript editor is archived (kept read-only, not deleted).

## Related

- [[ADR-0018 Pivot to a Simulation Module on a Rust BIM Platform]] · [[A17 BIMCraft Integration Analysis]] · [[A12 Roadmap]]
- [[A06 DES Engine]] · [[A07 Physics Engine Wrapper]] · [[A08 Controller]] · [[A11 Testing and Validation]]
