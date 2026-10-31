---
title: A17 IntraScript Language
lang: en
tags: [app, control, scripting]
status: proposed — waiting for the owner's confirmation
updated: 2026-10-31
---

# A17 IntraScript Language

**IntraScript** is the programming language of IntraCraft. It programs behaviour that parameters and If/Then rules cannot express: routing by order data, sequences with waits, counters, queues and tables, reports during the run. Code lives in **code blocks** placed on the 2D plan, next to the variables, tables and lists it works with, and grouped in **areas** that give the model a hierarchy.

Decision and scope: [[ADR-0018 IntraScript Scripting Language]]. Information objects: [[S18 Variable]], [[S19 Table]], [[S20 List]], [[S21 Code Block]], [[S22 Area]]. Rules it coexists with: [[A08 Controller]].

## 1. Goals

| Goal                        | What it means for the language                                                                     |
| --------------------------- | -------------------------------------------------------------------------------------------------- |
| Readable by plant engineers | English keywords, blocks closed with `end`, no symbols to learn for references (`self`, `pallet`…) |
| Checked before the run      | Types, units and names are checked while typing; a model with errors cannot start                  |
| Physical units in the code  | `150 mm`, `0.25 m/s`, `2 s`; adding a length to a time is an error, not a wrong result             |
| Same code in 2D and 3D      | Code sees the same objects, signals and events in both modes (§14)                                 |
| Deterministic and sandboxed | Seeded random numbers, no clock, no files, no network; same layout and seed give the same result   |
| Ready for AI agents         | Code is text inside the project, changed only through commands (§16)                               |

## 2. A first example

```intrascript
// Pallets for destination "B" leave the turntable towards Roller5; the others go to Roller7.
var routed: integer := 0

on choose exit of Turntable1
  routed += 1
  if pallet.destination = "B"
    return Roller5
  end
  return Roller7
end

// Hold every pallet two seconds at the photocell of zone 3.
on Photoeye3.detected rises
  Roller3.run := false
  wait 2 s
  Roller3.run := true
end
```

## 3. Writing code

- **One statement per line.** No semicolons. A line continues on the next one when it ends with an operator, a comma or an open bracket.
- **Blocks end with `end`.** `if`, `case`, `while`, `for`, `method` and `on` open a block; `repeat` closes with `until`.
- **Not case-sensitive.** `Roller3`, `roller3` and `ROLLER3` are the same name; the editor rewrites a name with the spelling of its declaration. Text values (`"B"`) are compared exactly.
- **Comments** start with `//` and run to the end of the line. Lines starting with `///` directly above a `method` are its **help text**, shown by the editor when the method is used.
- **Names** start with a letter and continue with letters, digits or `_`. Reserved words (§17) cannot be names.

### 3.1 Literals

| Kind            | Examples                                          | Notes                                                       |
| --------------- | ------------------------------------------------- | ----------------------------------------------------------- |
| Integer         | `12`, `-3`, `1_000_000`                           | `_` groups digits                                           |
| Real            | `0.25`, `1.5e3`                                   | Decimal point, never a comma                                |
| Quantity        | `150 mm`, `0.25 m/s`, `0.5 m/s2`, `90 deg`, `2 s` | A number followed by a unit (§4.2); `m/s²` is also accepted |
| Clock time      | `1:30:00`                                         | Hours, minutes, seconds; type `time` (here 5400 s)          |
| Rate            | `120 /h`                                          | Events per time; type `rate`                                |
| Text            | `"Hall A"`, `"say ""hi"""`                        | A double quote inside text is written twice                 |
| Boolean         | `true`, `false`                                   |                                                             |
| Empty reference | `none`                                            | An object, pallet or list reference that points to nothing  |
| List            | `[1, 2, 3]`, `[Roller1, Roller2]`, `[]`           | All items of one type                                       |

## 4. Types

Every variable, parameter, method result, table column and pallet attribute has a type. The checker infers the type of `var x := 3` from its value; writing the type (`var x: integer := 3`) is recommended for anything that is not obvious.

### 4.1 Basic types

| Type      | Values                                                      | Default |
| --------- | ----------------------------------------------------------- | ------- |
| `integer` | Whole numbers up to ±2^53                                   | `0`     |
| `real`    | Decimal numbers (64-bit)                                    | `0.0`   |
| `boolean` | `true`, `false`                                             | `false` |
| `text`    | Text of up to 100 000 characters                            | `""`    |
| `any`     | A value whose type is known only when the code runs (§10.6) | —       |

