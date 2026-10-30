---
title: S14 Pallet Elevator
lang: en
tags: [spec, conveyor, elevator]
library-id: conveyor.elevator
status: spec
updated: 2026-10-30
---

# S14 Pallet Elevator (`conveyor.elevator`)

A carriage with a roller or chain deck that runs on a mast and **moves a pallet between two levels over large heights**, for example from the floor to a mezzanine. In the most common layout the pallet **enters and leaves on the same side** (C shape): it comes in at one height, travels up or down, and runs back out at the other height. Leaving on the opposite side (Z shape) is also possible. Added by [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]]; 2D cycle in [[ADR-0013 Lift Station 2D Cycle]].

Values are **indicative**, not manufacturer data.

## 1. Parameters

| Key                     | Unit | Default | Range           | Description                                                                              |
| ----------------------- | ---- | ------- | --------------- | ---------------------------------------------------------------------------------------- |
| `deckType`              | enum | rollers | rollers, chains | Carriage deck; rollers carry EUR long side first, chains short side                      |
| `length`                | m    | 1.6     | 1.0 … 3.0       | Deck length along travel                                                                 |
| `usableWidth`           | m    | 1.0     | 0.5 … 1.8       | Clear width on the deck (chains with transverse EUR: 1.3)                                |
| `palletSideAlongTravel` | enum | long    | long, short     | Pallet side along the deck's travel                                                      |
| `rollerDiameter`        | m    | 0.089   | 0.076, 0.089    | Rollers deck only; list of [[S01 Roller Conveyor]]                                       |
| `rollerPitch`           | m    | 0.2063  | 0.133, 0.2063   | Rollers deck only                                                                        |
| `nominalSpeed`          | m/s  | 0.25    | 0.05 … 0.6      | Deck speed while a pallet moves on or off                                                |
| `accel`                 | m/s² | 0.5     | 0.05 … 2        | Start and stop ramp of the deck                                                          |
| `entryHeight`           | m    | 0.6     | 0.2 … 12        | Deck height where the pallet enters                                                      |
| `exitHeight`            | m    | 4.6     | 0.2 … 12        | Deck height where the pallet leaves (default: mezzanine floor at 4.0 m + 0.6 m conveyor) |
| `verticalSpeed`         | m/s  | 0.5     | 0.1 … 2.0       | Carriage speed, up and down                                                              |
| `verticalAccel`         | m/s² | 0.5     | 0.1 … 2.0       | Carriage start and stop                                                                  |

**Ports:**

- `IN`: at x = 0, height `entryHeight`, facing −x.
- `OUT_NEAR`: at x = 0, height `exitHeight`, facing −x. Same side as `IN` (C shape, the common case).
- `OUT_FAR`: at x = `length`, height `exitHeight`, facing +x. Opposite side (Z shape).

**Which exit is used:**

- The elevator uses the exit port that is connected; no parameter is needed.
- If both are connected, it warns (`twoExits`) and uses `OUT_NEAR`.
- `IN` and `OUT_NEAR` share the same position seen from above. Connections are matched by height ([[ADR-0013 Lift Station 2D Cycle]] §7).
- The conveyor at the upper level is placed on a raised base (mezzanine).

**Derived with the defaults:**

- Stroke: **4.0 m**.
- Up or down: 4.0 / 0.5 + 0.5 / 0.5 = **9 s** ([[T04 Conveyor Kinematics]] §2).
- Rollers under a 1200 mm side: **5** (≥ 3 ✓).

Presets: **Rollers, EUR long side** (defaults) and **Chains, EUR transverse** (`deckType = chains`, `palletSideAlongTravel = short`, `usableWidth` 1.3 m).

## 2. 2D behaviour

One pallet at a time; rest at `entryHeight`.

1. **Receive.** The pallet enters through `IN`, travelling +x, and stops centred.
2. **Move** to `exitHeight` (up or down, `verticalSpeed`, `verticalAccel`).
3. **Discharge.**
   - Through `OUT_NEAR`: the deck runs backwards (−x). The pallet **does not turn**; its side along travel stays the same and its former tail now leads.
   - Through `OUT_FAR`: the deck keeps running forwards (+x).
   - If the next object cannot take the pallet, the elevator is **blocked**.
4. **Return** empty to `entryHeight`; then idle.

## 3. Statistics

As [[S12 Manual Lift Table]] §3: `utilization`, `blockedShare`, `idleShare`, `palletsMoved`, `meanCycleTime`.

## 4. Validations

| Rule                     | When                                                     |
| ------------------------ | -------------------------------------------------------- |
| `noExit`                 | Neither `OUT_NEAR` nor `OUT_FAR` is connected            |
| `twoExits`               | Both exits are connected; `OUT_NEAR` is used             |
| `noHeightChange`         | `entryHeight` and `exitHeight` differ by less than 10 mm |
| `deckTooShort`           | `length` below the pallet side along travel + 0.1 m      |
| `usableWidthFitsPallet`  | As [[S01 Roller Conveyor]]                               |
| `min3RollersUnderPallet` | Rollers deck only, as S01                                |

## 5. Not included yet

- Three or more levels, entry from both ends, and several exits chosen by rule.
- Waiting at the exit level, or being called by a signal.
- A carriage that carries two pallets, and landing gates.
- 3D physics (`phys.lift_station`), which arrives with spike S1.

Variants and real speeds: [[Open Questions]] Q15 and Q17.

## 6. Reference tests

- **T9:** 0.6 → 4.6 m at 0.5 m/s and 0.5 m/s² → **9 s** up and **9 s** back down empty (error < 1 %).
- C exit:
  - the pallet leaves through `OUT_NEAR` travelling −x, with the same side along travel;
  - the conveyor at the upper level, on the same side as the entry, receives it.
- Z exit through `OUT_FAR`; a down move (entry above exit); `twoExits` and `noExit`.
- Blocked discharge holds the carriage at the exit height; same seed gives the same result.
