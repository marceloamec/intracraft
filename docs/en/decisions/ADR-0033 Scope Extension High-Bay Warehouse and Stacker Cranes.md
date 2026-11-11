---
title: ADR-0033 Scope Extension High-Bay Warehouse and Stacker Cranes
lang: en
tags: [decision, scope, library, storage]
status: accepted
date: 2026-11-10
---

# ADR-0033 Scope Extension: High-Bay Warehouse and Stacker Cranes

- **Status:** Accepted (project owner, 2026-11-10)
- **Date:** 2026-11-10

## Context

Until now IntraCraft only moved pallets: conveyors, stations and lifts with two levels. `AGENTS.md` ("Qué no hacer") kept
everything else out "until further notice". On 2026-11-10 the project owner asked for a **configurable high-bay warehouse
(HBW) for pallets**: length, width, height, number of aisles with stacker cranes, single or double deep, among other
values. Her conditions:

- The stacker cranes **always hand over and pick up pallets with their forks on chain conveyors**.
- These chain conveyors can be at **one height or at several different levels**.
- Other agents create **complete example layouts**: warehouse, conveyor loop, pallet entries and exits.

She left open whether this is an extra module or a wizard.

## Decision

The scope is extended with one library object and one wizard.

| What               | Id / place                                     | Specification                     |
| ------------------ | ---------------------------------------------- | --------------------------------- |
| Storage aisle      | `storage.aisle` (library, category `storage`)  | [[S26 Storage Aisle]]             |
| High-bay warehouse | Wizard of the Design module, and a CLI command | [[A22 High-Bay Warehouse Wizard]] |

1. **A wizard in the Design module, not a fourth module.** The warehouse is part of the one layout; the Discrete and the
   Physics simulation run it like every other object. A fourth module would hold a second model of the same plant.
2. **One library object per aisle.** A storage aisle is two rack rows and the stacker crane between them. The "warehouse"
   is what the wizard places: N equal aisles side by side, with their chain conveyors. Each aisle has its own statistics
   and can be edited, moved or deleted alone, and a plant can mix aisles of different sizes.
   - Rejected: one block object for the whole warehouse. Its number of ports would depend on the number of aisles, which
     the library's fixed port lists don't allow, and its statistics would hide which crane is the bottleneck.
3. **Every storage position is tracked**: which pallet stands where, and since when. First in, first out and the
   relocation of a double-deep front pallet need it. A statistical rack (a counter and a mean cycle time) was rejected.
4. **The wizard places aisles and their pick-up and deposit chain conveyors only** (owner, 2026-11-10). The front zone
   (loop, entries, exits) is drawn with the library or taken from an example.
5. **Discrete simulation first, physics last** (owner, 2026-11-10). In the Physics module the crane will carry the pallet
   guided, without friction; the pallet is a physics body again once it stands on the outfeed chains.
6. **Pick-up and deposit at the front end only**, infeed on the left rack row and outfeed on the right one, on up to
   three levels. Stations at the rear end come later.
7. **Indicative values.** Crane speeds, fork times and rack measures are typical orders of magnitude, not manufacturer
   data ([[Open Questions]] Q22).

## Still outside the scope

- Stacker cranes that change aisle (curve-going or with a transfer car), two cranes in one aisle, twin forks or two
  pallets per trip, satellite and shuttle systems, more than double deep.
- Pick-up and deposit stations at the rear end or inside the rack; roller conveyors as stations.
- Storage zones by turnover (ABC), batches and order lines, stock by article; orders from IntraScript code.
- Crane failures, energy, and the building (walls, sprinklers, fire sections).
- Curves, boxes, totes and OPC UA stay out as before.

## Consequences

- New specification [[S26 Storage Aisle]] and note [[A22 High-Bay Warehouse Wizard]]; reference tests T12 and T13.
- The library gains the category `storage`; the definition is Rust only (the TypeScript reference is frozen,
  [[ADR-0020 Rust Platform on BIMCraft with Three Modules]]).
- The discrete run gains a way for an object to **take a resting pallet off a conveyor's end and to set one down at a
  conveyor's start** (the forks), beside the crossing from deck to deck it already had.
- The run's snapshot gains each crane's position and the occupied storage positions, so that the 3D views show them.
- `AGENTS.md` lists the object and the wizard, and no longer keeps storage out.
- New question for the owner: [[Open Questions]] Q22.
