---
title: ADR-0013 Lift Station 2D Cycle
lang: en
tags: [decision, des, lift, elevator]
status: proposed
date: 2026-10-30
---

# ADR-0013 Lift Station 2D Cycle

- **Status:** Proposed. Waiting for the owner to confirm [[S12 Manual Lift Table]], [[S13 Scissor Lift Table]] and [[S14 Pallet Elevator]].
- **Date:** 2026-10-30

## Context

The manual lift table, the scissor lift table and the pallet elevator ([[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]]) all do the same thing in the 2D mode:

1. take one pallet at one height;
2. move it up or down;
3. give it out at another height;
4. go back empty.

They differ in stroke, speeds, drawing, and which ends the pallet enters and leaves through. The turntable, the transfer and the process station already share the `Station` interface (`flow/station.ts`, [[ADR-0009 Transfer Table 2D Cycle and Stations]]).

## Decision

1. **One behaviour for three objects.**
   - The three definitions use the same 2D behaviour id, `des.lift_station`.
   - The network turns each one into a node of kind `lift` that holds:
     - the entry port and its height;
     - the exit port and its height;
     - the up speed, the down speed and the vertical acceleration;
     - whether the pallet leaves through the **same end** it came in (C) or the **opposite end** (Z).
   - The class `LiftStation` implements `Station`.
2. **Cycle.** One pallet at a time. The station takes a pallet only when it is idle at the entry height.

| Phase       | What happens                                                                              | Ends when                   |
| ----------- | ----------------------------------------------------------------------------------------- | --------------------------- |
| idle        | Deck waits at the entry height                                                            | A pallet is accepted        |
| receiving   | The deck runs the pallet in and stops it centred, like the process station                | The pallet is centred       |
| moving      | Vertical move from the entry height to the exit height (timed step)                       | Profile time is over        |
| discharging | The deck runs the pallet out through the exit port; blocked while the next object is full | The deck is empty           |
| returning   | Empty vertical move back to the entry height (timed step)                                 | Profile time is over → idle |

3. **Vertical profile.**
   - The move starts and ends at rest and is trapezoidal ([[T04 Conveyor Kinematics]] §2), with $a = b$ = vertical acceleration.
   - Over a stroke $D = |h_{exit} - h_{entry}|$: $t = D/v + v/a$ if $D \ge v^2/a$; otherwise $t = 2\sqrt{D/a}$.
   - $v$ is the up or the down speed, depending on the direction of the move.
4. **Leaving on the same side (C).**
   - The deck runs the other way to discharge.
   - The pallet **does not turn**: its side along travel stays the same, and its former tail now leads.
   - In the station, the deck path takes the opposite heading and the centred pallet stays centred. The turntable does the same after a 180° turn.
5. **Return policy.**
   - After discharging, the station always goes back to the entry height at once.
   - Waiting at the exit height until the next pallet is called is not modelled yet ([[Open Questions]] Q15).
6. **Height in the viewer.** The snapshot gets `heights`: the deck height of every lift station, interpolated along the profile, like the turntable's `angles`. The 3D view uses it to raise the deck and the pallet.
7. **Ports at the same spot.**
   - The C elevator has its entry and its exit at the same position seen from above, at different heights.
   - Snapping and connection detection pick the port with the closest height. `portHeightMismatch` still warns when the heights differ by more than 10 mm.
8. **Raised base.**
   - An instance's position already has a vertical coordinate.
   - The properties panel will show it as "base height" (mezzanine), using the existing move command, so that conveyors can be placed at the elevator's upper level.

## Alternatives considered

- **One class per object.** Rejected: three copies of the same cycle.
- **A fixed lift time instead of speed and acceleration.**
  - Rejected for the elevator: its time depends on the stroke.
  - For the hydraulic tables, a speed per direction gives the same result as a time per direction and stays right when the heights change.

## Consequences

- Tests (`lift-station.test.ts`):
  - **T8:** scissor, 0.6 → 1.2 m at 0.05 m/s and 0.2 m/s² → 12.25 s.
  - **T9:** elevator, 0.6 → 4.6 m at 0.5 m/s and 0.5 m/s² → 9 s.
  - Blocked discharge; one pallet per cycle; down moves; C and Z exits; the manual table in both directions.
- 3D physics (`phys.lift_station`) waits for spike S1.
