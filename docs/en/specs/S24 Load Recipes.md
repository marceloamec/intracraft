---
title: S24 Load Recipes
lang: en
tags: [spec, library, load-units, physics]
library-id: mu.recipe.*
status: specified — awaiting the owner's confirmation; not built
updated: 2026-11-10
---

# S24 Load Recipes (`mu.recipe.*`)

A **load recipe** says how one load unit (MU) is put together: which carrier, which items, in which pattern, how many
layers, how it is held and what it weighs. A source creates units from recipes ([[S09 Source and Sink]] §1.2). A recipe is
a library entry like a [[S23 Load-Unit Types|load-unit type]]. Decision: [[ADR-0031 Asset Library Load Units and Loose Loads]].

## 1. Parameters

| Key            | Unit | Default  | Range                                                  | Description                                                                               |
| -------------- | ---- | -------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| `carrier`      | ref  | EUR      | a carrier type                                         | What the load stands on                                                                   |
| `item`         | ref  | —        | an item type, or none                                  | None gives an empty carrier                                                               |
| `pattern`      | enum | `column` | `block`, `column`, `interlocked`, `pinwheel`, `custom` | How the items are laid (§3)                                                               |
| `itemTurn`     | enum | `auto`   | `auto`, `along`, `across`                              | Which way the item's length points (§3)                                                   |
| `layersBy`     | enum | `count`  | `count`, `height`                                      | Whether `layers` or `loadHeight` decides                                                  |
| `layers`       | —    | 1        | 1 … 50                                                 | Number of layers                                                                          |
| `loadHeight`   | mm   | 1200     | 0 … 2200                                               | With `layersBy` = `height`: as many whole layers as fit; with `block`: the block's height |
| `gap`          | mm   | 0        | 0 … 100                                                | Free space between neighbouring items in a layer                                          |
| `maxOverhang`  | mm   | 0        | 0 … 200                                                | How far the load may stand out over the carrier on each side                              |
| `securing`     | enum | `rigid`  | `rigid`, `loose`                                       | How the load is held (§4)                                                                 |
| `wrap`         | enum | `none`   | `none`, `film`, `straps`                               | Drawing only; needs `securing` = `rigid`                                                  |
| `massBy`       | enum | `items`  | `items`, `total`                                       | `items`: each item weighs what its type says; `total`: `loadMass` shared equally          |
| `loadMass`     | kg   | 600      | 0 … 5000                                               | With `massBy` = `total` or `pattern` = `block`                                            |
| `centerOffset` | mm   | 0, 0     | inside the carrier                                     | Shifts the load's centre of mass along and across; `rigid` only                           |

## 2. What a recipe works out

Shown in the recipe editor and used by both simulation modules:

| Value          | Rule                                                                    |
| -------------- | ----------------------------------------------------------------------- |
| Items          | Sum over the layers of the items the pattern places                     |
| Load height    | Layers × item height (for `block`: `loadHeight`)                        |
| Unit height    | Carrier height + load height                                            |
| Footprint      | The larger of the carrier's and the load's, per side (rule of S09 §1.1) |
| Load mass      | `items`: number × (tare + content); `total` and `block`: `loadMass`     |
| Unit mass      | Carrier tare + load mass                                                |
| Centre of mass | Mass-weighted mean of carrier and items, plus `centerOffset`            |

## 3. Patterns

Positions are exact, so two machines build the same load. `L`, `W` are the carrier's sides plus twice `maxOverhang`;
`l`, `w` the item's; `g` the gap. Along a side of size `S`, items of size `s` fit `n = floor((S + g) / (s + g))` times,
with a tolerance of 1 mm. Every layer is centred on the carrier.

| Pattern       | Rule                                                                                                                                                                            |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `column`      | A grid, every layer the same. `along` puts `l` along `L`; `across` puts `w` along `L`; `auto` takes the one that places more items, and `along` when equal                      |
| `interlocked` | Odd layers as `column`; even layers with the other turn. Needs an item that may turn                                                                                            |
| `pinwheel`    | Four items per layer round the centre, each a quarter turn from the last, on a square of side `l + w + g`; even layers mirrored. Needs an item that may turn                    |
| `custom`      | A list of places (position along, position across, turn, layer) set by hand in the editor                                                                                       |
| `block`       | Today's load: the load is **one uniform block** on the carrier's footprint, `loadHeight` high, of `loadMass`; it is only *drawn* as items, stretched to fill it. Always `rigid` |