An `integer` becomes a `real` where a real is expected; the other way needs `round`, `floor`, `ceil` or `toInteger`.

### 4.2 Quantities and units

Quantities carry a physical dimension. Internally they are stored in SI units ([[A13 Conventions]]); the units only matter when writing literals, converting and displaying.

| Type           | Dimension      | Units accepted in code                 | Shown by default as |
| -------------- | -------------- | -------------------------------------- | ------------------- |
| `time`         | time           | `ms`, `s`, `min`, `h`, `d`, clock time | `s` (or clock time) |
| `length`       | length         | `mm`, `cm`, `m`, `km`                  | `mm`                |
| `speed`        | length / time  | `mm/s`, `m/s`, `m/min`, `m/h`          | `m/s`               |
| `acceleration` | length / time² | `mm/s2`, `m/s2`                        | `m/s²`              |
| `mass`         | mass           | `g`, `kg`, `t`                         | `kg`                |
| `angle`        | angle          | `deg`, `rad`                           | `°`                 |
| `angularSpeed` | angle / time   | `deg/s`, `rad/s`, `rpm`                | `°/s`               |
| `rate`         | 1 / time       | `/s`, `/min`, `/h`                     | `/h`                |

Rules:

- `+`, `-`, comparisons and assignment need the **same dimension**: `1 m + 20 mm` is `1.02 m`; `1 m + 2 s` is a check error.
- `*` and `/` combine dimensions: `150 mm / 0.25 m/s` is `0.6 s`; `0.25 m/s * 4 s` is `1 m`; `length / length` is a `real`. Intermediate results may have any dimension; a value stored in a variable must have one of the types above or be a plain number.
- An `integer` or `real` multiplies any quantity: `2 * Roller1.length`.
- **Conversion to a number:** `gap in mm` gives a `real` (`150.0`). **Conversion from a number:** multiply by a unit literal, `x * 1 mm`.
- `sqrt` halves even dimensions (`sqrt(4 m * 1 m)` is `2 m`); trigonometric functions take an `angle`.

### 4.3 References to objects and pallets

| Type                                                         | Points to                                                    |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| `object`                                                     | Any object of the layout                                     |
| `conveyor`, `sensor`, `source`, `sink`, `station`, `scenery` | Objects of that library category ([[A05 Component Library]]) |
| `area`, `variable`, `table`, `list`, `code`                  | Information objects ([[S18 Variable]]…[[S22 Area]])          |
| `pallet`                                                     | A pallet (or load) moving through the model                  |

A narrower reference fits a wider one (`conveyor` → `object`); the other way needs a check: `if target is conveyor`. Reading a member of `none` stops the run with `noneReference`.

### 4.4 Lists

`list of T` holds items of type `T`, numbered **from 1**. `list of conveyor`, `list of pallet`, `list of length` and `list of list of integer` are all valid.

```intrascript
var zones: list of conveyor := [Roller1, Roller2, Roller3]
for each zone in zones
  zone.run := true
end
var last: conveyor := zones[zones.count]
```

List members: `count`, `isEmpty`, `first`, `last`, `[i]`, `add(x)`, `insert(i, x)`, `removeAt(i)`, `remove(x)` (first match), `contains(x)`, `indexOf(x)` (0 when absent), `clear()`, `sort(descending := false)`, `copy()`, and for numbers or quantities `sum()`, `min()`, `max()`, `mean()`.

### 4.5 Values and references

Numbers, quantities, booleans and text are **copied** on assignment. Lists, objects and pallets are **shared**: after `var b := a`, both names see the same list. `a.copy()` makes an independent list.

## 5. Declarations

```intrascript
const MAX_WAITING: integer := 4          // fixed value
var released: integer := 0               // block variable (lives for the whole run)
pallet attribute destination: text := "" // every pallet gets this attribute

method releaseNext(zone: conveyor, pause: time := 2 s) -> boolean
  var started: time := now               // local variable (lives for one call)
  ...
end
```

| Declaration        | Where                       | Lifetime                                   |
| ------------------ | --------------------------- | ------------------------------------------ |
| `var` at the top   | Code block, outside methods | The whole run; reset at the start of a run |
| `var` inside       | Method or `on` handler      | One call or one activation                 |
| `const`            | Anywhere                    | Value fixed at check time                  |
| `pallet attribute` | Code block, outside methods | Every pallet, from creation to departure   |

A pallet attribute is declared once in the model (in any code block); declaring the same name twice with different types is a check error.

## 6. Statements

### 6.1 Assignment

