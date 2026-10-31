---
title: S05 Roller-to-Chain Transfer
lang: en
tags: [spec, conveyor, transfer, lift]
library-id: conveyor.transfer_rc
status: proposed rewrite — waiting for the owner's confirmation
updated: 2026-10-30
---

🌐 **EN** · [[S05 Transferencia rodillos-cadenas|ES]] · [[S05 Rollen-Ketten-Umsetzer|DE]] · [[Home (EN)|Index]]

# S05 Roller-to-Chain Transfer (`conveyor.transfer_rc`): lifting roller table

> **Status: proposed rewrite (2026-10-30), not implemented.**
> - On 2026-10-30 the owner decided that the MVP follows the real equipment: the **roller section lifts and lowers over fixed chains**.
> - This text describes that mechanism. It waits for the owner's confirmation before any code changes.
> - Until then, the implemented 2D cycle ([[ADR-0009 Transfer Table 2D Cycle and Stations]]) follows the previous version, kept in §9.
> - Values are **indicative**, not manufacturer data.

## 1. What it is

The table turns the flow by 90° where a **roller line crosses a chain line**, at practically the same level. The pallet changes direction without turning.

- **Moving part.** A short roller section, with its own driven rollers, lifts and lowers **100 mm** on a fixed base frame.
- **Fixed part.**
  - The **strands of the chain conveyor** run straight through the table, in channels between groups of rollers, and never move up or down.
  - The chain conveyor's top is **30 mm below** the roller conveyor's top.
- **Top position.** The rollers stand 30 mm above the chains, at the level of the roller conveyors: the pallet rides on the rollers.
- **Bottom position.** The rollers sink 70 mm below the chain top: the pallet rests on the chains and leaves on them.
- **Centre position** (optional). The rollers are still below the chains, but a **centre stop** fixed to the roller section stands above chain level. A pallet arriving on the chains is pushed against it at creep speed, which squares it before it goes onto the rollers.
- **Lift drive.** A crank mechanism turning one way only. The stroke follows a cosine-like profile: soft start, soft stop, with the top and bottom positions at the crank's dead centres.

Theory: [[T13 Turntables and Transfer Lifts]] part 2.

## 2. Types and ports

| Type  | Chain line                                | Ports                                                         | Stop for chains → rollers            |
| ----- | ----------------------------------------- | ------------------------------------------------------------- | ------------------------------------ |
| **T** | starts or ends at the table               | A, B (rollers); C1 **or** C2 (chains, one side)               | fixed end stop on the chain conveyor |
| **U** | continues on both sides (the lines cross) | A, B (rollers); C1 **and** C2 (chains), straight flow C1 ↔ C2 | centre stop on the table             |

