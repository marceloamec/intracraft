---
title: S17 Scenery Asset
lang: en
tags: [spec, scenery, physics, assets]
library-id: scenery.asset
status: spec
updated: 2026-10-31
---

# S17 Scenery Asset (`scenery.asset`)

A model from the asset catalogue ([[A16 Asset Catalog]]) placed in the layout: a box, a crate, a drum, a rack, a forklift, a fence. It has **no ports and no flow behaviour**; it takes part in the **3D physics run** only. It can be dropped from the air, left resting in place, or fixed to the floor. Decision: [[ADR-0017 Scenery Assets in the Layout and Live Spawn]].

Values are **indicative**. Mass, centre of mass and inertia come from the model's own rigid-body data (SimReady, CC BY 4.0); they describe the object as modelled (an empty cardboard box weighs about 1 kg), not a real load.

## 1. Parameters

| Key          | Unit | Default | Range                   | Description                                                                            |
| ------------ | ---- | ------- | ----------------------- | -------------------------------------------------------------------------------------- |
| `asset`      | ref  | —       | catalogue ids           | Model from the catalogue; the list shows its name, not the id                          |
| `placement`  | enum | `rest`  | `drop`, `rest`, `fixed` | How the object enters the physics run (§2)                                             |
| `dropHeight` | m    | 1.0     | 0.05 … 10               | Only `drop`: gap between the object's base and its support at the start                |
| `material`   | enum | `auto`  | `auto` or an S16 id     | Contact material; `auto` takes the catalogue's material ([[S16 Contact Materials]] §1) |

The instance's position is the point **on the support** under the object (floor y = 0, or the top of the object below it when placed in the editor); `yaw` turns it about the vertical axis. Pitch and roll are not set: objects start upright.

## 2. Placement modes (3D)

| Mode    | Body                                | Start pose                          | Typical use                               |
| ------- | ----------------------------------- | ----------------------------------- | ----------------------------------------- |
| `drop`  | Dynamic                             | Base `dropHeight` above the support | Falling load, drop tests, impact energies |
| `rest`  | Dynamic                             | Base 1 mm above the support         | Loose goods that can be pushed or knocked |
| `fixed` | Static (never moves, needs no mass) | Base on the support                 | Racks, fences, gates, ramps, machines     |

- **Collision shape:** the convex pieces the model declares (`KHR_physics_rigid_bodies` colliders), one convex hull per piece, at most 64 corners each. Hollow objects (racks, fences) keep their openings between pieces; a piece itself is filled.
- **Mass properties:** mass, centre of mass and diagonal inertia from the catalogue.
- **Rest:** dynamic scenery bodies may **sleep** when still, to save computing time; contact wakes them.
- **Impacts** of a pallet against a scenery object appear in the impact log with the object's name.
- **Not a flow item.** A scenery object on a moving conveyor is carried by friction, but the conveyor logic, the photocells (which detect pallets only) and the sinks do not see it, and it is not counted in any statistic.

## 3. 2D behaviour

None. The 2D (DES) model ignores scenery objects without a warning; they are drawn in the top view for orientation only.

## 4. Live spawn

While a 3D run is going, an object dropped from the catalogue into the scene enters the running world at the next fixed step with the chosen placement. It is **not** added to the layout, is not saved, cannot be undone and disappears when the run is stopped or reset. It is a typed simulation command (`sim.spawnObject`, actor-logged), so the same commands at the same steps give the same result.

## 5. Validations

| Rule               | When                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------ |
| `unknownAsset`     | `asset` is not in the catalogue (rejected by the command)                                              |
| `assetFileMissing` | The model file is not downloaded (`pnpm assets:fetch`); the editor shows a warning, physics still runs |

## 6. Reference tests

- `drop` from 1 m: first contact with the floor at √(2h/g) = 0.452 s (± one step of 1/120 s); after settling, base at y = 0 ± 5 mm.
- `rest`: moves less than 1 mm in 2 s.
- `fixed`: pose unchanged after any impact.
- A box dropped above a fixed rack stays on the shelf it falls on.
- Same layout and seed → same state hash.

## Related

[[A16 Asset Catalog]] · [[S16 Contact Materials]] · [[S09 Source and Sink]] · [[ADR-0016 Physics World Built from the Layout]]
