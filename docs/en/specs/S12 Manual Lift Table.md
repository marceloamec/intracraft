---
title: S12 Manual Lift Table
lang: en
tags: [spec, conveyor, lift, manual]
library-id: conveyor.lift_table_manual
status: implemented (2D)
updated: 2026-10-30
---

# S12 Manual Lift Table (`conveyor.lift_table_manual`)

A roller deck on a lift table where an **operator with a hand pallet truck (pallet jack)** puts pallets into the system or takes them out. On the operator side the deck goes down to pallet-jack level; on the other side it lifts to the height of the conveyor it serves. It is the interface between manual handling and the conveyor line. Added by [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]]; 2D cycle in [[ADR-0013 Lift Station 2D Cycle]].

Values are **indicative**, not manufacturer data. Rollers only for now.

## 1. Parameters

| Key                     | Unit | Default | Range         | Description                                          |
| ----------------------- | ---- | ------- | ------------- | ---------------------------------------------------- |
| `length`                | m    | 1.6     | 1.3 … 2.5     | Deck length; at least pallet length + margins        |
| `usableWidth`           | m    | 1.0     | 0.5 … 1.8     | Clear width between frames                           |
| `palletSideAlongTravel` | enum | long    | long, short   | Long on rollers (EUR 1200 mm along travel)           |
| `rollerDiameter`        | m    | 0.089   | 0.076, 0.089  | Same list as [[S01 Roller Conveyor]]                 |
| `rollerPitch`           | m    | 0.2063  | 0.133, 0.2063 | Same list as S01                                     |
| `nominalSpeed`          | m/s  | 0.25    | 0.05 … 0.6    | Roller speed while a pallet moves on or off          |
| `accel`                 | m/s² | 0.5     | 0.05 … 2      | Start and stop ramp of the rollers                   |
| `flowDirection`         | enum | in      | in, out       | `in`: operator → line; `out`: line → operator        |
| `lowHeight`             | m    | 0.10    | 0.05 … 0.40   | Deck height on the operator side (pallet-jack level) |
| `conveyorHeight`        | m    | 0.6     | 0.2 … 1.5     | Deck height on the conveyor side; match the conveyor |
| `liftSpeed`             | m/s  | 0.05    | 0.01 … 0.3    | Up speed of the table                                |
| `lowerSpeed`            | m/s  | 0.07    | 0.01 … 0.3    | Down speed of the table                              |
| `verticalAccel`         | m/s² | 0.2     | 0.05 … 2      | Start and stop of the vertical move (advanced)       |

**Ports** (both `inout`; which one is entry and which exit depends on `flowDirection`):

- `OP`: operator side, at x = 0, height `lowHeight`, facing −x.
- `LINE`: conveyor side, at x = `length`, height `conveyorHeight`, facing +x.

**Derived with the defaults:**

- Stroke: 0.6 − 0.1 = **0.5 m**.
- Up: 0.5 / 0.05 + 0.05 / 0.2 = **10.25 s**.
- Down: 0.5 / 0.07 + 0.07 / 0.2 = **7.49 s** (trapezoidal profile, [[T04 Conveyor Kinematics]] §2).
- Rollers under a 1200 mm side: **5** (≥ 3 ✓).

**Operator.** The operator is not a separate object yet. It is modelled with the existing blocks of [[S09 Source and Sink]], connected to `OP`:

- **Input** (`in`): a **source** whose `interval` is the time between two pallets brought by the operator.
- **Output** (`out`): a **sink** with `removalMode = afterTime`. Its `removalTime` is the operator's time per pallet (drive in, lift, leave, come back). The sink must be turned 180° to face `OP`.
- The minimum `surfaceHeight` of source and sink goes down to 0.05 m so they can meet `OP`.

Presets: **Manual input** (`flowDirection = in`) and **Manual output** (`flowDirection = out`).

## 2. 2D behaviour

One pallet at a time; cycle from [[ADR-0013 Lift Station 2D Cycle]].

**Input (`in`)**, rest at `lowHeight`:

