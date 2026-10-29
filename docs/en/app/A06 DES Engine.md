---
title: A06 DES Engine
lang: en
tags: [app, des, sim-des]
---

🌐 **EN** · [[A06 Motor discreto|ES]] · [[A06 DES-Engine|DE]] · [[Home (EN)|Index]]

# A06 DES Engine (`packages/sim-des`)

Theory: [[T02 Discrete-Event Simulation]], [[T03 Time, Event Ordering and Determinism]], [[T04 Conveyor Kinematics]], [[T05 Accumulation, Capacity and Throughput]].

## 1. What exists today

| Module           | Content                                                                                                                          | Tests                                  |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| `event-queue.ts` | `EventQueue<T>`: binary min-heap ordered by (time, priority, seq); integer µs only                                               | Ordering, ties, non-integer rejection  |
| `simulator.ts`   | `DesSimulator`: clock, `on(type, handler)`, `schedule(delay, event, priority)`, `cancel(ticket)`, `runUntil(t)`, event `trace`   | Clock jumps, cancellation, determinism |
| `kinematics.ts`  | `profileTime`, `transferCycleTime`, `maxThroughput`, `palletCapacity`, `impactEnergy`, `positionAt`, `velocityAt`, `timeToReach` | T1–T5, triangular case, segments       |

## 2. Planned design (phase 3)

### Tracks

Each conveyor is a **track** with an ordered list of pallets (front first). A pallet on a track stores:

- its current kinematic segment `{t0, s0, v0, a}`;
- its length along travel (depends on orientation);
- a **version** number, incremented whenever its motion changes, to invalidate stale predictions.

### Prediction cycle

After any event touching a pallet:

1. Close the current segment at `now` and open a new one (new speed/acceleration).
2. Compute candidate crossings with `timeToReach`: end of ramp, next sensor edge, stop, end of zone, leader's tail − min gap, end of track.
3. Schedule only the **earliest**; cancel the previous prediction.

### Event types

`pallet_enters`, `ramp_ends`, `front_reaches_stop`, `follower_reaches_leader`, `tail_clears_entry`, `sensor_edge`, `zone_released`, `turn_completed`, `lift_completed`, `transfer_completed`, `signal_changed`, `rule_fired`.

### Hand-over between tracks

At `front_reaches_end` the pallet is registered on the downstream track (if it can accept it); while straddling, its speed follows the hand-over rule ([[T04 Conveyor Kinematics]] §9). When the tail clears the upstream track (`tail_clears_entry` of the next / exit of the previous), it is removed from the upstream list.

### Statistics

Per object: time per state, counters, tallies (transit times) using integer-µs time-weighted accumulators ([[T08 Statistics and Output Analysis]]).

## 3. Performance target

≥ 1000× real time with 50 objects and 30 pallets. Each event is $O(\log n)$ in the heap plus constant work per affected pallet; the viewer evaluates `positionAt` per frame, not per event.

## 4. Rules for contributors

- No `Math.random()`, no `Date.now()`, no DOM, no React (ESLint enforced).
- All times entering the queue are integers (µs); convert with `secondsToSimTime` once.
- Every new event type gets a test that checks its time against an analytic formula.

## Related

[[A02 Architecture]] · [[A08 Controller]] · [[A14 API Reference]]
