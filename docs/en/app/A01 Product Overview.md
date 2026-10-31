---
title: A01 Product Overview
lang: en
tags: [app, overview]
---

🌐 **EN** · [[A01 Visión del producto|ES]] · [[A01 Produktüberblick|DE]] · [[Home (EN)|Index]]

# A01 Product Overview

## What IntraCraft is

An application to **design and simulate pallet conveyor systems**. Since 2026-10-31 it is built in Rust on the BIM platform BIMCraft ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]) and has **three modules on one project**:

| Module              | What it does                                                                                                                                                     | Engine                                                | Speed              | Answers                                                  |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- | ------------------ | -------------------------------------------------------- |
| Design              | BIM editor: place and connect library objects (families), attribute table with inherited flags, simulation tree, schedules, budgets, plans, sections, 3D, sheets | `bimcraft-core`, `intracraft-model`                   | Interactive        | What the plant is, what it is made of and what it costs  |
| Discrete simulation | 2D event-driven run of the layout                                                                                                                                | `intracraft-des`                                      | ≥ 1000× real time  | Throughput, accumulation, blocking, utilisation          |
| Physics simulation  | 3D run with rigid bodies                                                                                                                                         | `intracraft-physics` (Rapier), Bevy viewer (ADR-0022) | Real time (60 FPS) | Friction, impacts, alignment, sensors with real geometry |

Both simulation modules read the layout of the Design module and are driven by the same If/Then controller. Every placed object has a unique id and GUID, an attribute table and a statistics sheet ([[A18 BIMCraft Integration Analysis]]). Theory: [[T01 Simulation Paradigms]].

The first implementation, in TypeScript (web editor, 2D and 3D modes), is kept frozen as the **reference**: its tests and scenario files are what the Rust modules must reproduce.

## Design principles

1. The layout of the Design module is the source of truth; 2D, 3D, schedules and statistics are derived from it.
2. Same control logic, different plant.
3. Every parameter has a unit, range, default and description; the editing UI is generated from that schema.
4. Determinism: same data + same seed → same results.
5. Real pallet geometry matters (voids between boards and blocks).
6. Validate before extending: analytic tests and comparison with known reference models.
7. Small steps, always with a test.

## MVP scope

**In:** five conveyors ([[S01 Roller Conveyor]], [[S02 Chain Conveyor]], [[S03 Chain Turntable]], [[S04 Roller Turntable]], [[S05 Roller-to-Chain Transfer]]); photoelectric sensors (diffuse, retroreflective, through-beam); fixed and retractable stops; side guides with funnel; source and sink; EUR, 1200 × 1000, GMA and custom pallets; If/Then controller with boolean signals; both modes on the same layout (switching restarts); editor (place, connect, parametrise, save, open; play, pause, speed); basic statistics, CSV/JSON export, headless CLI; built-in library with presets and user components.

**Out (post-MVP):** curves, lifts, shuttles, diverters; boxes and totes; failures (MTBF/MTTR) and energy; user scripting; mid-run mode switch; automatic 2D↔3D calibration; experiments and optimisation; OPC UA / PLC; CAD import; collaboration.

## MVP definition of done

1. The five conveyors can be placed, connected and parametrised from the library.
2. A reference layout with all five runs in both modes unattended.
3. 2D results match the analytic tests within 1 % ([[A11 Testing and Validation]]).
4. In 3D a pallet stops against a stop within ±5 mm, rebound < 10 mm.
5. Transit times and sensor events differ between modes by < 5 % and ±50 ms.
6. Same layout + same seed → identical results in two runs.
7. 3D keeps 60 FPS with 20 conveyors and 10 pallets on a current laptop.
8. Three known reference models are reproduced and compared.

## Current status (2026-10-31)

- **TypeScript reference (frozen):** the five conveyors, lifts, elevator, gravity line, process station, sensors, source and sink in 2D; the 3D physics world on Jolt; editor, CLI and scenarios ([[A12 Roadmap]]).
- **Rust platform:** BIMCraft is in `platform/` with every library vendored; the port follows [[A19 Rust Implementation Plan]].

## Related

[[A02 Architecture]] · [[A12 Roadmap]] · [[Glossary]]
