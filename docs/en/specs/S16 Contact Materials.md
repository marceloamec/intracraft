---
title: S16 Contact Materials
lang: en
tags: [spec, physics, materials, friction]
status: implemented
updated: 2026-10-30
---

# S16 Contact Materials

Every physics body has a **contact material**. When two bodies touch, the contact takes its **friction coefficient** μ and its **restitution** e from the pair table. Pairs are symmetric: (wood, rubber) and (rubber, wood) are the same entry. Used by the 3D mode only ([[A07 Physics Engine Wrapper]], [[T11 Contact, Friction and Restitution]]).

Values are **indicative**, collected from general engineering tables. They are not measured on the owner's equipment and are not manufacturer data. The `range` column gives the spread found in the literature, for sensitivity studies.

Data: `packages/library/src/definitions/contact-materials.json` and `contact-pairs.json`, loaded by `reference-data.ts` (`CONTACT_MATERIALS`, `CONTACT_PAIRS`, `contactPair(a, b)`).

## 1. Materials

| Id              | Name              | Typical use                        |
| --------------- | ----------------- | ---------------------------------- |
| `wood`          | Wood              | Wooden pallets                     |
| `plastic`       | Plastic (HDPE)    | Plastic pallets                    |
| `steel-zinc`    | Zinc-plated steel | Plain roller tubes                 |
| `steel-painted` | Painted steel     | Side guides, stop plates, frames   |
| `steel-chain`   | Steel chain       | Conveyor chain strands             |
| `rubber`        | Rubber            | Roller coating, stop buffers       |
| `polyurethane`  | Polyurethane      | Roller coating, stop buffers       |
| `polyethylene`  | UHMW polyethylene | Low-friction guide and wear strips |
| `concrete`      | Concrete          | Floor                              |
| `cardboard`     | Cardboard         | Boxes and cartons (catalogue)      |

## 2. Pair table

| Material      | Against       | μ    | Range       | Restitution e |
| ------------- | ------------- | ---- | ----------- | ------------- |
| wood          | steel-zinc    | 0.35 | 0.25 … 0.5  | 0             |
| wood          | steel-painted | 0.3  | 0.2 … 0.45  | 0.1           |
| wood          | steel-chain   | 0.3  | 0.2 … 0.45  | 0             |
| wood          | rubber        | 0.6  | 0.5 … 0.8   | 0.35          |
| wood          | polyurethane  | 0.5  | 0.4 … 0.7   | 0.2           |
| wood          | polyethylene  | 0.2  | 0.15 … 0.25 | 0.1           |
| wood          | wood          | 0.4  | 0.3 … 0.5   | 0.1           |
| wood          | plastic       | 0.3  | 0.2 … 0.4   | 0.1           |
| wood          | concrete      | 0.5  | 0.4 … 0.6   | 0.1           |
| plastic       | steel-zinc    | 0.25 | 0.15 … 0.4  | 0             |
| plastic       | steel-painted | 0.25 | 0.15 … 0.4  | 0.1           |
| plastic       | steel-chain   | 0.2  | 0.15 … 0.3  | 0             |
| plastic       | rubber        | 0.45 | 0.35 … 0.6  | 0.35          |
| plastic       | polyurethane  | 0.4  | 0.3 … 0.55  | 0.2           |
| plastic       | polyethylene  | 0.15 | 0.1 … 0.2   | 0.1           |
| plastic       | plastic       | 0.3  | 0.2 … 0.4   | 0.1           |
| plastic       | concrete      | 0.4  | 0.3 … 0.5   | 0.1           |
| cardboard     | wood          | 0.35 | 0.3 … 0.5   | 0.05          |
| cardboard     | plastic       | 0.3  | 0.2 … 0.4   | 0.05          |
| cardboard     | steel-zinc    | 0.3  | 0.2 … 0.4   | 0             |
| cardboard     | steel-painted | 0.3  | 0.2 … 0.4   | 0.05          |
| cardboard     | steel-chain   | 0.3  | 0.2 … 0.4   | 0             |
| cardboard     | rubber        | 0.6  | 0.5 … 0.8   | 0.1           |
| cardboard     | polyurethane  | 0.5  | 0.4 … 0.7   | 0.1           |
| cardboard     | polyethylene  | 0.2  | 0.15 … 0.25 | 0.05          |
| cardboard     | concrete      | 0.5  | 0.4 … 0.6   | 0.05          |
| cardboard     | cardboard     | 0.4  | 0.3 … 0.6   | 0.05          |
| steel-painted | steel-zinc    | 0.4  | 0.3 … 0.6   | 0             |
| steel-painted | steel-painted | 0.4  | 0.3 … 0.6   | 0.3           |
| steel-painted | steel-chain   | 0.4  | 0.3 … 0.6   | 0             |
| steel-painted | rubber        | 0.6  | 0.5 … 0.8   | 0.35          |
| steel-painted | polyurethane  | 0.4  | 0.3 … 0.55  | 0.2           |
| steel-painted | polyethylene  | 0.2  | 0.15 … 0.25 | 0.1           |
| steel-painted | concrete      | 0.55 | 0.45 … 0.7  | 0.2           |
| steel-zinc    | steel-zinc    | 0.4  | 0.3 … 0.6   | 0             |
| steel-zinc    | steel-chain   | 0.4  | 0.3 … 0.6   | 0             |
| steel-zinc    | rubber        | 0.6  | 0.5 … 0.8   | 0             |
| steel-zinc    | polyurethane  | 0.4  | 0.3 … 0.55  | 0             |
| steel-zinc    | polyethylene  | 0.2  | 0.15 … 0.25 | 0             |
| steel-zinc    | concrete      | 0.55 | 0.45 … 0.7  | 0.2           |
| steel-chain   | steel-chain   | 0.4  | 0.3 … 0.6   | 0             |
| steel-chain   | rubber        | 0.6  | 0.5 … 0.8   | 0             |
| steel-chain   | polyurethane  | 0.4  | 0.3 … 0.55  | 0             |
| steel-chain   | polyethylene  | 0.2  | 0.15 … 0.25 | 0             |
| steel-chain   | concrete      | 0.55 | 0.45 … 0.7  | 0.2           |
| rubber        | rubber        | 1    | 0.8 … 1.2   | 0.5           |
| rubber        | polyurethane  | 0.8  | 0.6 … 1     | 0.4           |
| rubber        | polyethylene  | 0.4  | 0.3 … 0.5   | 0.3           |
| rubber        | concrete      | 0.7  | 0.6 … 0.85  | 0.35          |
| polyurethane  | polyurethane  | 0.6  | 0.5 … 0.8   | 0.3           |
| polyurethane  | polyethylene  | 0.3  | 0.2 … 0.4   | 0.2           |
| polyurethane  | concrete      | 0.6  | 0.5 … 0.8   | 0.2           |
| polyethylene  | polyethylene  | 0.2  | 0.15 … 0.25 | 0.1           |
| polyethylene  | concrete      | 0.3  | 0.25 … 0.4  | 0.1           |
| concrete      | concrete      | 0.6  | 0.5 … 0.8   | 0.1           |

