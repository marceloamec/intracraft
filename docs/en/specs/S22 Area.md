---
title: S22 Area
lang: en
tags: [spec, information, scripting, layout]
library-id: info.area
status: proposed — waiting for the owner's confirmation
updated: 2026-10-31
---

# S22 Area (`info.area`)

A **named rectangle on the 2D plan that contains objects**: a hall, a line, a loop, a pick station. Areas give the model its hierarchy (`root` → areas → objects) and its paths (`Hall1.LineA.Roller3`), and they let code use short names that keep working when an area is duplicated ([[A17 IntraScript Language]] §10). Decision: [[ADR-0018 IntraScript Scripting Language]].

## 1. Parameters

| Key          | Unit | Default | Range       | Description                         |
| ------------ | ---- | ------- | ----------- | ----------------------------------- |
| `width`      | m    | 10      | 0.5 … 500   | Size of the rectangle along x       |
| `depth`      | m    | 6       | 0.5 … 500   | Size of the rectangle along z       |
| `showOnPlan` | —    | true    | true, false | Draw the outline and the name in 2D |

The area's position is the centre of the rectangle; its `yaw` turns it like any other object.

## 2. Membership

- Every object has an optional **parent**: the id of the area that contains it (`parentId` in the project file). Without one it belongs to `root`.
- Areas may contain areas, up to 8 levels. A cycle (an area inside itself) is rejected.
- Membership is explicit: the command `layout.setParent` sets it, and the editor offers it when an object is dropped inside an area's rectangle. An object outside its area's rectangle stays a member (warning `outsideArea`).
- Moving an area moves all its descendants by the same offset; turning it turns them around its centre. Removing an area moves its children to its own parent; the area itself is removed.
- Connections and signals do not care about areas: a conveyor in `LineA` may connect to one in `LineB`.

```json
{ "id": "area1", "definitionId": "info.area", "name": "LineA", "parentId": "area2", "...": "..." }
{ "id": "roller3", "definitionId": "conveyor.roller", "name": "Roller3", "parentId": "area1", "...": "..." }
```

`parentId` is optional, so existing projects stay valid without a migration.

## 3. On the plan and in 3D

A dashed outline with the name in a corner, drawn below the objects. Selecting the name selects the area; a click inside the rectangle still selects the object under the cursor. Not drawn in 3D.

## 4. In code

| Member                      | Result                                                               |
| --------------------------- | -------------------------------------------------------------------- |
| `LineA.Roller3`             | Child by name                                                        |
| `LineA.objects(kind)`       | Direct children of a kind (`object`, `conveyor`, …), in layout order |
| `LineA.allObjects(kind)`    | All descendants of a kind                                            |
| `LineA.find("Sub.Roller3")` | Descendant by relative path, or `none`                               |
| `LineA.pallets`             | Pallets on all objects inside the area                               |
| `name`, `parent`            | Name; enclosing area (`root` at the top)                             |

`root` is the area of the whole layout; it also has `byId(id)`.

## 5. Names

Code refers to objects by **name** ([[A17 IntraScript Language]] §10.2), so names follow these rules for every object, not only areas:

| Rule                | Severity | When                                                                                 |
| ------------------- | -------- | ------------------------------------------------------------------------------------ |
| `duplicateName`     | error    | Two objects in the same area with the same name (not case-sensitive)                 |
| `nameNotIdentifier` | warning  | The name has spaces or symbols; code can reach the object only with `root.byId("…")` |
| `reservedName`      | error    | The name is a reserved word of IntraScript (`end`, `parent`, `pallet`…)              |

Default names are the object ids (`roller1`, `photoeye2`), which already satisfy the rules. A run does not start while a code block refers to an object through a name that breaks them.

## 6. Area-specific validations

| Rule          | Severity | When                                                   |
| ------------- | -------- | ------------------------------------------------------ |
| `parentCycle` | error    | An area would contain itself (rejected by the command) |
| `tooDeep`     | error    | More than 8 nested levels (rejected by the command)    |
| `outsideArea` | info     | A member's position is outside its area's rectangle    |
| `emptyArea`   | info     | An area without members                                |

## 7. Commands

- `layout.setParent { instanceId, parentId | null }`: puts an object into an area or back into `root`.
- `layout.move` on an area also moves its descendants (one undo step).
- `layout.remove` on an area keeps its children (they move to its parent).

## 8. Not included yet

Duplicating an area with everything inside (the reason for relative names; arrives with copy and paste in the editor), collapsing an area into a single symbol, area statistics, and areas drawn in 3D.

## 9. Reference tests

- `Hall1.LineA.Roller3` and a bare `Roller3` from a code block inside `LineA` reach the same object; from a block in `root`, a bare `Roller3` is `unknownName` when the only `Roller3` is inside `LineA`.
- Two areas with their own `Roller3` and the same code in a code block each: each block drives its own conveyor.
- `layout.setParent` that would make a cycle is rejected; moving an area moves its members by the same offset and undo restores all positions.
- A project file without `parentId` loads unchanged.

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                         |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | An information object placed on the plan as a family of category information; its content is edited in its own panel; listed in the Simulation tree |
| Discrete simulation | Read and written by IntraScript code and the controller during the run, as this spec says                                                           |
| Physics simulation  | The same as in the Discrete simulation module (information objects have no body)                                                                    |

The behaviour above does not change. **Rust status:** not ported yet (R10, after the owner answers Q20 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.