`x := value`, and the shortcuts `+=`, `-=`, `*=`, `/=`. The target can be a variable, a writable member (`Roller3.run`, `pallet.destination`), a table cell (`Orders[2, "qty"]`) or a list item.

### 6.2 Choices

```intrascript
if Buffer.count >= MAX_WAITING
  Source1.enable := false
elseif Buffer.isEmpty
  Source1.enable := true
else
  print("Buffer level ", Buffer.count)
end

case pallet.type
when "EUR", "EUR6"
  return Roller5
when "IND"
  return Roller7
else
  return none
end
```

`case` compares with `=`; the first matching `when` runs; `else` is optional.

### 6.3 Loops

```intrascript
while not Roller4.occupied
  wait 1 s
end

repeat
  attempts += 1
until pallet.move(Buffer2) or attempts = 3

for i := 1 to Orders.rows
  Orders[i, "done"] := false
end

for i := 10 to 1 step -1
  ...
end

for each p in Roller2.pallets
  p.destination := "B"
end
```

`break` leaves the innermost loop; `continue` jumps to its next turn. The bounds of `for … to` are evaluated once.

### 6.4 Waiting

| Statement                                  | Effect                                                                  |
| ------------------------------------------ | ----------------------------------------------------------------------- |
| `wait 2 s`                                 | Suspends the activation for 2 s of simulated time                       |
| `wait 0 s`                                 | Lets every other activation of the same instant run first               |
| `wait until Photoeye1.detected`            | Suspends until the condition is true (resumes at once if it already is) |
| `wait until Buffer.count > 0 timeout 30 s` | Same, but gives up after 30 s; then `timedOut` is `true`                |

```intrascript
wait until Photoeye5.detected timeout 30 s
if timedOut
  warn("No pallet reached Photoeye5 within 30 s")
end
```

`timedOut` belongs to the current activation and keeps the result of its last `wait until`. A negative wait stops the run (`negativeWait`). Waiting is not allowed in exit decisions (§9.2).

### 6.5 Output

| Call             | Effect                                                                 |
| ---------------- | ---------------------------------------------------------------------- |
| `print(a, b, …)` | Writes the values, joined, to the console of the run with the time     |
| `warn(a, b, …)`  | Same, as a warning in the run's issue list                             |
| `error(a, b, …)` | Stops the run with this message, the code block, the line and the time |

## 7. Expressions and operators

From the tightest to the loosest binding:

| Level | Operators                             | Notes                                                                     |
| ----- | ------------------------------------- | ------------------------------------------------------------------------- |
| 1     | `a.b`, `a[i]`, `f(x)`                 | Member, index, call                                                       |
| 2     | `^`                                   | Power; with quantities the exponent must be an integer literal            |
| 3     | `-a`                                  | Negation                                                                  |
| 4     | `*`, `/`, `div`, `mod`                | `/` always gives a real or quantity; `div` and `mod` take integers        |
| 5     | `+`, `-`                              | `+` also joins text                                                       |
| 6     | `in`                                  | Unit conversion to a number: `gap in mm`                                  |
| 7     | `=`, `<>`, `<`, `<=`, `>`, `>=`, `is` | Not chained: write `a < b and b < c`                                      |
| 8     | `not`                                 | `not Roller1.occupied and ready` means `(not Roller1.occupied) and ready` |
| 9     | `and`                                 | The right side is evaluated only when needed                              |
| 10    | `or`                                  | Same                                                                      |

Text is compared character by character (`"B" < "a"`), independent of the computer's language. Comparing reals with `=` is exact; for measured values prefer a tolerance: `abs(a - b) < 1 mm`.

## 8. Methods

```intrascript
/// Starts the zone after the given pause and reports whether it had a pallet.
method releaseZone(zone: conveyor, pause: time := 2 s) -> boolean
  wait pause
  zone.run := true
  return zone.occupied
end
```

- Parameters may have default values; those come last. A call may name its arguments: `releaseZone(Roller3, pause := 5 s)`.
- `-> type` declares the result; every path must end with `return value` (check error `missingReturn`). A method without `-> type` returns nothing.
- Arguments are passed like assignments (§4.5): a method can change a list it receives, never a number of the caller.
- Methods may call themselves; the depth is limited (§14.5).
- A method that waits makes its caller wait as well.
- Methods of another code block are called with its name: `Routing.nextDestination()`.

## 9. Events

Code runs only when something happens. An `on` block binds code to an event; each time the event occurs a new **activation** starts.

