---
title: A17 IntraScript Language
lang: en
tags: [app, control, scripting]
status: accepted — IntraScript 1.0, implemented in the Discrete simulation module
updated: 2026-11-09
---

# A17 IntraScript Language

**IntraScript** is IntraCraft's own programming language for simulation. It programs what parameters and If/Then rules
cannot: routing by pallet or order data, sequences with waits, counters and queues, reports during the run. Its code
lives in **code blocks** ([[S21 Code Block]]), objects placed on the plan beside the equipment they drive, edited in their
own code editor, as the method objects of commercial plant simulators.

IntraScript is written with **Python's syntax** and adds what a plant simulation needs: events as decorators, waiting
inside a sequence, physical units, and the layout's objects by name. It is not CPython: IntraCraft runs it with its own
small interpreter, open source and sandboxed. Decisions: [[ADR-0018 IntraScript Scripting Language]] (the language and its
objects) and [[ADR-0027 IntraScript in Python Syntax with Its Own Interpreter]] (the syntax and the interpreter). A guided
start: [[A20 IntraScript Tutorial]].

## 1. Goals

| Goal                        | What it means                                                                                        |
| --------------------------- | ---------------------------------------------------------------------------------------------------- |
| Familiar                    | Python's syntax: what engineers, the internet and AI assistants already know; any editor colours it  |
| Checked before the run      | Names, members, events and waits are checked while typing; a model with errors cannot start          |
| Physical units in the code  | `150 * mm`, `0.25 * m / s`, `2 * s`; adding a length to a time is an error, not a wrong number       |
| Made for simulation         | Events (`@on_enter`), waits (`wait`, `wait_until`), exit decisions, seeded random numbers            |
| Deterministic and sandboxed | Same layout, code and seed → same run; no files, network or clock; runaway loops stop with a message |
| Open source                 | The interpreter is `intracraft-script` (MIT OR Apache-2.0), no third-party code                      |

## 2. A first example

```python
"""Routes pallets by destination and holds each one 2 s at the photocell."""
from intrascript import *

routed = 0


@on_create(Infeed)
def tag(pallet):
    pallet.destination = "B" if random.chance(0.3) else "A"


@choose_exit(Turntable)
def route(pallet):
    global routed
    routed += 1
    return Line_B if pallet.destination == "B" else Line_A


@on_rise(Photoeye3.detected)
def hold():
    Roller3.run = False
    wait(2 * s)
    Roller3.run = True
```

`Infeed`, `Turntable`, `Line_A`, `Photoeye3` and `Roller3` are objects of the layout, named as §12.1 says. The first line
is optional: it only lets Python editors know IntraScript's names (§18.2).

## 3. Writing code

IntraScript follows Python 3:

- **Indentation makes blocks**, four spaces per level. Tabs are refused (the editor turns them into spaces).
- **A line ends a statement**, unless a bracket is open or it ends with `\`. `;` separates statements on one line.
- **`#` starts a comment.** A text right after `def …:` is the function's **help**, shown by the editor.
- **Names are case-sensitive**: `Roller3` and `roller3` are different. Python's keywords can't be names.
- **`=` assigns, `==` compares, `!=` is "not equal"**; `True`, `False`, `None` are capitalised.
- **Counting starts at 0**: the first item of a list is `items[0]`, the last `items[-1]`.

### 3.1 Literals

| Kind       | Examples                                | Notes                                                  |
| ---------- | --------------------------------------- | ------------------------------------------------------ |
| Integer    | `12`, `-3`, `1_000_000`, `0x1F`         | Whole numbers up to ±9.2·10¹⁸                          |
| Float      | `0.25`, `1.5e3`                         | Decimal point, never a comma                           |
| Quantity   | `150 * mm`, `0.25 * m / s`, `90 * deg`  | A number times a unit (§5); `150mm` is a syntax error  |
| Text       | `"Hall A"`, `'B'`, `"""…"""`            | Escapes `\n`, `\t`, `\\`, `\"`, `\xHH`, `\uHHHH`       |
| f-string   | `f"{n} pallets, {gap:.1f}"`             | Values between braces, with an optional format (§14.6) |
| Boolean    | `True`, `False`                         |                                                        |
| None       | `None`                                  | No value: no pallet, no object                         |
| List       | `[1, 2, 3]`, `[Roller1, Roller2]`, `[]` | Changeable, shared (§4.3)                              |
| Tuple      | `(1, "A")`, `(x,)`                      | Fixed                                                  |
| Dictionary | `{"A": 0, "B": 0}`, `{}`                | Keys in the order they came in                         |

## 4. Values

Variables have no declared type, as in Python; every value has one. Annotations (`x: int = 3`) are allowed and only
document.

### 4.1 Types

| Type     | Values                                 | Truth            |
| -------- | -------------------------------------- | ---------------- |
| `int`    | Whole numbers                          | false when 0     |
| `float`  | Decimal numbers (64-bit)               | false when 0.0   |
| quantity | A number with a dimension (§5)         | false when 0     |
| `bool`   | `True`, `False`                        |                  |
| `str`    | Text, up to 100 000 characters         | false when empty |
| `list`   | Items, up to 100 000                   | false when empty |
| `tuple`  | Fixed items                            | false when empty |
| `dict`   | Key → value, up to 100 000 keys        | false when empty |
| `range`  | `range(10)`, `range(2, 20, 3)`         |                  |
| object   | An object of the layout (§12)          | true             |
| pallet   | A pallet of the run (§13)              | true             |
| function | A function of a code block, a built-in | true             |
| `None`   | No value                               | false            |

