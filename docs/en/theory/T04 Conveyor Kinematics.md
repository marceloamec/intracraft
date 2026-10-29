---
title: T04 Conveyor Kinematics
lang: en
tags: [theory, des, kinematics]
---

🌐 **EN** · [[T04 Cinemática de transportadores|ES]] · [[T04 Kinematik von Förderern|DE]] · [[Home (EN)|Index]]

# T04 Conveyor Kinematics

In the 2D mode a pallet never "feels" forces: it follows prescribed **motion profiles**. This note derives every formula the DES engine uses. Code: `packages/sim-des/src/kinematics.ts`.

## 1. Motion primitives

With constant acceleration $a$ starting at $t_0$ with position $s_0$ and speed $v_0$:

$$v(t) = v_0 + a\,(t - t_0) \qquad s(t) = s_0 + v_0 (t - t_0) + \tfrac12 a (t - t_0)^2$$

Two useful identities:

- Distance to change speed from $v_1$ to $v_2$: $\;d = \dfrac{v_2^2 - v_1^2}{2a}$
- Time to change speed: $\;t = \dfrac{v_2 - v_1}{a}$

A full motion is a **piecewise** sequence of such segments `{t0, s0, v0, a}`; the engine stores only the current segment of each pallet.

## 2. Trapezoidal profile (start and end at rest)

Accelerate with $a$ to $v_{max}$, cruise, decelerate with $b$ to rest over a distance $D$.

```text
 v
 │      ┌───────────────┐   v_max
 │     /                 \
 │    /                   \
 │   /                     \
 └──┴───────────────────────┴──── t
     t_a      t_c          t_b
```

- Ramp distances: $d_a = \dfrac{v_{max}^2}{2a}$, $\;d_b = \dfrac{v_{max}^2}{2b}$
- If $D \ge d_a + d_b$ (cruise is reached):

$$t = \frac{v_{max}}{a} + \frac{D - d_a - d_b}{v_{max}} + \frac{v_{max}}{b} = \frac{D}{v_{max}} + \frac{v_{max}}{2a} + \frac{v_{max}}{2b}$$

- Otherwise the profile is **triangular**: the peak speed is $v_p = \sqrt{\dfrac{2 D a b}{a + b}}$ and $t = \dfrac{v_p}{a} + \dfrac{v_p}{b}$. With $a = b$: $t = 2\sqrt{D/a}$.

## 3. Arriving at cruise speed (no braking) — test T1

A pallet that leaves the conveyor at full speed (hand-over to the next one) has no deceleration phase:

$$t = \frac{v}{a} + \frac{D - v^2/2a}{v} = \frac{D}{v} + \frac{v}{2a} \qquad (D \ge v^2/2a)$$

**T1:** $D = 3$ m, $v = 0.25$ m/s, $a = 0.5$ m/s² → $t = 12 + 0.25 = \mathbf{12.25\ s}$. The ramp costs exactly $v/2a = 0.25$ s compared with an instantaneous start. If $D < v^2/2a$: $t = \sqrt{2D/a}$.

## 4. Rotations use the same formulas — test T3

Replace distance by angle $\theta$, speed by $\omega$, acceleration by $\alpha$. With $\alpha$ for both ramps:

- Triangular if $\omega^2/\alpha \ge \theta$: $\;t = 2\sqrt{\theta/\alpha}$
- Trapezoidal otherwise: $\;t = \theta/\omega + \omega/\alpha$

**T3:** $\theta = 90°$, $\omega = 15$ °/s, $\alpha = 30$ °/s² → $\omega^2/\alpha = 7.5° < 90°$, so $t = 6 + 0.5 = \mathbf{6.5\ s}$. (Units cancel, so degrees can be used directly; the code works in radians.)

## 5. Transfer-table cycle — test T4

$$t = t_{lift} + \left(\frac{d}{v} + \frac{v}{a}\right) + t_{lower} \;(+\ t_{settle})$$

