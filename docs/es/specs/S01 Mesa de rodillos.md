---
title: S01 Mesa de rodillos
lang: es
tags: [spec, conveyor, roller]
library-id: conveyor.roller
status: definition-json
updated: 2026-10-29
---

🌐 [[S01 Roller Conveyor|EN]] · **ES** · [[S01 Rollenförderer|DE]] · [[Inicio (ES)|Índice]]

# S01 Mesa de rodillos (`conveyor.roller`)

Tramo recto de rodillos que lleva el pallet sobre sus tablas inferiores en el sentido del avance. **Los pallets EUR van en sentido longitudinal (lado largo, 1200 mm, delante) en las mesas de rodillos**, confirmado por el responsable el 29-10-2026. Es el componente base: los demás reutilizan su grupo de rodillos. Definición: `packages/library/src/definitions/roller-conveyor.json`.

Los valores por defecto son **orientativos**; se muestran en unidades de visualización y se guardan en SI. **Todo parámetro de comportamiento pertenece a la instancia**: dos mesas de rodillos del mismo tipo pueden tener longitudes, capacidades y modos de acumulación distintos.

## 1. Parámetros comunes (todos los transportadores)

| Clave                   | Clave del plan                 | Unidad | Defecto                  | Rango                       | Descripción                                                           |
| ----------------------- | ------------------------------ | ------ | ------------------------ | --------------------------- | --------------------------------------------------------------------- |
| `name`                  | `nombre`                       | texto  | auto                     | —                           | Nombre visible; el `id` interno se genera solo                        |
| `position`              | `posicion`                     | m      | 0, 0, 0                  | —                           | Origen (centro del inicio)                                            |
| `yaw`                   | `giro`                         | °      | 0                        | −180 … 180                  | Rotación alrededor del eje vertical                                   |
| `surfaceHeight`         | `alturaSuperficie`             | mm     | 600                      | 200 … 1500                  | Altura del plano de apoyo del pallet sobre el suelo                   |
| `length`                | `longitud`                     | mm     | 3000                     | 500 … 12000                 | Longitud en el sentido del avance; los más largos admiten más pallets |
| `usableWidth`           | `anchoUtil`                    | mm     | 1000                     | 500 … 1800                  | Ancho libre entre bastidores o guías                                  |
| `palletSideAlongTravel` | `ladoParaleloAvance`           | enum   | largo                    | largo, corto                | Largo en rodillos (EUR con 1200 mm en el avance)                      |
| `nominalSpeed`          | `velocidadNominal`             | m/s    | 0,25                     | 0,05 … 0,60                 | Velocidad de crucero                                                  |
| `approachSpeed`         | `velocidadAproximacion`        | m/s    | 0,10                     | 0,02 … 0,30                 | Velocidad lenta antes de un tope                                      |
| `accel` / `decel`       | `aceleracion` / `deceleracion` | m/s²   | 0,5 / 0,5                | 0,05 … 2,0                  | Rampas de arranque y de parada                                        |
| `direction`             | `sentido`                      | enum   | adelante                 | adelante, atrás, reversible | Sentido de marcha                                                     |
| `maxPalletLoad`         | `cargaMaxPallet`               | kg     | 1000                     | 100 … 2000                  | Avisa si un pallet pesa más                                           |
| `minGap`                | `separacionMinima`             | mm     | 150                      | 50 … 500                    | Holgura mínima entre pallets (la usan ZPA / presión y la capacidad)   |
| `accumulationMode`      | `modoAcumulacion`              | enum   | **none**                 | none, zpa, pressure         | Ver §3                                                                |
| `guides`                | `guias`                        | grupo  | ver [[S08 Guía lateral]] | —                           | Guías laterales integradas                                            |
| `enabled`               | `habilitado`                   | bool   | sí                       | —                           | Desactivar sin borrar                                                 |
| `color`, `legs`         | `color`, `patas`               | —      | —                        | —                           | Solo apariencia                                                       |
| `failures`              | `fallos`                       | —      | desactivado              | —                           | Reservado (MTBF/MTTR, post-MVP)                                       |

**Señales comunes:** entradas `run`, `reverse`, `enable` y, por zona, `zone[i].run`; salidas `running`, `occupied`, `zone[i].occupied`, `fault` (reservada). **Estados (2D):** parado, acelerando, en marcha, deteniendo, bloqueado. **Estadísticas:** utilización, tiempo bloqueado, tiempo vacío, pallets procesados, rendimiento (pallets/h), tiempo medio de tránsito, ocupación media.

## 2. Parámetros de rodillos

Diámetro y paso **se eligen de una lista corta de opciones estándar**, no se escriben libremente.

