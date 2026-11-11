---
title: ADR-0018 IntraScript Scripting Language
lang: en
tags: [decision, scripting, control, scope, clean-room]
status: accepted — syntax and interpreter amended by ADR-0027
date: 2026-10-31
---

# ADR-0018 IntraScript Scripting Language

- **Status:** Accepted (2026-11-09) with the syntax and interpreter of [[ADR-0027 IntraScript in Python Syntax with Its Own Interpreter]]: Python's syntax (decorators for events, `wait()`, units as values such as `150 * mm`) and IntraCraft's own Rust interpreter; points 3 (Pascal-like syntax), 4 (`content` field) and 9 (`script.*` commands) below are replaced there, the rest stands. Earlier: proposed. On 2026-10-31 the owner asked for the language and chose its name, where information objects live and what code may do with pallets (§ Decision 1, 4 and 6). The language reference [[A17 IntraScript Language]] and the specifications [[S18 Variable]] to [[S22 Area]] wait for the owner's confirmation before any code is written; points to confirm are in [[Open Questions]] Q20. **Rust platform (2026-10-31):** the interpreter is written in Rust as `intracraft-script` (`platform/sim/script`, phase R10 of [[A19 Rust Implementation Plan]]) instead of a TypeScript package; the language itself does not change.
- **Date:** 2026-10-31

## Context

Behaviour is set today with component parameters and with If/Then rules over boolean signals ([[ADR-0010 Controller Rules in the Layout and in 2D]], [[A08 Controller]]). Rules cannot keep data (counters, order lists), decide by pallet data, wait inside a sequence or repeat logic over many objects. The MVP plan (§2.2, §4.3, §14.3) and `AGENTS.md` ("Qué no hacer") kept **user scripting** out of the MVP and suggested a sandboxed JavaScript engine for later.

On 2026-10-31 the owner asked for a programming language of IntraCraft's own, inspired by the scripting languages of plant simulators: an object structure with hierarchy and references, methods, tables, lists, variables and code blocks. This ADR records that scope extension and the main design choices; the language itself is specified in A17.

## Decision

1. **Name:** IntraScript (chosen by the owner). Code blocks are stored in the project; there is no separate file type.
2. **Own language, own interpreter.** A small language designed for this simulator, parsed, checked and run by a TypeScript package (`packages/script`) without third-party dependencies and without `eval`. It runs inside the simulation worker.
3. **Main traits** ([[A17 IntraScript Language]]):
   - English keywords, blocks closed with `end`, one statement per line, not case-sensitive, indexes from 1;
   - static types checked before the run, with **physical units as types** (`length`, `time`, `speed`…) and unit literals (`150 mm`, `0.25 m/s`);
   - events written in the code itself (`on enter Roller3`, `on Photoeye1.detected rises`, `on every 10 min`, `on choose exit of Turntable1`);
   - `wait` and `wait until … timeout …` inside sequences, with each activation suspended and resumed in event order;
   - references with words instead of symbols: `self`, `parent`, `root`, `pallet`, `sender`, `now`;
   - object members generated from the component definitions (parameters with their units, signals, statistics, free attributes), so new components need no extra scripting work.
4. **Information objects are objects of the layout** with an icon on the 2D plan (chosen by the owner): variable, table, list, code block and area (`info.*`, specifications S18–S22). They reuse the existing commands (add, move, parameters, remove, undo, log). Their content (initial value, cells, items, code) goes into a new optional `content` field of the instance, and membership of an area into an optional `parentId`; old project files stay valid without a migration.
5. **Hierarchy through areas** ([[S22 Area]]): `root` → areas → objects, with paths `Hall1.LineA.Roller3` and short names resolved from the code block's own area outwards, so duplicated areas keep working code.
6. **Code may move, create and delete pallets directly** (chosen by the owner), besides commanding the equipment. In 3D a direct move is a **teleport**: the rigid body is removed and created again at the target's entry with the deck speed, only if nothing overlaps there; it reuses the live-creation path of [[ADR-0017 Scenery Assets in the Layout and Live Spawn]]. Both modes log every direct move as `teleport`.
7. **Rules stay.** At one instant the If/Then rules run first, then the code ([[A08 Controller]]).
8. **Determinism and sandbox:** seeded random numbers with one stream per code block (derived from its id), no clock, files or network, and limits on steps, call depth, cascades and sizes (A17 §14.5). A run error stops the run with block, line, time and message; it never freezes the editor.
9. **Commands:** `script.setCode`, `data.setVariable`, `data.setTable`, `data.setCells`, `data.setItems`, `layout.setParent`; query `script.check`. Code with check errors can be saved; it only blocks the start of a run.

