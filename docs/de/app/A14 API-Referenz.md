---
title: A14 API-Referenz
lang: de
tags: [app, api, reference]
---

🌐 [[A14 API Reference|EN]] · [[A14 Referencia de API|ES]] · **DE** · [[Startseite (DE)|Index]]

# A14 API-Referenz (Stand 28.10.2026)

Alle Werte in SI-Einheiten, sofern nicht anders angegeben. Jedes Paket wird als `@intracraft/<paket>` importiert.

## core-model

| Export                                                                                                                      | Signatur / Form                                | Zweck                                                                 |
| --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------- |
| `MM_PER_M`, `GRAVITY`                                                                                                       | `1000`, `9.81`                                 | Konstanten                                                            |
| `mmToM`, `mToMm`                                                                                                            | `(x: number) => number`                        | Längenumrechnung an den Rändern (UI/Dateien)                          |
| `degToRad`, `radToDeg`                                                                                                      | `(x: number) => number`                        | Winkelumrechnung                                                      |
| `SimTime`                                                                                                                   | `number` (ganzzahlige µs)                      | Typ der Simulationszeit                                               |
| `US_PER_S`, `US_PER_MS`                                                                                                     | `1_000_000`, `1_000`                           | Konstanten                                                            |
| `secondsToSimTime`, `msToSimTime`                                                                                           | `(x: number) => SimTime` (gerundet)            | Dauern in ganzzahlige µs umrechnen                                    |
| `simTimeToSeconds`                                                                                                          | `(t: SimTime) => number`                       | Zurück in Sekunden                                                    |
| `createRng`                                                                                                                 | `(seed, stream = 54) => Rng`                   | PCG32-Generator: `nextUint32()`, `next()` ∈ [0,1), `uniform(min,max)` |
| `createLogger`                                                                                                              | `(sink: LogSink, minLevel = 'info') => Logger` | `debug/info/warn/error(nachricht, daten?)`                            |
| `createMemorySink`                                                                                                          | `() => LogSink & { entries }`                  | Sammelt Logeinträge (Tests)                                           |
| `SCHEMA_VERSION`                                                                                                            | `1`                                            | Aktuelle Version der Projektdatei                                     |
| `projectSchema`, `layoutSchema`, `instanceSchema`, `connectionSchema`, `portRefSchema`, `vec3Schema`, `localizedTextSchema` | Zod-Schemas                                    | Validieren Projektdateien                                             |
| `Project`, `Layout`, `Instance`, `Connection`, `Vec3`, `LocalizedText`                                                      | Aus den Schemas abgeleitete Typen              | —                                                                     |

## library

| Export                                                                        | Signatur / Form                                                      | Zweck                                                                  |
| ----------------------------------------------------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `componentDefinitionSchema`, `parameterSchema`, `portSchema`, `affectsSchema` | Zod-Schemas                                                          | Validieren Definitionen ([[A05 Komponentenbibliothek]])                |
| `ComponentDefinition`, `ParameterDefinition`, `PortDefinition`                | Typen                                                                | —                                                                      |
| `DefinitionRegistry`                                                          | `register(json)`, `get(id)`, `list()`                                | Hält validierte Definitionen; lehnt Duplikate ab                       |
| `createBuiltInRegistry`                                                       | `() => DefinitionRegistry`                                           | Registry mit allen integrierten Definitionen (heute `conveyor.roller`) |
| `defaultParams`                                                               | `(def) => Record<string, number \| boolean \| string>`               | Nicht abgeleitete Standardwerte                                        |
| `min3RollersUnderPallet`                                                      | `(teilung, tragendeSeite) => ValidationIssue[]`                      | Teilung ≤ Seite / 3                                                    |
| `usableWidthFitsPallet`                                                       | `(nutzbreite, palettenbreite, führungsabstand) => ValidationIssue[]` | Breite ≥ Palette + 2 × Abstand                                         |
| `chainFitsBetweenRollers`                                                     | `(teilung, rollendurchmesser, kettenbreite) => ValidationIssue[]`    | Lücke ≥ Kette + 10 mm                                                  |
| `PALLET_TYPES`                                                                | `PalletType[]`                                                       | EUR, 1200 × 1000, GMA mit Belegungsprofilen                            |
| `FRICTION_PAIRS`                                                              | `FrictionPair[]`                                                     | Werkstoffpaarungen mit μ und üblichem Bereich                          |

## geometry

| Export            | Signatur                                                     | Zweck                                                       |
| ----------------- | ------------------------------------------------------------ | ----------------------------------------------------------- |
| `rollerLayout`    | `(länge, teilung, durchmesser) => { count, positions, gap }` | Zentrierte Achspositionen; Fehler bei Durchmesser ≥ Teilung |
| `minRollersUnder` | `(auflagelänge, teilung) => number`                          | Rollenzahl unter einer Palettenseite im ungünstigsten Fall  |

