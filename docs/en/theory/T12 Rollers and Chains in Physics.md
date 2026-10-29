---
title: T12 Rollers and Chains in Physics
lang: en
tags: [theory, physics, conveyors]
---

🌐 **EN** · [[T12 Rodillos y cadenas en física|ES]] · [[T12 Rollen und Ketten in der Physik|DE]] · [[Home (EN)|Index]]

# T12 Rollers and Chains in Physics

## 1. Driven rollers

A driven roller is a **kinematic cylinder** spinning about its axis at

$$\omega = \frac{v}{r} = \frac{v}{D/2}$$

Ø89 mm (project default) at 0.25 m/s → $\omega = 0.25/0.0445 = 5.62$ rad/s; Ø76 mm → 6.58 rad/s (`rollerAngularVelocity`, tested). The top of the roller moves at $\omega r = v$; friction at the board–roller contact drives the pallet towards that speed ([[T11 Contact, Friction and Restitution]]).

Ramps are produced by the **wrapper**, not by the engine: every step it sets each zone's $\omega(t)$ from the same trapezoidal profile the 2D mode uses ([[T04 Conveyor Kinematics]]).

### Traction limit

With all supporting rollers driven, friction can accelerate the pallet by at most

$$a_{max} = \mu\,g$$

$\mu = 0.35$ → 3.43 m/s² (`maxTractionAccel`, tested). The default ramp of 0.5 m/s² is 7× below, so the pallet follows the rollers with only micro-slip. Slip appears when: μ is low (wet, plastic pallets), only part of the rollers is driven, the pallet is held (stop, queue) or the ramp is too steep.

### Support and the 3-roller rule

A pallet side of length $L$ rests on $\lfloor L/p \rfloor$ rollers (pitch $p$): 1200 mm at 206.3 mm → 5 rollers (at 133 mm: 9). At least **3** must always be under each supported side so the pallet never tips into a gap and bridges the gap between conveyors. The load distribution among rollers is statically indeterminate; the engine resolves it, and only the total matters for friction.

### Idle (gravity) rollers

Without slope (MVP), idle rollers only let a pushed pallet pass. Simulating each as a free dynamic body on a hinge is expensive. **Proposed MVP model:** idle rollers become a static support with low friction equal to the rolling resistance coefficient ($C_{rr} \approx 0.02$): resisting force $C_{rr}\,m\,g$.

### Locked rollers (turntables)

On a roller turntable the rollers are locked during the turn ($\omega = 0$). The contact is then **sliding**, not rolling, with full $\mu$ — which is what holds the pallet ([[T13 Turntables and Transfer Lifts]]). Free rollers would let the pallet roll off during angular acceleration.

## 2. Chains: the hard part

A real chain conveyor has 2–6 strands; each carrying strand slides on a wear strip and the pallet rests on the chain's links or rollers. EUR pallets travel **short side forward** on chains (confirmed by the owner), so the bottom boards are *perpendicular* to the chains and each board touches each strand over its chain width (40 mm).

The difficulty: the contact surface moves (the chain), but the body underneath (the rail) does not. A physics engine needs either many moving bodies or a contact that "knows" its surface velocity.

### The four plans

| Plan | Model                                                                  | Pros                              | Cons                                                                             |
| ---- | ---------------------------------------------------------------------- | --------------------------------- | -------------------------------------------------------------------------------- |
| A    | Strip of small kinematic rollers (Ø 30 mm every 40 mm), $\omega = v/r$ | Works in any engine; no hooks     | Many bodies (3 strands × 75 rollers per 3 m = 225); bumpy contact; more contacts |
| B    | Real **surface velocity** on the contact                               | One box per strand; smooth; cheap | Needs engine support in the JS binding                                           |
| C    | Drag force $F = \text{clamp}(k\,m\,(v_c - v_p),\ \pm\mu N)$            | Simple                            | $N$ must be estimated; not a true contact model                                  |
| D    | Own Rust physics module (WASM) using `modify_solver_contacts`          | Full control                      | Adds Rust and a build chain; last resort                                         |

Plan A with Ø30 mm at 0.25 m/s: $\omega = 0.25/0.015 = 16.7$ rad/s.

### Research result (2026-10-28)

- **Jolt** (`jolt-physics` 1.1.1 on npm): the TypeScript typings expose `ContactListenerJS` with `OnContactAdded` / `OnContactPersisted(body1, body2, manifold, ioSettings)` and `ContactSettings.mRelativeLinearSurfaceVelocity` / `mRelativeAngularSurfaceVelocity`. **Plan B appears feasible with Jolt** — the spike must confirm sign conventions, per-contact cost of a JS callback and determinism.
- **Rapier** (`@dimforge/rapier3d-compat` 0.21.0): the JS `PhysicsHooks` interface only has `filterContactPair` and `filterIntersectionPair` (`ActiveHooks`: `FILTER_CONTACT_PAIRS`, `FILTER_INTERSECTION_PAIRS`). **No solver-contact modification → plan B is not available**; Rapier would use plan A (or D).

Details in [[T15 Physics Engine Research]] and [[ADR-0002 Physics Engine Candidate]].

### How surface velocity works (plan B)

The solver normally drives the relative tangential velocity at the contact to zero (stick). With a surface velocity $\mathbf{u}$ the target becomes $\mathbf{v}_{rel,t} = \mathbf{u}$: the static rail behaves as if its surface moved at the chain speed. The friction limit $\mu N$ still applies, so a held pallet slips exactly like on a real chain.

## 3. Accumulation with slip

Under a held pallet, the chain (plan A rollers or plan B surface) keeps moving: kinetic friction pushes the pallet forward with $\mu\,m\,g$ (per pallet), and pushes add up along a queue ([[T05 Accumulation, Capacity and Throughput]] §6.3). Both plans reproduce this naturally; the 3D mode then reports the force on the stop.

## 4. Polygon effect

A chain wrapped around a sprocket with $z$ teeth does not move at constant speed: the speed oscillates with relative amplitude $1 - \cos(\pi/z)$ — 1.7 % for $z = 17$. The parameter `chainPitch` is reserved for it; the MVP ignores the effect (constant chain speed).

## 5. Acceptance tests (spike S1)

| Test | Setup                                  | Criterion                                                 |
| ---- | -------------------------------------- | --------------------------------------------------------- |
| a    | 1000 kg pallet on 10 kinematic rollers | Reaches nominal speed ±3 %, brakes without excessive slip |
| b    | Strip of small rollers as a chain      | Pallet speed = chain speed ±3 %, no visible vibration     |
| f    | 20 conveyors, 10 pallets               | 60 FPS, no numerical instabilities                        |

## Related

[[T11 Contact, Friction and Restitution]] · [[T13 Turntables and Transfer Lifts]] · [[S01 Roller Conveyor]] · [[S02 Chain Conveyor]] · [[T15 Physics Engine Research]]
