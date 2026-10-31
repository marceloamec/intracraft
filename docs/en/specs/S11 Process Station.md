---
title: S11 Process Station
lang: en
tags: [spec, flow, station]
library-id: flow.process_station
status: spec
updated: 2026-10-30
---

# S11 Process Station (`flow.process_station`)

A short deck (rollers or chains) where **one pallet at a time stops and is processed for a time**: inspection, strapping, wrapping, manual work. It sits between two conveyors, or between a source and a sink. It is the simplest block to find bottlenecks: its `utilization` and `blockedShare` show whether it limits the line.

Values are **indicative**, not manufacturer data. Inspiration for the concept (processing-time distribution, working / waiting / blocked shares) is recorded in [[ADR-0011 Reference Review of a Commercial Simulator Manual]]; nothing else is taken from it.

## 1. Parameters

| Key                     | Unit | Default | Range                      | Description                                       |
| ----------------------- | ---- | ------- | -------------------------- | ------------------------------------------------- |
| `length`                | m    | 1.6     | 1.0 … 3                    | Deck length; at least pallet length + margins     |
| `nominalSpeed`          | m/s  | 0.25    | 0.05 … 1                   | Speed of the deck while moving a pallet in or out |
| `accel`                 | m/s² | 0.5     | 0.05 … 2                   | Acceleration and deceleration of the deck         |
| `deckType`              | enum | rollers | rollers, chains            | Drawing only (3D); both convey the same way in 2D |
| `palletSideAlongTravel` | enum | long    | long, short                | Pallet side that travels first                    |
| `processingMode`        | enum | fixed   | fixed, normal, exponential | How the processing time of each pallet is chosen  |
| `processingTime`        | s    | 30      | 0.1 … 7200                 | Fixed time, or the mean of the distribution       |
| `processingStdDev`      | s    | 5       | 0 … 3600                   | Standard deviation, only for `normal`             |
| `surfaceHeight`         | m    | 0.6     | 0.2 … 1.5                  | Height of the ports; match the conveyors          |

Ports: `IN` at x = 0 facing −x, `OUT` at x = `length` facing +x, both at `surfaceHeight`.

## 2. 2D behaviour

One cycle, never more than one pallet on the deck:

1. **Receive.** The pallet enters from `IN` and is centred on the deck (same motion rules as the other stations).
2. **Process.** When centred and stopped, the processing time starts. `fixed` uses `processingTime`; `normal` draws from N(`processingTime`, `processingStdDev`²), cut at 0; `exponential` draws with `processingTime` as mean. Draws use the project's seeded generator, one stream per station.
3. **Discharge.** The deck runs the pallet out through `OUT`. If the next object cannot take it, the pallet waits at the end and the station is **blocked**.
4. **Free.** When the deck is empty the cycle ends and the station can take the next pallet.

A station whose `OUT` is not connected keeps the pallet after processing (`noExit` warning). Direct station-to-station connections are allowed when the next object has an input port.

## 3. Statistics

| Key                  | Meaning                                                           |
| -------------------- | ----------------------------------------------------------------- |
| `utilization`        | Share of time spent processing                                    |
| `blockedShare`       | Share of time with a processed pallet waiting for the next object |
| `idleShare`          | Remaining time: waiting for a pallet, or moving one in or out     |
| `palletsProcessed`   | Pallets that finished processing                                  |
| `meanProcessingTime` | Mean processing time of finished pallets (s)                      |
| `meanCycleTime`      | Mean time from accepting a pallet to being free again (s)         |

`utilization + blockedShare + idleShare = 1`.

## 4. Validations

| Rule             | When                                                      |
| ---------------- | --------------------------------------------------------- |
| `noExit`         | `OUT` is not connected                                    |
| `deckTooShort`   | `length` is below the EUR long side + 0.1 m               |
| `negativeStdDev` | `processingStdDev` below 0 (the editor range prevents it) |

## 5. Not included yet

Signals (`enable` input, `working` output), set-up times, failures, several pallets at once (parallel stations) and the 3D physics behaviour `phys.process_station` (arrives with spike S1). Ideas for further blocks: [[Block Ideas]].

## 6. Reference tests

- **T6:** one pallet, `fixed` 30 s, between a source and a sink: leaves after centring time + 30 s + discharge time (kinematics from [[T04 Conveyor Kinematics]]), error < 1 %.
- **T7:** saturated source: steady throughput = 1 / (centring + 30 s + discharge), error < 1 %.
- Shares sum to 1; same seed gives the same result; `normal` and `exponential` means converge to `processingTime`.

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                                                                                                                        |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | Family of the library (category flow); its parameters form the attribute table (inherited ✔ / overridden ✖), its presets are the family's types; ports IN and OUT; placed from the Project Browser, listed in the Simulation tree and in schedules |
| Discrete simulation | Station cycle with the process time; statistics of the station                                                                                                                                                                                     |
| Physics simulation  | A platform without vertical motion holding the pallet for the process time                                                                                                                                                                         |

The behaviour above does not change. **Rust status:** not ported yet (library R1, 2D R3, 3D R6 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.
