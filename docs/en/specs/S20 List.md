---
title: S20 List
lang: en
tags: [spec, information, scripting]
library-id: info.list
status: proposed — waiting for the owner's confirmation
updated: 2026-10-31
---

# S20 List (`info.list`)

A **list object** on the 2D plan: a queue of pallets waiting for a lift, a stack of empty pallets, the list of destinations still open. It is a single typed column with queue or stack behaviour and its own statistics. Inside code, lists can also be plain values (`list of T`, [[A17 IntraScript Language]] §4.4); the object is for lists that several code blocks share or that should show statistics. Decision: [[ADR-0018 IntraScript Scripting Language]].

## 1. Parameters

| Key            | Unit | Default  | Range                                                                            | Description                                                  |
| -------------- | ---- | -------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `elementType`  | enum | `pallet` | The types of [[S19 Table]] §2 (`object`, `pallet`, `text`, numbers, quantities…) | Type of every item                                           |
| `mode`         | enum | `queue`  | `list`, `queue`, `stack`                                                         | What `push` and `pop` do (§4)                                |
| `capacity`     | —    | 0        | 0 … 100 000                                                                      | Maximum number of items; 0 = no limit                        |
| `resetAtStart` | —    | true     | true, false                                                                      | Start every run with the initial items                       |
| `showOnPlan`   | —    | true     | true, false                                                                      | Show the name and the number of items next to the icon in 2D |

## 2. Project file

```json
"content": { "kind": "list", "items": ["A", "B", "C"] }
```

Items follow the cell rules of [[S19 Table]] §2. A list of pallets always starts empty.

## 3. On the plan

A small icon with three stacked bars, the name and, when `showOnPlan`, the number of items. Not drawn in 3D. It can be placed inside an area ([[S22 Area]]).

## 4. In code

| Member    | `list`          | `queue` (first in, first out) | `stack` (last in, first out) |
| --------- | --------------- | ----------------------------- | ---------------------------- |
| `push(x)` | Adds at the end | Adds at the end               | Adds at the end              |
| `pop()`   | Takes the last  | Takes the first               | Takes the last               |
| `peek()`  | Shows the last  | Shows the first               | Shows the last               |

Also: `count`, `isEmpty`, `isFull`, `[i]`, `first`, `last`, `add`, `insert`, `removeAt`, `remove`, `contains`, `indexOf`, `clear`, `sort`, `toList()` (a copy as a list value), and `stats.<key>`. `pop` or `peek` on an empty list stops the run with `emptyList`; `push` on a full list with `listFull` — check `isEmpty` and `isFull` first. Every change raises `on <name> changes` (A17 §9).

```intrascript
on enter LiftInfeed
  Waiting.push(pallet)
end

on Lift1.atStation rises
  if not Waiting.isEmpty
    var next: pallet := Waiting.pop()
    ...
  end
end
```

A pallet that leaves the model (sink or `delete()`) is removed from every list object at that moment, so a list never holds pallets that no longer exist.

## 5. Statistics

| Key              | Meaning                                                             |
| ---------------- | ------------------------------------------------------------------- |
| `maxCount`       | Largest number of items during the run                              |
| `meanCount`      | Number of items averaged over time                                  |
| `pushed`         | Items added during the run                                          |
| `meanTimeInList` | Mean time between `push` and `pop` of an item (`queue` and `stack`) |

## 6. Validations

| Rule                | Severity | When                                                       |
| ------------------- | -------- | ---------------------------------------------------------- |
| `badItemValue`      | error    | An initial item that does not fit `elementType` (rejected) |
| `tooManyItems`      | error    | More initial items than `capacity`                         |
| `nameNotIdentifier` | warning  | The name cannot be used in code ([[S22 Area]] §5)          |

## 7. Commands

- `data.setItems { instanceId, items }`: replaces the initial items.

## 8. Not included yet

Priority queues (sorted on insertion; use `sort` meanwhile), lists of lists as objects, and charts of the count over time.

## 9. Reference tests

- Queue: push A, B, C → pop gives A; stack: pop gives C; list: pop gives C.
- `meanCount` of a list that holds 2 items for 10 s and 0 for 10 s is 1.
- A pallet in `Waiting` that arrives at a sink is no longer in `Waiting`.
- `pop` on an empty queue stops the run with `emptyList`, naming the list, the code block, the line and the time.
