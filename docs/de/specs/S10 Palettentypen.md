---
title: S10 Palettentypen
lang: de
tags: [spec, pallet, reference-data]
library-id: pallet.*
status: reference-json
updated: 2026-10-29
---

🌐 [[S10 Pallet Types|EN]] · [[S10 Tipos de pallet|ES]] · **DE** · [[Startseite (DE)|Index]]

# S10 Palettentypen und Reibpaarungen

Daten: `packages/library/src/definitions/pallet-types.json` und `friction-pairs.json` (beim Laden validiert, getestet). **Das Projekt arbeitet mit EUR-Paletten** (Verantwortlicher, 29.10.2026): längs auf Rollen, quer auf Ketten. Die Werte sind **Richtwerte**; vor Vertrauen in Sensorergebnisse mit der Norm oder realen Paletten abgleichen.

## 1. Palettentypen

| Typ               | L × B (mm)  | Höhe (mm) | Eigengewicht (kg) | Profilstatus | Hinweise                          |
| ----------------- | ----------- | --------- | ----------------- | ------------ | --------------------------------- |
| EUR / EPAL        | 1200 × 800  | 144       | 25                | Richtwert    | Haupttyp des Projekts             |
| 1200 × 1000       | 1200 × 1000 | ≈ 150     | 28                | angenähert   | Querprofil vereinfacht            |
| GMA               | 1219 × 1016 | ≈ 140     | 25                | angenähert   | Kufenpalette; Platzhalterprofil   |
| Benutzerdefiniert | frei        | frei      | frei              | Benutzer     | Eigenes Profil im Editor (später) |

Eine Paletteninstanz hat außerdem Lastmasse und Ladehöhe, `centerOfMassOffset` und eigene Schlüssel–Wert-Attribute.

## 2. EUR-Aufbau für die Profile

| Lage (Höhe, mm)       | Teile                                                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Bodenbretter (0–22)   | 3 Bretter, 1200 mm lang, über die 800-mm-Seite bei 0–100, 327,5–472,5, 700–800 mm (100 / 145 / 100 breit)                             |
| Klötze (22–100)       | 9 Klötze; entlang der 1200-mm-Seite bei 0–145, 527,5–672,5, 1055–1200 mm; entlang der 800-mm-Seite bei 0–100, 327,5–472,5, 700–800 mm |
| Querbretter (100–122) | 3 Bretter, 800 mm lang, 145 mm breit, auf den Klotzreihen                                                                             |
| Deckbretter (122–144) | 5 Bretter, 1200 mm lang (145 / 100 / 145 / 100 / 145 breit) mit Lücken von ≈ 41 mm                                                    |

## 3. Belegungsprofile (was ein waagerechter Strahl quer zur Förderrichtung sieht)

Massive Intervalle ab Palettenvorderkante, in m.

**Längsseite voran — `alongLength` (Rollenförderer):**

| Band (mm) | Massive Intervalle                         | Strahlverhalten               |
| --------- | ------------------------------------------ | ----------------------------- |
| 0–22      | [0, 1,2]                                   | Durchgehend                   |
| 22–100    | [0, 0,145], [0,5275, 0,6725], [1,055, 1,2] | Lücken von 382,5 mm           |
| 100–122   | [0, 0,145], [0,5275, 0,6725], [1,055, 1,2] | Lücken von 382,5 mm           |
| 122–144   | [0, 1,2]                                   | **Durchgehend** (Deckbretter) |

**Schmalseite voran — `alongWidth` (Kettenförderer):**

| Band (mm) | Massive Intervalle                                                                 | Strahlverhalten               |
| --------- | ---------------------------------------------------------------------------------- | ----------------------------- |
| 0–22      | [0, 0,1], [0,3275, 0,4725], [0,7, 0,8]                                             | Lücken von 227,5 mm           |
| 22–100    | [0, 0,1], [0,3275, 0,4725], [0,7, 0,8]                                             | Lücken von 227,5 mm           |
| 100–122   | [0, 0,8]                                                                           | **Durchgehend** (Querbretter) |
| 122–144   | [0, 0,145], [0,18625, 0,28625], [0,3275, 0,4725], [0,51375, 0,61375], [0,655, 0,8] | Lücken von ≈ 41 mm            |

Folge: Die beste Strahlhöhe hängt von der Ausrichtung ab – ≈ 130 mm auf Rollen, ≈ 110 mm auf Ketten ([[S06 Lichtschranke]]). Die 3D-Kollisionskörper werden aus demselben Aufbau erzeugt.

## 4. Reibpaarungen

| Paarung              | μ Start | Üblicher Bereich | JSON-Id             |
| -------------------- | ------- | ---------------- | ------------------- |
| Holz – Stahl (Rolle) | 0,35    | 0,25 – 0,50      | `wood-steel-roller` |
| Holz – Stahlkette    | 0,30    | 0,20 – 0,45      | `wood-steel-chain`  |
| Kunststoff – Stahl   | 0,25    | 0,15 – 0,40      | `plastic-steel`     |
| Holz – Stahlführung  | 0,30    | 0,20 – 0,45      | `wood-steel-guide`  |

## Verwandt

[[T06 Fotoelektrische Erfassung]] · [[S06 Lichtschranke]] · [[T11 Kontakt, Reibung und Restitution]] · [[S09 Quelle und Senke]] · [[Offene Fragen]]
