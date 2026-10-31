---
title: S21 Code Block
lang: en
tags: [spec, information, scripting, control]
library-id: info.code
status: proposed — waiting for the owner's confirmation
updated: 2026-10-31
---

# S21 Code Block (`info.code`)

A **piece of IntraScript code** on the 2D plan ([[A17 IntraScript Language]]). It holds block variables, methods and event handlers (`on …`). Placing it inside an area lets its code use the area's objects by their short names ([[S22 Area]]). Decision: [[ADR-0018 IntraScript Scripting Language]].

## 1. Parameters

| Key          | Unit | Default | Range       | Description                                                            |
| ------------ | ---- | ------- | ----------- | ---------------------------------------------------------------------- |
| `enabled`    | —    | true    | true, false | A disabled block is not checked before a run and none of its code runs |
| `showOnPlan` | —    | true    | true, false | Show the name and the number of check errors next to the icon in 2D    |

## 2. Project file

```json
"content": {
  "kind": "code",
  "languageVersion": 1,
  "source": "on Photoeye1.detected rises\n  Counted += 1\nend\n"
}
```

`languageVersion` lets later versions of IntraScript upgrade old code with an explicit migration, like `schemaVersion` for the project file ([[A04 Data Model and Project File]]). At most 200 000 characters per block.

## 3. On the plan

A small icon with the symbol **{ }** and the name. When the block has check errors the icon turns red and shows their number; warnings show an amber dot. Not drawn in 3D.

## 4. Editing

- The editor opens the code in a code panel with colours for keywords, names, units and comments, the check errors and warnings underlined at their line, completion of names and members, and the `///` help of methods when hovering.
- Every change is checked against the current layout (names, members, types and units). Code with errors **can be saved**: work in progress is never lost. It only blocks the start of a run.
- Renaming or deleting an object that code refers to makes the references check errors (`unknownName`); the editor lists every code block affected.

## 5. During a run

1. **Start:** all enabled blocks are checked; any error stops the start with the list of errors. Block variables get their initial values; handlers are registered; `on start` handlers run.
2. **Events:** handlers start and wait as described in A17 §9 and §14, the same way in 2D and 3D.
3. **End:** `on finish` handlers run (they cannot wait); waiting activations are dropped.
4. A run error stops the run and shows the block, the line, the column, the time and the message; the editor opens the block at that line.

Several blocks may handle the same event; at the same instant and priority they run in layout order. Each block has its own random stream (A17 §14.5).

## 6. In code

From other blocks: `Routing.nextDestination()` calls a method; `Routing.routed` reads or writes a block variable; `Routing.enabled` reads the parameter. Inside the block: `self`.

## 7. Statistics

| Key           | Meaning                                                    |
| ------------- | ---------------------------------------------------------- |
| `activations` | Handler activations started during the run                 |
| `waiting`     | Activations waiting at the end of the run                  |
| `steps`       | Statements executed (to find code that does too much work) |

## 8. Validations

| Rule                | Severity | When                                                             |
| ------------------- | -------- | ---------------------------------------------------------------- |
| `scriptErrors`      | error    | The code has check errors (A17 §13.1); blocks the start of a run |
| `scriptWarnings`    | warning  | The code has check warnings                                      |
| `noHandlers`        | info     | No `on` handler and no method called from another block          |
| `nameNotIdentifier` | warning  | The name cannot be used in code ([[S22 Area]] §5)                |

## 9. Commands and queries

- `script.setCode { instanceId, source }`: replaces the code (with undo). It is accepted even with check errors and returns them as issues, so agents and the editor see them at once.
- `script.check { instanceId? }` (query): check errors and warnings of one block or of all blocks against the current layout.

## 10. Not included yet

Breakpoints and step-by-step execution, a watch window for block variables during a run, code shared as a library between projects, and a visual block editor that writes IntraScript.

## 11. Reference tests

- Scenario **IS-1** (counter): Source → Roller1 with Photoeye1 → Sink; `on Photoeye1.detected rises` adds 1 to `Counted`; after 1 h `Counted` equals `Sink1.stats.palletsReceived` plus the pallets between the photocell and the sink.
- Scenario **IS-2** (routing by order table): a turntable with two exits and `on choose exit of` reading `pallet.destination` set in `on create`; the share of pallets at each sink matches the weights within ±2 % over 10 000 pallets and is identical for the same seed.
- Scenario **IS-3** (timed stop): `wait 2 s` at a photocell delays each pallet by exactly 2 s in 2D; in 3D the photocell events of IS-1 to IS-3 agree with 2D within ±50 ms.
- A block with a typo cannot start a run; disabling it lets the run start.
- `while true` without `wait` stops the run with `stepLimit` instead of freezing the editor.
