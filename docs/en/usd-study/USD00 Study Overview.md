---
title: USD00 Study Overview
lang: en
tags: [study, usd, assets, physics]
status: study, no decision taken
updated: 2026-11-09
---

# USD00 Study Overview

Can IntraCraft use models in **USD** (OpenUSD, the scene format of NVIDIA's robotics simulator) inside its simulation, and
reuse the asset library and the CAD-to-USD workflow that come with that simulator? This folder is the study. Nothing in the
product was changed; every number below was measured on 2026-11-09 on the responsible's laptop.

| Note                                      | What it answers                                                         |
| ----------------------------------------- | ----------------------------------------------------------------------- |
| [[USD01 Isaac Sim Asset Library]]         | What the library holds, what "physics already prepared" means, licences |
| [[USD02 Routes to Read USD]]              | Three ways to bring USD into IntraCraft, and the test that one works    |
| [[USD03 Physics Mapping]]                 | How a USD model's physics maps onto IntraCraft's physics module         |
| [[USD04 CAD to USD Workflow]]             | What the CAD-to-SimReady skill does and what it needs                   |
| [[USD05 Phased Plan Risks and Decisions]] | Steps, risks, and the decisions that are the responsible's to take      |

## Answer in short

**Yes, it is possible, and the first step is small.** A throwaway test read a real conveyor of the library with a pure-Rust
USD reader and got exactly the same content as NVIDIA's own USD tools: 8 meshes, 105,392 points, the collision shapes and
the two friction materials.

What the study found, in order of importance:

1. **The models bring shape, not behaviour.** A library conveyor holds a detailed visual mesh, a few simple collision
   meshes (floor, side walls, feet) and friction values. It holds **no movement**: the belt speed and direction are added
   by the simulator at run time. So IntraCraft keeps deciding how a conveyor moves a load; USD decides how it looks and
   what a load can bump into. See [[USD01 Isaac Sim Asset Library]] §3.
2. **The library's conveyors are for cartons and totes, not pallets.** 47 pieces: 19 belt, 17 roller, 11 mixed; straight
   pieces, curves, ramps and merges. None is a pallet roller or chain conveyor, and each has a fixed size. IntraCraft's five
   pallet conveyors are parametric (length, width, roller pitch) and must stay so. The library is immediately useful as
   **scenery and loads** (forklifts, warehouse building, racks, bins), and later for carton lines if the scope grows.
3. **The licence differs folder by folder.** `Props/Conveyors`, `Props/Forklift` and some environments carry a
   **CC BY 4.0** licence file: reuse is allowed with attribution, the same terms as the catalogue IntraCraft already uses
   ([[ADR-0005 3D Asset Sources]]). Other folders (`Props/Pallet`, `Props/KLT_Bin`, `SimReady`, `Robots`, `DigitalTwin`) have
   no licence file and fall under NVIDIA's general licence, which forbids distributing and modifying. Those must not be
   bundled. See [[USD01 Isaac Sim Asset Library]] §4.
4. **IntraCraft's Rust platform cannot show or collide with any mesh model yet.** A scenery object is drawn as a
   500 mm box, and the physics module knows three shapes: box, rounded box, cylinder. This gap, not the USD format, is most
   of the work, and it has to be closed whatever the file format. See [[USD02 Routes to Read USD]] §4.
5. **The realism the responsible likes comes mostly from the renderer**, which is also what makes that simulator heavy
   (ray tracing on an RTX graphics card). The models carry part of it (fine geometry and material textures), and
   IntraCraft's 3D window can show that part at a fraction of the cost. It will look clearly better than today's
   box-and-cylinder models, but not the same as a ray-traced image.

## Recommendation

- **Read USD natively in Rust** (route B in [[USD02 Routes to Read USD]]), with the `openusd` crate (MIT licence, no C++),
  as an **import**: a USD model becomes a scenery or load family with a mesh to draw and collision pieces for the physics
  module.
- **Keep the parametric pallet conveyors** as they are. Their movement, sensors and the ±5 mm stop criterion do not depend
  on imported models.
- **Start with a spike** (U1 in [[USD05 Phased Plan Risks and Decisions]]): one CC BY 4.0 forklift and one conveyor shown
  as scenery in the 3D window, with pallets colliding against them in the physics module.
- **Treat the CAD workflow as a separate, later step.** It needs Linux, Docker and an NVIDIA graphics card, and its
  automatic material and physics assignment uses paid AI services ([[USD04 CAD to USD Workflow]]).

## Decisions needed before any code

Listed with alternatives in [[USD05 Phased Plan Risks and Decisions]] §3:

1. Route A (convert outside, as today's catalogue) or route B (read USD in the application).
2. Whether models are bundled with IntraCraft or fetched/imported by each user.
3. Whether reading the open-source code of that simulator is allowed under the clean-room rule. This study read only its
   documentation, its skill descriptions and data files, never its engine code.