## Clean-room review

The owner provided an unofficial style guide for the scripting language of a commercial plant simulator, written by a third party and published on GitHub Pages, and asked to use it as inspiration. It was read once on 2026-10-31 **for concepts only**, under the same terms as [[ADR-0011 Reference Review of a Commercial Simulator Manual]]. The guide describes coding style (formatting, naming, comments); it contains no product code. No text, example, keyword list, symbol or name was copied; the product's name does not appear in IntraCraft.

| Concept seen in the guide (common to many languages) | IntraScript                                                                     |
| ---------------------------------------------------- | ------------------------------------------------------------------------------- |
| Typed declarations of variables and parameters       | `var name: type := value` — the Pascal-family form, also used by Ada and Eiffel |
| Default values for trailing parameters               | Kept; IntraScript adds named arguments                                          |
| Pass-by-reference parameters                         | Not adopted: lists and objects are shared, numbers are copied (A17 §4.5)        |
| Blocks closed by `end`, no semicolons                | Kept (also Lua, Ruby, Eiffel)                                                   |
| Line comments                                        | `//`; help text with `///` (own form)                                           |
| Methods with typed result                            | `method name(…) -> type`, several methods per code block                        |
| Naming conventions and short methods                 | Own style guide (A17 §15), with rules about units in types                      |

Designed for IntraCraft and not taken from the guide: unit types and literals, events declared with `on`, exit decisions, `wait until … timeout` with `timedOut`, word references instead of symbols, pallet attribute declarations, areas with relative names, per-block random streams, and the 2D/3D execution model. The general notion of methods, tables, variables and event controls in plant simulators was already recorded from the introductory manual ([[ADR-0011 Reference Review of a Commercial Simulator Manual]]).

## Alternatives considered

- **JavaScript in a sandbox (QuickJS)**, as the MVP plan suggested. Rejected: no physical units, waiting inside a sequence needs asynchronous code that is hard for non-programmers, and the error messages speak about JavaScript, not about the model.
- **An existing small language (Lua compiled to WebAssembly).** Same problems with units and waiting, plus a binary dependency and a second runtime to keep deterministic.
- **Rules only, with more condition types.** Cannot hold data, loop over objects or describe sequences; the form-based editor would grow into a language anyway.
- **A visual block editor.** Easier for the first steps but slow for real models and hard to diff; it may later be a front end that writes IntraScript (A17 §20).
- **Information objects in a separate data tree** (offered to the owner): cleaner plan, but no hierarchy by area and new editing commands for everything. Not chosen.
- **Code that only commands the equipment, never moves pallets** (offered to the owner as the recommended option): the same code would give comparable 2D and 3D results. Not chosen; direct moves are allowed and marked as teleports.

## Consequences

- User scripting enters the scope. `AGENTS.md` and the MVP plan no longer list it as excluded; the other exclusions stay.
- The schema of the project file gains two optional fields (`content`, `parentId`); `schemaVersion` stays at 1. Code blocks carry their own `languageVersion`.
- The definition schema gains the category `information` and the five `info.*` definitions.
- Direct pallet moves can make 2D and 3D disagree (photocells do not see a teleported pallet); the run log marks them so results can be read correctly.
- Code that uses `pallet.move` or `delete()` falls outside the 2D/3D agreement tests, which use scenarios without them.
- New work: about 5–8 work sessions, in the steps below. Each step gets its tests first and a short summary in simulation terms.

## Implementation steps

| Step | Content                                                                                             |
| ---- | --------------------------------------------------------------------------------------------------- |
| 1    | Documentation: A17, S18–S22, this ADR (this change)                                                 |
| 2    | `packages/script`: lexer, parser, checker of types and units, interpreter with waiting activations  |
| 3    | Model and commands: `content`, `parentId`, `info.*` definitions, `script.*` and `data.*` commands   |
| 4    | 2D: events, waits, members from definitions, pallet actions, exit decisions; scenarios IS-1 to IS-3 |
| 5    | Editor: plan icons, code panel, table grid, console; the code editor library is asked for first     |
| 6    | 3D: code on the controller cycle, teleport moves, 2D/3D agreement on IS-1 to IS-3                   |

Status per step: [[A12 Roadmap]].
