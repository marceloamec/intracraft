---
title: A04 Datenmodell und Projektdatei
lang: de
tags: [app, data-model, file-format]
---

🌐 [[A04 Data Model and Project File|EN]] · [[A04 Modelo de datos y archivo de proyecto|ES]] · **DE** · [[Startseite (DE)|Index]]

# A04 Datenmodell und Projektdatei

## 1. Entitäten

| Entität    | Inhalt                                                                                   | Stand               |
| ---------- | ---------------------------------------------------------------------------------------- | ------------------- |
| Projekt    | `schemaVersion`, Name, Seed, Layout (später: verwendete Bibliotheken, Anzeigeeinheiten)  | Schema vorhanden    |
| Layout     | Instanzen, Verbindungen (später: Steuerungen, Szenarien)                                 | Schema vorhanden    |
| Instanz    | `id`, `definitionId` + `definitionVersion`, Name, Position (x, y, z), Drehung, Parameter | Schema vorhanden    |
| Port       | Lokale Position, Richtung, Förderhöhe, Nutzbreite, Toleranzen                            | In den Definitionen |
| Verbindung | Vom Ausgangsport von A zum Eingangsport von B, mit `gap` (Standard 0,02 m)               | Schema vorhanden    |
| Signal     | Boolesch, mit Name, Quelle und Ziel                                                      | Geplant             |
| Palette    | Typ, Masse, Ladung, Zustand und Ort (Objekt + Koordinate)                                | Typen vorhanden     |
| Szenario   | Layout + Quellen + Seed + Dauer + erwartete Kennzahlen                                   | Nur analytisch      |

Code: `packages/core-model/src/schema.ts` (Zod). Die Typen werden aus den Schemas abgeleitet (`Project`, `Layout`, `Instance`, `Connection`).

## 2. Ports und Verbindungen

- Jeder Port deklariert **Förderhöhe**, **Nutzbreite** und Ausrichtungs**toleranzen** (Standard ±10 mm Höhe, ±20 mm seitlich).
- Der Editor rastet nahe Ports ein und warnt, wenn Höhe, Breite oder Ausrichtung nicht passen.
- Drehtisch-Ports liegen im Plattformsystem und gelten nur als verbunden, wenn der Winkel einer Station entspricht ([[T13 Drehtische und Kettenumsetzer]]).
- Der **Spalt** zwischen Förderern ist eine Eigenschaft der Verbindung (Standard 20 mm). In 3D ist es ein echter Spalt, den die Palette überbrücken muss.

## 3. Die Projektdatei (`.intracraft.json`)

Lesbares JSON, mit git versionierbar. **Alle gespeicherten Werte sind SI** (m, kg, s, rad); der Editor rechnet nur zur Anzeige in mm und Grad um.

```json
{
  "schemaVersion": 1,
  "name": "Source to Sink",
  "seed": 1,
  "layout": {
    "instances": [
      {
        "id": "conv1",
        "definitionId": "conveyor.roller",
        "definitionVersion": "0.1.0",
        "name": "Rollenförderer 1",
        "position": { "x": 0, "y": 0, "z": 0 },
        "yaw": 0,
        "params": { "length": 3.0, "nominalSpeed": 0.25, "drive": "zoned", "accumulationMode": "zpa" }
      }
    ],
    "connections": [
      { "from": { "instanceId": "src1", "portId": "out" }, "to": { "instanceId": "conv1", "portId": "A" }, "gap": 0.02 }
    ]
  }
}
```

Gespeichert werden müssen nur Parameter, die von den Standardwerten der Definition abweichen; fehlende übernehmen den Standard der festgelegten Definitionsversion.

## 4. Versionierung und Migrationen

- `schemaVersion` ist im Schema ein Literal: Eine Datei mit anderer Version wird **abgelehnt**, nie stillschweigend gelesen.
- Jede Schemaänderung fügt eine Migrationsfunktion `v(n) → v(n+1)` hinzu; beim Laden werden sie nacheinander angewendet, danach wird validiert.
- Instanzen legen `definitionVersion` fest (semantische Versionierung). Eine Komponentendefinition zu aktualisieren ist eine explizite Migration – so verhält sich ein altes Layout weiterhin gleich.

## 5. Was nicht im Projekt gespeichert wird

Simulationsergebnisse (Statistik, Ereignisprotokolle, Referenzergebnisse) werden **nie** in der Projektdatei gespeichert. Sie werden getrennt exportiert (CSV/JSON) oder für Regressionstests in `test_data/` abgelegt.

## Verwandt

[[A05 Komponentenbibliothek]] · [[A13 Konventionen]] · [[A14 API-Referenz]]
