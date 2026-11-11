---
title: ADR-0027 IntraScript in Python Syntax with Its Own Interpreter
lang: en
tags: [decision, scripting, control, clean-room]
status: accepted
date: 2026-11-09
---

# ADR-0027 IntraScript in Python Syntax with Its Own Interpreter

- **Status:** Accepted (2026-11-09). Amends [[ADR-0018 IntraScript Scripting Language]] in its syntax and interpreter;
  keeps its scope (code blocks on the plan, events, waits, units, exit decisions, determinism, sandbox).
- **Date:** 2026-11-09

## Context

ADR-0018 specified IntraScript with a Pascal-like syntax (`:=`, blocks closed by `end`, not case-sensitive, counting from 1)
and left ten points to the owner (Q20 in [[Open Questions]]). On 2026-11-09 the owner asked to build the language now,
without further questions: an open-source language of IntraCraft's own, dedicated to simulation, that "may be based on
Python, or be a simulation library with Python's syntax", with an object placed in the model whose text editor holds the
code, as the method objects of commercial plant simulators. The owner again pointed to an unofficial style guide of a
commercial simulator's language as inspiration; it was not read again (ADR-0018 recorded the one reading, for concepts).

## Decision

1. **Python's syntax.** IntraScript code is written as Python 3: indentation, `def`, `if`/`elif`/`else`, `for … in`,
   `while`, `=` and `==`, `True`/`False`/`None`, lists, tuples, dictionaries, f-strings, list comprehensions, slices,
   case-sensitive names, counting from 0. This answers Q20 a–c with Python's choices.
2. **Simulation as a library with Python's syntax.** What a plant needs is expressed with Python constructs, so the code
   stays valid Python syntax:
   - events are **decorators**: `@on_start`, `@on_finish`, `@on_create(Source)`, `@on_enter(X)`, `@on_leave(X)`,
     `@on_rise(X.signal)`, `@on_fall`, `@on_change`, `@every(10 * minute, start=…)`, `@at(2 * h)`, `@choose_exit(Turntable)`,
     with `priority=`;
   - waiting is a call: `wait(2 * s)` and `wait_until(condition, timeout=…)` (a special form: its condition is looked at
     again after every event);
   - **units are built-in values**: `150 * mm` is a length, `gap / mm` a plain number; quantities carry their dimension
     and mixing dimensions is an error;
   - objects of the layout are names; their members come from the library definitions, in snake_case.
3. **Its own interpreter, not CPython.** `intracraft-script` (`platform/sim/script`): lexer, parser, checker, compiler to
   instructions and a runtime of **suspendable activations**, in safe Rust with no dependencies and no `eval`. Checked
   against the layout before a run; limits on steps, call depth, cascades and sizes; deterministic (no clock, files or
   network; one seeded random stream per block).
4. **A subset of Python.** Left out on purpose, each with a syntax error that names it: classes, `lambda`, exceptions,
   generators, `async`, `with`, `del`, `nonlocal`, nested functions, `*args`, sets, `:=`. Python's scoping is kept, with
   `global` to change a block variable, and a checker warning for the usual trap.
5. **The code block is a library object**, `info.code` (Rust-only definition, `platform/sim/library/definitions`), placed
   on the plan in the Design module, with its code in the text attribute `source` and the switch `enabled`. Changing the
   code is BIMCraft's `modify` command: undo, log and agents need nothing new. No new field in the project file (instead
   of ADR-0018's `content`), no new command (instead of `script.setCode`).
6. **The editor** is a window of the Design module: colours, line numbers, check while typing with problem lines
   underlined and listed, the names the code can use, Ctrl+S. The Discrete simulation module shows the code's error,
   console and variables; `intracraft-cli code check|run` does the same from a terminal.
7. **Python editors.** `platform/sim/script/python/intrascript.py` describes every IntraScript name, so
   `from intrascript import *` gives completion and help in any Python editor. It does not simulate.
8. **Open source:** the crate, the editor, the stub module and the language reference are under the repository's licence
   (MIT OR Apache-2.0).
9. **Answers to the rest of Q20:** parameters read-only during a run (d, as written); at one instant the If/Then rules run
   first, then the code (e); exit decisions override the exit rule and `None` falls back to it (g); limits as A17 §16.4 (h);
   block variables start fresh at every run (i). Teleport moves (f) and area membership (j) wait for those features.

## Clean-room review

The syntax is Python's (an open language definition, PSF); its grammar, semantics and built-in names were written from
general knowledge of Python, not from CPython's source. No code, text, keyword list or example was taken from a
commercial simulator or from the style guide the owner linked; the commercial product's name appears nowhere in the
product. Event decorators, `wait_until` as a special form, units as values and the checker's codes are IntraCraft's own
design.

## Alternatives considered

- **Keep ADR-0018's Pascal-like syntax.** Readable for engineers who know commercial simulators' languages, but a language
  nobody else uses: no editor support, no examples, and AI assistants write Python far more reliably. Not chosen, at the
  owner's suggestion.
- **Embed CPython** (PyO3) **or RustPython.** Real Python and its libraries, but: a large dependency to vendor and build
  offline, no sandbox worth the name (files, network, the clock), no determinism guarantee, no physical units, and waiting
  inside a handler would need generators or `async` in user code. Not chosen.
- **A SimPy-style library over generators** (`yield env.timeout(2)`). Real Python semantics for waits, but `yield` in every
  waiting function is hard to explain to non-programmers and forbids waiting in called functions without `yield from`.
  The runtime's suspendable activations let a plain `wait()` pause anywhere.
- **Units as suffixes** (`150 mm`). Shorter, but not Python syntax: no editor would colour it and the stub module couldn't
  describe it. `150 * mm` costs one character and stays Python.
- **Information objects (variables, tables, lists) first.** Specified in S18–S20; Python's lists and dictionaries inside
  a block cover the first models, so the code block came first.

## Consequences

- IntraScript 1.0 runs in the **Discrete simulation module**: events from the 2D run (pallets created, entering and
  leaving objects, signals changing), signal writes, timers in the run's own queue, exit decisions of turntables. The
  reference scenarios IS-1 to IS-3 of [[S21 Code Block]] pass.
- The **Physics simulation module** runs a layout's physics without its code for now. Next step: the same hook in the 3D
  run's controller cycle (every 10 ms), then the 2D/3D agreement tests on IS-1 to IS-3.
- `pallet.move`, `Source.create()`, `pallet.delete()`, areas and the variable, table and list objects stay specified for
  later steps (A17 §22).
- The library has a 15th definition (`info.code`, category `control`, `des.script`); the 2D network leaves it out
  (`SCRIPT_BEHAVIOR`), so the 3D run never sees it. The simulation tree lists code blocks in their own group.
- A layout whose enabled code block has a check error does not start a run; the message names the block and the line.

## Related

[[A17 IntraScript Language]] · [[A20 IntraScript Tutorial]] · [[S21 Code Block]] · [[ADR-0018 IntraScript Scripting Language]] ·
[[ADR-0010 Controller Rules in the Layout and in 2D]]
