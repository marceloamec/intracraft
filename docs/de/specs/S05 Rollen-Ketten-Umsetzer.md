---
title: S05 Rollen-Ketten-Umsetzer
lang: de
tags: [spec, conveyor, transfer]
library-id: conveyor.transfer_rc
status: spec
---

🌐 [[S05 Roller-to-Chain Transfer|EN]] · [[S05 Transferencia rodillos-cadenas|ES]] · **DE** · [[Startseite (DE)|Index]]

# S05 Rollen-Ketten-Umsetzer (`conveyor.transfer_rc`)

**Interpretation (zu bestätigen, Plan §18):** eine Station mit zwei rechtwinkligen Fördersystemen. Ein Rollenabschnitt nimmt die Palette längs auf; zwischen den Rollen liegen hebbare Ketten in Querrichtung. Gehoben nehmen die Ketten die Palette von den Rollen, fahren sie 90° auf die benachbarte Kettenlinie und senken ab. Die Palette ändert die Richtung, ohne sich zu drehen. Theorie: [[T13 Drehtische und Kettenumsetzer]] Teil 2.

**Ports:** A und B (Rollen, längs), C1 und C2 (Ketten, links und rechts).

## 1. Parameter

| Schlüssel                | Plan-Schlüssel              | Einheit | Standard      | Bereich                                     | Beschreibung                                                                |
| ------------------------ | --------------------------- | ------- | ------------- | ------------------------------------------- | --------------------------------------------------------------------------- |
| Rollengruppe             | —                           | —       | —             | —                                           | Wie [[S01 Rollenförderer]]; `length` ≈ Palettenlänge + 200 mm               |
| `transferChainCount`     | `numCadenasTransfer`        | —       | 3             | 2 … 5                                       | Anzahl Hubketten                                                            |
| `transferChainPositions` | `posicionesCadenasTransfer` | mm      | auto          | —                                           | Längsposition jeder Kette, zwischen den Rollen, an der Palette ausgerichtet |
| `transferChainWidth`     | `anchoCadenaTransfer`       | mm      | 40            | 20 … 80                                     | Breite je Kette                                                             |
| `liftStroke`             | `carreraElevacion`          | mm      | 40            | 20 … 100                                    | Höhe der Ketten über den Rollen im gehobenen Zustand                        |
| `lowerClearance`         | `holguraBajada`             | mm      | 10            | 0 … 30                                      | Abstand unter der Rollenebene im gesenkten Zustand                          |
| `liftTime`               | `tiempoSubida`              | s       | 1,0           | 0,3 … 5                                     | Hebezeit                                                                    |
| `lowerTime`              | `tiempoBajada`              | s       | 1,0           | 0,3 … 5                                     | Senkzeit                                                                    |
| `actuator`               | `actuador`                  | Enum    | pneumatisch   | pneumatisch, elektrisch                     | Pneumatisch: schnell mit Endlagendämpfung; elektrisch: mit Rampen           |
| `transferSpeed`          | `velocidadTransferencia`    | m/s     | 0,20          | 0,05 … 0,5                                  | Kettengeschwindigkeit beim Umsetzen                                         |
| `transferAccel`          | `aceleracionTransferencia`  | m/s²    | 0,5           | 0,1 … 2                                     | Kettenrampe                                                                 |
| `transferDistance`       | `distanciaTransferencia`    | mm      | auto          | —                                           | Kettenweg (≈ Tischbreite); editierbar                                       |
| `transferDirection`      | `direccionTransferencia`    | Enum    | rechts        | links, rechts, bidirektional                | Seite, zu der umgesetzt wird                                                |
| `operationMode`          | `modoOperacion`             | Enum    | Rollen→Ketten | Rollen→Ketten, Ketten→Rollen, bidirektional | Flussrichtung                                                               |
| `positionTolerance`      | `toleranciaPosicion`        | mm      | ±20           | 5 … 50                                      | Zulässige Palettenabweichung zum Heben                                      |
| `requiresStop`           | `requiereTope`              | bool    | ja            | —                                           | Enthält einen versenkbaren Positionieranschlag im Rollenteil                |
| `transferChainMu`        | `muCadenaTransfer`          | —       | 0,30          | 0,1 … 0,6                                   | Reibung Palette–Kette                                                       |

## 2. Ablauf und Verriegelungen

Ruhe (Ketten unten) → Palette in Position (Anschlag + Sensoren) → heben → oben → umsetzen → abgegeben → senken → Ruhe.

Verriegelungen: kein Heben ohne zentrierte Palette · Rollen stehen, solange die Ketten oben sind · nicht senken, bevor die Palette den Kettenbereich verlassen hat · im bidirektionalen Betrieb nie gleichzeitig von beiden Seiten annehmen.

## 3. Formeln

- **Idealzyklus:** $t = t_{Heben} + (d/v + v/a) + t_{Senken}$ (+ Beruhigen). 1 m, 0,2 m/s, 0,5 m/s² → 1,0 + 5,4 + 1,0 = **7,4 s** (T4).
- **Lückenprüfung:** Ø89 mm bei 206,3 mm Teilung (Projektwerte) → 117,3 mm Lücke ≥ Kette 40 mm + 10 mm ✓ (`chainFitsBetweenRollers`).
- **Höhenanschluss:** benachbarte Kettenlinie auf Rollenhöhe + `liftStroke` (Port-Toleranz ±10 mm).

## 4. Verhalten

**2D.** Zustandsautomat mit den obigen Dauern; während des Hebens ist der Rollenteil gesperrt.
**3D.** Die Ketten sind kinematische Körper, die hochfahren (vertikal) und dann horizontal fahren; die Palette wird über Kontakt angehoben. Der Simulator warnt, wenn eine Kette in eine Palettenlücke fällt und sie nicht berührt.

## 5. Tests

| Test | Kriterium                                                         | Stand      |
| ---- | ----------------------------------------------------------------- | ---------- |
| T4   | Zyklus 7,4 s                                                      | ✅ Formel  |
| P5   | 40 ± 5 mm gehoben, 1 m ± 10 mm verfahren, < 1° verdreht abgesetzt | ⏳ Phase 8 |

## Verwandt

[[T13 Drehtische und Kettenumsetzer]] · [[S01 Rollenförderer]] · [[S02 Kettenförderer]] · [[T07 Steuerungslogik und Signale]]
