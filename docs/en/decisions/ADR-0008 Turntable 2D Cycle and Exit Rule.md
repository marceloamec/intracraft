---
title: ADR-0008 Turntable 2D Cycle and Exit Rule
lang: en
tags: [decision, des, turntable]
status: proposed
date: 2026-10-30
---

# ADR-0008 Turntable 2D Cycle and Exit Rule

- **Status:** Proposed. It was implemented while the project owner was away; the owner must confirm the exit rule ([[Open Questions]] Q11).
- **Date:** 2026-10-30

## Context

The chain and roller turntables ([[S03 Chain Turntable]], [[S04 Roller Turntable]]) were inert in the 2D mode. S03 §3 gives the sequence: waiting → receiving → centred → turning → settling → discharging. The specifications do not say **which station a pallet leaves through**. In the long run, control rules decide that ([[A08 Controller]]); the controller does not exist yet.

## Decision

1. **The deck is a short conveyor** of `platformLength`, implemented with the same 2D line model as the conveyors ([[ADR-0007 2D Flow Model Without Accumulation]]):
   - its speed is `nominalSpeed` (chains) or `loadSpeed` (rollers);
   - `accel` is used for both ramps;
   - it moves together with the feeding conveyor while a pallet crosses.
2. **Cycle.**
   - **Positioning.** A pallet waiting at a station of an idle, empty platform that is not lined up with it makes the platform turn there first.
   - **Receiving.** The pallet stops with its centre on the platform centre (`requiresCenteredPallet`).
   - **Turning.** The platform turns to the exit station along the shortest way: at most 90°, because the deck conveys in both directions. A turn lasts the angular profile (T3: 6.5 s for 90°) plus `settleTime`. If the exit is on the same line as the entry, there is no turn.
   - **Discharging.** The pallet leaves as from a conveyor. The platform stays where it is until the next request.
3. **Exit rule.** A new parameter `exitRule` (group *routing*, 2D only) applies until control rules exist:
   - `alternate` (default) takes the connected exits in turn;
   - `firstExit` always takes the first one, counting clockwise from the entry as seen from above;
   - `random` chooses with equal chances, using the seeded generator of the object.

   Exits are station ports connected to the entry of a conveyor or a sink and listed in `stations`; the entry station is never an exit.
4. **Not covered yet:**
   - a turntable passing directly to another turntable (warning `turntableToTurntable`);
   - `pivotOffset`, `angularRange`, `turnMode = free`, `angularPrecision`;
   - choosing an exit by whether it is free.
5. **Viewer.**
   - Snapshots carry each turntable's deck heading (`angles`) and, for every pallet, the start and heading of its carrier's path.
   - The editor rotates the platform and the pallet on it during the turn.

## Alternatives considered

- **Always the first exit:** too restrictive for a splitter. It is kept as an option.
- **First free exit:** it needs a look-ahead that the controller will do better. Postponed to A08.
- **Turning with the pallet off-centre:** S03 requires a centred pallet by default.

## Consequences

- Tests in `turntable.test.ts`:
  - straight through;
  - a 90° corner adds exactly 6.8 s;
  - turning back for the next pallet;
  - the pallet turns with the platform;
  - alternate and first exit;
  - seeded random choice;
  - the no-exit warning.
- Example file `packages/scenarios/scenarios/layouts/demo-turntable-split.intracraft.json`.
- Phases 6 and 7 of [[A12 Roadmap]] are done in 2D; the physics part remains.