`int / int` gives a float (`7 / 2` is `3.5`); `//` and `%` round toward minus infinity, as Python. Whole numbers that
outgrow ±9.2·10¹⁸ stop the run (`tooLarge`) instead of growing without bound.

### 4.2 Dictionary keys

Keys are numbers, texts, `None`, tuples of keys, objects and pallets. `1`, `1.0` and `True` are the same key, as in Python.
Lists, dictionaries and quantities can't be keys.

### 4.3 Values and references

Numbers, quantities, texts and tuples behave as values. Lists and dictionaries are **shared**: after `b = a`, both names
see the same list; `a.copy()` makes another. Objects and pallets are references to things of the model.

## 5. Units

A quantity is a number in SI units with its **dimension** (length, mass, time, angle). Unit names are built-in values of
size one unit, so `150 * mm` is 0.15 m and a quantity divided by a unit is a plain number: `gap / mm` is `150.0`.

| Unit name                       | Size                         | Dimension     |
| ------------------------------- | ---------------------------- | ------------- |
| `mm`, `cm`, `m`, `km`           | 0.001, 0.01, 1, 1000 m       | length        |
| `ms`, `s`, `minute`, `h`, `day` | 0.001, 1, 60, 3600, 86 400 s | time          |
| `g`, `kg`, `tonne`              | 0.001, 1, 1000 kg            | mass          |
| `deg`, `rad`                    | π/180, 1 rad                 | angle         |
| `rpm`                           | 2π/60 rad/s                  | angular speed |
| `percent`                       | 0.01                         | none          |

