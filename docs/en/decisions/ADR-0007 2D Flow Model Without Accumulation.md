---
title: ADR-0007 2D Flow Model Without Accumulation
lang: en
tags: [decision, des, flow]
status: proposed
date: 2026-10-30
---

# ADR-0007 2D Flow Model Without Accumulation

- **Status:** Proposed. It was implemented while the project owner was away; the owner must confirm points 3–5.
- **Date:** 2026-10-30

## Context

Phase 3 needs pallets moving on the drawn layout in the 2D (discrete-event) mode: source → conveyors → sink, with throughput, blocking and statistics. The default behaviour is **no accumulation** ([[S01 Roller Conveyor]] §3): all pallets on a conveyor move with it. Questions Q2 and Q3 in [[Open Questions]] are still open, so this ADR states the rules that were implemented.

## Decision

1. **Where it lives.**
   - `packages/sim-des/src/flow/` holds the flow model.
   - `sim-des` now depends on `library`, to read definitions, parameters and pallet types. This is the dependency foreseen in [[A02 Architecture]] §4.
   - `buildFlowNetwork(layout, registry)` turns the layout into nodes; `FlowSimulation` runs them.
2. **Nodes by `behaviors.des`:**
   - `des.roller_conveyor` and `des.chain_conveyor` are *lines* (straight, forward, no accumulation);
   - `des.source` and `des.sink` (new definitions `flow.source`, `flow.sink`, [[S09 Source and Sink]]);
   - every other component is *inert*: it never accepts a pallet, and a warning says its 2D behaviour is missing (turntables and transfer until phases 6–8).
3. **Conveyors linked by a crossing pallet move as one unit.**
   - While a pallet straddles conveyor A and the next conveyor B, A and B form a group with one shared motion. The group is a chain if several such pallets link several conveyors.
   - The group uses the lowest nominal speed, acceleration and deceleration of its members (Q3, option 1 "the slower one").
   - When the groups join, the faster side changes speed instantly. In 2D this only happens when the speeds differ.
4. **Decision point and stop at the end (Q2).**
   - A conveyor's front pallet asks the next object to accept it at L − v²/(2·b), where v and b are the group's speed and deceleration. This is the virtual end sensor.
   - If the next object refuses, the group brakes so that the pallet stops with its **front exactly at the end** of the conveyor. If the pallet is already closer to the end than the normal braking distance, a stronger deceleration is used.
   - This replaces the earlier assumption "sensor at the end, overrun v²/2b". With the defaults, the earlier rule left a 62.5 mm overrun onto the next conveyor.
5. **Acceptance rules.**
   - A conveyor accepts a pallet when all of these hold:
     - nothing else is entering it;
     - it is not held (Q2c: no);
     - its last pallet has completely left the entry;
     - the incoming pallet can get completely aboard before the conveyor's own leading pallet reaches the end.

     The last condition means a conveyor never stops with a pallet half on it.
   - The source needs pallet length + `minGap` free behind the last pallet. This gives T2: 0.25/1.35 pallets/s.
   - A waiting source counts its "lost" time, and its next interval starts when the pallet is placed. With "do not wait", the pallet is rejected and counted.
   - A sink with a removal time takes one pallet at a time and is busy for that time after each pallet.
6. **The gap between ports** (20 mm) is part of the path. A pallet moves to the next conveyor's list when its tail clears the end of the previous one.
7. **The editor runs the model in a Web Worker** (`apps/editor/src/sim/`).
   - The worker posts frames at about 30 Hz: snapshot, statistics every 8 frames, and new warnings.
   - The viewer extrapolates pallet positions between frames at ×1 and ×10.
   - Any layout edit stops and resets the run.
   - "Max" speed is limited to one simulated hour per frame.

## Alternatives considered

- **Independent conveyors with min speed only for the crossing pallet.** Followers on the faster conveyor would catch up with the crossing pallet, which needs collision handling. Rejected.
- **Overrun past the end (literal reading of the Q2 assumption).** The pallet would end up on the next conveyor without being accepted. Rejected.
- **Fixed source schedule with a backlog.** It measures queueing better, but [[S09 Source and Sink]] speaks of "lost time" when waiting. Kept as a possible later option.

## Consequences

- New tests: `flow-simulation.test.ts` covers T1 (12.25 s), T2 (≈ 667/h), hand-over, blocking, rejection, slower speed while crossing, a busy sink, determinism and invariants over a 4 h run. `sim-runner.test.ts` covers the run clock.
- Q2 and Q3 in [[Open Questions]] now point here; the owner's answers may change points 3–5.
- **Next:** ZPA and pressure accumulation, the photocell (T6), turntables and the transfer in 2D, and pallet orientation changes.