- A cylinder uses `column` (a square grid) or `custom`; turning does not apply.
- Worked example: EUR carrier (1200 × 800), `mu.item.carton-400x300`, gap 0. `along` gives 3 × 2 = 6 per layer on
  1200 × 600. `across` gives 4 × 2 = 8 per layer on 1200 × 800. `auto` takes `across`: 8 per layer. With 5 layers: 40
  cartons, load height 1250 mm, load mass 40 × 12.5 = 500 kg, unit mass 525 kg, unit height 1394 mm.
- With `interlocked` the same example alternates 8 and 6 items; the recipe reports `layersDiffer`.

### Built-in recipes

The four loads the Rust platform draws today become four recipes of pattern `block`, so every existing project and example
gives the same results:

| Id                          | Carrier | Drawn as        | Replaces (S09 §1.1)                         |
| --------------------------- | ------- | --------------- | ------------------------------------------- |
| `mu.recipe.wrapped-cartons` | EUR     | Cartons in film | `loadKind` = `block`                        |
| `mu.recipe.cartons`         | EUR     | Cartons         | `loadKind` = `asset`, a catalogue box       |
| `mu.recipe.drums`           | EUR     | Four drums      | `loadKind` = `asset`, a catalogue drum      |
| `mu.recipe.crates`          | EUR     | Plastic crates  | `loadKind` = `asset`, a catalogue container |
| `mu.recipe.empty`           | EUR     | Nothing         | `loadKind` = `none`                         |

## 4. Securing: how the load is physically built

| Securing | Physics simulation                                                                          | Discrete simulation |
| -------- | ------------------------------------------------------------------------------------------- | ------------------- |
| `rigid`  | One body: the carrier and all items joined. Nothing shifts                                  | One unit            |
| `loose`  | The carrier is one body and **each item is a body of its own**, held by weight and friction | One unit            |

### 4.1 Rigid

