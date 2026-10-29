---
title: T11 Contact, Friction and Restitution
lang: en
tags: [theory, physics, contact]
---

🌐 **EN** · [[T11 Contacto, rozamiento y restitución|ES]] · [[T11 Kontakt, Reibung und Restitution|DE]] · [[Home (EN)|Index]]

# T11 Contact, Friction and Restitution

Contacts are where conveyor physics happens: rollers drive the pallet only through friction at their contact points, and stops and guides act only through contacts.

## 1. The collision pipeline (every step)

1. **Broad phase** — cheap bounding-box tests (bounding volume hierarchies or sweep-and-prune) find pairs that *might* touch.
2. **Narrow phase** — exact tests (GJK/EPA, SAT for boxes) compute a **contact manifold**: up to 4 points, a normal $\mathbf{n}$ and penetration depths. A box board on a cylinder roller gives a line contact, typically 2 points.
3. **Constraint solving** — impulses are computed so bodies do not penetrate, friction is respected and restitution is applied.
4. **Integration** — velocities and positions are updated ([[T10 Time Integration and Fixed Step]]).

## 2. Non-penetration

At each contact point, with relative normal velocity $v_n$ and normal impulse $\lambda_n$:

$$v_n \ge 0, \qquad \lambda_n \ge 0, \qquad \lambda_n\,v_n = 0$$

(the bodies separate or stay in contact; the contact can push but never pull; it only pushes while touching). This *complementarity* problem is solved iteratively.

## 3. Coulomb friction

The tangential force is limited by the normal force:

$$\|\mathbf{F}_t\| \le \mu\,F_n$$

- **Stick** — if friction can cancel the relative tangential velocity within that limit, the surfaces move together.
- **Slip** — otherwise the friction force has magnitude $\mu F_n$ and opposes the relative sliding velocity.

The set of admissible contact forces is the **friction cone** (half-angle $\arctan\mu$); engines approximate it with a pyramid or by clamping the two tangent impulses.

**Key point for conveyors:** friction acts on the **relative** velocity between the pallet and the *surface* at the contact point. The surface of a kinematic roller spinning at $\omega$ moves at $\omega r$ at its top; friction pushes the board towards that velocity. That is the whole mechanism of a driven roller conveyor ([[T12 Rollers and Chains in Physics]]).

**Static vs kinetic.** Real materials have $\mu_s > \mu_k$. Both Rapier and Jolt use a single coefficient per collider. IntraCraft's friction table gives one value per material pair ([[S10 Pallet Types]]).

### Combining coefficients

Each collider has its own friction; the engine combines the two:

| Engine | Default combination                        | How to get the exact table value $\mu_{pair}$                     |
| ------ | ------------------------------------------ | ----------------------------------------------------------------- |
| Rapier | Average (configurable: min, max, multiply) | Give both colliders $\mu_{pair}$, or choose the rule per collider |
| Jolt   | Geometric mean $\sqrt{\mu_1\mu_2}$         | Override `mCombinedFriction` per contact in the contact listener  |

(Defaults checked on 2026-10-28 against `@dimforge/rapier3d-compat` 0.21.0 — `frictionCombineRule` = Average, default friction 0.5 — and `jolt-physics` 1.1.1; see [[T15 Physics Engine Research]].)

## 4. How the solver works: sequential impulses

Most real-time engines use **projected Gauss–Seidel / sequential impulses**: loop over all contacts several times; for each, compute the impulse that would fix its velocity error, add it to an **accumulated** impulse and clamp the accumulated value ($\lambda_n \ge 0$, $|\lambda_t| \le \mu\lambda_n$). **Warm starting** reuses last step's impulses so a pallet resting on rollers converges in few iterations. Heavy bodies on many contacts need more iterations — measured in S1-a.

## 5. Penetration correction

Numerical errors leave bodies slightly overlapping. Engines push them apart gently (Baumgarte terms, split impulses, or soft constraints) and allow a small **slop** (tolerated penetration) to avoid jitter.

> ⚠️ The slop directly affects the ±5 mm stop criterion. Measured defaults: **Jolt** `mPenetrationSlop` = 0.02 m (and `mSpeculativeContactDistance` = 0.02 m); **Rapier** `normalizedAllowedLinearError` = 0.005 m (× `lengthUnit` = 1). Both are at or above the tolerance, so IntraCraft must lower them to ≈ 1 mm or compensate. Rapier also exposes a per-collider **contact skin**.

## 6. Restitution (bounce)

The coefficient of restitution relates separation and approach speeds along the normal:

$$e = -\frac{v_n^{after}}{v_n^{before}}, \qquad 0 \le e \le 1$$

$e = 0$: perfectly plastic (no bounce); $e = 1$: perfectly elastic. Stops default to $e = 0.1$.

> ⚠️ **Velocity threshold.** Many engines ignore restitution below a minimum approach speed (Jolt: `mMinVelocityForRestitution`, default **1 m/s**, measured) to avoid endless micro-bounces. Pallet impacts happen at 0.1–0.25 m/s, so restitution would be silently ignored unless the threshold is lowered.

**Expected rebound.** With $e = 0.1$ at 0.25 m/s the pallet bounces back at 0.025 m/s; rollers still driving forward stop it within $v^2/(2\mu g) = 0.025^2/(2 \cdot 0.35 \cdot 9.81) \approx 0.09$ mm. The < 10 mm criterion is easy unless friction is very low.

## 7. Continuous collision detection (CCD)

A body **tunnels** through an obstacle if it moves more than the obstacle's thickness in one step. At 0.6 m/s and 120 Hz the displacement is 5 mm — less than a stop face, but CCD is enabled on pallets as a safety net, as the plan prescribes.

## 8. Performance: count the contacts

A pallet with 3 bottom boards over 9 rollers creates up to 27 board–roller pairs, each with ~2 points. Ten pallets → ~500 contact points per step, 60 000 per second at 120 Hz. This is why rollers are grouped, bodies sleep when idle, and why the 20-conveyor / 10-pallet / 60 FPS criterion (S1-f) is a real test.

## Related

[[T12 Rollers and Chains in Physics]] · [[T14 Stops, Impacts and Alignment]] · [[T15 Physics Engine Research]]
