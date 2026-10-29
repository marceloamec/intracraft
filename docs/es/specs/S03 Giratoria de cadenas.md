---
title: S03 Giratoria de cadenas
lang: es
tags: [spec, conveyor, turntable, chain]
library-id: conveyor.chain_turntable
status: spec
---

🌐 [[S03 Chain Turntable|EN]] · **ES** · [[S03 Kettendrehtisch|DE]] · [[Inicio (ES)|Índice]]

# S03 Giratoria de cadenas (`conveyor.chain_turntable`)

Plataforma giratoria con un tramo de cadenas encima. Recibe un pallet por cadenas, gira a otra estación (normalmente 90° o 180°) y lo entrega a otra línea de cadenas. Su ciclo suele condicionar el rendimiento del cruce. Teoría: [[T13 Giratorias y mesas de transferencia]].

## 1. Parámetros de plataforma y giro (compartidos con [[S04 Giratoria de rodillos]])

| Clave                    | Clave del plan           | Unidad | Defecto         | Rango             | Descripción                                        |
| ------------------------ | ------------------------ | ------ | --------------- | ----------------- | -------------------------------------------------- |
| `platformLength`         | `largoPlataforma`        | mm     | 1400            | 1000 … 1800       | Dimensión de la plataforma en el avance            |
| `platformWidth`          | `anchoPlataforma`        | mm     | 1200            | 800 … 1500        | Dimensión transversal                              |
| `pivotOffset` (dx, dz)   | `centroGiro`             | mm     | 0, 0            | ±300              | Desplazamiento del eje respecto al centro          |
| `turnMode`               | `modoGiro`               | enum   | estaciones      | estaciones, libre | En *estaciones* solo para en ángulos definidos     |
| `stations`               | `estaciones`             | °      | 0, 90, 180, 270 | lista             | Ángulos de parada; cada uno publica un puerto      |
| `angularRange`           | `rangoAngular`           | °      | ilimitado       | −720 … 720        | Límite mecánico (p. ej. cables)                    |
| `shortestPath`           | `caminoMasCorto`         | bool   | sí              | —                 | Girar por el ángulo menor                          |
| `maxAngularSpeed`        | `velocidadAngularMax`    | °/s    | 15              | 2 … 45            | Velocidad angular de crucero                       |
| `angularAccel`           | `aceleracionAngular`     | °/s²   | 30              | 5 … 120           | Rampa de arranque                                  |
| `angularDecel`           | `deceleracionAngular`    | °/s²   | 30              | 5 … 120           | Rampa de parada                                    |
| `settleTime`             | `tiempoAsentamiento`     | s      | 0,3             | 0 … 2             | Pausa tras llegar a la estación antes de descargar |
| `angularPrecision`       | `precisionAngular`       | °      | 0,2             | 0,05 … 2          | Error de posición al detenerse                     |
| `sweepClearance`         | `holguraBarrido`         | mm     | 50              | 0 … 300           | Margen entre el círculo de barrido y los vecinos   |
| `maxLoad`                | `cargaMax`               | kg     | 1500            | 200 … 3000        | Pallet + carga                                     |
| `requiresCenteredPallet` | `requierePalletCentrado` | bool   | sí              | —                 | Enclavamiento: no gira sin pallet centrado         |
| `centeringTolerance`     | `toleranciaCentrado`     | mm     | ±20             | 5 … 50            | Desviación admisible respecto al centro            |
| `platformInertia`        | `inerciaPlataforma`      | kg·m²  | auto            | —                 | Física: a partir de masa y dimensiones             |
| `motorTorque`            | `parMotor`               | N·m    | 1500            | 100 … 10000       | Física: par máximo del accionamiento               |
| `bearingFrictionTorque`  | `parFriccionCorona`      | N·m    | 20              | 0 … 200           | Física: fricción del rodamiento de giro            |
| `sweptRadius` (derivado) | `radioBarrido`           | mm     | —               | —                 | ½·√(L² + A²) + holgura → 972 mm por defecto        |
| `turnTime` (derivado)    | `tiempoGiro`             | s      | —               | —                 | 90° → 6,5 s por defecto (T3)                       |

Más el **grupo de cadenas** de [[S02 Mesa de cadenas]] (`chainCount`, `chainPositions`, `chainSupportWidth`, `chainMu`, velocidad y rampas).

## 2. Fórmulas

- **Tiempo de giro:** triangular si $\omega^2/\alpha \ge \theta$: $t = 2\sqrt{\theta/\alpha}$; si no, $t = \theta/\omega + \omega/\alpha$. 90°, 15 °/s, 30 °/s² → **6,5 s**.
- **Comprobación de fricción:** $\mu \ge \sqrt{a_t^2 + a_c^2}/g$ con $a_t = \alpha r$, $a_c = \omega^2 r$ y $r$ = esquina más alejada del pallet (0,72 m en un EUR centrado) → hace falta **μ ≈ 0,04**.
- **Comprobación de par:** $\tau = I_{total}\,\alpha + \tau_{corona} \le$ `motorTorque`.

## 3. Secuencia y puertos

esperando → recibiendo (cadenas en marcha) → centrado → girando → asentando → descargando → esperando. Un puerto por estación, definido en el marco de la plataforma; solo conectado en el ángulo de esa estación.

## 4. Comportamiento

**2D.** El pallet viaja con la plataforma; el giro dura lo que da el perfil angular y la descarga se habilita tras el asentamiento.
**3D.** Plataforma cinemática con velocidad angular sobre el eje vertical; tiras de cadena encima; el pallet es dinámico y se sostiene por fricción.

## 5. Validaciones

Radio de barrido frente a vecinos · par motor suficiente · fricción necesaria frente a la disponible.

## 6. Pruebas

| Prueba    | Criterio                                                     | Estado     |
| --------- | ------------------------------------------------------------ | ---------- |
| T3        | Giro de 90° → 6,5 s                                          | ✅ fórmula |
| S1-e / P4 | El pallet se desplaza < 5 mm respecto a la plataforma en 90° | ⏳ fase 6  |

## Relacionado

[[T13 Giratorias y mesas de transferencia]] · [[S04 Giratoria de rodillos]] · [[T05 Acumulación, capacidad y rendimiento]]
