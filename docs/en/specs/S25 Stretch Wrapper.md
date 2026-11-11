---
title: S25 Stretch Wrapper
lang: en
tags: [spec, flow, station, wrapper]
library-id: flow.stretch_wrapper
status: built
updated: 2026-11-11
---

# S25 Stretch Wrapper (`flow.stretch_wrapper`)

A turntable stretch wrapper in a pallet line: a round rotating table with a roller deck, a mast beside it and a film
carriage that travels up and down the mast. **One pallet at a time** runs in, stops centred, is turned while the film is
laid round its load, and runs out wrapped. It sits in line between two roller conveyors.

It is a [[S11 Process Station]] whose processing time is **computed from a wrap programme** instead of drawn, and whose
table **turns** during the processing. Scope and design decisions: [[ADR-0032 Scope Extension Stretch Wrapper]].

Values are **indicative**, not manufacturer data.

## 1. Parameters

| Key                     | Unit  | Default | Range       | Description                                                                |
| ----------------------- | ----- | ------- | ----------- | -------------------------------------------------------------------------- |
| `length`                | m     | 1.8     | 1.5 … 2.4   | Diameter of the rotating table and distance between the ports              |
| `usableWidth`           | m     | 1.0     | 0.8 … 1.4   | Roller length at the centre of the table                                   |
| `surfaceHeight`         | m     | 0.6     | 0.2 … 1.5   | Height of the conveying surface and of both ports; match the conveyors     |
| `palletSideAlongTravel` | enum  | long    | long, short | Pallet side that travels first                                             |
| `nominalSpeed`          | m/s   | 0.25    | 0.05 … 1    | Roller speed while moving a pallet in or out                               |
| `accel`                 | m/s²  | 0.5     | 0.05 … 2    | Acceleration and deceleration of the rollers                               |
| `turnSpeed`             | rpm   | 12      | 4 … 20      | Turning speed of the table while wrapping                                  |
| `turnAccel`             | °/s²  | 36      | 5 … 180     | Angular acceleration and deceleration: soft start and stop                 |
| `bottomWraps`           | turns | 2       | 0 … 10      | Reinforcing turns at the bottom, before the carriage rises                 |
| `topWraps`              | turns | 2       | 0 … 10      | Reinforcing turns at the top                                               |
| `filmWidth`             | m     | 0.5     | 0.25 … 1.0  | Width of the film roll                                                     |
| `overlap`               | %     | 50      | 0 … 90      | Share of the film width that each turn lays over the previous one          |
| `passes`                | enum  | upDown  | up, upDown  | The carriage wraps going up only, or going up and coming down              |
| `attachTime`            | s     | 3       | 0 … 60      | Fixed time to clamp the film before turning                                |
| `cutTime`               | s     | 4       | 0 … 60      | Fixed time to cut and seal the film after turning                          |
| `maxLoadHeight`         | m     | 2.0     | 0.5 … 2.6   | Highest load the carriage reaches (mast height); above it, no film is laid |
| `loadFriction`          | —     | 0.3     | 0.1 … 1.0   | Advanced. Weakest friction inside the load, for `loadMayShift` (§6)        |

Ports: `IN` at x = 0 facing −x, `OUT` at x = `length` facing +x, both at `surfaceHeight`. The table's centre is at
x = `length` / 2.

**Rollers inside the circle.** The table is round, so that it can turn between two conveyors without sweeping into them.
A roller at distance d from the centre is as long as the chord there allows:
min(`usableWidth`, 2 · √(R² − d²)), with R = `length` / 2. The pallet's corners stay inside the circle too (`tableTooSmall`).

## 2. The wrap programme

One function gives the turns and the times of a pallet from its load height. Both simulation modules use it.

| Quantity          | Formula                                                                                  |
| ----------------- | ---------------------------------------------------------------------------------------- |
| Advance per turn  | a = `filmWidth` × (1 − `overlap`)                                                        |
| Wrapped height    | H = min(load height, `maxLoadHeight`)                                                    |
| Climbing turns    | c = ⌈ max(0, H − `filmWidth`) / a ⌉                                                      |
| Turns of a pallet | N = `bottomWraps` + `topWraps` + c × (1 for `up`, 2 for `upDown`), at least 1            |
| Turning time      | Rest to rest over N × 360° at `turnSpeed` with `turnAccel` ([[T04 Conveyor Kinematics]]) |
| Processing time   | `attachTime` + turning time + `cutTime`                                                  |
| Film length       | N × perimeter of the pallet's footprint (pre-stretch not counted)                        |
| Carriage speed    | a × `turnSpeed` (m per turn × turns per second)                                          |

- N is a whole number, so the table always stops **in line with the conveyors**, as it started.
- A pallet **without a load** is not wrapped: processing time 0, it runs straight through.
- A load higher than `maxLoadHeight` is wrapped up to that height and counted in `tooTallPallets`.

With the defaults and a load 1.2 m high: a = 0.25 m, c = ⌈0.7 / 0.25⌉ = 3, N = 2 + 2 + 3 × 2 = **10 turns**; turning time
3600 / 72 + 72 / 36 = **52 s**; processing time 3 + 52 + 4 = **59 s**. With the pallet's entry and exit that is about one
pallet a minute.

## 3. 2D behaviour

The cycle of [[S11 Process Station]], never more than one pallet on the table:

1. **Receive.** The pallet enters from `IN` and is centred on the table.
2. **Wrap.** When centred and stopped: `attachTime`, then the table turns N turns with the pallet on it, then `cutTime`.
   The pallet's heading follows the table's angle; the object's state is `processing`.
3. **Discharge.** The rollers run the pallet out through `OUT`. If the next object cannot take it, the pallet waits and
   the station is **blocked**.
