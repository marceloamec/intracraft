---
title: T06 Fotoelektrische Erfassung
lang: de
tags: [theory, sensors, des, physics]
updated: 2026-10-29
---

🌐 [[T06 Photoelectric Sensing|EN]] · [[T06 Detección fotoeléctrica|ES]] · **DE** · [[Startseite (DE)|Index]]

# T06 Fotoelektrische Erfassung

## 1. Sensortypen und ihr Simulationsmodell

| Typ                     | Reales Gerät                                                    | Simulationsmodell                               |
| ----------------------- | --------------------------------------------------------------- | ----------------------------------------------- |
| Lichttaster             | Sender und Empfänger in einem Gehäuse; Objekt reflektiert Licht | Strahl vom Sensor in eine Richtung, bis `range` |
| Reflexionslichtschranke | Reflektor gegenüber; das Objekt **unterbricht** den Strahl      | Strecke Sensor → Reflektor                      |
| Einweglichtschranke     | Getrennter Sender und Empfänger                                 | Strecke Sender → Empfänger                      |

Der MVP ignoriert Farbe, Reflexionsgrad und Fremdlicht. Der Ausgang ist ein **boolesches Signal** mit wählbarer Polarität.

## 2. Geometrie: ein Strahl zwischen zwei vom Benutzer gewählten Punkten

Der Benutzer setzt den **Sensor** bei $E = (s_E, y_E, h_E)$ und den **Reflektor/Empfänger** bei $R = (s_R, y_R, h_R)$, im System des Trägerförderers ($s$ in Förderrichtung, $y$ seitlich, $h$ Höhe über der Förderebene). Der Strahl ist die Strecke

$$P(u) = E + u\,(R - E), \qquad u \in [0, 1]$$

Üblich ist ein waagerechter Querstrahl ($s_E = s_R$, $h_E = h_R$), schräge oder geneigte Strahlen sind aber erlaubt ([[S06 Lichtschranke]]). Die Höhe ist der wichtigste Parameter, denn eine Palette ist kein massiver Block.

## 3. Das Belegungsprofil (2D-Modus)

Jeder Palettentyp beschreibt je **Höhenband** die massiven Intervalle in seiner Förderrichtung (ab Vorderkante), getrennt für beide Ausrichtungen: `alongLength` (Längsseite voran – Rollenförderer) und `alongWidth` (Schmalseite voran – Kettenförderer). Vollständige EUR-Tabellen: [[S10 Palettentypen]]. Übersicht:

| EUR-Band (mm)       | Längsseite voran (Rollen) | Schmalseite voran (Ketten) |
| ------------------- | ------------------------- | -------------------------- |
| 0–22 Bodenbretter   | durchgehend               | Lücken 227,5 mm            |
| 22–100 Klötze       | Lücken 382,5 mm           | Lücken 227,5 mm            |
| 100–122 Querbretter | Lücken 382,5 mm           | **durchgehend**            |
| 122–144 Deckbretter | **durchgehend**           | Lücken ≈ 41 mm             |

Jedes Band × Intervall ist ein Quader, der sich mit der Palette bewegt: seitlich über die ganze Palettenbreite, Höhe = Band, in Förderrichtung $[s_{vorn} - b_k,\ s_{vorn} - a_k]$.

## 4. Flanken für jeden beliebigen Strahl vorhersagen

Für einen massiven Quader $k$ mit seitlichem Bereich $[y_0, y_1]$ und Höhenbereich $[h_0, h_1]$ (fest, während die Palette fährt) und Intervall in Förderrichtung $[a_k, b_k]$ ab Vorderkante:

1. **Strecke beschneiden** an den Seiten- und Höhenschichten: Die Werte von $u$ mit $y(u) \in [y_0, y_1]$ und $h(u) \in [h_0, h_1]$ bilden ein Intervall $[u_0, u_1]$ (leer → dieser Quader verdeckt den Strahl nie).
2. Der beschnittene Strahlteil überdeckt in Förderrichtung $[x_{min}, x_{max}]$ = Wertebereich von $s(u)$ für $u \in [u_0, u_1]$.
3. Der Quader verdeckt den Strahl genau dann, wenn sich beide Intervalle in Förderrichtung überlappen:

$$s_{vorn} \in [\,x_{min} + a_k,\; x_{max} + b_k\,]$$