**T4:** $d = 1$ m, $v = 0.2$ m/s, $a = 0.5$ m/s², 1 s lift, 1 s lower → $1 + (5 + 0.4) + 1 = \mathbf{7.4\ s}$. Check: $v^2/a = 0.08$ m < 1 m, so the chain does reach cruise speed.

## 6. Approach (creep) speed before a stop

Real conveyors slow down to `approachSpeed` before a stop to limit impact energy ([[T14 Stops, Impacts and Alignment]]). From cruise $v$ to creep $v_c$ over a braking distance:

$$d_{slow} = \frac{v^2 - v_c^2}{2b}, \qquad t_{slow} = \frac{v - v_c}{b}$$

The slow-down must start at $s_{stop} - d_{slow} - d_{creep}$, where $d_{creep}$ is the distance travelled at creep speed (typically triggered by a pre-stop sensor). With defaults ($v = 0.25$, $v_c = 0.10$, $b = 0.5$): $d_{slow} = 0.0525$ m, $t_{slow} = 0.3$ s.

## 7. Predicting crossings: `timeToReach`

To schedule "front reaches sensor at $s^{\star}$", solve $s(t) = s^{\star}$ for the earliest $t \ge t_0$:

$$\tfrac12 a\,\Delta t^2 + v_0\,\Delta t - (s^{\star} - s_0) = 0$$

- $a = 0$: $\Delta t = (s^{\star} - s_0)/v_0$ if $v_0 \ne 0$ and the result is $\ge 0$.
- $a \ne 0$: discriminant $\Delta = v_0^2 + 2a(s^{\star} - s_0)$. If $\Delta < 0$ the target is never reached (a braking pallet stops short). Otherwise take the smallest non-negative root of $\Delta t = (-v_0 \pm \sqrt{\Delta})/a$.

**Caveat:** a decelerating segment is only valid until $v = 0$. The engine must end the segment at $t_0 + v_0/|a|$ (a "stopped" event) so that the parabola is never extrapolated backwards.

**Numerical note:** when $v_0^2 \gg |2a\Delta s|$ the form $(-v_0 + \sqrt{\Delta})/a$ loses precision (subtracting two close numbers). The equivalent $\Delta t = \dfrac{2\,\Delta s}{v_0 + \sqrt{\Delta}}$ is stable; adopt it if a test ever shows rounding issues.

## 8. Front, tail and orientation

The engine tracks the **front**; the tail is $s_{tail} = s_{front} - L$, where $L$ is the pallet dimension *along travel*: 1.2 m for an EUR with the long side forward, 0.8 m with the short side forward. EUR pallets travel long side forward on roller conveyors and short side forward on chain conveyors (confirmed by the project owner on 2026-10-29).

## 9. Hand-over between conveyors

While a pallet straddles two conveyors A → B, both carry it. Proposed 2D rule (to be confirmed in [[S01 Roller Conveyor]]):

- If both run, the pallet moves at $\min(v_A, v_B)$ (the slower surface dominates through friction).
- If B is stopped or full, the pallet stops with its front at B's entry (minus the gap) — **blocking**.
- The physical gap between conveyors (default 20 mm) is bridged instantly in 2D; in 3D it is a real gap the boards must cross.

## 10. Quick reference with default values

| Quantity                             | Formula             | Default result |
| ------------------------------------ | ------------------- | -------------- |
| Ramp time to 0.25 m/s                | $v/a$               | 0.5 s          |
| Ramp distance                        | $v^2/2a$            | 62.5 mm        |
| Extra time of one start (vs instant) | $v/2a$              | 0.25 s         |
| Extra time of start + stop           | $v/2a + v/2b$       | 0.5 s          |
| Transit 3 m, no stop                 | $D/v + v/2a$        | 12.25 s        |
| Transit 3 m, start and stop          | $D/v + v/2a + v/2b$ | 12.5 s         |

## Related

[[T02 Discrete-Event Simulation]] · [[T05 Accumulation, Capacity and Throughput]] · [[T13 Turntables and Transfer Lifts]] · [[A14 API Reference]]
