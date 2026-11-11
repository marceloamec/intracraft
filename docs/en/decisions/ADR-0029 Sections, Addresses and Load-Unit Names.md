---
title: ADR-0029 Sections, Addresses and Load-Unit Names
lang: en
tags: [decision, design, scripting, naming]
status: accepted (core and IntraScript); the section filter and the load-unit panel follow
date: 2026-11-09
---

# ADR-0029 Sections, Addresses and Load-Unit Names

- **Status:** Accepted for the model, the layout and IntraScript. The section visibility (hide or show by section) and
  the load-unit panel are the next steps of the same plan.
- **Date:** 2026-11-09

## Context

The owner asked that every element of the plant can be referred to by a path, in the way a plant engineer names things
in a simulator: first the **section** (a group of elements, like a layer), then the object, then the load units on it,
for example `Seccion1.Conveyor1` and `Seccion1.Conveyor1.MU1`. Each load unit must have a unique id from the moment the
source creates it, and its own properties. Sections are not necessarily one level: a section can span several levels,
and the owner wants to hide or show by section and by level.

## Decision

1. **A section is a project parameter.** `Section` is a text parameter of components, like `Name` (`set_name` in the
   model), so it lives in the BIM model and no new object is needed. Assigning a section is a `modify` command: one undo step, checked
   by `Model::check`, open to agents. A section exists while an object uses it. Sections do not nest for now.
2. **An address is the section and the object's name, joined by `.`:** `Seccion1.Conveyor1`; an object with no section
   has the address `Conveyor1`. A section name cannot hold `.`.
3. **A load unit (MU) is named by its pallet id:** `MU<n>`, where `n` is the number the run gives it when the source
   creates it, unique in the run and the same in the discrete and the physics engine. Its **address** is its carrier's
   address and its name: `Seccion1.Conveyor1.MU3`. The number never changes; the address changes when the unit moves to
   the next object.
4. **IntraScript reads addresses.** `Seccion1.Conveyor1.run` names an object in code (the checker resolves whole
   addresses before it resolves names, so plain names keep working). `pallet.name` and `pallet.address` read a load
   unit's name and address; both are read-only.
5. **The layout carries the section and the address** in the free `attributes` of each instance (`section`, `address`).
   The project file format does not change.

## Alternatives considered

- **Sections as levels (BIMCraft's own).** Nothing to build, but a level is a height (a floor), not a zone of the plant,
  and levels do not nest. It stays available for hiding and showing by level, which the owner also asked for.
- **Sections as rectangles on the plan (S22 Area).** Membership by position is attractive, but a layer is a group the
  designer chooses, and an object may belong to a section without lying inside a rectangle. The S22 specification
  stays for later: it can become a way to create a section, not the way sections are stored.
- **A load-unit id that encodes the address** (`Seccion1.Conveyor1.MU3` as the id). Rejected: the address changes as
  the unit moves, so the id would change with it; the number stays and the address is derived.

## Consequences

- The layout gained two attributes per instance; old projects have no section and keep plain names.
- The two engines count load units separately, so a run's `MU<n>` numbers are unique within that run only, which is
  what a run needs.
- Two objects with the same address would be ambiguous. The layout reports each such address as a problem when it is
  built (`layout()` in the model), so the run and the editor list it; the run still starts, as with other warnings.

## Related

[[ADR-0018 IntraScript Scripting Language]] · [[ADR-0027 IntraScript in Python Syntax with Its Own Interpreter]] ·
[[A17 IntraScript Language]] · [[S22 Area]]
