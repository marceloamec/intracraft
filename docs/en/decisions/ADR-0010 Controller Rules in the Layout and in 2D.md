---
title: ADR-0010 Controller Rules in the Layout and in 2D
lang: en
tags: [decision, control, des]
status: proposed
date: 2026-10-30
---

# ADR-0010 Controller Rules in the Layout and in 2D

- **Status:** Proposed. It was implemented while the project owner was away; the owner should review points 2–4.
- **Date:** 2026-10-30

## Context

[[A08 Controller]] had a pure If/Then evaluator (`packages/control`), but no place to store rules and no connection to the simulation. Phase 3 of [[A12 Roadmap]] asks for a basic controller, so that sensors can act on conveyors.

## Decision

1. **Rules live in the layout.**
   - `layout.rules` is optional, so older files stay valid without a migration.
   - It has the same shape as `control`'s `Rule`.
   - Because the rules are part of the layout, they take part in undo/redo, `layout.load`, saving and opening.
   - One new command, `control.setRules`, replaces the list. It rejects duplicate ids and only warns about signals of objects that are not in the layout.
2. **Signals in 2D** (`<instance id>.<signal>`):

| Object                     | Inputs (written by rules)                                 | Outputs (read by rules)                                     |
| -------------------------- | --------------------------------------------------------- | ----------------------------------------------------------- |
| Roller and chain conveyors | `run`: true by default; false brakes the conveyor to rest | `running`, `occupied`                                       |
| Source                     | `enable`: true by default; false holds waiting pallets    | —                                                           |
| Photocell                  | —                                                         | `detected` (or `outputSignal`), inverted for `activeOnDark` |

   A conveyor whose `run` is false (state *stopped*):
   - takes no pallet from the one before;
   - stops its whole group when a pallet is crossing.
3. **Evaluation.**
   - After each event the plant settles. If any signal changed since the last evaluation, the rules are evaluated with the previous and current values (edges work).
   - Writes without delay apply at once and the plant settles again. Writes with a delay become `signal.write` events.
   - Delayed writes are **never cancelled**. This settles the open point "cancellation semantics" in A08 §3 provisionally.
   - More than 100 evaluations at one instant stop the run with an error ("the rules keep changing signals"). Such rules contradict each other.
4. **Statistics.** Time switched off counts as *stopped*. It is neither *working* nor *blocked*, so it does not count toward utilisation or blocked time.
5. **Editor.** The Controller panel lists the rules. Each rule has:
   - conditions: a signal plus *is on*, *is off*, *switches on* or *switches off*;
   - actions: an input signal, switch on or off, and a delay in ms.

   During a run, a dot shows each signal's value. The template *Stop at sensor* stops the conveyor the first mounted photocell sits on and restarts it after 5 s.

## Consequences

- `sim-des` depends on `control` ([[A02 Architecture]]).
- Tests (`control.test.ts`):
  - a 10 s stop makes the pallet arrive exactly 10 s later;
  - stopped state and signals in the snapshot;
  - a stopped conveyor takes nothing;
  - oscillating rules raise an error.

  Further tests: `set-rules` in the commands session tests and `signals.test.ts` in the editor.
- **Still to do:** inputs for turntables and transfers (`turnTo`, `lift`), the ZPA template ([[T05 Accumulation, Capacity and Throughput]]), and rule evaluation in 3D every 10 ms.
