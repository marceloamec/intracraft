---
title: T06 Photoelectric Sensing
lang: en
tags: [theory, sensors, des, physics]
updated: 2026-10-29
---

🌐 **EN** · [[T06 Detección fotoeléctrica|ES]] · [[T06 Fotoelektrische Erfassung|DE]] · [[Home (EN)|Index]]

# T06 Photoelectric Sensing

## 1. Sensor types and their simulation model

| Type            | Real device                                                        | Simulation model                                  |
| --------------- | ------------------------------------------------------------------ | ------------------------------------------------- |
| Diffuse         | Emitter and receiver in one housing; light reflected by the object | Ray from the sensor in a direction, up to `range` |
| Retroreflective | Reflector opposite; the object **interrupts** the beam             | Segment sensor → reflector                        |
| Through-beam    | Separate emitter and receiver                                      | Segment emitter → receiver                        |

The MVP ignores colour, reflectivity and ambient light. The output is a **boolean signal** with selectable polarity.

## 2. Geometry: a beam between two user-chosen points

The user places the **sensor** at $E = (s_E, y_E, h_E)$ and the **reflector/receiver** at $R = (s_R, y_R, h_R)$ in the host conveyor's frame ($s$ along travel, $y$ lateral, $h$ height above the conveying surface). The beam is the segment

$$P(u) = E + u\,(R - E), \qquad u \in [0, 1]$$

The usual case is a transverse horizontal beam ($s_E = s_R$, $h_E = h_R$), but oblique or inclined beams are allowed ([[S06 Photoelectric Sensor]]). Height is the parameter that matters most, because a pallet is not a solid box.

## 3. The occupancy profile (2D mode)

Each pallet type describes, per **height band**, the solid intervals along its travel direction (from the front), separately for the two orientations: `alongLength` (long side first — roller conveyors) and `alongWidth` (short side first — chain conveyors). Full EUR tables: [[S10 Pallet Types]]. Summary:

| EUR band (mm)           | Long side first (rollers) | Short side first (chains) |
| ----------------------- | ------------------------- | ------------------------- |
| 0–22 bottom boards      | continuous                | gaps 227.5 mm             |
| 22–100 blocks           | gaps 382.5 mm             | gaps 227.5 mm             |
| 100–122 stringer boards | gaps 382.5 mm             | **continuous**            |
| 122–144 top deck        | **continuous**            | gaps ≈ 41 mm              |

Each band × interval is a box moving with the pallet: lateral extent = full pallet width across travel, height = band, along travel = $[s_{front} - b_k,\ s_{front} - a_k]$.

## 4. Predicting edges for any beam

For one solid box $k$ with lateral range $[y_0, y_1]$ and height range $[h_0, h_1]$ (fixed while the pallet moves) and along-travel interval $[a_k, b_k]$ from the front:

1. **Clip the segment** against the lateral and height slabs: the values of $u$ for which $y(u) \in [y_0, y_1]$ and $h(u) \in [h_0, h_1]$ form an interval $[u_0, u_1]$ (empty → this box never blocks the beam).
2. The clipped part of the beam spans along travel $[x_{min}, x_{max}]$ = the range of $s(u)$ for $u \in [u_0, u_1]$.
3. The box blocks the beam exactly when the two along-travel intervals overlap:

$$s_{front} \in [\,x_{min} + a_k,\; x_{max} + b_k\,]$$

So every box gives one interval of front positions — one rising and one falling edge — and the beam is blocked on the **union** of those intervals (overlapping ones merge). The DES engine schedules the edges with `timeToReach` ([[T04 Conveyor Kinematics]]). For a transverse horizontal beam $x_{min} = x_{max} = s_{sens}$ and the rule reduces to $s_{front} \in [s_{sens} + a_k,\ s_{sens} + b_k]$.

Why it is always one interval: the set of translations for which two convex shapes intersect is convex; along one axis that is an interval.

### Test T6

Beam in a continuous band at $s = 2.0$ m, $v = 0.25$ m/s, EUR long side first: rising edge when the front passes 2.0 m, falling edge when the tail passes it; duration $1.2/0.25$ = **4.8 s**.

## 5. The pallet-gap problem

| Case (EUR, 0.25 m/s)                 | Gap      | Signal off for            |
| ------------------------------------ | -------- | ------------------------- |
| Rollers, beam at 60 mm (block layer) | 382.5 mm | 1.53 s (twice per pallet) |
| Chains, beam at 60 mm (block layer)  | 227.5 mm | 0.91 s (twice per pallet) |
| Chains, beam at 130 mm (top deck)    | ≈ 41 mm  | ≈ 0.16 s (four times)     |

A controller counting pallets would count three (or five) per pallet. Remedies:

1. **Choose the beam height** in a continuous band for the orientation on that conveyor: ≈ 130 mm on rollers, ≈ 110 mm on chains (or above the pallet, on the load).
2. **Filter:** require the new state to persist for `filter` ms; to bridge a gap $t_{filter} > \text{gap}/v$. A filter also delays the **real** falling edge by the same time, which shifts zone releases and can cost throughput.

The editor warns when the chosen height falls in a band with voids ([[S06 Photoelectric Sensor]] §2).

## 6. Spot size

A beam of diameter $d$ is partly blocked before an edge reaches its axis. 2D treats the beam as a line; 3D can use a **shape-cast** for large spots, which widens each solid interval by about $d/2$ on each side.

## 7. Timing: response time and PLC scan

$$t_{seen} = t_{geometric} + t_{response} + t_{filter} + t_{sampling}$$

`responseTime` (default 2 ms) applies in both modes; the controller cycle (default 10 ms) gives a delay uniform in $[0, 10)$ ms in 3D, which 2D can reproduce or approximate by its mean.

## 8. Physics mode: raycasts

Every physics step the beam segment $E \to R$ is cast against the colliders (filtering out the conveyor's own rollers, frames and guides). Reporting the edge at the step midpoint bounds the error to $\Delta t/2 = 4.2$ ms at 120 Hz.

## 9. 2D ↔ 3D error budget for sensor edges (target ±50 ms)

| Source                                        | Typical size |
| --------------------------------------------- | ------------ |
| Physics step quantisation (midpoint estimate) | ≤ 4.2 ms     |
| Controller sampling (if not mirrored in 2D)   | ≤ 10 ms      |
| Pallet slip / micro-sliding (3D)              | 0 – 20 ms    |
| Different speed during hand-over              | 0 – 20 ms    |
| **Total (worst case, uncorrelated)**          | < 50 ms      |

## Related

[[S06 Photoelectric Sensor]] · [[S10 Pallet Types]] · [[T07 Control Logic and Signals]] · [[T11 Contact, Friction and Restitution]]
