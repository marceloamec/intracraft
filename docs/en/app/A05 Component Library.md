---
title: A05 Component Library
lang: en
tags: [app, library]
---

🌐 **EN** · [[A05 Librería de componentes|ES]] · [[A05 Komponentenbibliothek|DE]] · [[Home (EN)|Index]]

# A05 Component Library

## 1. Concepts

| Concept    | Meaning                                                                                       |
| ---------- | --------------------------------------------------------------------------------------------- |
| Definition | A component **type**: parameters, ports, signals, geometry, behaviours. Lives in the library. |
| Instance   | One use of a definition in a layout, with its own parameter values.                           |
| Preset     | Named set of values for a definition (e.g. "Heavy 1000 kg, 0.25 m/s").                        |
| Origin     | *built-in* (read-only), *user* (editable, shareable), *project* (saved with the layout).      |

Definitions carry a semantic version; a layout pins the version it uses.

## 2. What a definition contains

Implemented in `packages/library/src/definition-schema.ts`:

| Field                 | Meaning                                                                                                                                                                                                                      |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id`                  | `category.name`, e.g. `conveyor.roller`                                                                                                                                                                                      |
| `version`             | Semantic version `x.y.z`                                                                                                                                                                                                     |
| `category`            | conveyors, sensors, mechanics, flow, pallets, control                                                                                                                                                                        |
| `name`, `description` | Localised text `{ en, es, de }`                                                                                                                                                                                              |
| `parameters`          | List of parameter definitions (§3)                                                                                                                                                                                           |
| `ports`               | Id, direction (in/out/inout), `facing` (+x, −x, +z, −z), local position per axis: number, parameter key, or list of terms `{param, factor}` / numbers added up ([[ADR-0006 Command Package and Geometric Port Connections]]) |
| `signals`             | Input and output signal names                                                                                                                                                                                                |
| `statistics`          | Names of the statistics the component publishes                                                                                                                                                                              |
| `geometry`            | Id of the geometry generator                                                                                                                                                                                                 |
| `behaviors`           | `{ des, physics }` ids of the registered behaviours                                                                                                                                                                          |
| `validationRules`     | Ids of validation rules to run                                                                                                                                                                                               |
| `presets`             | Named value sets                                                                                                                                                                                                             |

## 3. What a parameter contains

| Field                           | Meaning                                                                |
| ------------------------------- | ---------------------------------------------------------------------- |
| `key`                           | Identifier (English camelCase, e.g. `rollerPitch`)                     |
| `label`, `description`          | Localised `{ en, es, de }`                                             |
| `type`                          | number, integer, boolean, enum, text                                   |
| `unit`                          | **Stored** SI unit (`m`, `m/s`, `kg`…)                                 |
| `displayUnit`                   | Unit shown in the editor (`mm`, `°`…)                                  |
| `default`, `min`, `max`, `step` | Value and range, in the stored unit                                    |
| `options`                       | Values of an enum                                                      |
| `choices`                       | Allowed values of a number parameter (dropdown), e.g. roller diameters |
| `group`                         | Property-panel group (geometry, motion, rollers…)                      |
| `advanced`                      | Hidden by default                                                      |
| `affects`                       | `2d`, `3d`, `both` or `geometry`                                       |
| `derived`, `formula`            | Read-only computed value, e.g. `floor(length / rollerPitch)`           |
| `dependsOn`                     | Conditional visibility, e.g. `drive=zoned`                             |

The property panel is **generated** from this schema: no hand-written forms.

## 4. Data and code

JSON describes **what** a component is; TypeScript behaviours registered by id implement **how** it behaves. `createBuiltInRegistry()` loads and validates every built-in JSON definition at start-up; an invalid definition fails loudly. `defaultParams(definition)` returns the non-derived defaults.

## 5. Validation rules (implemented)

| Rule                      | Check                                             | Example                  |
| ------------------------- | ------------------------------------------------- | ------------------------ |
| `min3RollersUnderPallet`  | pitch ≤ supported side / 3                        | 206.3 mm under 1200 mm ✓ |
| `usableWidthFitsPallet`   | usable width ≥ pallet width + 2 × guide clearance | 1000 ≥ 800 + 50 ✓        |
| `chainFitsBetweenRollers` | pitch − diameter ≥ chain width + 10 mm            | 117.3 ≥ 40 + 10 ✓        |

Hooked up per instance by `validateInstance` (checked against the EUR pallet in the orientation of `palletSideAlongTravel`); the properties panel shows the warnings. Built-in definitions: `conveyor.roller`, `conveyor.chain`, `conveyor.chain_turntable`, `conveyor.roller_turntable`, `conveyor.transfer_rc` (defaults indicative). Derived values (`rollerCount`, `sweptRadius`) are computed by `derivedValues`.

Planned: turntable swept radius vs neighbours, impact speed vs limit, port height tolerance, zone length ≥ pallet + 100 mm.

## 6. MVP library contents and status

| Category             | Components                                                                  | Status (2026-10-28)                          |
| -------------------- | --------------------------------------------------------------------------- | -------------------------------------------- |
| Conveyors            | Roller, chain, chain turntable, roller turntable, roller→chain transfer     | Roller: definition JSON; others: spec only   |
| Conveyors (ADR-0012) | Manual lift table, scissor lift table, pallet elevator, gravity roller line | Spec only (S12–S15)                          |
| Sensors              | Diffuse, retroreflective, through-beam photoeye                             | Spec only                                    |
| Mechanics            | Fixed stop, retractable stop, side guide (with funnel)                      | Spec only                                    |
| Flow                 | Source, sink                                                                | Spec only                                    |
| Pallets              | EUR, 1200 × 1000, GMA, custom                                               | EUR, 1200 × 1000, GMA in `pallet-types.json` |
| Materials            | Friction pair table                                                         | `friction-pairs.json`                        |
| Control              | Rule controller and templates                                               | Rule evaluator + ZPA test template           |

## 7. Definition of done for a library object

1. Specification in `docs/<lang>/specs/` (three languages).
2. Definition JSON with parameters, ranges, derived values and validations.
3. Parametric geometry and 2D symbol.
4. 2D behaviour with analytic tests.
5. 3D behaviour with physics tests.
6. Control template.
7. Reference scenario in `packages/scenarios`.
8. Parameter documentation generated from the definition.

## Related

[[A04 Data Model and Project File]] · [[S01 Roller Conveyor]] · [[S10 Pallet Types]] · [[ADR-0003 Trilingual Documentation and English Identifiers]]