Minutes are `minute` (`min` is Python's minimum) and tonnes `tonne`.

**Rules**

- `+`, `-`, comparisons, `//` and `%` need the **same dimension**: `1 * m + 20 * mm` is 1.02 m; `1 * m + 2 * s` stops the run
  with `unitMismatch`; `gap + 2` too ("give the plain number its unit, for example `2 * mm`").
- `*` and `/` combine dimensions: `150 * mm / (0.25 * m / s)` is 0.6 s; `0.25 * m / s * 4 * s` is 1 m; a length over a length
  is a plain float.
- `**` takes a whole exponent: `0.5 * m / s**2` is an acceleration; `math.sqrt(4 * m * 1 * m)` is 2 m.
- Products and quotients that convert units are rounded to 14 significant digits, so `150 * mm / mm` is exactly 150.
- `round`, `int` and `float` refuse quantities: say the unit, `round(gap / mm)`.
- Parameters of the layout come with their units: `Roller3.length` is a length, `Roller3.nominal_speed` a speed.
- **Display:** lengths in mm, times in s, speeds in m/s, accelerations in m/s², masses in kg, angles in deg, rates per hour —
  `print(Roller3.length)` shows `3000 mm`. An f-string format applies to that number: `f"{gap:.1f}"` is `150.0 mm`.

## 6. Variables

| Where it is assigned                      | What it is                             | Lifetime                           |
| ----------------------------------------- | -------------------------------------- | ---------------------------------- |
| At the top of a block                     | A **block variable**                   | The whole run; reset at each start |
| Inside a function                         | A local variable of that call          | One call (kept while it waits)     |
| Inside a function after `global name`     | The block variable                     | The whole run                      |
| `pallet.name = value`                     | An **attribute of the pallet** (§13.1) | Until the pallet leaves the model  |
| `Other.name = value` (another code block) | That block's variable                  | The whole run                      |

Python's rule applies: a name assigned anywhere inside a function is local to it. To change a block variable, write
`global name` at the top of the function. The checker warns when a function assigns a name that is also a block
variable without `global` (`localShadowsGlobal`), because that is nearly always the mistake.

```python
counted = 0

@on_rise(Photoeye1.detected)
def count():
    global counted
    counted += 1
```

The top of a block holds only variables, functions, imports and the block's help text. Code that should run at the start
goes in a function decorated with `@on_start` (§10).

## 7. Statements

| Statement                                                   | Notes                                                                                    |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `x = value`, `a = b = 0`                                    | Also items (`d["A"] = 1`), attributes (`Roller3.run = False`) and unpacking (`a, b = t`) |
| `x += 1` (`-=`, `*=`, `/=`, `//=`, `%=`, `**=`)             | On names, items and attributes                                                           |
| `if … elif … else`                                          |                                                                                          |
| `while condition:`                                          | `break` and `continue` as in Python; no `else` on loops                                  |
| `for x in items:`                                           | Over lists, tuples, texts, dictionaries (their keys), ranges, `enumerate`, `zip`…        |
| `return`, `return value`                                    | Inside a function                                                                        |
| `pass`                                                      |                                                                                          |
| `global a, b`                                               | §6                                                                                       |
| `assert condition, "message"`                               | Stops the run with `assertionFailed` when false                                          |
| `from intrascript import *`, `import math`, `import random` | Allowed and ignored: those names are built in; other imports are an error                |

## 8. Expressions and operators

From the loosest to the tightest binding, as Python:

| Level | Operators                                                        | Notes                                      |
| ----- | ---------------------------------------------------------------- | ------------------------------------------ |
| 1     | `a if condition else b`                                          |                                            |
| 2     | `or`                                                             | Gives the first true operand, or the last  |
| 3     | `and`                                                            | Gives the first false operand, or the last |
| 4     | `not`                                                            |                                            |
| 5     | `==`, `!=`, `<`, `<=`, `>`, `>=`, `in`, `not in`, `is`, `is not` | Chained: `0 < n <= 3`                      |
| 6     | `+`, `-`                                                         | `+` also joins texts, lists and tuples     |
| 7     | `*`, `/`, `//`, `%`                                              | `*` also repeats texts and lists           |
| 8     | `-x`, `+x`                                                       |                                            |
| 9     | `**`                                                             | Right to left; `-2 ** 2` is −4             |
| 10    | `a.b`, `a[i]`, `a[i:j:k]`, `f(x)`                                | Attribute, item, slice, call               |

Also: list comprehensions with one `for` and an optional `if` (`[z for z in zones if not z.occupied]`). Comparing
quantities of different dimensions is an error, never just `False`.

## 9. Functions

```python
def release(zone, pause=2 * s):
    """Starts the zone after the pause and says whether it held a pallet."""
    wait(pause)
    zone.run = True
    return zone.occupied
```

- Functions are defined at the top of a block, not inside another function. Calls may name arguments:
  `release(Roller3, pause=5 * s)`.
- A default value is a fixed value: a number, a text, a quantity (`2 * s`), `True`, `False` or `None`.
- A function that waits makes its caller wait too.
- Functions call each other and themselves, up to 200 calls deep (`depthLimit`).
- Functions and variables of **another code block** are reached through its name: `Routing.next_exit(pallet)`,
  `Routing.routed`. The checker knows each block's names.
- `sorted`, `min`, `max` and `list.sort` take `key=` a function of a block (no `lambda` in 1.0).

## 10. Events

Code runs when something happens. A function becomes an **event handler** with a decorator; each time the event occurs, a
new **activation** of the handler starts.

| Decorator                          | When it runs                                                            | `pallet`       | `sender`            |
| ---------------------------------- | ----------------------------------------------------------------------- | -------------- | ------------------- |
| `@on_start`                        | At time 0, after every block's variables have their values              | `None`         | `None`              |
| `@on_finish`                       | When the run is ended (the CLI's `code run`, a closing run); can't wait | `None`         | `None`              |
| `@on_create(Source1)`              | A source made a pallet                                                  | The new pallet | The source          |
| `@on_enter(Roller3)`               | A pallet now belongs to the object (§10.3)                              | The pallet     | The object          |
| `@on_leave(Roller3)`               | A pallet no longer belongs to the object                                | The pallet     | The object          |
| `@on_rise(Photoeye3.detected)`     | The signal turns `True`                                                 | `None`         | The signal's object |
| `@on_fall(Photoeye3.detected)`     | The signal turns `False`                                                | `None`         | The signal's object |
| `@on_change(Roller3.run)`          | The signal changes either way                                           | `None`         | The signal's object |
| `@every(10 * minute, start=0 * s)` | Again and again; the first time at `start` (default: one interval)      | `None`         | `None`              |
| `@at(2 * h)`                       | Once, at that time                                                      | `None`         | `None`              |
| `@choose_exit(Turntable1)`         | A pallet centred on the turntable needs an exit (§10.4); can't wait     | The pallet     | The turntable       |

- A handler takes the parameters it wants among `pallet` and `sender`, by those names; others need a default value.
- Several objects at once: `@on_enter(Roller1, Roller2)`. Several decorators on one function bind it to all of them.
- An object whose name is not an identifier can be given as a text with its name or id: `@on_enter("Roller 1")`; an
  object in a section by its address: `@on_enter(Seccion1.Roller3)`, `@on_rise(Seccion1.Photoeye3.detected)`.
- The checker refuses events an object doesn't have: `@on_enter` on a photocell, `@on_create` on a conveyor, a signal the
  object doesn't have (`unknownEvent`, listing the ones it has).

### 10.1 Order at one instant

- `priority=` on any decorator: **higher runs first** at the same instant (default 0).
- Equal priority: blocks in layout order, then handlers in the order they are written.
- An activation runs until it ends or waits; nothing else runs meanwhile (no two pieces of code ever run at once).

### 10.2 What a handler sees

The handler's `pallet` and `sender`, `now()` (the time), every object of the layout, the block's variables and the other
blocks. Pallets that just went into a sink can still be read inside `@on_enter(Sink1)`.

### 10.3 When a pallet belongs to an object (2D)

In the Discrete simulation module a pallet belongs to the object its tail is on: a conveyor takes it over when the pallet
has crossed its entry entirely; a station's deck when it is on it; a sink when it takes it away (the pallet leaves the
model right after its `@on_enter`). A source's new pallet is created on its first conveyor: `@on_create(Source)` comes
first, then `@on_enter(Conveyor)`. For the moment the front edge passes a point, put a photocell there and use `@on_rise`.

### 10.4 Exit decisions

`@choose_exit(Turntable1)` answers where the pallet centred on the turntable goes. It returns one of the objects connected
to the turntable's exits (`Turntable1.exits`) or `None`; `None` lets the turntable's own exit rule decide
([[ADR-0008 Turntable 2D Cycle and Exit Rule]]). It runs to its end at once, so neither it nor anything it calls may wait
(`waitInDecision`, found by the checker through the calls it can see, and at run time otherwise). Returning an object
that is not one of the exits stops the run (`notAnExit`). In 1.0 turntables are the objects with exit decisions.

## 11. Waiting

| Call                                           | Effect                                                                     |
| ---------------------------------------------- | -------------------------------------------------------------------------- |
| `wait(2 * s)`                                  | Pauses this activation for 2 s of simulated time; the plant goes on        |
| `wait(0 * s)`                                  | Lets everything else due at this instant run first                         |
| `wait(5)`                                      | A plain number counts as seconds                                           |
| `wait_until(Roller4.occupied)`                 | Pauses until the condition is true; at once if it already is; gives `True` |
| `wait_until(Buffer.count > 0, timeout=30 * s)` | Gives `False` if 30 s pass first                                           |

```python
@on_enter(Roller5)
def watch(pallet):
    if not wait_until(Photoeye5.detected, timeout=30 * s):
        warn("No pallet reached Photoeye5 within 30 s")
```

`wait_until` is a special form: its condition is looked at again after every event of the run (and at its timeout), not
evaluated once as a Python call would. A paused activation keeps everything it was doing — loop counters, calls in
progress, local variables — and resumes in event order. A negative wait stops the run (`negativeWait`). `wait` and
`wait_until` are not allowed at the top of a block, in `@on_finish` or in an exit decision.

**Wait for conditions, don't poll**: `wait_until(Roller4.occupied)` instead of a loop with `wait(0.1 * s)`.

## 12. Objects of the layout

### 12.1 Names

Every object of the layout is a name in code. An object's **name in the layout** is its project parameter `Name` (a text of components; the Design module's attribute table will show it, and until then it is set with BIMCraft's `modify` command, as `set_name` does in `intracraft-model`); without one it is its family and id (`Roller conveyor 100023`). Its **code name** is that name when it is an identifier
(`Roller3`), otherwise the name with every other character turned into `_` (`Roller conveyor 100023` →
`Roller_conveyor_100023`, `Outfeed (15 s)` → `Outfeed_15_s`); two objects with the same code name get `_<id>` added. The
editor lists every name with its members (S21 §4). A name of the block hides an object of the same name (`shadowedName`).

Sections (ADR-0029) add a path: an object in section `Seccion1` is `Seccion1.Roller3` in code, and the checker resolves the whole path before the names. Each load unit has `pallet.name` (`MU3`, its number) and `pallet.address` (`Seccion1.Conveyor1.MU3`, where it is now); the number stays, the address follows the unit.

A bare name is looked up: the function's local variables, the block's variables and functions, the layout's objects,
the built-in names and units. Areas ([[S22 Area]]) and paths such as `Hall1.LineA.Roller3` are not part of 1.0.

### 12.2 Members

Members come from each object's **library definition** and from what the run knows of it; nothing is written by hand per
component. The library's camelCase keys read in Python's snake_case: `nominalSpeed` → `nominal_speed`.

| Member                | Objects                          | Access      | Value                                                                                |
| --------------------- | -------------------------------- | ----------- | ------------------------------------------------------------------------------------ |
| `name`, `id`, `kind`  | All                              | read        | Text: layout name, id, definition (`conveyor.roller`)                                |
| Every parameter       | All                              | read        | With its unit (§5); options as text; switches as `bool`                              |
| `run`                 | Conveyors                        | read, write | Signal: `False` stops the conveyor (it brakes), `True` runs it                       |
| `running`, `occupied` | Conveyors                        | read        | Signals                                                                              |
| `enable`              | Sources                          | read, write | Signal: `False` stops making pallets                                                 |
| `detected`            | Photocells                       | read        | Signal: the beam is interrupted                                                      |
| `occupied`, `count`   | Objects that carry pallets       | read        | A pallet is on it; how many                                                          |
| `pallets`             | Objects that carry pallets       | read        | List of its pallets, the one furthest ahead first                                    |
| `exits`               | Turntables                       | read        | List of the objects connected to its exits                                           |
| `next`                | Objects of the flow              | read        | The object its output leads to, or `None`                                            |
| `stats`               | Objects of the flow, code blocks | read        | Dictionary of its statistics now, snake_case keys: `Sink1.stats["pallets_received"]` |

- **Parameters can't be changed while the model runs** (`notWritable`): they are design data, changed in the Design module.
- Signals are written as `True` or `False`. Code's writes take effect at once and count as signal changes, so other
  handlers and If/Then rules react to them at the same instant.
- A code block's members are its variables and functions (§9).

## 13. Pallets

A `pallet` is anything a source creates: a pallet, a pallet with a load, or a load alone ([[S09 Source and Sink]]).

| Member                      | Value                                                  | Access         |
| --------------------------- | ------------------------------------------------------ | -------------- |
| `id`                        | Whole number, unique in the run, in creation order     | read           |
| `type`                      | Pallet type id ([[S10 Pallet Types]])                  | read           |
| `length`, `width`, `height` | Along travel, across travel, total height (lengths)    | read           |
| `load_mass`                 | Mass of the load                                       | read           |
| `created_at`, `age`         | Creation time, time since                              | read           |
| `location`                  | The object carrying it now, or `None`                  | read           |
| `speed`                     | Its speed now                                          | read           |
| `attributes`                | A copy of the attributes code gave it, as a dictionary | read           |
| Any other name              | An attribute code gives it (§13.1)                     | read and write |

### 13.1 Attributes

`pallet.destination = "B"` gives the pallet an attribute; any value fits, up to 1000 attributes per pallet. Reading one it
doesn't have stops the run with a message that says so (`unknownAttribute`); `pallet.attributes.get("destination")` reads
it or `None`. Attributes live as long as the pallet is in the model. The pallet's own data (the table above) can't be
written.

Moving, creating and deleting pallets from code (`pallet.move`, `Source1.create()`, `pallet.delete()`) are specified in
[[ADR-0018 IntraScript Scripting Language]] and not part of 1.0 (§22).

## 14. Built-in names

### 14.1 Functions

| Function                                                    | Notes                                                                      |
| ----------------------------------------------------------- | -------------------------------------------------------------------------- |
| `print(a, b, …)`                                            | Writes the values, separated by spaces, to the run's console with the time |
| `warn(a, b, …)`                                             | The same, as a warning                                                     |
| `error(a, b, …)`                                            | Stops the run with this message (`userError`)                              |
| `now()`                                                     | The simulation time, a time quantity                                       |
| `len`, `range`, `abs`, `min`, `max`, `sum`                  | As Python; `min`/`max` take `key=`; `sum` of quantities keeps the unit     |
| `round(x)`, `round(x, digits)`                              | Halves to even, as Python                                                  |
| `int`, `float`, `str`, `bool`, `repr`                       |                                                                            |
| `list`, `tuple`, `dict`                                     |                                                                            |
| `sorted(x, key=f, reverse=False)`, `reversed(x)`            |                                                                            |
| `enumerate(x, start=0)`, `zip(a, b, …)`, `any(x)`, `all(x)` |                                                                            |
| `wait`, `wait_until`                                        | §11                                                                        |

### 14.2 `math`

`math.sqrt` (keeps even dimensions), `sin`, `cos`, `tan` (take angles or plain numbers), `atan2` (gives an angle), `floor`,
`ceil`, `hypot`, `isclose(a, b, rel_tol=…, abs_tol=…)`, `exp`, `log`, and `math.pi`, `math.e`, `math.inf`.

### 14.3 `random`

Each code block draws from **its own stream**, seeded by the project's seed and the block's id, so adding or editing one
block never changes another's numbers ([[T03 Time, Event Ordering and Determinism]]). All accept quantities of one
dimension: `random.uniform(20 * s, 40 * s)` is a time.

| Function                                 | Gives                                              |
| ---------------------------------------- | -------------------------------------------------- |
| `random.random()`                        | Uniform in [0, 1)                                  |
| `random.uniform(a, b)`                   | Uniform in [a, b)                                  |
| `random.normal(mean, std_dev)` (`gauss`) | Normal                                             |
| `random.exponential(mean)`               | Exponential with that **mean** (not Python's rate) |
| `random.triangular(low, high, mode)`     | Triangular, in Python's order                      |
| `random.randint(a, b)`                   | Whole number from a to b, both included            |
| `random.chance(p)`                       | `True` with probability p                          |
| `random.choice(items)`                   | One of the items                                   |

### 14.4 `simulation`

`simulation.mode` (`"2D"`), `simulation.seed`, `simulation.stop(reason)`: the run ends at the end of this instant and
says why.

### 14.5 Methods

| Of    | Methods                                                                                                                                  |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| text  | `upper`, `lower`, `strip`, `lstrip`, `rstrip`, `startswith`, `endswith`, `split`, `join`, `replace`, `find`, `count`, `isdigit`, `zfill` |
| list  | `append`, `extend`, `insert`, `pop`, `remove`, `index`, `count`, `clear`, `sort(key=, reverse=)`, `reverse`, `copy`                      |
| tuple | `count`, `index`                                                                                                                         |
| dict  | `get`, `keys`, `values`, `items`, `pop`, `setdefault`, `update`, `clear`, `copy`                                                         |

### 14.6 f-string formats

`{value:spec}` with `spec` = `[[fill]align][0][width][.precision][type]`, align `<`, `>`, `^`, type `d`, `f`, `e`, `g`, `%`,
`s`: `f"{x:.2f}"`, `f"{n:03d}"`, `f"{name:>10}"`, `f"{share:.1%}"`. A quantity formats its number in its display unit and
keeps the unit (§5).

## 15. Errors

### 15.1 Found before the run (the checker)

The editor checks a block at every change against the layout as it is; `intracraft-cli code check` does the same. **A run
does not start while an enabled code block has an error**; warnings don't stop it. Every problem has a code, a line, a
column and a message that says what to do.

| Code                                        | Severity | Example                                                                        |
| ------------------------------------------- | -------- | ------------------------------------------------------------------------------ |
| `syntax`                                    | error    | A missing `:`, a wrong indentation, `150mm`, a Python feature left out (§18.1) |
| `unknownName`                               | error    | `Roler3.run` — with "did you mean `Roller3`?"                                  |
| `unknownMember`                             | error    | `Roller3.rnu`, `math.sqroot`, `Routing.next_exti`                              |
| `notWritable`                               | error    | `Roller3.length = 4 * m`, `Photoeye1.detected = True`, `pallet.id = 3`         |
| `unknownEvent`                              | error    | `@on_enter(Photoeye1)`, `@on_rise(Roller3.detected)`, `@on_enterr`             |
| `handlerParameters`                         | error    | `@on_enter(Roller3) def f(box):`                                               |
| `wrongArguments`                            | error    | `@every()` without a time, `wait()` without one                                |
| `waitInDecision`                            | error    | `wait` in `@choose_exit` or `@on_finish`, or in a function they call           |
| `waitOutsideHandler`                        | error    | `wait` at the top of a block                                                   |
| `topLevelCode`                              | error    | A call or a loop at the top of a block                                         |
| `importNotAllowed`                          | error    | `import os`                                                                    |
| `duplicateName`                             | error    | Two functions with one name                                                    |
| `nestedFunction`                            | error    | `def` inside a function                                                        |
| `defaultNotConstant`                        | error    | `def f(x=Roller3.length)`                                                      |
| `breakOutsideLoop`, `returnOutsideFunction` | error    | As their names say                                                             |
| `tooLarge`                                  | error    | Code longer than 200 000 characters                                            |
| `localShadowsGlobal`                        | warning  | Assigning a block variable inside a function without `global`                  |
| `shadowedName`                              | warning  | A variable named like an object or a built-in                                  |
| `unusedVariable`                            | warning  | A local variable that is never read (names starting with `_` are exempt)       |
| `unreachableCode`                           | warning  | Lines after `return`, `break` or `continue`                                    |

### 15.2 Found while running

A run error **stops the run** and says the block, the line, the time and the message, for example
`Routing, line 12: the dictionary has no key 'C' [keyNotFound] (at 0:12:31.250)`. The Discrete simulation module shows it
above the console; `intracraft-cli code run` prints it and exits with 1.

| Code                           | Cause                                                                                                    |
| ------------------------------ | -------------------------------------------------------------------------------------------------------- |
| `unitMismatch`                 | Adding, comparing or waiting with the wrong dimensions                                                   |
| `typeMismatch`                 | An operation on the wrong kind of value (`"a" + 1`)                                                      |
| `divisionByZero`               | `/`, `//` or `%` by zero                                                                                 |
| `indexOutOfRange`              | `items[5]` of a list of 3                                                                                |
| `keyNotFound`                  | `d["C"]` of a dictionary without it (use `d.get("C")`)                                                   |
| `unknownAttribute`             | An attribute a pallet hasn't been given, a method a value doesn't have                                   |
| `noneReference`                | `.x` of `None`; a pallet read after it left the model                                                    |
| `unboundLocal`                 | A local variable read before it gets a value (often a missing `global`)                                  |
| `emptyList`, `notFound`        | `pop()` of an empty list, `remove(x)` of an absent item                                                  |
| `wrongArguments`               | Missing, extra or repeated arguments                                                                     |
| `negativeWait`                 | `wait(-1 * s)`                                                                                           |
| `waitInDecision`               | A wait reached at run time where nothing may wait                                                        |
| `notAnExit`                    | An exit decision returned something that is not one of the exits                                         |
| `notANumber`                   | A result that is not a number (`math.sqrt(-1)`)                                                          |
| `assertionFailed`, `userError` | `assert` failed; `error(…)` was called                                                                   |
| `stepLimit`                    | An activation ran 1 000 000 steps without waiting (a loop that never ends)                               |
| `depthLimit`                   | Calls nested more than 200 deep                                                                          |
| `cascadeLimit`                 | More than 1 000 handlers started at one instant (code and signals triggering each other)                 |
| `tooLarge`                     | A list, text or dictionary over 100 000 items; a whole number over ±9.2·10¹⁸; 10 000 waiting activations |

## 16. How code runs

### 16.1 Activations

Each event starts an **activation** of its handler with its own local variables. It runs until it ends or waits; a
waiting activation keeps its calls, loops and variables and resumes later. Waiting activations are ordered like every
event of the run: by time, then priority, then the order they started in ([[T03 Time, Event Ordering and Determinism]]).

### 16.2 In the Discrete simulation module (2D)

1. An event of the run happens (a pallet arrives, a sensor switches, a station finishes, a code timer is due).
2. The plant settles at that time: pallets move on, objects update their output signals.
3. The If/Then rules run ([[A08 Controller]]).
4. The code runs: due timers (`wait`, `@every`, `@at`) first, then the handlers of what changed — new pallets, pallets
   that entered and left objects, signals that rose or fell — in the order of §10.1, then every `wait_until` looks at its
   condition again.
5. If the rules or the code wrote a signal, the plant settles again at the same instant and steps 3–5 repeat, up to the
   cascade limit.

`wait(2 * s)` resumes exactly 2 s later (to the microsecond). Exit decisions are asked while the plant settles, when a
pallet is centred on a turntable.

### 16.3 In the Physics simulation module (3D)

Not part of 1.0: a layout with code blocks runs its physics without them. The plan ([[ADR-0027 IntraScript in Python
Syntax with Its Own Interpreter]]) runs the same code on the controller cycle (every 10 ms), where the physics reports
sensor edges and pallets entering and leaving objects; times then differ from 2D by up to one cycle, inside the ±50 ms
agreement criterion ([[A11 Testing and Validation]]).

### 16.4 Determinism and limits

- Random numbers come from the block's seeded stream; there is no clock, no file and no network.
- Every collection is visited in a defined order (layout order, list order, insertion order of dictionaries).
- Same layout, code and seed → the same run on every computer.

| Limit                                               | Value                           |
| --------------------------------------------------- | ------------------------------- |
| Steps of one activation without waiting             | 1 000 000                       |
| Nested calls                                        | 200                             |
| Handlers started at one instant                     | 1 000                           |
| Items of a list or dictionary, characters of a text | 100 000                         |
| Waiting activations at once                         | 10 000                          |
| Characters of a block's code                        | 200 000                         |
| Attributes of a pallet                              | 1 000                           |
| Console lines kept                                  | 10 000 (the oldest are dropped) |

### 16.5 Sandbox

Code reaches the model only through the members of this note. It cannot import modules, read or write files, open
network connections, read the clock, start programs or see API tokens ([[A15 AI Agent Integration]]). The interpreter is
safe Rust (no `unsafe`, no `eval`, no panics): hostile code ends in a check error or a run error, never in a crash.

## 17. Style guide

- **Follow PEP 8**: four spaces, lines up to about 100 characters, two blank lines between functions.
- **Names:** variables, functions and pallet attributes in `snake_case` (`released_pallets`); fixed values in `UPPER_SNAKE`
  (`MAX_WAITING = 4`); objects keep their layout names. Yes/no names start with `is_` or `has_`.
- **Units belong in the value, not in the name:** `gap = 150 * mm`, never `gap_mm = 150`. Write the unit in every quantity,
  even when it is SI.
- **Handlers say when, functions say how:** a handler longer than about 20 lines calls functions.
- **Help text:** every function another block calls gets a one-line docstring.
- **Comments** explain why, not what.
- **Wait for conditions, don't poll** (§11). **Command the equipment** (`run`, `enable`, exit decisions) so the 2D and 3D
  results stay comparable.
- **One concern per block:** routing in one, reports in another; blocks talk through functions, not shared globals.

## 18. Python compatibility

### 18.1 What IntraScript leaves out

Classes, `lambda`, `try`/`except`/`raise` (use `error()`), `with`, generators (`yield`), `async`/`await`, `del`,
`nonlocal`, nested functions, `*args`/`**kwargs`, sets, `:=`, `while … else`, `for … else`, comprehensions with more than
one `for`, dictionary comprehensions, `%` text formatting, `import` of anything but `intrascript`, `math` and `random`.
Each one is a syntax error that names it. Everything else of this note is Python's meaning, except where it says
otherwise: `wait_until` (§11), quantities (§5), `random` (§14.3), event decorators (§10).

### 18.2 Python editors: `intrascript.py`

`platform/sim/script/python/intrascript.py` describes every IntraScript name for Python tools. With it beside the code (or
on `PYTHONPATH`), `from intrascript import *` makes any Python editor complete and explain the events, functions and
units, and Python's own syntax check accepts the code. It does not simulate: run the code in IntraCraft.

## 19. Code in the project, the editor, agents

- A code block keeps its code in its `source` attribute (text) and its `enabled` switch ([[S21 Code Block]]); the project
  file needs no new field. Changing the code is BIMCraft's `modify` command, so it is undone, logged and open to agents
  like every other change ([[ADR-0004 AI-Ready Command Layer and MCP]]).
- The **editor** (Design module, "Edit code…" in the block's attributes):
  - colours the code (keywords, events, objects and sections, built-ins, units, numbers, texts, comments), with line
    numbers, a status line (line, column, saved or not) and a zoom (Ctrl++, Ctrl+-, Ctrl+0);
  - checks it while typing, underlines the problem lines and lists them; a click on a problem selects its line;
  - **completes names** (`intracraft_script::complete`): after `Roller3.` its members, after `@` the events, after
    `pallet.`, `math.`, `random.`, `simulation.`, a section or another code block their names, and Ctrl+Space anywhere;
    Up and Down choose, Enter or Tab insert, Esc closes;
  - keeps the indentation on Enter (one level more after `:`), indents and outdents the selected lines with Tab and
    Shift+Tab, comments them out and in with Ctrl+/;
  - finds and replaces (Ctrl+F, Ctrl+H; match case, previous, next, replace, replace all) and goes to a line (Ctrl+G);
  - inserts templates from its Insert menu (a handler for each event, holding a conveyor, waiting with a timeout, a
    function, a block variable), indented, with the word to change selected;
  - lists the names the code can use and inserts one at the cursor with a click; saves with Ctrl+S.
- The **Discrete simulation module** shows the code's error, its console and every block's variables beside the run.
- **Command line:** `intracraft-cli code check LAYOUT.intracraft.json [CODE.py]` lists the problems (exit 1 on an error);
  `intracraft-cli code run LAYOUT.intracraft.json [CODE.py] [--seconds N]` runs the layout for N simulated seconds (3600 by
  default) and prints the console and each block's statistics. `CODE.py` is added to the layout as a block called `Code`.
  `intracraft-cli code names LAYOUT.intracraft.json` lists every name code can use with that layout (objects with their
  addresses, events and members; the load units' members; events; built-ins; units); without a layout it lists one
  object of every kind of the library.
- **Documentation for users** (Spanish): the guide `docs/tutorial/intrascript-tutorial.html` and the full manual
  `docs/tutorial/intrascript-manual.html`, whose object tables come from `code names` and whose examples were run.
- The language version is 1 (`intracraft_script::LANGUAGE_VERSION`); a later version that changes meaning migrates old
  code explicitly.

## 20. Grammar (summary)

```ebnf
block       = { NEWLINE | toplevel } ;
toplevel    = simple_line | funcdef ;
funcdef     = { "@" decorator NEWLINE } "def" NAME "(" [ param { "," param } ] ")" [ "->" expr ] ":" suite ;
decorator   = NAME [ "(" [ arg { "," arg } ] ")" ] ;
param       = NAME [ ":" expr ] [ "=" expr ] ;
suite       = simple_line | NEWLINE INDENT { statement } DEDENT ;
statement   = simple_line | if | while | for ;
if          = "if" expr ":" suite { "elif" expr ":" suite } [ "else" ":" suite ] ;
while       = "while" expr ":" suite ;
for         = "for" targets "in" exprs ":" suite ;
simple_line = small { ";" small } NEWLINE ;
small       = exprs | targets "=" { exprs "=" } exprs | target augop exprs | target ":" expr [ "=" exprs ]
            | "pass" | "break" | "continue" | "return" [ exprs ] | "global" NAME { "," NAME }
            | "assert" expr [ "," expr ] | "import" dotted | "from" dotted "import" ( "*" | NAME { "," NAME } ) ;
augop       = "+=" | "-=" | "*=" | "/=" | "//=" | "%=" | "**=" ;
expr        = or_expr [ "if" or_expr "else" expr ] ;
or_expr     = and_expr { "or" and_expr } ;
and_expr    = not_expr { "and" not_expr } ;
not_expr    = "not" not_expr | comparison ;
comparison  = arith { cmp_op arith } ;
cmp_op      = "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" | "not" "in" | "is" | "is" "not" ;
arith       = term { ( "+" | "-" ) term } ;
term        = factor { ( "*" | "/" | "//" | "%" ) factor } ;
factor      = ( "-" | "+" ) factor | power ;
power       = primary [ "**" factor ] ;
primary     = atom { "." NAME | "[" subscript "]" | "(" [ arg { "," arg } ] ")" } ;
arg         = expr | NAME "=" expr ;
atom        = NAME | NUMBER | STRING { STRING } | FSTRING | "True" | "False" | "None"
            | "(" [ exprs ] ")" | "[" [ exprs | expr "for" targets "in" or_expr [ "if" or_expr ] ] "]"
            | "{" [ expr ":" expr { "," expr ":" expr } ] "}" ;
```

## 21. Examples

### 21.1 Count pallets at a photocell

```python
counted = 0

@on_rise(Photoeye1.detected)
def count():
    global counted
    counted += 1
```

### 21.2 Destinations from a weighted list

```python
DESTINATIONS = [("A", 0.5), ("B", 0.3), ("C", 0.2)]

def draw_destination():
    """A destination drawn with the weights of DESTINATIONS."""
    u = random.random()
    for name, weight in DESTINATIONS:
        if u < weight:
            return name
        u -= weight
    return DESTINATIONS[-1][0]

@on_create(Infeed)
def tag(pallet):
    pallet.destination = draw_destination()
```

### 21.3 Zone logic without pressure

```python
ZONES = [Roller1, Roller2, Roller3, Roller4]

@every(0.5 * s, start=0 * s)
def scan():
    """Each zone runs only while the next one is free, like a PLC scan."""
    for here, ahead in zip(ZONES, ZONES[1:]):
        here.run = not ahead.occupied
```

### 21.4 Limit the pallets in a section

```python
MAX_IN_LINE = 6
SECTION = [Roller1, Roller2, Roller3]

@on_enter(Roller1)
def limit(pallet):
    if sum([z.count for z in SECTION]) >= MAX_IN_LINE:
        Infeed.enable = False
        wait_until(sum([z.count for z in SECTION]) < MAX_IN_LINE)
        Infeed.enable = True
```

Write the brackets: `sum(z.count for z in SECTION)` without them is a generator, left out of 1.0.

### 21.5 Hourly report

```python
@every(1 * h)
def report():
    done = Sink1.stats["pallets_received"]
    busy = Station1.stats["utilization"]
    print(f"{now()}: {done:.0f} pallets, station busy {busy:.1%}")
```

### 21.6 Time between pallets at a sink

```python
last = None
gaps = []

@on_enter(Sink1)
def arrived(pallet):
    global last
    if last is not None:
        gaps.append(now() - last)
    last = now()

@on_finish
def summary():
    if gaps:
        print(f"mean gap {sum(gaps) / len(gaps):.1f}, longest {max(gaps):.1f}")
```

## 22. Not included yet

| What                                                                             | Where it stands                                                              |
| -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Code in the Physics simulation module                                            | Planned (§16.3)                                                              |
| `pallet.move(target)`, `Source.create()`, `pallet.delete()`                      | Specified (ADR-0018, teleport in 3D); to be built                            |
| Variable, table and list objects on the plan ([[S18 Variable]]–[[S20 List]])     | Specified; until then, block variables, lists and dictionaries hold the data |
| Areas and paths ([[S22 Area]])                                                   | Specified                                                                    |
| Writing other signals (`reverse`, `turnTo`) and choosing exits of other stations | Need their 2D behaviour first                                                |
| Breakpoints, stepping, watch expressions                                         | The variables list of the run is the first step                              |
| Classes, `lambda`, exceptions, generators                                        | Left out on purpose (§18.1); may come when models need them                  |
| A visual block editor that writes IntraScript                                    | Not planned                                                                  |

## 23. Implementation

The crate `intracraft-script` (`platform/sim/script`, safe Rust, no dependencies) holds the language; it knows nothing of
conveyors. An engine describes its objects (`host::ModelInfo`) and answers the program (`host::Host`).

| File                    | What it does                                                                    |
| ----------------------- | ------------------------------------------------------------------------------- |
| `lexer.rs`              | Text to tokens: indentation, texts, f-strings, numbers                          |
| `parser.rs`             | Tokens to statements and expressions, Python's grammar; left-out features named |
| `compile.rs`            | The checker (§15.1) and the instructions; events from decorators                |
| `complete.rs`           | Completion: the names that fit at the cursor, without needing the code to parse |
| `vm.rs`                 | The runtime: activations, waits, timers, exit decisions, console, limits (§16)  |
| `ops.rs`                | Arithmetic, comparisons, indexing and formats, with units                       |
| `builtins.rs`           | Built-in functions, `math`, `random`, methods                                   |
| `units.rs`, `value.rs`  | Dimensions and units; values, dictionaries                                      |
| `python/intrascript.py` | The names for Python editors (§18.2)                                            |

The 2D engine's side is `intracraft-des`'s `flow/sim/script.rs` (events, members, exit decisions); the editor and the run
panel are `apps/intracraft/src/code.rs`, its text operations (indent, comment, find, templates) `code/edit.rs`; the
command line is `apps/intracraft-cli/src/code.rs`. Tests: the language's
own (`cargo test -p intracraft-script`), the reference scenarios IS-1 to IS-3 in the 2D engine (`cargo test -p
intracraft-des intrascript`) and the editor driven as a user does (`cargo test -p intracraft`).

## Related

[[ADR-0018 IntraScript Scripting Language]] · [[ADR-0027 IntraScript in Python Syntax with Its Own Interpreter]] ·
[[A20 IntraScript Tutorial]] · [[S21 Code Block]] · [[A08 Controller]] · [[T07 Control Logic and Signals]]
