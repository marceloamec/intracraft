---
title: T10 Zeitintegration und fester Zeitschritt
lang: de
tags: [theory, physics, numerics]
---

🌐 [[T10 Time Integration and Fixed Step|EN]] · [[T10 Integración temporal y paso fijo|ES]] · **DE** · [[Startseite (DE)|Index]]

# T10 Zeitintegration und fester Zeitschritt

## 1. Von Gleichungen zu Schritten

Sobald Kontakte im Spiel sind, kann die Physik-Engine die Newton-Euler-Gleichungen ([[T09 Starrkörperdynamik]]) nicht geschlossen lösen. Sie schreitet den Zustand in kleinen Schritten $\Delta t$ fort.

| Integrator                | Aktualisierung                                                  | Verhalten                                                              |
| ------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Explizites Euler          | $x \mathrel{+}= v\,\Delta t$; dann $v \mathrel{+}= a\,\Delta t$ | Gewinnt Energie, Schwinger explodieren                                 |
| **Semi-implizites Euler** | $v \mathrel{+}= a\,\Delta t$; dann $x \mathrel{+}= v\,\Delta t$ | Symplektisch: Energie bleibt beschränkt; von Rapier und Jolt verwendet |
| Runge-Kutta 4             | Vier Auswertungen je Schritt                                    | Genau bei glatten DGL, nutzlos über Kontakte hinweg                    |

Semi-implizites Euler ist nur erster Ordnung, aber für einen harmonischen Schwinger **stabil**, solange $\omega_n\,\Delta t < 2$ – und bei Kontakten zählt Stabilität weit mehr als die Ordnung.

## 2. Warum der Schritt fest sein muss

1. **Determinismus** — ein anderes $\Delta t$ ergibt andere Kontaktimpulse und damit andere Ergebnisse. Ein bildratenabhängiger Schritt würde jeden Lauf einzigartig machen.
2. **Stabilität** — Solver-Parameter (Steifigkeit, Restitutionsbehandlung) sind auf ein $\Delta t$ abgestimmt.
3. **Reproduzierbare Tests** — S1-g verlangt nach 60 s identische Zustands-Hashes in zwei Läufen.

## 3. Das Akkumulator-Muster

Der Bildschirm läuft mit eigener Rate (60 Hz, 144 Hz oder unregelmäßig). Die Simulation wird davon entkoppelt:

```text
akku += frameZeit
schritte = floor(akku / dt)
falls schritte > maxSchritte:      # Schutz vor der „Todesspirale“
    schritte = maxSchritte; akku = 0
sonst:
    akku -= schritte · dt
schritte-mal: welt.step(dt)
alpha = akku / dt                  # 0 ≤ alpha < 1
gezeichneter Zustand = lerp(vorher, jetzt, alpha)   # slerp für Drehungen
```

Umgesetzt als `FixedStepClock` in `packages/sim-physics/src/fixed-step.ts` (getestet). Die **Interpolation** beseitigt das Ruckeln, das entsteht, wenn die Zahl der Physikschritte je Frame wechselt (z. B. 2, 2, 3, 2 …). Die Darstellung hinkt höchstens einen Schritt hinterher – bei 120 Hz unsichtbar.

Kommt der Rechner nicht mit, verwirft der Schutz den Rückstand: Die Simulationszeit läuft dann langsamer als die Echtzeit, bleibt aber korrekt.

## 4. Warum $\Delta t = 1/120$ s

Faustregel: Der Weg eines Körpers je Schritt muss deutlich kleiner sein als das kleinste geometrische Merkmal, mit dem er interagiert.

| Geschwindigkeit          | Weg je Schritt | Vergleich mit                                             |
| ------------------------ | -------------- | --------------------------------------------------------- |
| 0,10 m/s (Schleichfahrt) | 0,83 mm        | Anschlagtoleranz ±5 mm                                    |
| 0,25 m/s                 | 2,1 mm         | Bodenbrett 22 mm dick, Kette 40 mm breit                  |
| 0,60 m/s (max.)          | 5,0 mm         | Rollenlücke 57 mm, Anschlagfläche — sicherheitshalber CCD |

Bei 0,25 m/s überquert eine Palette eine Rollenteilung (133 mm) in 64 Schritten: Das Kontaktmuster ändert sich gleichmäßig. Ein Brett, das sich einem Anschlag nähert, bewegt sich 0,83–2,1 mm je Schritt, passend zum ±5-mm-Kriterium von S1-c.

## 5. Solver-Iterationen und Unterschritte

In jedem Schritt iteriert der Constraint-Solver mehrmals über alle Kontakte ([[T11 Kontakt, Reibung und Restitution]]):

- **Jolt**: Geschwindigkeitsiterationen (Standard 10) und Positionsiterationen (Standard 2) je Schritt.
- **Rapier**: iterativer Solver mit internen Unterschritten (im Stil von TGS-soft) und einstellbarer Iterationszahl.

Mehr Iterationen → steifere, genauere Kontakte (schwere Paletten auf vielen Rollen) bei linearem Aufwand. Der Spike S1 misst, wie viele für Test a (1000 kg auf 10 Rollen) nötig sind.

## 6. Steuerungszyklus vs. Physikschritt

Die Steuerung läuft alle 10 ms, die Physik alle 8,33 ms. Das sind keine Vielfachen; die Steuerung läuft also im ersten Schritt nach jeder 10-ms-Grenze, mit bis zu 8,3 ms Jitter. Optionen (offene Entscheidung für S1):

1. Den Jitter hinnehmen (vom ±50-ms-Sensorbudget abgedeckt, [[T06 Fotoelektrische Erfassung]]).
2. $\Delta t = 1/100$ s oder $1/200$ s wählen, damit Steuerzyklen auf Schritte fallen.

## 7. Schlafende Körper – eine klassische Falle

Engines legen ruhende Körper **schlafen**, um Rechenzeit zu sparen. Eine Palette, die auf einem stehenden Förderer wartet, schläft ein. Läuft der Förderer wieder an, ändern die kinematischen Rollen ihre Geschwindigkeit – eine schlafende Palette wird dadurch aber nicht unbedingt geweckt. **Der Wrapper muss jede Palette wecken, die einen Förderer berührt, dessen Geschwindigkeit sich ändert.** Wer das vergisst, bekommt Paletten, die sich „weigern“, loszufahren.

## Verwandt

[[T09 Starrkörperdynamik]] · [[T11 Kontakt, Reibung und Restitution]] · [[T03 Zeit, Ereignisreihenfolge und Determinismus]] · [[A07 Physik-Wrapper]]