| Event                          | When it fires                                                        | `pallet`                     | `sender`             |
| ------------------------------ | -------------------------------------------------------------------- | ---------------------------- | -------------------- |
| `on start`                     | At time 0, after tables and variables are reset                      | `none`                       | `none`               |
| `on finish`                    | When the run reaches its end time or `simulation.stop()`             | `none`                       | `none`               |
| `on create Source1`            | A source has created a pallet (before it moves)                      | The new pallet               | The source           |
| `on enter Roller3`             | A pallet is handed over to the object (front edge at the entry port) | The pallet                   | The object           |
| `on leave Roller3`             | A pallet's tail has left the object                                  | The pallet                   | The object           |
| `on Photoeye3.detected rises`  | The signal changes from `false` to `true` (`falls`, `changes`)       | Pallet at the sensor, if any | The signal's object  |
| `on Counter changes`           | The value of a variable or the content of a list changes             | `none`                       | The variable or list |
| `on every 10 min`              | Every 10 min, first at 10 min; `from 0 s` moves the first one        | `none`                       | `none`               |
| `on at 2 h`                    | Once, at 2 h                                                         | `none`                       | `none`               |
| `on choose exit of Turntable1` | A pallet needs an exit (§9.2)                                        | The pallet                   | The object           |

`enter` and `leave` exist for every object with ports; a sink's `enter` fires before the pallet is removed.

### 9.1 Several activations at the same time

- `on … priority 5` sets the priority; **higher runs first** at the same instant (default 0).
- Equal priority: code blocks in layout order, then handlers in line order.
- An activation runs until it ends or waits; nothing else runs in between (no two pieces of code run at once).

### 9.2 Exit decisions

`on choose exit of X` answers which way a pallet leaves an object with more than one exit (turntables, transfer tables, elevators). It must **return** one of the objects connected to an exit of `X` (`X.exits`) or `none`; `none` lets the object's own exit rule decide ([[ADR-0008 Turntable 2D Cycle and Exit Rule]]). The decision runs to the end without waiting (`wait` there is a check error), because the object needs the answer at once. Returning an object that is not connected to an exit stops the run (`notAnExit`).

## 10. Objects, hierarchy and references

### 10.1 The model tree

```
root                      the layout
├─ Hall1                  area
│  ├─ LineA               area
│  │  ├─ Roller3          conveyor
│  │  ├─ Photoeye3        sensor
│  │  └─ ZoneLogic        code block
│  └─ Orders              table
└─ Source1                source
```

Every object has a **name**, unique inside its area (not case-sensitive), and a **path** from the root: `Hall1.LineA.Roller3`. Areas are described in [[S22 Area]].

### 10.2 How a name is found

A bare name is looked up, in this order:

1. local variables and parameters of the method or handler;
2. variables, constants and methods of the code block;
3. objects in the code block's own area;
4. objects in each enclosing area, up to `root`;
5. built-in names (§12).

So code inside `LineA` writes `Roller3`, and the same code keeps working when the area is copied as `LineB` with its own `Roller3`. A local name that hides an object gives the warning `shadowedName`.

### 10.3 Context words

| Word     | Means                                                     |
| -------- | --------------------------------------------------------- |
| `self`   | The code block that contains the code                     |
| `parent` | The area that contains the code block (`root` at the top) |
| `root`   | The layout                                                |
| `pallet` | The pallet of the current event (§9), or `none`           |
| `sender` | The object that raised the current event, or `none`       |
| `now`    | The current simulation time (`time`)                      |

`X.parent` gives the area of any object, so `pallet.location.parent` is the area the pallet is in.

### 10.4 Members of an object

Members come from the object's **definition** ([[A05 Component Library]]): nothing is written by hand per component.

| Member               | Type                              | Access         | Example                                        |
| -------------------- | --------------------------------- | -------------- | ---------------------------------------------- |
| `name`, `id`, `kind` | `text`                            | read           | `Roller3.kind` is `"conveyor.roller"`          |
| `parent`             | `area`                            | read           |                                                |
| Parameters           | From the parameter's unit (below) | read           | `Roller3.nominalSpeed` → `0.25 m/s`            |
| Input signals        | `boolean`                         | read and write | `Roller3.run := false`                         |
| Output signals       | `boolean`                         | read           | `Photoeye3.detected`                           |
| Free attributes      | From the value in the layout      | read and write | `Roller3.zone` → `"A"`                         |
| `occupied`, `count`  | `boolean`, `integer`              | read           | Pallets on the object                          |
| `pallets`            | `list of pallet`                  | read (copy)    | First item = the pallet furthest ahead         |
| `next`, `previous`   | `object`                          | read           | Connected neighbour; `none` if none or several |
| `exits`              | `list of object`                  | read           | Objects connected to its exits                 |
| `stats.<key>`        | Per statistic                     | read           | `Station1.stats.utilization`                   |

