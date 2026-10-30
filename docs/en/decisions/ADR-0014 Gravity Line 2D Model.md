---
title: ADR-0014 Gravity Line 2D Model
lang: en
tags: [decision, des, gravity, accumulation]
status: proposed
date: 2026-10-30
---

# ADR-0014 Gravity Line 2D Model

- **Status:** Proposed. Waiting for the owner to confirm [[S15 Gravity Roller Line]].
- **Date:** 2026-10-30

## Context

The current 2D conveyors move all their pallets as one unit ([[ADR-0007 2D Flow Model Without Accumulation]]). A gravity line works the other way:

- each pallet rolls down on its own;
- the brake rollers hold it at a controlled speed;
- it stops when it touches the pallet ahead, the separator or the end stop.

The owner is not sure whether a queue starts moving as one block or spreads out because of the brakes, and asked for a parameter.

## Decision

1. **Pallets move one by one.**
   - A new class, `GravityLine`, keeps a position, a speed and a state for each pallet: accelerating, at brake speed, or stopped.
   - It predicts its own next event: a contact, a pallet reaching the separator or the end stop, the entry becoming free, or a delayed start. It schedules that event as `gravity.motion`.
   - Any change cancels the prediction and computes it again ([[T02 Discrete-Event Simulation]]).
   - The existing conveyors are not changed.
2. **Speed.**
   - From rest, a pallet accelerates with $a = g\,(s - C_{rr})$, where $s$ is the slope as a ratio (small-angle form) and $C_{rr}$ is the rolling resistance. At 4 % and 0.02: $a$ = 0.196 m/s².
   - Its speed is capped at `brakeSpeed` by the brake rollers.
   - There is no braking before a contact: the pallet stops when it touches what is ahead. With brake speeds around 0.2 m/s that is what happens on a real line. The impact energy is left to the 3D mode.
3. **Queue start.** When a stopped pallet starts, the pallet behind it starts `startDelay` seconds later, if it is still blocked only by that pallet.
   - `startDelay` = 0: the queue moves as one block.
   - Larger values: gaps of about `brakeSpeed` × `startDelay` open between pallets.
4. **Separator.** It works as a time-delay separator at $x_S$ = `length` − `pickupLength`.
   - While the pick-up position (from $x_S$ to the end stop) is empty, the separator is open and pallets roll through.
   - When a pallet's front passes $x_S$, the position is taken. The separator closes as soon as that pallet's tail has passed. A pallet that reaches $x_S$ while the separator is closed stops there.
   - When the operator takes the pallet at the end stop, the position is free again. The separator opens after `separatorTime`, and the first waiting pallet rolls down to the end stop. The others follow, each `startDelay` after the one ahead of it.
5. **Entry.**
   - The line accepts a pallet only when a whole pallet fits behind the last one.
   - The feeding object pushes the pallet in at its own speed, the same way it hands a pallet to a sink. After that the pallet moves at the gravity speed, starting from min(entry speed, `brakeSpeed`).
   - While the line is full, the feeding conveyor is blocked.
6. **Exit.**
   - The sink connected to `OUT` is the operator with the pallet jack.
   - When the pallet at the end stop is at rest and the sink accepts, the pallet is handed over at once. The sink's `removalTime` is the operator's time per pallet.
   - Connecting a conveyor to `OUT` is not supported yet (warning).

## Alternatives considered

- **A chain of one-pallet slots** reusing the existing conveyor model. Rejected: a queue moves forward one pallet at a time, and each pallet waits until the one ahead has left its slot (about 6 s per position at 0.2 m/s). That is not how a gravity line behaves, and it cannot give the `startDelay` = 0 case.
- **A plain FIFO with a fixed transit time.** Rejected: no pallet positions for sensors, for the 3D view or for the separator.

## Consequences

- Tests (`gravity-line.test.ts`):
  - **T10:** single pallet, 6 m, EUR long side, 0.2 m/s → 24 s to the end stop after entering.
  - **T11:** saturated line, operator 60 s per pallet → 60 pallets/h.
  - `startDelay` 0 compared with > 0; the separator cycle; entry blocked when full; determinism.
- The per-pallet model can later carry `pressure` accumulation on roller conveyors ([[S01 Roller Conveyor]] §3).
- 3D physics (`phys.gravity`, real brake rollers on a slope) waits for spike S1.
