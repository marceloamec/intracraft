---
title: ADR-0015 MTBH Rigid Body from Pallet and Load Models
lang: en
tags: [decision, physics, pallet]
status: accepted
date: 2026-10-30
---

# ADR-0015 MTBH Rigid Body from Pallet and Load Models

- **Status:** Accepted (implemented 2026-10-30, `packages/sim-physics/src/mtbh.ts`)
- **Date:** 2026-10-30

## Context

In the 3D mode a pallet and its load move as one **MTBH** (material to be handled). A single body needs four things:

- a contact shape that touches rollers, chains and stops where a real pallet does;
- a mass and a centre of mass;
- moments of inertia, for tipping and turning;
- a drawing.

The asset catalogue ([[A16 Asset Catalog]]) has real 3D models of a wooden EUR pallet, a plastic pallet and a EUR pallet loaded with cartons. Some models declare their mass, centre of mass and inertia (glTF `KHR_physics_rigid_bodies`), which the catalogue copies into its `physics` field. A model mesh is too detailed to collide with dozens of rollers at 120 steps per second.

## Decision

1. **One rigid body per MTBH.** The pallet and the load never slide against each other. Load shift is outside the MVP.
2. **Contact shape from the pallet type, not from the mesh.**
   - The occupancy bands of [[S10 Pallet Types]] (boards and blocks per height layer) become one box per solid cell.
   - The boxes are scaled to the model's real size.
   - With a EUR pallet, rollers touch the three bottom boards and chains touch the blocks, as on the real pallet.
3. **Frame.**
   - X runs along the pallet length (the travel direction on rollers), Y up, Z across.
   - The origin is at the centre of the footprint, on the underside.
   - A model whose long side runs along Z is turned a quarter turn; only quarter turns are used, so the values stay exact.
4. **Pallet mass properties** come from the model's declared mass, centre of mass and inertia.
5. **Load.**
   - The load is a **uniform block** from the pallet top to the loaded model's top, with the user's `loadMass`.
   - The loaded models only declare the mass of empty packaging, so their own value is not used.
   - An optional `centerOffset` moves the load's centre of mass, for a load that is heavier on one side.
6. **Combination.**
   - Total mass, centre of mass and principal moments come from the parallel-axis theorem (`combineMassProperties`).
   - **Inertia products are neglected:** an off-centre load only shifts the centre of mass. The error is small for the offsets tested (≤ 0.1 m) and stays inside the P6 tolerance.
7. **Contact material.** The MTBH takes the pallet's material (`wood` or `plastic`, [[S16 Contact Materials]]).
8. **Drawing.** The editor draws the model at the `placement` returned by `buildMtbh`, so the drawing and the body stay together.

## Alternatives considered

- **Convex hull or mesh of the model as the shape.**
  - Rejected: the hull fills the gaps between the bottom boards, so rollers would carry the pallet in the wrong places.
  - A mesh is too slow for many contacts.
- **Two bodies joined by a fixed constraint** (pallet + load). Rejected: same behaviour as one body, but slower and less stable.
- **The loaded model's declared mass.** Rejected: it is the mass of empty cartons, not of the goods.

## Consequences

- Tests in `mtbh.test.ts`:
  - layers and boxes of the EUR shape;
  - quarter-turn placement;
  - combined mass and centre of mass;
  - an off-centre load.
- Bench scenarios ([[A10 CLI and Scenarios]]):
  - P1 and P4 use a load mass that makes the MTBH weigh 1000 kg and 1025 kg;
  - P6 checks that a load shifted 100 mm forward makes the pallet tip 100 × 600/631.4 mm earlier at a conveyor end.
- A pallet type without rigid-body data in the catalogue cannot be used in 3D until it gets one.