Jeder Quader liefert also ein Intervall von Vorderkantenpositionen – eine steigende und eine fallende Flanke –, und der Strahl ist auf der **Vereinigung** dieser Intervalle verdeckt (überlappende verschmelzen). Die DES-Engine plant die Flanken mit `timeToReach` ([[T04 Kinematik von Förderern]]). Für einen waagerechten Querstrahl ist $x_{min} = x_{max} = s_{sens}$, und die Regel wird zu $s_{vorn} \in [s_{sens} + a_k,\ s_{sens} + b_k]$.

Warum es immer ein einziges Intervall ist: Die Menge der Verschiebungen, bei denen sich zwei konvexe Formen schneiden, ist konvex; auf einer Achse ist das ein Intervall.

### Test T6

Strahl in einem durchgehenden Band bei $s = 2{,}0$ m, $v = 0{,}25$ m/s, EUR längs: steigende Flanke, wenn die Vorderkante 2,0 m passiert, fallende, wenn die Hinterkante sie passiert; Dauer $1{,}2/0{,}25$ = **4,8 s**.

## 5. Das Problem der Palettenlücken

| Fall (EUR, 0,25 m/s)                   | Lücke    | Signal aus für              |
| -------------------------------------- | -------- | --------------------------- |
| Rollen, Strahl in 60 mm (Klotzlage)    | 382,5 mm | 1,53 s (zweimal je Palette) |
| Ketten, Strahl in 60 mm (Klotzlage)    | 227,5 mm | 0,91 s (zweimal je Palette) |
| Ketten, Strahl in 130 mm (Deckbretter) | ≈ 41 mm  | ≈ 0,16 s (viermal)          |

Eine Steuerung, die Paletten zählt, käme auf drei (oder fünf) je Palette. Abhilfen:

1. **Strahlhöhe wählen** in einem für die Ausrichtung auf diesem Förderer durchgehenden Band: ≈ 130 mm auf Rollen, ≈ 110 mm auf Ketten (oder oberhalb der Palette, auf der Ladung).
2. **Filter:** Der neue Zustand muss `filter` ms anliegen; zum Überbrücken $t_{Filter} > \text{Lücke}/v$. Ein Filter verzögert auch die **echte** fallende Flanke um dieselbe Zeit, was Zonenfreigaben verschiebt und Durchsatz kosten kann.

Der Editor warnt, wenn die gewählte Höhe in einem Band mit Lücken liegt ([[S06 Lichtschranke]] §2).

## 6. Lichtfleckgröße

Ein Strahl mit Durchmesser $d$ wird teilweise verdeckt, bevor eine Kante seine Achse erreicht. 2D behandelt den Strahl als Linie; 3D kann bei großen Lichtflecken einen **Shape-Cast** nutzen, der jedes massive Intervall um etwa $d/2$ je Seite verbreitert.

## 7. Zeitverhalten: Ansprechzeit und SPS-Zyklus

$$t_{gesehen} = t_{geometrisch} + t_{Ansprech} + t_{Filter} + t_{Abtastung}$$

`responseTime` (Standard 2 ms) gilt in beiden Modi; der Steuerungszyklus (Standard 10 ms) ergibt in 3D eine gleichverteilte Verzögerung in $[0, 10)$ ms, die 2D nachbilden oder durch ihren Mittelwert annähern kann.

## 8. Physikmodus: Raycasts

In jedem Physikschritt wird die Strecke $E \to R$ gegen die Kollisionskörper geprüft (ohne die eigenen Rollen, Rahmen und Führungen des Förderers). Die Flanke in der Schrittmitte zu melden begrenzt den Fehler auf $\Delta t/2 = 4{,}2$ ms bei 120 Hz.

## 9. Fehlerbudget 2D ↔ 3D für Sensorflanken (Ziel ±50 ms)

| Quelle                                               | Typische Größe |
| ---------------------------------------------------- | -------------- |
| Quantisierung durch den Physikschritt (Mitte)        | ≤ 4,2 ms       |
| Steuerungsabtastung (falls nicht in 2D nachgebildet) | ≤ 10 ms        |
| Schlupf / Mikrogleiten (3D)                          | 0 – 20 ms      |
| Abweichende Geschwindigkeit bei der Übergabe         | 0 – 20 ms      |
| **Summe (ungünstigster Fall, unkorreliert)**         | < 50 ms        |

## Verwandt

[[S06 Lichtschranke]] · [[S10 Palettentypen]] · [[T07 Steuerungslogik und Signale]] · [[T11 Kontakt, Reibung und Restitution]]
