---
title: A18 BIMCraft Integration Analysis
lang: en
tags: [app, architecture, rust, bim, pivot]
updated: 2026-10-31
---

# A18 BIMCraft Integration Analysis

What BIMCraft already offers, what IntraCraft brings, how the two map onto each other, and the data model the simulation module adds. The decision is [[ADR-0020 Rust Platform on BIMCraft with Three Modules]]; the phased plan is [[A19 Rust Implementation Plan]].

Analysed: `IPedrax/BIMCraft`, commit `0733fd4` (2026-10-31), version 0.9.0, ~150 000 lines of Rust.

## 1. BIMCraft at a glance

| Crate / app         | Lines (approx.) | What it owns                                                                                                                                     |
| ------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `crates/core`       | 85 000          | Model, elements, ids and GUIDs, library, families, schedules, plans, 3D scene, sheets/PDF, IFC, DXF, commands and the `Session` (undo, rollback) |
| `crates/ui-egui`    | 48 000          | Desktop shell in egui 0.36: ribbon, Project Browser (tree), Properties palette, canvas, 3D orbit view on the GPU (wgpu), docking, Family Editor  |
| `crates/i18n`       | 700             | PO catalogues compiled into the app (English fallback; `es`, `pt`, `fr`, `cs`, `zh`)                                                             |
| `crates/update`     | 4 100           | Auto-update (only crate that uses the network)                                                                                                   |
| `crates/rvt`, `dwg` | 12 000 (opt.)   | Revit and DWG readers (licence fences: Apache-2.0 only; MPL-2.0 dependency)                                                                      |
| `apps/bimcraft`     | 750             | Desktop executable                                                                                                                               |
| `apps/bimcraft-cli` | 1 200           | Command line: scripts, conversion, **MCP server** for agents                                                                                     |

