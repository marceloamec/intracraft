---
title: S04 Giratoria de rodillos
lang: es
tags: [spec, conveyor, turntable, roller]
library-id: conveyor.roller_turntable
status: spec
---

🌐 [[S04 Roller Turntable|EN]] · **ES** · [[S04 Rollendrehtisch|DE]] · [[Inicio (ES)|Índice]]

# S04 Giratoria de rodillos (`conveyor.roller_turntable`)

La misma plataforma giratoria que [[S03 Giratoria de cadenas]] con un tramo de rodillos encima; es la opción habitual cuando las líneas de entrada y salida son de rodillos. Usa el grupo *plataforma y giro* de S03 y el grupo de rodillos de [[S01 Mesa de rodillos]], más:

| Clave                   | Clave del plan          | Unidad | Defecto | Rango                        | Descripción                                                   |
| ----------------------- | ----------------------- | ------ | ------- | ---------------------------- | ------------------------------------------------------------- |
| `lockRollersDuringTurn` | `bloqueoRodillosEnGiro` | bool   | sí      | —                            | Rodillos frenados durante el giro para que el pallet no ruede |
| `holdDuringTurn`        | `retencionEnGiro`       | enum   | ninguna | ninguna, topes de plataforma | Topes que retienen el pallet durante el giro                  |
| `loadSpeed`             | `velocidadCarga`        | m/s    | 0,25    | 0,05 … 0,6                   | Velocidad de los rodillos al recibir y descargar              |

## Comportamiento

**2D.** Como S03: el pallet viaja con la plataforma; el tiempo de giro sale del perfil angular (T3).

**3D.** Con los rodillos bloqueados ($\omega = 0$) el contacto es de **deslizamiento** con μ, no de rodadura; con rodillos libres el pallet rodaría y se desplazaría durante la aceleración angular; por eso el bloqueo es la opción por defecto ([[T12 Rodillos y cadenas en física]], [[T13 Giratorias y mesas de transferencia]]).

## Validaciones

Si `lockRollersDuringTurn` está desactivado y `holdDuringTurn` = ninguna → aviso de riesgo de desplazamiento. Además, todas las validaciones de S03.

## Pruebas

La misma batería que S03 (T3, S1-e / P4).

## Relacionado

[[S03 Giratoria de cadenas]] · [[S01 Mesa de rodillos]] · [[T13 Giratorias y mesas de transferencia]]
