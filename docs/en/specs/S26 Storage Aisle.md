---
title: S26 Storage Aisle
lang: en
tags: [spec, storage, crane]
library-id: storage.aisle
status: built in the Design and Discrete simulation modules; physics to come
updated: 2026-11-10
---

# S26 Storage Aisle (`storage.aisle`)

One aisle of a **high-bay warehouse for pallets**: a rack row on each side and one **stacker crane** that runs between
them on a floor rail. The crane lifts a pallet with its forks **off the end of an infeed chain conveyor**, stores it in a
rack position, and later takes it out and sets it down **on the start of an outfeed chain conveyor**. A warehouse is
several aisles side by side; [[A22 High-Bay Warehouse Wizard]] places them. Added by
[[ADR-0033 Scope Extension High-Bay Warehouse and Stacker Cranes]].

Values are **indicative**, not manufacturer data ([[Open Questions]] Q22).

## 1. Geometry

The aisle's own frame: the origin on the floor, at the **front** of the rack, on the aisle's centre line; x runs along the
aisle into the rack, z across (the **left** row at −z, the **right** row at +z), y up.

- **Bay** `b` (1 … `bays`) is centred at x = (`b` − 0.5) × `bayPitch`. One pallet per bay.
- **Level** `l` (1 … `levels`): the pallet stands at y = `firstLevelHeight` + (`l` − 1) × `levelPitch`.
- **Front position** of a row: centred at z = ±(`aisleWidth` / 2 + `positionDepth` / 2).
- **Rear position** (double deep only): one `positionDepth` further from the aisle.
- A pallet stands with its **1200 mm side across the aisle** (into the rack) and its 800 mm side along it. This is how a
  chain conveyor carries a EUR pallet ([[S02 Chain Conveyor]]), so the forks take it without turning it.

**Pick-up and deposit stations** are at the front, in line with the front positions:

- The infeed chain conveyor ends at x = 0 on the **left** row; its last pallet rests at x = −0.8 … 0.
- The outfeed chain conveyor starts at x = 0 on the **right** row and runs away from the rack.
- The crane serves both from x = −0.4 m, the centre of a EUR pallet resting at the conveyor's end, so its rail starts
  2.2 m in front of the rack.

## 2. Parameters

| Key                 | Unit | Default | Range                 | Description                                                                |
| ------------------- | ---- | ------- | --------------------- | -------------------------------------------------------------------------- |
| `bays`              | –    | 20      | 1 … 150               | Pallet positions along the aisle, per row and level                        |
| `levels`            | –    | 8       | 1 … 25                | Storage levels                                                             |
| `deep`              | –    | 1       | 1, 2                  | Pallets deep on each side: 1 single deep, 2 double deep                    |
| `bayPitch`          | m    | 1.0     | 0.9 … 1.6             | Distance between bay centres (EUR 800 mm + clearance and upright)          |
| `levelPitch`        | m    | 1.8     | 0.6 … 3.0             | Distance between levels (pallet + load + clearance + beam)                 |
| `firstLevelHeight`  | m    | 0.6     | 0.2 … 3.0             | Height at which a pallet stands on the first level                         |
| `aisleWidth`        | m    | 1.5     | 1.4 … 2.5             | Clear width between the two rows                                           |
| `positionDepth`     | m    | 1.3     | 1.25 … 1.6            | Depth of one position (EUR 1200 mm + clearance)                            |
| `travelSpeed`       | m/s  | 3.0     | 0.5 … 5.0             | Crane speed along the aisle                                                |
| `travelAccel`       | m/s² | 0.5     | 0.1 … 2.0             | Start and stop along the aisle                                             |
| `liftSpeed`         | m/s  | 1.0     | 0.1 … 2.0             | Speed of the lifting carriage, up and down                                 |
| `liftAccel`         | m/s² | 0.5     | 0.1 … 2.0             | Start and stop of the lifting carriage                                     |
| `forkTime`          | s    | 8       | 2 … 60                | One fork cycle at a front position or a station: out, lift or lower, back  |
| `forkTimeDouble`    | s    | 12      | 2 … 90                | One fork cycle at a rear position (double deep only)                       |
| `dwellPoint`        | enum | last    | last, front           | Where an idle crane waits: where it finished, or at the lowest station     |
| `pdLevels`          | –    | 1       | 1 … 3                 | Number of heights with pick-up and deposit stations                        |
| `pdHeight1`         | m    | 0.6     | 0.2 … 40              | Conveying height of the stations of the first height                       |
| `pdHeight2`         | m    | 4.6     | 0.2 … 40              | Second height (`pdLevels` ≥ 2)                                             |
| `pdHeight3`         | m    | 8.6     | 0.2 … 40              | Third height (`pdLevels` = 3)                                              |
| `storageStrategy`   | enum | nearest | nearest, random       | Which free position a new pallet gets (§4.2)                               |
| `retrievalMode`     | enum | dwell   | none, dwell, interval | What makes a pallet leave (§4.3)                                           |
| `dwellTime`         | s    | 3600    | 1 … 2 592 000         | `dwell`: time a pallet stays stored (mean)                                 |
| `retrievalInterval` | s    | 120     | 1 … 86 400            | `interval`: time between two retrieval orders (mean)                       |
| `timeDistribution`  | enum | fixed   | fixed, exponential    | Whether `dwellTime` and `retrievalInterval` are exact or random about them |
| `retrievalOrder`    | enum | fifo    | fifo, random          | `interval`: the oldest pallet first, or any                                |
| `initialFill`       | –    | 0.5     | 0 … 1                 | Share of the positions occupied when the run starts                        |
| `initialLoadHeight` | m    | 1.2     | 0 … 2.5               | Load height of the pallets of the initial stock (EUR, block load)          |
| `initialLoadMass`   | kg   | 600     | 0 … 1500              | Load mass of the pallets of the initial stock                              |

