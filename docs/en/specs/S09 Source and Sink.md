---
title: S09 Source and Sink
lang: en
tags: [spec, flow]
library-id: flow.source, flow.sink
status: spec
---

🌐 **EN** · [[S09 Generador y sumidero|ES]] · [[S09 Quelle und Senke|DE]] · [[Home (EN)|Index]]

# S09 Source and Sink (`flow.source`, `flow.sink`)

Definitions: `flow.source` (`source.json`) and `flow.sink` (`sink.json`). Implemented in 2D on 2026-10-30 ([[ADR-0007 2D Flow Model Without Accumulation]]). Not yet implemented: load mass as a distribution, the "on demand" and "list" arrival rules, and sink removal by signal. The 3D behaviour exists since 2026-10-31 ([[ADR-0016 Physics World Built from the Layout]]): the pallet appears moving with the conveying surface, and a receiving bed carries it into the sink. The source's `generationMode` options today are `fixedInterval` (first pallet at time 0) and `exponential` (seeded, interval = mean). `maxPallets` 0 means unlimited.

## 1. Source (pallet generator)

| Key               | Plan key            | Unit    | Default           | Range                                                  | Description                                                                                 |
| ----------------- | ------------------- | ------- | ----------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `palletType`      | `tipoPallet`        | ref     | EUR               | —                                                      | Pallet type from the library ([[S10 Pallet Types]])                                         |
| `palletModel`     | —                   | ref     | auto              | auto, a catalogue pallet, none                         | Model drawn and simulated (§1.1)                                                            |
| `loadKind`        | —                   | enum    | block             | block, none, asset                                     | What sits on the pallet (§1.1)                                                              |
| `loadAsset`       | —                   | ref     | —                 | catalogue boxes, containers, drums                     | Only with `loadKind` = asset                                                                |
| `loadMass`        | `masaCarga`         | kg      | 600               | 0 … 1500                                               | Fixed or a distribution (uniform, normal, table)                                            |
| `loadHeight`      | `alturaCarga`       | mm      | 1200              | 0 … 2200                                               | Load height above the pallet                                                                |
| `orientation`     | `orientacion`       | enum    | long along travel | long, short                                            | Orientation when created                                                                    |
| `generationMode`  | `modoGeneracion`    | enum    | fixed interval    | fixed interval, distribution, on demand (signal), list | Arrival rule                                                                                |
| `interval`        | `intervalo`         | s       | 20                | 1 … 3600                                               | For fixed interval                                                                          |
| `maxPallets`      | `maxPallets`        | —       | unlimited         | —                                                      | Limit of generated pallets                                                                  |
| `blockIfOccupied` | `bloquearSiOcupado` | bool    | yes               | —                                                      | Waits if there is no free space at the start                                                |
| `seed`            | `semilla`           | integer | global            | —                                                      | For distributions (derived per object stream, [[T03 Time, Event Ordering and Determinism]]) |

**2D.** Schedules `pallet_enters` events: fixed interval → every `interval`; distribution → inter-arrival time drawn from the seeded RNG; on demand → when the signal rises. With `blockIfOccupied`, a creation that finds the entry occupied waits (this "lost" time is reported) and the next interval counts from the moment it is placed. The entry is free when the last pallet's tail is at least pallet length + the conveyor's `minGap` from the entry.
**3D.** Spawns the compound pallet body at the output port, at rest, aligned with the conveyor.

### 1.1 Pallet model and load (amendment 2026-10-31)

