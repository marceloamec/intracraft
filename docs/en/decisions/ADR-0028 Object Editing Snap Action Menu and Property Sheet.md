---
title: ADR-0028 Object Editing - Snap, Action Menu and Property Sheet
lang: en
tags: [decision, design, editor, ui]
status: accepted
date: 2026-11-09
---

# ADR-0028 Object Editing: Snap, Action Menu and Property Sheet

- **Status:** Accepted and built.
- **Date:** 2026-11-09

## Context

In the Rust platform ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]) the Design module had no snapping
between conveyors, and the only place to change an object's values was a text grid in the side panel. The 3D view
inside the main window ([[ADR-0026 Bevy and Avian for the Physics Simulation Module]]) could only be looked at.

The owner asked for four things:

1. Snapping between conveyors, where it makes sense, in the 2D plan.
2. Moving objects in the 3D view, with the same snapping.
3. An action menu on every object clicked, in 2D and in 3D, with Properties among its items.
4. A property window per object, which floats or sits at the side, with sections (not tabs) and one control per value
   type: drop-down lists, tick boxes, whole numbers and numbers with units.

They chose, out of two alternatives each:

- **Where 3D editing happens:** in IntraCraft's own 3D view, inside the main window (2D + 3D). The Bevy window stays a
  viewer: making it edit would take two programs talking to each other, for little gain.
- **How the menu opens:** with a right click. A left click selects and drags, as before; a double click opens the
  properties.
- **How the properties show:** a floating window per object, with a button to dock it in the side panel, rather than a
  BIMCraft palette (which would need BIMCraft changes and shows only one at a time).

## Decision

1. **Snapping is the library's** (`intracraft_library::snap`, ported from the TypeScript reference, [[A09 Editor and Viewer]]).
   A new variant, `snap_fitting`, only lines an out port up with an in port (or an inout with either), so a conveyor
   never snaps backwards onto another.
   - `intracraft_model::snap::snap_move` applies it to a drag in the model's millimetres. One intralogistics object
     dragged within 300 mm of a free port that faces one of its own ends with its port 20 mm from it; else its entry
     falls on the 100 mm grid.
   - Anything else (two objects, a wall) moves as dragged.
   - A join at different heights still snaps, and the status bar says by how much they differ.
2. **BIMCraft gets four generic hooks**, all inert without an application (`platform/INTRACRAFT-CHANGES.md`):
   - `BimApp::move_snap` corrects a drag and the Move tool, and their preview;
   - `extension_actions` and `extension_chosen` add items to the right-click menu;
   - `double_clicked` hands over a component double-clicked;
   - a right click on an element outside the selection picks it first, so the menu is of what is under the cursor.
3. **The action menu** (`apps/intracraft/src/actions.rs`) offers, for one intralogistics object:
   - Properties…;
   - in the Design module only: Turn 90° left or right (about its entry, BIMCraft's `rotate`) and Duplicate after its
     exit (a copy joined to its free out port, one undo step).
   BIMCraft's own items (Delete, Hide…) follow. The 3D view shows the same items.
4. **Placing from the ribbon with one object selected puts the new object after it**, joined to its free exit
   (`place_after`). With nothing selected, it goes to the next free spot as before. This replaces the "Add after ▸"
   submenu first planned: BIMCraft's menu has no submenus, and the ribbon already lists the library.
5. **The property sheet** (`apps/intracraft/src/properties.rs`):
   - is generated from the definition: one section per parameter `group`, in the definition's order, with the
     `advanced` values folded in each;
   - each value gets the control its type asks for:

| Type                         | Control                                            |
| ---------------------------- | -------------------------------------------------- |
| number                       | a drag field with its display unit, range and step |
| number with `choices`        | a drop-down list                                   |
| integer                      | a drag field without decimals                      |
| boolean                      | a tick box                                         |
| enum                         | a drop-down list with `optionLabels`               |
| text                         | a text field                                       |
| derived                      | shown, not edited                                  |
| not applicable (`dependsOn`) | greyed out, with the condition in its tip          |

   - a square beside each value marks it inherited (■) or the object's own (□); a click on □ inherits it again;
   - each change applies when the field is let go: one command, one undo step. A refused value is marked under its
     field;
   - in the simulation modules the sheet is read-only, with the object's state and statistics first;
   - it replaces the text grid in the side panel.
6. **In the 3D view** a ray from the camera through the cursor picks the nearest object (by the box round its parts).
   - A click selects; the selection is the plan's, and the selected object is tinted blue.
   - In the Design module, a drag from an object slides it over the floor at the height it was taken at. It snaps
     through the same `move_snap`, shows where it would land, and is applied with `move` when let go.
   - A drag from the floor orbits, as before.

## Consequences

- **Snapping, the menu and the sheet are the same in 2D and 3D:** one function each, tested without a window.
- **The 3D view in the Design module re-reads the layout only when the model changes,** like the plan.
- **Turning has no keyboard shortcut in 3D yet.** BIMCraft's two-letter shortcuts (RO, MV…) would clash with single
  keys; it is done from the menu.
- **The picking box is axis-aligned:** an object turned 45° can be picked a little outside its outline, and the nearest
  box wins.

## Related

[[A09 Editor and Viewer]] · [[A05 Component Library]] · [[ADR-0025 Simulation Modules on BIMCraft's Ribbon and Detailed Families]] ·
[[ADR-0026 Bevy and Avian for the Physics Simulation Module]]
