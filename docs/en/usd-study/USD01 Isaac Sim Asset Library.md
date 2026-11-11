---
title: USD01 Isaac Sim Asset Library
lang: en
tags: [study, usd, assets, license]
updated: 2026-11-09
---

# USD01 Isaac Sim Asset Library

Part of [[USD00 Study Overview]]. What the library holds and what can be reused.

## 1. Where the models are

| Place                                      | Content                                                                                                     |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| `D:\IsaacSIM\IsaacSim` (local)             | The simulator's source, version 6.2.0-rc.14. Its 1,864 USD files are test and schema files, not the library |
| NVIDIA's asset server, `Assets/Isaac/6.2/` | The library itself. Two roots: `Isaac/` and `NVIDIA/`                                                       |
| A local asset pack (optional download)     | The same library on disk, selected with the `ISAACSIM_ASSET_ROOT` setting                                   |

**The giant library is not on the laptop.** The simulator streams each model from the server when a scene opens. To reuse
models in IntraCraft they must be fetched one by one (as `pnpm assets:fetch` does today for the GLB catalogue) or taken from
a downloaded asset pack.

Folders of interest for intralogistics:

| Folder                                       | Content                                              |
| -------------------------------------------- | ---------------------------------------------------- |
| `Isaac/Props/Conveyors`                      | 47 conveyor pieces, materials and textures (3.1 GB)  |
| `Isaac/Props/Forklift`                       | Forklift                                             |
| `Isaac/Props/Pallet`                         | `pallet.usd`, `o3dyn_pallet.usd`, two pallet holders |
| `Isaac/Props/KLT_Bin`, `Isaac/Props/Dolly`   | Small load carriers                                  |
| `Isaac/Environments/Simple_Warehouse`        | Warehouse building with racks                        |
| `Isaac/Environments/Digital_Twin_Warehouse`  | Larger warehouse                                     |
| `NVIDIA/Assets/DigitalTwin/Assets/Warehouse` | Equipment, shelving, storage, shipping, safety       |
| `Isaac/SimReady`                             | Models with full physics data (mass, materials)      |

## 2. The conveyor folder

47 pieces, `ConveyorBelt_A01` to `A49` (two numbers unused), 13 to 117 MB each. The simulator's own track-builder table
classifies them:

| Property  | Values (number of pieces)                                |
| --------- | -------------------------------------------------------- |
| Style     | Belt 19, roller 17, dual (both) 11                       |
| Type      | Straight 32, start 3, Y merge 5, T merge 5, fork merge 2 |
| Ramp      | Flat 37, one level 4, two levels 4, four levels 2        |
| Curvature | None 32, small 9, medium 3, large 3                      |

They are conveyors for **cartons and totes**: about 1.15 m wide overall, fixed length (2 m for the piece measured). There is
no pallet roller conveyor, chain conveyor, turntable or transfer table. [[ADR-0005 3D Asset Sources]] had already noted
this for the GLB conversion of the same family of models.

## 3. What is inside one conveyor

`ConveyorBelt_A06.usd` (13 MB, the smallest) was opened with the USD tools that ship with the simulator.

| Item             | Finding                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------- |
| Units and axes   | Metres, **Z up** (IntraCraft's Design module is also Z up; its engines are Y up)          |
| Visual body      | One mesh, 112,778 faces and 104,440 points, in six material groups (body, bolts, caps, …) |
| Belt             | A separate mesh of 438 faces, marked as a collision surface                               |
| Collision pieces | Five simple meshes: two side walls (146 faces each), one floor (162), two feet (6 each)   |
| Friction         | Two materials in a shared file: belt 0.9, roller 0.4, no bounce                           |
| Connection point | An `Anchorpoint` 2.0 m along x: where the next piece attaches (like an IntraCraft port)   |
| Materials (look) | References to six material files that point to PNG textures and NVIDIA's MDL shaders      |
| Mass, rigid body | None. The piece is a fixed obstacle                                                       |
| Movement         | **None in the file**                                                                      |

So "physics already prepared" means three things: simplified collision pieces drawn by hand apart from the visual mesh,
friction values, and a connection point. That is real value: making good collision pieces is the slow part of preparing a
model.

It does **not** include the movement. In that simulator a conveyor moves because a control node, added when the scene is
built, gives the belt body a surface velocity; the direction and whether it is a curve come from the track-builder table,
not from the model. IntraCraft's physics module already has the same idea (`surface` and `set_surface_velocity` in the
physics contract, used by the chain conveyor), so an imported belt could be driven by an IntraCraft parameter. See
[[USD03 Physics Mapping]].

## 4. Licences

Checked on the server on 2026-11-09 by looking for a `LICENSE` file in each folder.

| Folder                                      | Licence file       | What it allows                                         |
| ------------------------------------------- | ------------------ | ------------------------------------------------------ |
| `Isaac/Props/Conveyors` (and its materials) | CC BY 4.0          | Use, convert, modify, redistribute; attribution needed |
| `Isaac/Props/Forklift`                      | CC BY 4.0          | Same                                                   |
| `Isaac/Environments/Simple_Warehouse`       | CC BY 4.0          | Same                                                   |
| `Isaac/Environments/Digital_Twin_Warehouse` | Has a licence file | Not read; check before use                             |
| `Isaac/Props/Pallet`, `Isaac/Props/KLT_Bin` | None               | NVIDIA's general licence applies                       |
| `Isaac/SimReady`, `Isaac/Robots`            | None               | Same                                                   |
| `NVIDIA/Assets/DigitalTwin`                 | None               | Same                                                   |

NVIDIA's general licence for the simulator's additional software and materials (linked from the repository's `LICENSE`)
allows use only with that simulator on NVIDIA graphics cards, and forbids distributing, sublicensing and making derivative
works. Converting such a model for IntraCraft, or shipping it, is therefore not allowed.

Consequences for IntraCraft:

- Only folders with their own open licence file may enter the catalogue, each with its attribution, as today.
- The licence must be checked **per folder at fetch time** and recorded per model, never assumed for the whole library.
- Product and interface texts must not use the simulator's or its maker's names; they appear only in the attribution
  (clean-room rule in `AGENTS.md`).
- This is a reading of public licence texts, not legal advice.

## 5. Weight of the models

| Measure                          | Value                                     |
| -------------------------------- | ----------------------------------------- |
| One conveyor piece               | 13 to 117 MB, about 100,000 faces or more |
| Whole conveyor folder            | 3.1 GB (47 models, 227 textures)          |
| Today's IntraCraft GLB catalogue | 40 models, 46 MB in total                 |

The models are built for close-up ray-traced images. For a plant with hundreds of objects IntraCraft should store a
lighter copy (fewer faces, smaller textures) made once at import. The collision pieces are already light (a few hundred
faces) and need no reduction.
