---
title: A11 Testing and Validation
lang: en
tags: [app, testing, validation]
---

🌐 **EN** · [[A11 Pruebas y validación|ES]] · [[A11 Tests und Validierung|DE]] · [[Home (EN)|Index]]

# A11 Testing and Validation

> **Rust platform (2026-10-31):** this note describes the TypeScript reference, which is frozen ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). In Rust the same expected values are checked by `cargo test --workspace` in `platform/`; the TypeScript results stay the oracle.

**Test first, then implement.** Expected results come from the specification, never from the implementation.

## 1. Test layers

1. **Unit** — units, kinematics, event queue, parameter validation.
2. **2D scenarios with analytic results** (§2).
3. **Physics scenarios** (§3).
4. **Consistency between modes** (§4).
5. **Regression with golden results** and **determinism** (§5).
6. **Comparison with known reference models** (§6).

## 2. Analytic tests of the 2D mode (error < 1 %)

| Id  | Scenario                                                  | Expected                                | Status              |
| --- | --------------------------------------------------------- | --------------------------------------- | ------------------- |
| T1  | One pallet, 3 m from rest, 0.25 m/s, 0.5 m/s², no braking | $D/v + v/2a$ = 12.25 s                  | ✅ formula          |
| T2  | Saturation, EUR long side (1.2 m), gap 0.15 m             | $v/(L+g)$ ≈ 0.185 pallets/s ≈ 667/h     | ✅ formula          |
| T3  | 90° turn, 15 °/s, 30 °/s²                                 | 6.5 s                                   | ✅ formula          |
| T4  | Transfer: 1 m, 0.2 m/s, 0.5 m/s², 1 s lift and lower      | 7.4 s                                   | ✅ formula          |
| T5  | Impact energy 1025 kg at 0.25 m/s                         | 32 J                                    | ✅ formula          |
| T6  | Sensor at 2.0 m, high beam                                | Rising at front, falling at tail; 4.8 s | ⏳ needs DES tracks |

"✅ formula" = the closed-form functions pass; the full DES run of each layout comes in phases 3–8.

## 3. Physics scenarios

| Id  | Scenario     | Criterion                                                  |
| --- | ------------ | ---------------------------------------------------------- |
| P1  | Stop         | Front at ±5 mm; rebound < 10 mm                            |
| P2  | Impact       | Speed and energy recorded; warning above `maxImpactEnergy` |
| P3  | Funnel guide | 50 mm and 5° corrected to ±10 mm and ±1°                   |
| P4  | 90° turn     | Pallet displacement < 5 mm; no sliding with μ ≥ 0.04       |
| P5  | Transfer     | Lifted 40 ± 5 mm, moved 1 m ± 10 mm, rotated < 1°          |
| P6  | Sensors      | 3D and 2D edges within ±50 ms                              |

Spike S1 criteria a–g are in [[T15 Physics Engine Research]] and [[ADR-0002 Physics Engine Candidate]].

## 4. Consistency between modes

Same scenario in both modes: transit times ±5 %, sensor edges ±50 ms. Larger differences are findings ([[T16 Hybrid Consistency and Calibration]]).

## 5. Regression and determinism

- Golden results in `test_data/` per scenario. Any change must be explained in the commit message and changelog. Never edit them to make a test pass.
- Two runs with the same seed must give the same hash of the event log (2D) and of the body states (3D). Already tested today: identical event traces (`simulator.test.ts`) and identical RNG sequences (`rng.test.ts`).

## 6. Comparison with reference models

Pick three models the team knows well (simple flow, accumulation, turntable/transfer), rebuild them in 2D, compare throughput, mean cycle time and occupancy. Document differences instead of forcing them; only observable results are compared (clean-room rule).

## 7. Current test inventory (2026-10-28)

`pnpm test` → **11 files, 62 tests, all passing** (2026-10-29); `pnpm lint` (ESLint + type-check) clean; `pnpm run scenario all` → 5/5.

| Package     | Test file(s)                                     |
| ----------- | ------------------------------------------------ |
| core-model  | `units.test.ts`, `rng.test.ts`, `schema.test.ts` |
| sim-des     | `kinematics.test.ts`, `simulator.test.ts`        |
| sim-physics | `mechanics.test.ts`                              |
| control     | `rules.test.ts`                                  |
| geometry    | `rollers.test.ts`                                |
| library     | `library.test.ts`                                |
| scenarios   | `analytic.test.ts`                               |
| editor      | `i18n/strings.test.ts`                           |

## Related

[[A10 CLI and Scenarios]] · [[A12 Roadmap]] · [[T08 Statistics and Output Analysis]]
