---
title: S23 Load-Unit Types
lang: en
tags: [spec, library, load-units]
library-id: mu.carrier.*, mu.item.*
status: specified — awaiting the owner's confirmation; not built
updated: 2026-11-10
---

# S23 Load-Unit Types (`mu.carrier.*`, `mu.item.*`)

A **load-unit type** describes one kind of thing that travels or is stacked: a pallet, a carton, a drum, a tote. It is a
library entry beside the equipment families ([[A21 Asset Library and Load Units Plan]] §1), lives in the general and in the
project library, and is edited by commands. A [[S24 Load Recipes|load recipe]] puts types together into the load unit (MU)
a source creates. Decision: [[ADR-0031 Asset Library Load Units and Loose Loads]].

Values are **indicative**. They are not manufacturer or standard data; check them against the real packaging before
relying on a result.

## 1. Two roles

| Role      | What it is                            | Conveyed by                 | Examples                        |
| --------- | ------------------------------------- | --------------------------- | ------------------------------- |
| `carrier` | Carries a load; the conveyors move it | The pallet conveyors        | EUR pallet, 1200 × 1000, GMA    |
| `item`    | Stands on a carrier, alone or stacked | Nothing on its own, for now | Carton, drum, tote, crate, sack |

An item is never conveyed on its own in this version (ADR-0031 §5). The type has a reserved property `conveyedOn`
(`palletConveyors` for carriers, `none` for items) so that carton, tote and parcel conveyors can be added later without
changing projects.

## 2. Parameters

| Key                  | Unit | Default   | Range                           | Role    | Description                                                        |
| -------------------- | ---- | --------- | ------------------------------- | ------- | ------------------------------------------------------------------ |
| `role`               | enum | —         | `carrier`, `item`               | both    | Fixed when the type is created                                     |
| `shape`              | enum | by role   | `pallet`, `box`, `cylinder`     | both    | `pallet` only for carriers; a cylinder stands upright              |
| `length`             | mm   | —         | 50 … 3000                       | both    | Longer horizontal side; for a cylinder, its diameter               |
| `width`              | mm   | —         | 50 … 3000                       | both    | Shorter horizontal side; for a cylinder, equal to `length`         |
| `height`             | mm   | —         | 10 … 3000                       | both    |                                                                    |
| `tare`               | kg   | —         | 0 … 500                         | both    | Mass when empty                                                    |
| `contentMass`        | kg   | 0         | 0 … 2000                        | item    | Mass of what it holds; a recipe may replace it (S24 §2)            |
| `material`           | enum | by shape  | an [[S16 Contact Materials]] id | both    | Contact material                                                   |
| `construction`       | ref  | —         | an [[S10 Pallet Types]] id      | carrier | The boards and blocks that rollers, chains and photocells meet     |
| `maxLoad`            | kg   | 1500      | 0 … 5000                        | carrier | Heaviest load allowed on it; a recipe above it warns               |
| `mayTurn`            | bool | yes       | —                               | item    | May be turned a quarter turn in a pattern                          |
| `stackable`          | bool | yes       | —                               | item    | Others may stand on it                                             |
| `maxLoadOnTop`       | kg   | unlimited | ≥ 0                             | item    | Heaviest weight allowed on one item; a recipe above it warns       |
| `centerOfMassOffset` | mm   | 0, 0, 0   | inside the shape                | both    | From the shape's centre; for a part-filled drum or a lopsided item |
| `look`               | enum | by shape  | §4                              | both    | How it is drawn                                                    |
| `conveyedOn`         | enum | by role   | `palletConveyors`, `none`       | both    | Reserved, not editable yet                                         |

- Mass of an item = `tare` + `contentMass`, taken as evenly spread unless `centerOfMassOffset` is set.
- A carrier of shape `pallet` takes its footprint, height and tare from `construction` by default; they may be overridden,
  with the warning `carrierDiffersFromConstruction` when the footprint differs by more than 50 mm (the rule of
  [[S09 Source and Sink]] §1.1).
- Names have `en` (required) and optional `es` and `de`, like every library text.

## 3. Built-in types

Carriers are the pallet types of [[S10 Pallet Types]], unchanged:

| Id                        | L × W × H (mm)    | Tare (kg) | Material |
| ------------------------- | ----------------- | --------- | -------- |
| `mu.carrier.eur`          | 1200 × 800 × 144  | 25        | `wood`   |
| `mu.carrier.iso1200x1000` | 1200 × 1000 × 150 | 28        | `wood`   |
| `mu.carrier.gma`          | 1219 × 1016 × 140 | 25        | `wood`   |

Items. Sizes of the carton and the crate are those the Rust platform already draws (`intracraft-assets`); the others are
common sizes. All masses are indicative.

