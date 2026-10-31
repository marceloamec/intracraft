---
title: A14 API Reference
lang: en
tags: [app, api, reference]
---

🌐 **EN** · [[A14 Referencia de API|ES]] · [[A14 API-Referenz|DE]] · [[Home (EN)|Index]]

# A14 API Reference (state on 2026-10-29)

> **Rust platform (2026-10-31):** this note describes the TypeScript reference, which is frozen ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). The Rust API is documented in the crates themselves (`cargo doc` in `platform/`); the command list is BIMCraft's `catalog()`, IntraCraft's commands included.

All values in SI units unless stated. Every package is imported as `@intracraft/<package>`.

## core-model

| Export                                                                                                                      | Signature / shape                              | Purpose                                                               |
| --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------- |
| `MM_PER_M`, `GRAVITY`                                                                                                       | `1000`, `9.81`                                 | Constants                                                             |
| `mmToM`, `mToMm`                                                                                                            | `(x: number) => number`                        | Length conversion at UI/file edges                                    |
| `degToRad`, `radToDeg`                                                                                                      | `(x: number) => number`                        | Angle conversion                                                      |
| `SimTime`                                                                                                                   | `number` (integer µs)                          | Simulation time type                                                  |
| `US_PER_S`, `US_PER_MS`                                                                                                     | `1_000_000`, `1_000`                           | Constants                                                             |
| `secondsToSimTime`, `msToSimTime`                                                                                           | `(x: number) => SimTime` (rounded)             | Convert durations to integer µs                                       |
| `simTimeToSeconds`                                                                                                          | `(t: SimTime) => number`                       | Back to seconds                                                       |
| `createRng`                                                                                                                 | `(seed, stream = 54) => Rng`                   | PCG32 generator: `nextUint32()`, `next()` ∈ [0,1), `uniform(min,max)` |
| `createLogger`                                                                                                              | `(sink: LogSink, minLevel = 'info') => Logger` | `debug/info/warn/error(message, data?)`                               |
| `createMemorySink`                                                                                                          | `() => LogSink & { entries }`                  | Collect log entries (tests)                                           |
| `SCHEMA_VERSION`                                                                                                            | `1`                                            | Current project-file version                                          |
| `projectSchema`, `layoutSchema`, `instanceSchema`, `connectionSchema`, `portRefSchema`, `vec3Schema`, `localizedTextSchema` | Zod schemas                                    | Validate project files                                                |
| `Project`, `Layout`, `Instance`, `Connection`, `Vec3`, `LocalizedText`                                                      | Types inferred from the schemas                | —                                                                     |
| `attributesSchema`, `Attributes`                                                                                            | Zod schema / type                              | Free instance attributes                                              |
| `localize`                                                                                                                  | `(text, language) => string`                   | Text in a language, English fallback                                  |
| `readProjectFile`, `writeProjectFile`, `projectFileName`, `PROJECT_FILE_EXTENSION`                                          | `(text) => { ok, project } \| { ok, error }` … | Project file read (with migrations) and write                         |

## library

