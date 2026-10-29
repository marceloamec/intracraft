---
title: S04 Rollendrehtisch
lang: de
tags: [spec, conveyor, turntable, roller]
library-id: conveyor.roller_turntable
status: spec
---

🌐 [[S04 Roller Turntable|EN]] · [[S04 Giratoria de rodillos|ES]] · **DE** · [[Startseite (DE)|Index]]

# S04 Rollendrehtisch (`conveyor.roller_turntable`)

Dieselbe Drehplattform wie [[S03 Kettendrehtisch]], aber mit einem Rollenabschnitt darauf – die übliche Wahl, wenn Zu- und Abförderung Rollenförderer sind. Nutzt die Gruppe *Plattform und Drehung* aus S03 und die Rollengruppe aus [[S01 Rollenförderer]], dazu:

| Schlüssel               | Plan-Schlüssel          | Einheit | Standard | Bereich                   | Beschreibung                                                       |
| ----------------------- | ----------------------- | ------- | -------- | ------------------------- | ------------------------------------------------------------------ |
| `lockRollersDuringTurn` | `bloqueoRodillosEnGiro` | bool    | ja       | —                         | Rollen während der Drehung gebremst, damit die Palette nicht rollt |
| `holdDuringTurn`        | `retencionEnGiro`       | Enum    | keine    | keine, Plattformanschläge | Anschläge, die die Palette beim Drehen halten                      |
| `loadSpeed`             | `velocidadCarga`        | m/s     | 0,25     | 0,05 … 0,6                | Rollengeschwindigkeit beim Aufnehmen und Abgeben                   |

## Verhalten

**2D.** Wie S03: Die Palette fährt mit der Plattform; die Drehzeit ergibt sich aus dem Winkelprofil (T3).

**3D.** Mit blockierten Rollen ($\omega = 0$) ist der Kontakt **Gleitreibung** mit μ, nicht Rollen; mit freien Rollen würde die Palette bei der Winkelbeschleunigung rollen und verrutschen – daher ist Blockieren der Standard ([[T12 Rollen und Ketten in der Physik]], [[T13 Drehtische und Kettenumsetzer]]).

## Validierungen

Ist `lockRollersDuringTurn` aus und `holdDuringTurn` = keine → Warnung: Gefahr des Verrutschens. Dazu alle Validierungen aus S03.

## Tests

Dieselbe Testreihe wie S03 (T3, S1-e / P4).

## Verwandt

[[S03 Kettendrehtisch]] · [[S01 Rollenförderer]] · [[T13 Drehtische und Kettenumsetzer]]
