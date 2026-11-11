---
title: ADR-0031 Asset Library Load Units and Loose Loads
lang: en
tags: [decision, library, load-units, physics, source]
status: accepted (direction); specifications S23 and S24 follow before any code
date: 2026-11-10
---

# ADR-0031 Asset Library, Load Units and Loose Loads

- **Status:** Accepted as direction by the owner on 2026-11-10. Nothing is built. The plan is
  [[A21 Asset Library and Load Units Plan]]; the specifications of its step L0 need the owner's confirmation before code.
- **Date:** 2026-11-10

## Context

The owner asked for one library of every kind of asset: the intralogistics equipment families, and the **load units**
(MU, mobile units) of several kinds (pallets, boxes, drums and others). It must be possible to **assemble** a load, to
choose **how loads leave a source**, and to decide **how a load is physically put together**.

Today:

- a source creates one pallet type with one of four fixed loads, which always fills the pallet's footprint up to a height
  ([[S09 Source and Sink]] §1.1, `intracraft-assets`);
- pallet and load are one rigid body, and load shift is outside the MVP
  ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]] §1);
- boxes and totes as conveyed units, and conveyors for them, are outside the MVP (`AGENTS.md`);
- library objects live in each project as BIMCraft families; there is no library shared between projects.

Three questions were put to the owner with two alternatives each. The answers are the decisions below.

## Decision

1. **Load units are library entries of their own**, beside the equipment families: a *load-unit type* (a pallet, a box, a
   drum, a tote: size, tare, contact material, what a photocell sees) and a *load recipe* (a carrier, the items on it, the
   stacking pattern, how it is secured, its mass). Specifications: [[S23 Load-Unit Types]] and [[S24 Load Recipes]].
2. **A source has an output table** instead of one load: rows of recipes leaving by share, in a repeating sequence, by a
   timetable, or on demand from IntraScript. Every draw uses the project's seeded stream.
3. **A load can be loose** (owner's answer 1). The recipe's securing says how the load is built in the Physics simulation
   module:
   - `rigid` (film, straps): one body, as today. This stays the default.
   - `loose`: each item is a body of its own, held by friction and weight, so it can shift, tip and fall.

   This amends [[ADR-0015 MTBH Rigid Body from Pallet and Load Models]] §1: "the pallet and the load never slide against
   each other" now holds for `rigid` recipes only. The rest of ADR-0015 stands (contact shape of the pallet from its type,
   mass combination for rigid loads).
4. **In the Discrete simulation module a load unit is always one unit**, whatever its securing. Loose behaviour exists in
   the Physics simulation module only.
5. **For now load units travel on pallets** (owner's answer 2). Boxes, drums and totes are items of a recipe on a pallet.
   Conveyors for cartons, totes and parcels stay outside the scope; the owner intends to add them later. Load-unit types
   are therefore described so that a type can later be conveyed on its own without changing existing projects. What
   [[S09 Source and Sink]] §1.1 already allows (a load travelling without a pallet on pallet conveyors) is kept as it is
   and not extended.
6. **Two libraries** (owner's answer 3):
   - the **general library**, outside any project: what ships with IntraCraft plus what the user has added;
   - the **project library**, inside the project file: what this project uses.

   An asset enters a project by **dragging it from the general library into the project library**, which makes a copy. The
   project never reads the general library when it is opened or run.

## Rules that follow

- **A project stays self-sufficient.** The same project file and seed give the same result on a machine with a different
  general library, or none.
- **A copy remembers where it came from** (the entry's GUID and version). When the general entry changes later, the
  project shows that an update exists; nothing changes until the user accepts it, as one undoable command.
- **Dragging brings what the asset needs:** a recipe brings its carrier and item types; a source preset brings its recipes.
- **The other way exists too:** an asset made in a project can be sent to the general library.
- **Every one of these actions is a command** (copy into project, update from general, send to general, edit a type, edit
  a recipe, edit a source's output table), with undo and open to agents.
- **Loose loads never run decomposition or random choices at start-up.** Item positions come from the recipe; the same
  layout and seed give the same result.
- **No new friction pair is needed.** [[S16 Contact Materials]] already has every pair between packaging, pallets and
  conveyors (carton on carton 0.4, carton on wood 0.35, plastic on plastic 0.3, painted steel on wood 0.3). They are
  indicative values and are not tuned to make a loose-load test pass.

## Alternatives considered

- **Keep every load rigid.** Fast and simple; rejected by the owner: loads that shift or fall are part of what the physics
  module should show.
- **Open the scope to carton conveyors now.** Rejected for now: new equipment, specifications and a discrete model, on the
  scale of [[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]].
- **One shared library file that projects read directly.** Easier reuse, but a project's results could change when the
  library changes, and a project sent to someone else would not open the same. Rejected in favour of copies.
- **Everything inside the project, with export and import only.** Self-sufficient, but no place to keep one's own
  collection; the owner wants a general library.

## Consequences

- A loose load of 45 cartons is 45 bodies on a moving conveyor; they cannot sleep while carried. The physics run slows
  with the number of loose items, so the plan includes a budget and a warning ([[A21 Asset Library and Load Units Plan]]
  §5).
- Photocells must see loose items, and an item that leaves its pallet needs a defined fate (logged, removed with the run).
- The four fixed loads of `intracraft-assets` become four built-in recipes with the same look and the same results; the
  example installations must give the same numbers before and after.
- `AGENTS.md` ("Qué no hacer") keeps boxes and totes as conveyed units outside the MVP; it gets a line pointing here when
  step L0 starts.
- The TypeScript reference is frozen and gets none of this; the new definitions live in `platform/sim/library/definitions`.

## Related

[[A21 Asset Library and Load Units Plan]] · [[S09 Source and Sink]] · [[S10 Pallet Types]] · [[S16 Contact Materials]] ·
[[S17 Scenery Asset]] · [[ADR-0015 MTBH Rigid Body from Pallet and Load Models]] ·
[[ADR-0029 Sections, Addresses and Load-Unit Names]] · [[A16 Asset Catalog]] · [[USD00 Study Overview]]
