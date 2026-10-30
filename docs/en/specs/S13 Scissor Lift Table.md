---
title: S13 Scissor Lift Table
lang: en
tags: [spec, conveyor, lift]
library-id: conveyor.scissor_lift
status: spec
updated: 2026-10-30
---

# S13 Scissor Lift Table (`conveyor.scissor_lift`)

A roller deck on a scissor lift that **joins two roller conveyors at different heights**. The pallet enters at one end at the entry height, the table lifts (or lowers) it, and the pallet leaves at the opposite end at the exit height. Added by [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]]; 2D cycle in [[ADR-0013 Lift Station 2D Cycle]]. For large height differences use the [[S14 Pallet Elevator]].

Values are **indicative**, not manufacturer data.

## 1. Parameters

| Key                     | Unit | Default | Range         | Description                                                         |
| ----------------------- | ---- | ------- | ------------- | ------------------------------------------------------------------- |
| `length`                | m    | 1.6     | 1.3 … 2.5     | Deck length; at least pallet length + margins                       |
| `usableWidth`           | m    | 1.0     | 0.5 … 1.8     | Clear width between frames                                          |
| `palletSideAlongTravel` | enum | long    | long, short   | Long on rollers (EUR 1200 mm along travel)                          |
| `rollerDiameter`        | m    | 0.089   | 0.076, 0.089  | Same list as [[S01 Roller Conveyor]]                                |
| `rollerPitch`           | m    | 0.2063  | 0.133, 0.2063 | Same list as S01                                                    |
| `nominalSpeed`          | m/s  | 0.25    | 0.05 … 0.6    | Roller speed while a pallet moves on or off                         |
| `accel`                 | m/s² | 0.5     | 0.05 … 2      | Start and stop ramp of the rollers                                  |
| `entryHeight`           | m    | 0.6     | 0.2 … 3.0     | Deck height at the entry end; match the conveyor                    |
| `exitHeight`            | m    | 1.2     | 0.2 … 3.0     | Deck height at the exit end; lower than entry = the table goes down |
| `liftSpeed`             | m/s  | 0.05    | 0.01 … 0.3    | Up speed                                                            |
| `lowerSpeed`            | m/s  | 0.07    | 0.01 … 0.3    | Down speed                                                          |
| `verticalAccel`         | m/s² | 0.2     | 0.05 … 2      | Start and stop of the vertical move (advanced)                      |

**Ports:**

- `IN`: at x = 0, height `entryHeight`, facing −x.
- `OUT`: at x = `length`, height `exitHeight`, facing +x.

**Derived with the defaults:**

- Stroke: **0.6 m**.
- Up with a pallet: 0.6 / 0.05 + 0.05 / 0.2 = **12.25 s**.
- Back down empty: 0.6 / 0.07 + 0.07 / 0.2 = **8.92 s** ([[T04 Conveyor Kinematics]] §2).

## 2. 2D behaviour

One pallet at a time; rest at `entryHeight`.

1. **Receive.** The pallet enters through `IN` and stops centred.
2. **Move** to `exitHeight`: up at `liftSpeed` or down at `lowerSpeed`.
3. **Discharge** through `OUT`. If the next conveyor cannot take the pallet, the table is **blocked**.
4. **Return** empty to `entryHeight`; then idle.

## 3. Statistics

As [[S12 Manual Lift Table]] §3: `utilization`, `blockedShare`, `idleShare`, `palletsMoved`, `meanCycleTime`.

## 4. Validations

| Rule                     | When                                                                             |
| ------------------------ | -------------------------------------------------------------------------------- |
| `noExit`                 | `OUT` is not connected                                                           |
| `noHeightChange`         | `entryHeight` and `exitHeight` differ by less than 10 mm (use a roller conveyor) |
| `deckTooShort`           | `length` below the pallet side along travel + 0.1 m                              |
| `min3RollersUnderPallet` | As [[S01 Roller Conveyor]]                                                       |

## 5. Not included yet

- Signals, and a table that waits at the exit height.
- Entry and exit at the same end.
- Double scissors, and a stroke limit that depends on the scissor size.
- 3D physics (`phys.lift_station`), which arrives with spike S1.

Real lift speeds: [[Open Questions]] Q15.

## 6. Reference tests

- **T8:** 0.6 → 1.2 m with `liftSpeed` 0.05 m/s and `verticalAccel` 0.2 m/s² → **12.25 s** of lifting; return 8.92 s (error < 1 %).
- Down table (entry 1.2, exit 0.6): the move with the pallet uses `lowerSpeed` (8.92 s) and the return uses `liftSpeed` (12.25 s).
- Blocked discharge holds the table at the exit height; one pallet per cycle; same seed gives the same result.
