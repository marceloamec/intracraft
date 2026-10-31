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

## 2. Sink

| Key           | Plan key         | Unit | Default   | Range                              | Description        |
| ------------- | ---------------- | ---- | --------- | ---------------------------------- | ------------------ |
| `removalMode` | `modoRetirada`   | enum | immediate | immediate, after a time, by signal | How pallets leave  |
| `removalTime` | `tiempoRetirada` | s    | 0         | ≥ 0                                | For "after a time" |

Publishes pallets received, throughput and mean cycle time (entry into the model → arrival at the sink).

## Related

[[S10 Pallet Types]] · [[T08 Statistics and Output Analysis]] · [[T03 Time, Event Ordering and Determinism]]
