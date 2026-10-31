---
title: A08 Controller
lang: en
tags: [app, control]
---

🌐 **EN** · [[A08 Controlador|ES]] · [[A08 Steuerung|DE]] · [[Home (EN)|Index]]

# A08 Controller (`packages/control`)

> **Rust platform (2026-10-31):** this note describes the TypeScript reference, which is frozen ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). The Rust port is `intracraft-control` (`platform/sim/control`), phase R3 of [[A19 Rust Implementation Plan]].

Theory: [[T07 Control Logic and Signals]].

## 1. What exists today

`rules.ts` — the If/Then rule model and a **pure** evaluator:

```ts
type Condition = { signal: string; is: boolean } | { signal: string; edge: 'rising' | 'falling' };
interface Rule { id: string; when: Condition[]; then: { signal: string; value: boolean; delayMs?: number }[] }
evaluateRules(rules, previous, current) → SignalWrite[]   // { signal, value, delayMs, ruleId }
```

- All conditions of a rule are combined with **AND**. (OR = several rules.)
- Missing signals read as `false`.
- The evaluator never reads a clock and never touches the plant.

Tested with a two-zone ZPA template and edge/delay cases (`rules.test.ts`).

**Since 2026-10-30** ([[ADR-0010 Controller Rules in the Layout and in 2D]]):
- Rules are stored in `layout.rules` and edited with `control.setRules`.
- The 2D simulation evaluates them on every signal change. The signals are:
  - conveyor `run` (input), `running` and `occupied` (outputs);
  - source `enable` (input);
  - photocell `detected` (output).
- The editor has a Controller panel with live signal values and a *Stop at sensor* template.
- Delayed writes are never cancelled (provisional).

## 2. How each mode drives it

| Step                | 2D (DES)                                                 | 3D (physics)                        |
| ------------------- | -------------------------------------------------------- | ----------------------------------- |
| When to evaluate    | On every `signal_changed` event                          | Every control cycle (default 10 ms) |
| `previous` snapshot | Signal values before the change                          | Values at the previous cycle        |
| Apply a write       | Immediately, or schedule `signal_changed` at now + delay | At the first cycle ≥ now + delay    |
| Write conflicts     | Last rule in list order wins (deterministic)             | Same                                |

## 3. Planned additions

- Rule editor in the UI with templates (ZPA, stop with sensor, turntable waiting for centred pallet, full transfer sequence).
- Cancellation semantics of delayed writes (on-delay vs pulse) — provisionally "never cancelled" (ADR-0010).
- Signal registry with naming convention `<instance>.<signal>` and validation that every referenced signal exists.
- Interlocks are **not** rules: they live inside component behaviours and log a warning when they block a command.
- User code: [[A17 IntraScript Language]] ([[ADR-0018 IntraScript Scripting Language]], proposed). Rules stay; at one instant the rules run first, then the code.

## Related

[[T07 Control Logic and Signals]] · [[A06 DES Engine]] · [[A07 Physics Engine Wrapper]]
