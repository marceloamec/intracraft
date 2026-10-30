---
title: Open Questions
lang: en
tags: [questions, reference]
updated: 2026-10-30
---

🌐 **EN** · [[Preguntas abiertas|ES]] · [[Offene Fragen|DE]] · [[Home (EN)|Index]]

# Open Questions

Each question states **what must be decided**, **why it matters** (which result changes), **options** and **what is assumed meanwhile**. Answers move into the relevant specification and are marked as answered.

## Answered (2026-10-29)

| Topic                | Answer                                                                                                                                                  | Applied in                                                                |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Default accumulation | None: every pallet follows the chain or roller movement; when another pallet enters, the one already there moves; behaviour is a per-instance parameter | [[S01 Roller Conveyor]] §3, [[S02 Chain Conveyor]]                        |
| Photocell            | Height and position of the photocell and of the reflector are instance parameters                                                                       | [[S06 Photoelectric Sensor]]                                              |
| Pallet orientation   | Longitudinal on rollers, transverse on chains                                                                                                           | S01, S02, [[T06 Photoelectric Sensing]]                                   |
| Pallet type          | European (EUR / EPAL)                                                                                                                                   | [[S10 Pallet Types]]                                                      |
| Rollers              | Ø 89 mm, pitch 206.3 mm; diameter and pitch chosen from a few options                                                                                   | S01 §2, `roller-conveyor.json`                                            |
| AI agents            | Designed for agents from the start (direct and MCP); own gateway + token; document only                                                                 | [[A15 AI Agent Integration]], [[ADR-0004 AI-Ready Command Layer and MCP]] |

## Open

