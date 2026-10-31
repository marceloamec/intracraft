---
title: ADR-0017 Scenery Assets in the Layout and Live Spawn
lang: en
tags: [decision, physics, assets, layout, commands]
status: proposed
date: 2026-10-31
---

# ADR-0017 Scenery Assets in the Layout and Live Spawn

- **Status:** Proposed on 2026-10-31, after the owner chose the options below (placement in the layout and live, convex hulls from the model, pallet model plus catalogue load in the source).
- **Date:** 2026-10-31

## Context

The asset catalogue ([[A16 Asset Catalog]], [[ADR-0005 3D Asset Sources]]) holds 43 SimReady models with mass, centre of mass and inertia, but they were only shown in a preview. The owner wants to drag them into the 3D scene with physics, choosing to drop them from the air, leave them resting or fix them to the floor, both while editing and during a running 3D simulation, and to let the pallet source create a catalogue pallet with a catalogue load. Gaps found on 2026-10-31:

- The physics wrapper had box and cylinder shapes only.
- The catalogue stored no collision geometry.
- The contact pair table ([[S16 Contact Materials]]) only had wood and plastic rows, so a cardboard or metal object touching the floor would stop the run.
- The layout had no place for scenery, and any layout change resets a running simulation.

## Decision

1. **Scenery is a registered definition**, `scenery.asset` ([[S17 Scenery Asset]]), built in `createBuiltInRegistry()` from the catalogue (its `asset` options are the catalogue ids, with their names as option labels). No `schemaVersion` change: add, move, set parameters, remove, undo, load and the command log work unchanged. The definition schema gains a `scenery` category and optional `optionLabels`.
2. **Collision shapes are convex hulls taken from the model's own collider declarations** (`KHR_physics_rigid_bodies.collider`, one per node). The catalogue script decodes the binary chunk (`EXT_meshopt_compression`, `KHR_mesh_quantization`), moves each collider's vertices into the model frame, builds its convex hull and keeps at most 64 corners. The hulls are stored in `packages/asset-catalog/src/colliders.json` (in git, derived data under CC BY 4.0) so tests and the physics world do not need the downloaded models. Development-only dependencies for the script: `meshoptimizer` and `quickhull3d` (both MIT).
3. **Pallets on conveyors keep the MTBH construction** ([[ADR-0015 MTBH Rigid Body from Pallet and Load Models]]): a hull would fill the gaps between bottom boards that the photocells and the rollers must see. Free-standing pallets placed as scenery use their hulls; they are scenery, not flow items.
4. **Materials.** One contact material per model: the SimReady family of its largest collision piece, mapped to S16 (§2.1 there). New material `cardboard`. The pair table now covers all 55 pairs of the ten materials; the rule "missing pair = error" stays.
5. **Sleeping.** Dynamic scenery bodies may sleep when still (`BodyDesc.allowSleep`). Pallets and every existing body keep `allowSleep = false`, so existing results do not change.
6. **Live spawn is a simulation command**, `sim.spawnObject` (Zod schema, actor-logged, applied at the next fixed step). It does not change the layout, cannot be undone, is not saved and is cleared with the run. It satisfies the AGENTS.md rule that every action changing the simulation is a typed command, without forcing a reset of the run.
7. **The source's catalogue load is rigidly joined to the pallet**, like today's load block: one compound body. Its mass is the source's `loadMass` (catalogue masses describe empty packaging).
8. **2D ignores scenery** without a warning; the flow network skips `scenery.*` definitions.

## Friction and restitution values

Values are **indicative**, taken from general engineering tables on 2026-10-31 and rounded; the ranges in S16 show the spread. None is measured on the owner's equipment.

| Pairs                                 | Basis                                                                                               |
| ------------------------------------- | --------------------------------------------------------------------------------------------------- |
| steel–steel (all steel variants)      | Steel on steel, clean and dry: static 0.5–0.8, kinetic 0.42 → μ 0.4 (range 0.3–0.6)                 |
| steel–concrete                        | Rolled steel plate on concrete, static 0.57–0.70 (ASCE 1985) → μ 0.55 (0.45–0.7)                    |
| rubber–concrete                       | Rubber on dry concrete 0.6–0.85 → μ 0.7                                                             |
| rubber–rubber                         | 1.16 in one table → μ 1.0 (0.8–1.2)                                                                 |
| polyethylene–steel, –polyethylene     | 0.2 clean and dry → μ 0.2                                                                           |
| cardboard–rubber                      | Rubber on cardboard 0.5–0.8 → μ 0.6                                                                 |
| cardboard–steel, –plastic             | Paper on cast iron 0.2; packaging board on steel commonly 0.2–0.4 → μ 0.3                           |
| cardboard–wood, –cardboard, –concrete | No direct table value; between paper on metal and rubber on cardboard → 0.35, 0.4, 0.5              |
| polyurethane pairs                    | No table value found; placed between nylon (0.4) and rubber, as in the existing wood/plastic rows   |
| Restitution                           | 0 on rollers and chains (existing rule); 0.05 for cardboard; 0.2–0.3 for steel; 0.35–0.5 for rubber |

Sources: [Engineering ToolBox, friction coefficients](https://engineeringtoolbox.com/friction-coefficients-d_778.html); [Friction Coefficient of Steel on Concrete or Grout, J. Struct. Eng. 1985](https://ascelibrary.com/doi/10.1061/%28ASCE%290733-9445%281985%29111%3A3%28505%29). The TAPPI T 815 method for packaging board was found but its typical values are behind a paywall; if the owner measures cardboard on rollers, the measured value replaces 0.3.

## Consequences

- Any catalogue model can be placed and simulated; new models need only `node scripts/build-catalog.mjs`.
- Collision is coarser than the drawing where a model declares few pieces (each piece is filled).
- The source can create objects that are not pallets; sizes along travel come from the models in both modes, so 2D and 3D stay comparable.
- Live objects do not appear in the project file or in 2D results.

## Alternatives considered

- **Bounding boxes only.** Rejected by the owner: racks and fences would be solid blocks.
- **A separate `layout.scenery[]` list.** More code (own commands, picking, properties) for the same result.
- **Resetting the run on every drop.** Rejected by the owner: dropping into a running simulation is part of the request.
