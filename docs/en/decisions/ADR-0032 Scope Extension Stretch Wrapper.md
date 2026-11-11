---
title: ADR-0032 Scope Extension Stretch Wrapper
lang: en
tags: [decision, scope, library, station, physics]
status: accepted
date: 2026-11-10
---

# ADR-0032 Scope Extension: Stretch Wrapper

- **Status:** Accepted (project owner, 2026-11-10)
- **Date:** 2026-11-10

## Context

The owner asked for a stretch wrapper: a machine that takes a pallet with a loose load on it, turns it on a rotating table
while film is laid round the load, and gives it out wrapped. It must have parameters such as the turning speed, connect to
roller conveyors at its entry and its exit, and work in the Physics simulation module and in the Discrete simulation
module.

The library has no machine that changes a load. The nearest objects are the process station ([[S11 Process Station]]),
which holds one pallet for a time, and the roller turntable ([[S04 Roller Turntable]]), which turns a pallet on a
kinematic platform.

On the same day the owner accepted loads that can be loose in the Physics simulation module as a direction
([[ADR-0031 Asset Library Load Units and Loose Loads]]); nothing of it is built yet.

## Decision

1. **The scope is extended with one library object**, `flow.stretch_wrapper`, specified in [[S25 Stretch Wrapper]]. It
   needs the library checklist of every object ([[A05 Component Library]] §7).
2. **The wrapper is a process station with a wrap programme.** It keeps the cycle of S11 (receive, centre, process,
   discharge) in both simulation modules. Two things differ:
   - the processing time is **computed** for each pallet from its load height and the programme (film width, overlap,
     reinforcing turns, passes, turning speed), not drawn;
   - during the processing the **table turns** a whole number of turns, from rest to rest, and stops in line.
3. **The film is drawn, not simulated.** No cloth or rope model: the carriage moves on the mast and the load changes to
   its wrapped look when the cycle ends.
4. **The load stays one rigid body for now**, as [[ADR-0015 MTBH Rigid Body from Pallet and Load Models]] says. "Loose"
   and "wrapped" are a mark on the load unit (`wrapped`) that changes how it is drawn. The risk that a loose load shifts
   on the turning table is a **warning by formula** (`loadMayShift`), not a result of the run.
5. **When `loose` loads exist** (step L5 of [[A21 Asset Library and Load Units Plan]]), the wrapper becomes the place
   where a `loose` unit turns `rigid`. That step amends [[S25 Stretch Wrapper]] §4 and needs no change to projects: the
   parameters stay.
6. **The table is round and its rollers lie inside the circle**, so that it turns between two conveyors without sweeping
   into them or into a pallet waiting at the entry.

The owner's answers on 2026-11-10 that shape the specification:

- A turntable wrapper, with roller conveyors at the entry and at the exit, in line.
- The discrete module has it too.
- Rigid load with a mark first; separate bodies later.

## Alternatives considered

- **A new station type of its own**, beside the process station and the turntable. Rejected: it would repeat the cycle,
  the connection rules, the issues and the statistics of S11 in both engines for the sake of one turning table.
- **A parameter set of the process station** (a preset "wrapper" with a fixed processing time). Rejected: the time would
  not follow the load height, the pallet would not turn, and the physics run would show nothing of the machine.
- **Film as a simulated material.** Rejected: far outside what the engines do in real time, and it adds no result a
  planner uses.
- **Loose bodies now.** Postponed to step L5 of A21: it needs load recipes, new friction pairs and a standing-stack test
  first.
- **Rotating-arm or ring wrapper** (the pallet stands still). Not asked for; a later variant.

## Still outside the scope

Film breaks and roll changes (failures), top sheets, pre-stretch, strapping, an exit at 90°, and energy.

## Consequences

- The TypeScript reference is frozen and does not get the object. Its definition lives in
  `platform/sim/library/definitions`, like the code block's.
- The load unit gets its first mark that a machine changes (`wrapped`). It is the first of the "pallet attributes" that
  [[A21 Asset Library and Load Units Plan]] §3 describes, and must move into them when they are built.
- `loadFriction` is a parameter of the wrapper because a load unit does not yet say what its items are made of. Its
  default, 0.3, is the lowest packaging pair of [[S16 Contact Materials]]. When load recipes name their items, the pair of
  the unit's items is the source and the parameter only overrides it.
- The source's default load (`block`) already looks wrapped. A line that shows wrapping needs a source with a load of
  loose cartons (`loadKind` = `asset`, a model of the category boxes).
- `AGENTS.md` and [[Home (EN)]] list the new object.

## Related

[[S25 Stretch Wrapper]] · [[S11 Process Station]] · [[S04 Roller Turntable]] ·
[[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]] · [[ADR-0031 Asset Library Load Units and Loose Loads]] ·
[[A21 Asset Library and Load Units Plan]]
