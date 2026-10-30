---
title: S02 Chain Conveyor
lang: en
tags: [spec, conveyor, chain]
library-id: conveyor.chain
status: spec
updated: 2026-10-29
---

🌐 **EN** · [[S02 Mesa de cadenas|ES]] · [[S02 Kettenförderer|DE]] · [[Home (EN)|Index]]

# S02 Chain Conveyor (`conveyor.chain`)

Straight section where the pallet rests on several synchronised chain strands. **EUR pallets travel transversely (short side, 800 mm, first) on chain conveyors** — confirmed 2026-10-29. The three bottom boards (1200 mm long) then lie **across** the chains, so every strand touches every board. Adds to the common parameters of [[S01 Roller Conveyor]] §1; all behaviour parameters are per instance.

## 1. Parameters

| Key                     | Plan key                | Unit | Default             | Range                        | Description                                                                                                                                       |
| ----------------------- | ----------------------- | ---- | ------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `palletSideAlongTravel` | `ladoParaleloAvance`    | enum | **short**           | long, short                  | EUR: 800 mm along travel, 1200 mm across                                                                                                          |
| `usableWidth`           | `anchoUtil`             | mm   | **1300** (proposed) | 500 … 1800                   | Must be ≥ 1200 + 2 × clearance for a transverse EUR (common default 1000 mm would fail) — to confirm                                              |
| `chainCount`            | `numCadenas`            | —    | 3                   | 2 … 6                        | Number of chain strands                                                                                                                           |
| `chainPositions`        | `posicionesCadenas`     | mm   | auto                | —                            | Lateral position of each strand from the axis                                                                                                     |
| `chainSupportWidth`     | `anchoApoyoCadena`      | mm   | 40                  | 20 … 100                     | Contact width of each strand                                                                                                                      |
| `chainType`             | `tipoCadena`            | enum | roller              | simple, roller, double pitch | Friction and support geometry                                                                                                                     |
| `chainPitch`            | `pasoCadena`            | mm   | 38.1                | 19.05 … 76.2                 | Pitch; geometry and polygon effect in 3D                                                                                                          |
| `chainMu`               | `muCadena`              | —    | 0.30                | 0.1 … 0.6                    | Pallet–chain friction                                                                                                                             |
| `accumulationMode`      | `modoAcumulacion`       | enum | **none**            | none, slip, zoned            | none: all pallets move with the chain, the conveyor runs and stops as one unit; slip: chains slide under held pallets; zoned: independent strands |
| `zoneLength`            | `longitudZona`          | mm   | 1400                | 1000 … 2000                  | Zoned only                                                                                                                                        |
| `stopContactTime`       | `tiempoDetencionTope`   | s    | 0.2                 | 0.05 … 1                     | 2D: time for the pallet to stop at a stop (slip mode)                                                                                             |
| `maxDragForce`          | `fuerzaMaxArrastre`     | N    | derived             | —                            | μ × pallet weight; informative                                                                                                                    |
| `driveSide`             | `ladoAccionamiento`     | enum | right               | right, left                  | Side of the geared motor, seen in the direction of travel (drawing only)                                                                          |
| `drivePosition`         | `posicionAccionamiento` | enum | centre              | centre, end                  | Centre drive (the chains loop down to the drive sprockets mid-length) or head drive (drawing only)                                                |
| `sideGuard`             | `proteccionLateral`     | enum | none                | none, right, left, both      | Rail on posts outside the outer strand; guides mesh boxes and plastic pallets (drawing only)                                                      |
| `endStop`               | `topeFinal`             | bool | no                  | —                            | Fixed plate at a free line end (drawing only)                                                                                                     |

Capacity with a transverse EUR: $\lfloor \text{length}/(0.8 + \text{minGap}) \rfloor$ → 3 m holds 3 pallets with a 150 mm gap. Spacing-limited throughput at 0.25 m/s: $0.25/(0.8+0.15)$ = 0.263 pallets/s ≈ **947 pallets/h**.

Ports A (in) and B (out), as for the roller conveyor.

## 2. Behaviour

- **`none` (default):** identical to the roller conveyor: every pallet follows the chain; receiving a pallet moves the pallets already on it; a blocked exit stops the whole conveyor ([[S01 Roller Conveyor]] §3).
- **`slip`:** a held pallet stops and the chain keeps running under it (line pressure $\sum \mu m g$, [[T05 Accumulation, Capacity and Throughput]] §6.3).
- **3D:** each strand is a strip of small kinematic rollers (plan A) or a static box with **surface velocity** (plan B, available in Jolt's JS binding) — [[T12 Rollers and Chains in Physics]], [[ADR-0002 Physics Engine Candidate]].

## 3. Validations

- At least 2 chains under the pallet; with transverse EUR every strand crosses all three bottom boards.
- Usable width ≥ 1200 + 2 × clearance for transverse EUR.

## 4. Tests

| Test | Criterion                                             | Status     |
| ---- | ----------------------------------------------------- | ---------- |
| S1-b | Pallet speed = chain speed ±3 %, no visible vibration | ⏳ S1      |
| —    | `none`: entering pallet moves the resident pallet     | ⏳ phase 5 |
| —    | `slip`: line force ≈ Σ μ m g                          | ⏳ phase 5 |

## Related

[[T12 Rollers and Chains in Physics]] · [[S05 Roller-to-Chain Transfer]] · [[S03 Chain Turntable]] · [[Open Questions]]
