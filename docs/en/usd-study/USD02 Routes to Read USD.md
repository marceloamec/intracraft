---
title: USD02 Routes to Read USD
lang: en
tags: [study, usd, architecture]
updated: 2026-11-09
---

# USD02 Routes to Read USD

Part of [[USD00 Study Overview]]. Three ways to bring USD models into IntraCraft, the test done, and what IntraCraft lacks
today.

## 1. What USD is

OpenUSD is an open scene format with a public specification (AOUSD Core Specification 1.0). A `.usd` file is either text
(`.usda`) or binary (`.usdc`); `.usdz` is a zip of both with textures. Unlike glTF, one model is usually **several files
that reference each other**: the conveyor of [[USD01 Isaac Sim Asset Library]] pulls in a physics-material file and six
look-material files. Physics data follows the public `UsdPhysics` schema (collision, rigid body, mass, material, joints);
NVIDIA adds its own `Physx…` properties on top.

Reading a public format from its specification is within the clean-room rule; it is what IntraCraft already does for IFC
and DXF.

## 2. The three routes

| Route                              | How                                                                                                                           |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **A. Convert outside**             | A script turns USD into what the catalogue already uses: a GLB model plus collision pieces in JSON. IntraCraft never sees USD |
| **B. Read USD in the application** | IntraCraft opens `.usd` files itself with a Rust library and turns them into its own families                                 |
| **C. Hand the layout over**        | IntraCraft writes its layout as a USD scene, and the heavy simulator plays it                                                 |

### Route A: convert outside

- The converter would be a Python script using Pixar's USD library. The simulator's own environment on the laptop already
  has it (it is what this study used to inspect the model).
- Fits [[ADR-0005 3D Asset Sources]] and [[A16 Asset Catalog]] exactly: manifest with hashes and licence, files out of git.
- **For:** no new library inside the product; the conversion can also reduce faces and textures.
- **Against:** only people with Python and the script can add a model; a user cannot import their own USD from the
  application; two formats and one more tool to maintain.

### Route B: read USD in the application (recommended)

- Library: **`openusd`** 0.8.0 (crates.io, MIT licence, pure Rust, no C++). Reads and writes `.usda`, `.usdc`, `.usdz`;
  composes references, payloads, variants and instancing; a companion crate gives typed views for geometry, materials and
  physics. It passes the specification's test suites for both file formats and for composition.
- **For:** one application, no outside tool; "Import USD" becomes a command like "Import IFC"; the same reader serves the
  library, the CAD workflow's output ([[USD04 CAD to USD Workflow]]) and any USD a supplier provides; it can also *write*
  USD, which opens route C later.
- **Against:** the library is young (before 1.0, its interface may change between versions); it is a new dependency to
  vendor (about 6 MB of source plus some forty small supporting crates, many already vendored) and its licence, with the
  bundled schema files' own licence, needs the usual review; NVIDIA's MDL look-materials cannot be evaluated, only their
  colour and texture inputs read (§3).

### Route C: hand the layout over

- IntraCraft would stay the place where the plant is designed and simulated fast (discrete module), and export the layout
  so the other simulator renders and simulates it in full detail.
- **For:** the realism of that simulator with no effort on IntraCraft's renderer.
- **Against:** needs that simulator installed and an RTX graphics card; the conveyors' behaviour (speeds, stops, sensors,
  control rules, IntraScript) would have to be rebuilt there; two physics results to reconcile. It contradicts the idea of
  one layout with two modes in one tool. Not recommended now; it stays possible once route B can write USD.

## 3. The test done (throwaway, outside the repository)

A 30-line Rust program with `openusd` 0.8.0 opened `ConveyorBelt_A06.usd` and walked the scene.

| Check                                | NVIDIA's USD tools | `openusd` in Rust |
| ------------------------------------ | ------------------ | ----------------- |
| Meshes                               | 8                  | 8                 |
| Points in all meshes                 | 105,392            | 105,392           |
| Faces of the visual body             | 112,778            | 112,778           |
| Collision pieces and their marks     | 5 found            | 5 found           |
| Friction of belt and roller material | 0.9 and 0.4        | 0.9 and 0.4       |
| Time to open and read                | not measured       | 0.01 s            |

Two things learned:

- The model references a folder named `Material Library` written as `Material%20Library`. Neither reader finds it on disk
  unless the folder is named with the literal `%20`. An importer must decode such names.
- The reader compiled first time on the laptop's Rust 1.99 (the crate asks for 1.96 or newer).

Not tested yet: reading normals, texture coordinates and the look-material inputs; models with instancing or variants;
the larger files (117 MB); the typed physics views of the companion crate.

## 4. What IntraCraft lacks today, whatever the route

This is the larger part of the work.

| Area              | Today in the Rust platform                                                                  | Needed                                                          |
| ----------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| Scenery in Design | A scenery object is a 500 mm box (`sim/model/src/forms.rs`)                                 | A family form that holds a mesh                                 |
| 3D views          | Draw IntraCraft's own shapes; the Bevy window can load glTF                                 | Draw an imported mesh with its colours and textures             |
| Physics shapes    | Box, rounded box, cylinder (`Shape` in `sim/physics/src/bench.rs`)                          | A convex piece and a fixed triangle mesh                        |
| Catalogue in Rust | The TypeScript catalogue files are reused; its GLB models are refused (meshopt compression) | A catalogue entry that points to a mesh the Rust readers accept |
| Photocells        | A beam is tested against the three shapes                                                   | The same test against the new shapes                            |

The physics engine underneath already has what is needed: Avian 0.8 (and Rapier, the reference) offer triangle-mesh,
convex-hull and convex-decomposition colliders. Only IntraCraft's own contract (`PhysicsBench`) does not expose them yet.
Extending that contract is an architecture change and gets its own decision record.

## 5. Look: what survives outside the heavy renderer

| In the model                                | In IntraCraft's 3D window                                        |
| ------------------------------------------- | ---------------------------------------------------------------- |
| Fine geometry (bolts, cable ducts)          | Kept, or reduced at import                                       |
| Colour, roughness, metal textures           | Kept when read from the material's inputs; Bevy draws them (PBR) |
| MDL shader effects (decals, layered)        | Lost or approximated by a plain colour                           |
| Ray-traced light, reflections, soft shadows | Not available; Bevy gives shadow maps and environment light      |
| Moving belt texture                         | Possible later (scrolling the texture with the surface speed)    |

## Sources

- <https://github.com/mxpv/openusd> and <https://crates.io/crates/openusd>
- <https://github.com/openusd-rs/bevy_openusd> (a Bevy USD editor for Bevy 0.19; looked at, not needed)
- <https://lib.rs/crates/openusd-rs> (another, earlier Rust reader; not chosen)
- <https://aousd.org/blog/foundations-of-open-3d-development-introducing-aousd-core-specification-1-0/>
