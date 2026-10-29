---
title: T14 Stops, Impacts and Alignment
lang: en
tags: [theory, physics, stops, guides]
---

🌐 **EN** · [[T14 Topes, impactos y alineación|ES]] · [[T14 Anschläge, Aufprall und Ausrichtung|DE]] · [[Home (EN)|Index]]

# T14 Stops, Impacts and Alignment

## 1. Impact energy — test T5

$$E = \tfrac12\,m\,v^2$$

| Mass (pallet + load) | 0.10 m/s (creep) | 0.25 m/s | 0.50 m/s |
| -------------------- | ---------------- | -------- | -------- |
| 1025 kg              | 5.1 J            | **32 J** | 128 J    |

Energy grows with the **square** of speed: approaching at creep speed (0.10 m/s) reduces impact energy six-fold compared with 0.25 m/s. This is why `approachSpeed` exists ([[T04 Conveyor Kinematics]] §6). The simulator records speed and energy of every impact and warns above `maxImpactEnergy` (default 200 J).

## 2. Impulse, contact time and peak force

The impulse delivered by the stop is $J = m\,v\,(1 + e)$. The average force depends on how long the contact lasts:

$$\bar{F} = \frac{J}{\Delta t_{contact}}, \qquad \bar{a} = \frac{v(1+e)}{\Delta t_{contact}}$$

For 1025 kg at 0.25 m/s with $e = 0.1$: $J = 282$ N·s. With a 10 ms contact: $\bar F \approx 28$ kN and $\bar a \approx 27.5$ m/s².

**Caveat for rigid-body engines:** a rigid stop stops the pallet within about **one physics step** (8 ms), so peak forces are an artefact of the time step, not a prediction. Real stops have rubber buffers or dampers. Use **energy** for decisions, not the simulated peak force.

### Will a tall load tip at the stop?

A quick energy test: the pallet tips about its front-bottom edge if its kinetic energy can lift the COM over that edge:

$$\Delta E_{tip} = m\,g\,\Big(\sqrt{h^2 + (L/2)^2} - h\Big)$$

EUR with 600 kg load ($h = 0.717$ m, $L/2 = 0.4$ m short side forward): $\Delta E_{tip} = 625 \cdot 9.81 \cdot (0.821 - 0.717) \approx 638$ J ≫ 32 J → no tipping. (Rough: only part of the energy goes into rotation; use as an upper-bound screening.)

## 3. Rebound

With restitution $e$ the pallet leaves the stop at $e\,v$. Rollers that keep driving forward bring it back within $\approx (e v)^2 / (2\mu g)$ — 0.09 mm for the default values ([[T11 Contact, Friction and Restitution]] §6). Criterion P1: front within **±5 mm** of the theoretical position and rebound **< 10 mm**.

## 4. 2D model of a stop

- The stop is a **position limit** for the front: the predicted event `front_reaches_stop` ends the pallet's motion segment at $s_{stop}$.
- Rollers: the pallet stops at the stop position (instantaneous in the ideal model).
- Chains: the pallet decelerates over `stopContactTime` (default 0.2 s) and the chains keep sliding underneath.
- `raiseTime` / `lowerTime` (0.3 s) delay the effect of a command. A stop must **not rise under a pallet**: interlock "stop up only if the zone above the stop is clear".

## 5. Alignment with stops

A stop fixes the **longitudinal** position of the front edge. The geometry of the contact decides whether it also fixes the **yaw**:

- **One narrow contact off the pallet axis:** the impact impulse $J$ applied at lateral distance $d$ from the COM produces an angular impulse $J d$; the pallet yaws ($I\,\Delta\omega = J d$) until a second contact or a guide stops it. Friction with the rollers resists the rotation, so the final angle depends on μ, mass and inertia — a 3D-only result.
- **A wide stop or two stops:** contacts on both sides of the COM balance the torque and the front edge is squared against the stop. That is why `stopWidth` matters (default 100 mm; wider aligns better).

In 2D only the ideal final position is applied; the 3D mode delivers the real pose (position + angle), which is the measure of alignment quality.

## 6. Side guides and funnels

### 6.1 Clearance

Usable width ≥ pallet width + 2 × clearance: EUR 800 mm + 2 × 25 mm = 850 mm ≤ 1000 mm ✓ (`usableWidthFitsPallet`, tested).

### 6.2 Funnel capture range

A funnel of length $L_f$ and angle $\theta_f$ widens the entry on each side by $L_f \tan\theta_f$: 300 mm at 10° → 53 mm. It catches pallets offset by up to clearance + 53 mm ≈ 78 mm (criterion S1-d / P3: 50 mm and 5° corrected to ±10 mm and ±1°).

### 6.3 The wedge: drive force becomes lateral push

The rollers push the pallet forward with force $F_d$ into the inclined guide. With guide friction $\mu_g$, the contact normal force $N$ must balance the forward push: $F_d = N(\sin\theta + \mu_g\cos\theta)$, and the lateral push is about $N\cos\theta$:

$$F_{lat} \approx \frac{F_d}{\tan\theta + \mu_g}$$

At 10° with $\mu_g = 0.3$: $F_{lat} \approx 2.1\,F_d$. The pallet must also **slide sideways on the rollers** (rollers roll only along travel), which costs up to $\mu_r m g$. Because roller friction is one **friction cone**, forward drive and lateral sliding share the same budget $\mu_r m g$ — the pallet moves diagonally. Steep funnels (large θ) or high guide friction reduce $F_{lat}$ and can **jam** the pallet. The outcome is a 3D test, not a formula; small funnel angles (≤ 10°) are the safe default.

## Related

[[S07 Mechanical Stop]] · [[S08 Side Guide]] · [[T11 Contact, Friction and Restitution]] · [[T09 Rigid Body Dynamics]]
