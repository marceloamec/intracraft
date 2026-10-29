---
title: A16 Asset Catalog
lang: en
tags: [app, library, assets]
updated: 2026-10-29
---

# A16 Asset Catalog

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
- Placing a model in the layout is not implemented: it needs a typed command with undo ([[A15 AI Agent Integration]]) and a layout model for scenery.

## 4. Adding a model

1. Add an entry to `scripts/asset-selection.json` (names the dataset file without `.glb`).
2. Run `node scripts/build-catalog.mjs`; check the printed size.
3. Run `pnpm test`.
