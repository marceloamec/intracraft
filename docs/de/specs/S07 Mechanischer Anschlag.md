---
title: S07 Mechanischer Anschlag
lang: de
tags: [spec, mechanics, stop]
library-id: mechanics.stop
status: spec
---

🌐 [[S07 Mechanical Stop|EN]] · [[S07 Tope mecánico|ES]] · **DE** · [[Startseite (DE)|Index]]

# S07 Mechanischer Anschlag (`mechanics.stop`)

Theorie: [[T14 Anschläge, Aufprall und Ausrichtung]].

## 1. Parameter

| Schlüssel         | Plan-Schlüssel      | Einheit | Standard   | Bereich          | Beschreibung                                                   |
| ----------------- | ------------------- | ------- | ---------- | ---------------- | -------------------------------------------------------------- |
| `type`            | `tipo`              | Enum    | versenkbar | fest, versenkbar | Fest ist immer oben; versenkbar hat einen Antrieb              |
| `host`, `s`       | `anfitrion`, `s`    | Ref, mm | —          | —                | Lage der Kontaktebene entlang des Förderers                    |
| `stopWidth`       | `anchoTope`         | mm      | 100        | 20 … 400         | Querbreite; ein breiter oder zwei Anschläge richten besser aus |
| `stopHeight`      | `alturaTope`        | mm      | 100        | 30 … 300         | Höhe über der Förderebene                                      |
| `raiseTime`       | `tiempoSubida`      | s       | 0,3        | 0,05 … 2         | Hebezeit                                                       |
| `lowerTime`       | `tiempoBajada`      | s       | 0,3        | 0,05 … 2         | Senkzeit                                                       |
| `initialState`    | `estadoInicial`     | Enum    | oben       | oben, unten      | Zustand beim Start                                             |
| `restitution`     | `restitucion`       | —       | 0,1        | 0 … 0,5          | Rückprall der Palette beim Aufprall                            |
| `stopMu`          | `muTope`            | —       | 0,3        | 0,1 … 0,6        | Reibung mit der Palette                                        |
| `maxImpactEnergy` | `energiaMaxImpacto` | J       | 200        | 10 … 2000        | Warnt, wenn die kinetische Aufprallenergie größer ist          |
| `commandSignal`   | `señalMando`        | Text    | auto       | —                | Boolescher Eingang (oben oder unten)                           |
| `positionSignals` | `señalPosicion`     | bool    | nein       | —                | Veröffentlicht Endschalter oben/unten                          |

## 2. Aufprallenergie

$E = \tfrac12 m v^2$: 1025 kg (Palette + Ladung) → **32 J** bei 0,25 m/s, 128 J bei 0,5 m/s (T5). Der Simulator erfasst Geschwindigkeit und Energie jedes Aufpralls.

## 3. Verhalten

**2D.** Positionsgrenze der Palettenvorderkante; Hebe-/Senkzeiten verzögern die Wirkung; auf Ketten hält die Palette innerhalb `stopContactTime` ([[S02 Kettenförderer]]).
**3D.** Kinematischer Körper mit Kollisionskörper; erlaubt Rückprall und Schräglage. Restitutionsschwelle und Slop der Engine müssen eingestellt werden ([[A07 Physik-Wrapper]]).

## 4. Verriegelung

Nicht unter einer Palette heben (der Bereich über dem Anschlag muss frei sein).

## 5. Tests

| Test      | Kriterium                                                 | Stand           |
| --------- | --------------------------------------------------------- | --------------- |
| T5        | 1025 kg bei 0,25 m/s → 32 J                               | ✅ Formel       |
| P1 / S1-c | Vorderkante ±5 mm, Rückprall < 10 mm                      | ⏳ S1 / Phase 4 |
| P2        | Geschwindigkeit und Energie erfasst; Warnung über Maximum | ⏳ Phase 4      |

## Verwandt

[[T14 Anschläge, Aufprall und Ausrichtung]] · [[T11 Kontakt, Reibung und Restitution]]
