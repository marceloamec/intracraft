---
title: A11 Pruebas y validación
lang: es
tags: [app, testing, validation]
---

🌐 [[A11 Testing and Validation|EN]] · **ES** · [[A11 Tests und Validierung|DE]] · [[Inicio (ES)|Índice]]

# A11 Pruebas y validación

**Primero la prueba, después la implementación.** Los resultados esperados salen de la especificación, nunca de la implementación.

## 1. Capas de pruebas

1. **Unitarias** — unidades, cinemática, cola de eventos, validación de parámetros.
2. **Escenarios 2D con resultado analítico** (§2).
3. **Escenarios de física** (§3).
4. **Consistencia entre modos** (§4).
5. **Regresión con resultados de referencia** y **determinismo** (§5).
6. **Contraste con modelos de referencia conocidos** (§6).

## 2. Pruebas analíticas del modo 2D (error < 1 %)

| Id  | Escenario                                                        | Esperado                                        | Estado                 |
| --- | ---------------------------------------------------------------- | ----------------------------------------------- | ---------------------- |
| T1  | Un pallet, 3 m desde reposo, 0,25 m/s, 0,5 m/s², sin frenar      | $D/v + v/2a$ = 12,25 s                          | ✅ fórmula             |
| T2  | Saturación, EUR lado largo (1,2 m), separación 0,15 m            | $v/(L+g)$ ≈ 0,185 pallets/s ≈ 667/h             | ✅ fórmula             |
| T3  | Giro de 90°, 15 °/s, 30 °/s²                                     | 6,5 s                                           | ✅ fórmula             |
| T4  | Transferencia: 1 m, 0,2 m/s, 0,5 m/s², 1 s de subida y de bajada | 7,4 s                                           | ✅ fórmula             |
| T5  | Energía de impacto de 1025 kg a 0,25 m/s                         | 32 J                                            | ✅ fórmula             |
| T6  | Sensor a 2,0 m, haz alto                                         | Subida con el frente, bajada con la cola; 4,8 s | ⏳ requiere pistas DES |

«✅ fórmula» = las funciones en forma cerrada pasan; la ejecución DES completa de cada layout llega en las fases 3–8.

## 3. Escenarios de física

| Id  | Escenario          | Criterio                                                               |
| --- | ------------------ | ---------------------------------------------------------------------- |
| P1  | Parada contra tope | Frente a ±5 mm; rebote < 10 mm                                         |
| P2  | Impacto            | Velocidad y energía registradas; aviso por encima de `maxImpactEnergy` |
| P3  | Guía con embudo    | 50 mm y 5° corregidos a ±10 mm y ±1°                                   |
| P4  | Giro de 90°        | Desplazamiento del pallet < 5 mm; sin deslizar con μ ≥ 0,04            |
| P5  | Transferencia      | Elevado 40 ± 5 mm, desplazado 1 m ± 10 mm, girado < 1°                 |
| P6  | Sensores           | Flancos 3D y 2D dentro de ±50 ms                                       |

Los criterios a–g del spike S1 están en [[T15 Investigación de motores de física]] y [[ADR-0002 Candidato a motor de física]].

## 4. Consistencia entre modos

Mismo escenario en ambos modos: tiempos de tránsito ±5 %, flancos de sensores ±50 ms. Las diferencias mayores son hallazgos ([[T16 Consistencia híbrida y calibración]]).

## 5. Regresión y determinismo

- Resultados de referencia en `test_data/` por escenario. Cualquier cambio debe explicarse en el mensaje del commit y en el registro de cambios. Nunca se editan para que una prueba pase.
- Dos ejecuciones con la misma semilla deben dar el mismo hash del registro de eventos (2D) y de los estados de los cuerpos (3D). Ya probado hoy: trazas de eventos idénticas (`simulator.test.ts`) y secuencias aleatorias idénticas (`rng.test.ts`).

## 6. Contraste con modelos de referencia

Elegir tres modelos que el equipo conozca bien (flujo simple, acumulación, giratoria/transferencia), reconstruirlos en 2D y comparar rendimiento, tiempo medio de ciclo y ocupación. Documentar las diferencias en lugar de forzarlas; solo se comparan resultados observables (regla clean-room).

## 7. Inventario actual de pruebas (28-10-2026)

`pnpm test` → **11 archivos, 62 pruebas, todas en verde** (29-10-2026); `pnpm lint` (ESLint + tipos) sin errores; `pnpm run scenario all` → 5/5.

| Paquete     | Archivo(s) de prueba                             |
| ----------- | ------------------------------------------------ |
| core-model  | `units.test.ts`, `rng.test.ts`, `schema.test.ts` |
| sim-des     | `kinematics.test.ts`, `simulator.test.ts`        |
| sim-physics | `mechanics.test.ts`                              |
| control     | `rules.test.ts`                                  |
| geometry    | `rollers.test.ts`                                |
| library     | `library.test.ts`                                |
| scenarios   | `analytic.test.ts`                               |
| editor      | `i18n/strings.test.ts`                           |

## Relacionado

[[A10 CLI y escenarios]] · [[A12 Hoja de ruta]] · [[T08 Estadísticas y análisis de resultados]]