| Clave               | Clave del plan        | Unidad | Defecto         | Opciones / rango                     | Descripción                                                   |
| ------------------- | --------------------- | ------ | --------------- | ------------------------------------ | ------------------------------------------------------------- |
| `rollerDiameter`    | `diametroRodillo`     | mm     | **89**          | 76, 89 (lista provisional)           | Diámetro exterior del rodillo                                 |
| `rollerPitch`       | `pasoRodillos`        | mm     | **206,3**       | 133, 206,3 (lista provisional)       | Distancia entre ejes                                          |
| `rollerMaterial`    | `materialRodillo`     | enum   | acero           | acero, acero recubierto, PVC         | Define el par de fricción                                     |
| `rollerMu`          | `muRodillo`           | —      | 0,35            | 0,1 … 0,8                            | Rozamiento pallet–rodillo                                     |
| `rollingResistance` | `resistenciaRodadura` | —      | 0,02            | 0 … 0,1                              | Pérdida por rodadura (rodillos libres)                        |
| `drive`             | `accionamiento`       | enum   | cadena continua | gravedad, cadena continua, por zonas | Tren único por cadena, zonas independientes o rodillos libres |
| `zoneLength`        | `longitudZona`        | mm     | 1400            | 1000 … 2000                          | Solo por zonas                                                |
| `zoneEndSensor`     | `sensorFinDeZona`     | bool   | sí              | —                                    | Sensor virtual de fin de zona para ZPA                        |
| `rollerInertia`     | `inerciaRodillo`      | kg·m²  | auto            | —                                    | Solo física                                                   |

Los valores 76/133 vienen del plan; 89/206,3 son los de tu equipo. La lista final es una pregunta abierta ([[Preguntas abiertas]]). 206,3 mm es casi exactamente 13 × 15,875 mm (13 eslabones de una cadena de 5/8"), lo que sugiere que el paso va ligado a la cadena de accionamiento; a confirmar.

**Derivados con los valores por defecto:** rodillos en una mesa de 3 m $\lfloor 3000/206{,}3 \rfloor$ = **14**; rodillos bajo un lado de 1200 mm $\lfloor 1200/206{,}3 \rfloor$ = **5** (≥ 3 ✓); hueco libre entre rodillos 206,3 − 89 = **117,3 mm**; velocidad del rodillo para 0,25 m/s $\omega = 0{,}25/0{,}0445$ = **5,62 rad/s**; capacidad de una mesa de 3 m $\lfloor 3000/(1200+150) \rfloor$ = **2** pallets EUR.

## 3. Comportamiento de acumulación (por instancia)

| Modo                   | Comportamiento                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **none** (por defecto) | Sin acumulación. Cada pallet del transportador sigue el movimiento de los rodillos: el transportador marcha o para **como una unidad**. Cuando entra un pallet nuevo, el transportador tiene que marchar, así que los pallets que ya están **se mueven también**. Las separaciones entre pallets se mantienen (nadie se junta). Cuando el pallet delantero llega a la salida y el siguiente transportador no puede recibirlo, se para todo el transportador (estado *bloqueado*) y no puede recibir más pallets hasta que pueda volver a marchar. |
| zpa                    | Acumulación sin presión: exige `drive = zoned`; cada zona marcha solo si la siguiente está libre; los pallets nunca se tocan ([[T05 Acumulación, capacidad y rendimiento]] §6.2).                                                                                                                                                                                                                                                                                                                                                                 |
| pressure               | Los pallets se juntan hasta `minGap` (o contacto) detrás de un pallet retenido.                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |

**Modelo 2D de `none`.** Todos los pallets del transportador comparten el perfil de movimiento del transportador (un único segmento cinemático para todo el transportador, desplazado para cada pallet). Eventos predichos: el pallet delantero llega a la salida (entrega o bloqueo), la cola del último pallet libera la entrada (la entrada vuelve a estar libre), flancos de sensores de cada pallet.

## 4. Puertos, comportamiento 3D y validaciones

- **Puertos:** A (entrada) en (0, surfaceHeight, 0); B (salida) en (length, surfaceHeight, 0).
- **3D:** cada rodillo es un cilindro cinemático con $\omega = v/(D/2)$, agrupado por zona; el pallet avanza por rozamiento con sus tablas inferiores y puede patinar ([[T12 Rodillos y cadenas en física]]).
- **Validaciones:** regla de 3 rodillos (`min3RollersUnderPallet`); ancho útil ≥ ancho del pallet + 2 × holgura de guía (`usableWidthFitsPallet`; EUR longitudinal: 800 + 50 ≤ 1000 ✓); longitud de zona ≥ largo del pallet + 100 mm.

## 5. Pruebas

| Prueba | Esperado                                                                                      | Estado                |
| ------ | --------------------------------------------------------------------------------------------- | --------------------- |
| T1     | 3 m desde reposo, 0,25 m/s, 0,5 m/s² → 12,25 s                                                | ✅ fórmula            |
| T2     | EUR lado largo + 0,15 m → ≈ 667 pallets/h                                                     | ✅ fórmula            |
| —      | Valores por defecto: Ø89, paso 206,3, acumulación `none`                                      | ✅ prueba de librería |
| T6     | Sensor a 2,0 m, banda continua → 4,8 s ocupado                                                | ⏳ fase 3             |
| —      | `none`: el pallet que entra mueve al que ya está; salida bloqueada para todo el transportador | ⏳ fase 3             |
| S1-a   | 1000 kg sobre 10 rodillos cinemáticos → velocidad nominal ±3 %                                | ⏳ spike S1           |

## Relacionado

[[T04 Cinemática de transportadores]] · [[T05 Acumulación, capacidad y rendimiento]] · [[T12 Rodillos y cadenas en física]] · [[A05 Librería de componentes]] · [[Preguntas abiertas]]
