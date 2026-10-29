---
title: S02 Kettenförderer
lang: de
tags: [spec, conveyor, chain]
library-id: conveyor.chain
status: spec
updated: 2026-10-29
---

🌐 [[S02 Chain Conveyor|EN]] · [[S02 Mesa de cadenas|ES]] · **DE** · [[Startseite (DE)|Index]]

# S02 Kettenförderer (`conveyor.chain`)

Gerader Abschnitt, auf dem die Palette auf mehreren synchronen Kettensträngen liegt. **EUR-Paletten fahren auf Kettenförderern quer (Schmalseite, 800 mm, voran)** – bestätigt am 29.10.2026. Die drei Bodenbretter (1200 mm lang) liegen dann **quer** zu den Ketten, jeder Strang berührt also jedes Brett. Ergänzt die gemeinsamen Parameter aus [[S01 Rollenförderer]] §1; alle Verhaltensparameter gelten je Instanz.

## 1. Parameter

| Schlüssel               | Plan-Schlüssel        | Einheit | Standard                 | Bereich                             | Beschreibung                                                                                                                                               |
| ----------------------- | --------------------- | ------- | ------------------------ | ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `palletSideAlongTravel` | `ladoParaleloAvance`  | Enum    | **kurz**                 | lang, kurz                          | EUR: 800 mm in Förderrichtung, 1200 mm quer                                                                                                                |
| `usableWidth`           | `anchoUtil`           | mm      | **1300** (vorgeschlagen) | 500 … 1800                          | Muss für EUR quer ≥ 1200 + 2 × Abstand sein (der gemeinsame Standard 1000 mm reicht nicht) – zu bestätigen                                                 |
| `chainCount`            | `numCadenas`          | —       | 3                        | 2 … 6                               | Anzahl Kettenstränge                                                                                                                                       |
| `chainPositions`        | `posicionesCadenas`   | mm      | auto                     | —                                   | Querposition jedes Strangs zur Achse                                                                                                                       |
| `chainSupportWidth`     | `anchoApoyoCadena`    | mm      | 40                       | 20 … 100                            | Auflagebreite je Strang                                                                                                                                    |
| `chainType`             | `tipoCadena`          | Enum    | Rollenkette              | einfach, Rollenkette, Doppelteilung | Reibung und Auflagegeometrie                                                                                                                               |
| `chainPitch`            | `pasoCadena`          | mm      | 38,1                     | 19,05 … 76,2                        | Teilung; Geometrie und Polygoneffekt in 3D                                                                                                                 |
| `chainMu`               | `muCadena`            | —       | 0,30                     | 0,1 … 0,6                           | Reibung Palette–Kette                                                                                                                                      |
| `accumulationMode`      | `modoAcumulacion`     | Enum    | **none**                 | none, slip, zoned                   | none: alle Paletten folgen der Kette, der Förderer läuft und hält als Einheit; slip: Ketten rutschen unter gehaltenen Paletten; zoned: unabhängige Stränge |
| `zoneLength`            | `longitudZona`        | mm      | 1400                     | 1000 … 2000                         | Nur bei Zonen                                                                                                                                              |
| `stopContactTime`       | `tiempoDetencionTope` | s       | 0,2                      | 0,05 … 1                            | 2D: Zeit bis zum Halt am Anschlag (Modus slip)                                                                                                             |
| `maxDragForce`          | `fuerzaMaxArrastre`   | N       | abgeleitet               | —                                   | μ × Palettengewicht; informativ                                                                                                                            |

Kapazität bei EUR quer: $\lfloor \text{Länge}/(0{,}8 + \text{Mindestabstand}) \rfloor$ → auf 3 m passen 3 Paletten bei 150 mm Lücke. Abstandsbegrenzter Durchsatz bei 0,25 m/s: $0{,}25/(0{,}8+0{,}15)$ = 0,263 Paletten/s ≈ **947 Paletten/h**.

Ports A (Eingang) und B (Ausgang) wie beim Rollenförderer.

## 2. Verhalten

- **`none` (Standard):** wie beim Rollenförderer: Jede Palette folgt der Kette; beim Aufnehmen bewegen sich die vorhandenen Paletten mit; ein blockierter Auslauf hält den ganzen Förderer ([[S01 Rollenförderer]] §3).
- **`slip`:** Eine gehaltene Palette steht, die Kette läuft darunter weiter (Staudruck $\sum \mu m g$, [[T05 Stau, Kapazität und Durchsatz]] §6.3).
- **3D:** Jeder Strang ist ein Streifen kleiner kinematischer Rollen (Plan A) oder ein statischer Quader mit **Oberflächengeschwindigkeit** (Plan B, im JS-Binding von Jolt verfügbar) – [[T12 Rollen und Ketten in der Physik]], [[ADR-0002 Physik-Engine-Kandidat]].

## 3. Validierungen

- Mindestens 2 Ketten unter der Palette; bei EUR quer kreuzt jeder Strang alle drei Bodenbretter.
- Nutzbreite ≥ 1200 + 2 × Abstand für EUR quer.

## 4. Tests

| Test | Kriterium                                                                     | Stand      |
| ---- | ----------------------------------------------------------------------------- | ---------- |
| S1-b | Palettengeschwindigkeit = Kettengeschwindigkeit ±3 %, kein sichtbares Zittern | ⏳ S1      |
| —    | `none`: einfahrende Palette bewegt die vorhandene                             | ⏳ Phase 5 |
| —    | `slip`: Linienkraft ≈ Σ μ m g                                                 | ⏳ Phase 5 |

## Verwandt

[[T12 Rollen und Ketten in der Physik]] · [[S05 Rollen-Ketten-Umsetzer]] · [[S03 Kettendrehtisch]] · [[Offene Fragen]]
