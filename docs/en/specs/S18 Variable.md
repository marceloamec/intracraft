---
title: S18 Variable
lang: en
tags: [spec, information, scripting]
library-id: info.variable
status: proposed — waiting for the owner's confirmation
updated: 2026-10-31
---

# S18 Variable (`info.variable`)

A **named value** on the 2D plan: a counter, a level, a switch, a target speed. Code reads and writes it ([[A17 IntraScript Language]]); the plan shows its current value during a run; numeric variables keep statistics. Decision: [[ADR-0018 IntraScript Scripting Language]].

## 1. Parameters

| Key            | Unit | Default   | Range                                                                                                                    | Description                                        |
| -------------- | ---- | --------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------- |
| `valueType`    | enum | `integer` | `integer`, `real`, `boolean`, `text`, `time`, `length`, `speed`, `acceleration`, `mass`, `angle`, `angularSpeed`, `rate` | Type of the value (A17 §4)                         |
| `resetAtStart` | —    | true      | true, false                                                                                                              | Start every run with the initial value             |
| `showOnPlan`   | —    | true      | true, false                                                                                                              | Show the name and the value next to the icon in 2D |

No ports, no signals. The **initial value** is content, not a parameter, because its type depends on `valueType` (§2).

## 2. Project file

```json
{ "id": "variable1", "definitionId": "info.variable", "name": "Counted",
  "params": { "valueType": "integer", "resetAtStart": true, "showOnPlan": true },
  "content": { "kind": "variable", "value": 0 } }
```

`content.value` is stored in SI units (a `length` of 150 mm is `0.15`), as a boolean for `boolean` and as a string for `text`. Changing `valueType` converts the value when possible (integer ↔ real, quantity → same number) and otherwise resets it to the type's default (A17 §4.1), with a warning.

## 3. On the plan

A small square icon with the letter **x**, the name and, when `showOnPlan`, the value in display units (`Counted = 12`, `Gap = 150 mm`). Not drawn in 3D. It can be placed inside an area ([[S22 Area]]).

## 4. During a run (2D and 3D alike)

- At the start the running value is the initial value (or, with `resetAtStart` = false, the value left by the previous run of the same session; a new session or a loaded project always starts from the initial value).
- Code changes it; every change raises the event `on <name> changes` (A17 §9).
- The value at the end of a run belongs to the results, never to the project file.

## 5. In code

```intrascript
Counted += 1                    // the name is the value
if Counted.stats.max > 20       // a name followed by "." is the object
  warn("More than 20 pallets counted at once")
end
```

Members: `value` (read and write), `initialValue` (read), `name`, `parent`, `stats.<key>`.

## 6. Statistics (numeric types only)

| Key        | Meaning                                     |
| ---------- | ------------------------------------------- |
| `min`      | Smallest value during the run               |
| `max`      | Largest value during the run                |
| `timeMean` | Mean value weighted by the time it was held |
| `changes`  | Number of changes                           |

## 7. Validations

| Rule                | Severity | When                                                             |
| ------------------- | -------- | ---------------------------------------------------------------- |
| `badInitialValue`   | error    | The stored value does not fit `valueType` (rejected by commands) |
| `nameNotIdentifier` | warning  | The name cannot be used in code ([[S22 Area]] §5)                |
| `unusedVariable`    | info     | No code block refers to it                                       |

## 8. Commands

- `layout.addInstance`, `layout.move`, `layout.setParams`, `layout.remove` as for every object.
- `data.setVariable { instanceId, value }`: sets the initial value; rejects a value that does not fit `valueType`.

## 9. Not included yet

Changing the value of a running model from the editor, charts of the value over time, and arrays (use [[S20 List]] or [[S19 Table]]).

## 10. Reference tests

- A variable with initial value 5 and `resetAtStart` reads 5 at the start of two consecutive runs, whatever the first run did.
- `Counted += 1` on every rising edge of a photocell equals the number of pallets that passed it (same as the sink count on a straight line).
- `timeMean` of a variable that is 0 for 10 s and 4 for 30 s is 3.
- `data.setVariable` with `"five"` on an integer variable is rejected; undo restores the previous value.
