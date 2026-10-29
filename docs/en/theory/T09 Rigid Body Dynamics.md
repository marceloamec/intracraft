---
title: T09 Rigid Body Dynamics
lang: en
tags: [theory, physics]
---

🌐 **EN** · [[T09 Dinámica del sólido rígido|ES]] · [[T09 Starrkörperdynamik|DE]] · [[Home (EN)|Index]]

# T09 Rigid Body Dynamics

## 1. State of a rigid body

A rigid body does not deform; its full state is

| Symbol              | Meaning                        | Notes                            |
| ------------------- | ------------------------------ | -------------------------------- |
| $\mathbf{x}$        | Position of the centre of mass | metres, world frame (Y up)       |
| $\mathbf{q}$        | Orientation                    | Unit quaternion (no gimbal lock) |
| $\mathbf{v}$        | Linear velocity                | m/s                              |
| $\boldsymbol\omega$ | Angular velocity               | rad/s                            |

plus constant **mass properties**: mass $m$, centre of mass (COM) and inertia tensor $\mathbf{I}$.

## 2. Equations of motion (Newton–Euler)

$$m\,\dot{\mathbf{v}} = \mathbf{F} \qquad \mathbf{I}\,\dot{\boldsymbol\omega} + \boldsymbol\omega \times (\mathbf{I}\,\boldsymbol\omega) = \boldsymbol\tau$$

$\mathbf{F}$ is the sum of forces (gravity $m\mathbf{g}$ + contact forces), $\boldsymbol\tau$ the sum of torques about the COM. The inertia tensor in world coordinates is $\mathbf{I}_w = \mathbf{R}\,\mathbf{I}_b\,\mathbf{R}^T$, with $\mathbf{R}$ the rotation matrix of $\mathbf{q}$. The term $\boldsymbol\omega \times \mathbf{I}\boldsymbol\omega$ (gyroscopic) is negligible for slowly turning pallets but the engine includes it.

In practice the engine does not apply contact forces directly; it solves for **impulses** that satisfy the contact constraints ([[T11 Contact, Friction and Restitution]]).

## 3. Three motion types

| Type      | Moved by                 | Affected by forces | Used for                                                 |
| --------- | ------------------------ | ------------------ | -------------------------------------------------------- |
| Static    | Nothing                  | No                 | Guides, frames, floor                                    |
| Kinematic | Prescribed velocity/pose | No (infinite mass) | Rollers, chains, turntable platforms, stops, lift chains |
| Dynamic   | Forces and contacts      | Yes                | Pallets                                                  |

A kinematic roller "spins" with a set angular velocity; it has no inertia from the solver's point of view, so a 1000 kg pallet cannot slow it down. The motor is assumed strong enough — torque limits are a validation, not a simulation, in the MVP.

## 4. Inertia of the shapes we need

| Shape                                 | Axis                                | Moment of inertia           |
| ------------------------------------- | ----------------------------------- | --------------------------- |
| Solid box $a \times b \times c$       | Through COM, ⟂ to $a \times b$ face | $\frac{m}{12}(a^2 + b^2)$   |
| Solid cylinder radius $r$, length $h$ | Own axis                            | $\frac12 m r^2$             |
| Solid cylinder                        | Transverse, through COM             | $\frac{m}{12}(3r^2 + h^2)$  |
| Tube (roller shell) $r_o, r_i$        | Own axis                            | $\frac12 m (r_o^2 + r_i^2)$ |

**Parallel-axis theorem** — about an axis at distance $d$ from the COM: $I = I_{cm} + m\,d^2$. It is how the inertia of a compound body is assembled from its parts.

## 5. The compound pallet

The pallet is **not** one box. It is a dynamic body made of:

- 3 boxes for the bottom boards (the only parts that touch rollers or chains);
- 1 box for the load block (with load height and mass);
- optionally the top deck and blocks for sensor rays (or the occupancy profile drives collider generation — [[S10 Pallet Types]]).

The voids between bottom boards matter: a chain that falls into a void does not lift the pallet, and a sensor beam can pass between blocks.

### Worked example: EUR pallet with a 600 kg load, 1200 mm high

| Part      | Mass (kg) | COM height (m)                                               | Yaw inertia about own COM (kg·m²)   |
| --------- | --------- | ------------------------------------------------------------ | ----------------------------------- |
| Pallet    | 25        | 0.072                                                        | $25(1.2^2 + 0.8^2)/12 = 4.3$        |
| Load      | 600       | $0.144 + 0.6 = 0.744$                                        | $600(1.2^2 + 0.8^2)/12 = 104$       |
| **Total** | **625**   | $\dfrac{25 \cdot 0.072 + 600 \cdot 0.744}{625} = $ **0.717** | **≈ 108** (both centred, no offset) |

The `centerOfMassOffset` parameter shifts the load COM horizontally to model off-centre loads.

## 6. Tipping vs sliding — why the COM height matters

During an acceleration $a$ along travel, the inertial force $m a$ acts at the COM height $h$. Taking moments about the rear edge of the support footprint (half-length $L/2$):

- The pallet **tips** if $a\,h > g\,(L/2)$, i.e. $a_{tip} = g\,\dfrac{L/2}{h}$.
- It **slides** first if $\mu < \dfrac{L/2}{h}$ (friction saturates before tipping).

Example (short side 0.8 m along travel, $h = 0.717$ m): $a_{tip} = 9.81 \times 0.4/0.717 = 5.5$ m/s². Normal ramps (0.5 m/s²) are far below, but the **deceleration at a stop impact** can be much larger for a few milliseconds; tall loads therefore need low approach speeds ([[T14 Stops, Impacts and Alignment]]).

## 7. Damping

Small linear and angular damping (≈ 0.01–0.05 s⁻¹) remove numerical jitter. They must stay small: damping is an artificial force and changes transit times. Any change must be recorded in a decision note (AGENTS.md rule).

## 8. MVP simplifications

- The load is rigidly attached to the pallet (no load shifting, no deformation).
- Rollers and chains are kinematic: no motor dynamics, no belt stretch.
- Wood is rigid; board deflection between rollers is ignored.

## Related

[[T10 Time Integration and Fixed Step]] · [[T11 Contact, Friction and Restitution]] · [[T12 Rollers and Chains in Physics]] · [[A07 Physics Engine Wrapper]]
