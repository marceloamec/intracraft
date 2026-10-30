---
title: A10 CLI and Scenarios
lang: en
tags: [app, cli, scenarios]
---

🌐 **EN** · [[A10 CLI y escenarios|ES]] · [[A10 CLI und Szenarien|DE]] · [[Home (EN)|Index]]

# A10 CLI and Scenarios

## 1. Scenarios (`packages/scenarios`)

A **scenario** is a reproducible experiment with an expected result. Today the package holds the five **analytic** scenarios of the plan in `scenarios/analytic.json`:

| Id  | Kind            | Inputs (SI)                                 | Expected          |
| --- | --------------- | ------------------------------------------- | ----------------- |
| T1  | `profileTime`   | 3 m, 0.25 m/s, 0.5 m/s², no braking         | 12.25 s           |
| T2  | `throughput`    | 0.25 m/s, 1.2 m pallet, 0.15 m gap          | 0.18519 pallets/s |
| T3  | `profileTime`   | π/2 rad, 15 °/s, 30 °/s² (in rad)           | 6.5 s             |
| T4  | `transferCycle` | 1 s lift, 1 s lower, 1 m, 0.2 m/s, 0.5 m/s² | 7.4 s             |
| T5  | `impactEnergy`  | 1025 kg, 0.25 m/s                           | 32 J              |

Each scenario has a localised title (en/es/de), a unit and a relative tolerance (1 %). `runScenario` computes the result with the `sim-des` formulas and reports `{ actual, expected, relativeError, passed }`.

**Layout scenarios** (since 2026-10-30, `scenarios/layouts.json`, `src/flow.ts`):
- A layout scenario runs a saved project file from `scenarios/layouts/` in the 2D mode for a fixed time. It reads one statistic of one object.
- `T1-2D` (`flow-t1`): the transit time of one pallet on a 3 m roller conveyor = 12.25 s.
- `T2-2D` (`flow-t2`): sink throughput of a saturated 12 m line over 8 h ≈ 666.7 pallets/h. The empty start costs about 0.16 %.
- `REF-5` (`reference-all-five`): source → roller → roller turntable → roller → transfer → chain → chain turntable → chain → sink. With one pallet every 30 s, the sink receives 120 pallets/h.
- `demo-roller-line` and `demo-turntable-split` are examples to open in the editor (File → Open…).

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
