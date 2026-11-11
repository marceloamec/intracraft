---
title: USD03 Physics Mapping
lang: en
tags: [study, usd, physics]
updated: 2026-11-09
---

# USD03 Physics Mapping

Part of [[USD00 Study Overview]]. How the physics data of a USD model would map onto IntraCraft's Physics simulation
module ([[ADR-0026 Bevy and Avian for the Physics Simulation Module]], contract `PhysicsBench`).

## 1. Mapping table

| In the USD model (public `UsdPhysics` schema)              | In IntraCraft                                                               | State today             |
| ---------------------------------------------------------- | --------------------------------------------------------------------------- | ----------------------- |
| Mesh marked as collision, approximation `none`             | Fixed triangle-mesh part of a `Fixed` body                                  | Shape missing           |
| Mesh with approximation `convexHull`                       | One convex part                                                             | Shape missing           |
| Mesh with approximation `convexDecomposition`              | Several convex parts, computed once at import and stored                    | Shape missing           |
| Mesh with approximation `boundingCube` or a `Cube` prim    | `Shape::Cuboid`                                                             | Exists                  |
| `Cylinder` prim                                            | `Shape::Cylinder`                                                           | Exists                  |
| Rigid body mark                                            | `Motion::Dynamic`; without it, `Motion::Fixed`                              | Exists                  |
| Kinematic flag                                             | `Motion::Kinematic`                                                         | Exists                  |
| Mass, centre of mass                                       | `Part::mass` and the part's offset                                          | Exists                  |
| Physics material: static and dynamic friction, restitution | A contact material of [[S16 Contact Materials]]                             | Needs a rule (§3)       |
| Material bound to a collision piece                        | `Part::material`                                                            | Exists                  |
| Joints (hinge, slider)                                     | Nothing                                                                     | Out of scope            |
| NVIDIA's own `Physx…` properties                           | Ignored, listed in the import report                                        | —                       |
| Connection point of a conveyor piece                       | A port of the family (for snapping)                                         | Possible, by convention |
| Movement of belt or rollers                                | **Not in the model.** An IntraCraft parameter drives `set_surface_velocity` | Contract exists         |

## 2. Three kinds of imported object

In growing order of work and risk:

1. **Fixed scenery** (building, racks, a parked forklift, a machine housing). Collision pieces become a fixed body. A fixed
   triangle mesh is exact and cheap for the engine. This is where imported models pay off first.
2. **Loads and free objects** (a bin, a drum, a crate on a pallet, something dropped with `sim.spawnObject`). They move, so
   they need **convex** pieces and a mass. A moving triangle mesh is not reliable in any real-time engine. Models with a
   hand-made convex set or a `convexDecomposition` mark import directly; a bare mesh needs decomposition at import.
3. **Conveying equipment** (an imported belt or roller piece that actually moves loads). The belt mesh becomes a moving
   surface whose speed comes from an IntraCraft parameter. This needs a discrete (2D) counterpart, ports, sensors and a
   specification, like every library object. It is new scope (carton conveyors are outside the MVP) and should not start
   without an explicit decision.

## 3. Friction: two different rules

- USD puts friction **on each material** and combines the two touching materials with a mode (`average`, `min`, `max`,
  `multiply`). The sample conveyor's belt uses `max`, its rollers `average`.
- IntraCraft uses a **pair table** ([[S16 Contact Materials]]): the friction of wood on steel is a value of its own.

Proposed rule for the import: each USD physics material is matched to an S16 material by a small table chosen by the person
importing (belt rubber, steel roller, …), and the USD numbers are kept only as a note on the family. Inventing new pairs
from single-material values would change simulation results silently, which `AGENTS.md` forbids without a decision record.

## 4. What does not change

- **The parametric pallet conveyors** keep their own geometry of rollers and chains. Their acceptance criteria (front
  within ±5 mm at a stop, rebound under 10 mm, sensor events of 2D and 3D within ±50 ms) do not involve imported models.
- **Determinism.** Convex pieces are computed once at import and stored with the family, so the same layout and seed give
  the same result on every machine. Decomposition must never run at the start of a simulation.
- **Photocells** must see the same pieces the physics module collides with, as they do today for pallets.
- **The discrete module** sees fixed scenery as nothing (`des.none`), as today.

## 5. Cost of physics with imported models

| Collision shape                                  | Cost for the engine                | Use                                                           |
| ------------------------------------------------ | ---------------------------------- | ------------------------------------------------------------- |
| Box, cylinder                                    | Lowest                             | Everything parametric (today)                                 |
| Convex piece                                     | Low                                | Loads, small equipment                                        |
| Fixed triangle mesh, small                       | Low to medium                      | Scenery with hand-made collision pieces (a few hundred faces) |
| Triangle mesh of the visual body (100,000 faces) | High, and catches pallets on bolts | Never: use the collision pieces                               |

The heavy simulator is slow mainly because of its renderer and because it simulates many more things (robots, sensors,
soft bodies). Its collision pieces are light; using them in Avian does not import that weight.
