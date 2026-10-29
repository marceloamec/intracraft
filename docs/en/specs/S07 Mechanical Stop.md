---
title: S07 Mechanical Stop
lang: en
tags: [spec, mechanics, stop]
library-id: mechanics.stop
status: spec
---

🌐 **EN** · [[S07 Tope mecánico|ES]] · [[S07 Mechanischer Anschlag|DE]] · [[Home (EN)|Index]]

# S07 Mechanical Stop (`mechanics.stop`)

Theory: [[T14 Stops, Impacts and Alignment]].

## 1. Parameters

| Key               | Plan key            | Unit    | Default     | Range              | Description                                                       |
| ----------------- | ------------------- | ------- | ----------- | ------------------ | ----------------------------------------------------------------- |
| `type`            | `tipo`              | enum    | retractable | fixed, retractable | Fixed is always up; retractable has an actuator                   |
| `host`, `s`       | `anfitrion`, `s`    | ref, mm | —           | —                  | Position of the contact plane along the conveyor                  |
| `stopWidth`       | `anchoTope`         | mm      | 100         | 20 … 400           | Transverse width; a wide stop or two stops align the front better |
| `stopHeight`      | `alturaTope`        | mm      | 100         | 30 … 300           | Height above the conveying surface                                |
| `raiseTime`       | `tiempoSubida`      | s       | 0.3         | 0.05 … 2           | Time to rise                                                      |
| `lowerTime`       | `tiempoBajada`      | s       | 0.3         | 0.05 … 2           | Time to lower                                                     |
| `initialState`    | `estadoInicial`     | enum    | up          | up, down           | State at start                                                    |
| `restitution`     | `restitucion`       | —       | 0.1         | 0 … 0.5            | Pallet rebound on impact                                          |
| `stopMu`          | `muTope`            | —       | 0.3         | 0.1 … 0.6          | Friction with the pallet                                          |
| `maxImpactEnergy` | `energiaMaxImpacto` | J       | 200         | 10 … 2000          | Warn if impact kinetic energy exceeds it                          |
| `commandSignal`   | `señalMando`        | text    | auto        | —                  | Boolean input (up or down)                                        |
| `positionSignals` | `señalPosicion`     | bool    | no          | —                  | Publishes up/down limit switches                                  |

## 2. Impact energy

$E = \tfrac12 m v^2$: 1025 kg (pallet + load) → **32 J** at 0.25 m/s, 128 J at 0.5 m/s (T5). The simulator records speed and energy of each impact.

## 3. Behaviour

**2D.** Position limit for the pallet front; raise/lower times delay the effect; on chains the pallet stops over `stopContactTime` ([[S02 Chain Conveyor]]).
**3D.** Kinematic body with a collider; supports rebound and misalignment. Engine restitution threshold and slop must be configured ([[A07 Physics Engine Wrapper]]).

## 4. Interlock

Do not raise under a pallet (zone above the stop must be clear).

## 5. Tests

| Test      | Criterion                                    | Status          |
| --------- | -------------------------------------------- | --------------- |
| T5        | 1025 kg at 0.25 m/s → 32 J                   | ✅ formula      |
| P1 / S1-c | Front ±5 mm, rebound < 10 mm                 | ⏳ S1 / phase 4 |
| P2        | Speed and energy recorded; warning above max | ⏳ phase 4      |

## Related

[[T14 Stops, Impacts and Alignment]] · [[T11 Contact, Friction and Restitution]]