Rules worth keeping (from BIMCraft's `CLAUDE.md`): never crash (no `unwrap`/`panic`, no `unsafe`); **everything is a command** run by `Session::execute` with rollback; **the core has no UI**; the UI is thin and changes the model only through commands; test first; files are read leniently, commands strictly (`deny_unknown_fields`); file `format` with explicit migrations. These are the same principles as IntraCraft's [[ADR-0004 AI-Ready Command Layer and MCP]] and [[ADR-0006 Command Package and Geometric Port Connections]].

### 1.1 What the owner wants and where it already is

| Owner's requirement           | In BIMCraft today                                                                                                                                   | Gap                                                                                     |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| Tables (schedules)            | `schedule.rs`: per category, fields, calculated fields, filters, sorting, grouping, totals, CSV and XLSX; components show their family's parameters | A schedule category for the simulation objects; statistics as schedule fields           |
| Budgets                       | Not built; listed as idea `CON-01` (quantity take-off and cost)                                                                                     | Cost per type/material, price list import, a "Budget" schedule (see §5)                 |
| Visualisation                 | Plans, sections, elevations, 3D view (software and GPU), sheets, PDF, IFC                                                                           | Animated pallets in plan and 3D during a run                                            |
| Parametric "families"         | `family.rs`: parameters (type or instance, formulas), types, forms (extrusions, revolves, voids, meshes), materials; Family Editor                  | A family category for intralogistics; ports; a behaviour id; more value kinds           |
| Unique ID per instance        | `ids.rs`: six-digit `Id` (from 100000) + `Guid` that never changes; library entries count down from 999999                                          | Run-time ids for MTBH (§3.3)                                                            |
| Inherit from a library object | Family → type → instance: an instance takes its type's values unless it sets its own                                                                | Explicit *inherited* flag and description per attribute; "detach" (no inheritance) (§4) |
| Attribute table               | Properties palette (instance and type parameters); project parameters of kind Text, Number, Length, Yes/No                                          | Attribute table with kinds, description, inherited flag, units (§4)                     |
| Tree of simulation elements   | Project Browser (`browser.rs`) is a searchable tree of views, schedules, sheets, families                                                           | A **Simulation** tab/tree (§3.2)                                                        |
| Statistics sheet              | Nothing                                                                                                                                             | Per-object statistics from the last run (§6)                                            |
| 2D discrete simulation        | Nothing (no time, no events)                                                                                                                        | New crate `sim-des` (port of `packages/sim-des`)                                        |
| 3D physics simulation         | Nothing (the 3D view is static)                                                                                                                     | New crate `sim-physics` (port of `packages/sim-physics` on Rapier)                      |
| Agents                        | CLI + MCP server over the same commands                                                                                                             | Simulation commands appear there automatically once they are commands                   |

## 2. What IntraCraft brings

| IntraCraft asset                                  | Size / status                                | Fate in the new platform                                                                               |
| ------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Specs S01–S17, theory T01–T15, ADR-0007–0017      | Complete for the MVP and the 30-10 extension | **Kept as is**: the specification of the Rust code                                                     |
| Analytical tests T1–T5, scenarios, `test_data/`   | Pass in TypeScript                           | **Kept**: become Rust tests; the TypeScript results are the oracle                                     |
| `packages/library` (JSON definitions, validation) | ~1 400 lines; 17 object definitions          | Definitions become **family templates** (JSON → family + simulation metadata); validation rules ported |
| `packages/core-model` (layout, ports, units)      | ~400 lines                                   | Ports and connections move into a `sim-model` crate on top of BIMCraft's elements                      |
| `packages/commands` (session, undo, log)          | ~750 lines                                   | Replaced by BIMCraft's `Session` and commands; IntraCraft's commands are re-declared there             |
| `packages/sim-des` (flow model, stations, eyes)   | ~5 000 lines                                 | **Ported** to the `sim-des` crate                                                                      |
| `packages/sim-physics` (Jolt, world, MTBH)        | ~4 700 lines                                 | **Ported** to the `sim-physics` crate on Rapier                                                        |
| `packages/control` (if/then rules)                | ~100 lines                                   | Ported to the `sim-control` crate                                                                      |
| `packages/geometry` (parametric 3D, 2D symbols)   | ~230 lines                                   | Replaced by family forms and symbols                                                                   |
| `packages/asset-catalog` (third-party GLB models) | ~400 lines + manifest                        | Kept as a catalogue; GLB meshes become family `mesh` forms (licences re-checked)                       |
| `apps/editor` (React + three.js)                  | ~5 100 lines                                 | **Not ported**: replaced by egui panels; its behaviour serves as the UI reference                      |

## 3. Data model added by the simulation module

### 3.1 Simulation objects are elements

A placed conveyor is a BIMCraft **component** (`Element::Component`) of a family whose category is one of the new intralogistics categories. It therefore gets for free: an `Id` and a `Guid`, a level, position and angle, its type, its instance values, schedules, IFC export (as `IfcTransportElement`/`IfcBuildingElementProxy`), plans, sections and the 3D view.

What a component does not have, and the module adds as a `sim` block on the family and on the instance:

| Field         | Where                     | Meaning                                                                                       |
| ------------- | ------------------------- | --------------------------------------------------------------------------------------------- |
| `behaviour`   | family                    | Id of the Rust behaviour (`roller-conveyor`, `chain-turntable`, …), as `library` registers it |
| `ports`       | family (formulas allowed) | Entry/exit ports in family axes, with direction and transport type (rollers, chains)          |
| `signals`     | family                    | Inputs and outputs for the controller (sensor states, motor on/off, stop up/down)             |
| `included`    | instance                  | Whether the element takes part in the simulation (shown as a check box in the tree)           |
| `connections` | model                     | Port-to-port links, by GUID (geometric snapping as in ADR-0006, stored explicitly)            |
| `rules`       | model                     | Controller rules (ADR-0010), referring to elements by GUID                                    |

Sensors, stops and guides are components hosted on a conveyor (as doors are hosted on walls in BIMCraft): their position along the deck is an instance attribute.

### 3.2 The simulation tree

A new **Simulation** tab in the Project Browser, built by a core query (not by the UI), lists every element with `included = true`:

```
Simulation
├─ Model "Line A"                        (the project's simulation settings: seed, duration, mode)
│  ├─ Sources
│  │  └─ 100231  Source · EUR pallet
│  ├─ Conveyors
│  │  ├─ 100245  Roller conveyor · RC-2000   [inherited 14 / overridden 2]
│  │  ├─ 100246  Chain turntable · CT-90
│  │  └─ …
│  ├─ Sensors · Stops · Guides            (grouped under their host conveyor, or by kind; user's choice)
│  ├─ Stations · Lifts · Elevators
│  ├─ Sinks
│  ├─ Controller rules
│  └─ MTBH (last run)                     (read-only list of pallets created in the last run)
└─ Library (simulation families and their types)
```

Selecting a node selects the element (BIMCraft's selection is the model's, so plan, 3D view and schedules highlight it). Grouping, search and filtering reuse `browser.rs`. A check box per node toggles `included` through a command.

### 3.3 Unique IDs

| Object                                 | ID                                                      | Lifetime                             |
| -------------------------------------- | ------------------------------------------------------- | ------------------------------------ |
| Placed objects (conveyors, sensors, …) | BIMCraft `Id` (6 digits, e.g. `100245`) + `Guid`        | As long as the element exists        |
| Library objects (families, types)      | Library `Id` (counting down from `999999`) + `Guid`     | As long as the entry exists          |
| MTBH (pallets, loads) during a run     | `MTBH-` + run sequence (`MTBH-000123`), seeded          | One run; identical for the same seed |
| Ports                                  | Part `Id` + `Guid` of the family (BIMCraft's `HasPart`) | As long as the family keeps it       |

Connections, rules and statistics always hold the **GUID**, never the six-digit id (BIMCraft's rule), so ids may be renumbered on import without breaking links.

## 4. Attribute table with inheritance

### 4.1 One attribute

Every object (library family, type, placed instance) shows an **attribute table**. Each row is one attribute:

| Column       | Content                                                                                        |
| ------------ | ---------------------------------------------------------------------------------------------- |
| Key          | Stable English identifier (`speed`, `rollerPitch`, `accumulation`)                             |
| Label        | Shown name, translated (`en` required; `es`, `de`, `pt` optional)                              |
| Kind         | One of the kinds below                                                                         |
| Value        | The effective value, shown in display units (mm, °, mm/s)                                      |
| Unit         | SI internally; display unit at the edges                                                       |
| Description  | Free text: what the attribute means and what result it changes                                 |
| Inherited    | ✔ the value comes from the type/library; ✖ the instance overrides it                           |
| Source       | Where the effective value comes from: family default, type, instance, formula                  |
| Group, flags | Group (Geometry, Kinematics, Control, Statistics…), advanced, affects (2D, 3D, both, geometry) |
| Limits       | Min, max, list of allowed values, validation message                                           |

Value kinds (BIMCraft today has Length, Number, Angle, Yes/No, Material; the module adds the rest): **Integer, Real, Length, Angle, Speed, Acceleration, Angular speed, Time, Mass, Yes/No, Text, Choice (enum), Material, Element reference (GUID), Table (list of rows, e.g. a sequence of pallet types)**.

### 4.2 Inheritance chain

```
Library family  ──►  Type  ──►  Instance
(defaults)          (preset)   (placed object)
```

- The effective value is the first that is set, from the instance back to the family default; formulas are evaluated with BIMCraft's formula engine (`family::eval`).
- **Inherited ✔** means the instance holds no value of its own: a change to the type changes this instance. Editing the value in the table sets it and clears the flag; the command `attribute_reset` sets the flag again (removes the override).
- **Born without inheritance:** a placed object can be **detached** (`detach_from_library`): all effective values are copied into the instance, every flag becomes ✖, and later library changes no longer reach it. The object keeps a note of the family it came from.
- **Library updates:** when a library family is reloaded, inherited values follow it; overridden ones are kept; a report lists the instances with overrides that now differ from the new default.
- **Attributes added to one instance only** ("user attributes") are allowed, marked ✖ and listed under a *User* group. They can be scheduled like BIMCraft's project parameters.

This is the same mechanism as BIMCraft's type/instance parameters made visible and complete; IntraCraft's parameter schema (key, label, type, unit, default, min, max, group, description, advanced, affects, derived, dependsOn; `AGENTS.md`) maps one to one onto §4.1.

## 5. Budgets

Budgets do not exist yet in BIMCraft. The proposal, kept small and generic so it can go upstream:

- A **cost** attribute group on types and materials: unit cost, unit (each, m, m², m³, kg), currency, price list code.
- **Price lists** imported from CSV/XLSX (public tables such as SINAPI in Brazil, or the user's own); clean-room: no bundled commercial catalogues.
- A **Budget** schedule category: one row per type (count or quantity × unit cost), grouped by discipline; the intralogistics objects add their own quantities (number of rollers, chain length, motors, sensors).
- Export to XLSX through BIMCraft's existing `xlsx.rs`.

## 6. Statistics sheet

Each simulation object has a **statistics sheet** filled from the last run (2D or 3D). Statistics are not saved in the project file; they live in the run result and can be exported (CSV/XLSX/JSON) or shown as schedule fields while a result is loaded.

| Object              | Basic statistics                                                                                                                 |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Every object        | Pallets entered, pallets left, current content, time working / waiting / blocked / empty (s and %), utilisation, throughput (/h) |
| Conveyor, deck      | Mean and max occupancy, mean dwell time, blocked time at exit                                                                    |
| Turntable, transfer | Cycles, mean cycle time, time turning or lifting                                                                                 |
| Sensor              | Activations, total time occupied, mean gap between pallets                                                                       |
| Stop                | Times raised, pallets held, mean hold time; 3D: mean and max impact energy, final position error (mm)                            |
| Source              | Pallets created, pallets refused (blocked), mean inter-arrival time                                                              |
| Sink                | Pallets removed, mean and max lead time (from source), throughput                                                                |
| MTBH                | Lead time, distance travelled, waiting time, path (list of objects passed)                                                       |

Definitions follow [[T08 Statistics and Output Analysis]] (time-weighted averages, warm-up period excluded). The sheet shows the numbers, a bar of the time states and, for 3D, the physics checks (stop error, rebound).

## 7. Architecture of the new workspace

```
intracraft-bim/                      (fork of BIMCraft)
├─ crates/
│  ├─ core/                ← BIMCraft (minimal extension points only)
│  ├─ ui-egui/             ← BIMCraft (+ Simulation tab, attribute table, statistics, run controls)
│  ├─ i18n/ update/ rvt/ dwg/          ← BIMCraft, unchanged
│  ├─ sim-model/           NEW  ports, connections, attributes with inheritance, flow network, validation
│  ├─ sim-library/         NEW  intralogistics family templates (from library/definitions), behaviour registry
│  ├─ sim-des/             NEW  event queue, flow model, stations, photoeyes (port of packages/sim-des)
│  ├─ sim-physics/         NEW  Rapier world, decks, MTBH bodies, raycast sensors (port of packages/sim-physics)
│  ├─ sim-control/         NEW  signals and if/then rules (port of packages/control)
│  ├─ sim-stats/           NEW  statistics collectors and the statistics sheet data
│  └─ sim-scenarios/       NEW  T1–T5 and the reference scenarios, golden results
└─ apps/
   ├─ bimcraft/            desktop (renamed if the owner wishes)
   └─ bimcraft-cli/        + `scenario <id|all>` and simulation commands for the MCP server
```

Dependency rules: `sim-*` crates depend on `bimcraft-core`, never the reverse; the engines (`sim-des`, `sim-physics`, `sim-control`) have no UI dependency; the UI shows snapshots produced by a run on a background thread (the Rust equivalent of the Web Worker).

## Related

- [[ADR-0020 Rust Platform on BIMCraft with Three Modules]] · [[A19 Rust Implementation Plan]]
- [[A04 Data Model and Project File]] · [[A05 Component Library]] · [[A06 DES Engine]] · [[A07 Physics Engine Wrapper]] · [[T08 Statistics and Output Analysis]]
