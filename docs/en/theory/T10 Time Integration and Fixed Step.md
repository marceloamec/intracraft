---
title: T10 Time Integration and Fixed Step
lang: en
tags: [theory, physics, numerics]
---

🌐 **EN** · [[T10 Integración temporal y paso fijo|ES]] · [[T10 Zeitintegration und fester Zeitschritt|DE]] · [[Home (EN)|Index]]

# T10 Time Integration and Fixed Step

> **Rust platform (2026-10-31):** the fixed step and clock described here carry over to `intracraft-physics` on Rapier; the Jolt-specific settings are re-tuned in phase R5 of [[A19 Rust Implementation Plan]].

## 1. From equations to steps

The physics engine cannot solve Newton–Euler equations ([[T09 Rigid Body Dynamics]]) in closed form once contacts are involved. It advances the state in small steps $\Delta t$.

| Integrator              | Update                                                          | Behaviour                                                 |
| ----------------------- | --------------------------------------------------------------- | --------------------------------------------------------- |
| Explicit Euler          | $x \mathrel{+}= v\,\Delta t$; then $v \mathrel{+}= a\,\Delta t$ | Gains energy, oscillators blow up                         |
| **Semi-implicit Euler** | $v \mathrel{+}= a\,\Delta t$; then $x \mathrel{+}= v\,\Delta t$ | Symplectic: energy stays bounded; used by Rapier and Jolt |
| Runge–Kutta 4           | Four evaluations per step                                       | Accurate for smooth ODEs, useless across contacts         |

Semi-implicit Euler is first-order accurate but **stable** for a harmonic oscillator as long as $\omega_n\,\Delta t < 2$ — and with contacts, stability matters far more than order.

## 2. Why the step must be fixed

1. **Determinism** — a different $\Delta t$ gives different contact impulses and therefore different results. A frame-rate-dependent step would make every run unique.
2. **Stability** — solver parameters (stiffness, restitution handling) are tuned for one $\Delta t$.
3. **Reproducible tests** — S1-g requires identical state hashes after 60 s in two runs.

## 3. The accumulator pattern

The screen refreshes at its own rate (60 Hz, 144 Hz, or irregularly). The simulation decouples from it:

```text
accumulator += frameTime
steps = floor(accumulator / dt)
if steps > maxSteps:            # "spiral of death" guard
    steps = maxSteps; accumulator = 0
else:
    accumulator -= steps · dt
repeat steps times: world.step(dt)
alpha = accumulator / dt        # 0 ≤ alpha < 1
render state = lerp(previous, current, alpha)   # slerp for rotations
```

Implemented as `FixedStepClock` in `packages/sim-physics/src/fixed-step.ts` (tested). The **interpolation** removes the stutter that appears when the number of physics steps per frame alternates (e.g. 2, 2, 3, 2…). Rendering is up to one step behind the physics — invisible at 120 Hz.

If the computer cannot keep up, the guard drops the backlog: simulated time then runs slower than real time but stays correct.

## 4. Choosing $\Delta t = 1/120$ s

Rule of thumb: the distance a body moves per step must be much smaller than the smallest geometric feature it interacts with.

| Speed            | Displacement per step | Compare with                                     |
| ---------------- | --------------------- | ------------------------------------------------ |
| 0.10 m/s (creep) | 0.83 mm               | Stop tolerance ±5 mm                             |
| 0.25 m/s         | 2.1 mm                | Bottom board 22 mm thick, chain 40 mm wide       |
| 0.60 m/s (max)   | 5.0 mm                | Roller gap 57 mm, stop face — use CCD for safety |

At 0.25 m/s a pallet crosses one roller pitch (133 mm) in 64 steps: the contact pattern changes smoothly. A board approaching a stop moves 0.83–2.1 mm per step, consistent with the ±5 mm criterion of S1-c.

## 5. Solver iterations and substeps

Each step, the constraint solver iterates over all contacts several times ([[T11 Contact, Friction and Restitution]]):

- **Jolt**: velocity iterations (default 10) and position iterations (default 2) per step.
- **Rapier**: an iterative solver with internal substeps (TGS-soft style) and configurable iteration counts.

More iterations → stiffer, more accurate contacts (heavy pallets resting on many rollers) at linear cost. The spike S1 measures how many are needed for test a (1000 kg on 10 rollers).

## 6. Controller cycle vs physics step

The controller runs every 10 ms; physics every 8.33 ms. They are not multiples, so the controller executes at the first step after each 10 ms boundary, with up to 8.3 ms jitter. Options (open decision for S1):

1. Accept the jitter (covered by the ±50 ms sensor budget, [[T06 Photoelectric Sensing]]).
2. Use $\Delta t = 1/100$ s or $1/200$ s so control cycles align with steps.

## 7. Sleeping — a classic pitfall

Engines put bodies at rest to **sleep** to save time. A pallet waiting on a stopped conveyor will fall asleep. When the conveyor restarts, the kinematic rollers change velocity, but a sleeping pallet is not necessarily woken. **The wrapper must wake every pallet in contact with a conveyor whose speed changes.** Forgetting this produces pallets that "refuse" to move.

## Related

[[T09 Rigid Body Dynamics]] · [[T11 Contact, Friction and Restitution]] · [[T03 Time, Event Ordering and Determinism]] · [[A07 Physics Engine Wrapper]]
