---
title: S03 Chain Turntable
lang: en
tags: [spec, conveyor, turntable, chain]
library-id: conveyor.chain_turntable
status: spec
---

🌐 **EN** · [[S03 Giratoria de cadenas|ES]] · [[S03 Kettendrehtisch|DE]] · [[Home (EN)|Index]]

# S03 Chain Turntable (`conveyor.chain_turntable`)

Rotating platform with a chain section on top. Receives a pallet by chains, turns to another station (usually 90° or 180°) and hands it to another chain line. Its cycle usually limits the throughput of the crossing. Theory: [[T13 Turntables and Transfer Lifts]].

## 1. Platform and turn parameters (shared with [[S04 Roller Turntable]])

| Key                      | Plan key                 | Unit  | Default         | Range          | Description                                       |
| ------------------------ | ------------------------ | ----- | --------------- | -------------- | ------------------------------------------------- |
| `platformLength`         | `largoPlataforma`        | mm    | 1400            | 1000 … 1800    | Platform size along travel                        |
| `platformWidth`          | `anchoPlataforma`        | mm    | 1200            | 800 … 1500     | Transverse size                                   |
| `pivotOffset` (dx, dz)   | `centroGiro`             | mm    | 0, 0            | ±300           | Pivot offset from platform centre                 |
| `turnMode`               | `modoGiro`               | enum  | stations        | stations, free | In *stations* it only stops at defined angles     |
| `stations`               | `estaciones`             | °     | 0, 90, 180, 270 | list           | Stop angles; each publishes a port                |
| `angularRange`           | `rangoAngular`           | °     | unlimited       | −720 … 720     | Mechanical limit (e.g. cables)                    |
| `shortestPath`           | `caminoMasCorto`         | bool  | yes             | —              | Turn in the direction of smaller angle            |
| `maxAngularSpeed`        | `velocidadAngularMax`    | °/s   | 15              | 2 … 45         | Cruise angular speed                              |
| `angularAccel`           | `aceleracionAngular`     | °/s²  | 30              | 5 … 120        | Start ramp                                        |
| `angularDecel`           | `deceleracionAngular`    | °/s²  | 30              | 5 … 120        | Stop ramp                                         |
| `settleTime`             | `tiempoAsentamiento`     | s     | 0.3             | 0 … 2          | Pause after reaching the station before discharge |
| `angularPrecision`       | `precisionAngular`       | °     | 0.2             | 0.05 … 2       | Position error when stopping                      |
| `sweepClearance`         | `holguraBarrido`         | mm    | 50              | 0 … 300        | Margin between swept circle and neighbours        |
| `maxLoad`                | `cargaMax`               | kg    | 1500            | 200 … 3000     | Pallet + load                                     |
| `requiresCenteredPallet` | `requierePalletCentrado` | bool  | yes             | —              | Interlock: no turn without a centred pallet       |
| `centeringTolerance`     | `toleranciaCentrado`     | mm    | ±20             | 5 … 50         | Allowed deviation from centre                     |
| `platformInertia`        | `inerciaPlataforma`      | kg·m² | auto            | —              | Physics: from mass and dimensions                 |
| `motorTorque`            | `parMotor`               | N·m   | 1500            | 100 … 10000    | Physics: max drive torque                         |
| `bearingFrictionTorque`  | `parFriccionCorona`      | N·m   | 20              | 0 … 200        | Physics: slewing-bearing friction                 |
| `sweptRadius` (derived)  | `radioBarrido`           | mm    | —               | —              | ½·√(L² + W²) + clearance → 972 mm with defaults   |
| `turnTime` (derived)     | `tiempoGiro`             | s     | —               | —              | 90° → 6.5 s with defaults (T3)                    |

Plus the **chain group** of [[S02 Chain Conveyor]] (`chainCount`, `chainPositions`, `chainSupportWidth`, `chainMu`, speed and ramps).

## 2. Formulas

- **Turn time:** triangular if $\omega^2/\alpha \ge \theta$: $t = 2\sqrt{\theta/\alpha}$; else $t = \theta/\omega + \omega/\alpha$. 90°, 15 °/s, 30 °/s² → **6.5 s**.
- **Friction check:** $\mu \ge \sqrt{a_t^2 + a_c^2}/g$ with $a_t = \alpha r$, $a_c = \omega^2 r$, $r$ = farthest pallet corner (0.72 m for a centred EUR) → **μ ≈ 0.04** needed.
- **Torque check:** $\tau = I_{total}\,\alpha + \tau_{bearing} \le$ `motorTorque`.

## 3. Sequence and ports

waiting → receiving (chains running) → centred → turning → settling → discharging → waiting. One port per station, defined in the platform frame; connected only at that station's angle.

## 4. Behaviour

**2D.** The pallet travels with the platform; the turn lasts what the angular profile gives; discharge is enabled after settling. Implemented 2026-10-30 ([[ADR-0008 Turntable 2D Cycle and Exit Rule]]). The exit is chosen by the new parameter `exitRule` (`alternate`, `firstExit`, `random`) until control rules exist.
**3D.** Kinematic platform with angular velocity about the vertical axis; chain strips on top; the pallet is dynamic and held by friction.

## 5. Validations

Swept radius vs neighbours · enough motor torque · required vs available friction.

## 6. Tests

| Test      | Criterion                                           | Status          |
| --------- | --------------------------------------------------- | --------------- |
| T3        | 90° turn → 6.5 s (+ 0.3 s settle in the 2D cycle)   | ✅ formula + 2D |
| S1-e / P4 | Pallet moves < 5 mm relative to platform during 90° | ⏳ phase 6      |

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                                                                                                                                                                                                                                    |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | Family of the library (category conveyor); its parameters form the attribute table (inherited ✔ / overridden ✖), its presets are the family's types; ports P0, P90, P180 and P270 (in and out); placed from the Project Browser, listed in the Simulation tree and in schedules                                                                                |
| Discrete simulation | Turntable cycle and exit rule ([[ADR-0008 Turntable 2D Cycle and Exit Rule]])                                                                                                                                                                                                                                                                                  |
| Physics simulation  | Since 2026-11-02 (ADR-0024 amendment): a kinematic platform turning about its centre along the 2D angular profile, its deck (chain strands with a moving surface) turning with it, the pallet held by friction alone with the deck stopped during the turn; the same cycle and exit rule as in 2D. REF-5 in 3D: cycle within 1 % of the 2D one, the same turns |

The behaviour above does not change. **Rust status:** not ported yet (library R1, 2D R3, 3D R7 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.

## Related

[[T13 Turntables and Transfer Lifts]] · [[S04 Roller Turntable]] · [[T05 Accumulation, Capacity and Throughput]]
