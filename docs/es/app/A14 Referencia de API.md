---
title: A14 Referencia de API
lang: es
tags: [app, api, reference]
---

🌐 [[A14 API Reference|EN]] · **ES** · [[A14 API-Referenz|DE]] · [[Inicio (ES)|Índice]]

# A14 Referencia de API (estado a 28-10-2026)

Todos los valores en unidades SI salvo indicación. Cada paquete se importa como `@intracraft/<paquete>`.

## core-model

| Exportación                                                                                                                 | Firma / forma                                  | Para qué sirve                                                        |
| --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------- |
| `MM_PER_M`, `GRAVITY`                                                                                                       | `1000`, `9.81`                                 | Constantes                                                            |
| `mmToM`, `mToMm`                                                                                                            | `(x: number) => number`                        | Conversión de longitudes en los bordes (interfaz/archivos)            |
| `degToRad`, `radToDeg`                                                                                                      | `(x: number) => number`                        | Conversión de ángulos                                                 |
| `SimTime`                                                                                                                   | `number` (µs enteros)                          | Tipo del tiempo de simulación                                         |
| `US_PER_S`, `US_PER_MS`                                                                                                     | `1_000_000`, `1_000`                           | Constantes                                                            |
| `secondsToSimTime`, `msToSimTime`                                                                                           | `(x: number) => SimTime` (redondeado)          | Pasar duraciones a µs enteros                                         |
| `simTimeToSeconds`                                                                                                          | `(t: SimTime) => number`                       | Volver a segundos                                                     |
| `createRng`                                                                                                                 | `(semilla, flujo = 54) => Rng`                 | Generador PCG32: `nextUint32()`, `next()` ∈ [0,1), `uniform(min,max)` |
| `createLogger`                                                                                                              | `(sink: LogSink, minLevel = 'info') => Logger` | `debug/info/warn/error(mensaje, datos?)`                              |
| `createMemorySink`                                                                                                          | `() => LogSink & { entries }`                  | Recoge entradas del log (pruebas)                                     |
| `SCHEMA_VERSION`                                                                                                            | `1`                                            | Versión actual del archivo de proyecto                                |
| `projectSchema`, `layoutSchema`, `instanceSchema`, `connectionSchema`, `portRefSchema`, `vec3Schema`, `localizedTextSchema` | Esquemas Zod                                   | Validan archivos de proyecto                                          |
| `Project`, `Layout`, `Instance`, `Connection`, `Vec3`, `LocalizedText`                                                      | Tipos deducidos de los esquemas                | —                                                                     |

## library

| Exportación                                                                   | Firma / forma                                                | Para qué sirve                                                         |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------- |
| `componentDefinitionSchema`, `parameterSchema`, `portSchema`, `affectsSchema` | Esquemas Zod                                                 | Validan definiciones ([[A05 Librería de componentes]])                 |
| `ComponentDefinition`, `ParameterDefinition`, `PortDefinition`                | Tipos                                                        | —                                                                      |
| `DefinitionRegistry`                                                          | `register(json)`, `get(id)`, `list()`                        | Guarda definiciones validadas; rechaza duplicados                      |
| `createBuiltInRegistry`                                                       | `() => DefinitionRegistry`                                   | Registro con todas las definiciones integradas (hoy `conveyor.roller`) |
| `defaultParams`                                                               | `(def) => Record<string, number \| boolean \| string>`       | Valores por defecto no derivados                                       |
| `min3RollersUnderPallet`                                                      | `(paso, ladoApoyado) => ValidationIssue[]`                   | paso ≤ lado / 3                                                        |
| `usableWidthFitsPallet`                                                       | `(anchoUtil, anchoPallet, holguraGuia) => ValidationIssue[]` | ancho ≥ pallet + 2 × holgura                                           |
| `chainFitsBetweenRollers`                                                     | `(paso, diametroRodillo, anchoCadena) => ValidationIssue[]`  | hueco ≥ cadena + 10 mm                                                 |
| `PALLET_TYPES`                                                                | `PalletType[]`                                               | EUR, 1200 × 1000 y GMA con perfiles de ocupación                       |
| `FRICTION_PAIRS`                                                              | `FrictionPair[]`                                             | Pares de materiales con μ y rango habitual                             |

## geometry

| Exportación       | Firma                                                     | Para qué sirve                                             |
| ----------------- | --------------------------------------------------------- | ---------------------------------------------------------- |
| `rollerLayout`    | `(longitud, paso, diametro) => { count, positions, gap }` | Posiciones centradas de los ejes; error si diámetro ≥ paso |
| `minRollersUnder` | `(longitudApoyo, paso) => number`                         | Número de rodillos bajo un lado del pallet en el peor caso |

