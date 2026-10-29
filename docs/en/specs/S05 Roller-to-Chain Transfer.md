---
title: S05 Roller-to-Chain Transfer
lang: en
tags: [spec, conveyor, transfer]
library-id: conveyor.transfer_rc
status: spec
---

🌐 **EN** · [[S05 Transferencia rodillos-cadenas|ES]] · [[S05 Rollen-Ketten-Umsetzer|DE]] · [[Home (EN)|Index]]

# S05 Roller-to-Chain Transfer (`conveyor.transfer_rc`)

**Interpretation (to confirm, plan §18):** a single station with two orthogonal conveying systems. A roller section receives the pallet longitudinally; between the rollers sit liftable chains oriented transversely. Lifted, the chains raise the pallet off the rollers, move it 90° to the adjacent chain line and lower. The pallet changes direction without rotating. Theory: [[T13 Turntables and Transfer Lifts]] part 2.

**Ports:** A and B (rollers, longitudinal), C1 and C2 (chains, left and right).

## 1. Parameters

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

## 2. Sequence and interlocks

idle (chains down) → pallet in position (stop + sensors) → lifting → raised → transferring → discharged → lowering → idle.

Interlocks: no lift without a centred pallet · rollers stopped while chains are up · do not lower until the pallet has left the chain zone · in bidirectional mode never accept pallets from both sides at once.

## 3. Formulas

- **Ideal cycle:** $t = t_{lift} + (d/v + v/a) + t_{lower}$ (+ settling). 1 m, 0.2 m/s, 0.5 m/s² → 1.0 + 5.4 + 1.0 = **7.4 s** (T4).
- **Gap check:** Ø89 mm at 206.3 mm pitch (project values) → gap 117.3 mm ≥ chain 40 mm + 10 mm ✓ (`chainFitsBetweenRollers`).
- **Height continuity:** adjacent chain line at roller height + `liftStroke` (port tolerance ±10 mm).

## 4. Behaviour

**2D.** State machine with the durations above; the roller section is blocked while lifted.
**3D.** Chains are kinematic bodies that rise (vertical motion) and then move horizontally; the pallet is lifted by contact. The simulator warns if a chain falls into a void of the pallet and does not touch it.

## 5. Tests

| Test | Criterion                                                  | Status     |
| ---- | ---------------------------------------------------------- | ---------- |
| T4   | Cycle 7.4 s                                                | ✅ formula |
| P5   | Lifted 40 ± 5 mm, moved 1 m ± 10 mm, set down rotated < 1° | ⏳ phase 8 |

## Related

[[T13 Turntables and Transfer Lifts]] · [[S01 Roller Conveyor]] · [[S02 Chain Conveyor]] · [[T07 Control Logic and Signals]]