4. **Free.** When the table is empty the cycle ends.

A pallet that leaves is **wrapped**: the load unit carries the mark `wrapped`, and the views draw its load in film.

There are no random draws: the same pallets give the same times.

## 4. 3D behaviour

- The table is a **kinematic platform** that turns about its vertical axis along the same angular profile as in 2D, with
  its rollers on it ([[T13 Turntables and Transfer Lifts]]). During the turn the rollers are stopped; the pallet is held
  **by friction alone**, as on [[S04 Roller Turntable]].
- The film is **not simulated as a material**. It is drawn: the carriage travels on the mast and the load changes to its
  wrapped look when the cycle ends.
- The load is one rigid body with its pallet, as every load today
  ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]). When `loose` loads exist
  ([[ADR-0031 Asset Library Load Units and Loose Loads]], step L5 of [[A21 Asset Library and Load Units Plan]]), the
  wrapper is where a `loose` unit becomes `rigid`: its items can shift while the table turns, and are joined into one body
  when the film is cut. That step amends this section.

## 5. Statistics

Those of [[S11 Process Station]] (`utilization`, `blockedShare`, `idleShare`, `palletsProcessed`, `meanProcessingTime`,
`meanCycleTime`), plus:

| Key              | Meaning                                            |
| ---------------- | -------------------------------------------------- |
| `meanTurns`      | Mean turns per wrapped pallet                      |
| `filmUsed`       | Film laid since the start of the run (m)           |
| `tooTallPallets` | Pallets whose load was higher than `maxLoadHeight` |

`utilization` counts the whole processing time (attach, turn and cut).

## 6. Validations

| Rule            | When                                                                                           |
| --------------- | ---------------------------------------------------------------------------------------------- |
| `noExit`        | `OUT` is not connected                                                                         |
| `tableTooSmall` | `length` is below the diagonal of the EUR pallet + 0.1 m (1.542 m)                             |
| `loadMayShift`  | Warning: the acceleration at the pallet's corner exceeds what friction between layers can hold |

`loadMayShift`: with r half the diagonal of the EUR pallet (0.721 m), ω = `turnSpeed` and α = `turnAccel` in rad, the
warning appears when √((ω² r)² + (α r)²) > `loadFriction` × 9.81. With the defaults that is above about 19 rpm. It is a
warning by formula: the run itself does not move the load (§4).

The default of `loadFriction`, 0.3, is the lowest packaging pair of [[S16 Contact Materials]] (plastic on plastic;
cardboard on wood is 0.35 and cardboard on cardboard 0.4). It is a cautious value, because a load unit does not yet say
what its items are made of. When load recipes name their items ([[A21 Asset Library and Load Units Plan]]), the value
comes from the pair of the unit's items and the parameter only overrides it.

## 7. Not included yet

Signals (`enable` input, `working` output), film breaks and roll changes (failures are outside the MVP), top sheets,
pre-stretch, rotating-arm and ring wrappers (the pallet stands still), an exit at 90°, strapping, and loads that shift in
the physics run (§4).

## 8. Reference tests

Expected values come from §2 and from [[T04 Conveyor Kinematics]], not from a run.

- **W1:** one pallet with a load 1.2 m high, defaults: 10 turns, processing time 59 s, error < 1 %; it leaves `wrapped`,
  and the table's angle is back at 0.
- **W2:** saturated source: steady throughput = 1 / (centring + 59 s + discharge), error < 1 %.
- **W3:** `passes` = `up`: 7 turns, processing time 3 + (2520 / 72 + 2) + 4 = 44 s. A pallet without a load: processing
  time 0, not `wrapped`.
- **W4:** `loadMayShift` appears at 20 rpm and not at 12 rpm (defaults otherwise).
- **W5 (3D):** cycle time within 1 % of the 2D one; after the turn the pallet's centre is within 10 mm of the table's
  centre and its heading within 1° of the line, held by friction alone.
- Shares sum to 1; the same layout and seed give the same result.

## In the three modules

| Module              | This object                                                                                                                                                                                      |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Design              | Family of the library (category flow): round base, roller table, mast, film carriage and roll; ports IN and OUT; placed from the Project Browser, listed in the Simulation tree and in schedules |
| Discrete simulation | Station cycle with the wrap programme's time; the pallet turns with the table; statistics of the station and of the film                                                                         |
| Physics simulation  | A kinematic table that turns with the pallet on it, held by friction; the same cycle as in 2D                                                                                                    |

**Rust status:** built 2026-11-11, Rust platform only (the TypeScript reference is frozen and never had it).

- Definition: `platform/sim/library/definitions/stretch-wrapper.json`; validations `tableTooSmall` and `loadMayShift`.
- The wrap programme: `intracraft_des::flow::wrap`, one function for both runs.
- 2D: a process station (`Kind::Process`) with the programme; W1 to W4 pass.
- 3D: the process station's platform turns along the programme; its rollers are centred on the table, because the
  engines turn a kinematic body about its centre of mass. W5 passes on Rapier and on Avian: the pallet stays within
  2 mm of the table's centre and 0.2° of the line through the ten turns.
- Example installation: `examples/layouts/stretch-wrapper-line.intracraft.json`. A source with loose cartons there is
  a catalogue box (`loadKind` = `asset`), one layer 0.51 m high: 6 turns.

## Related

[[S11 Process Station]] · [[S04 Roller Turntable]] · [[S01 Roller Conveyor]] · [[T04 Conveyor Kinematics]] ·
[[T13 Turntables and Transfer Lifts]] · [[ADR-0032 Scope Extension Stretch Wrapper]] ·
[[ADR-0031 Asset Library Load Units and Loose Loads]]
