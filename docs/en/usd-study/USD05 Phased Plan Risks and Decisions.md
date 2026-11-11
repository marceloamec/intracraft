---
title: USD05 Phased Plan Risks and Decisions
lang: en
tags: [study, usd, plan]
updated: 2026-11-09
---

# USD05 Phased Plan Risks and Decisions

Part of [[USD00 Study Overview]]. A proposal, not an accepted plan: nothing here is in [[A19 Rust Implementation Plan]] or
[[A12 Roadmap]] yet.

## 1. Steps (assuming route B of [[USD02 Routes to Read USD]])

Each step ends with something the responsible can see and with tests.

| Step | Result in simulation terms                                                                                                                      | Main work                                                                                                        |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| U1   | **Spike.** One forklift and one conveyor piece (both CC BY 4.0) stand in a layout as scenery, visible in the Bevy 3D window                     | Reader in `spikes/`, mesh to the viewer, no product change                                                       |
| U2   | Pallets in the Physics module collide with imported scenery; a photocell sees it                                                                | Two new shapes in the physics contract (convex piece, fixed triangle mesh), in Avian and in the Rapier reference |
| U3   | **Import USD** command in the Design module: a USD file becomes a scenery family (mesh, collision pieces, licence, attribution), stored lighter | Command, family mesh form, reduction of faces and textures, import report                                        |
| U4   | Catalogue entries for the openly licensed folders, fetched and verified like today's models                                                     | Manifest with hash and licence per model; per-folder licence check                                               |
| U5   | Imported loads: a source can create a pallet carrying an imported load with its own mass and convex pieces                                      | Convex pieces at import, mass, material matching ([[USD03 Physics Mapping]] §3)                                  |
| U6   | CAD trial ([[USD04 CAD to USD Workflow]] §5)                                                                                                    | Outside IntraCraft, then the same importer                                                                       |
| U7   | *(new scope, only if decided)* Imported carton conveyors that move loads, with a discrete model, ports and sensors                              | Specifications first; outside the MVP today                                                                      |
| U8   | *(optional)* Export the layout as USD for the heavy simulator (route C)                                                                         | Writer; behaviour is not exported                                                                                |

U1 and U2 answer the open technical questions. U3 to U5 are the useful product. U7 and U8 are separate decisions.

Acceptance checks to write before the code (test first):

- An imported model's size in the layout equals the size in the USD file within 1 mm (units and Z-up to Y-up conversion).
- The same layout and seed give the same physics result with imported scenery, twice and on a second machine.
- A pallet pushed against an imported fixed wall stops with its front within ±5 mm of the wall's collision face.
- A photocell beam crossing an imported piece is blocked exactly where the piece's collision shape is.
- A model from a folder without an open licence file is refused by the catalogue fetch.
- A hostile or broken USD file gives an error message, never a crash (platform rule "never crash").

## 2. Risks

| Risk                                                                 | Effect                                                 | Mitigation                                                                                          |
| -------------------------------------------------------------------- | ------------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| The Rust USD reader is young (version 0.8)                           | A file that does not open; interface changes on update | Pin the version (it is vendored anyway); keep route A as fallback; report unreadable files upstream |
| Models are heavy (100,000 faces, large textures)                     | Slow 3D view with many objects, large projects         | Reduce once at import; instancing for repeated models                                               |
| NVIDIA's look-materials (MDL) cannot be evaluated                    | Models look plainer than in the heavy simulator        | Read colour and texture inputs; accept the difference (USD02 §5)                                    |
| Licence differs per folder and may change with a new library version | Shipping something not allowed                         | Check and record the licence per model at fetch; pin the library version (6.2)                      |
| Mesh collisions catch pallets on small details                       | Wrong physics results                                  | Only use the model's collision pieces or drawn primitives, never the visual mesh                    |
| Friction rules differ (per material against pair table)              | Results change silently                                | Match to S16 materials by hand at import (USD03 §3)                                                 |
| Scope creep toward a second, heavier simulator                       | The MVP slips                                          | U7 and U8 need their own decision; pallet conveyors stay parametric                                 |
| The server layout or addresses change                                | Fetch fails                                            | Hashes in the manifest; support a local asset pack as source                                        |

## 3. Decisions for the responsible

**D1. Where USD is read.**

- *Option 1, read in the application (route B, recommended).* IntraCraft gets an Import USD command; users can bring their
  own USD and CAD results. Cost: one new vendored library that is still young.
- *Option 2, convert outside (route A).* IntraCraft stays without USD; a script prepares models for the catalogue. Cost:
  only whoever runs the script can add models; users cannot import USD themselves.

**D2. How models reach users.**

- *Option 1, fetched on demand (as today).* The repository holds only the manifest; models download and are verified on
  the user's machine. Small repository; needs network once.
- *Option 2, bundled.* A reduced set of CC BY 4.0 models ships with the application, with attribution. Works offline;
  the installer grows and every bundled model's licence must be right.

**D3. Reading the heavy simulator's source code.** Its code is open (Apache-2.0), but `AGENTS.md` forbids reading the code
of third-party simulation software. This study stayed with documentation, skill descriptions, data tables and model files.

- *Option 1, keep the rule as it is (recommended).* Everything needed so far came from public documentation and from the
  models themselves.
- *Option 2, allow reading openly licensed code for facts about file content,* recorded case by case in a decision
  record, as was done for the Revit reader's fence in BIMCraft.

**D4. Carton conveyors (step U7).** Stay outside the MVP as `AGENTS.md` says, or open a scope extension like
[[ADR-0012 Scope Extension Lifts Elevator and Gravity Line]].

Once D1 to D3 are answered, the outcome is written as a decision record in `docs/en/decisions/` and step U1 can start.

## 4. What was and was not done in this study

| Done                                                                                                                     | Not done                                                        |
| ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| Listed the conveyor folder and the library's top folders on the asset server                                             | Downloading the library or an asset pack                        |
| Downloaded one conveyor (13 MB), its physics-material file and its licence, to a temporary folder outside the repository | Adding any model or dependency to the repository                |
| Inspected that model with the USD tools shipped with the simulator                                                       | Running the simulator                                           |
| Read the same model with a throwaway Rust program and compared the counts                                                | Reading normals, texture coordinates and look-materials in Rust |
| Read the CAD skill's description files                                                                                   | Running the CAD workflow                                        |
| Read IntraCraft's physics contract and scenery form                                                                      | Any change to IntraCraft's code, tests or expected results      |
