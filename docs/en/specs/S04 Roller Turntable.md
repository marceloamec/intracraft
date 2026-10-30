---
title: S04 Roller Turntable
lang: en
tags: [spec, conveyor, turntable, roller]
library-id: conveyor.roller_turntable
status: spec
---

🌐 **EN** · [[S04 Giratoria de rodillos|ES]] · [[S04 Rollendrehtisch|DE]] · [[Home (EN)|Index]]

# S04 Roller Turntable (`conveyor.roller_turntable`)

The same rotating platform as [[S03 Chain Turntable]] with a roller section on top — the usual choice when both incoming and outgoing lines are roller conveyors. Uses the *platform and turn* group of S03 and the roller group of [[S01 Roller Conveyor]], plus:

| Key                     | Plan key                | Unit | Default | Range               | Description                                                |
| ----------------------- | ----------------------- | ---- | ------- | ------------------- | ---------------------------------------------------------- |
| `lockRollersDuringTurn` | `bloqueoRodillosEnGiro` | bool | yes     | —                   | Rollers braked during the turn so the pallet does not roll |
| `holdDuringTurn`        | `retencionEnGiro`       | enum | none    | none, platformStops | Stops that hold the pallet during the turn                 |
| `loadSpeed`             | `velocidadCarga`        | m/s  | 0.25    | 0.05 … 0.6          | Roller speed while receiving and discharging               |

## Behaviour

**2D.** As S03: the pallet travels with the platform; turn time from the angular profile (T3).

**3D.** With rollers locked ($\omega = 0$) the contact is **sliding** friction with μ, not rolling; with free rollers the pallet would roll and shift during angular acceleration — hence locking is the default ([[T12 Rollers and Chains in Physics]], [[T13 Turntables and Transfer Lifts]]).

## Validations

If `lockRollersDuringTurn` is off and `holdDuringTurn` = none → warning: risk of pallet displacement. Plus all validations of S03.

## Tests

Same battery as S03 (T3, S1-e / P4).

2D behaviour: the same cycle as [[S03 Chain Turntable]] with the deck speed `loadSpeed` (implemented 2026-10-30, [[ADR-0008 Turntable 2D Cycle and Exit Rule]]). `lockRollersDuringTurn` and `holdDuringTurn` only matter in 3D.

## Related

[[S03 Chain Turntable]] · [[S01 Roller Conveyor]] · [[T13 Turntables and Transfer Lifts]]
