---
title: S21 Code Block
lang: en
tags: [spec, information, scripting, control]
library-id: info.code
status: built — IntraScript 1.0 in the Design and Discrete simulation modules
updated: 2026-11-09
---

# S21 Code Block (`info.code`)

A **piece of IntraScript code** placed on the plan ([[A17 IntraScript Language]]), as the method objects of plant
simulators: its block variables, functions and event handlers (`@on_enter(Roller3)`, `@every(1 * h)`…), written in
Python's syntax and run with the simulation. Decisions: [[ADR-0018 IntraScript Scripting Language]],
[[ADR-0027 IntraScript in Python Syntax with Its Own Interpreter]]. A first hour with it: [[A20 IntraScript Tutorial]].

## 1. Parameters

| Key       | Kind   | Default        | Description                                                                  |
| --------- | ------ | -------------- | ---------------------------------------------------------------------------- |
| `source`  | text   | A starter (§2) | The block's code; edited in the code editor (§4), at most 200 000 characters |
| `enabled` | yes/no | yes            | A disabled block is neither checked before a run nor run                     |

The definition is Rust-only (`platform/sim/library/definitions/code-block.json`, category `control`, behaviours
`des.script` and `phys.script`, no ports, statistics `activations`, `steps`, `waiting`). The TypeScript reference is frozen
and never had code blocks.

## 2. The starter

A new block holds a short example that runs as it is:

```python
"""What this block does, in one sentence."""
from intrascript import *

# Variables of this block keep their value for the whole run.
counted = 0


@on_start
def begin():
    print("The run starts")
```

## 3. On the plan

A thin purple tablet, 600 × 600 mm, with raised braces `{ }`: seen and picked beside the objects its code drives. It
carries no pallets and has no ports; the 2D network leaves it out and the 3D run never sees it. The simulation tree lists
code blocks in their own group, **Code blocks**. In code the block is called by its code name (A17 §12.1): a block placed in the Design module is `Code block 100042` in the layout (its name is its family and id until it is given a `Name`) and `Code_block_100042` in code.

## 4. Editing (Design module)

- In the block's attributes, **Edit code…** opens the editor, a window over the views.
- The code is coloured (keywords, events, the layout's objects, built-ins, units, numbers, texts, comments) with line
  numbers; Tab indents by four spaces.
- Every change is checked against the layout as it is now (names, members, events, waits; A17 §15.1). Problem lines are
  underlined (errors red, warnings amber) and listed with their line, code and message; the toolbar counts them.
- **Names you can use** lists the events, every object of the layout with its members (signals marked writable or not),
  the built-ins and the units; a click copies one.
- **Save** (Ctrl+S) runs BIMCraft's `modify` command on `source`: one undo step, logged, the same for agents. **Revert**
  goes back to the saved code. Code with errors can be saved: it only keeps a run from starting.
- Renaming or deleting an object that code names makes those names check errors at the next check or run start.

## 5. During a run (Discrete simulation module)

1. **Start:** every enabled block is checked; any error stops the start with the list (block, line, message). Each
   block's variables get their values, every `@every` and `@at` timer is set, then the `@on_start` handlers run, in layout
   order.
2. **Events:** handlers start and wait as A17 §10, §11 and §16.2 say; turntables ask `@choose_exit` handlers for their
   exits.
3. **Beside the run:** the code's error, if one stopped the run; the **Console** (the last 200 lines; 10 000 are kept)
   with each line's time and block; **Variables of** each block, as they are now; the block's statistics in the
   statistics sheet.
4. **Errors:** a run error stops the run and shows the block, the line, the time and the message (A17 §15.2).

Several blocks may handle the same event: at the same instant and priority they run in layout order. Each block has its
own random stream (A17 §14.3).

The **Physics simulation module** runs a layout's physics without its code for now (A17 §16.3).

## 6. In code

From other blocks: `Routing.next_exit(pallet)` calls a function; `Routing.routed` reads or writes a block variable. A
block's `stats` holds its statistics.

## 7. Statistics

| Key           | Meaning                                                 |
| ------------- | ------------------------------------------------------- |
| `activations` | Handler activations started (exit decisions included)   |
| `steps`       | Instructions run (to find code that does too much work) |
| `waiting`     | Activations waiting now                                 |

## 8. Command line

`intracraft-cli code check LAYOUT.intracraft.json [CODE.py]` and `intracraft-cli code run LAYOUT.intracraft.json [CODE.py]
[--seconds N]` (A17 §19): `CODE.py` is added to the layout as a block called `Code`.

## 9. Validations

| Rule           | Severity | When                                                       |
| -------------- | -------- | ---------------------------------------------------------- |
| Check errors   | error    | The code has check errors (A17 §15.1): a run doesn't start |
| Check warnings | warning  | Listed in the editor; the run starts                       |

## 10. Not included yet

Breakpoints and step-by-step execution, watch expressions, code shared between projects as a library, the code in the
Physics simulation module, and moving, creating or deleting pallets from code (A17 §22).

## 11. Reference tests

All pass in the Rust 2D engine (`cargo test -p intracraft-des intrascript`):

- **IS-1** (counter): the roller line `demo-roller-line` with a photocell 1.5 m into its second roller; `@on_rise` counts
  exactly the photocell's pulses, `@on_enter` of the sink exactly its pallets received, and the block's activations are
  their sum.
- **IS-2** (routing): `demo-turntable-split`; `@on_create` gives each pallet a destination ("B" with probability 0.3),
  `@choose_exit` routes by it, `@on_enter(Outfeed_B)` stops the run if a pallet for A arrives there. Over 10 h (more than
  2000 pallets) the share at line B is within ±3 % of 30 %, and the same seed gives the same counts.
- **IS-3** (timed stop): `flow-t1`; stopping the conveyor for `wait(2 * s)` when the pallet enters makes its transit
  14.25 s instead of 12.25 s, within 1 %.
- A typo (`Roler_3_m`) keeps the run from starting and names the block, the line and the right name; disabling the block
  lets it start.
- `while True: pass` stops the run with `stepLimit`, the block, the line and the time, instead of freezing the window.
- Parameters read with their units (`3000 mm`, `0.25 m/s`, `12 s`), a pallet read in its sink's `@on_enter`, and
  `simulation.stop()` ending the run.
- The editor driven as a user: place, edit, see a typo found, save (undoable), run and read the console
  (`cargo test -p intracraft`).

## In the three modules

| Module              | This object                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------ |
| Design              | A family of the library placed on the plan; its code edited in the code editor; in the simulation tree |
| Discrete simulation | Its code runs with the run; console, variables and errors shown beside it                              |
| Physics simulation  | Not run yet (A17 §16.3)                                                                                |
