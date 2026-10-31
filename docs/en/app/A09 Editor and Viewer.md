---
title: A09 Editor and Viewer
lang: en
tags: [app, editor, ui]
---

🌐 **EN** · [[A09 Editor y visor|ES]] · [[A09 Editor und Viewer|DE]] · [[Home (EN)|Index]]

# A09 Editor and Viewer (`apps/editor`)

## 0. The Rust desktop: three modules

Since 2026-10-31 the editor is BIMCraft's desktop with IntraCraft's modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]), application `platform/apps/intracraft` (`cargo run -p intracraft`). A selector at the top of the window switches between:

| Module              | Window                                                                                                                                                                                              |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | BIMCraft's ribbon, plan, sections, 3D view, Project Browser and Properties; the intralogistics families placed from the browser; attribute table with inherited flags; Simulation tab with the tree |
| Discrete simulation | The plan of the layout with animated pallets; Run, Pause, Step, speed, seed and duration; statistics sheet of the selected object; export to CSV and XLSX                                           |
| Physics simulation  | The 3D run in real time, drawn by a Bevy viewer ([[ADR-0022 Bevy Viewer for the Physics Simulation Module]]); the same controls; physics figures (stop error, impact energy)                        |

The run happens on a background thread; the window only receives snapshots, as the Web Worker did in the reference. Sections 1–3 below describe the TypeScript reference editor (frozen); they stay as the list of features the modules must offer.

## 1. The TypeScript reference editor

Run with `pnpm dev` → http://localhost:5173.

