---
title: A10 CLI y escenarios
lang: es
tags: [app, cli, scenarios]
---

🌐 [[A10 CLI and Scenarios|EN]] · **ES** · [[A10 CLI und Szenarien|DE]] · [[Inicio (ES)|Índice]]

# A10 CLI y escenarios

## 1. Escenarios (`packages/scenarios`)

Un **escenario** es un experimento reproducible con un resultado esperado. Hoy el paquete contiene los cinco escenarios **analíticos** del plan en `scenarios/analytic.json`:

| Id  | Tipo            | Entradas (SI)                                  | Esperado          |
| --- | --------------- | ---------------------------------------------- | ----------------- |
| T1  | `profileTime`   | 3 m, 0,25 m/s, 0,5 m/s², sin frenar            | 12,25 s           |
| T2  | `throughput`    | 0,25 m/s, pallet de 1,2 m, separación 0,15 m   | 0,18519 pallets/s |
| T3  | `profileTime`   | π/2 rad, 15 °/s, 30 °/s² (en rad)              | 6,5 s             |
| T4  | `transferCycle` | 1 s subida, 1 s bajada, 1 m, 0,2 m/s, 0,5 m/s² | 7,4 s             |
| T5  | `impactEnergy`  | 1025 kg, 0,25 m/s                              | 32 J              |

Cada escenario tiene un título traducido (en/es/de), una unidad y una tolerancia relativa (1 %). `runScenario` calcula el resultado con las fórmulas de `sim-des` y devuelve `{ actual, expected, relativeError, passed }`.

Desde la fase 3 los escenarios contendrán también **layouts completos** (generador → transportadores → sumidero) ejecutados por el motor DES, con resultados de referencia en `test_data/`.

## 2. CLI (`apps/cli`)

```text
pnpm run scenario all     # ejecutar todos los escenarios
pnpm run scenario t3      # ejecutar uno (sin distinguir mayúsculas)
```

Ejemplo de salida:

```text
PASS T1 Roller conveyor: 3 m from rest, 0.25 m/s, 0.5 m/s², no braking: 12.2500 s (expected 12.25 s, error 0.000 %)
…
5/5 scenarios passed
```

Códigos de salida: `0` todos pasan · `1` al menos uno falla · `2` id de escenario desconocido. La CLI escribe a través del logger del proyecto (sin `console.log`).

## Relacionado

[[A11 Pruebas y validación]] · [[T04 Cinemática de transportadores]] · [[A14 Referencia de API]]
