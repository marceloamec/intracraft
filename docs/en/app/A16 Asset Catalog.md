---
title: A16 Asset Catalog
lang: en
tags: [app, library, assets]
updated: 2026-10-29
---

# A16 Asset Catalog

> **Rust platform (2026-10-31):** this note describes the TypeScript reference, which is frozen ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). The catalogue (`catalog.json`, `colliders.json`) is reused as is: its GLB models become family mesh forms in the Design module and bodies in the Physics simulation module.

Catalogue of third-party 3D models (GLB) used as scenery and loads. Decision and sources: [[ADR-0005 3D Asset Sources]]. Parametric components are described in [[A05 Component Library]].

## 1. Layout

| Path                                            | Content                                                     |
| ----------------------------------------------- | ----------------------------------------------------------- |
| `packages/asset-catalog/src/catalog.json`       | Manifest (in git): 40 models, sizes in m, hashes, licence   |
| `packages/asset-catalog/src/catalog-schema.ts`  | Zod schema and the category list                            |
| `packages/asset-catalog/src/catalog-queries.ts` | Query layer: `list`, `get`, `byCategory`, `search`, `urlOf` |
| `library-assets/models/<category>/*.glb`        | Downloaded files (not in git, about 46 MB)                  |
| `scripts/asset-selection.json`                  | Curated selection (source name, category, names, tags)      |
| `scripts/fetch-assets.mjs`                      | `pnpm assets:fetch`: download and verify                    |
| `scripts/build-catalog.mjs`                     | Regenerates the manifest from the selection                 |

## 2. Categories

`pallets`, `boxes`, `containers`, `drums`, `racks`, `vehicles`, `conveyors` (decorative belts), `safety` (fences, gates, ramps).

## 3. Use

- Models are Y-up, in metres. `dimensions` is the bounding box (x width, y height, z depth).
- Ids look like `asset.pallets.pallet_a01`. Names have `en` (required) and optional `es`/`de`.
- The editor opens the catalogue with the **Assets** toggle in the toolbar: search, category filter, orbitable preview. The dev server and the build serve `library-assets/` at `/library-assets/`.
- **Placing a model** ([[S17 Scenery Asset]], [[ADR-0017 Scenery Assets in the Layout and Live Spawn]]): drag it from the catalogue into the 3D view. Choose first how it enters the 3D run: drop from the air (with a drop height), resting in place, or fixed to the floor. It stands on the surface under the pointer and becomes a `scenery.asset` instance through `layout.addInstance` (undo, project file). During a 3D run the drop becomes the simulation command `sim.spawnObject` instead: the object falls into the running world and disappears when the run stops.
- **Physics data:** `packages/asset-catalog/src/colliders.json` holds the convex pieces of every model (millimetres, imported as `@intracraft/asset-catalog/colliders`, only by the physics world); `contactMaterial` in `catalog.json` is the S16 material of the largest piece.
- **Sources:** a source can create a catalogue pallet with a catalogue load, or the load alone ([[S09 Source and Sink]] §1.1); the 3D view draws those models.

## 4. IntraCraft's own pallet and load models (Rust platform, 2026-11-09)

The catalogue's third-party models are meshopt-compressed, and the Rust readers refuse them
([[ADR-0022 Bevy Viewer for the Physics Simulation Module]]). The Rust platform therefore draws pallets and loads with
models of its own, `intracraft-assets` ([[ADR-0026 Bevy and Avian for the Physics Simulation Module]]).

| Model  | What it is                                                                                                                                                                      |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pallet | Each pallet type of the library (`eur`, `iso1200x1000`, `gma`), built from its occupancy bands: what the 3D view shows and what a photocell sees are the same boards and blocks |
| Load   | Cartons in layers, the same in stretch film, steel drums, plastic crates; built at the source's load height on the pallet's footprint                                           |

A source's choice picks the load:

- `block` gives cartons in stretch film;
- an asset of the catalogue gives the load of its category (boxes, drums, containers).

`cargo run -p intracraft-assets` writes the models as glTF 2.0 binary files into
`platform/apps/intracraft-viewer/assets/pallets/`, for any 3D program. A test keeps them up to date with their models.

## 5. Adding a model

1. Add an entry to `scripts/asset-selection.json` (names the dataset file without `.glb`).
2. Run `node scripts/build-catalog.mjs`; check the printed size, number of collision pieces and contact material.
3. Run `pnpm test`.
