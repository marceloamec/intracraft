---
title: T08 Statistics and Output Analysis
lang: en
tags: [theory, statistics]
---

🌐 **EN** · [[T08 Estadísticas y análisis de resultados|ES]] · [[T08 Statistik und Ergebnisanalyse|DE]] · [[Home (EN)|Index]]

# T08 Statistics and Output Analysis

## 1. Two kinds of statistics

| Kind                                | Examples                                | How it is computed                               |
| ----------------------------------- | --------------------------------------- | ------------------------------------------------ |
| **Observation-based** (tally)       | Transit time, cycle time, impact energy | One sample per pallet/event: mean, min, max, std |
| **Time-weighted** (time-persistent) | Occupancy, queue length, utilisation    | Value weighted by how long it lasted             |

Mixing them up is the most common statistics bug: the average queue length is **not** the average of the queue lengths observed at events.

## 2. Time-weighted averages in an event-driven engine

A state variable $x(t)$ (e.g. number of pallets on a conveyor) is piecewise constant between events. Its time average over $[0, T]$ is

$$\bar{x} = \frac{1}{T}\int_0^T x(t)\,dt = \frac{1}{T}\sum_i x_i\,\Delta t_i$$

Implementation: keep `lastValue` and `lastChangeTime`; at every change add `lastValue × (now − lastChangeTime)` to an accumulator. At the end, close the last interval. With integer microseconds the accumulation is exact.

## 3. State times and the standard object statistics

Each conveyor is always in exactly one state (stopped, accelerating, running, decelerating, blocked); the engine accumulates the time in each. From these:

| Statistic              | Definition                                          |
| ---------------------- | --------------------------------------------------- |
| Utilisation            | Time carrying a pallet in motion / measured time    |
| Blocked time           | Time holding a finished pallet that cannot leave    |
| Empty time             | Time with no pallet                                 |
| Pallets processed      | Counter of pallets that left through an output port |
| Throughput (pallets/h) | Pallets processed / measured time × 3600            |
| Mean transit time      | Tally of exit time − entry time                     |
| Mean occupancy         | Time-weighted number of pallets                     |

**Sanity check:** Little's law $\bar{L} = \lambda W$ must hold between mean occupancy, throughput and mean transit time ([[T05 Accumulation, Capacity and Throughput]]).

## 4. Numerically stable mean and variance

For tallies, use **Welford's algorithm** instead of summing $x$ and $x^2$ (which loses precision):

```text
n ← n + 1
δ ← x − mean
mean ← mean + δ / n
M2 ← M2 + δ · (x − mean)
variance = M2 / (n − 1)
```

## 5. Warm-up (initial transient)

A model usually starts **empty**, which is not representative: the first pallets find no queues and transit faster. Statistics collected from $t = 0$ are biased.

- Choose a **warm-up period** after which statistics are reset (MVP: a scenario parameter).
- Find it by plotting a moving average of the output (e.g. throughput per 10 min) and seeing where it flattens (*Welch's method*).
- For a 3 m conveyor line the warm-up is short (the time to fill the line); for loops with turntables it can be tens of minutes.

## 6. Replications and confidence intervals

With **stochastic** inputs one run is one sample. Run $n$ independent replications (different seeds), compute the result $Y_j$ of each and report

$$\bar{Y} \pm t_{n-1,\,0.975}\,\frac{s}{\sqrt{n}}$$

Example: 5 replications give throughputs 612, 598, 620, 605, 615 pallets/h → $\bar{Y} = 610$, $s = 8.6$, $t_{4,0.975} = 2.776$ → **610 ± 10.7 pallets/h** (95 %). If two layout variants' intervals overlap, the simulation has not shown a difference yet.

A **deterministic** model (fixed intervals, fixed masses) needs one run. Replications and experiments are post-MVP features, but the seeded RNG and per-object streams are designed for them ([[T03 Time, Event Ordering and Determinism]]).

## 7. Physics-mode measurements

The 3D mode adds quantities with no 2D equivalent:

- **Impacts**: speed and kinetic energy at each stop contact, warning above `maxImpactEnergy`.
- **Final pose**: position and yaw error of a pallet at rest against a stop (alignment quality).
- **Contact forces**: e.g. line pressure on a stop during accumulation with slip.

## 8. Event log and export

Every processed event can be logged (`time`, `type`, `target`, extra data). The log is:

- exported to CSV/JSON for analysis in other tools;
- hashed for the determinism test;
- never stored inside the project file (results are separate, plan §4.4).

## Related

[[T05 Accumulation, Capacity and Throughput]] · [[A11 Testing and Validation]] · [[A10 CLI and Scenarios]]
