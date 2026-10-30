---
title: S15 Gravity Roller Line
lang: en
tags: [spec, conveyor, gravity, accumulation]
library-id: conveyor.gravity
status: implemented (2D)
updated: 2026-10-30
---

# S15 Gravity Roller Line (`conveyor.gravity`)

A roller line **without motors**, laid on a slope. Pallets roll down by gravity. **Brake rollers**, one every N rollers, hold them at a controlled speed. At the low end, a **separator** and an **end stop** keep the first pallet free of the queue's pressure, so an **operator with a pallet jack** can take it. The line works as a buffer: it fills when the operator is slower than the supply. Added by [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]]; 2D model in [[ADR-0014 Gravity Line 2D Model]].

Values are **indicative**, not manufacturer data.

## 1. Parameters

| Key                     | Unit  | Default | Range         | Description                                                                          |
| ----------------------- | ----- | ------- | ------------- | ------------------------------------------------------------------------------------ |
| `length`                | m     | 6.0     | 2 … 30        | Length of the line (plan view)                                                       |
| `usableWidth`           | m     | 1.0     | 0.5 … 1.8     | Clear width between frames                                                           |
| `palletSideAlongTravel` | enum  | long    | long, short   | Long on rollers (EUR 1200 mm along travel)                                           |
| `slope`                 | ratio | 0.04    | 0.01 … 0.08   | Fall per metre; shown in % (4 %)                                                     |
| `exitHeight`            | m     | 0.10    | 0.05 … 1.5    | Roller top at the pick-up end (pallet-jack level)                                    |
| `entryHeight`           | m     | derived | —             | `exitHeight` + `slope` × `length` = 0.34 m                                           |
| `rollerDiameter`        | m     | 0.089   | 0.076, 0.089  | List of [[S01 Roller Conveyor]]                                                      |
| `rollerPitch`           | m     | 0.2063  | 0.133, 0.2063 | List of S01                                                                          |
| `brakeEvery`            | —     | 4       | 1 … 20        | One brake roller every N rollers                                                     |
| `brakeSpeed`            | m/s   | 0.2     | 0.05 … 0.4    | Speed the brake rollers hold                                                         |
| `rollingResistance`     | —     | 0.02    | 0 … 0.1       | Rolling loss of the idle rollers ($C_{rr}$)                                          |
| `startDelay`            | s     | 1.0     | 0 … 10        | Delay between the start of a pallet and the start of the one behind it               |
| `pickupLength`          | m     | 1.3     | 0.9 … 2.5     | Pick-up position: from the separator to the end stop                                 |
| `separatorTime`         | s     | 3       | 0 … 30        | Delay from the pick-up position emptying to the separator letting the next pallet go |

**Ports:**

- `IN`: at x = 0, height `entryHeight`, facing −x.
- `OUT`: at x = `length`, height `exitHeight`, facing +x.

**Operator.** Connect a **sink** with `removalMode = afterTime` to `OUT`; its `removalTime` is the operator's time per pallet ([[S09 Source and Sink]]).

**Derived with the defaults:**

- Entry height: 0.1 + 0.04 × 6 = **0.34 m**.
- Start acceleration: $a = g\,(s - C_{rr})$ = 9.81 × (0.04 − 0.02) = **0.196 m/s²**. Distance to reach brake speed: 0.2² / (2 × 0.196) = **0.10 m**.
- Brake spacing: 4 × 206.3 mm = **825 mm** (≤ 1200 mm ✓, at least one brake under each pallet).
- Separator position: 6.0 − 1.3 = **4.7 m**.
- Capacity with EUR long side: 1 in the pick-up position + ⌊4.7 / 1.2⌋ = 3 in the queue = **4 pallets**.

## 2. 2D behaviour

Each pallet moves on its own ([[ADR-0014 Gravity Line 2D Model]]).

1. **Entry.**
   - The line accepts a pallet only when a whole pallet fits behind the last one.
   - The feeding object pushes it in at its own speed. On the line it continues at min(that speed, `brakeSpeed`).
   - While the line is full, the feeding conveyor is **blocked**.
2. **Rolling.**
   - From rest, a pallet accelerates with $a$ up to `brakeSpeed`, then keeps that speed.
   - It stops when it touches the pallet ahead, the closed separator or the end stop.
3. **Queue start.** When a pallet starts moving, the pallet that was stopped behind it starts `startDelay` seconds later.
   - 0: the queue moves as one block.
   - Larger values: the pallets spread out, as the brakes do.
4. **Separator.**
   - While the pick-up position is empty, the separator is open.
   - It closes behind the pallet that enters the pick-up position, and holds the next one.
   - When the operator takes the pallet at the end stop, the separator opens after `separatorTime`, and the next pallet rolls to the end stop.
5. **Pick-up.** When the pallet at the end stop is at rest and the operator (sink) is free, the operator takes it.

## 3. Statistics

| Key                | Meaning                                                              |
| ------------------ | -------------------------------------------------------------------- |
| `palletsDelivered` | Pallets taken by the operator                                        |
| `meanQueue`        | Time-average number of pallets on the line                           |
| `maxQueue`         | Largest number of pallets on the line at the same time               |
| `fullShare`        | Share of time the entry cannot take a pallet (the feeder is blocked) |
| `pickupWaitShare`  | Share of time a pallet waits at the end stop for the operator        |
| `meanTransitTime`  | Mean time from entering the line to being taken (s)                  |

## 4. Validations

| Rule                     | When                                                                                         |
| ------------------------ | -------------------------------------------------------------------------------------------- |
| `slopeTooLow`            | `slope` ≤ `rollingResistance`: a pallet at rest does not start                               |
| `brakeSpacingTooLong`    | `brakeEvery` × `rollerPitch` > pallet side along travel: a pallet can have no brake under it |
| `pickupTooShort`         | `pickupLength` below the pallet side along travel + 0.05 m                                   |
| `noExit`                 | `OUT` is not connected                                                                       |
| `exitNotSink`            | `OUT` is connected to something other than a sink (not supported yet)                        |
| `usableWidthFitsPallet`  | As [[S01 Roller Conveyor]]                                                                   |
| `min3RollersUnderPallet` | As S01                                                                                       |

## 5. Not included yet

- Signals (`pickupOccupied`, `full`) and photocells mounted on the line.
- Brake behaviour that depends on pallet mass, and impact at the end stop. Both belong to the 3D mode.
- Handing pallets to a conveyor at the end.
- 3D physics (`phys.gravity`, brake rollers on a slope), which arrives with spike S1.

Real slope, brake spacing, brake speed, separator delay and queue behaviour: [[Open Questions]] Q16.

## 6. Reference tests

- **T10:** one pallet, defaults, entering at 0.2 m/s → (6.0 − 1.2) / 0.2 = **24 s** from fully on the line to the end stop (error < 1 %). Measured: 24.000 s (scenario T10).
- **T11:** saturated supply, operator `removalTime` 60 s → **60 pallets/h**; the line fills to 4 pallets and the feeder is blocked. The separator cycle is 3 s + 7.01 s from rest over 1.3 m = 10.01 s, shorter than the operator, so the operator sets the rate. Measured: 60.000 pallets/h, largest queue 4 (scenario T11).
- `startDelay` = 0: after a release the whole queue starts at the same instant. `startDelay` = 2 s: gaps of 0.4 m open while rolling and close again at the separator.
- Separator: the second pallet stops at 4.7 m while the first is in the pick-up position.
- Validations: `slopeTooLow`, `brakeSpacingTooLong`, `exitNotSink`. Same seed gives the same result.