## sim-des

| Export                     | Signatur                                                                                                                       | Zweck                                                                |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------- |
| `EventQueue<T>`            | `push(zeit, priorität, nutzlast)`, `pop()`, `peek()`, `size`                                                                   | Min-Heap nach (Zeit, Priorität, Sequenz); nur ganzzahlige Zeiten     |
| `compareEvents`            | `(a, b) => number`                                                                                                             | Totale Ordnung der Warteschlange                                     |
| `DesSimulator`             | `now`, `on(typ, handler)`, `schedule(verzögerung, ereignis, priorität?) → Ticket`, `cancel(ticket)`, `runUntil(ende)`, `trace` | Next-Event-Schleife                                                  |
| `profileTime`              | `({ distance, maxSpeed, accel, decel? }) => Sekunden`                                                                          | Trapez-/Dreieckszeit; ohne `decel` = Ankunft mit Fahrgeschwindigkeit |
| `transferCycleTime`        | `({ liftTime, lowerTime, distance, speed, accel, settleTime? }) => Sekunden`                                                   | Zyklus des Kettenumsetzers (T4)                                      |
| `maxThroughput`            | `(geschwindigkeit, palettenlänge, lücke) => Paletten/s`                                                                        | $v/(L+g)$ (T2)                                                       |
| `palletCapacity`           | `(länge, palettenlänge, lücke) => Ganzzahl`                                                                                    | $\lfloor \text{Länge}/(L+g) \rfloor$                                 |
| `impactEnergy`             | `(masse, geschwindigkeit) => J`                                                                                                | $\frac12 m v^2$ (T5)                                                 |
| `positionAt`, `velocityAt` | `(segment, t) => number`                                                                                                       | Werten ein kinematisches Segment aus                                 |
| `timeToReach`              | `(segment, s) => t \| undefined`                                                                                               | Frühester Zeitpunkt, zu dem ein Segment *s* erreicht                 |

## sim-physics

| Export                                                      | Signatur                                                                                   | Zweck                                 |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------- |
| `PhysicsBackend`                                            | Schnittstelle ([[A07 Physik-Wrapper]])                                                     | Vertrag des Jolt/Rapier-Adapters      |
| `BodyDesc`, `Shape`, `Pose`, `Quat`, `RayHit`, `BodyHandle` | Typen                                                                                      | Beschreibung von Körpern und Abfragen |
| `FixedStepClock`                                            | `new FixedStepClock(dt = 1/120, maxSchritte = 8)`, `advance(frameZeit) → { steps, alpha }` | Festschritt-Akkumulator               |
| `DEFAULT_PHYSICS_DT`, `DEFAULT_MAX_STEPS_PER_FRAME`         | `1/120`, `8`                                                                               | Konstanten                            |
| `rollerAngularVelocity`                                     | `(oberflächengeschw., durchmesser) => rad/s`                                               | ω = v / r                             |
| `maxTractionAccel`                                          | `(mu) => m/s²`                                                                             | μ·g                                   |
| `turntableRequiredFriction`                                 | `(winkelbeschl., winkelgeschw., radius) => μ`                                              | $\sqrt{a_t^2 + a_c^2}/g$              |
| `turntableRequiredTorque`                                   | `(trägheit, winkelbeschl., reibmoment) => N·m`                                             | I·α + Reibung                         |
| `sweptRadius`                                               | `(länge, breite, freiraum) => m`                                                           | ½√(L² + B²) + Freiraum                |
| `boxYawInertia`                                             | `(masse, länge, breite) => kg·m²`                                                          | m(a² + b²)/12                         |

## control

| Export                                             | Signatur                                   | Zweck                                             |
| -------------------------------------------------- | ------------------------------------------ | ------------------------------------------------- |
| `Rule`, `Condition`, `SignalWrite`, `SignalValues` | Typen                                      | Regelmodell ([[A08 Steuerung]])                   |
| `evaluateRules`                                    | `(regeln, vorher, jetzt) => SignalWrite[]` | Reine Wenn/Dann-Auswertung mit Pegeln und Flanken |

## scenarios

| Export           | Signatur                        | Zweck                                             |
| ---------------- | ------------------------------- | ------------------------------------------------- |
| `SCENARIOS`      | `Scenario[]`                    | T1–T5 aus `scenarios/analytic.json`               |
| `findScenario`   | `(id) => Scenario \| undefined` | Suche ohne Beachtung der Groß-/Kleinschreibung    |
| `runScenario`    | `(szenario) => ScenarioResult`  | `{ id, actual, expected, relativeError, passed }` |
| `scenarioSchema` | Zod-Schema                      | Validiert die JSON-Datei                          |

## Verwandt

[[A03 Monorepo und Stack]] · [[A06 DES-Engine]] · [[A07 Physik-Wrapper]]