| File                                                                             | Role                                                                                                                                                                                                                                           |
| -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/App.tsx`                                                                    | Window layout: menu bar, toolbar, Components panel (left), viewport, Properties panel (right), asset catalogue (optional, right), status bar                                                                                                   |
| `src/components/MenuBar.tsx`                                                     | File (New layout, Open… Ctrl+O, Save Ctrl+S, Export results CSV), Edit (Undo Ctrl+Z, Redo Ctrl+Y, Rotate 90° R, Delete Del), View (3D / 2D top V, Frame all F, show panels, assets)                                                            |
| `src/components/Toolbar.tsx`, `icons.tsx`                                        | Icon buttons for the same actions, simulation-mode selector (not functional yet), language selector                                                                                                                                            |
| `src/actions.ts`                                                                 | Actions shared by menu, toolbar and keyboard shortcuts (Esc cancels a drag)                                                                                                                                                                    |
| `src/layout-store.ts`                                                            | Command session ([[ADR-0006 Command Package and Geometric Port Connections]]), selection, drag preview with snapping                                                                                                                           |
| `src/panel-store.ts`, `src/store.ts`                                             | Panel visibility; language, view, simulation mode and asset-catalogue state                                                                                                                                                                    |
| `src/components/LibraryPanel.tsx`                                                | Components panel: one card per definition with a top-view symbol (`ComponentSymbol.tsx`), dragged into the view                                                                                                                                |
| `src/components/Viewport.tsx`                                                    | `Canvas` in 3D perspective or 2D top view; drop target; picking by bounding box; drag to move; orbit controls                                                                                                                                  |
| `src/components/models/`                                                         | Parametric prototype models of the five conveyors, source and sink (rollers as one `InstancedMesh`, chains, frames, legs, turntable base and swept circle; the turntable platform turns during a run)                                          |
| `src/components/PortMarkers.tsx`                                                 | Port arrows: green connected, orange free, cyan = snap target                                                                                                                                                                                  |
| `src/components/PropertiesPanel.tsx`, `ParamField.tsx`, `AttributesEditor.tsx`   | Properties generated from the definition: identity (name, id, type, X/Z in mm, rotation in °), free attributes, parameters by group (choices and enums as lists, advanced folded, derived read-only, `dependsOn` disables), ports and warnings |
| `src/components/StatusBar.tsx`                                                   | Snap state while dragging, result of the last command, simulation clock and pallet count, object and connection counts                                                                                                                         |
| `src/sim/` (`sim.worker.ts`, `sim-runner.ts`, `protocol.ts`), `src/sim-store.ts` | 2D run in a Web Worker: Run/Pause (Space), Next event, Stop and reset, speed ×1 ×10 ×100 max; any layout edit resets the run                                                                                                                   |
| `src/components/PalletLayer.tsx`                                                 | Pallets of the run (base + load, two `InstancedMesh`), red when their conveyor is blocked, extrapolated between frames                                                                                                                         |
| `src/project-io.ts`                                                              | Save downloads `<name>.intracraft.json`; Open reads a file and runs `layout.load`; project name and random seed are edited in the Properties panel when nothing is selected                                                                    |
| `src/sensor-mount.ts`, `models/PhotoeyeModel.tsx`                                | Photocell: dropped on a roller or chain conveyor it is mounted there (host + position along); drawn on its host with posts and a beam that turns red while detecting                                                                           |
| `src/components/ControlPanel.tsx`, `src/signals.ts`                              | Controller rules (If/Then) with signal lists from the layout, live signal values, *Stop at sensor* template ([[ADR-0010 Controller Rules in the Layout and in 2D]])                                                                            |
| `src/components/RunResults.tsx`                                                  | Statistics and status of the selected object, run warnings                                                                                                                                                                                     |
| `src/components/CatalogPanel.tsx`, `AssetPreview.tsx`                            | Asset catalogue and preview ([[A16 Asset Catalog]]); models are dragged into the 3D view as scenery objects ([[S17 Scenery Asset]]) or, during a 3D run, dropped into it                                                                       |
| `src/units-display.ts`                                                           | SI ↔ display units (m ↔ mm, rad ↔ °)                                                                                                                                                                                                           |
| `src/i18n/strings.ts`, `useT.ts`                                                 | UI strings: English required, Spanish/German optional with English fallback                                                                                                                                                                    |

**Placing.** Drag a card into either view. The component follows the pointer (centre of its footprint). It snaps when one of its ports faces a free port within 300 mm (20 mm gap), otherwise it lands on the 100 mm grid. **Moving.** Press on an object and drag; release applies one `layout.move` (one undo step); R rotates during the drag. Connections are recomputed after every change. **Running.** Add a source and a sink, press Run (Space). The 3D physics mode is not available yet; the run always uses the 2D mode.

## 2. MVP feature list (to build)

- **Views:** 3D perspective and 2D top (orthographic), switchable with a key; orbit, zoom, frame selection.
- **Editing:** drag components from the library, move/rotate with gizmos, 100 mm grid with snapping to ports and grid, copy/paste, array repeat, undo/redo.
- **Connecting:** automatic port snapping with warnings on height, width and orientation.
- **Properties:** panel generated from the definition schema (groups, units, ranges, derived values, validation warnings) — [[A05 Component Library]].
- **Model tree:** list of objects with search, hide, lock.
- **Run:** play, pause, step, speeds ×1, ×10, ×100 (+ max in 2D); simulation clock; 2D/3D mode selector.
- **Live visualisation:** sensor beams green/red, stops up/down, pallets coloured by state, signal panel, statistics panel.
- **Controller:** If/Then rule editor with templates.
- **File:** ~~save/open `.intracraft.json`~~, ~~export statistics CSV~~ (done, `results-export.ts`), JSON export, screenshots.
- **Tools:** distance measurement, visible layers (guides, sensors, dimensions).

Postponed: background drawing (image/DXF), video recording, reports, run comparison.

## 3. Rendering rules

- Rollers and other repeated parts use `InstancedMesh` (one draw call per kind).
- The simulation runs in a Web Worker; the viewer interpolates snapshots ([[A02 Architecture]]).
- UI shows mm and degrees; everything stored is SI ([[A13 Conventions]]).
- The viewer never decides simulation results; it only displays snapshots.

## Related

[[A02 Architecture]] · [[A05 Component Library]] · [[A13 Conventions]]
