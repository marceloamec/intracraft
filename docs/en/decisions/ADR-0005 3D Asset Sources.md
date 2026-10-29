---
title: ADR-0005 3D Asset Sources
lang: en
tags: [decision, assets, license]
status: accepted
date: 2026-10-29
---

# ADR-0005 3D Asset Sources

- **Status:** Accepted
- **Date:** 2026-10-29

## Context

The editor needs scenery and load models (pallets, boxes, crates, racks, forklifts, fences). Two sources were reviewed on 2026-10-29:

- **NVIDIA Omniverse downloadable USD packs** (Warehouse, Industrial, SimReady Warehouse/Containers). Zips of 1.8–21 GB in OpenUSD; not downloadable per model and not loadable in three.js without conversion.
- **`codepawl/simready-glb` (Hugging Face)**: the same NVIDIA SimReady models (`nvidia/simready-assets`) converted to GLB, 2,595 files of 0.06–30 MB each, individually downloadable, licensed **CC BY 4.0**.

## Decision

- Use `codepawl/simready-glb`, pinned to a dataset revision (commit `e52172290393dde80e8d383861db94b8d0e92ec3`).
- Keep the `.glb` files **out of git**. The manifest `packages/asset-catalog/src/catalog.json` (id, category, names, size in m, bytes, SHA-256, source, licence) is the source of truth; `pnpm assets:fetch` downloads into `library-assets/` and verifies each file.
- Curation is reproducible: `scripts/asset-selection.json` + `node scripts/build-catalog.mjs` regenerate the manifest.
- Attribution is mandatory (CC BY 4.0): `ATTRIBUTION.md` and the `license` field of every entry. This respects the clean-room rule: openly licensed, not third-party simulation software.

## Consequences

- Assets are **visual only**: no ports, physics colliders or signals. The models carry rigid-body hints (mass, convex hull) that may be used later; not used now.
- NVIDIA ships no roller or chain conveyors, only belt conveyors (`conveyors` category, decorative). MVP conveyors stay parametric.
- Pallet models are not EUR/EPAL 1200 × 800 × 144 mm (e.g. `pallet_a01`: 1.204 × 0.209 × 0.800 m). The simulation pallet remains the one in [[S10 Pallet Types]].
- GLB files use `EXT_meshopt_compression`, `KHR_mesh_quantization` and WebP textures; the viewer needs the meshopt decoder. Positions are normalized integers; the node scale restores metres (handled in `scripts/lib/assets.mjs`).
- The third-party dataset could change or disappear; the pinned revision and SHA-256 hashes detect this.
