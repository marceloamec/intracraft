---
title: T13 Turntables and Transfer Lifts
lang: en
tags: [theory, des, physics, turntable, transfer]
---

🌐 **EN** · [[T13 Giratorias y mesas de transferencia|ES]] · [[T13 Drehtische und Kettenumsetzer|DE]] · [[Home (EN)|Index]]

# T13 Turntables and Transfer Lifts

Both components change the direction of flow by 90° (or 180°), handle **one pallet at a time** and are usually the bottlenecks of a layout ([[T05 Accumulation, Capacity and Throughput]] §3).

## Part 1 — Turntables

### 1.1 Angular profile

Turning by $\theta$ with maximum speed $\omega$ and ramps $\alpha$ uses the trapezoidal/triangular formulas of [[T04 Conveyor Kinematics]] §4: 90° at 15 °/s with 30 °/s² takes **6.5 s** (T3).

**Choosing the direction.** With `shortestPath`, the signed angle is
$$\Delta\theta = \big((\theta_{target} - \theta_{now} + 180°) \bmod 360°\big) - 180°$$
With a limited `angularRange` (cables), the turntable must take the allowed direction even if longer.

### 1.2 Stations and ports

Ports are defined in the **platform frame**. A port only counts as connected when the platform angle equals a station angle within `angularPrecision` (default 0.2°). In 2D the pallet simply travels with the platform; at `turn_completed` + `settleTime` the station's port is enabled for discharge.

### 1.3 Swept radius

$$R_{sweep} = \tfrac12\sqrt{L_p^2 + W_p^2} + \text{clearance}$$

1400 × 1200 mm with 50 mm clearance → **972 mm** (`sweptRadius`, tested). The validation checks that no neighbour lies inside this circle.

### 1.4 Will the pallet slide during the turn?

A point at radius $r$ on a rotating platform has tangential acceleration $a_t = \alpha r$ and centripetal acceleration $a_c = \omega^2 r$. Friction must supply both:

$$\mu \ge \frac{\sqrt{a_t^2 + a_c^2}}{g}$$

Take $r$ at the farthest pallet corner: EUR centred, $r = \tfrac12\sqrt{1.2^2 + 0.8^2} = 0.72$ m. With defaults ($\alpha = 30$ °/s² = 0.524 rad/s², $\omega = 15$ °/s = 0.262 rad/s): $a_t = 0.377$ m/s², $a_c = 0.049$ m/s² → **μ ≥ 0.039** (`turntableRequiredFriction`, tested). Available μ is 0.30–0.35: a wide margin. It shrinks with steeper ramps, larger pallets and off-centre positions (centring tolerance ±20 mm).

(Conservative: $a_t$ is maximal during the ramp while $a_c$ is maximal at full speed; combining both peaks overestimates the need.)

### 1.5 Drive torque

$$\tau = I_{total}\,\alpha + \tau_{bearing}$$

Illustrative: platform 400 kg, 1.4 × 1.2 m → $I = 400(1.4^2 + 1.2^2)/12 = 113$ kg·m²; loaded EUR (625 kg) ≈ 108 kg·m² ([[T09 Rigid Body Dynamics]]); total ≈ 221 kg·m². With $\alpha = 0.524$ rad/s² and 20 N·m bearing friction: $\tau \approx 136$ N·m, far below the 1500 N·m default (`turntableRequiredTorque`). An off-centre pallet adds $m d^2$ (parallel-axis theorem).

### 1.6 Physics model

- Platform: kinematic body with angular velocity $\omega(t)$ about the vertical axis.
- Its rollers or chains are separate kinematic bodies: every step the wrapper computes their pose from the platform angle and sets their kinematic targets so that their velocities are consistent (a body teleported without velocity does not drag the pallet).
- Roller turntable: rollers **locked** during the turn ($\omega_{roller} = 0$) so the pallet is held by sliding friction ([[T12 Rollers and Chains in Physics]]).
- Test S1-e / P4: the pallet moves less than 5 mm relative to the platform over a 90° turn.

### 1.7 Sequence

waiting → receiving (conveyor running) → centred (sensors) → turning → settling → discharging → waiting. Interlock: no turn unless the pallet is centred and fully on the platform.

## Part 2 — Roller-to-chain transfer table

### 2.1 Mechanism (interpretation to be confirmed, plan §18)

A roller section receives the pallet **longitudinally**. Between the rollers sit **liftable chain strands**, oriented transversely. When lifted, the chains raise the pallet off the rollers, move it 90° sideways onto the adjacent chain line and lower. The pallet changes direction without rotating.

```text
     top view                           side view (section across rollers)
  ┌─────────────────────┐
  │ ║ │ ║ │ ║ │ ║ │ ║ │ │  ← rollers (│) and chains (║) in the gaps     pallet ▄▄▄▄▄▄
  │ ║ │ ║ │ ║ │ ║ │ ║ │ │                                       raised: ║ ↑ 40 mm
  └─────────────────────┘                                       rollers: ○ ○ ○ ○
     travel on rollers →     transfer on chains ↓
```

### 2.2 Geometric checks

- **Chains fit between rollers:** gap $= p - D \ge w_{chain} + 10$ mm. Ø89 at 206.3 mm pitch (project values) → 117.3 mm ≥ 40 + 10 ✓; the plan's Ø76 at 133 mm → 57 mm ✓ (`chainFitsBetweenRollers`, tested).
- **Chains touch the boards:** with the pallet long side along the rollers, its bottom boards run along the roller direction and every transverse chain crosses all of them ✓. If the pallet were rotated 90°, the boards would be parallel to the chains and a chain could fall **into a void** between boards — the simulator warns.
- **Height continuity:** raised chains are `liftStroke` (40 mm) above the roller plane, so the adjacent chain conveyor's surface must be at roller height + 40 mm (port tolerance ±10 mm).

### 2.3 Cycle time — test T4

$$t = t_{lift} + \frac{d}{v} + \frac{v}{a} + t_{lower}$$

1 s + (5 + 0.4) s + 1 s = **7.4 s** for 1 m at 0.2 m/s.

### 2.4 Lifting by contact

The chains are kinematic bodies moving up by `liftStroke` + `lowerClearance` (50 mm) in `liftTime`. They lift the pallet by pushing on the boards. If the chains stopped instantly at the top, the pallet would continue upward and rise by $v^2/2g$; at 0.05 m/s that is 0.13 mm — negligible. A pneumatic actuator has end cushioning; an electric one has ramps (`actuator` parameter).

Test P5: lifted 40 ± 5 mm, moved 1 m ± 10 mm, set down with < 1° rotation.

### 2.5 Interlocks

No lift without a centred pallet (stop + sensors, tolerance ±20 mm) · rollers stopped while chains are up · do not lower until the pallet has left the chains · in bidirectional mode never accept from both sides.

## Related

[[T04 Conveyor Kinematics]] · [[T07 Control Logic and Signals]] · [[S03 Chain Turntable]] · [[S04 Roller Turntable]] · [[S05 Roller-to-Chain Transfer]]
