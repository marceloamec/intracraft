---
title: T13 Drehtische und Kettenumsetzer
lang: de
tags: [theory, des, physics, turntable, transfer]
---

🌐 [[T13 Turntables and Transfer Lifts|EN]] · [[T13 Giratorias y mesas de transferencia|ES]] · **DE** · [[Startseite (DE)|Index]]

# T13 Drehtische und Kettenumsetzer

Beide Komponenten ändern die Förderrichtung um 90° (oder 180°), bedienen **eine Palette nach der anderen** und sind meist die Engpässe eines Layouts ([[T05 Stau, Kapazität und Durchsatz]] §3).

## Teil 1 — Drehtische

### 1.1 Winkelprofil

Eine Drehung um $\theta$ mit Höchstgeschwindigkeit $\omega$ und Rampen $\alpha$ nutzt die Trapez-/Dreiecksformeln aus [[T04 Kinematik von Förderern]] §4: 90° bei 15 °/s mit 30 °/s² dauert **6,5 s** (T3).

**Drehrichtung wählen.** Mit `shortestPath` ist der vorzeichenbehaftete Winkel
$$\Delta\theta = \big((\theta_{Ziel} - \theta_{jetzt} + 180°) \bmod 360°\big) - 180°$$
Bei begrenztem `angularRange` (Kabel) muss der Drehtisch die erlaubte Richtung nehmen, auch wenn sie länger ist.

### 1.2 Stationen und Ports

Ports sind im **Plattformsystem** definiert. Ein Port gilt nur als verbunden, wenn der Plattformwinkel innerhalb von `angularPrecision` (Standard 0,2°) einem Stationswinkel entspricht. In 2D fährt die Palette einfach mit der Plattform mit; bei `turn_completed` + `settleTime` wird der Port der Station zur Abgabe freigegeben.

### 1.3 Schwenkradius

$$R_{Schwenk} = \tfrac12\sqrt{L_p^2 + W_p^2} + \text{Freiraum}$$

1400 × 1200 mm mit 50 mm Freiraum → **972 mm** (`sweptRadius`, getestet). Die Validierung prüft, dass kein Nachbar in diesem Kreis liegt.

### 1.4 Rutscht die Palette beim Drehen?

Ein Punkt im Radius $r$ auf einer drehenden Plattform hat die Tangentialbeschleunigung $a_t = \alpha r$ und die Zentripetalbeschleunigung $a_c = \omega^2 r$. Die Reibung muss beide aufbringen:

$$\mu \ge \frac{\sqrt{a_t^2 + a_c^2}}{g}$$

$r$ wird an der entferntesten Palettenecke genommen: EUR mittig, $r = \tfrac12\sqrt{1{,}2^2 + 0{,}8^2} = 0{,}72$ m. Mit Standardwerten ($\alpha = 30$ °/s² = 0,524 rad/s², $\omega = 15$ °/s = 0,262 rad/s): $a_t = 0{,}377$ m/s², $a_c = 0{,}049$ m/s² → **μ ≥ 0,039** (`turntableRequiredFriction`, getestet). Verfügbar sind 0,30–0,35: eine große Reserve, die mit steileren Rampen, größeren Paletten und außermittiger Lage (Zentriertoleranz ±20 mm) schrumpft.

(Konservativ: $a_t$ ist während der Rampe maximal, $a_c$ bei voller Drehzahl; beide Spitzen zu kombinieren überschätzt den Bedarf.)

### 1.5 Antriebsmoment

$$\tau = I_{gesamt}\,\alpha + \tau_{Lager}$$

Beispiel: Plattform 400 kg, 1,4 × 1,2 m → $I = 400(1{,}4^2 + 1{,}2^2)/12 = 113$ kg·m²; beladene EUR-Palette (625 kg) ≈ 108 kg·m² ([[T09 Starrkörperdynamik]]); gesamt ≈ 221 kg·m². Mit $\alpha = 0{,}524$ rad/s² und 20 N·m Lagerreibung: $\tau \approx 136$ N·m, weit unter dem Standard von 1500 N·m (`turntableRequiredTorque`). Eine außermittige Palette addiert $m d^2$ (Satz von Steiner).

### 1.6 Physikmodell

