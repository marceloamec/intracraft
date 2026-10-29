---
title: S02 Mesa de cadenas
lang: es
tags: [spec, conveyor, chain]
library-id: conveyor.chain
status: spec
updated: 2026-10-29
---

🌐 [[S02 Chain Conveyor|EN]] · **ES** · [[S02 Kettenförderer|DE]] · [[Inicio (ES)|Índice]]

# S02 Mesa de cadenas (`conveyor.chain`)

Tramo recto en el que el pallet apoya sobre varios ramales de cadena sincronizados. **Los pallets EUR van en sentido transversal (lado corto, 800 mm, delante) en las mesas de cadenas**, confirmado el 29-10-2026. Las tres tablas inferiores (de 1200 mm) quedan entonces **perpendiculares** a las cadenas, así que cada ramal toca todas las tablas. Se suma a los parámetros comunes de [[S01 Mesa de rodillos]] §1; todos los parámetros de comportamiento son por instancia.

## 1. Parámetros

| Clave                   | Clave del plan        | Unidad | Defecto              | Rango                           | Descripción                                                                                                                                                                   |
| ----------------------- | --------------------- | ------ | -------------------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `palletSideAlongTravel` | `ladoParaleloAvance`  | enum   | **corto**            | largo, corto                    | EUR: 800 mm en el avance, 1200 mm transversal                                                                                                                                 |
| `usableWidth`           | `anchoUtil`           | mm     | **1300** (propuesto) | 500 … 1800                      | Debe ser ≥ 1200 + 2 × holgura para un EUR transversal (el valor común de 1000 mm no vale); a confirmar                                                                        |
| `chainCount`            | `numCadenas`          | —      | 3                    | 2 … 6                           | Número de ramales                                                                                                                                                             |
| `chainPositions`        | `posicionesCadenas`   | mm     | auto                 | —                               | Posición transversal de cada ramal respecto al eje                                                                                                                            |
| `chainSupportWidth`     | `anchoApoyoCadena`    | mm     | 40                   | 20 … 100                        | Ancho de contacto de cada ramal                                                                                                                                               |
| `chainType`             | `tipoCadena`          | enum   | de rodillos          | simple, de rodillos, doble paso | Rozamiento y geometría de apoyo                                                                                                                                               |
| `chainPitch`            | `pasoCadena`          | mm     | 38,1                 | 19,05 … 76,2                    | Paso; geometría y efecto poligonal en 3D                                                                                                                                      |
| `chainMu`               | `muCadena`            | —      | 0,30                 | 0,1 … 0,6                       | Rozamiento pallet–cadena                                                                                                                                                      |
| `accumulationMode`      | `modoAcumulacion`     | enum   | **none**             | none, slip, zoned               | none: todos los pallets siguen a la cadena y el transportador marcha y para como una unidad; slip: la cadena patina bajo los pallets retenidos; zoned: ramales independientes |
| `zoneLength`            | `longitudZona`        | mm     | 1400                 | 1000 … 2000                     | Solo por zonas                                                                                                                                                                |
| `stopContactTime`       | `tiempoDetencionTope` | s      | 0,2                  | 0,05 … 1                        | 2D: tiempo que tarda el pallet en detenerse en un tope (modo slip)                                                                                                            |
| `maxDragForce`          | `fuerzaMaxArrastre`   | N      | derivado             | —                               | μ × peso del pallet; informativo                                                                                                                                              |

Capacidad con un EUR transversal: $\lfloor \text{longitud}/(0{,}8 + \text{separación}) \rfloor$ → en 3 m caben 3 pallets con 150 mm de separación. Rendimiento limitado por la separación a 0,25 m/s: $0{,}25/(0{,}8+0{,}15)$ = 0,263 pallets/s ≈ **947 pallets/h**.

Puertos A (entrada) y B (salida), como en la mesa de rodillos.

## 2. Comportamiento

- **`none` (por defecto):** igual que la mesa de rodillos: cada pallet sigue a la cadena; al recibir un pallet se mueven los que ya están; una salida bloqueada para todo el transportador ([[S01 Mesa de rodillos]] §3).
- **`slip`:** un pallet retenido se detiene y la cadena sigue girando bajo él (presión de línea $\sum \mu m g$, [[T05 Acumulación, capacidad y rendimiento]] §6.3).
- **3D:** cada ramal es una tira de rodillos cinemáticos pequeños (plan A) o una caja estática con **superficie móvil** (plan B, disponible en el binding JS de Jolt): [[T12 Rodillos y cadenas en física]], [[ADR-0002 Candidato a motor de física]].

## 3. Validaciones

- Al menos 2 cadenas bajo el pallet; con EUR transversal cada ramal cruza las tres tablas inferiores.
- Ancho útil ≥ 1200 + 2 × holgura para EUR transversal.

## 4. Pruebas

| Prueba | Criterio                                                        | Estado    |
| ------ | --------------------------------------------------------------- | --------- |
| S1-b   | Velocidad del pallet = de la cadena ±3 %, sin vibración visible | ⏳ S1     |
| —      | `none`: el pallet que entra mueve al que ya está                | ⏳ fase 5 |
| —      | `slip`: fuerza de línea ≈ Σ μ m g                               | ⏳ fase 5 |

## Relacionado

[[T12 Rodillos y cadenas en física]] · [[S05 Transferencia rodillos-cadenas]] · [[S03 Giratoria de cadenas]] · [[Preguntas abiertas]]
