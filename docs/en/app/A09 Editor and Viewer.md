---
title: A09 Editor and Viewer
lang: en
tags: [app, editor, ui]
---

🌐 **EN** · [[A09 Editor y visor|ES]] · [[A09 Editor und Viewer|DE]] · [[Home (EN)|Index]]

# A09 Editor and Viewer (`apps/editor`)

## 1. What exists today

Run with `pnpm dev` → http://localhost:5173.

| File                                                                           | Role                                                                                                                                                                                                                                           |
| ------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/App.tsx`                                                                  | Window layout: menu bar, toolbar, Components panel (left), viewport, Properties panel (right), asset catalogue (optional, right), status bar                                                                                                   |
| `src/components/MenuBar.tsx`                                                   | File (New layout), Edit (Undo Ctrl+Z, Redo Ctrl+Y, Rotate 90° R, Delete Del), View (3D / 2D top V, show panels, assets)                                                                                                                        |
| `src/components/Toolbar.tsx`, `icons.tsx`                                      | Icon buttons for the same actions, simulation-mode selector (not functional yet), language selector                                                                                                                                            |
| `src/actions.ts`                                                               | Actions shared by menu, toolbar and keyboard shortcuts (Esc cancels a drag)                                                                                                                                                                    |
| `src/layout-store.ts`                                                          | Command session ([[ADR-0006 Command Package and Geometric Port Connections]]), selection, drag preview with snapping                                                                                                                           |
| `src/panel-store.ts`, `src/store.ts`                                           | Panel visibility; language, view, simulation mode and asset-catalogue state                                                                                                                                                                    |
| `src/components/LibraryPanel.tsx`                                              | Components panel: one card per definition with a top-view symbol (`ComponentSymbol.tsx`), dragged into the view                                                                                                                                |
| `src/components/Viewport.tsx`                                                  | `Canvas` in 3D perspective or 2D top view; drop target; picking by bounding box; drag to move; orbit controls                                                                                                                                  |
| `src/components/models/`                                                       | Parametric prototype models of the five conveyors (rollers as one `InstancedMesh`, chains, frames, legs, turntable base and swept circle)                                                                                                      |
| `src/components/PortMarkers.tsx`                                               | Port arrows: green connected, orange free, cyan = snap target                                                                                                                                                                                  |
| `src/components/PropertiesPanel.tsx`, `ParamField.tsx`, `AttributesEditor.tsx` | Properties generated from the definition: identity (name, id, type, X/Z in mm, rotation in °), free attributes, parameters by group (choices and enums as lists, advanced folded, derived read-only, `dependsOn` disables), ports and warnings |
| `src/components/StatusBar.tsx`                                                 | Snap state while dragging, result of the last command, object and connection counts                                                                                                                                                            |
| `src/components/CatalogPanel.tsx`, `AssetPreview.tsx`                          | Decorative 3D asset catalogue and preview ([[A16 Asset Catalog]])                                                                                                                                                                              |
| `src/units-display.ts`                                                         | SI ↔ display units (m ↔ mm, rad ↔ °)                                                                                                                                                                                                           |
| `src/i18n/strings.ts`, `useT.ts`                                               | UI strings: English required, Spanish/German optional with English fallback                                                                                                                                                                    |

**Placing.** Drag a card into either view. The component follows the pointer (centre of its footprint). It snaps when one of its ports faces a free port within 300 mm (20 mm gap), otherwise it lands on the 100 mm grid. **Moving.** Press on an object and drag; release applies one `layout.move` (one undo step); R rotates during the drag. Connections are recomputed after every change.

## 2. MVP feature list (to build)

- **Views:** 3D perspective and 2D top (orthographic), switchable with a key; orbit, zoom, frame selection.
- **Editing:** drag components from the library, move/rotate with gizmos, 100 mm grid with snapping to ports and grid, copy/paste, array repeat, undo/redo.
- **Connecting:** automatic port snapping with warnings on height, width and orientation.
- **Properties:** panel generated from the definition schema (groups, units, ranges, derived values, validation warnings) — [[A05 Component Library]].
- **Model tree:** list of objects with search, hide, lock.
- **Run:** play, pause, step, speeds ×1, ×10, ×100 (+ max in 2D); simulation clock; 2D/3D mode selector.
- **Live visualisation:** sensor beams green/red, stops up/down, pallets coloured by state, signal panel, statistics panel.
- **Controller:** If/Then rule editor with templates.
- **File:** save/open `.intracraft.json`, export statistics CSV/JSON, screenshots.
- **Tools:** distance measurement, visible layers (guides, sensors, dimensions).

Postponed: background drawing (image/DXF), video recording, reports, run comparison.

## 3. Rendering rules

- Rollers and other repeated parts use `InstancedMesh` (one draw call per kind).
- The simulation runs in a Web Worker; the viewer interpolates snapshots ([[A02 Architecture]]).
- UI shows mm and degrees; everything stored is SI ([[A13 Conventions]]).
- The viewer never decides simulation results; it only displays snapshots.

## Related

[[A02 Architecture]] · [[A05 Component Library]] · [[A13 Conventions]]
