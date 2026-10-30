---
title: S06 Photoelectric Sensor
lang: en
tags: [spec, sensor]
library-id: sensor.photoeye
status: spec
updated: 2026-10-29
---

🌐 **EN** · [[S06 Fotocélula|ES]] · [[S06 Lichtschranke|DE]] · [[Home (EN)|Index]]

# S06 Photoelectric Sensor (`sensor.photoeye`)

Theory: [[T06 Photoelectric Sensing]].

**Owner requirement (2026-10-29):** the position **and the height** of the photocell and of its reflector are parameters of each instance — the user chooses them. The beam is therefore the segment between two points chosen by the user, not a fixed horizontal line.

**Implemented 2026-10-30 (2D):**
- Definition `sensor.photoeye` (`photoeye.json`); behaviour in `packages/sim-des/src/flow/photoeye.ts`.
- **Mounting.** Drag the sensor onto a roller or chain conveyor. That conveyor becomes `host`, and the drop point sets `sensorS` = `reflectorS`.
- **Defaults.** The sensor and reflector sit at ∓800 mm lateral, both at 60 mm high; two presets give 130 mm (rollers) and 110 mm (chains).
- **Simplifications:**
  - `diffuse` is treated as a beam straight across the conveyor;
  - an inclined beam is evaluated at its mean height, with a warning;
  - `orientation`, `spotDiameter` and `cycleDelay` are not used yet.
- **`filter` is an off-delay.** The output switches on after `responseTime`, and switches off only when the beam has stayed free for `responseTime + filter`. A symmetric minimum-state time would never switch on at 60 mm, where the solid parts last only 0.58 s, so it could not "bridge pallet voids" as §1 asks. The output therefore lasts `filter` longer than the pallet.
- **Run warnings:** `sensorWithoutHost`, `inclinedBeam`, `beamOutOfRange`, `beamInVoidBand` (with the void length and its duration at nominal speed).
- **Viewer.** The beam is drawn green while free and red while detecting. The sensor's statistics are pulses, detecting time and detecting share.

## 1. Parameters

| Key                | Plan key          | Unit | Default                      | Range                                 | Description                                                                                 |
| ------------------ | ----------------- | ---- | ---------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------- |
| `type`             | `tipo`            | enum | retroreflective (to confirm) | diffuse, retroreflective, throughBeam | Retroreflective: sensor + reflector; through-beam: emitter + receiver; diffuse: sensor only |
| `host`             | `anfitrion`       | ref  | —                            | —                                     | Conveyor it is mounted on (positions are in the host frame)                                 |
| `sensorS`          | `s`               | mm   | 0                            | 0 … host length                       | Sensor position along the host                                                              |
| `sensorLateral`    | `lateral`         | mm   | left frame (auto)            | any                                   | Sensor lateral position from the host axis                                                  |
| `sensorHeight`     | `altura`          | mm   | 60                           | 0 … 400                               | Sensor beam height above the conveying surface                                              |
| `reflectorS`       | — (new)           | mm   | = `sensorS`                  | 0 … host length                       | Reflector / receiver position along the host                                                |
| `reflectorLateral` | — (new)           | mm   | right frame (auto)           | any                                   | Reflector / receiver lateral position                                                       |
| `reflectorHeight`  | — (new)           | mm   | = `sensorHeight`             | 0 … 400                               | Reflector / receiver height                                                                 |
| `orientation`      | `orientacion`     | °    | 90                           | 0 … 180                               | Diffuse only: beam direction relative to travel                                             |
| `range`            | `alcance`         | mm   | 1000 (diffuse) / 5000        | 50 … 5000                             | Maximum detection distance                                                                  |
| `spotDiameter`     | `diametroSpot`    | mm   | 10                           | 1 … 50                                | Beam diameter                                                                               |
| `responseTime`     | `tiempoRespuesta` | ms   | 2                            | 0 … 50                                | Sensor's own delay                                                                          |
| `polarity`         | `polaridad`       | enum | active on detect             | active on detect, active on dark      | Output logic                                                                                |
| `filter`           | `filtro`          | ms   | 0                            | 0 … 2000                              | Minimum time to change state; can bridge pallet voids                                       |
| `cycleDelay`       | `retardoCiclo`    | ms   | 10                           | 0 … 100                               | Controller sampling (PLC cycle)                                                             |
| `outputSignal`     | `señalSalida`     | text | auto                         | —                                     | Name of the boolean signal                                                                  |
| `showBeam`         | `mostrarHaz`      | bool | yes                          | —                                     | Draw the beam in 3D (green free, red detecting)                                             |

`reflector*` parameters are hidden for `diffuse` (`dependsOn: type≠diffuse`). The editor places both ends with handles in the 3D view and shows the values in mm.

**Derived (read-only):** beam length, beam angle to travel (90° when `sensorS = reflectorS`), beam inclination (0° when both heights are equal).

## 2. Validations and warnings

- Beam length ≤ `range`.
- The beam must cross the space a pallet occupies on the host; otherwise it can never detect.
- **Void warning:** for the pallet type and orientation expected on the host, if the beam height falls in a band with voids, warn with the expected gap duration at nominal speed and suggest a height in a continuous band or a `filter` value. EUR examples ([[S10 Pallet Types]]):

| Host (EUR orientation)    | Continuous bands (no flicker)              | At 60 mm (plan default)               |
| ------------------------- | ------------------------------------------ | ------------------------------------- |
| Rollers (long side first) | 122–144 mm (top deck) → e.g. 130 mm        | Gaps of 382.5 mm → 1.53 s at 0.25 m/s |
| Chains (short side first) | 100–122 mm (stringer boards) → e.g. 110 mm | Gaps of 227.5 mm → 0.91 s at 0.25 m/s |

Above the pallet (> 144 mm) the beam sees the load instead, if there is one.

## 3. Behaviour

**2D.** The beam is the segment sensor → reflector. Each solid part of the pallet (from its occupancy profile) produces one interval of front positions during which the segment is blocked; edges are the interval limits, predicted analytically ([[T06 Photoelectric Sensing]] §4).
**3D.** Raycast (or shape-cast for large spots) along the same segment every physics step, edge time stamped at the step midpoint.

## 4. Tests

| Test | Criterion                                                                                          | Status     |
| ---- | -------------------------------------------------------------------------------------------------- | ---------- |
| T6   | Sensor and reflector at 2.0 m, beam in a continuous band → rising at front, falling at tail, 4.8 s | ✅ 2D test |
| —    | Beam at 60 mm on rollers with EUR → three pulses per pallet; with `filter` 2000 ms → one           | ✅ 2D test |
| —    | Oblique beam (different `sensorS` and `reflectorS`) → edges match the analytic intervals           | ✅ 2D test |
| P6   | 3D and 2D edges within ±50 ms                                                                      | ⏳ phase 4 |

## Related

[[T06 Photoelectric Sensing]] · [[S10 Pallet Types]] · [[T07 Control Logic and Signals]] · [[Open Questions]]
