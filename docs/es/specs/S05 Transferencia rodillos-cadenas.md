---
title: S05 Transferencia rodillos-cadenas
lang: es
tags: [spec, conveyor, transfer]
library-id: conveyor.transfer_rc
status: spec
---

🌐 [[S05 Roller-to-Chain Transfer|EN]] · **ES** · [[S05 Rollen-Ketten-Umsetzer|DE]] · [[Inicio (ES)|Índice]]

# S05 Mesa de transferencia rodillos→cadenas (`conveyor.transfer_rc`)

**Interpretación (a confirmar, plan §18):** estación única con dos sistemas de transporte ortogonales. Un tramo de rodillos recibe el pallet en sentido longitudinal; entre los rodillos hay cadenas elevables orientadas en sentido transversal. Al subir, las cadenas levantan el pallet de los rodillos, lo desplazan 90° hacia la línea de cadenas contigua y bajan. El pallet cambia de dirección sin girar. Teoría: [[T13 Giratorias y mesas de transferencia]], parte 2.

**Puertos:** A y B (rodillos, longitudinal), C1 y C2 (cadenas, izquierda y derecha).

## 1. Parámetros

| Clave                    | Clave del plan              | Unidad | Defecto          | Rango                                             | Descripción                                                                   |
| ------------------------ | --------------------------- | ------ | ---------------- | ------------------------------------------------- | ----------------------------------------------------------------------------- |
| Grupo de rodillos        | —                           | —      | —                | —                                                 | Como [[S01 Mesa de rodillos]]; `length` ≈ largo del pallet + 200 mm           |
| `transferChainCount`     | `numCadenasTransfer`        | —      | 3                | 2 … 5                                             | Número de cadenas elevables                                                   |
| `transferChainPositions` | `posicionesCadenasTransfer` | mm     | auto             | —                                                 | Posición longitudinal de cada cadena, entre rodillos y alineada con el pallet |
| `transferChainWidth`     | `anchoCadenaTransfer`       | mm     | 40               | 20 … 80                                           | Ancho de cada cadena                                                          |
| `liftStroke`             | `carreraElevacion`          | mm     | 40               | 20 … 100                                          | Altura de las cadenas sobre los rodillos al subir                             |
| `lowerClearance`         | `holguraBajada`             | mm     | 10               | 0 … 30                                            | Distancia bajo el plano de rodillos con las cadenas abajo                     |
| `liftTime`               | `tiempoSubida`              | s      | 1,0              | 0,3 … 5                                           | Tiempo de elevación                                                           |
| `lowerTime`              | `tiempoBajada`              | s      | 1,0              | 0,3 … 5                                           | Tiempo de descenso                                                            |
| `actuator`               | `actuador`                  | enum   | neumático        | neumático, eléctrico                              | Neumático: rápido con amortiguación final; eléctrico: con rampas              |
| `transferSpeed`          | `velocidadTransferencia`    | m/s    | 0,20             | 0,05 … 0,5                                        | Velocidad de las cadenas durante la transferencia                             |
| `transferAccel`          | `aceleracionTransferencia`  | m/s²   | 0,5              | 0,1 … 2                                           | Rampa de las cadenas                                                          |
| `transferDistance`       | `distanciaTransferencia`    | mm     | auto             | —                                                 | Recorrido de la cadena (≈ ancho de la mesa); editable                         |
| `transferDirection`      | `direccionTransferencia`    | enum   | derecha          | izquierda, derecha, bidireccional                 | Lado hacia el que se transfiere                                               |
| `operationMode`          | `modoOperacion`             | enum   | rodillos→cadenas | rodillos→cadenas, cadenas→rodillos, bidireccional | Sentido del flujo                                                             |
| `positionTolerance`      | `toleranciaPosicion`        | mm     | ±20              | 5 … 50                                            | Desviación admisible del pallet para permitir la elevación                    |
| `requiresStop`           | `requiereTope`              | bool   | sí               | —                                                 | Incluye un tope retráctil de posicionamiento en los rodillos                  |
| `transferChainMu`        | `muCadenaTransfer`          | —      | 0,30             | 0,1 … 0,6                                         | Rozamiento pallet–cadena                                                      |

## 2. Secuencia y enclavamientos

reposo (cadenas abajo) → pallet en posición (tope y sensores) → subiendo → elevado → transfiriendo → descargado → bajando → reposo.

Enclavamientos: no subir sin pallet centrado · rodillos detenidos mientras las cadenas están arriba · no bajar hasta que el pallet haya dejado la zona de cadenas · en modo bidireccional no aceptar pallets por los dos lados a la vez.

## 3. Fórmulas

- **Ciclo ideal:** $t = t_{subida} + (d/v + v/a) + t_{bajada}$ (+ asentamiento). 1 m, 0,2 m/s, 0,5 m/s² → 1,0 + 5,4 + 1,0 = **7,4 s** (T4).
- **Hueco:** Ø89 mm con paso 206,3 mm (valores del proyecto) → hueco de 117,3 mm ≥ cadena 40 mm + 10 mm ✓ (`chainFitsBetweenRollers`).
- **Continuidad de alturas:** la línea de cadenas contigua a la altura de rodillos + `liftStroke` (tolerancia de puerto ±10 mm).

## 4. Comportamiento

**2D.** Máquina de estados con las duraciones anteriores; durante la elevación el tramo de rodillos queda bloqueado.
**3D.** Las cadenas son cuerpos cinemáticos que suben (movimiento vertical) y luego se desplazan en horizontal; el pallet se levanta por contacto. El simulador avisa si una cadena cae en un hueco del pallet y no lo toca.

## 5. Pruebas

| Prueba | Criterio                                                           | Estado     |
| ------ | ------------------------------------------------------------------ | ---------- |
| T4     | Ciclo de 7,4 s                                                     | ✅ fórmula |
| P5     | Elevado 40 ± 5 mm, desplazado 1 m ± 10 mm, depositado girando < 1° | ⏳ fase 8  |

## Relacionado

[[T13 Giratorias y mesas de transferencia]] · [[S01 Mesa de rodillos]] · [[S02 Mesa de cadenas]] · [[T07 Lógica de control y señales]]
