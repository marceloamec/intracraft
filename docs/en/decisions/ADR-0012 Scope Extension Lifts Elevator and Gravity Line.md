---
title: ADR-0012 Scope Extension Lifts Elevator and Gravity Line
lang: en
tags: [decision, scope, library]
status: accepted
date: 2026-10-30
---

# ADR-0012 Scope Extension: Lifts, Elevator and Gravity Line

- **Status:** Accepted (project owner, 2026-10-30)
- **Date:** 2026-10-30

## Context

The MVP plan (§2.2) and `AGENTS.md` ("Qué no hacer") listed elevators as post-MVP. They also said that idle (gravity) rollers have no slope in the MVP: a pallet only crosses them when a driven section pushes it. On 2026-10-30 the project owner asked for four new pallet-handling objects that change pallet height or roll pallets without a drive. These objects are common at the edges of a pallet system, where operators with hand pallet trucks take pallets in and out.

## Decision

The MVP scope is extended with four library objects. Each one needs a specification, tests and the library checklist ([[A05 Component Library]] §7), like the original five conveyors.

| Object              | Library id                   | Specification               |
| ------------------- | ---------------------------- | --------------------------- |
| Manual lift table   | `conveyor.lift_table_manual` | [[S12 Manual Lift Table]]   |
| Scissor lift table  | `conveyor.scissor_lift`      | [[S13 Scissor Lift Table]]  |
| Pallet elevator     | `conveyor.elevator`          | [[S14 Pallet Elevator]]     |
| Gravity roller line | `conveyor.gravity`           | [[S15 Gravity Roller Line]] |

The owner's answers on 2026-10-30 that shape the specifications:

- The "small palletizer" of the operator is a **hand pallet truck** (pallet jack). The manual lift table lowers so that the operator can put down or take a pallet with it.
- The two lift tables are **two separate objects**: the manual one, for an operator with a pallet jack, and the scissor table between two roller conveyors.
- The most common elevator takes a pallet in at one height and gives it out at another height **on the same side** (C shape). Other variants will come later. The specification also covers the opposite side (Z shape), because it costs one more port.
- The gravity line ends in a **separator and an end stop**. The operator takes the first pallet with a pallet jack.
- The owner is not sure whether the queue moves as one block or spreads out because of the brake rollers. It becomes a parameter (`startDelay`).

## Still outside the MVP

- Curves, diverters, shuttles, and elevators with **three or more levels**.
- The 3D physics of brake rollers, scissors and elevator carriages (arrives with spike S1, like the other objects).
- Manual palletising (an operator stacking cases on a pallet), boxes, totes, failures, energy, scripting and OPC UA.

## Consequences

- The MVP plan §2.1/§2.2 and `AGENTS.md` are updated to list the four objects.
- Two behaviour decisions follow: [[ADR-0013 Lift Station 2D Cycle]] and [[ADR-0014 Gravity Line 2D Model]].
- Common groundwork:
  - Ports that share a position seen from above (the C elevator) are matched by height.
  - An instance can be placed on a raised base (mezzanine), so that a conveyor at 4 m does not need 4 m legs.
- New questions for the owner: [[Open Questions]] Q14–Q17.