- **`palletModel`.** `auto` keeps today's behaviour: the model linked to the pallet type (EUR → the catalogue's block pallet), or a plain box for types without a model. A catalogue pallet (wood, plastic, steel…) gives the drawn model, its mass, centre of mass and inertia; the contact shape is still the S10 construction of `palletType`, scaled to the model ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]). `none` creates no pallet: the load travels alone on the conveyor (a crate on rollers).
- **`loadKind`.** `block` keeps today's uniform block of `loadMass` and `loadHeight`. `none` sends the pallet empty. `asset` puts one catalogue object (`loadAsset`) centred on the pallet: its shape is the model's convex pieces ([[S17 Scenery Asset]] §2), its mass is `loadMass` (the catalogue mass describes empty packaging), its centre of mass is the model's, scaled. The load is rigidly joined to the pallet, as the block is today.
- **Sizes.** Length and width along travel come from the larger of pallet and load footprints, in 2D and 3D alike, so both modes see the same object. Height for the photocells: the pallet profile up to the pallet height, the load as a solid above it.
- **Warning `palletModelMismatch`** when the model footprint differs by more than 50 mm from the `palletType` footprint.
- `palletModel` = none with `loadKind` = none would create nothing: the source warns (`sourceCreatesNothing`) and sends an empty pallet of its type.
- A load block without a pallet travels as a carton stack (contact material `cardboard`). With a pallet, the block keeps the pallet footprint as before; a catalogue load may overhang it, and then the footprint is the larger one.

### 1.2 Load recipes and the output table (amendment 2026-11-10, awaiting the owner's confirmation; not built)

Decision: [[ADR-0031 Asset Library Load Units and Loose Loads]]. A source creates load units from **load recipes** ([[S24 Load Recipes]]), and an **output table** says which recipe each unit gets. The arrival rule (`generationMode`, `interval`, `blockIfOccupied`) keeps deciding **when** a unit leaves; the table decides **what** leaves.

New parameters:

| Key           | Unit | Default                     | Range                                                  | Description                                                       |
| ------------- | ---- | --------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------- |
| `outputRule`  | enum | `single`                    | `single`, `share`, `sequence`, `timetable`, `onDemand` | How the recipe of each unit is chosen                             |
| `recipe`      | ref  | `mu.recipe.wrapped-cartons` | a recipe of the project library                        | With `single`                                                     |
| `repeatEvery` | s    | 0                           | 0 … 86400                                              | With `timetable`: start the table again after this time; 0 = once |

The **output table** is the source's own content, edited as a grid in its property sheet (the grid of [[S19 Table]]). Its columns:

| Column     | Used by     | Meaning                                                                         |
| ---------- | ----------- | ------------------------------------------------------------------------------- |
| `recipe`   | all         | A recipe of the project library                                                 |
| `share`    | `share`     | Weight of the row; the rows need not add up to 100                              |
| `count`    | `sequence`  | Units of this row before the next row                                           |
| `time`     | `timetable` | Time since the start of the run                                                 |
| `quantity` | `timetable` | Units to create at that time                                                    |
| `mass`     | all         | Optional: replaces the recipe's load mass for this row (kg)                     |
| any other  | all         | Becomes an attribute of the unit (`destination` → `pallet.destination` in code) |

Rules:

| Rule        | Behaviour                                                                                                                                                    |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `single`    | Every unit is `recipe`. The table is not used                                                                                                                |
| `share`     | Each unit draws its row with probability share / sum of shares                                                                                               |
| `sequence`  | Row 1 `count` times, then row 2, and so on; after the last row, row 1 again                                                                                  |
| `timetable` | At each row's `time`, `quantity` units join a queue; they leave one after another as soon as the entry is free. `generationMode` and `interval` are not used |
| `onDemand`  | No arrivals by themselves; code asks for a unit (`Source1.create("recipe name")`, [[A17 IntraScript Language]]; specified, to be built)                      |

- **Seed.** The `share` draw uses a stream of its own, derived from the source's stream. Adding or changing an output table therefore never changes the arrival times of the same project.
- **`maxPallets`** counts all units, whatever their recipe.
- **Every unit** is born with its name `MU<n>` ([[ADR-0029 Sections, Addresses and Load-Unit Names]]) and the read-only attribute `recipe`, plus the row's extra columns.
- **Units of different sizes.** The entry-free rule of §1 uses the length of the unit that is leaving and of the one before it; each unit keeps its own length, width, height and mass in both modules.
- **Orientation.** `orientation` stays a parameter of the source and applies to every recipe.
- **Statistics.** Units created and waiting time lost, in total and per recipe.
- **3D.** A `rigid` recipe leaves as one body, a `loose` recipe as a carrier with its items ([[S24 Load Recipes]] §4), at the conveying speed as today.