- Plattform: kinematischer Körper mit Winkelgeschwindigkeit $\omega(t)$ um die Hochachse.
- Ihre Rollen oder Ketten sind eigene kinematische Körper: In jedem Schritt berechnet der Wrapper ihre Lage aus dem Plattformwinkel und setzt ihre kinematischen Ziele so, dass ihre Geschwindigkeiten stimmen (ein ohne Geschwindigkeit „teleportierter“ Körper nimmt die Palette nicht mit).
- Rollendrehtisch: Rollen während der Drehung **blockiert** ($\omega_{Rolle} = 0$), damit Gleitreibung die Palette hält ([[T12 Rollen und Ketten in der Physik]]).
- Test S1-e / P4: Die Palette verschiebt sich bei einer 90°-Drehung um weniger als 5 mm gegenüber der Plattform.

### 1.7 Ablauf

wartend → aufnehmend (Förderer läuft) → zentriert (Sensoren) → drehend → beruhigend → abgebend → wartend. Verriegelung: keine Drehung, solange die Palette nicht zentriert und vollständig auf der Plattform ist.

## Teil 2 — Rollen-Ketten-Umsetzer

### 2.1 Mechanismus (Interpretation zu bestätigen, Plan §18)

Ein Rollenabschnitt nimmt die Palette **längs** auf. Zwischen den Rollen liegen **hebbare Kettenstränge** in Querrichtung. Beim Heben nehmen die Ketten die Palette von den Rollen ab, fahren sie 90° seitlich auf die benachbarte Kettenlinie und senken ab. Die Palette ändert die Richtung, ohne sich zu drehen.

```text
     Draufsicht                          Seitenansicht (Schnitt quer zu den Rollen)
  ┌─────────────────────┐
  │ ║ │ ║ │ ║ │ ║ │ ║ │ │  ← Rollen (│) und Ketten (║) in den Lücken     Palette ▄▄▄▄▄▄
  │ ║ │ ║ │ ║ │ ║ │ ║ │ │                                        gehoben: ║ ↑ 40 mm
  └─────────────────────┘                                        Rollen: ○ ○ ○ ○
     Förderung auf Rollen →     Umsetzen auf Ketten ↓
```

### 2.2 Geometrische Prüfungen

- **Ketten passen zwischen die Rollen:** Lücke $= p - D \ge b_{Kette} + 10$ mm. Ø89 bei 206,3 mm Teilung (Projektwerte) → 117,3 mm ≥ 40 + 10 ✓; Ø76 bei 133 mm aus dem Plan → 57 mm ✓ (`chainFitsBetweenRollers`, getestet).
- **Ketten berühren die Bretter:** Liegt die Palette längs auf den Rollen, verlaufen ihre Bodenbretter in Förderrichtung, und jede Querkette kreuzt alle ✓. Wäre die Palette um 90° gedreht, lägen die Bretter parallel zu den Ketten, und eine Kette könnte **in eine Lücke** zwischen den Brettern fallen – der Simulator warnt.
- **Höhenanschluss:** Gehobene Ketten liegen `liftStroke` (40 mm) über der Rollenebene; der anschließende Kettenförderer muss also auf Rollenhöhe + 40 mm liegen (Port-Toleranz ±10 mm).

### 2.3 Zykluszeit — Test T4

$$t = t_{Heben} + \frac{d}{v} + \frac{v}{a} + t_{Senken}$$

1 s + (5 + 0,4) s + 1 s = **7,4 s** für 1 m bei 0,2 m/s.

### 2.4 Anheben über Kontakt

Die Ketten sind kinematische Körper, die in `liftTime` um `liftStroke` + `lowerClearance` (50 mm) hochfahren. Sie heben die Palette, indem sie gegen die Bretter drücken. Stoppten sie oben schlagartig, flöge die Palette um $v^2/2g$ weiter; bei 0,05 m/s sind das 0,13 mm – vernachlässigbar. Ein Pneumatikantrieb hat Endlagendämpfung, ein elektrischer Rampen (Parameter `actuator`).

Test P5: gehoben 40 ± 5 mm, 1 m ± 10 mm verfahren, mit < 1° Verdrehung abgesetzt.

### 2.5 Verriegelungen

Kein Heben ohne zentrierte Palette (Anschlag + Sensoren, Toleranz ±20 mm) · Rollen stehen, solange die Ketten oben sind · nicht senken, bevor die Palette die Ketten verlassen hat · im bidirektionalen Betrieb nie von beiden Seiten annehmen.

## Verwandt

[[T04 Kinematik von Förderern]] · [[T07 Steuerungslogik und Signale]] · [[S03 Kettendrehtisch]] · [[S04 Rollendrehtisch]] · [[S05 Rollen-Ketten-Umsetzer]]