Parameter types follow their unit: `m` → `length`, `s` → `time`, `m/s` → `speed`, `m/s²` → `acceleration`, `kg` → `mass`, `rad` → `angle`, `rad/s` → `angularSpeed`, no unit → `real` (or `integer`), enumerations → `text`, switches → `boolean`. **Parameters cannot be changed while the model runs** (`notWritable`); they are design data, changed in the editor. Writes to free attributes change the running copy only and are undone at the next start.

### 10.5 Finding objects

| Call                               | Result                                                    |
| ---------------------------------- | --------------------------------------------------------- |
| `root.find("Hall1.LineA.Roller3")` | The object with that path, or `none`                      |
| `LineA.find("Roller3")`            | Path relative to an area                                  |
| `root.byId("roller3")`             | The object with that id (ids never change when renaming)  |
| `LineA.objects(conveyor)`          | Direct children of that kind, in layout order             |
| `LineA.allObjects(sensor)`         | All descendants of that kind                              |
| `LineA.pallets`                    | Pallets on all objects inside the area (work in progress) |

### 10.6 Information objects

| Object               | In code                                                          | Specification      |
| -------------------- | ---------------------------------------------------------------- | ------------------ |
| Variable `Counter`   | `Counter` is its value: `Counter += 1`; `Counter.stats.max`      | [[S18 Variable]]   |
| Table `Orders`       | `Orders[row, column]`, `Orders.rows`, `Orders.find("id", "A17")` | [[S19 Table]]      |
| List `Waiting`       | `Waiting.push(pallet)`, `Waiting.pop()`, `Waiting.count`         | [[S20 List]]       |
| Code block `Routing` | `Routing.nextDestination()`, `Routing.routed`                    | [[S21 Code Block]] |
| Area `LineA`         | `LineA.Roller3`, `LineA.objects(conveyor)`                       | [[S22 Area]]       |

A variable name followed by `.` names the object itself (`Counter.stats.max`); on its own it is the value. A table cell whose column is given by a constant name or number has that column's type; a column chosen at run time gives an `any` value, which can only be assigned to a declared variable or passed to `toText` and is checked when the code runs (`typeMismatch`).

## 11. Pallets

A `pallet` is anything a source creates: a pallet, a pallet with a load, or a load alone ([[S09 Source and Sink]]).

| Member                      | Type      | Access         | Notes                                           |
| --------------------------- | --------- | -------------- | ----------------------------------------------- |
| `id`                        | `integer` | read           | Unique in the run, in creation order            |
| `type`                      | `text`    | read           | Pallet type id ([[S10 Pallet Types]])           |
| `length`, `width`, `height` | `length`  | read           | Along travel, across travel, total height       |
| `mass`, `loadMass`          | `mass`    | read           | Whole item, load only                           |
| `createdAt`, `age`          | `time`    | read           | Creation time, time since creation              |
| `location`                  | `object`  | read           | Object carrying it now                          |
| `speed`                     | `speed`   | read           | Current speed along travel                      |
| Declared attributes         | Declared  | read and write | `pallet attribute destination: text := ""` (§5) |
| `move(target)`              | `boolean` | action         | See below                                       |
| `delete()`                  | —         | action         | Takes the pallet out of the model at once       |

Creating pallets: `Source1.create(type := "")` creates one pallet at the source now, as if its interval had ended, and returns it; it returns `none` when the source exit is blocked. `on create` fires for it.

### 11.1 Moving pallets directly

`pallet.move(target)` places the pallet at the entry of `target` (a conveyor, station or sink), with its front edge at the entry port, moving with the target's current speed. It returns `false` and changes nothing when the target cannot take it now: its entry is occupied, a station is in a cycle, or the pallet itself is inside a station cycle (turning, lifting, processing). Moving onto a sink counts as an arrival at that sink. `delete()` counts the pallet in the run statistic `deletedByCode`, never as an arrival.

| Mode | What happens                                                                                                                         |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------ |
| 2D   | The pallet is taken from its carrier and put on the target at the same instant                                                       |
| 3D   | **Teleport:** the rigid body is removed and created again at the target's entry with the deck speed, if no other body overlaps there |

A direct move is not physical: photocells between the old and the new place do not see the pallet, and travel time is zero. Both modes record it in the run log as `teleport`. Prefer commanding the equipment (`run`, `turnTo`, exit decisions) when the result should be comparable with the real installation.