1. **Receive.** The operator's pallet enters through `OP` and stops centred on the deck.
2. **Raise** to `conveyorHeight` (timed step, `liftSpeed`).
3. **Discharge** through `LINE`. If the conveyor cannot take the pallet, the table is **blocked**.
4. **Lower** empty to `lowHeight` (`lowerSpeed`); then idle.

**Output (`out`)**, rest at `conveyorHeight`:

1. **Receive.** The pallet enters through `LINE`, travelling −x, and stops centred.
2. **Lower** to `lowHeight`.
3. **Hand over** through `OP` to the operator (sink). The table is **blocked** while the operator is busy.
4. **Raise** empty to `conveyorHeight`; then idle.

The pallet does not turn. In `out` its side along travel is the same as on the conveyor.

## 3. Statistics

Same set for [[S12 Manual Lift Table]], [[S13 Scissor Lift Table]] and [[S14 Pallet Elevator]].

| Key             | Meaning                                                                           |
| --------------- | --------------------------------------------------------------------------------- |
| `utilization`   | Share of time in a cycle (receiving, moving, discharging, returning), not blocked |
| `blockedShare`  | Share of time with a pallet waiting at the exit for the next object               |
| `idleShare`     | Share of time waiting for a pallet                                                |
| `palletsMoved`  | Pallets discharged                                                                |
| `meanCycleTime` | Mean time from accepting a pallet to being idle again (s)                         |

`utilization + blockedShare + idleShare = 1`.

## 4. Validations

| Rule                     | When                                                     |
| ------------------------ | -------------------------------------------------------- |
| `noOperator`             | `OP` is not connected                                    |
| `operatorMismatch`       | `in` with a sink on `OP`, or `out` with a source on `OP` |
| `noExit`                 | `in` and `LINE` is not connected                         |
| `heightsReversed`        | `lowHeight` ≥ `conveyorHeight`                           |
| `deckTooShort`           | `length` below the pallet side along travel + 0.1 m      |
| `min3RollersUnderPallet` | As [[S01 Roller Conveyor]]                               |

## 5. Not included yet

- Signals (`enable`, `atTop`, `atBottom`, `occupied`), push-button or foot-pedal control, and safety devices (light curtain).
- The operator as a separate object: walking times, or one operator serving several tables.
- A table that waits at the other level.
- 3D physics: done on 2026-10-31 (`Station3D`). The platform carries the pallet up and down by contact; the cycle is the 2D one ([[ADR-0016 Physics World Built from the Layout]]).

The real pallet-jack height and how the jack enters are open: [[Open Questions]] Q14.

## 6. Reference tests

- With the defaults: **up 10.25 s, down 7.49 s** (error < 1 %). The full cycle adds the centring and discharge times from T04.
- `in`: source on `OP` → pallets reach the conveyor, one per cycle; the table is blocked while the conveyor is full.
- `out`: sink with `removalTime` 60 s on `OP` → at most 60 pallets/h; the table is blocked while the operator is busy.
  - Measured (scenario LIFT-OUT): **55.3 pallets/h**. While the operator is busy, the next pallet already waits at the deck end. When the operator is free it needs 1.2 m from rest to clear the deck (1.2 / 0.25 + 0.25 / (2 × 0.5) = 5.05 s), so one cycle is 60 + 5.05 s.
- Validations: `noOperator`, `operatorMismatch`, `heightsReversed`.

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                                                                                                                                          |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | Family of the library (category conveyor); its parameters form the attribute table (inherited ✔ / overridden ✖), its presets are the family's types; ports OP and LINE (in and out); placed from the Project Browser, listed in the Simulation tree and in schedules |
| Discrete simulation | Lift station cycle ([[ADR-0013 Lift Station 2D Cycle]])                                                                                                                                                                                                              |
| Physics simulation  | Kinematic platform following the trapezoidal height profile                                                                                                                                                                                                          |

The behaviour above does not change. **Rust status:** not ported yet (library R1, 2D R3, 3D R6 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.
