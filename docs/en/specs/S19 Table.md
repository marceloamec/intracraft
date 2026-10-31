---
title: S19 Table
lang: en
tags: [spec, information, scripting]
library-id: info.table
status: proposed — waiting for the owner's confirmation
updated: 2026-10-31
---

# S19 Table (`info.table`)

A **grid of typed columns** on the 2D plan: order lists, destination shares, processing times by pallet type, hourly reports. Code reads and writes it ([[A17 IntraScript Language]]); the editor edits it like a spreadsheet and imports or exports CSV. Decision: [[ADR-0018 IntraScript Scripting Language]].

## 1. Parameters

| Key            | Unit | Default | Range        | Description                                                                     |
| -------------- | ---- | ------- | ------------ | ------------------------------------------------------------------------------- |
| `keyColumn`    | —    | `""`    | column names | Text column whose values name the rows (`Orders["A17", "qty"]`); empty = no key |
| `resetAtStart` | —    | true    | true, false  | Start every run with the initial content                                        |
| `showOnPlan`   | —    | true    | true, false  | Show the name and the size (`Orders 12 × 3`) next to the icon in 2D             |

## 2. Columns and content

Each column has a **name** (unique in the table, not case-sensitive), a **type** and, for quantities, a **display unit**:

| Column type                                                                        | Cell value in the project file                                          |
| ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `integer`, `real`                                                                  | Number                                                                  |
| `boolean`                                                                          | `true` / `false`                                                        |
| `text`                                                                             | String                                                                  |
| `time`, `length`, `speed`, `acceleration`, `mass`, `angle`, `angularSpeed`, `rate` | Number in SI units                                                      |
| `object`                                                                           | Path of an object (`"Hall1.LineA.Roller3"`), resolved when a run starts |
| `pallet`                                                                           | Only during a run; the initial content cannot hold pallets              |

```json
"content": {
  "kind": "table",
  "columns": [
    { "name": "destination", "type": "text" },
    { "name": "weight", "type": "real" },
    { "name": "gap", "type": "length", "unit": "mm" }
  ],
  "rows": [["A", 0.7, 0.15], ["B", 0.3, 0.2]]
}
```

Empty cells hold the column type's default (A17 §4.1). At most 256 columns and 100 000 rows.

## 3. On the plan

A small grid icon with the name and, when `showOnPlan`, its size in rows × columns. Not drawn in 3D. It can be placed inside an area ([[S22 Area]]).

## 4. During a run (2D and 3D alike)

- The run starts with the initial content (or, with `resetAtStart` = false, the content left by the previous run of the same session).
- Code changes cells, adds, deletes and sorts rows; the final content can be exported with the results (CSV) and is never written back into the project.
- `object` cells whose path no longer exists stop the start with `unknownObjectInCell`.

## 5. In code

| Member                                          | Result                                                                       |
| ----------------------------------------------- | ---------------------------------------------------------------------------- |
| `T[row, column]`                                | Cell; `row` is 1 … `rows` or a key text, `column` is 1 … `columns` or a name |
| `T.rows`, `T.columns`                           | Number of rows and columns                                                   |
| `T.addRow(v1, v2, …)`                           | Appends a row (missing values = defaults) and returns its number             |
| `T.insertRow(i)`, `T.deleteRow(i)`              | Inserts an empty row before `i`; deletes row `i`                             |
| `T.clear()`                                     | Deletes all rows (columns stay)                                              |
| `T.find(column, value)`                         | First row whose cell equals `value`, or 0                                    |
| `T.column(column)`                              | The column as a `list` (a copy)                                              |
| `T.sum(c)`, `T.min(c)`, `T.max(c)`, `T.mean(c)` | Over a numeric or quantity column                                            |
| `T.sort(column, descending := false)`           | Sorts the rows; equal cells keep their order                                 |
| `random.row(T, weightColumn)`                   | A row drawn with probability proportional to the column (A17 §12)            |

A cell with a constant column has the column's type (`Orders[i, "gap"]` is a `length`); a column chosen at run time gives an `any` value (A17 §10.6). An index outside the table stops the run with `indexOutOfRange`.

## 6. CSV in the editor

Import and export use UTF-8, a header row with `name [unit]` (`gap [mm]`), a comma as separator and a point as decimal mark; values are converted from the header unit to SI. Import replaces the content through `data.setTable`, so it can be undone.

## 7. Validations

| Rule                  | Severity | When                                                               |
| --------------------- | -------- | ------------------------------------------------------------------ |
| `duplicateColumnName` | error    | Two columns with the same name (rejected by commands)              |
| `badCellValue`        | error    | A value that does not fit its column (rejected by commands)        |
| `duplicateKey`        | error    | Two rows with the same value in `keyColumn`                        |
| `keyColumnNotText`    | error    | `keyColumn` names a column that is not `text`                      |
| `unknownObjectInCell` | warning  | An `object` cell names a path that does not exist (error at start) |
| `nameNotIdentifier`   | warning  | The name cannot be used in code ([[S22 Area]] §5)                  |

## 8. Commands

- `data.setTable { instanceId, columns, rows }`: replaces columns and content.
- `data.setCells { instanceId, cells: [{ row, column, value }] }`: changes single cells (rows and columns numbered from 1); the editor's grid uses it.

## 9. Not included yet

Formulas in cells, charts fed from a table, linking a table to an external file during the run, and tables inside cells.

## 10. Reference tests

- `Orders["B", "weight"]` with `keyColumn` = destination returns 0.3 (example of §2).
- `random.row(Orders, "weight")` over 10 000 draws with a fixed seed gives shares of 0.7 / 0.3 within ±2 %, and the same rows for the same seed.
- `sort` is stable; `find` returns 0 for a missing value; `deleteRow(5)` on a 4-row table stops the run with `indexOutOfRange` naming the table, the line and the time.
- CSV export and import of the example give back the same content (length column round-trips through mm).

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                         |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | An information object placed on the plan as a family of category information; its content is edited in its own panel; listed in the Simulation tree |
| Discrete simulation | Read and written by IntraScript code and the controller during the run, as this spec says                                                           |
| Physics simulation  | The same as in the Discrete simulation module (information objects have no body)                                                                    |

The behaviour above does not change. **Rust status:** not ported yet (R10, after the owner answers Q20 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.
