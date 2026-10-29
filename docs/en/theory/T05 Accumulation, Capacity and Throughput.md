---
title: T05 Accumulation, Capacity and Throughput
lang: en
tags: [theory, des, flow]
---

🌐 **EN** · [[T05 Acumulación, capacidad y rendimiento|ES]] · [[T05 Stau, Kapazität und Durchsatz|DE]] · [[Home (EN)|Index]]

# T05 Accumulation, Capacity and Throughput

## 1. Key quantities

| Quantity          | Symbol    | Definition                                                            |
| ----------------- | --------- | --------------------------------------------------------------------- |
| Throughput        | $\lambda$ | Pallets leaving per unit time (pallets/s or pallets/h)                |
| Capacity (static) | $N$       | Maximum number of pallets that fit on a conveyor                      |
| Transit time      | $W$       | Time from entry to exit of one pallet                                 |
| Work in progress  | $\bar{L}$ | Average number of pallets inside an object                            |
| Headway           | $h$       | Time between two consecutive pallets passing a point; $\lambda = 1/h$ |
| Utilisation       | $\rho$    | Fraction of time an object is busy                                    |

## 2. Spacing-limited throughput — test T2

On a conveyor running continuously at speed $v$, pallets of length $L$ (along travel) separated by a gap $g$ pass a point every $h = (L + g)/v$ seconds:

$$\lambda_{max} = \frac{v}{L + g}$$

**T2:** EUR long side forward ($L = 1.2$ m), $g = 0.15$ m, $v = 0.25$ m/s → $h = 5.4$ s, $\lambda_{max} = 0.185$ pallets/s ≈ **667 pallets/h**. No line made of these conveyors can exceed this, whatever the control logic.

Static capacity: $N = \left\lfloor \dfrac{\text{length}}{L + g} \right\rfloor$ → a 3 m conveyor holds 2 EUR pallets.

## 3. Station-limited throughput and bottlenecks

Turntables and transfer tables handle **one pallet at a time**. Their throughput is $1/t_{cycle}$, where the cycle includes every phase until the station can accept the next pallet.

Illustrative chain turntable cycle (values are an example, not a specification):

| Phase                                 | Estimate         | Time                         |
| ------------------------------------- | ---------------- | ---------------------------- |
| Receive (≈ 1.4 m at 0.25 m/s + ramps) | $1.4/0.25 + 0.5$ | 6.1 s                        |
| Turn 90°                              | T3               | 6.5 s                        |
| Settle                                | parameter        | 0.3 s                        |
| Discharge                             | same as receive  | 6.1 s                        |
| Turn back 90° (empty)                 | T3               | 6.5 s                        |
| **Cycle**                             |                  | **25.5 s** → ≈ 141 pallets/h |

The line throughput is the minimum over its stages — the **bottleneck**. Here the turntable (141/h) limits a line whose conveyors could carry 667/h. Typical improvements: avoid the empty return turn (serve alternate stations), shorten ramps, overlap discharge and receive.

## 4. Little's law

For any stable system (long-run averages, no accumulation without bound):

$$\bar{L} = \lambda \cdot W$$

Example: a buffer section passes 120 pallets/h ($\lambda = 1/30$ s⁻¹) and each pallet spends on average 90 s there → on average $\bar L = 3$ pallets are in it. Little's law is a powerful sanity check of simulation statistics: if the three reported numbers violate it, something is wrong in the statistics code ([[T08 Statistics and Output Analysis]]).

## 5. Object states, blocking and starving

Each conveyor is always in exactly one state: **stopped, accelerating, running, decelerating, blocked**. Two kinds of lost time dominate flow studies:

- **Starved** — ready to work but nothing arrives (upstream problem). Shown as "empty time".
- **Blocked** — has a pallet ready to leave but the next object cannot accept it (downstream problem). Conveyors behave as **finite buffers with blocking after service**: a pallet that has finished its transport waits on the conveyor, occupying space.

Utilisation $\rho = t_{working}/t_{total}$; a bottleneck has $\rho \to 1$ and its upstream neighbours show blocked time, its downstream neighbours show starved time.

## 6. Accumulation modes

### 6.1 No accumulation (`none`, default)

The whole conveyor runs or stops as one unit and every pallet follows the rollers or chains. When a new pallet enters, the conveyor must run, so the pallets already on it move too; the gaps between pallets are kept. When the front pallet reaches the exit and the next conveyor cannot take it, the whole conveyor stops (blocked) and receives nothing more. A longer conveyor simply holds more pallets in a row. This is the default of every conveyor and, like all behaviour, a per-instance parameter ([[S01 Roller Conveyor]] §3).

### 6.2 Zero-pressure accumulation (ZPA)

The conveyor is split into **zones** (default 1.4 m ≥ pallet length + clearance), each with its own drive and end-of-zone sensor. Basic rule:

> Zone *i* runs if zone *i + 1* is free (or running); otherwise it stops when its own sensor is blocked.

Pallets therefore stop one per zone and **never touch**. Two release modes:

- **Singulation** — each zone restarts only after the zone ahead has cleared. Pallets leave with a larger spacing: the headway is roughly the time for a pallet to clear a zone from standstill, $h \approx L_z/v + v/2a = 1.4/0.25 + 0.25 = 5.85$ s (≈ 615/h, versus 667/h in continuous flow). This "accordion effect" is a real throughput loss that the 2D mode must reproduce.
- **Slug (train) release** — all zones of a queue start together, keeping the minimum spacing; faster, but needs downstream space.

The If/Then template for ZPA is in `control/src/rules.test.ts` and [[T07 Control Logic and Signals]].

### 6.3 Accumulation with pressure (chains with slip)

On chain conveyors the chains keep running under held pallets and **slip** under them. Each held pallet is pushed forward by the kinetic friction force $F = \mu\,m\,g$, and the forces add up along the queue:

$$F_{stop} = \sum_{k=1}^{n} \mu\,m_k\,g$$

Example: 3 pallets of 625 kg on chains with $\mu = 0.3$ → $F_{stop} = 3 \times 0.3 \times 625 \times 9.81 \approx 5.5$ kN on the first stop (and on the boards of the first pallet). In 2D the pallets simply stop in contact with the minimum gap (default 0 for pressure mode) after `stopContactTime`; the 3D mode computes the real contact forces.

## 7. Practical consequences for the 2D engine

1. A follower can never advance beyond the tail of its leader minus the minimum gap — this is a **predicted event** (`follower_reaches_leader`), not a per-step check.
2. Every start/stop costs $v/2a$ (start) and $v/2b$ (stop) compared with instantaneous motion ([[T04 Conveyor Kinematics]] §10); queues that start and stop often lose throughput.
3. Throughput must be measured after the **warm-up** period, when the queues have reached their steady pattern ([[T08 Statistics and Output Analysis]]).

## Related

[[T04 Conveyor Kinematics]] · [[T07 Control Logic and Signals]] · [[T08 Statistics and Output Analysis]] · [[S01 Roller Conveyor]] · [[S02 Chain Conveyor]]
