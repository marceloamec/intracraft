---
title: S10 Pallet Types
lang: en
tags: [spec, pallet, reference-data]
library-id: pallet.*
status: reference-json
updated: 2026-10-29
---

🌐 **EN** · [[S10 Tipos de pallet|ES]] · [[S10 Palettentypen|DE]] · [[Home (EN)|Index]]

# S10 Pallet Types and Friction Pairs

Data: `packages/library/src/definitions/pallet-types.json` and `friction-pairs.json` (validated at load time, tested). **The project works with EUR pallets** (owner, 2026-10-29): longitudinal on rollers, transverse on chains. Values are **indicative**; check against the standard or real pallets before relying on sensor results.

## 1. Pallet types

| Type        | L × W (mm)  | Height (mm) | Tare (kg) | Profile status | Notes                                |
| ----------- | ----------- | ----------- | --------- | -------------- | ------------------------------------ |
| EUR / EPAL  | 1200 × 800  | 144         | 25        | indicative     | Main type of the project             |
| 1200 × 1000 | 1200 × 1000 | ≈ 150       | 28        | approximate    | Width profile simplified             |
| GMA         | 1219 × 1016 | ≈ 140       | 25        | approximate    | Stringer pallet; placeholder profile |
| Custom      | free        | free        | free      | user           | Own profile in the editor (later)    |

A pallet instance also has load mass and height, `centerOfMassOffset` and custom key–value attributes.

## 2. EUR construction used for the profiles

| Layer (height, mm)        | Parts                                                                                                                         |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Bottom boards (0–22)      | 3 boards, 1200 mm long, across the 800 mm side at 0–100, 327.5–472.5, 700–800 mm (100 / 145 / 100 wide)                       |
| Blocks (22–100)           | 9 blocks; along the 1200 mm side at 0–145, 527.5–672.5, 1055–1200 mm; along the 800 mm side at 0–100, 327.5–472.5, 700–800 mm |
| Stringer boards (100–122) | 3 boards, 800 mm long, 145 mm wide, on top of the block rows                                                                  |
| Top deck (122–144)        | 5 boards, 1200 mm long (145 / 100 / 145 / 100 / 145 wide) with gaps of ≈ 41 mm                                                |

## 3. Occupancy profiles (what a horizontal beam perpendicular to travel sees)

Solid intervals are measured from the pallet front, in m.

**Long side first — `alongLength` (roller conveyors):**

| Band (mm) | Solid intervals                            | Beam behaviour            |
| --------- | ------------------------------------------ | ------------------------- |
| 0–22      | [0, 1.2]                                   | Continuous                |
| 22–100    | [0, 0.145], [0.5275, 0.6725], [1.055, 1.2] | Gaps of 382.5 mm          |
| 100–122   | [0, 0.145], [0.5275, 0.6725], [1.055, 1.2] | Gaps of 382.5 mm          |
| 122–144   | [0, 1.2]                                   | **Continuous** (top deck) |

**Short side first — `alongWidth` (chain conveyors):**

| Band (mm) | Solid intervals                                                                    | Beam behaviour                   |
| --------- | ---------------------------------------------------------------------------------- | -------------------------------- |
| 0–22      | [0, 0.1], [0.3275, 0.4725], [0.7, 0.8]                                             | Gaps of 227.5 mm                 |
| 22–100    | [0, 0.1], [0.3275, 0.4725], [0.7, 0.8]                                             | Gaps of 227.5 mm                 |
| 100–122   | [0, 0.8]                                                                           | **Continuous** (stringer boards) |
| 122–144   | [0, 0.145], [0.18625, 0.28625], [0.3275, 0.4725], [0.51375, 0.61375], [0.655, 0.8] | Gaps of ≈ 41 mm                  |

Consequence: the best beam height depends on the orientation — ≈ 130 mm on rollers, ≈ 110 mm on chains ([[S06 Photoelectric Sensor]]). The 3D colliders are generated from the same construction.

## 4. Friction pairs

| Pair                  | Initial μ | Usual range | JSON id             |
| --------------------- | --------- | ----------- | ------------------- |
| Wood – steel (roller) | 0.35      | 0.25 – 0.50 | `wood-steel-roller` |
| Wood – steel chain    | 0.30      | 0.20 – 0.45 | `wood-steel-chain`  |
| Plastic – steel       | 0.25      | 0.15 – 0.40 | `plastic-steel`     |
| Wood – steel guide    | 0.30      | 0.20 – 0.45 | `wood-steel-guide`  |

## Related

[[T06 Photoelectric Sensing]] · [[S06 Photoelectric Sensor]] · [[T11 Contact, Friction and Restitution]] · [[S09 Source and Sink]] · [[Open Questions]]
