---
title: A20 IntraScript Tutorial
lang: en
tags: [app, scripting, tutorial]
status: current
updated: 2026-11-09
---

# A20 IntraScript Tutorial

A first hour with IntraScript, IntraCraft's programming language, for someone who knows plant simulation and has never
programmed. Each step adds a little to one model and says what to look at. The full reference is
[[A17 IntraScript Language]]; the object that holds the code is [[S21 Code Block]].

## 1. What code is for

Parameters and If/Then rules ([[A08 Controller]]) already make a line work. Code is for what they can't say: "send 30 % of
the pallets to line B", "hold each pallet 2 s at the photocell", "print the throughput every hour", "stop the infeed while
six pallets are in the section". IntraScript reads like Python, the most common language in engineering, with four
additions: **events** (when code runs), **waits** (pausing in the middle of a sequence), **units** (`150 * mm`) and **the
layout's objects by name**.

## 2. Place a code block

1. Open the layout `demo-turntable-split.intracraft.json` (a source, a turntable and two lines; drag it onto IntraCraft or
   use `scripts/intracraft.cmd`).
2. In the **Intralogistics** tab, panel **Logic**, click **Code block**. A purple tablet with braces `{ }` appears on the
   plan; move it beside the turntable, where it belongs.
3. In its attributes (the panel beside the views), click **Edit code…**. The editor opens with a starter:

```python
"""What this block does, in one sentence."""
from intrascript import *

# Variables of this block keep their value for the whole run.
counted = 0


@on_start
def begin():
    print("The run starts")
```

- The first text is the block's description. `from intrascript import *` only helps Python editors (A17 §18.2).
- `counted = 0` is a **block variable**: it lives for the whole run.
- `@on_start` is an **event**: the function under it runs when the run starts.

Press **Ctrl+S** to save. Every save is one undo step, like any other change of the model.

## 3. Run it

Open the **Discrete simulation** tab and press **Run**. Beside the run, under **Code**, the **Console** shows:

```text
0:00:00.000  Code_block_100042: The run starts
```

The time comes first, then the block's name as code calls it: the block is `Code block 100042` in the layout (its number
is its id), and code names turn spaces into `_`. **Variables of Code_block_100042** shows `counted = 0`.

## 4. Names of things

On the right of the editor, **Names you can use** lists every object of the layout as code calls it. A name that isn't an
identifier gets `_` for every space or sign: the turntable is `Turntable`, the lines `Line_A` and `Line_B`, the outfeeds
`Outfeed_A` and `Outfeed_B`, the source `Infeed`. Open an object to see its **members**: signals (`run`), values
(`count`, `pallets`, `stats`) and every parameter with its unit (`length`, `nominal_speed`). A click copies a name.

Change the start to:

```python
@on_start
def begin():
    print("Line A is", Line_A.length, "long and runs at", Line_A.nominal_speed)
```

The console says `Line A is 3000 mm long and runs at 0.25 m/s`: parameters carry their units.

## 5. Count pallets

Add, below the rest:

```python
@on_enter(Outfeed_B)
def arrived(pallet):
    global counted
    counted += 1
```

- `@on_enter(Outfeed_B)` runs the function each time a pallet enters `Outfeed_B`; `pallet` is that pallet.
- `global counted` says "the block's `counted`", not a new one. Forget it and the editor warns you
  (`localShadowsGlobal`); run anyway and the run stops at that line saying to add it.

Save, go to the Discrete simulation tab, **Reset** and **Run**. Watch `counted` grow under the block's **Variables**.

## 6. When the editor finds a mistake

Type `Line_C.run = False` somewhere in a function. The line is underlined in red and the list under the code says:

```text
line 12: error [unknownName] `Line_C` is not defined: … (did you mean `Line_A`?)
```

The toolbar says how many errors there are: **a run does not start while a block has errors**. Code with errors can still
be saved, so nothing is lost. Undo the change.

## 7. Wait inside a sequence

Hold every pallet 2 s when it reaches line A:

```python
@on_enter(Line_A)
def hold(pallet):
    Line_A.run = False
    wait(2 * s)
    Line_A.run = True
```

`wait(2 * s)` pauses this function for 2 s of **simulated** time; the rest of the plant goes on, and so do other handlers.
Each pallet gets its own paused copy of the function. Run it and watch line A stop and start.

Units: `2 * s` is two seconds, `150 * mm` a length, `0.25 * m / s` a speed. Mixing them wrongly is caught:
`Line_A.length + 2 * s` stops the run with `can't add a length and a time`.

## 8. Decide where pallets go

The turntable sends pallets to its exits in turn (its exit rule). Let code decide instead:

```python
@on_create(Infeed)
def tag(pallet):
    pallet.destination = "B" if random.chance(0.3) else "A"


@choose_exit(Turntable)
def route(pallet):
    return Line_B if pallet.destination == "B" else Line_A
```

- `@on_create(Infeed)` runs for each new pallet. `pallet.destination = …` gives the pallet an **attribute** that stays with
  it through the plant.
- `random.chance(0.3)` is true 30 % of the time. Random numbers come from the project's seed: the same seed gives the same
  run every time.
- `@choose_exit(Turntable)` answers "which exit?" when a pallet is centred on the turntable. It must answer at once, so it
  can't `wait`. Returning `None` leaves it to the turntable's exit rule.

After an hour of simulated time, about 30 % of the pallets are at `Outfeed_B`.

## 9. Report every hour

```python
@every(1 * h)
def report():
    a = Outfeed_A.stats["pallets_received"]
    b = Outfeed_B.stats["pallets_received"]
    print(f"{now()}: {a:.0f} at A, {b:.0f} at B, {b / (a + b):.1%} to B")
```

`@every(1 * h)` runs at 1 h, 2 h, 3 h… `stats` holds an object's statistics, the same the run panel shows, with Python
names (`pallets_received`). `f"…{value:.1%}"` formats a share as a percentage.

## 10. Wait for something to happen

```python
@on_start
def watch():
    if not wait_until(Outfeed_B.count > 0, timeout=10 * minute):
        warn("No pallet reached line B in the first 10 minutes")
```

`wait_until` pauses until the condition holds, looking at it again after every event, and gives `False` if the timeout
passes first. `warn` writes a warning to the console.

## 11. When the run stops with an error

An error while running stops the run and shows the block, the line, the time and what happened, above the console:

```text
Code_block_100042, line 23: the dictionary has no key 'C' [keyNotFound] (at 0:12:31.250)
```

Go back to the Design tab, open the block, fix that line, save and press **Reset**. A loop that never waits stops with
`stepLimit` instead of freezing the window.

## 12. Without the window

The same code runs from a terminal, in `platform/`:

```text
cargo run -p intracraft-cli -- code check ../packages/scenarios/scenarios/layouts/demo-turntable-split.intracraft.json routing.py
cargo run -p intracraft-cli -- code run ../packages/scenarios/scenarios/layouts/demo-turntable-split.intracraft.json routing.py --seconds 10800
```

`check` lists the problems; `run` simulates three hours and prints the console. The code file is added to the layout as a
block called `Code`.

## 13. Where to go next

- Every event, member, function and error: [[A17 IntraScript Language]].
- Python editors: copy `platform/sim/script/python/intrascript.py` next to your code files (A17 §18.2).
- Style: short handlers that call functions, units in values not names, one concern per block (A17 §17).
