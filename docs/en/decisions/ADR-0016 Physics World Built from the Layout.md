---
title: ADR-0016 Physics World Built from the Layout
lang: en
tags: [decision, physics, sim-physics, phase-4]
status: proposed
date: 2026-10-31
---

# ADR-0016 Physics World Built from the Layout

- **Status:** Proposed on 2026-10-31. Implemented in `packages/sim-physics/src/world/` while the owner was away ("autopilot" session); the owner should confirm points 3, 6 and 7.
- **Date:** 2026-10-31

## Context

Spike S1 closed with Jolt and chain plan B ([[ADR-0002 Physics Engine Candidate]]), but only the physics bench used the engine. The editor's "3D physics" mode still ran the 2D model. Phase 4 of [[A12 Roadmap]] asks for the world to be built from the layout, with kinematic rollers, the compound pallet, raycast sensors and 2D/3D consistency. The new objects of [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]] (S12–S15) had no 3D behaviour either.

## Decision

1. **Where it lives.**
   - `packages/sim-physics/src/world/` holds the 3D run: `PhysicsFlowSimulation` (world building and step loop), `Deck`, `Station3D`, `GravityLine3D`, `Sensor3D`, deck frames and pallet bodies.
   - `sim-physics` now depends on `sim-des`, `library`, `control`, `geometry` and `asset-catalog`.
   - The layout is read with the same `buildFlowNetwork` as the 2D mode. Arrivals (`PalletSource`), removal (`PalletSink`), sensor timing (`Photoeye`) and the controller rules are the 2D classes. **Only the motion differs**: in 3D the pallets move by friction, contact and impact.
2. **Step loop** ([[A07 Physics Engine Wrapper]] §4).
   - Fixed step 1/120 s; the controller runs on every 10 ms boundary.
   - Each step, the plant logic settles on the *measured* pallet positions: crossings complete, stations move on, sources place pallets, and front pallets at their decision point are accepted or make their deck hold.
   - Then the decks ramp their speed, the engine steps, the poses are read back and the photocells cast their beams.
3. **Conveyors without accumulation, as in 2D** ([[ADR-0007 2D Flow Model Without Accumulation]]).
   - The decision point, the stop with the front at the end, the acceptance rules and the groups of decks linked by a crossing pallet are the same.
   - They are evaluated on the bodies (front and rear projected on the deck axis), not on 2D positions.
   - An empty deck takes the speed of the deck that feeds it at once, as in 2D.
4. **Bodies per object.**

| Object                | Bodies                                                                                                                                     |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Pallet                | Dynamic MTBH ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]): the catalogue model of the type and a uniform load block; CCD on  |
| Roller conveyor       | One kinematic cylinder per roller, ω = v / r (S01 §4); material `steel-zinc`                                                               |
| Chain conveyor        | One static strand per chain with a surface velocity (plan B); positions from `chainLayout`; material `steel-chain`                         |
| Lift tables, elevator | One kinematic platform: rollers or strands as shapes, surface velocity along the deck, vertical velocity following the trapezoidal profile |
| Process station       | The same platform, without vertical motion                                                                                                 |
| Gravity line          | Kinematic rollers on the slope; a kinematic separator plate; a static end stop; material `steel-painted` for both                          |
| Sink                  | A static receiving bed (1.5 m) beyond the feeding deck, moving with it while it hands a pallet over                                        |
| Photocell             | A ray from sensor to reflector each step that only pallets interrupt; the edge is stamped at the step midpoint                             |
| Floor                 | A static concrete slab at y = 0                                                                                                            |

5. **Sources.**
   - A pallet appears with its rear at the entry end, aligned with travel, at rest **relative to the conveying surface**: it starts with the deck's surface velocity, as the 2D pallet does.
   - S09 says "at rest". With a stopped deck both readings are the same.
6. **Gravity line: free rollers follow their pallet.**
   - The engine has no hinge joints in our wrapper, so the free rollers are kinematic.
   - Each step a roller turns at the speed its pallet reaches rolling freely: $v + g\,(s - C_{rr})\,\Delta t$, capped at the brake speed. A pallet stopped in the queue holds its rollers; a pallet still pushed by the feeder turns them freely.
   - Contacts with the separator, the end stop and the other pallets stay physical, so the impact energy is measured: 12.6 J for 631 kg at 0.2 m/s.
   - The entry needs a pallet length **plus the gap** behind the last pallet. The 2D model leaves the 20 mm gap out; in 3D the entering pallet would hit the one ahead.
   - The separator plate rises only once the holding pallet's tail has passed the whole plate and nothing lies over it.
7. **Per-instance friction.** `rollerMu` and `chainMu` replace the pair table only when they differ from the definition default. With the default, the pair table decides ([[S16 Contact Materials]]).
8. **Not in 3D yet.** Turntables and the transfer table get the warning `behaviorNotAvailable3D`; pallets stop in front of them.
9. **Outputs.**
   - The snapshot is the 2D one plus `poses`: the real pose of every pallet.
   - Statistics use the 2D keys.
   - Two more outputs: `takeImpacts()` lists impacts against stops, separators, pallets or the floor, never the rollers that carry the pallet; `takeSensorEdges()` lists sensor edges with exact times. A pallet that drops 0.25 m below its deck counts as fallen (`palletFell`).

## Alternatives considered

- **Drive the 3D world from a running 2D model (co-simulation).** It is simpler and always consistent. But the 3D signals would not drive the controller, and slips or jams would not change the flow. Rejected: the 3D mode must stand on its own.
- **Dynamic free rollers with hinge joints.** Closer to the real line, but the wrapper exposes no joints yet, and about 30 more dynamic bodies per line slow the run. Kept for later if braking that depends on the pallet mass is needed.
- **Plan B (surface velocity) for roller conveyors too.** It is cheaper, but S01 asks for kinematic rollers, and the 3-roller rule and the board gaps only show with real cylinders.

## Consequences

- Tests (`physics-flow.test.ts`, 15 tests, about 4 s), compared with the 2D model:
  - **T1** on rollers and on chains: 12.25 s ±1 %.
  - Hand-over: 24.33 s.
  - Front at the open end: within ±5 mm (measured 0.5 mm).
  - A blocked line with two pallets per conveyor, without impacts.
  - A rule that stops a conveyor at a photocell.
  - **P6:** every 2D/3D photocell edge within ±50 ms.
  - Voids seen at the bottom-board height.
  - Determinism.
  - C elevator cycle: 30.03 s.
  - Manual input table.
  - Gravity line: impact ½·m·v² at the end stop, nothing falls.
  - Process station: 42.03 s.
  - Warning for turntables.
- Measured on the demo layouts, 3D compared with 2D:
  - sink cycle times differ by less than 0.1 % (elevator, process station, manual table);
  - the gravity line ends with the same queue to the millimetre and the same throughput.
- A 12 m line with ten pallets costs about 0.5 ms per step: about 15 × real time on a desktop.
- The comparison showed an ordering error in the 2D station statistics: the time of a timed step (processing, lifting) was counted as "blocked". Fixed in `FlowSimulation` (the time is added before the step ends); a new test checks that 30 s of processing in 120 s gives 25 % utilization.
- **Next:** 3D turntables and transfer table; the editor's 3D mode uses this run.