## 12. Built-in functions

| Group      | Functions                                                                                                                                                                                                                    |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Numbers    | `abs`, `min(a, b, …)`, `max(a, b, …)`, `round(x)`, `round(x, digits)`, `floor`, `ceil`, `sqrt`, `sin`, `cos`, `tan`, `atan2(y, x)`, `PI`                                                                                     |
| Conversion | `toText(x)`, `toText(x, "mm")`, `toText(x, digits := 2)`, `toText(now, "hh:mm:ss")`, `toInteger(text)`, `toReal(text)`, `x in unit`                                                                                          |
| Text       | `t.size`, `t.upper`, `t.lower`, `t.trim`, `t.contains(s)`, `t.startsWith(s)`, `t.part(from, count)`, `t.split(separator)`                                                                                                    |
| Time       | `now`, `simulation.endTime`                                                                                                                                                                                                  |
| Random     | `random.uniform(a, b)`, `random.normal(mean, stdDev)`, `random.exponential(mean)`, `random.triangular(min, mode, max)`, `random.integer(a, b)`, `random.chance(p)`, `random.choice(list)`, `random.row(table, weightColumn)` |
| Simulation | `simulation.mode` (`"2D"` or `"3D"`), `simulation.seed`, `simulation.stop(reason := "")`                                                                                                                                     |
| Output     | `print`, `warn`, `error` (§6.5)                                                                                                                                                                                              |

`random.*` accept quantities (`random.uniform(20 s, 40 s)` is a `time`). `round` rounds halves away from zero. `random.row` picks a table row with probability proportional to a numeric column (a distribution table).

## 13. Errors

### 13.1 Found before the run (check errors)

The editor checks every code block on each change and lists the problems by line. **A run does not start while any enabled code block has an error**; warnings do not block it.

| Code                 | Severity | Example                                                        |
| -------------------- | -------- | -------------------------------------------------------------- |
| `syntax`             | error    | `if` without `end`                                             |
| `unknownName`        | error    | `Roler3.run` (typo), or an object that was deleted or renamed  |
| `typeMismatch`       | error    | `var n: integer := "five"`                                     |
| `unitMismatch`       | error    | `Roller1.length + 2 s`                                         |
| `notWritable`        | error    | `Roller1.length := 4 m`, `Photoeye1.detected := true`          |
| `wrongArguments`     | error    | Missing, extra or wrongly typed arguments                      |
| `missingReturn`      | error    | A method with `-> boolean` that can end without `return`       |
| `waitInDecision`     | error    | `wait` inside `on choose exit of`                              |
| `unknownEvent`       | error    | `on enter Photoeye1` (a sensor has no ports)                   |
| `duplicateName`      | error    | Two methods or block variables with the same name              |
| `signalWrittenTwice` | warning  | An If/Then rule and a code block write the same signal (§14.4) |
| `unusedVariable`     | warning  | A local variable that is never read                            |
| `shadowedName`       | warning  | A local variable with the name of an object                    |
| `unreachableCode`    | warning  | Statements after `return`                                      |
| `anyValue`           | warning  | A table column chosen at run time (§10.6)                      |

### 13.2 Found while running (run errors)

A run error **stops the run** and shows: code block, line, column, simulation time and message, for example `Routing, line 12: row 5 does not exist; Orders has 4 rows (at 00:12:31.250)`.

| Code                    | Cause                                                                                       |
| ----------------------- | ------------------------------------------------------------------------------------------- |
| `indexOutOfRange`       | List or table index outside 1 … count                                                       |
| `divisionByZero`        | `/`, `div` or `mod` by zero                                                                 |
| `noneReference`         | Member of `none`                                                                            |
| `typeMismatch`          | An `any` value of the wrong type                                                            |
| `negativeWait`          | `wait` with a negative time                                                                 |
| `notAnExit`             | Exit decision returned an object that is not connected to an exit                           |
| `emptyList`, `listFull` | `pop` on an empty list, `push` on a full one ([[S20 List]])                                 |
| `stepLimit`             | An activation ran more than 1 000 000 steps without waiting (a loop that never ends)        |
| `depthLimit`            | Methods nested more than 200 deep                                                           |
| `cascadeLimit`          | More than 1 000 activations started at one instant (code and signals triggering each other) |
| `tooLarge`              | List or table over 100 000 items or rows, text over 100 000 characters, integer over ±2^53  |
| `userError`             | `error(…)` was called                                                                       |

## 14. How code runs