**Derived** (shown, not edited):

| Key           | Formula                                      | With the defaults |
| ------------- | -------------------------------------------- | ----------------- |
| `length`      | `bays` × `bayPitch`                          | 20 m              |
| `height`      | `firstLevelHeight` + `levels` × `levelPitch` | 15 m              |
| `moduleWidth` | `aisleWidth` + 2 × `deep` × `positionDepth`  | 4.1 m (6.7 m)     |
| `positions`   | `bays` × `levels` × 2 × `deep`               | 320 (640)         |

**Ports** (all at x = 0, facing −x, at the height `pdHeight1` … `pdHeight3`):

- `IN1`, `IN2`, `IN3`: entries, on the left row (z = −(`aisleWidth` + `positionDepth`) / 2).
- `OUT1`, `OUT2`, `OUT3`: exits, on the right row (z = +(`aisleWidth` + `positionDepth`) / 2).

The aisle uses the ports that are **connected**; a height may have an entry only, an exit only, or both. Ports above
`pdLevels` are ignored. Conveyors at an upper height stand on a raised base, as at the elevator ([[S14 Pallet Elevator]]).

Presets: **Single deep** (defaults) and **Double deep** (`deep = 2`).

## 3. Crane motion

- The crane carries **one pallet**.
- It travels along the aisle and lifts **at the same time**. A move from (x₁, y₁) to (x₂, y₂) takes the **longer** of
  the two times, each with its own speed and ramp ([[T04 Conveyor Kinematics]] §2):
  t = max( t(|x₂ − x₁|, `travelSpeed`, `travelAccel`), t(|y₂ − y₁|, `liftSpeed`, `liftAccel`) ).
- At each end of a move it makes **one fork cycle**: `forkTime` at a station or a front position, `forkTimeDouble` at a
  rear position.
- The pallet changes hands in the **middle** of a fork cycle: the forks have reached out and lifted it off the conveyor
  (or the rack), or lowered it onto the conveyor (or into the rack). In the second half the forks come back; the crane
  moves on when the cycle ends. So a conveyor is free again half a fork cycle before the crane leaves.

## 4. 2D behaviour

### 4.1 Jobs

The crane does one job after another. There are three kinds:

| Job        | Steps                                                                                          |
| ---------- | ---------------------------------------------------------------------------------------------- |
| Storage    | move to the entry station → pick → move to the position → set down                             |
| Retrieval  | move to the position → pick → move to the exit station → set down (waits there if it is full)  |
| Relocation | move to a front position → pick → move to another free position → set down (double deep, §4.4) |

- A **storage request** starts when a pallet has **stopped at the end** of the conveyor connected to an entry, and a free
  position exists. The position is reserved at once.
- With storage and retrieval requests both waiting, the crane **alternates** between them. After setting a pallet down in
  the rack it goes straight to the pallet it retrieves next (a **combined cycle**).
- Among requests of the same kind, the oldest first.
- An idle crane stays where it is, or goes back to the lowest connected station (`dwellPoint = front`).

### 4.2 Where a pallet is stored

- `nearest`: the free position the crane reaches soonest from the entry station (§3). In a double-deep channel the
  **rear** position is filled first. Ties: lower bay, then lower level, then the left row.
- `random`: any free position, drawn with the project's seeded generator; rear before front in a channel.
- **Full aisle:** the pallet waits at the end of its conveyor, which is then blocked, and the line behind it fills up.

### 4.3 When a pallet leaves

- `none`: pallets are stored and never leave.
- `dwell`: each pallet asks to leave `dwellTime` after it was set down in the rack (exactly, or exponentially distributed
  about it). For the initial stock the remaining time is spread evenly between 0 and `dwellTime` (fixed), or drawn
  exponentially.
- `interval`: every `retrievalInterval` (exact or exponential) one order arrives and asks for one pallet that has no order
  yet: the oldest (`fifo`) or any (`random`). An order that finds no such pallet is counted as missed and dropped.
- The exit: the connected `OUT` ports take turns. If the outfeed conveyor has no room at its start, the crane **waits
  blocked** at the station with the pallet on its forks.