- Roller ports A and B: at `surfaceHeight` (the roller conveyors' height).
- Chain ports C1 (−z) and C2 (+z): at `surfaceHeight − chainOffset`, the chain conveyor's height.
- When a chain conveyor ends at the table (type T), its strands are the ones that run through the table.

## 3. Parameters

| Key             | Unit | Default                   | Range                                                            | Description                                                                                    |
| --------------- | ---- | ------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Roller group    | —    | —                         | —                                                                | As [[S01 Roller Conveyor]]: `length` ≈ pallet length + margin, `rollerPitch`, `rollerDiameter` |
| `tableType`     | enum | T                         | T, U                                                             | §2                                                                                             |
| `surfaceHeight` | mm   | 600 (540 proposed in Q19) | 540 … 1400                                                       | Roller height at the top position                                                              |
| `liftStroke`    | mm   | 100                       | 60 … 150                                                         | Travel of the roller section                                                                   |
| `chainOffset`   | mm   | 30                        | 10 … 50                                                          | Chain top below the roller top at the top position                                             |
| `strokeTime`    | s    | 1.75                      | 0.5 … 5                                                          | Time for one full stroke, up or down (fitted to a throughput chart, §6)                        |
| `chainCount`    | —    | 2                         | 2 … 4                                                            | Strands crossing the table                                                                     |
| `trackGauge`    | mm   | 1050                      | 870, 1050                                                        | Distance between the outer strands                                                             |
| `transferSpeed` | m/s  | 0.25                      | 0.05 … 0.5                                                       | Chain speed while a pallet moves on or off the table                                           |
| `approachSpeed` | m/s  | 0.04                      | 0.02 … 0.1                                                       | Creep speed for the last centimetres before a stop                                             |
| `operationMode` | enum | rollersToChains           | rollersToChains, chainsToRollers, bidirectional, straightThrough | Flow through the table; `straightThrough` needs type U for the chains                          |
| `stopType`      | enum | end                       | end, centre                                                      | End stop on the chain conveyor (type T) or centre stop on the table (type U)                   |

**Derived with the defaults:**

- Bottom position: rollers 100 − 30 = **70 mm** below the chain top.
- One cycle rollers → chains at 0.3 m/s and 1.0 m/s² ([[T04 Conveyor Kinematics]]):
  - pallet onto the table, 1.25 m including the last 3 cm at creep speed: 1.25 / 0.3 + 0.3 / 1.0 + 0.75 = 5.22 s;
  - lower: 1.75 s;
  - chains carry it 0.85 m clear: 0.85 / 0.3 + 0.3 / 1.0 = 3.13 s;
  - lift: 1.75 s;
  - total **11.85 s → ≈ 304 pallets/h**.

## 4. Sequences

**Rollers → chains** (table at the top):

| Step | Condition                                                            | Action                                                                                     |
| ---- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| 1    | Table at the top and empty, chain outfeed free                       | The roller infeed and the table rollers run together; creep speed for the last centimetres |
| 2    | The barrier at the far roller end is reached                         | Table rollers and roller infeed stop together; **lowering starts at once**                 |
| 3    | Table at the bottom                                                  | The chain conveyor starts and carries the pallet away                                      |
| 4    | The pallet reaches the outfeed barrier, at least one load width away | **Lifting starts** (the chains keep running); at the top the next pallet may enter         |

**Chains → rollers with a centre stop** (type U, table at the bottom):

| Step | Condition                                     | Action                                                                                  |
| ---- | --------------------------------------------- | --------------------------------------------------------------------------------------- |
| 1    | Pallet waiting on the chains before the table | —                                                                                       |
| 2    | Table free                                    | Lift to the **centre** position: the centre stop is now up                              |
| 3    | At the centre                                 | Chains convey the pallet onto the table, then slow to creep speed                       |
| 4    | The chain-side barrier sees the pallet        | Keep creeping for a short time: the pallet is **pushed square** against the centre stop |
| 5    | —                                             | Chains stop, then **reverse briefly** at creep speed to free the pallet                 |
| 6    | —                                             | Lift to the **top**: the pallet is now on the rollers                                   |
| 7    | At the top                                    | Table rollers and roller outfeed start together                                         |

With type T the chains push the pallet against the fixed end stop of the chain conveyor instead (steps 4–5), and the lift goes from the bottom straight to the top.

**Straight through:** rollers A → B with the table at the top; chains C1 ↔ C2 (type U) with the table at the bottom.

## 5. Interlocks and checks

- Lift or lower only while the table rollers and the chains are at standstill, or while the table is empty. Otherwise the load may tip.
- The lift has a **time limit**: a stroke that takes too long is a fault.
- No pallet enters an occupied table.
- The table rollers run at the speed of the roller line. A chain line that delivers to the table needs two speeds (nominal and creep).
- With two strands, a pallet may sag between them. It may cross the side where the roller drive chains are only with the table fully lowered (warning in the editor).
- Validation `chainFitsBetweenRollers`: each channel left for a strand between two roller groups must be at least the strand width + 10 mm.

## 6. Behaviour

**2D (proposed).** A station with the cycles of §4.

- Lowering and lifting are timed steps of `strokeTime`. A lift in two stages (bottom → centre → top) takes the same total time.
- The push against the centre stop and the release add `stopContactTime` (S02) plus a short release time.
- The pallet keeps its orientation and swaps the side that runs along travel, as today ([[ADR-0009 Transfer Table 2D Cycle and Stations]]).

**3D (proposed).**

- The roller section is one kinematic body that moves vertically with the cosine-like crank profile, carrying its driven rollers.
- The strands are fixed strips with a surface velocity (plan B, [[ADR-0002 Physics Engine Candidate]]) in channels between the roller groups.
- The centre stop moves with the roller section; the end stop is fixed on the chain conveyor.

## 7. Reference tests

| Test   | Setup                                                                                             | Expected (± 5 %)          |
| ------ | ------------------------------------------------------------------------------------------------- | ------------------------- |
| T4     | The analytic cycle formula of the MVP plan stays unchanged                                        | 7.4 s                     |
| REF-L1 | Saturated roller line → table → chain line; EUR, 0.3 m/s, 1.0 m/s², creep 0.04 m/s, stroke 1.75 s | ≈ 300 pallets/h           |
| REF-L2 | As REF-L1 at 0.2 and 0.4 m/s                                                                      | ≈ 240 and ≈ 350 pallets/h |
| REF-L3 | Chain line → table with centre stop → roller line, 0.3 m/s, 1.0 m/s²                              | ≈ 285 pallets/h           |
| P-L1   | 3D: the pallet ends on the chains square to within 1° and within ±10 mm of the strand centre      | —                         |

REF-L1 to REF-L3 come from a published throughput chart of such tables. The stroke time of 1.75 s was fitted so that the cycle model of §3 matches that chart within ±2.3 % at six points.

## 8. What changes compared to the implemented version

| Topic       | Implemented today (§9)                  | Proposed                                                           |
| ----------- | --------------------------------------- | ------------------------------------------------------------------ |
| Moving part | Liftable chains between fixed rollers   | Roller section; the chains are fixed                               |
| Heights     | Chain ports 40 mm **above** the rollers | Chain ports 30 mm **below** the rollers                            |
| Stroke      | 40 mm, `liftTime` / `lowerTime` 1.0 s   | 100 mm, `strokeTime` 1.75 s, crank profile                         |
| Positions   | down / up                               | bottom / centre (optional) / top                                   |
| Stop        | Retractable stop on the roller section  | End stop on the chain conveyor (T) or centre stop on the table (U) |
| Modes       | three                                   | four, with straight-through flow                                   |
| Results     | T4 7.4 s; REF-5 120 pallets/h           | T4 formula unchanged; REF-L1…L3 added; REF-5 to be re-measured     |

## 9. Previous version (implemented in 2D until the rewrite is confirmed)

**Interpretation (to confirm, plan §18):** a single station with two orthogonal conveying systems. A roller section receives the pallet longitudinally; between the rollers sit liftable chains oriented transversely. Lifted, the chains raise the pallet off the rollers, move it 90° to the adjacent chain line and lower. The pallet changes direction without rotating. Theory: [[T13 Turntables and Transfer Lifts]] part 2.

**Ports:** A and B (rollers, longitudinal), C1 and C2 (chains, left and right).

### Previous 1. Parameters

| Key                      | Plan key                    | Unit | Default        | Range                                         | Description                                                                   |
| ------------------------ | --------------------------- | ---- | -------------- | --------------------------------------------- | ----------------------------------------------------------------------------- |
| Roller group             | —                           | —    | —              | —                                             | As [[S01 Roller Conveyor]]; `length` ≈ pallet length + 200 mm                 |
| `transferChainCount`     | `numCadenasTransfer`        | —    | 3              | 2 … 5                                         | Number of liftable chains                                                     |
| `transferChainPositions` | `posicionesCadenasTransfer` | mm   | auto           | —                                             | Longitudinal position of each chain, between rollers, aligned with the pallet |
| `transferChainWidth`     | `anchoCadenaTransfer`       | mm   | 40             | 20 … 80                                       | Width of each chain                                                           |
| `liftStroke`             | `carreraElevacion`          | mm   | 40             | 20 … 100                                      | Height of the chains above the rollers when lifted                            |
| `lowerClearance`         | `holguraBajada`             | mm   | 10             | 0 … 30                                        | Distance below the roller plane when lowered                                  |
| `liftTime`               | `tiempoSubida`              | s    | 1.0            | 0.3 … 5                                       | Lifting time                                                                  |
| `lowerTime`              | `tiempoBajada`              | s    | 1.0            | 0.3 … 5                                       | Lowering time                                                                 |
| `actuator`               | `actuador`                  | enum | pneumatic      | pneumatic, electric                           | Pneumatic: fast with end cushioning; electric: with ramps                     |
| `transferSpeed`          | `velocidadTransferencia`    | m/s  | 0.20           | 0.05 … 0.5                                    | Chain speed during transfer                                                   |
| `transferAccel`          | `aceleracionTransferencia`  | m/s² | 0.5            | 0.1 … 2                                       | Chain ramp                                                                    |
| `transferDistance`       | `distanciaTransferencia`    | mm   | auto           | —                                             | Chain travel (≈ table width); editable                                        |
| `transferDirection`      | `direccionTransferencia`    | enum | right          | left, right, bidirectional                    | Side to transfer to                                                           |
| `operationMode`          | `modoOperacion`             | enum | rollers→chains | rollers→chains, chains→rollers, bidirectional | Flow direction                                                                |
| `positionTolerance`      | `toleranciaPosicion`        | mm   | ±20            | 5 … 50                                        | Allowed pallet deviation to allow lifting                                     |
| `requiresStop`           | `requiereTope`              | bool | yes            | —                                             | Includes a retractable positioning stop on the roller section                 |
| `transferChainMu`        | `muCadenaTransfer`          | —    | 0.30           | 0.1 … 0.6                                     | Pallet–chain friction                                                         |

### Previous 2. Sequence and interlocks

idle (chains down) → pallet in position (stop + sensors) → lifting → raised → transferring → discharged → lowering → idle.

Interlocks: no lift without a centred pallet · rollers stopped while chains are up · do not lower until the pallet has left the chain zone · in bidirectional mode never accept pallets from both sides at once.

### Previous 3. Formulas

- **Ideal cycle:** $t = t_{lift} + (d/v + v/a) + t_{lower}$ (+ settling). 1 m, 0.2 m/s, 0.5 m/s² → 1.0 + 5.4 + 1.0 = **7.4 s** (T4).
- **Gap check:** Ø89 mm at 206.3 mm pitch (project values) → gap 117.3 mm ≥ chain 40 mm + 10 mm ✓ (`chainFitsBetweenRollers`).
- **Height continuity:** adjacent chain line at roller height + `liftStroke` (port tolerance ±10 mm).

### Previous 4. Behaviour

**2D.** State machine with the durations above; the roller section is blocked while lifted. Implemented 2026-10-30 ([[ADR-0009 Transfer Table 2D Cycle and Stations]]): the chains rest down, lift after the pallet is centred on the rollers, carry it out sideways (its side along travel swaps) and lower once it has left.
**3D.** Chains are kinematic bodies that rise (vertical motion) and then move horizontally; the pallet is lifted by contact. The simulator warns if a chain falls into a void of the pallet and does not touch it.

### Previous 5. Tests

| Test | Criterion                                                  | Status                                            |
| ---- | ---------------------------------------------------------- | ------------------------------------------------- |
| T4   | Cycle 7.4 s                                                | ✅ formula; 2D cycle adds lift/lower times (test) |
| P5   | Lifted 40 ± 5 mm, moved 1 m ± 10 mm, set down rotated < 1° | ⏳ phase 8                                        |

## In the three modules

Since 2026-10-31 IntraCraft is built in Rust on BIMCraft with three modules ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). What this object is in each:

| Module              | This object                                                                                                                                                                                                                                                 |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Design              | Family of the library (category conveyor); its parameters form the attribute table (inherited ✔ / overridden ✖), its presets are the family's types; ports A, B, C1 and C2; placed from the Project Browser, listed in the Simulation tree and in schedules |
| Discrete simulation | Transfer cycle and stations ([[ADR-0009 Transfer Table 2D Cycle and Stations]]); the rewrite of this spec waits for the owner, the port follows the confirmed version                                                                                       |
| Physics simulation  | Not in 3D yet; planned in R7                                                                                                                                                                                                                                |

The behaviour above does not change. **Rust status:** not ported yet (library R1, 2D R3, 3D R7 of [[A19 Rust Implementation Plan]]); the tests of this spec are the expected values of the port.

## Related

[[T13 Turntables and Transfer Lifts]] · [[S01 Roller Conveyor]] · [[S02 Chain Conveyor]] · [[T07 Control Logic and Signals]] · [[Open Questions]] Q6, Q19
