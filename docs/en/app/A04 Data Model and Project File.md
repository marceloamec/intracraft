---
title: A04 Data Model and Project File
lang: en
tags: [app, data-model, file-format]
---

🌐 **EN** · [[A04 Modelo de datos y archivo de proyecto|ES]] · [[A04 Datenmodell und Projektdatei|DE]] · [[Home (EN)|Index]]

# A04 Data Model and Project File

## 1. Entities

| Entity     | Content                                                                                                                                        | Status         |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| Project    | `schemaVersion`, name, seed, layout (later: libraries used, display units)                                                                     | Schema exists  |
| Layout     | Instances, connections (later: controllers, scenarios)                                                                                         | Schema exists  |
| Instance   | `id`, `definitionId` + `definitionVersion`, name, position (x, y, z), yaw, params, free `attributes` (key → number/boolean/text, default `{}`) | Schema exists  |
| Port       | Local position, direction, surface height, usable width, tolerances                                                                            | In definitions |
| Connection | From output port of A to input port of B, with `gap` (default 0.02 m)                                                                          | Schema exists  |
| Signal     | Boolean with name, source and target                                                                                                           | Planned        |
| Pallet     | Type, mass, load, state and location (object + coordinate)                                                                                     | Types exist    |
| Scenario   | Layout + sources + seed + duration + expected metrics                                                                                          | Analytic only  |

Connections are not drawn by hand: after every edit command they are derived from the port geometry ([[ADR-0006 Command Package and Geometric Port Connections]]).

Code: `packages/core-model/src/schema.ts` (Zod). Types are inferred from the schemas (`Project`, `Layout`, `Instance`, `Connection`).

## 2. Ports and connections

- Each port declares **surface height**, **usable width** and alignment **tolerances** (defaults ±10 mm height, ±20 mm lateral).
- The editor snaps nearby ports and warns when height, width or orientation do not match.
- Turntable ports live in the platform frame and only count as connected when the angle matches a station ([[T13 Turntables and Transfer Lifts]]).
- The **gap** between conveyors is a property of the connection (default 20 mm). In 3D it is a real gap the pallet must bridge.

## 3. The project file (`.intracraft.json`)

Readable JSON, versionable with git. **All stored values are SI** (m, kg, s, rad); the editor converts to mm and degrees for display only.

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
        "name": "Roller conveyor 1",
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

Only parameters that differ from the definition defaults need to be stored; missing ones take the default of the pinned definition version.

## 4. Versioning and migrations

- `schemaVersion` is a literal in the schema: a file with another version is **rejected**, never silently read.
- Each schema change adds a migration function `v(n) → v(n+1)`; loading applies them in sequence, then validates.
- Instances pin `definitionVersion` (semantic versioning). Upgrading a component definition is an explicit migration, so an old layout keeps behaving the same.

## 5. What is not stored in the project

Simulation results (statistics, event logs, golden files) are **never** saved inside the project file. They are exported separately (CSV/JSON) or kept in `test_data/` for regression tests.

## Related

[[A05 Component Library]] · [[A13 Conventions]] · [[A14 API Reference]]