### 4.4 Double deep

- A pallet asked for at a **rear** position behind another pallet: the crane first **relocates** the front pallet to the
  nearest free position (never into the same channel), then retrieves the rear one.
- If no free position exists, the **front** pallet leaves instead and the order stays with the rear one.

### 4.5 Start of the run

`initialFill` × `positions` pallets (rounded down) stand in the rack, placed as `storageStrategy` says, as EUR pallets with
a block load of `initialLoadHeight` and `initialLoadMass`. The crane waits at the lowest connected station.

## 5. Statistics

| Key                | Meaning                                                          |
| ------------------ | ---------------------------------------------------------------- |
| `utilization`      | Share of the time the crane moves or works its forks             |
| `blockedShare`     | Share of the time it waits at a full exit with a pallet          |
| `idleShare`        | Share of the time without a job                                  |
| `palletsStored`    | Pallets set down in the rack from an entry                       |
| `palletsRetrieved` | Pallets set down on an exit                                      |
| `relocations`      | Front pallets moved to free a rear one                           |
| `ordersMissed`     | `interval` orders that found no pallet                           |
| `meanCycleTime`    | Mean time of a job, from its first move to its last set-down, s  |
| `combinedShare`    | Share of the retrievals that started straight after a storage    |
| `meanWaitAtEntry`  | Mean time a pallet rests at an entry before the forks take it, s |
| `stock`            | Pallets in the rack now                                          |
| `fillLevel`        | `stock` / `positions`                                            |

Status shown on the plan: `idle`, `travelling`, `picking`, `placing`, `blocked`.

## 6. Validations

| Rule                   | When                                                                                        |
| ---------------------- | ------------------------------------------------------------------------------------------- |
| `noStation`            | No `IN` and no `OUT` port is connected                                                      |
| `retrievalWithoutExit` | `retrievalMode` is not `none` and no `OUT` is connected; pallets stay stored                |
| `exitWithoutRetrieval` | An `OUT` is connected and `retrievalMode` is `none`                                         |
| `stationNotOnChains`   | A station port is connected to something that is not a chain conveyor                       |
| `stationAboveRack`     | A used station height is above the top level                                                |
| `stationLevelNotUsed`  | A port above `pdLevels` is connected; it is ignored                                         |
| `loadTooTall`          | A pallet with its load is taller than `levelPitch` − 0.25 m; it is stored, with a warning   |
| `palletAcrossAisle`    | A pallet arrives with its 1200 mm side along the conveyor; the forks can't take it that way |

## 7. Not included yet

- Stations at the rear end or beside the rack, and roller conveyors as stations.
- Cranes that change aisle, two cranes in an aisle, two pallets per trip, shuttles, more than double deep.
- Storage zones (ABC), stock by article, batches; orders written in IntraScript (`Aisle.retrieve()`), signals.
- The crane's approach offsets, positioning time and fork speeds in detail: `forkTime` holds them all.
- 3D physics: the last step of the plan. The crane carries the pallet guided; see
  [[ADR-0033 Scope Extension High-Bay Warehouse and Stacker Cranes]] §5.

## 8. Reference tests

- **T12 (move):** 30 m along and 10 m up at 3 m/s, 0.5 m/s² and 1 m/s, 0.5 m/s² →
  max(30 / 3 + 3 / 0.5, 10 / 1 + 1 / 0.5) = max(16, 12) = **16 s** (error < 1 %).
- **T13 (single cycle):** `bays` 30, `bayPitch` 1.0, `levelPitch` 2.0, one pallet stored at bay 30, level 6, from the
  station at 0.6 m, crane back at the station: the position is 29.9 m along and 10 m up →
  move max(29.9 / 3 + 6, 12) = 15.97 s; cycle 2 × 15.97 + 2 × 8 = **47.93 s** (error < 1 %).
- Storage: the nearest position is filled first; rear before front in double deep.
- Combined cycle: with a pallet waiting at the entry and one asked for, the crane goes from the storage position straight
  to the retrieval position.
- `dwell` fixed: a pallet set down at time t is asked for at t + `dwellTime`. `interval` with `fifo`: pallets leave in
  the order they were stored.
- Double deep: asking for a rear pallet behind a front one gives one relocation, then the retrieval.
- A full aisle blocks its infeed conveyor; a full outfeed conveyor holds the crane `blocked` with its pallet.
- `noStation`, `retrievalWithoutExit`, `stationNotOnChains`.
- The same seed gives the same run.

## In the three modules

| Module              | This object                                                                                                                                          |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | Family of the library (category storage); placed alone from the Intralogistics tab or in rows by the warehouse wizard; listed in the Simulation tree |
| Discrete simulation | Crane jobs with timed moves and fork cycles (§3–4); the 3D view shows the crane, its lifting carriage, its forks and the pallets in the rack         |
| Physics simulation  | Last step: the same jobs; a pallet on the forks or in the rack is carried guided, and is a physics body again on the outfeed chains                  |