### 14.1 Activations

Each event starts an **activation** with its own local variables. An activation runs until it ends or reaches a `wait`; then it is suspended with everything it was doing (loop counters, method calls) and resumes later. Waiting activations are ordered like all events: by time, then priority, then creation order ([[T03 Time, Event Ordering and Determinism]]).

### 14.2 In the 2D mode

1. An event of the model happens (a pallet enters, a sensor switches, a station finishes).
2. The object updates its state and its output signals.
3. The If/Then rules are evaluated ([[A08 Controller]]).
4. The handlers for that event start, in the order of §9.1.
5. Signals written by code take effect at once; rules and handlers that depend on them run next, at the same instant, up to the cascade limit.

`wait 2 s` resumes exactly 2 s later (rounded to the microsecond). `wait until` is checked again after every event of the run.

### 14.3 In the 3D mode

The same code runs on the controller cycle (every 10 ms, like the rules): events detected by the physics during the cycle start their handlers in the order they happened; `wait` resumes at the first cycle at or after its time; `wait until` is checked on every cycle; exit decisions are answered at once. Times therefore differ from the 2D mode by up to one cycle, inside the ±50 ms agreement criterion ([[A11 Testing and Validation]]).

### 14.4 Code and If/Then rules

Both stay. Rules suit simple on/off logic and are quicker to read; code suits decisions, data and sequences. At one instant the rules run first, then the code. When both write the same signal, the last write wins; the checker warns (`signalWrittenTwice`) when a rule and a code block write the same signal.

### 14.5 Determinism and limits

- Random numbers come from the project's seeded generator; each code block has its **own stream**, derived from its id, so editing or adding one block does not change the random numbers of another.
- No access to the computer clock; `now` is simulation time.
- Every collection is visited in a defined order (layout order, list order, row order).
- Same layout, code and seed → the same result in every run and on every computer.

| Limit                                   | Value     |
| --------------------------------------- | --------- |
| Steps of one activation without waiting | 1 000 000 |
| Nested method calls                     | 200       |
| Activations started at one instant      | 1 000     |
| Items in a list, rows in a table        | 100 000   |
| Characters in a text                    | 100 000   |
| Waiting activations at once             | 10 000    |

### 14.6 Sandbox

Code reaches the model only through the members listed in this note. It cannot read or write files, open network connections, read the clock, start programs or see API tokens ([[A15 AI Agent Integration]]). The interpreter is written in TypeScript inside the simulation worker and never turns code into JavaScript (`eval` is not used).

## 15. Style guide

- **Indent** with two spaces, like the rest of IntraCraft.
- **Names:** objects in `PascalCase` (`InfeedSensor`, `Orders`); variables, methods and attributes in `camelCase` (`releasedPallets`); constants in `UPPER_SNAKE` (`MAX_WAITING`). Names of yes/no values and methods start with `is` or `has` (`isFree`).
- **Units belong in the type, not in the name:** `gap: length := 150 mm`, never `gapMm := 150`.
- **Write units** in every quantity literal, even when the value is in SI units.
- **Spaces** around operators and `:=`, after commas and after the `:` of a type; none before `(` in a call.
- **Blank lines:** one between methods and handlers; none just after a line that opens a block.
- **Short handlers:** a handler says *when*; a method says *how*. Move logic of more than about 20 lines into methods.
- **Help text:** every method called from another code block gets a `///` line.
- **Comments** explain why, not what.
- **Wait for conditions, do not poll:** `wait until Roller4.occupied` instead of a loop with `wait 0.1 s`.
- **Prefer equipment over teleport:** command conveyors and decide exits; use `pallet.move` only for model shortcuts.

## 16. Code in the project and AI agents

Code is stored as text in the project file ([[S21 Code Block]]) with a `languageVersion`, so future versions of the language can upgrade it with an explicit migration. It changes only through commands (`script.setCode`; tables and lists through `data.*`) with undo and the actor log; the query `script.check` returns the check errors. Agents use the same commands ([[ADR-0004 AI-Ready Command Layer and MCP]]); code written by an agent is checked like any other.

## 17. Reserved words

`and`, `any`, `attribute`, `break`, `case`, `const`, `continue`, `div`, `each`, `else`, `elseif`, `end`, `false`, `for`, `if`, `in`, `is`, `method`, `mod`, `none`, `not`, `now`, `of`, `on`, `or`, `pallet`, `parent`, `priority`, `repeat`, `return`, `root`, `self`, `sender`, `step`, `timedOut`, `timeout`, `to`, `true`, `until`, `var`, `wait`, `when`, `while`, and the type names of §4. Event words (`start`, `finish`, `create`, `enter`, `leave`, `choose`, `exit`, `rises`, `falls`, `changes`, `every`, `at`, `from`) are keywords only right after `on`.

