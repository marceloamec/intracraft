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

## 2. Pair table

| Pallet  | Against       | μ    | Range       | Restitution e |
| ------- | ------------- | ---- | ----------- | ------------- |
| wood    | steel-zinc    | 0.35 | 0.25 … 0.5  | 0             |
| wood    | steel-painted | 0.3  | 0.2 … 0.45  | 0.1           |
| wood    | steel-chain   | 0.3  | 0.2 … 0.45  | 0             |
| wood    | rubber        | 0.6  | 0.5 … 0.8   | 0.35          |
| wood    | polyurethane  | 0.5  | 0.4 … 0.7   | 0.2           |
| wood    | polyethylene  | 0.2  | 0.15 … 0.25 | 0.1           |
| wood    | wood          | 0.4  | 0.3 … 0.5   | 0.1           |
| wood    | plastic       | 0.3  | 0.2 … 0.4   | 0.1           |
| wood    | concrete      | 0.5  | 0.4 … 0.6   | 0.1           |
| plastic | steel-zinc    | 0.25 | 0.15 … 0.4  | 0             |
| plastic | steel-painted | 0.25 | 0.15 … 0.4  | 0.1           |
| plastic | steel-chain   | 0.2  | 0.15 … 0.3  | 0             |
| plastic | rubber        | 0.45 | 0.35 … 0.6  | 0.35          |
| plastic | polyurethane  | 0.4  | 0.3 … 0.55  | 0.2           |
| plastic | polyethylene  | 0.15 | 0.1 … 0.2   | 0.1           |
| plastic | plastic       | 0.3  | 0.2 … 0.4   | 0.1           |
| plastic | concrete      | 0.4  | 0.3 … 0.5   | 0.1           |

Restitution 0 on rollers and chains: a pallet that drops a few millimetres onto a conveyor must not bounce.

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
