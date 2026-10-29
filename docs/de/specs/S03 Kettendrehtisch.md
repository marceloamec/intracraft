---
title: S03 Kettendrehtisch
lang: de
tags: [spec, conveyor, turntable, chain]
library-id: conveyor.chain_turntable
status: spec
---

🌐 [[S03 Chain Turntable|EN]] · [[S03 Giratoria de cadenas|ES]] · **DE** · [[Startseite (DE)|Index]]

# S03 Kettendrehtisch (`conveyor.chain_turntable`)

Drehbare Plattform mit einem Kettenabschnitt darauf. Nimmt eine Palette per Kette auf, dreht zu einer anderen Station (meist 90° oder 180°) und gibt sie an eine andere Kettenlinie ab. Sein Zyklus begrenzt meist den Durchsatz der Kreuzung. Theorie: [[T13 Drehtische und Kettenumsetzer]].

## 1. Plattform- und Drehparameter (gemeinsam mit [[S04 Rollendrehtisch]])

| Schlüssel                  | Plan-Schlüssel           | Einheit | Standard        | Bereich         | Beschreibung                                        |
| -------------------------- | ------------------------ | ------- | --------------- | --------------- | --------------------------------------------------- |
| `platformLength`           | `largoPlataforma`        | mm      | 1400            | 1000 … 1800     | Plattformmaß in Förderrichtung                      |
| `platformWidth`            | `anchoPlataforma`        | mm      | 1200            | 800 … 1500      | Quermaß                                             |
| `pivotOffset` (dx, dz)     | `centroGiro`             | mm      | 0, 0            | ±300            | Versatz der Drehachse zur Plattformmitte            |
| `turnMode`                 | `modoGiro`               | Enum    | Stationen       | Stationen, frei | Bei *Stationen* nur Halt an definierten Winkeln     |
| `stations`                 | `estaciones`             | °       | 0, 90, 180, 270 | Liste           | Haltewinkel; jede Station veröffentlicht einen Port |
| `angularRange`             | `rangoAngular`           | °       | unbegrenzt      | −720 … 720      | Mechanische Grenze (z. B. Kabel)                    |
| `shortestPath`             | `caminoMasCorto`         | bool    | ja              | —               | In Richtung des kleineren Winkels drehen            |
| `maxAngularSpeed`          | `velocidadAngularMax`    | °/s     | 15              | 2 … 45          | Dreh-Fahrgeschwindigkeit                            |
| `angularAccel`             | `aceleracionAngular`     | °/s²    | 30              | 5 … 120         | Anfahrrampe                                         |
| `angularDecel`             | `deceleracionAngular`    | °/s²    | 30              | 5 … 120         | Bremsrampe                                          |
| `settleTime`               | `tiempoAsentamiento`     | s       | 0,3             | 0 … 2           | Pause nach Erreichen der Station vor der Abgabe     |
| `angularPrecision`         | `precisionAngular`       | °       | 0,2             | 0,05 … 2        | Positionsfehler beim Halten                         |
| `sweepClearance`           | `holguraBarrido`         | mm      | 50              | 0 … 300         | Abstand zwischen Schwenkkreis und Nachbarn          |
| `maxLoad`                  | `cargaMax`               | kg      | 1500            | 200 … 3000      | Palette + Ladung                                    |
| `requiresCenteredPallet`   | `requierePalletCentrado` | bool    | ja              | —               | Verriegelung: keine Drehung ohne zentrierte Palette |
| `centeringTolerance`       | `toleranciaCentrado`     | mm      | ±20             | 5 … 50          | Zulässige Abweichung von der Mitte                  |
| `platformInertia`          | `inerciaPlataforma`      | kg·m²   | auto            | —               | Physik: aus Masse und Abmessungen                   |
| `motorTorque`              | `parMotor`               | N·m     | 1500            | 100 … 10000     | Physik: max. Antriebsmoment                         |
| `bearingFrictionTorque`    | `parFriccionCorona`      | N·m     | 20              | 0 … 200         | Physik: Reibung des Drehkranzlagers                 |
| `sweptRadius` (abgeleitet) | `radioBarrido`           | mm      | —               | —               | ½·√(L² + B²) + Freiraum → 972 mm bei Standardwerten |
| `turnTime` (abgeleitet)    | `tiempoGiro`             | s       | —               | —               | 90° → 6,5 s bei Standardwerten (T3)                 |

Dazu die **Kettengruppe** aus [[S02 Kettenförderer]] (`chainCount`, `chainPositions`, `chainSupportWidth`, `chainMu`, Geschwindigkeit und Rampen).

## 2. Formeln

- **Drehzeit:** dreieckig, falls $\omega^2/\alpha \ge \theta$: $t = 2\sqrt{\theta/\alpha}$; sonst $t = \theta/\omega + \omega/\alpha$. 90°, 15 °/s, 30 °/s² → **6,5 s**.
- **Reibungsprüfung:** $\mu \ge \sqrt{a_t^2 + a_c^2}/g$ mit $a_t = \alpha r$, $a_c = \omega^2 r$, $r$ = entfernteste Palettenecke (0,72 m bei mittiger EUR-Palette) → nötig **μ ≈ 0,04**.
- **Momentenprüfung:** $\tau = I_{gesamt}\,\alpha + \tau_{Lager} \le$ `motorTorque`.

## 3. Ablauf und Ports

wartend → aufnehmend (Ketten laufen) → zentriert → drehend → beruhigend → abgebend → wartend. Ein Port je Station, im Plattformsystem definiert; nur beim Winkel dieser Station verbunden.

## 4. Verhalten

**2D.** Die Palette fährt mit der Plattform; die Drehung dauert so lange, wie das Winkelprofil ergibt; die Abgabe wird nach dem Beruhigen freigegeben.
**3D.** Kinematische Plattform mit Winkelgeschwindigkeit um die Hochachse; Kettenstreifen darauf; die Palette ist dynamisch und wird durch Reibung gehalten.

## 5. Validierungen

Schwenkradius gegen Nachbarn · ausreichendes Antriebsmoment · benötigte gegen verfügbare Reibung.

## 6. Tests

| Test      | Kriterium                                               | Stand      |
| --------- | ------------------------------------------------------- | ---------- |
| T3        | 90°-Drehung → 6,5 s                                     | ✅ Formel  |
| S1-e / P4 | Palette verschiebt sich bei 90° um < 5 mm zur Plattform | ⏳ Phase 6 |

## Verwandt

[[T13 Drehtische und Kettenumsetzer]] · [[S04 Rollendrehtisch]] · [[T05 Stau, Kapazität und Durchsatz]]
