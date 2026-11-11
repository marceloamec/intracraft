---
title: A21 Asset Library and Load Units Plan
lang: en
tags: [app, plan, library, load-units, source, physics]
status: plan; nothing built
updated: 2026-11-10
---

# A21 Asset Library and Load Units Plan

How IntraCraft gets one library for every kind of asset, load units (MU) that can be assembled, sources that choose what
leaves and how, and loads that are physically rigid or loose. Decisions: [[ADR-0031 Asset Library Load Units and Loose Loads]].
This is a plan; each step starts with its specification and its tests ([[A19 Rust Implementation Plan]] stays the plan of
the Rust platform, and this note adds to it).

## 1. The library has four shelves

| Shelf           | What it holds                                                                           | Example                                                 | Today                                     |
| --------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------- | ----------------------------------------- |
| Equipment       | Intralogistics families with parameters, ports and behaviour in both simulation modules | Roller conveyor, elevator, photocell                    | 15 built-in families                      |
| Load-unit types | What travels or is stacked: carriers and items                                          | EUR pallet, carton 400 × 300, 200 L drum, tote          | 3 pallet types; items are fixed shapes    |
| Load recipes    | How a load unit is put together                                                         | "EUR, 45 cartons in 5 interlocked layers, film, 600 kg" | 4 fixed loads                             |
| Scenery         | What stands in the plant outside the flow                                               | Rack, fence, forklift                                   | A box placeholder ([[S17 Scenery Asset]]) |

A **load-unit type** (specification S23, to write) says:

- role: `carrier` (a pallet: loads stand on it and conveyors carry it) or `item` (a box, drum, tote: stands on a carrier);
- size, tare, contact material ([[S16 Contact Materials]]);
- for a carrier, the board-and-block construction that rollers, chains and photocells meet ([[S10 Pallet Types]]);
- for an item, its shape for physics (box or cylinder today; imported shapes later) and whether others may stand on it;
- how it is drawn.

Items are not conveyed on their own for now ([[ADR-0031 Asset Library Load Units and Loose Loads]] §5). The type keeps
room for a later "conveyed on" property, so carton and tote conveyors can be added without changing projects.

## 2. A load recipe: how the load is assembled

Specification S24, to write. A recipe has:

| Part     | Choices                                                                                                  |
| -------- | -------------------------------------------------------------------------------------------------------- |
| Carrier  | A carrier type                                                                                           |
| Item     | One item type (several item types in one recipe come later)                                              |
| Pattern  | Column (every layer the same), interlocked (alternate layers turned), pinwheel, or positions set by hand |
| Layers   | A number of layers, or "fill up to a height" (today's behaviour)                                         |
| Securing | `rigid` (film or straps) or `loose`                                                                      |
| Mass     | Per item or for the whole load; an offset of the centre of mass for a lopsided load                      |
| Look     | Film, straps, labels: drawing only                                                                       |

The recipe computes and shows: number of items, total mass, centre of mass, total height, footprint and overhang. It warns
when the load overhangs the carrier, exceeds a height limit or is heavier than a conveyor in the project allows.

**Securing in the Physics simulation module:**

| Securing | Bodies                         | What can happen                    | Cost                           |
| -------- | ------------------------------ | ---------------------------------- | ------------------------------ |
| `rigid`  | One for the whole unit         | Tipping of the whole unit, impacts | As today                       |
| `loose`  | One per item, plus the carrier | Items slide, tip and fall off      | Grows with the number of items |

In the Discrete simulation module both are one unit with the recipe's footprint and mass.

## 3. How loads leave a source

The source's single load becomes an **output table** (amendment of [[S09 Source and Sink]], to write). Each row names a
recipe; one rule applies to the table:

| Rule      | Behaviour                                                         | Typical use                |
| --------- | ----------------------------------------------------------------- | -------------------------- |
| Share     | Each unit is drawn by percentage from the project's seeded stream | Product mix                |
| Sequence  | The rows repeat in order, each with a count                       | Fixed production pattern   |
| Timetable | At a time, a number of units of a recipe                          | Shift plan, truck arrivals |
| On demand | Code asks for a unit of a recipe (IntraScript)                    | Orders, pull systems       |

- The arrival rule (fixed interval, distribution) stays as it is; the table decides **what** arrives, not when, except in
  the timetable.
- Each unit is born with its name `MU<n>` ([[ADR-0029 Sections, Addresses and Load-Unit Names]]) and with attributes:
  recipe, mass, and free ones such as a destination. This is the "pallet attributes" idea of [[Block Ideas]].
- Statistics can be read by recipe.
- A source with one row behaves exactly as a source does today.

## 4. Two libraries and dragging between them

| Library         | Where                                      | Holds                                           | Changes when                            |
| --------------- | ------------------------------------------ | ----------------------------------------------- | --------------------------------------- |
| General library | Outside any project, on the user's machine | What ships with IntraCraft, plus the user's own | The user edits it or IntraCraft updates |
| Project library | Inside the project file                    | Copies of what this project uses                | Only by a command in this project       |

- **Drag from general to project** copies the asset and everything it needs (a recipe brings its carrier and item types).
- A copy remembers its origin. If the general entry changes later, the project shows "update available"; accepting is one
  undoable command, and nothing changes otherwise.
- **Send to general** puts a project asset into the general library.
- Placing an asset in the layout uses the project library. Dragging straight from the general library onto the plan does
  both steps at once.
- The four shelves look the same in both libraries: one browser with categories, search and a preview.

To check in step L0: BIMCraft already has Insert from File (copies a family with what it needs between files) and
Favorites (the user's own collection). The general library should build on them rather than beside them.

## 5. Steps

Each step ends with something to see and with tests written first.

| Step | What the responsible sees                                                                                                                                   | Depends on                                |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| L0   | Specifications S23 (load-unit types) and S24 (load recipes), amendment of S09 (output table); written 2026-11-10, to be confirmed by the owner              | —                                         |
| L1   | A "Load units" table in the Design module: create, duplicate, edit types by commands. General and project library with dragging, for load-unit types first  | L0                                        |
| L2   | A source with an output table (share, sequence, timetable) using the four loads of today as built-in recipes; both simulation modules; statistics by recipe | L1                                        |
| L3   | The recipe editor: choose carrier, item and pattern, see the load in 3D with its mass, centre of mass and warnings                                          | L1                                        |
| L4   | The Physics module builds and draws a `rigid` unit from its recipe; the four fixed loads are gone, the example installations give the same numbers          | L3                                        |
| L5   | `loose` loads in the Physics module: items shift and fall; load-lost events; a budget warning                                                               | L4                                        |
| L6   | Equipment families and scenery in the general library; on demand from IntraScript (`create` on a source)                                                    | L1, L2                                    |
| L7   | Imported models (USD) as the look and shape of an item, a scenery object or equipment                                                                       | [[USD05 Phased Plan Risks and Decisions]] |
| —    | *(not planned)* Items conveyed on their own; carton, tote and parcel conveyors                                                                              | A scope decision                          |

L2 comes before the recipe editor on purpose: mixing products at a source is useful with today's four loads already.

### Acceptance checks to write first

Expected values come from the specifications and from mechanics, not from a run.

- **Same results as before:** every example installation and reference scenario gives the same numbers with a one-row
  output table and the built-in recipes (L2, L4).
- **Share:** over 10,000 units with shares 70/30, the counts are within the binomial 99 % interval, and identical for the
  same seed (L2).
- **Recipe arithmetic:** an EUR pallet (1200 × 800) with cartons 400 × 300 holds 8 per layer with the cartons turned
  (4 × 2) and 6 unturned (3 × 2); total mass = tare + items; centre of mass by the weighted mean (L3). Worked out in
  [[S24 Load Recipes]] §3 and §7.
- **Rigid unit:** mass, centre of mass and tipping point equal the P6 scenario of
  [[ADR-0015 MTBH Rigid Body from Pallet and Load Models]] for the same load (L4).
- **Loose, standing:** a loaded pallet stands 60 s without any item moving more than 1 mm (L5). This comes first.
- **Loose, no slip:** on a conveyor accelerating at 0.5 m/s² no item moves more than 1 mm relative to the pallet, because
  cartons slide on wood only above 0.35 × 9.81 = 3.43 m/s² (L5).
- **Loose, slip:** a unit hitting a mechanical stop at 0.5 m/s has its load slide forward 36 mm, v² / (2 μ g) (L5).
- **Loose, tipping:** a tall item of depth d and height h tips, rather than slides, when d/h is below the friction
  coefficient, and starts at a deceleration of 9.81 × d/h (L5).
- **Determinism:** the same layout and seed give the same loose-load result twice (L5). A failure here has top priority.
- **Self-sufficient project:** a project opens and runs with the same result when the general library is empty (L1).

The exact tests are in [[S23 Load-Unit Types]] §8, [[S24 Load Recipes]] §7 and [[S09 Source and Sink]] §1.2.

### Loose loads: rules to specify in L0

- A unit keeps its `MU<n>` name with its carrier. An item that leaves the carrier is logged ("load lost", with the unit's
  name and the place) and stays where it falls until the run ends; the sink removes the carrier with the items still on it.
- Photocells see loose items as well as carriers.
- Statistics count units, never loose items.
- A budget of loose bodies per run, with a warning before the run when the output table and the plant's capacity would
  exceed it; the value is set after measuring in L5, not guessed now.
- The pallet conveyors' criteria (front within ±5 mm at a stop, rebound under 10 mm) keep being checked with `rigid`
  units.

## 6. Risks

| Risk                                             | Effect                                       | Mitigation                                                                                                          |
| ------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Many loose items                                 | Physics run slower than real time            | `rigid` by default; budget and warning; loose only where it matters                                                 |
| Stacks that jitter or creep while standing still | Loads that fall without a physical reason    | A standing-stack test (no drift in 60 s) before any other loose test; solver settings recorded in a decision record |
| Friction values between packaging are guesses    | Wrong slip or tipping results                | Values marked indicative in S16; editable per type                                                                  |
| Units of different length in one flow            | Wrong gaps between a long and a short unit   | The discrete model already keeps each unit's own size; a mixed-size test is in S09 §1.2                             |
| Two libraries drift apart                        | Confusion about which version a project uses | Origin and version on every copy; update only by command                                                            |
| Scope creeping to carton conveyors               | The plan grows without a decision            | Items stay on pallets until a scope decision                                                                        |

## Related

[[ADR-0031 Asset Library Load Units and Loose Loads]] · [[A05 Component Library]] · [[A16 Asset Catalog]] ·
[[A19 Rust Implementation Plan]] · [[S09 Source and Sink]] · [[S10 Pallet Types]] · [[S16 Contact Materials]] ·
[[USD00 Study Overview]]
