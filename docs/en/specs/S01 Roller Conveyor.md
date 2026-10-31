---
title: S01 Roller Conveyor
lang: en
tags: [spec, conveyor, roller]
library-id: conveyor.roller
status: definition-json
updated: 2026-10-29
---

🌐 **EN** · [[S01 Mesa de rodillos|ES]] · [[S01 Rollenförderer|DE]] · [[Home (EN)|Index]]

# S01 Roller Conveyor (`conveyor.roller`)

Straight roller section carrying the pallet on its bottom boards in the direction of travel. **EUR pallets travel longitudinally (long side, 1200 mm, first) on roller conveyors** — confirmed by the project owner on 2026-10-29. Base component: the others reuse its roller group. Definition: `packages/library/src/definitions/roller-conveyor.json`.

Defaults are **indicative**; values shown in display units, stored in SI. **Every behaviour parameter belongs to the instance**: two roller conveyors of the same type can have different lengths, capacities and accumulation modes.

## 1. Common parameters (all conveyors)

| Key                     | Plan key                       | Unit  | Default                | Range                        | Description                                                                |
| ----------------------- | ------------------------------ | ----- | ---------------------- | ---------------------------- | -------------------------------------------------------------------------- |
| `name`                  | `nombre`                       | text  | auto                   | —                            | Visible name; the internal `id` is generated                               |
| `position`              | `posicion`                     | m     | 0, 0, 0                | —                            | Origin (centre of the start)                                               |
| `yaw`                   | `giro`                         | °     | 0                      | −180 … 180                   | Rotation about the vertical axis                                           |
| `surfaceHeight`         | `alturaSuperficie`             | mm    | 600                    | 200 … 1500                   | Height of the pallet support plane above the floor                         |
| `length`                | `longitud`                     | mm    | 3000                   | 500 … 12000                  | Length in the direction of travel; longer conveyors hold more pallets      |
| `usableWidth`           | `anchoUtil`                    | mm    | 1000                   | 500 … 1800                   | Clear width between frames or guides                                       |
| `palletSideAlongTravel` | `ladoParaleloAvance`           | enum  | long                   | long, short                  | Long for roller conveyors (EUR 1200 mm along travel)                       |
| `nominalSpeed`          | `velocidadNominal`             | m/s   | 0.25                   | 0.05 … 0.60                  | Cruise speed                                                               |
| `approachSpeed`         | `velocidadAproximacion`        | m/s   | 0.10                   | 0.02 … 0.30                  | Slow speed before a stop                                                   |
| `accel` / `decel`       | `aceleracion` / `deceleracion` | m/s²  | 0.5 / 0.5              | 0.05 … 2.0                   | Start-up and stopping ramps                                                |
| `direction`             | `sentido`                      | enum  | forward                | forward, reverse, reversible | Direction of travel                                                        |
| `maxPalletLoad`         | `cargaMaxPallet`               | kg    | 1000                   | 100 … 2000                   | Warns if a pallet is heavier                                               |
| `minGap`                | `separacionMinima`             | mm    | 150                    | 50 … 500                     | Minimum clearance between pallets (used by ZPA / pressure and by capacity) |
| `accumulationMode`      | `modoAcumulacion`              | enum  | **none**               | none, zpa, pressure          | See §3                                                                     |
| `guides`                | `guias`                        | group | see [[S08 Side Guide]] | —                            | Integrated side guides                                                     |
| `enabled`               | `habilitado`                   | bool  | yes                    | —                            | Disable without deleting                                                   |
| `color`, `legs`         | `color`, `patas`               | —     | —                      | —                            | Appearance only                                                            |
| `failures`              | `fallos`                       | —     | off                    | —                            | Reserved (MTBF/MTTR, post-MVP)                                             |

**Common signals:** inputs `run`, `reverse`, `enable`, per zone `zone[i].run`; outputs `running`, `occupied`, `zone[i].occupied`, `fault` (reserved). **States (2D):** stopped, accelerating, running, decelerating, blocked. **Statistics:** utilisation, blocked time, empty time, pallets processed, throughput (pallets/h), mean transit time, mean occupancy.

## 2. Roller parameters

Diameter and pitch are **chosen from a short list of standard options**, not typed freely.

