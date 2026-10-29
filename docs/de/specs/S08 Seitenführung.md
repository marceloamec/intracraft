---
title: S08 Seitenführung
lang: de
tags: [spec, mechanics, guide]
library-id: mechanics.guide
status: spec
---

🌐 [[S08 Side Guide|EN]] · [[S08 Guía lateral|ES]] · **DE** · [[Startseite (DE)|Index]]

# S08 Seitenführung (`mechanics.guide`)

In jeden Förderer integriert (Gruppe `guides`) und zusätzlich als eigenständige Komponente für weitere Abschnitte verfügbar. Theorie: [[T14 Anschläge, Aufprall und Ausrichtung]] §6.

## 1. Parameter

| Schlüssel            | Plan-Schlüssel    | Einheit | Standard | Bereich                     | Beschreibung                                  |
| -------------------- | ----------------- | ------- | -------- | --------------------------- | --------------------------------------------- |
| `side`               | `lado`            | Enum    | beide    | links, rechts, beide, keine | Seiten mit Führung                            |
| `guideHeight`        | `alturaGuia`      | mm      | 150      | 50 … 400                    | Höhe über der Förderebene                     |
| `clearance`          | `holgura`         | mm      | 25       | 0 … 100                     | Abstand je Seite zwischen Palette und Führung |
| `startPct`, `endPct` | `sInicio`, `sFin` | %       | 0, 100   | 0 … 100                     | Abgedeckter Teil der Länge                    |
| `funnelEntry`        | `entradaEmbudo`   | bool    | nein     | —                           | Fügt einen Einlauftrichter hinzu              |
| `funnelAngle`        | `anguloEmbudo`    | °       | 10       | 3 … 30                      | Trichterwinkel                                |
| `funnelLength`       | `longitudEmbudo`  | mm      | 300      | 100 … 800                   | Trichterlänge                                 |
| `guideMu`            | `muGuia`          | —       | 0,30     | 0,1 … 0,6                   | Reibung mit der Palette                       |
| `guideRestitution`   | `restitucionGuia` | —       | 0,1      | 0 … 0,5                     | Seitlicher Rückprall                          |

## 2. So werden Paletten ausgerichtet

- Der **Anschlag** legt die Längslage der Vorderkante fest; mit einem Kontaktpunkt kann sich die Palette drehen, ein breiter oder zwei Anschläge richten sie rechtwinklig aus.
- **Führungen** legen die Seitenlage fest; der Trichter korrigiert schräg einlaufende Paletten (Fangbereich ≈ Freiraum + Trichterlänge · tan(Trichterwinkel) = 25 + 53 mm bei Standardwerten).
- In 2D wird nur die ideale Endlage angesetzt (Führungen sind eine Freiraumprüfung); in 3D ergibt sich die reale Lage – das Maß der Ausrichtungsqualität.

## 3. Verhalten

**2D.** Nur Freiraumprüfung. **3D.** Statische Kollisionskörper mit Reibung und Restitution.

## 4. Tests

| Test        | Kriterium                                             | Stand                      |
| ----------- | ----------------------------------------------------- | -------------------------- |
| S1-d / P3   | 50 mm und 5° Schräglage auf ±10 mm und ±1° korrigiert | ⏳ S1 / Phase 4            |
| Validierung | Nutzbreite ≥ Palette + 2 × Freiraum                   | ✅ `usableWidthFitsPallet` |

## Verwandt

[[T14 Anschläge, Aufprall und Ausrichtung]] · [[S07 Mechanischer Anschlag]]
