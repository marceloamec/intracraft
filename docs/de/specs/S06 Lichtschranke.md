---
title: S06 Lichtschranke
lang: de
tags: [spec, sensor]
library-id: sensor.photoeye
status: spec
updated: 2026-10-29
---

🌐 [[S06 Photoelectric Sensor|EN]] · [[S06 Fotocélula|ES]] · **DE** · [[Startseite (DE)|Index]]

# S06 Lichtschranke (`sensor.photoeye`)

Theorie: [[T06 Fotoelektrische Erfassung]].

**Anforderung des Verantwortlichen (29.10.2026):** Position **und Höhe** der Lichtschranke und ihres Reflektors sind Parameter jeder Instanz – der Benutzer wählt sie. Der Strahl ist daher die Strecke zwischen zwei vom Benutzer gewählten Punkten, keine feste waagerechte Linie.

## 1. Parameter

| Schlüssel          | Plan-Schlüssel    | Einheit | Standard                  | Bereich                        | Beschreibung                                                                  |
| ------------------ | ----------------- | ------- | ------------------------- | ------------------------------ | ----------------------------------------------------------------------------- |
| `type`             | `tipo`            | Enum    | Reflexion (zu bestätigen) | Taster, Reflexion, Einweg      | Reflexion: Sensor + Reflektor; Einweg: Sender + Empfänger; Taster: nur Sensor |
| `host`             | `anfitrion`       | Ref     | —                         | —                              | Trägerförderer (Positionen in dessen Koordinaten)                             |
| `sensorS`          | `s`               | mm      | 0                         | 0 … Länge                      | Sensorposition entlang des Trägers                                            |
| `sensorLateral`    | `lateral`         | mm      | linker Rahmen (auto)      | frei                           | Seitliche Sensorposition zur Trägerachse                                      |
| `sensorHeight`     | `altura`          | mm      | 60                        | 0 … 400                        | Strahlhöhe am Sensor über der Förderebene                                     |
| `reflectorS`       | — (neu)           | mm      | = `sensorS`               | 0 … Länge                      | Position von Reflektor / Empfänger entlang des Trägers                        |
| `reflectorLateral` | — (neu)           | mm      | rechter Rahmen (auto)     | frei                           | Seitliche Position von Reflektor / Empfänger                                  |
| `reflectorHeight`  | — (neu)           | mm      | = `sensorHeight`          | 0 … 400                        | Höhe von Reflektor / Empfänger                                                |
| `orientation`      | `orientacion`     | °       | 90                        | 0 … 180                        | Nur Taster: Strahlrichtung zur Förderrichtung                                 |
| `range`            | `alcance`         | mm      | 1000 (Taster) / 5000      | 50 … 5000                      | Maximale Erfassungsweite                                                      |
| `spotDiameter`     | `diametroSpot`    | mm      | 10                        | 1 … 50                         | Strahldurchmesser                                                             |
| `responseTime`     | `tiempoRespuesta` | ms      | 2                         | 0 … 50                         | Eigene Verzögerung des Sensors                                                |
| `polarity`         | `polaridad`       | Enum    | hellschaltend             | hellschaltend, dunkelschaltend | Ausgangslogik                                                                 |
| `filter`           | `filtro`          | ms      | 0                         | 0 … 2000                       | Mindestzeit für einen Wechsel; kann Lücken überbrücken                        |
| `cycleDelay`       | `retardoCiclo`    | ms      | 10                        | 0 … 100                        | Abtastung durch die Steuerung (SPS-Zyklus)                                    |
| `outputSignal`     | `señalSalida`     | Text    | auto                      | —                              | Name des booleschen Signals                                                   |
| `showBeam`         | `mostrarHaz`      | bool    | ja                        | —                              | Strahl in 3D zeichnen (grün frei, rot erkannt)                                |

Die `reflector*`-Parameter sind beim Taster ausgeblendet (`dependsOn: type≠diffuse`). Der Editor setzt beide Enden mit Anfassern in der 3D-Ansicht und zeigt die Werte in mm.

**Abgeleitet (schreibgeschützt):** Strahllänge, Winkel zur Förderrichtung (90°, wenn `sensorS = reflectorS`), Neigung (0°, wenn beide Höhen gleich sind).

## 2. Validierungen und Warnungen

- Strahllänge ≤ `range`.
- Der Strahl muss den Raum kreuzen, den eine Palette auf dem Träger einnimmt; sonst erkennt er nie etwas.
- **Lückenwarnung:** Liegt die Strahlhöhe – für Palettentyp und Ausrichtung auf dem Träger – in einem Band mit Lücken, wird mit der erwarteten Lückendauer bei Nenngeschwindigkeit gewarnt und eine Höhe in einem durchgehenden Band oder ein `filter`-Wert vorgeschlagen. Beispiele mit EUR ([[S10 Palettentypen]]):

| Träger (EUR-Ausrichtung) | Durchgehende Bänder (kein Flackern)     | Bei 60 mm (Planwert)                      |
| ------------------------ | --------------------------------------- | ----------------------------------------- |
| Rollen (längs)           | 122–144 mm (Deckbretter) → z. B. 130 mm | Lücken von 382,5 mm → 1,53 s bei 0,25 m/s |
| Ketten (quer)            | 100–122 mm (Querbretter) → z. B. 110 mm | Lücken von 227,5 mm → 0,91 s bei 0,25 m/s |

Oberhalb der Palette (> 144 mm) sieht der Strahl stattdessen die Ladung, falls vorhanden.

## 3. Verhalten

**2D.** Der Strahl ist die Strecke Sensor → Reflektor. Jedes massive Teil der Palette (aus ihrem Belegungsprofil) erzeugt ein Intervall von Vorderkantenpositionen, in dem die Strecke verdeckt ist; die Flanken sind die Intervallgrenzen und werden analytisch vorhergesagt ([[T06 Fotoelektrische Erfassung]] §4).
**3D.** Raycast (oder Shape-Cast bei großen Lichtflecken) entlang derselben Strecke in jedem Physikschritt, Zeitstempel der Flanke in der Schrittmitte.

## 4. Tests

| Test | Kriterium                                                                                                               | Stand      |
| ---- | ----------------------------------------------------------------------------------------------------------------------- | ---------- |
| T6   | Sensor und Reflektor bei 2,0 m, Strahl in durchgehendem Band → steigend mit Vorderkante, fallend mit Hinterkante, 4,8 s | ⏳ Phase 3 |
| —    | Strahl in 60 mm auf Rollen mit EUR → drei Impulse je Palette; mit `filter` 2000 ms → einer                              | ⏳ Phase 3 |
| —    | Schräger Strahl (verschiedene `sensorS` und `reflectorS`) → Flanken wie die analytischen Intervalle                     | ⏳ Phase 3 |
| P6   | 3D- und 2D-Flanken innerhalb ±50 ms                                                                                     | ⏳ Phase 4 |

## Verwandt

[[T06 Fotoelektrische Erfassung]] · [[S10 Palettentypen]] · [[T07 Steuerungslogik und Signale]] · [[Offene Fragen]]
