---
title: S09 Source and Sink
lang: en
tags: [spec, flow]
library-id: flow.source, flow.sink
status: spec
---

🌐 **EN** · [[S09 Generador y sumidero|ES]] · [[S09 Quelle und Senke|DE]] · [[Home (EN)|Index]]

# S09 Source and Sink (`flow.source`, `flow.sink`)

Definitions: `flow.source` (`source.json`) and `flow.sink` (`sink.json`). Implemented in 2D on 2026-10-30 ([[ADR-0007 2D Flow Model Without Accumulation]]). Not yet implemented: load mass as a distribution, the "on demand" and "list" arrival rules, sink removal by signal, and the 3D behaviour. The source's `generationMode` options today are `fixedInterval` (first pallet at time 0) and `exponential` (seeded, interval = mean). `maxPallets` 0 means unlimited.

## 1. Source (pallet generator)

| Key               | Plan key            | Unit    | Default           | Range                                                  | Description                                                                                 |
| ----------------- | ------------------- | ------- | ----------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `palletType`      | `tipoPallet`        | ref     | EUR               | —                                                      | Pallet type from the library ([[S10 Pallet Types]])                                         |
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

## 2. Sink

| Key           | Plan key         | Unit | Default   | Range                              | Description        |
| ------------- | ---------------- | ---- | --------- | ---------------------------------- | ------------------ |
| `removalMode` | `modoRetirada`   | enum | immediate | immediate, after a time, by signal | How pallets leave  |
| `removalTime` | `tiempoRetirada` | s    | 0         | ≥ 0                                | For "after a time" |

Publishes pallets received, throughput and mean cycle time (entry into the model → arrival at the sink).

## Related

[[S10 Pallet Types]] · [[T08 Statistics and Output Analysis]] · [[T03 Time, Event Ordering and Determinism]]