### Q1. List of roller diameters and pitches
- **Decide:** which diameter/pitch combinations the dropdown offers.
- **Why:** sets the rollers under the pallet (3-roller rule), the gap for transfer chains and the 3D roller speed.
- **Specific questions:** (a) Which diameters besides 89 mm? (b) Is each diameter tied to one pitch (pairs) or can they be combined freely? (c) Is the pitch always a multiple of the drive chain? 206.3 mm ≈ 13 × 15.875 mm (5/8" chain). (d) Keep the plan's 76/133 as an option or remove it?
- **Assumed:** diameters {76, 89}, pitches {133, 206.3}, freely combined; default 89/206.3.

### Q2. Exact behaviour without accumulation (`none`)
- **Decide:** what happens when the front pallet reaches the end and the next conveyor is not free.
- **Why:** changes effective capacity, blocked time and stopping position.
- **Specific questions:** (a) Does the conveyor stop when the front reaches the conveyor end, or when an end sensor X mm before it detects it? What is X? (b) Does it brake with the normal ramp (the pallet overruns the sensor by $v^2/2b$ = 62.5 mm at 0.25 m/s) or stop at an exact position? (c) While stopped with a pallet at the end, can another pallet enter? (In `none` this would move the front one, so I assume **no**.) (d) Is there always a default end sensor?
- **Assumed (implemented 2026-10-30, [[ADR-0007 2D Flow Model Without Accumulation]]):** a virtual end sensor at L − v²/2b, so the conveyor brakes with its ramp and the pallet stops with its front exactly at the end. It does not receive while blocked, and it only accepts a pallet that can get completely aboard.

### Q3. Speed during hand-over between conveyors with different speeds
- **Decide:** the speed of a pallet straddling A and B in 2D.
- **Why:** transit times and 2D/3D consistency (±5 %).
- **Options:** (1) the slower one; (2) the receiving one; (3) the delivering one until the centre of mass crosses over.
- **Assumed (implemented, ADR-0007):** the slower one; both conveyors move together while the pallet crosses.

### Q4. Chain conveyor with transverse EUR: width and guides
- **Decide:** default usable width and whether there are side guides.
- **Why:** with 1200 mm across, a 1000 mm width fails validation.
- **Specific questions:** (a) Does the pallet run between frames/guides (width ≥ 1250 mm) or overhang the chains without guides? (b) How many strands and at what spacing?
- **Assumed:** usable width 1300 mm, 3 strands, guides on both sides.

### Q5. Chain turntable with transverse EUR: platform size
- **Decide:** default platform dimensions.
- **Why:** the plan gives 1400 (travel) × 1200 (across); a transverse EUR occupies 800 × 1200 — exactly the width. Affects swept radius and clearances.
- **Specific question:** what are the dimensions of your chain and roller turntables?
- **Assumed:** the plan's values.

### Q6. Roller-to-chain transfer table
- **Decide:** confirm the mechanism and its values.
- **Why:** highest-risk component (phase 8); its cycle limits throughput.
- **Specific questions:** (a) Is it correct: longitudinal rollers + liftable chains between rollers that rise, move 90° and lower? (b) Number of chains, lift stroke (40 mm), lift/lower times (1 s). (c) Is the adjacent chain conveyor 40 mm higher than the rollers, or do the chains lower to hand over at the same height? (d) Is there a positioning stop?
- **Assumed:** the plan's interpretation; with Ø89/206.3 the roller gap is 117.3 mm (chains up to ~107 mm fit).

### Q7. Photocells: type and usual heights
- **Decide:** default type and height when placing a new photocell.
- **Why:** with EUR, at 60 mm the signal flickers (gaps of 382.5 mm on rollers, 227.5 mm on chains); continuous bands are ≈ 122–144 mm on rollers and 100–122 mm on chains.
- **Specific questions:** (a) Retroreflective (photocell + reflector) or through-beam? (b) Usual mounting heights on rollers and on chains? (c) Are oblique or inclined beams relevant, or always perpendicular and horizontal? (d) Do you use PLC filters/timers for the voids?
- **Assumed:** retroreflective, 60 mm (plan) with the void warning.

### Q8. Loads
- **Decide:** typical and maximum load mass and height; fixed or variable.
- **Why:** inertia, turntable torque, impact energy at stops, tipping risk.
- **Assumed:** 600 kg, 1200 mm high; maximum 1000 kg (up to 1500 kg?).

### Q9. Real speeds and ramps
- **Decide:** your equipment's defaults for rollers, chains, turntables and transfers.
- **Assumed:** 0.25 m/s and 0.5 m/s²; turntable 15 °/s and 30 °/s²; transfer 0.2 m/s.

### Q11. Which exit a turntable sends a pallet to
- **Decide:** until control rules exist, how a turntable with several connected exits chooses one.
- **Why:** throughput of each branch, platform turns and utilisation.
- **Options:** (1) in turn (`alternate`); (2) always the first exit clockwise from the entry; (3) random with equal chances; (4) the first free exit (needs look-ahead; better in the controller).
- **Assumed (implemented 2026-10-30, [[ADR-0008 Turntable 2D Cycle and Exit Rule]]):** a parameter `exitRule` per turntable with options 1–3, default `alternate`.

### Q10. Reference models for comparison
- **Decide:** three models you know well (simple flow, accumulation, turntable or transfer) with their results (throughput, cycle time, occupancy).
- **Why:** criterion 8 of the MVP definition of done.

### Q11. AI agents
- **Specific questions:** (a) Which providers or gateways should the embedded chat support first (Anthropic directly, a company gateway, other providers, local models)? (b) Which MCP clients will you use? (c) May an agent modify the layout without confirmation, or always ask? (d) Local only, or remote access to the MCP server too?
- **Assumed:** read-only by default; confirmation for delete/overwrite; MCP local only.

### Q12. 3D hardware, licence and name
- (a) Laptop with a dedicated GPU, or modest machines too? (b) Open source, internal or commercial? (c) Shall we check the IntraCraft trademark (domain, GitHub, npm)?

### Q13. Real pallet measurements
- **Decide:** whether your pallets match the EPAL construction used in [[S10 Pallet Types]] (bottom boards 100/145/100 mm, 78 mm blocks, 22 mm stringer boards, 5 top boards).
- **Why:** it determines the voids the photocells see and the support on chains.
- **Request:** if possible, measure a real pallet (block and board positions).

### Q14. Pallet-jack side of the manual lift table and the gravity line
- **Decide:** the deck height at pallet-jack level and how the jack reaches the pallet.
- **Why:** it sets the stroke and the cycle time of [[S12 Manual Lift Table]], and the entry height of [[S15 Gravity Roller Line]].
- **Specific questions:** (a) What is the lowest deck height at the operator side (`lowHeight`)? (b) Does the jack enter along the line (end) or from the side? (c) Is the deck rollers only, or is there a chain or fork-pocket section for the jack? (d) How long does an operator need per pallet (drive in, lift, leave, come back)?
- **Assumed:** 0.10 m, entry from the end, rollers only; operator time is a sink `removalTime`.

### Q15. Lift speeds and the elevator's rest level
- **Decide:** real vertical speeds and what a lift does after discharging.
- **Why:** they set the cycle time and the throughput of the scissor table, the manual table and the elevator.
- **Specific questions:** (a) Scissor and manual tables: lifting and lowering speed, or time for the full stroke? (b) Elevator: carriage speed and acceleration? (c) Does a lift always go back to the entry level, or wait where it is until the next call?
- **Assumed:** tables 0.05 m/s up, 0.07 m/s down, 0.2 m/s²; elevator 0.5 m/s and 0.5 m/s²; always back to the entry level ([[ADR-0013 Lift Station 2D Cycle]]).

### Q16. Gravity line values and queue behaviour
- **Decide:** slope, brake spacing, brake speed, separator delay and how a queue starts.
- **Why:** they set the time a pallet needs to reach the operator, the line's capacity and whether the operator or the line limits throughput.
- **Specific questions:** (a) Typical slope (%)? (b) One brake roller every how many rollers? (c) Speed held by the brakes? (d) Is the separator time-delayed, and with what delay? (e) When the separator lets go, does the whole queue move as one block, or do the pallets spread out? If they spread, roughly how far apart (that gives `startDelay` = gap / brake speed)? (f) Length of the pick-up position?
- **Assumed:** 4 %, every 4 rollers (825 mm), 0.2 m/s, 3 s, `startDelay` 1 s (gaps of ≈ 0.2 m), pick-up 1.3 m ([[S15 Gravity Roller Line]]).

### Q17. Elevator variants
- **Decide:** which variants beyond the two-level C and Z elevator are needed, and in what order.
- **Why:** several levels and entries from both ends need a destination rule and change the cycle ([[S14 Pallet Elevator]]).
- **Specific questions:** (a) How many levels at most? (b) Entry from both ends? (c) How is the destination level chosen (fixed, by pallet type, by rule)?
- **Assumed:** two levels, entry on one end, exit on the same side (C) or the opposite side (Z).

### Q18. Side guides: material and height
- **Decide:** the default guide surface and height of [[S08 Side Guide]].
- **Why:** in the 3D mode a pallet that enters turned 5° **wedges** between painted steel guides (μ = 0.3) with 10 mm clearance. With low-friction polyethylene strips (μ = 0.2) it is straightened ([[ADR-0002 Physics Engine Candidate]], P9). The default height of 150 mm is above the top of a EUR pallet (144 mm), so a load that overhangs the pallet hits the guides.
- **Options:** (1) polyethylene strips as the default guide surface; (2) keep painted steel and add a warning when the entry may be turned more than about 3°; (3) guide height limited to the pallet height (about 100–140 mm), or a validation against the load overhang.
- **Assumed:** S08 unchanged; the bench uses 100 mm polyethylene guides for S1-d.

### Q19. Default values from the pallet-conveyor knowledge base
- **Decide:** which of these defaults to change. They come from application guides of heavy pallet conveyors (indicative values, not manufacturer data in IntraCraft).
- **Why:** they change heights, widths and speeds, so 2D results and saved layouts that rely on the defaults change. Only drawing details were applied on 2026-10-30 (`driveSide`, `driveMounting`, `runnerGuide`, `endStop`, `drivePosition`, `sideGuard`).

| #   | Object                     | Parameter                         | Today                           | Proposed                                                        | Changes results |
| --- | -------------------------- | --------------------------------- | ------------------------------- | --------------------------------------------------------------- | --------------- |
| a   | Roller conveyor (S01)      | `surfaceHeight`                   | 600 mm                          | 520 mm (540 mm next to turntables and transfer tables)          | No (2D)         |
| b   | Roller conveyor            | `usableWidth`                     | free, 1000 mm                   | List 969 / 1169 / 1388 mm for loads up to 800 / 1000 / 1219 mm  | No (2D)         |
| c   | Roller conveyor            | `rollerPitch`, `rollerDiameter`   | {133, 206.3} mm, {76, 89} mm    | {126.9, 206.3} mm, Ø 89 mm only                                 | 3D only         |
| d   | Roller conveyor            | `nominalSpeed`                    | 0.25 m/s                        | 0.3 m/s (list 0.2 / 0.3 / 0.4 / 0.5)                            | Yes             |
| e   | Roller conveyor            | `approachSpeed`                   | 0.10 m/s                        | 0.04 m/s creep speed                                            | When used       |
| f   | Roller conveyor            | `maxPalletLoad`                   | 1000 kg                         | 1500 kg per load                                                | Warnings only   |
| g   | Chain conveyor (S02)       | `usableWidth`, `chainCount`       | 1300 mm, 3 (2–6)                | Track gauge 700 / 870 / 1050 mm (default 1050) with 2–4 strands | 3D only         |
| h   | Chain conveyor             | `chainPitch`, `surfaceHeight`     | 38.1 mm, 600 mm                 | 15.875 mm, 510 mm                                               | 3D only         |
| i   | Chain conveyor             | length check                      | —                               | Warn above 9 m (centre drive) or 6 m (head drive)               | Warnings only   |
| j   | Turntables (S03, S04)      | `maxAngularSpeed`, `angularAccel` | 15 °/s, 30 °/s² (6.5 s per 90°) | 45 °/s, 45 °/s² (3 s per 90°)                                   | Yes             |
| k   | Turntables                 | angular range                     | unlimited                       | 0–270°, shortest path inside the range                          | Yes             |
| l   | Turntables                 | size                              | 1400 × 1200 mm platform         | Turning circle Ø 1710 mm                                        | Yes             |
| m   | Roller and chain conveyors | receive rule                      | receive while not blocked       | Option "receive only when empty"                                | Yes             |

- **Assumed:** today's values stay until the owner ticks the rows to apply. The roller↔chain transfer (S05) is handled separately: its rewritten specification is waiting for confirmation.
