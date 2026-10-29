---
title: A08 Controller
lang: en
tags: [app, control]
---

🌐 **EN** · [[A08 Controlador|ES]] · [[A08 Steuerung|DE]] · [[Home (EN)|Index]]

# A08 Controller (`packages/control`)

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

## 2. How each mode drives it

| Step                | 2D (DES)                                                 | 3D (physics)                        |
| ------------------- | -------------------------------------------------------- | ----------------------------------- |
| When to evaluate    | On every `signal_changed` event                          | Every control cycle (default 10 ms) |
| `previous` snapshot | Signal values before the change                          | Values at the previous cycle        |
| Apply a write       | Immediately, or schedule `signal_changed` at now + delay | At the first cycle ≥ now + delay    |
| Write conflicts     | Last rule in list order wins (deterministic)             | Same                                |

## 3. Planned additions

- Rule editor in the UI with templates (ZPA, stop with sensor, turntable waiting for centred pallet, full transfer sequence).
- Cancellation semantics of delayed writes (on-delay vs pulse) — to specify before phase 3.
- Signal registry with naming convention `<instance>.<signal>` and validation that every referenced signal exists.
- Interlocks are **not** rules: they live inside component behaviours and log a warning when they block a command.
- User scripting (sandboxed) is post-MVP.

## Related

[[T07 Control Logic and Signals]] · [[A06 DES Engine]] · [[A07 Physics Engine Wrapper]]