## 18. Grammar (summary)

```ebnf
codeBlock   = { declaration | method | handler } ;
declaration = ( "var" | "const" ) name [ ":" type ] ":=" expression NEWLINE
            | "var" name ":" type NEWLINE
            | "pallet" "attribute" name ":" type ":=" expression NEWLINE ;
type        = "integer" | "real" | "boolean" | "text" | "any" | quantityType | objectType
            | "pallet" | "list" "of" type ;
method      = { "///" docText NEWLINE } "method" name "(" [ param { "," param } ] ")" [ "->" type ] NEWLINE
              body "end" ;
param       = name ":" type [ ":=" expression ] ;
handler     = "on" event [ "priority" integer ] NEWLINE body "end" ;
event       = "start" | "finish" | ( "create" | "enter" | "leave" ) path
            | path ( "rises" | "falls" | "changes" ) | "every" expression [ "from" expression ]
            | "at" expression | "choose" "exit" "of" path ;
body        = { statement NEWLINE } ;
statement   = declaration | target assignOp expression | call
            | "if" expression NEWLINE body { "elseif" expression NEWLINE body } [ "else" NEWLINE body ] "end"
            | "case" expression NEWLINE { "when" expression { "," expression } NEWLINE body } [ "else" NEWLINE body ] "end"
            | "while" expression NEWLINE body "end"
            | "repeat" NEWLINE body "until" expression
            | "for" name ":=" expression "to" expression [ "step" expression ] NEWLINE body "end"
            | "for" "each" name "in" expression NEWLINE body "end"
            | "break" | "continue" | "return" [ expression ]
            | "wait" expression | "wait" "until" expression [ "timeout" expression ] ;
assignOp    = ":=" | "+=" | "-=" | "*=" | "/=" ;
path        = name { "." name } ;
```

## 19. Examples

### 19.1 Count pallets at a photocell

```intrascript
// Counted is a variable object (integer) next to this code block.
on Photoeye1.detected rises
  Counted += 1
end
```

### 19.2 Destinations from an order table

```intrascript
// Orders: columns "destination" (text) and "weight" (real). Each new pallet gets a destination
// drawn with the weights of the table.
pallet attribute destination: text := ""

on create Source1
  var row: integer := random.row(Orders, "weight")
  pallet.destination := Orders[row, "destination"]
end
```

### 19.3 Zone logic over a whole area

```intrascript
// Each zone of this area runs only while the next zone is free (no pressure between pallets).
// A scan every 0.5 s, like a PLC cycle: one handler serves however many zones the area has.
method isFree(zone: object) -> boolean
  return zone = none or not zone.occupied
end

on every 0.5 s from 0 s
  for each zone in parent.objects(conveyor)
    zone.run := isFree(zone.next)
  end
end
```

### 19.4 Limit the pallets in a section

```intrascript
const MAX_IN_LINE: integer := 6

on enter Roller1
  if LineA.pallets.count >= MAX_IN_LINE
    Source1.enable := false
    wait until LineA.pallets.count < MAX_IN_LINE
    Source1.enable := true
  end
end
```

### 19.5 Hourly report into a table

```intrascript
on every 1 h
  Report.addRow(now, Sink1.stats.palletsReceived, Station1.stats.utilization)
end
```

### 19.6 Shortcut with a direct move

```intrascript
// Rejected pallets jump to the rework line (not physical: see §11.1).
on enter Inspection
  if random.chance(0.02)
    if not pallet.move(Rework1)
      warn("Rework line full; pallet ", pallet.id, " stays on the main line")
    end
  end
end
```

## 20. Not included yet

User-defined record types and classes, dictionaries (tables with a key column cover lookups), reading or writing files during the run, a step-by-step debugger with breakpoints (planned with the editor), a visual block editor that writes IntraScript, changing variables of a running model from the editor, and code that runs inside a physics step.

## 21. Implementation

Steps and status: [[A12 Roadmap]] (IntraScript section). The language core is a separate package (`packages/script`) without dependencies on React, the DOM or the engines; the engines provide it with a description of the model (names, kinds, members and their types) and carry out what the code asks.

## Related

[[ADR-0018 IntraScript Scripting Language]] · [[A08 Controller]] · [[T07 Control Logic and Signals]] · [[S21 Code Block]] · [[S22 Area]]