- `block`: exactly [[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]: the carrier's construction plus one uniform
  block.
- Other patterns: the carrier's construction plus one box or cylinder per item, all in one body. Mass, centre of mass and
  moments of inertia by the parallel-axis rule of ADR-0015 §6.

### 4.2 Loose

- **Start.** Each item is placed exactly where the pattern says, touching what is under it, at the carrier's speed.
  Nothing is dropped and nothing is random.
- **Contacts.** Friction and restitution of each pair from [[S16 Contact Materials]]: item on carrier, item on item, item
  on anything else it meets.
- **Identity.** The unit keeps its name `MU<n>` ([[ADR-0029 Sections, Addresses and Load-Unit Names]]) with its carrier.
- **Load lost.** An item is lost when it touches the floor or its centre is outside the carrier's footprint for more than
  1 s. The run logs `loadLost` with the unit's name, the item's number, the time and the place. A lost item stays where
  it is, as an obstacle, until the run ends or is reset.
- **Sink.** The sink removes the carrier and every item that is not lost.
- **Photocells** see every item, on its carrier or lost.
- **Statistics** count units. Each unit also reports its items lost.
- **Budget.** A run warns (`looseBodyBudget`) when the loose items alive at once would exceed a limit. The limit is a
  project setting whose default is fixed by measurement in step L5 of [[A21 Asset Library and Load Units Plan]].
- **Not modelled:** film that stretches or tears, straps that break, items that deform or burst, friction that changes
  with the weight on top.
- **Stretch wrapper.** A unit keeps the securing of its recipe for its whole life, with one exception agreed with
  [[S25 Stretch Wrapper]] (§4 there, and ADR-0032): once loose loads exist, a `loose` unit that is wrapped becomes `rigid`
  with `wrap` = `film` when the film is cut, and stays so. Until then the wrapper only marks the unit as wrapped, which
  changes its drawing.

A fact worth knowing when reading results: steel rollers cannot brake a wooden pallet harder than 3.43 m/s² (wood on
steel 0.35, test P2 of S16), and cartons start to slide on the pallet at 0.35 × 9.81 = 3.43 m/s² too. So in normal
conveying a loose carton load stays put; it moves at **stops, impacts, transfers and tilts**.

## 5. Validations

| Id                  | Level   | When                                                                            |
| ------------------- | ------- | ------------------------------------------------------------------------------- |
| `noItemFits`        | error   | The pattern places no item on the carrier                                       |
| `patternNeedsTurn`  | error   | `interlocked` or `pinwheel` with an item that may not turn                      |
| `looseWithWrap`     | error   | `securing` = `loose` with `wrap` other than `none`, or with `pattern` = `block` |
| `notStackable`      | error   | More than one layer of an item that is not stackable                            |
| `loadOverhang`      | warning | The load stands out over the carrier (allowed up to `maxOverhang`)              |
| `carrierOverloaded` | warning | Load mass above the carrier's `maxLoad`                                         |
| `itemOverloaded`    | warning | The weight on a bottom item above its `maxLoadOnTop`                            |
| `unitTooHigh`       | warning | Unit height above the lowest clearance of the project's equipment, where known  |
| `layersDiffer`      | info    | Layers hold different numbers of items                                          |
| `centerOffsetLoose` | warning | `centerOffset` set on a `loose` recipe (ignored)                                |
| `recipeInUse`       | error   | Deleting a recipe a source of the project uses                                  |

## 6. Recipe editor

A window with the parameters on one side and the unit in 3D on the other. The values of §2 and the validations of §5 update
with every change. With `custom`, items are dragged on the layer shown and snap to the carrier's edges and to each other.
Every change is a command.

## 7. Reference tests

Expected values come from this note and from mechanics. Tolerances are for the physics tests only.

**Arithmetic (both modules)**

- The worked example of §3: 8 per layer with `auto`, 6 with `along`; 40 cartons, 500 kg, 525 kg, 1394 mm.
- The same with `massBy` = `total`, `loadMass` 600: each carton 15 kg, unit 625 kg.
- `mu.item.drum-200l` on EUR, `column`, one layer: 2 × 1 = 2 drums (1200 / 585 = 2.05; 800 / 585 = 1.37); with
  `maxOverhang` 190: 2 × 2 = 4 (1180 / 585 = 2.02).
- Centre of mass of the worked example: on the vertical through the carrier's centre, at
  (25 × 72 + 500 × (144 + 625)) / 525 = 736 mm above the underside.
- Each validation of §5 fires on a recipe built to break it.

**Same results as today**

- Every example installation and reference scenario gives the same numbers with the built-in `block` recipes as with the
  S09 §1.1 parameters they replace.

**Rigid**

- The worked example as `rigid`: mass 525 kg and centre of mass at 736 mm in the physics body, within 0.1 %.
- With `centerOffset` 100, 0 the unit tips at a conveyor end earlier by the amount scenario P6 of ADR-0015 gives for the
  same masses.

**Loose** (bench with one EUR pallet)

- **Standing.** The worked example stands for 60 s on a stopped conveyor: no item moves more than 1 mm. This test comes
  first; the others mean nothing until it passes.
- **Conveying.** Started at 0.5 m/s² to 0.25 m/s and carried 5 m: no item moves more than 1 mm relative to the carrier
  (0.5 < 3.43 m/s²).
- **Stop.** The unit hits a mechanical stop at 0.5 m/s. The carrier stops; the load slides forward on the deck by
  v² / (2 μ g) = 0.25 / (2 × 0.35 × 9.81) = 36 mm, within 25 % (the carrier's own stopping distance and rebound are inside
  that margin).
- **Tipping.** A test item 200 × 200 × 1000 mm stands on the carrier, on rubber-coated rollers. Braking at 1.5 m/s² leaves
  it standing; braking at 3.0 m/s² tips it over (limit 9.81 × 200 / 1000 = 1.96 m/s², below the sliding limit 3.43 m/s²).
- **Load lost.** The tipped item is reported once as `loadLost`, with the unit's name.
- **Determinism.** The stop test run twice with the same seed gives the same item positions to the last digit.

## In the three modules

| Module              | This object                                                                               |
| ------------------- | ----------------------------------------------------------------------------------------- |
| Design              | Library entry with the recipe editor; listed in the "Load units" table; chosen by sources |
| Discrete simulation | Footprint, height and mass of the unit; always one unit                                   |
| Physics simulation  | One body (`rigid`) or a carrier and loose items (`loose`); load-lost events               |

**Rust status:** not built (steps L3, L4 and L5 of [[A21 Asset Library and Load Units Plan]]).

## Related

[[S23 Load-Unit Types]] · [[S09 Source and Sink]] · [[S10 Pallet Types]] · [[S16 Contact Materials]] ·
[[ADR-0015 MTBH Rigid Body from Pallet and Load Models]] · [[ADR-0031 Asset Library Load Units and Loose Loads]]