Restitution 0 on rollers and chains: a pallet or object that drops a few millimetres onto a conveyor must not bounce.

The table covers **every pair of the ten materials** (55 pairs) since 2026-10-31, because scenery objects ([[S17 Scenery Asset]]) can touch anything. Rows after `plastic | concrete` were added then; sources and the reasoning per value are in [[ADR-0017 Scenery Assets in the Layout and Live Spawn]]. Pairs that only occur between two fixed parts (for example chain on concrete) never produce a contact; they exist so that the rule "missing pair = error" stays simple.

### 2.1 Catalogue materials (amendment 2026-10-31)

Catalogue models name SimReady material families. Each model gets one contact material, the family of its largest collision piece by volume:

| Family           | Contact material |
| ---------------- | ---------------- |
| wood             | `wood`           |
| plastic, nylon   | `plastic`        |
| cardboard, paper | `cardboard`      |
| metal            | `steel-painted`  |
| rubber           | `rubber`         |
| glass            | `steel-painted`  |

Glass only appears in small parts (vehicle windows), so it never decides the material of a model; the mapping exists for completeness.

## 3. Rules

- **Exact pair values.** The engine's own combination (geometric mean or average) is replaced by the pair value on every contact (Jolt contact listener, [[ADR-0002 Physics Engine Candidate]]).
- **Missing pair = error.** A contact between two materials without an entry stops the run with a clear message. No silent fallback.
- **Per-instance override.** A static or kinematic body (roller, stop, guide) may set its own μ or e, for example to test a worn roller. When both bodies of a contact set a value, the conveyor side wins.
- **Mesh boxes on steel runners** need a steel–steel pair. They arrive with the mesh box pallet type ([[S10 Pallet Types]]).

## 4. Reference tests

Physics bench scenarios ([[A10 CLI and Scenarios]]):

- **P2:** the largest acceleration a roller conveyor can give a pallet is a = μ·g.
  - Steel rollers with wood: 3.43 m/s².
  - Rubber-coated rollers with wood: 5.89 m/s².
  - Steel rollers with plastic: 2.45 m/s².
  - All within 10 %.
- **P5:** rebound speed / approach speed at a full-height stop face: 0.1 (painted steel), 0.2 (polyurethane) and 0.35 (rubber), within 10 %.