| Key                 | Plan key               | Unit  | Default         | Options / range                         | Description                                                                                         |
| ------------------- | ---------------------- | ----- | --------------- | --------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `rollerDiameter`    | `diametroRodillo`      | mm    | **89**          | 76, 89 (provisional list)               | Outer roller diameter                                                                               |
| `rollerPitch`       | `pasoRodillos`         | mm    | **206.3**       | 133, 206.3 (provisional list)           | Axis-to-axis distance                                                                               |
| `rollerMaterial`    | `materialRodillo`      | enum  | steel           | steel, coated steel, PVC                | Selects the friction pair                                                                           |
| `rollerMu`          | `muRodillo`            | —     | 0.35            | 0.1 … 0.8                               | Pallet–roller friction                                                                              |
| `rollingResistance` | `resistenciaRodadura`  | —     | 0.02            | 0 … 0.1                                 | Rolling loss (idle rollers)                                                                         |
| `drive`             | `accionamiento`        | enum  | continuousChain | gravity, continuousChain, zoned         | One chain-driven train, independent zones or idle rollers                                           |
| `zoneLength`        | `longitudZona`         | mm    | 1400            | 1000 … 2000                             | Zoned drive only                                                                                    |
| `zoneEndSensor`     | `sensorFinDeZona`      | bool  | yes             | —                                       | Virtual end-of-zone sensor for ZPA                                                                  |
| `rollerInertia`     | `inerciaRodillo`       | kg·m² | auto            | —                                       | Physics only                                                                                        |
| `driveSide`         | `ladoAccionamiento`    | enum  | right           | right, left                             | Side of the drive and of the chain guard, seen in the direction of travel (drawing only)            |
| `driveMounting`     | `montajeAccionamiento` | enum  | inside          | inside, outside                         | Geared motor under the deck inside the frame, or on a drive roller outside the frame (drawing only) |
| `runnerGuide`       | `guiadoPatines`        | enum  | none            | none, outerOutside, outerInside, centre | Guide disks on every third roller that guide the EUR runners (drawing only)                         |
| `endStop`           | `topeFinal`            | bool  | no              | —                                       | Fixed plate at a free line end (drawing only; in 2D a pallet always stops at an open end)           |

The list values 76/133 come from the plan; 89/206.3 are the owner's equipment. The final list is an open question ([[Open Questions]]). 206.3 mm is very close to 13 × 15.875 mm (13 links of a 5/8" chain), which suggests the pitch is tied to the drive chain — to confirm.

**Drawing (since 2026-10-30).** The 3D model shows how heavy pallet roller conveyors are usually built (indicative):

- a chain guard on top of the frame on the drive side;
- one geared motor per conveyor;
- feet at most 2 m apart, each on an adjustable foot with a floor plate;
- a cross beam between the feet from 650 mm conveying height, and diagonal bracing from 850 mm.

These choices do not change any 2D result.

**Derived with the defaults:** rollers on a 3 m conveyor $\lfloor 3000/206.3 \rfloor$ = **14**; rollers under a 1200 mm pallet side $\lfloor 1200/206.3 \rfloor$ = **5** (≥ 3 ✓); free gap between rollers 206.3 − 89 = **117.3 mm**; roller speed for 0.25 m/s $\omega = 0.25/0.0445$ = **5.62 rad/s**; capacity of a 3 m conveyor $\lfloor 3000/(1200+150) \rfloor$ = **2** EUR pallets.

## 3. Accumulation behaviour (per instance)

| Mode               | Behaviour                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **none** (default) | No accumulation. Every pallet on the conveyor moves with the rollers: the conveyor runs or stops **as one unit**. When a new pallet enters, the conveyor must run, so the pallets already on it **move too**. Gaps between pallets stay as they were (nobody closes up). When the front pallet reaches the exit and the next conveyor cannot take it, the whole conveyor stops (state *blocked*) and cannot receive more pallets until it can run again. |
| zpa                | Zero-pressure accumulation: requires `drive = zoned`; each zone runs only if the next one is free; pallets never touch ([[T05 Accumulation, Capacity and Throughput]] §6.2).                                                                                                                                                                                                                                                                             |
| pressure           | Pallets are pushed together down to `minGap` (or contact) behind a held pallet.                                                                                                                                                                                                                                                                                                                                                                          |

**2D model of `none`.** All pallets on the conveyor share the conveyor's motion profile (one kinematic segment for the whole conveyor, shifted per pallet). Predicted events: front pallet reaches the exit (hand-over or block), tail of the last pallet clears the entry (the entry is free again), sensor edges of every pallet.

## 4. Ports, behaviour in 3D, validations

- **Ports:** A (in) at (0, surfaceHeight, 0); B (out) at (length, surfaceHeight, 0).
- **3D:** each roller is a kinematic cylinder with $\omega = v/(D/2)$, grouped by zone; the pallet moves by friction on its bottom boards and can slip ([[T12 Rollers and Chains in Physics]]).
- **Validations:** 3-roller rule (`min3RollersUnderPallet`); usable width ≥ pallet width + 2 × guide clearance (`usableWidthFitsPallet`, EUR longitudinal: 800 + 50 ≤ 1000 ✓); zone length ≥ pallet length + 100 mm.

## 5. Tests

| Test | Expected                                                                                 | Status          |
| ---- | ---------------------------------------------------------------------------------------- | --------------- |
| T1   | 3 m from rest, 0.25 m/s, 0.5 m/s² → 12.25 s                                              | ✅ formula + 2D |
| T2   | EUR long side + 0.15 m gap → ≈ 667 pallets/h                                             | ✅ formula + 2D |
| —    | Defaults: Ø89, pitch 206.3, accumulation `none`                                          | ✅ library test |
| T6   | Sensor at 2.0 m, continuous band → 4.8 s occupied                                        | ⏳ phase 3      |
| —    | `none`: entering pallet moves the resident pallet; blocked exit stops the whole conveyor | ✅ 2D flow test |
| S1-a | 1000 kg on 10 kinematic rollers → nominal speed ±3 %                                     | ⏳ spike S1     |

## Related

[[T04 Conveyor Kinematics]] · [[T05 Accumulation, Capacity and Throughput]] · [[T12 Rollers and Chains in Physics]] · [[A05 Component Library]] · [[Open Questions]]
