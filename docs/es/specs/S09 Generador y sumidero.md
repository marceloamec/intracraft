---
title: S09 Generador y sumidero
lang: es
tags: [spec, flow]
library-id: flow.source, flow.sink
status: spec
---

🌐 [[S09 Source and Sink|EN]] · **ES** · [[S09 Quelle und Senke|DE]] · [[Inicio (ES)|Índice]]

# S09 Generador y sumidero (`flow.source`, `flow.sink`)

## 1. Generador de pallets (Source)

| Clave             | Clave del plan      | Unidad | Defecto            | Rango                                                     | Descripción                                                                                    |
| ----------------- | ------------------- | ------ | ------------------ | --------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `palletType`      | `tipoPallet`        | ref    | EUR                | —                                                         | Tipo de pallet de la librería ([[S10 Tipos de pallet]])                                        |
| `loadMass`        | `masaCarga`         | kg     | 600                | 0 … 1500                                                  | Fija o distribución (uniforme, normal, tabla)                                                  |
| `loadHeight`      | `alturaCarga`       | mm     | 1200               | 0 … 2200                                                  | Altura de la carga sobre el pallet                                                             |
| `orientation`     | `orientacion`       | enum   | largo en el avance | largo, ancho                                              | Orientación al generarse                                                                       |
| `generationMode`  | `modoGeneracion`    | enum   | intervalo fijo     | intervalo fijo, distribución, bajo demanda (señal), lista | Regla de llegada                                                                               |
| `interval`        | `intervalo`         | s      | 20                 | 1 … 3600                                                  | Para intervalo fijo                                                                            |
| `maxPallets`      | `maxPallets`        | —      | ilimitado          | —                                                         | Límite de pallets generados                                                                    |
| `blockIfOccupied` | `bloquearSiOcupado` | bool   | sí                 | —                                                         | Espera si no hay espacio libre al inicio                                                       |
| `seed`            | `semilla`           | entero | global             | —                                                         | Para distribuciones (flujo propio por objeto, [[T03 Tiempo, orden de eventos y determinismo]]) |

**2D.** Programa eventos `pallet_enters`: intervalo fijo → cada `interval`; distribución → tiempo entre llegadas sacado del generador con semilla; bajo demanda → al subir la señal. Con `blockIfOccupied`, una creación que encuentra la entrada ocupada espera (ese tiempo «perdido» se publica).
**3D.** Crea el cuerpo compuesto del pallet en el puerto de salida, en reposo y alineado con el transportador.

## 2. Sumidero (Sink)

| Clave         | Clave del plan   | Unidad | Defecto   | Rango                                | Descripción            |
| ------------- | ---------------- | ------ | --------- | ------------------------------------ | ---------------------- |
| `removalMode` | `modoRetirada`   | enum   | inmediata | inmediata, tras un tiempo, por señal | Cómo salen los pallets |
| `removalTime` | `tiempoRetirada` | s      | 0         | ≥ 0                                  | Para «tras un tiempo»  |

Publica pallets recibidos, rendimiento y tiempo medio de ciclo (entrada al modelo → llegada al sumidero).

## Relacionado

[[S10 Tipos de pallet]] · [[T08 Estadísticas y análisis de resultados]] · [[T03 Tiempo, orden de eventos y determinismo]]
