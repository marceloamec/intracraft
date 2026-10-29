---
title: T03 Time, Event Ordering and Determinism
lang: en
tags: [theory, des, determinism, rng]
---

🌐 **EN** · [[T03 Tiempo, orden de eventos y determinismo|ES]] · [[T03 Zeit, Ereignisreihenfolge und Determinismus|DE]] · [[Home (EN)|Index]]

# T03 Time, Event Ordering and Determinism

**Determinism** means: same layout + same seed ⇒ identical results, event by event. It is what makes debugging possible ("run it again and stop at 812.4 s") and what makes comparisons between two layout variants meaningful. In IntraCraft a failing determinism test is a top-priority bug.

## 1. Why time is an integer (microseconds)

Floating-point numbers are not exact: $0.1 + 0.2 = 0.30000000000000004$ and $(a + b) + c \ne a + (b + c)$ in general. In a DES engine this causes two problems:

1. **Drift** — adding many small durations accumulates error, so a clock after 10 000 cycles is not where it should be.
2. **False ties / false orderings** — two events that should be simultaneous end up a few ulps apart, and their order depends on the arithmetic path that produced them.

IntraCraft stores time as an **integer number of microseconds** (`SimTime`). Integers add exactly. JavaScript numbers represent integers exactly up to $2^{53}$, i.e. $9 \times 10^{15}$ µs ≈ **285 years** of simulated time — plenty.

**Rounding policy.** Kinematic predictions are computed in floating-point seconds (they involve square roots) and converted once with `secondsToSimTime(t) = round(t · 10⁶)`. The maximum error is 0.5 µs per event, five orders of magnitude below the ±50 ms sensor tolerance. `EventQueue.push` rejects non-integer times.

## 2. A total order of events

The future event list is ordered by the key **(time, priority, sequence)**:

| Key        | Meaning                                      | Why                                    |
| ---------- | -------------------------------------------- | -------------------------------------- |
| `time`     | When the event happens (µs)                  | Causality                              |
| `priority` | Smaller first, among events at the same time | Encodes "what logically happens first" |
| `seq`      | Monotonic counter assigned at insertion      | Makes every tie reproducible           |

Because `seq` is unique, the key is a **total order**: no two events compare equal, so the processing order never depends on the internals of the heap. (A binary heap is *not* a stable structure; without `seq`, equal keys could come out in any order.)

**Proposed priority bands** (to be confirmed in `docs/*/decisions`):

| Band | Events                                          |
| ---- | ----------------------------------------------- |
| 0    | Kinematic/physical: arrivals, stops, hand-overs |
| 10   | Sensor edges                                    |
| 20   | Signal changes                                  |
| 30   | Controller rule evaluation                      |
| 40   | Statistics sampling                             |

With these bands, a controller that fires at the same microsecond as a sensor edge always sees the new sensor value.

## 3. The binary heap

The queue is a binary min-heap stored in an array: parent of node $i$ is $\lfloor (i-1)/2 \rfloor$, children are $2i+1$ and $2i+2$. `push` appends and *sifts up*; `pop` moves the last element to the root and *sifts down*. Both are $O(\log n)$; `peek` is $O(1)$. Implemented in `packages/sim-des/src/event-queue.ts` and tested in `simulator.test.ts`.

## 4. Pseudo-random numbers with a seed

Stochastic inputs (inter-arrival times, load masses) come from a **pseudo-random number generator** (PRNG): a deterministic function that produces a sequence that *looks* random, fully determined by its **seed**.

`Math.random()` is forbidden in the engines: it cannot be seeded and its algorithm differs between browsers. An ESLint rule enforces this in `sim-des`, `sim-physics` and `control`.

### PCG32 (implemented in `core-model/src/rng.ts`)

PCG = *permuted congruential generator*. It has two stages:

1. **State transition** — a 64-bit linear congruential generator: $s_{n+1} = (a \cdot s_n + c) \bmod 2^{64}$ with $a = 6364136223846793005$ and an odd increment $c$ chosen by the **stream** number.
2. **Output permutation (XSH-RR)** — xor-shift the high bits, then rotate right by the top 5 bits of the state, giving a 32-bit output with much better statistical quality than the raw LCG.

Properties: period $2^{64}$, $2^{63}$ independent streams, tiny state, fast, passes standard statistical batteries. JavaScript `BigInt` is used for exact 64-bit arithmetic.

### Turning uniforms into distributions

| Distribution         | Method                                                    |
| -------------------- | --------------------------------------------------------- |
| Uniform $[a, b)$     | $a + (b - a)\,u$                                          |
| Exponential (rate λ) | Inverse transform: $-\ln(1 - u)/\lambda$                  |
| Normal (μ, σ)        | Box–Muller: $\mu + \sigma \sqrt{-2\ln u_1}\cos(2\pi u_2)$ |
| Empirical table      | Inverse of the cumulative table                           |

### One stream per object (proposal)

If all objects share one generator, adding a Source changes the random numbers every *other* Source receives — two layout variants are no longer compared under the same conditions. The proposed scheme derives a stream for each object from `hash(projectSeed, objectId)`. This is the classic **common random numbers** technique for comparing alternatives.

## 5. Determinism checklist

- Time as integer µs; events ordered by (time, priority, seq).
- No `Math.random()`, `Date.now()` or wall-clock time inside engines.
- Iterate collections in a defined order (JavaScript `Map`/`Set` keep insertion order — keep insertion deterministic; sort by id when in doubt).
- No dependence on object identity, hash codes or `for…in` order of numeric-like keys.
- In 3D: fixed step, same body creation order, same solver settings, engine build with determinism guarantees ([[T15 Physics Engine Research]]).
- **Hash the result**: the 2D event trace (`time`, `type`, `target` per event) and the 3D body states every N steps are hashed; two runs must give the same hash (test S1-g and the DES determinism test).

## Related

[[T02 Discrete-Event Simulation]] · [[T10 Time Integration and Fixed Step]] · [[A13 Conventions]] · [[A11 Testing and Validation]]
