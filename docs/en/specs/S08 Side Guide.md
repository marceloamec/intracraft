---
title: S08 Side Guide
lang: en
tags: [spec, mechanics, guide]
library-id: mechanics.guide
status: spec
---

🌐 **EN** · [[S08 Guía lateral|ES]] · [[S08 Seitenführung|DE]] · [[Home (EN)|Index]]

# S08 Side Guide (`mechanics.guide`)

Integrated in every conveyor (`guides` group) and also available as a standalone component for extra sections. Theory: [[T14 Stops, Impacts and Alignment]] §6.

## 1. Parameters

| Key                  | Plan key          | Unit | Default | Range                   | Description                                 |
| -------------------- | ----------------- | ---- | ------- | ----------------------- | ------------------------------------------- |
| `side`               | `lado`            | enum | both    | left, right, both, none | Sides with a guide                          |
| `guideHeight`        | `alturaGuia`      | mm   | 150     | 50 … 400                | Height above the conveying surface          |
| `clearance`          | `holgura`         | mm   | 25      | 0 … 100                 | Clearance per side between pallet and guide |
| `startPct`, `endPct` | `sInicio`, `sFin` | %    | 0, 100  | 0 … 100                 | Covered part of the length                  |
| `funnelEntry`        | `entradaEmbudo`   | bool | no      | —                       | Adds a diverging entry                      |
| `funnelAngle`        | `anguloEmbudo`    | °    | 10      | 3 … 30                  | Entry angle                                 |
| `funnelLength`       | `longitudEmbudo`  | mm   | 300     | 100 … 800               | Entry length                                |
| `guideMu`            | `muGuia`          | —    | 0.30    | 0.1 … 0.6               | Friction with the pallet                    |
| `guideRestitution`   | `restitucionGuia` | —    | 0.1     | 0 … 0.5                 | Lateral rebound                             |

## 2. How pallets are aligned

- The **stop** fixes the longitudinal position of the front edge; one contact point lets the pallet rotate, a wide stop or two stops square it.
- **Guides** fix the lateral position; the funnel corrects misaligned entries (capture ≈ clearance + funnelLength · tan(funnelAngle) = 25 + 53 mm with defaults).
- In 2D only the ideal final position is applied (guides are a clearance validation); in 3D the real pose is obtained — the measure of alignment quality.

## 3. Behaviour

**2D.** Clearance validation only. **3D.** Static colliders with friction and restitution.

## 4. Tests

| Test       | Criterion                                             | Status                                                                                      |
| ---------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| S1-d / P9  | 50 mm and 5° misalignment corrected to ±10 mm and ±1° | ✅ with polyethylene strips; painted steel up to 3° ([[ADR-0002 Physics Engine Candidate]]) |
| Validation | Usable width ≥ pallet + 2 × clearance                 | ✅ `usableWidthFitsPallet`                                                                  |

## Related

[[T14 Stops, Impacts and Alignment]] · [[S07 Mechanical Stop]]
