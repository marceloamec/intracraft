---
title: USD04 CAD to USD Workflow
lang: en
tags: [study, usd, cad, assets]
updated: 2026-11-09
---

# USD04 CAD to USD Workflow

Part of [[USD00 Study Overview]]. What the CAD-to-USD skill in `D:\IsaacSIM\IsaacSim\skills\omniverse-cad-to-simready` does,
what it needs, and how its result would reach IntraCraft. Read from the skill's own description files (Apache-2.0); it was
not run.

## 1. What the skill is

It is not a converter but a **set of instructions for an AI coding agent** that runs several separate tools in order and
checks each result. "SimReady" is NVIDIA's name for a USD model that passes a checklist: metres, a clean structure, collision
shapes, mass, physics materials.

| Stage                   | What happens                                                                            | Needs                                                        |
| ----------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| 1. Preflight            | Installs and checks the tools                                                           | Python 3.12, `uv`                                            |
| 2. Identify the asset   | Looks up what the object is, to guess materials later                                   | Web search                                                   |
| 3. Convert to USD       | CAD or mesh file to a USD model                                                         | NVIDIA's `usd-convert-cad` package and its converter licence |
| 4. Minimum validation   | Is the USD usable at all; are the units metres                                          | —                                                            |
| 5. Material assignment  | An AI service looks at renders and assigns look-materials                               | "Content Agents" services (below)                            |
| 6. Physics assignment   | An AI service adds rigid bodies, collision shapes, mass, density, friction, restitution | Same                                                         |
| 7. Conform and validate | Repairs and checks against the SimReady profile (geometry, physics)                     | NVIDIA validation tools                                      |
| 8. Preview and package  | Renders thumbnails, builds a deliverable folder                                         | A render service on an NVIDIA GPU                            |

Accepted sources: URDF and MuJoCo robot files, mesh and scene files (`.fbx`, `.obj`, `.gltf`, `.glb`, `.dae`, `.stl`),
Gaussian splats, existing USD, and the CAD formats listed by the upstream converter (that list was not read in this study).

## 2. What it needs

| Requirement                         | On the responsible's laptop                                                       |
| ----------------------------------- | --------------------------------------------------------------------------------- |
| Linux or macOS                      | **No**: Windows 11. Would need WSL2 or a Linux machine                            |
| Docker and NVIDIA Container Toolkit | Not checked                                                                       |
| NVIDIA graphics card                | Present (NVIDIA software is installed); model not checked                         |
| A model-provider API key            | Needed for stages 5 and 6 (NVIDIA, OpenAI, Anthropic or Google); **paid per use** |
| Converter licence for CAD formats   | Named in the skill as a condition; terms not checked                              |
| Network access                      | First CAD run downloads components from NVIDIA's registry                         |

The skill allows a **conversion-only** run (stages 1, 3 and 4): no AI services, no API key, no GPU render. That is the
cheap way to try it.

## 3. What it would give IntraCraft

A USD file with the same kind of content as the library models of [[USD01 Isaac Sim Asset Library]]: a visual mesh,
collision shapes, mass and physics materials. So **the same importer serves both** (route B in
[[USD02 Routes to Read USD]]): a real machine, rack or load unit drawn in CAD could become an IntraCraft scenery or load
family with believable collisions.

Limits to keep in mind:

- **Collision shapes and mass from stage 6 are an AI's estimate**, not measured data. For a load whose behaviour matters
  (a pallet with an offset centre of mass), the values must be checked or entered by hand, and never presented as
  manufacturer data (`AGENTS.md`, "La librería").
- **CAD models of real equipment belong to their makers.** The clean-room rule forbids bringing in manufacturer models or
  catalogues without a licence. The workflow is for the user's own CAD or CAD they are licensed to use; such models stay in
  the user's project and are not shipped with IntraCraft.
- **A CAD model is not a parametric component.** It has one size. It suits scenery, loads and one-off machines; it does
  not replace a library conveyor whose length and roller pitch are parameters.
- **Triangle counts from CAD are high.** The same reduction step as for library models applies at import.

## 4. Two ways to get collision pieces without the AI services

If only stage 3 is used, the USD has a mesh and no physics. IntraCraft could then:

1. compute convex pieces itself at import (the physics engine's convex decomposition, run once, stored with the family), or
2. let the user draw simple boxes and cylinders over the model in the Design module's family editor.

Option 2 gives the best physics for conveyor work (flat, exact contact faces) and needs no new algorithm; option 1 is
automatic but approximate. Both are deterministic once stored.

## 5. Suggested first trial (not done)

1. Pick one simple own or openly licensed CAD or mesh file (for example a steel rack frame).
2. Run the skill in conversion-only mode under WSL2.
3. Open the result with the importer spike (U1 in [[USD05 Phased Plan Risks and Decisions]]).
4. Only then decide whether the paid material and physics stages are worth it.