| Export                                                                        | Signature / shape                                                 | Purpose                                              |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------- | ---------------------------------------------------- |
| `componentDefinitionSchema`, `parameterSchema`, `portSchema`, `affectsSchema` | Zod schemas                                                       | Validate definitions ([[A05 Component Library]])     |
| `ComponentDefinition`, `ParameterDefinition`, `PortDefinition`                | Types                                                             | —                                                    |
| `DefinitionRegistry`                                                          | `register(json)`, `has(id)`, `get(id)`, `list()`                  | Holds validated definitions; rejects duplicates      |
| `createBuiltInRegistry`                                                       | `() => DefinitionRegistry`                                        | Registry with the five MVP conveyors                 |
| `instanceParams`, `derivedValues`                                             | `(def, params) => values`                                         | Defaults + instance values; derived values           |
| `resolvePorts`, `worldPorts`, `rotateY`                                       | `(def, params)`, `(instance, def)`, `(v, yaw)`                    | Port positions in local / world coordinates          |
| `snapPose`                                                                    | `(candidate, layout, registry, radius = 0.3) => SnapResult`       | Snap to a facing free port, else the 100 mm grid     |
| `deriveConnections`, `connectionIssues`                                       | `(layout, registry, instanceId?)`                                 | Connections from geometry; height/direction warnings |
| `validateInstance`                                                            | `(def, params) => ValidationIssue[]`                              | Runs the definition's validation rules               |
| `defaultParams`                                                               | `(def) => Record<string, number \| boolean \| string>`            | Non-derived default values                           |
| `min3RollersUnderPallet`                                                      | `(pitch, palletSupportLength) => ValidationIssue[]`               | pitch ≤ side / 3                                     |
| `usableWidthFitsPallet`                                                       | `(usableWidth, palletWidth, guideClearance) => ValidationIssue[]` | width ≥ pallet + 2 × clearance                       |
| `chainFitsBetweenRollers`                                                     | `(pitch, rollerDiameter, chainWidth) => ValidationIssue[]`        | gap ≥ chain + 10 mm                                  |
| `PALLET_TYPES`                                                                | `PalletType[]`                                                    | EUR, 1200 × 1000, GMA with occupancy profiles        |
| `FRICTION_PAIRS`                                                              | `FrictionPair[]`                                                  | Material pairs with μ and usual range                |

## geometry

| Export                   | Signature                                                | Purpose                                                   |
| ------------------------ | -------------------------------------------------------- | --------------------------------------------------------- |
| `rollerLayout`           | `(length, pitch, diameter) => { count, positions, gap }` | Centred roller axis positions; throws if diameter ≥ pitch |
| `minRollersUnder`        | `(supportLength, pitch) => number`                       | Worst-case number of rollers under a pallet side          |
| `chainLayout`            | `(width, count, edgeInset = 0.1) => number[]`            | Lateral chain strand positions                            |
| `transferChainPositions` | `(length, pitch, diameter, count) => number[]`           | Transfer chains in the gaps between rollers               |

## commands

| Export        | Signature                                                                                                           | Purpose                                                                                                                                         |
| ------------- | ------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `Session`     | `execute(name, input, actor)`, `undo(actor)`, `redo(actor)`, `replay(entry)`, `layout`, `log`, `canUndo`, `canRedo` | Editing session ([[ADR-0006 Command Package and Geometric Port Connections]])                                                                   |
| `COMMANDS`    | `layout.addInstance`, `.move`, `.setParams`, `.setAttributes`, `.remove`, `.clear`, `.load`                         | Command catalogue with Zod input schemas; `layout.move` may also change parameters (a sensor mounted on another conveyor) in the same undo step |
| `checkParams` | `(def, values) => ValidationIssue[]`                                                                                | Type, range, choice and option checks                                                                                                           |

## sim-des

