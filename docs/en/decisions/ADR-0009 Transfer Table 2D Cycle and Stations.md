---
title: ADR-0009 Transfer Table 2D Cycle and Stations
lang: en
tags: [decision, des, transfer]
status: proposed
date: 2026-10-30
---

# ADR-0009 Transfer Table 2D Cycle and Stations

- **Status:** Proposed. It was implemented while the project owner was away. It follows the "interpretation to confirm" in [[S05 Roller-to-Chain Transfer]] and [[Open Questions]] Q6.
- **Date:** 2026-10-30

## Context

The roller-to-chain transfer was the last inert conveyor in the 2D mode. Like the turntable, it has conveying surfaces and timed steps (lift, lower). Handling each such object separately inside `FlowSimulation` would duplicate the logic.

## Decision

1. **Stations.** Turntables and transfers implement one interface, `Station` (`flow/station.ts`).
   - A station has one or two **decks**. A deck is a short conveyor that uses the same 2D model as the lines ([[ADR-0007 2D Flow Model Without Accumulation]]).
   - A station also has timed steps: `prepareFor` (turn or lift before receiving), `step` (start the next step when its condition holds) and `finishTimedStep`.
   - `FlowSimulation` drives all stations the same way. It uses one event type, `station.done`.
2. **Transfer decks.**
   - The rollers run along x from A to B, at `nominalSpeed` / `accel`.
   - The chains run across between C1 (−z) and C2 (+z), at `transferSpeed` / `transferAccel`. Their length is `usableWidth`; the spec's `transferDistance` "auto ≈ table width" is not a stored parameter yet.
   - The pallet changes direction **without turning**, so its side along travel swaps (EUR: long on the rollers, short on the chains).
3. **Cycles.**
   - **From A** (`operationMode` rollersToChains or bidirectional): the pallet is received on the rollers and stops centred (`requiresStop`). Then:
     1. lift (`liftTime`);
     2. the chains carry it out through C2 (`transferDirection` right), C1 (left) or the connected sides in turn (bidirectional);
     3. once it has left the chains, lower (`lowerTime`);
     4. idle.
   - **From C1/C2** (chainsToRollers or bidirectional): the chains must be up, so an idle table lifts first. Then the pallet is received on the chains and centred, the chains lower, and the rollers carry it out through B.
   - The chains rest **down**. The roller section takes nothing while the chains work (S05 interlock).
   - "Right" means +z, the right-hand side when looking along A → B from above.
4. **Warnings:** `noExit` when the chain side (or B) needed by the operation mode is not connected.
5. **Viewer:** pallets on the chains are drawn `liftStroke` higher (`pathY` in the snapshot).

## Consequences

- Tests in `transfer.test.ts`:
  - rollers → chains with the orientation swap;
  - lift and lowering times add to the cycle;
  - one pallet per cycle;
  - unconnected side;
  - bidirectional in turn;
  - chains → rollers.
- `reference-all-five.intracraft.json` runs pallets through all five conveyors. CLI scenario `REF-5`: 120 pallets/h get through when the source sends one every 30 s.
- The transfer's 3D model does not yet show the chains rising during a run.
