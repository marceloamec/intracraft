---
title: Home (EN)
lang: en
tags: [index]
---

🌐 **EN** · [[Inicio (ES)|ES]] · [[Startseite (DE)|DE]] · [[Home|All languages]]

# IntraCraft — documentation (English)

IntraCraft simulates pallet conveyor systems with **one layout and two modes**: a fast 2D **discrete-event** mode (throughput, accumulation, blocking, utilisation) and a real-time 3D **physics** mode (roller and chain friction, collisions with stops and guides, pallet alignment, sensors with real geometry).

## Reading paths

- **"I want to know how the 2D mode computes its results"** → [[T01 Simulation Paradigms]] → [[T02 Discrete-Event Simulation]] → [[T03 Time, Event Ordering and Determinism]] → [[T04 Conveyor Kinematics]] → [[T05 Accumulation, Capacity and Throughput]].
- **"I want to know how the 3D mode moves a pallet"** → [[T09 Rigid Body Dynamics]] → [[T10 Time Integration and Fixed Step]] → [[T11 Contact, Friction and Restitution]] → [[T12 Rollers and Chains in Physics]].
- **"I want to know how the app is built"** → [[A01 Product Overview]] → [[A02 Architecture]] → [[A03 Monorepo and Stack]] → [[A14 API Reference]].

## Theory

| #   | Note                                          | Mode |
| --- | --------------------------------------------- | ---- |
| T01 | [[T01 Simulation Paradigms]]                  | both |
| T02 | [[T02 Discrete-Event Simulation]]             | 2D   |
| T03 | [[T03 Time, Event Ordering and Determinism]]  | both |
| T04 | [[T04 Conveyor Kinematics]]                   | 2D   |
| T05 | [[T05 Accumulation, Capacity and Throughput]] | 2D   |
| T06 | [[T06 Photoelectric Sensing]]                 | both |
| T07 | [[T07 Control Logic and Signals]]             | both |
| T08 | [[T08 Statistics and Output Analysis]]        | both |
| T09 | [[T09 Rigid Body Dynamics]]                   | 3D   |
| T10 | [[T10 Time Integration and Fixed Step]]       | 3D   |
| T11 | [[T11 Contact, Friction and Restitution]]     | 3D   |
| T12 | [[T12 Rollers and Chains in Physics]]         | 3D   |
| T13 | [[T13 Turntables and Transfer Lifts]]         | both |
| T14 | [[T14 Stops, Impacts and Alignment]]          | both |
| T15 | [[T15 Physics Engine Research]]               | 3D   |
| T16 | [[T16 Hybrid Consistency and Calibration]]    | both |

## Application

| #   | Note                                |
| --- | ----------------------------------- |
| A01 | [[A01 Product Overview]]            |
| A02 | [[A02 Architecture]]                |
| A03 | [[A03 Monorepo and Stack]]          |
| A04 | [[A04 Data Model and Project File]] |
| A05 | [[A05 Component Library]]           |
| A06 | [[A06 DES Engine]]                  |
| A07 | [[A07 Physics Engine Wrapper]]      |
| A08 | [[A08 Controller]]                  |
| A09 | [[A09 Editor and Viewer]]           |
| A10 | [[A10 CLI and Scenarios]]           |
| A11 | [[A11 Testing and Validation]]      |
| A12 | [[A12 Roadmap]]                     |
| A13 | [[A13 Conventions]]                 |
| A14 | [[A14 API Reference]]               |
| A15 | [[A15 AI Agent Integration]]        |
| A16 | [[A16 Asset Catalog]]               |
| A17 | [[A17 IntraScript Language]]        |

## Specifications (library objects)

| #   | Note                             | Library id                   |
| --- | -------------------------------- | ---------------------------- |
| S01 | [[S01 Roller Conveyor]]          | `conveyor.roller`            |
| S02 | [[S02 Chain Conveyor]]           | `conveyor.chain`             |
| S03 | [[S03 Chain Turntable]]          | `conveyor.chain_turntable`   |
| S04 | [[S04 Roller Turntable]]         | `conveyor.roller_turntable`  |
| S05 | [[S05 Roller-to-Chain Transfer]] | `conveyor.transfer_rc`       |
| S06 | [[S06 Photoelectric Sensor]]     | `sensor.photoeye`            |
| S07 | [[S07 Mechanical Stop]]          | `mechanics.stop`             |
| S08 | [[S08 Side Guide]]               | `mechanics.guide`            |
| S09 | [[S09 Source and Sink]]          | `flow.source`, `flow.sink`   |
| S10 | [[S10 Pallet Types]]             | `pallet.*`                   |
| S11 | [[S11 Process Station]]          | `flow.process_station`       |
| S12 | [[S12 Manual Lift Table]]        | `conveyor.lift_table_manual` |
| S13 | [[S13 Scissor Lift Table]]       | `conveyor.scissor_lift`      |
| S14 | [[S14 Pallet Elevator]]          | `conveyor.elevator`          |
| S15 | [[S15 Gravity Roller Line]]      | `conveyor.gravity`           |
| S16 | [[S16 Contact Materials]]        | `material.*` (pair table)    |
| S17 | [[S17 Scenery Asset]]            | `scenery.asset`              |
| S18 | [[S18 Variable]]                 | `info.variable`              |
| S19 | [[S19 Table]]                    | `info.table`                 |
| S20 | [[S20 List]]                     | `info.list`                  |
| S21 | [[S21 Code Block]]               | `info.code`                  |
| S22 | [[S22 Area]]                     | `info.area`                  |

## Decisions

- [[ADR-0001 Toolchain and Versions]]
- [[ADR-0002 Physics Engine Candidate]]
- [[ADR-0003 Trilingual Documentation and English Identifiers]]
- [[ADR-0004 AI-Ready Command Layer and MCP]]
- [[ADR-0005 3D Asset Sources]]
- [[ADR-0011 Reference Review of a Commercial Simulator Manual]]
- [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]]
- [[ADR-0013 Lift Station 2D Cycle]]
- [[ADR-0014 Gravity Line 2D Model]]
- [[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]
- [[ADR-0016 Physics World Built from the Layout]]
- [[ADR-0017 Scenery Assets in the Layout and Live Spawn]]
- [[ADR-0018 IntraScript Scripting Language]]

## Reference

- [[Glossary]] — English / Spanish / German terms.
- [[Open Questions]] — decisions waiting for the project owner (with context and options).