| Export                       | Signature                                                                                                                | Purpose                                                               |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------- |
| `EventQueue<T>`              | `push(time, priority, payload)`, `pop()`, `peek()`, `size`                                                               | Min-heap by (time, priority, seq); integer times only                 |
| `compareEvents`              | `(a, b) => number`                                                                                                       | Total order used by the queue                                         |
| `DesSimulator`               | `now`, `on(type, handler)`, `schedule(delay, event, priority?) → ticket`, `cancel(ticket)`, `runUntil(endTime)`, `trace` | Next-event loop                                                       |
| `profileTime`                | `({ distance, maxSpeed, accel, decel? }) => seconds`                                                                     | Trapezoidal/triangular move time; no `decel` = arrive at cruise speed |
| `transferCycleTime`          | `({ liftTime, lowerTime, distance, speed, accel, settleTime? }) => seconds`                                              | Transfer-table cycle (T4)                                             |
| `maxThroughput`              | `(speed, palletLength, gap) => pallets/s`                                                                                | $v/(L+g)$ (T2)                                                        |
| `palletCapacity`             | `(length, palletLength, gap) => integer`                                                                                 | $\lfloor \text{length}/(L+g) \rfloor$                                 |
| `impactEnergy`               | `(mass, speed) => J`                                                                                                     | $\frac12 m v^2$ (T5)                                                  |
| `positionAt`, `velocityAt`   | `(segment, t) => number`                                                                                                 | Evaluate a kinematic segment                                          |
| `timeToReach`                | `(segment, s) => t \| undefined`                                                                                         | Earliest time a segment reaches position *s*                          |
| `DesSimulator.nextEventTime` | `() => SimTime \| undefined`                                                                                             | Next pending event (editor "Next event" button)                       |
| `buildFlowNetwork`           | `(layout, registry) => { nodes, issues }`                                                                                | Layout → 2D nodes (line, source, sink, inert) and warnings            |
| `FlowSimulation`             | `new (network, { seed })`, `runUntil(t)`, `nextEventTime()`, `snapshot()`, `statistics()`, `issues`, `now`               | 2D run ([[ADR-0007 2D Flow Model Without Accumulation]])              |

## sim-physics

| Export                                                      | Signature                                                                               | Purpose                              |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------ |
| `PhysicsBackend`                                            | interface ([[A07 Physics Engine Wrapper]])                                              | Contract for the Jolt/Rapier adapter |
| `BodyDesc`, `Shape`, `Pose`, `Quat`, `RayHit`, `BodyHandle` | Types                                                                                   | Body and query descriptions          |
| `FixedStepClock`                                            | `new FixedStepClock(dt = 1/120, maxSteps = 8)`, `advance(frameTime) → { steps, alpha }` | Fixed-step accumulator               |
| `DEFAULT_PHYSICS_DT`, `DEFAULT_MAX_STEPS_PER_FRAME`         | `1/120`, `8`                                                                            | Constants                            |
| `rollerAngularVelocity`                                     | `(surfaceSpeed, diameter) => rad/s`                                                     | ω = v / r                            |
| `maxTractionAccel`                                          | `(mu) => m/s²`                                                                          | μ·g                                  |
| `turntableRequiredFriction`                                 | `(angularAccel, angularSpeed, radius) => μ`                                             | $\sqrt{a_t^2 + a_c^2}/g$             |
| `turntableRequiredTorque`                                   | `(inertia, angularAccel, frictionTorque) => N·m`                                        | I·α + friction                       |
| `sweptRadius`                                               | `(length, width, clearance) => m`                                                       | ½√(L² + W²) + clearance              |
| `boxYawInertia`                                             | `(mass, length, width) => kg·m²`                                                        | m(a² + b²)/12                        |

## control

| Export                                             | Signature                                     | Purpose                                       |
| -------------------------------------------------- | --------------------------------------------- | --------------------------------------------- |
| `Rule`, `Condition`, `SignalWrite`, `SignalValues` | Types                                         | Rule model ([[A08 Controller]])               |
| `evaluateRules`                                    | `(rules, previous, current) => SignalWrite[]` | Pure If/Then evaluation with levels and edges |

## scenarios

| Export           | Signature                       | Purpose                                           |
| ---------------- | ------------------------------- | ------------------------------------------------- |
| `SCENARIOS`      | `Scenario[]`                    | T1–T5 from `scenarios/analytic.json`              |
| `findScenario`   | `(id) => Scenario \| undefined` | Case-insensitive lookup                           |
| `runScenario`    | `(scenario) => ScenarioResult`  | `{ id, actual, expected, relativeError, passed }` |
| `scenarioSchema` | Zod schema                      | Validates the JSON file                           |

## Related

[[A03 Monorepo and Stack]] · [[A06 DES Engine]] · [[A07 Physics Engine Wrapper]]