**Old parameters.** `palletType`, `palletModel`, `loadKind`, `loadAsset`, `loadMass` and `loadHeight` (§1 and §1.1) keep working: a source that has no `recipe` set behaves exactly as before, and its values are shown as the matching built-in recipe (S24 §3) with that pallet type, mass and height. New sources use recipes. `palletModel` = `none` (a load travelling without a pallet) stays available through the old parameters only; recipes always have a carrier for now (ADR-0031 §5).

Validations:

| Id                    | Level   | When                                                             |
| --------------------- | ------- | ---------------------------------------------------------------- |
| `outputTableEmpty`    | error   | A rule other than `single` and `onDemand` with no rows           |
| `outputRecipeMissing` | error   | A row names a recipe that is not in the project library          |
| `outputShareInvalid`  | error   | `share`: a negative share, or all shares zero                    |
| `outputCountInvalid`  | error   | `sequence` or `timetable`: a count or quantity below 1           |
| `outputTimesUnsorted` | warning | `timetable`: times not rising; the rows are used in time order   |
| `unitWiderThanDeck`   | warning | A recipe's footprint is wider than the conveyor the source feeds |

Reference tests:

- **Unchanged.** A source with `single` and a built-in recipe gives the same event times and statistics as the same source with the old parameters, in every reference scenario.
- **Share.** Two rows, shares 70 and 30, 10,000 units: the first recipe is drawn 7000 ± 118 times (99 % of a binomial with standard deviation √(10000 × 0.7 × 0.3) = 45.8); the same seed gives the same sequence; shares 7 and 3 give the same sequence as 70 and 30.
- **Arrival times untouched.** With `exponential` arrivals, the arrival times with and without a `share` table are identical.
- **Sequence.** Rows A × 2 and B × 1: the first nine units are A A B A A B A A B.
- **Timetable.** One row at 60 s with quantity 3, EUR pallets long side first onto a roller conveyor running at 0.25 m/s with `minGap` 0.15 m: units leave at 60 s, 65.4 s and 70.8 s (headway (1.2 + 0.15) / 0.25 = 5.4 s, as in test T2), and nothing before 60 s.
- **Mixed sizes.** A sequence of an EUR unit and a 1200 × 1000 unit, both long side first: both have the same headway; short side first, the gap between them is still `minGap`, measured tail to front.
- **Attributes.** A row with `destination` = B gives units whose `pallet.destination` reads "B" in code.

## 2. Sink

| Key           | Plan key         | Unit | Default   | Range                              | Description        |
| ------------- | ---------------- | ---- | --------- | ---------------------------------- | ------------------ |
| `removalMode` | `modoRetirada`   | enum | immediate | immediate, after a time, by signal | How pallets leave  |
| `removalTime` | `tiempoRetirada` | s    | 0         | ≥ 0                                | For "after a time" |

Publishes pallets received, throughput and mean cycle time (entry into the model → arrival at the sink).

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                                                                                                                                        |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Design              | Family of the library (category flow); its parameters form the attribute table (inherited ✔ / overridden ✖), its presets are the family's types; ports OUT (source) and IN (sink); placed from the Project Browser, listed in the Simulation tree and in schedules |
| Discrete simulation | Arrivals at fixed or exponential intervals from the project's seeded stream; the sink frees its place after its delay                                                                                                                                              |
| Physics simulation  | The source places a dynamic MTBH (catalogue pallet and load allowed); the sink removes it                                                                                                                                                                          |

The behaviour above does not change. **Rust status:** not ported yet (library R1, 2D R3, 3D R6 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.

## Related

[[S10 Pallet Types]] · [[T08 Statistics and Output Analysis]] · [[T03 Time, Event Ordering and Determinism]]