## sim-des

| Exportación                | Firma                                                                                                                    | Para qué sirve                                                            |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| `EventQueue<T>`            | `push(tiempo, prioridad, carga)`, `pop()`, `peek()`, `size`                                                              | Montículo por (tiempo, prioridad, secuencia); solo tiempos enteros        |
| `compareEvents`            | `(a, b) => number`                                                                                                       | Orden total que usa la cola                                               |
| `DesSimulator`             | `now`, `on(tipo, manejador)`, `schedule(retardo, evento, prioridad?) → tique`, `cancel(tique)`, `runUntil(fin)`, `trace` | Bucle de siguiente evento                                                 |
| `profileTime`              | `({ distance, maxSpeed, accel, decel? }) => segundos`                                                                    | Tiempo trapezoidal/triangular; sin `decel` = llega a velocidad de crucero |
| `transferCycleTime`        | `({ liftTime, lowerTime, distance, speed, accel, settleTime? }) => segundos`                                             | Ciclo de la mesa de transferencia (T4)                                    |
| `maxThroughput`            | `(velocidad, largoPallet, separacion) => pallets/s`                                                                      | $v/(L+g)$ (T2)                                                            |
| `palletCapacity`           | `(longitud, largoPallet, separacion) => entero`                                                                          | $\lfloor \text{longitud}/(L+g) \rfloor$                                   |
| `impactEnergy`             | `(masa, velocidad) => J`                                                                                                 | $\frac12 m v^2$ (T5)                                                      |
| `positionAt`, `velocityAt` | `(segmento, t) => number`                                                                                                | Evalúan un segmento cinemático                                            |
| `timeToReach`              | `(segmento, s) => t \| undefined`                                                                                        | Primer instante en que un segmento alcanza la posición *s*                |

## sim-physics

| Exportación                                                 | Firma                                                                                         | Para qué sirve                     |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ---------------------------------- |
| `PhysicsBackend`                                            | interfaz ([[A07 Envoltorio del motor físico]])                                                | Contrato del adaptador Jolt/Rapier |
| `BodyDesc`, `Shape`, `Pose`, `Quat`, `RayHit`, `BodyHandle` | Tipos                                                                                         | Descripción de cuerpos y consultas |
| `FixedStepClock`                                            | `new FixedStepClock(dt = 1/120, maxPasos = 8)`, `advance(tiempoFotograma) → { steps, alpha }` | Acumulador de paso fijo            |
| `DEFAULT_PHYSICS_DT`, `DEFAULT_MAX_STEPS_PER_FRAME`         | `1/120`, `8`                                                                                  | Constantes                         |
| `rollerAngularVelocity`                                     | `(velocidadSuperficie, diametro) => rad/s`                                                    | ω = v / r                          |
| `maxTractionAccel`                                          | `(mu) => m/s²`                                                                                | μ·g                                |
| `turntableRequiredFriction`                                 | `(aceleracionAngular, velocidadAngular, radio) => μ`                                          | $\sqrt{a_t^2 + a_c^2}/g$           |
| `turntableRequiredTorque`                                   | `(inercia, aceleracionAngular, parFriccion) => N·m`                                           | I·α + fricción                     |
| `sweptRadius`                                               | `(largo, ancho, holgura) => m`                                                                | ½√(L² + W²) + holgura              |
| `boxYawInertia`                                             | `(masa, largo, ancho) => kg·m²`                                                               | m(a² + b²)/12                      |

## control

| Exportación                                        | Firma                                         | Para qué sirve                                    |
| -------------------------------------------------- | --------------------------------------------- | ------------------------------------------------- |
| `Rule`, `Condition`, `SignalWrite`, `SignalValues` | Tipos                                         | Modelo de reglas ([[A08 Controlador]])            |
| `evaluateRules`                                    | `(reglas, anterior, actual) => SignalWrite[]` | Evaluación pura Si/Entonces con niveles y flancos |

## scenarios

| Exportación      | Firma                           | Para qué sirve                                    |
| ---------------- | ------------------------------- | ------------------------------------------------- |
| `SCENARIOS`      | `Scenario[]`                    | T1–T5 de `scenarios/analytic.json`                |
| `findScenario`   | `(id) => Scenario \| undefined` | Búsqueda sin distinguir mayúsculas                |
| `runScenario`    | `(escenario) => ScenarioResult` | `{ id, actual, expected, relativeError, passed }` |
| `scenarioSchema` | Esquema Zod                     | Valida el archivo JSON                            |

## Relacionado

[[A03 Monorepo y stack]] · [[A06 Motor discreto]] · [[A07 Envoltorio del motor físico]]