| Id                       | Shape      | L × W × H (mm)  | Tare (kg) | Content (kg) | Material        | Stackable |
| ------------------------ | ---------- | --------------- | --------- | ------------ | --------------- | --------- |
| `mu.item.carton-400x300` | `box`      | 400 × 300 × 250 | 0.5       | 12           | `cardboard`     | yes       |
| `mu.item.carton-600x400` | `box`      | 600 × 400 × 300 | 0.8       | 20           | `cardboard`     | yes       |
| `mu.item.crate-600x400`  | `box`      | 600 × 400 × 250 | 2.0       | 15           | `plastic`       | yes       |
| `mu.item.tote-400x300`   | `box`      | 400 × 300 × 220 | 1.2       | 8            | `plastic`       | yes       |
| `mu.item.drum-200l`      | `cylinder` | Ø 585 × 880     | 18        | 200          | `steel-painted` | yes       |
| `mu.item.sack-25kg`      | `box`      | 600 × 400 × 120 | 0.1       | 25           | `cardboard`     | yes       |

A user creates a type by duplicating one of these or from nothing.

## 4. Look

`look` chooses the drawing; it never changes a simulation result.

| Look                        | Drawn as                                                                     |
| --------------------------- | ---------------------------------------------------------------------------- |
| `pallet`                    | The boards and blocks of the construction (today's pallet model)             |
| `carton`, `cartonDark`      | A carton with a strip of tape                                                |
| `crate`                     | A plastic crate with a darker top band                                       |
| `drum`                      | A drum with rims and a lid                                                   |
| `plain`                     | A box or cylinder in one colour (`colour` parameter)                         |
| an imported model *(later)* | A mesh from the library ([[USD05 Phased Plan Risks and Decisions]], step U5) |

An imported model changes the look only. The shape for physics stays the type's `box`, `cylinder` or `pallet` until
imported shapes are specified.

## 5. In the two simulation modules

- **Discrete simulation.** A type has no behaviour of its own. Through the recipe it gives the unit's footprint, height
  and mass.
- **Physics simulation.**
  - A carrier's contact shape is the board-and-block construction ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]
    §2), not a plain box.
  - An item's contact shape is its box or its upright cylinder, with its mass at its centre plus `centerOfMassOffset`.
  - Friction and restitution come from the pair table of [[S16 Contact Materials]]. All pairs between the six built-in
    types and the conveyors already exist there (for example cardboard on cardboard 0.4, cardboard on wood 0.35,
    painted steel on wood 0.3); no new pair is needed.
- **Photocells** see a carrier through its occupancy profile ([[S10 Pallet Types]] §3) and an item as a solid.

## 6. Validations

| Id                               | Level   | When                                                              |
| -------------------------------- | ------- | ----------------------------------------------------------------- |
| `typeSizeInvalid`                | error   | A size is zero, negative or outside its range; `width` > `length` |
| `carrierNeedsConstruction`       | error   | A carrier of shape `pallet` names no construction                 |
| `carrierDiffersFromConstruction` | warning | Footprint more than 50 mm from the construction's                 |
| `centerOfMassOutside`            | error   | `centerOfMassOffset` puts the centre of mass outside the shape    |
| `typeInUse`                      | error   | Deleting a type a recipe of the project uses                      |
| `typeRoleFixed`                  | error   | Changing `role` after creation                                    |

## 7. Commands

Each is one undo step, checked by `Model::check`, and open to agents: create a type, duplicate, edit, rename, delete; copy
from the general library into the project; update a project copy from the general library; send a project type to the
general library.

## 8. Reference tests

- The three built-in carriers give the same footprint, height, tare, contact shape and occupancy profile as the pallet
  types of [[S10 Pallet Types]] do today.
- `mu.item.carton-400x300` weighs 12.5 kg; `mu.item.drum-200l` weighs 218 kg.
- A drum with `centerOfMassOffset` 0, 0, −200 has its centre of mass 240 mm above its base (440 − 200).
- Each validation of §6 fires on a type built to break it, and on no built-in type.
- A project opened with an empty general library lists the same types as before.

## In the three modules

| Module              | This object                                                                                                      |
| ------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Design              | Library entries listed in a "Load units" table; created and edited by commands; dragged from the general library |
| Discrete simulation | Size and mass of the units that recipes build                                                                    |
| Physics simulation  | Contact shape, mass and material of carriers and items                                                           |

**Rust status:** not built (step L1 of [[A21 Asset Library and Load Units Plan]]).

## Related

[[S24 Load Recipes]] · [[S09 Source and Sink]] · [[S10 Pallet Types]] · [[S16 Contact Materials]] ·
[[ADR-0031 Asset Library Load Units and Loose Loads]] · [[A21 Asset Library and Load Units Plan]]
