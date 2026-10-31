---
title: A10 CLI and Scenarios
lang: en
tags: [app, cli, scenarios]
---

🌐 **EN** · [[A10 CLI y escenarios|ES]] · [[A10 CLI und Szenarien|DE]] · [[Home (EN)|Index]]

# A10 CLI and Scenarios

> **Rust platform (2026-10-31):** this note describes the TypeScript reference, which is frozen ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). In Rust, `intracraft-scenarios` reads the same scenario files and `intracraft-cli scenario <id|all>` runs them (phase R4 of [[A19 Rust Implementation Plan]]); BIMCraft's own CLI and MCP server run every command, IntraCraft's included.

## 1. Scenarios (`packages/scenarios`)

A **scenario** is a reproducible experiment with an expected result. Today the package holds the **analytic** scenarios of the plan (T1–T5) and of the lifts (T8, T9) in `scenarios/analytic.json`:

| Id  | Kind            | Inputs (SI)                                           | Expected          |
| --- | --------------- | ----------------------------------------------------- | ----------------- |
| T1  | `profileTime`   | 3 m, 0.25 m/s, 0.5 m/s², no braking                   | 12.25 s           |
| T2  | `throughput`    | 0.25 m/s, 1.2 m pallet, 0.15 m gap                    | 0.18519 pallets/s |
| T3  | `profileTime`   | π/2 rad, 15 °/s, 30 °/s² (in rad)                     | 6.5 s             |
| T4  | `transferCycle` | 1 s lift, 1 s lower, 1 m, 0.2 m/s, 0.5 m/s²           | 7.4 s             |
| T5  | `impactEnergy`  | 1025 kg, 0.25 m/s                                     | 32 J              |
| T8  | `profileTime`   | 0.6 m, 0.05 m/s, 0.2 m/s² up and down (scissor table) | 12.25 s           |
| T9  | `profileTime`   | 4 m, 0.5 m/s, 0.5 m/s² up and down (elevator)         | 9 s               |

Each scenario has a localised title (en/es/de), a unit and **one** tolerance: relative (1 % for the analytic ones) or absolute (for expected values of 0, such as a position error). `runScenario` computes the result and reports `{ actual, expected, error, relativeError, passed }`.

**Layout scenarios** (since 2026-10-30, `scenarios/layouts.json`, `src/flow.ts`):
- A layout scenario runs a saved project file from `scenarios/layouts/` in the 2D mode for a fixed time. It reads one statistic of one object.
- `T1-2D` (`flow-t1`): the transit time of one pallet on a 3 m roller conveyor = 12.25 s.
- `T2-2D` (`flow-t2`): sink throughput of a saturated 12 m line over 8 h ≈ 666.7 pallets/h. The empty start costs about 0.16 %.
- `REF-5` (`reference-all-five`): source → roller → roller turntable → roller → transfer → chain → chain turntable → chain → sink. With one pallet every 30 s, the sink receives 120 pallets/h.
- `LIFT-C` (`demo-elevator-c`): a C elevator takes one pallet a minute from the floor (0.6 m) to a conveyor on a 4 m mezzanine; all 60 pallets/h get through.
- `LIFT-OUT` (`demo-manual-output`): a manual output table hands the pallets to an operator who needs 60 s per pallet; with the 5.05 s run-out the line gives 3600 / 65.05 ≈ 55.3 pallets/h.
- `T10` (`flow-t10`): one pallet rolls down a 6 m gravity line at 0.2 m/s; it reaches the end stop 24 s after it is fully on the line.
- `T11` (`demo-gravity-line`): a saturated gravity line whose operator needs 60 s per pallet delivers 60 pallets/h; the line holds 4 pallets and blocks the feeder.
- `demo-roller-line` and `demo-turntable-split` are examples to open in the editor (File → Open…).

**Physics bench scenarios** (since 2026-10-30, `scenarios/physics.json`, `src/physics.ts`, `src/physics-bench.ts`):

- The bench is a roller section with an optional fixed stop and one MTBH built from a real pallet model and its load ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]). It runs with the Jolt backend at the fixed step of 1/120 s.
- Its parameters (`scenarios/physics-bench.json`) use the same schema as a library definition; each scenario only lists the values that differ from the defaults.
- A scenario runs the bench for a fixed time and measures one value.

| Id         | Measures                                                                  | Expected (tolerance)    | Criterion                                   |
| ---------- | ------------------------------------------------------------------------- | ----------------------- | ------------------------------------------- |
| P1         | Steady pallet speed, 1000 kg on 10 rollers                                | 0.25 m/s (3 %)          | S1-a                                        |
| P2-…       | Largest acceleration the rollers give (steel, rubber, plastic pallet)     | a = μ·g (10 %)          | [[S16 Contact Materials]]                   |
| P3         | Pallet front against a steel stop at 0.25 m/s                             | 0 m (±5 mm)             | S1-c                                        |
| P3-rebound | Backward travel after the impact                                          | 0 m (±10 mm)            | S1-c                                        |
| P4         | Impact energy of 1025 kg at 0.25 m/s                                      | 32.03 J (2 %)           | T5 in 3D                                    |
| P5-…       | Rebound speed / approach speed (steel, polyurethane, rubber)              | 0.1 / 0.2 / 0.35 (10 %) | S16                                         |
| P6         | Earlier tipping at a conveyor end with the load 100 mm forward            | 95 mm (15 %)            | ADR-0015                                    |
| P7         | Same state hash after 60 s in two runs                                    | equal                   | S1-g                                        |
| P8         | EUR pallet across 3 chain strands, plan B: steady speed                   | 0.25 m/s (3 %)          | S1-b                                        |
| P8-jitter  | Spread of the pallet height on the chains                                 | 0 m (±1 mm)             | S1-b                                        |
| P8-planA   | Same on strips of small driven rollers (plan A)                           | 0.25 m/s (3 %)          | S1-b                                        |
| P9         | Lateral error after guides with a funnel (50 mm, 5°, polyethylene strips) | 0 m (±10 mm)            | S1-d                                        |
| P9-yaw     | Angle error after the same guides                                         | 0° (±1°)                | S1-d                                        |
| P9-steel   | Angle error with painted steel guides, pallet turned 3°                   | 0° (±1°)                | S1-d, [[ADR-0002 Physics Engine Candidate]] |
| P10        | Pallet shift on a platform turning 90° (T3 profile)                       | 0 m (±5 mm)             | S1-e                                        |
| P10-fast   | Same at 45 °/s and 120 °/s²                                               | 0 m (±5 mm)             | S1-e                                        |
| P11        | Mean step time, 20 conveyors and 10 pallets (wall clock)                  | ≤ 4.17 ms               | S1-f                                        |

Golden result files in `test_data/` will follow when the outputs grow beyond one number per scenario.

## 2. CLI (`apps/cli`)

```text
pnpm run scenario all     # run every scenario
pnpm run scenario t3      # run one (case-insensitive)
```

Output example:

```text
PASS T1 Roller conveyor: 3 m from rest, 0.25 m/s, 0.5 m/s², no braking: 12.2500 s (expected 12.25 s, error 0.000 %)
…
8/8 scenarios passed
```

Exit codes: `0` all passed · `1` at least one failed · `2` unknown scenario id. The CLI writes through the project logger (no `console.log`).

## Related

[[A11 Testing and Validation]] · [[T04 Conveyor Kinematics]] · [[A14 API Reference]]
