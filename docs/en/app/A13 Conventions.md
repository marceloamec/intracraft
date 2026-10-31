---
title: A13 Conventions
lang: en
tags: [app, conventions]
---

🌐 **EN** · [[A13 Convenciones|ES]] · [[A13 Konventionen|DE]] · [[Home (EN)|Index]]

# A13 Conventions

> **Rust platform (2026-10-31):** this note describes the TypeScript reference, which is frozen ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]). The Rust platform keeps these conventions in the engines (SI, y up, integer microseconds). The BIM model of the Design module follows BIMCraft's: millimetres, x east, y north, z up; `intracraft-model` converts at the boundary. Code rules are BIMCraft's: no `unwrap`, `panic` or `unsafe`, every change a command, validation in one place.

## 1. Model conventions

| Topic        | Rule                                                                                                                             |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| Units        | Internally **SI**: m, kg, s, rad. The UI shows mm and degrees. Convert only at the edges (UI, files) with `mmToM`, `degToRad`…   |
| Axes         | **Y up**, right-handed (same as three.js and glTF). Conveyors run along local +X; the 2D view is the top view looking toward −Y. |
| Time         | Integer **microseconds** (`SimTime`). Event order (time, priority, sequence).                                                    |
| Randomness   | Always the seeded project RNG (`createRng`). Never `Math.random()` in engines.                                                   |
| Determinism  | Same layout + same seed → same result. A failing determinism test is top priority.                                               |
| Project file | JSON `.intracraft.json` with `schemaVersion` and explicit migrations; results stored separately.                                 |
| Defaults     | Library defaults are **indicative**, never presented as manufacturer data.                                                       |

## 2. Code conventions

- TypeScript strict everywhere (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`); 2-space indentation; single quotes.
- Identifiers, parameter keys and file names in **English**; user-facing text needs `en`, with optional `es` / `de` that fall back to English ([[ADR-0003 Trilingual Documentation and English Identifiers]]).
- No `console.log` outside tests: use `createLogger` from `core-model`.
- One file per library component or command; shared state, types and helpers in support modules.
- Engines (`sim-des`, `sim-physics`, `control`) do not depend on React or the DOM.
- File order: imports → constants → types (external before internal) → public functions → private helpers below their users. Tests: constants → test cases (grouped in `describe` blocks) → helpers at the end.

## 3. Documentation conventions (this vault)

- Three parallel folders `en/`, `es/`, `de/` with the same structure and note numbers (T = theory, A = application, S = specification, ADR = decision).
- Note file names are translated and unique across the vault so `[[wikilinks]]` resolve without paths.
- Every note starts with a language bar linking the other two translations.
- Formulas use LaTeX (`$…$`, `$$…$$`); diagrams use Mermaid — both render natively in Obsidian.
- Markdown tables are padded so columns align in plain text (AGENTS.md rule).
- **English only for now (2026-10-29):** new notes and changes go to `en/` only. The `es/` and `de/` notes created so far are kept but may be outdated; English is the reference ([[ADR-0003 Trilingual Documentation and English Identifiers]], amendment).

## 4. Clean-room rules

- Never read, decompile, import or copy code, binaries, project files or assets of third-party software (including commercial simulators).
- No third-party names, logos or trademarks in the product, code or UI.
- No unlicensed manufacturer 3D models or catalogues.
- Check the licence of every new dependency (current inventory in ADR-0001).
- Other programs may be referenced only to compare observable results.

## 5. Workflow

1. Read the object's specification (write it first if it does not exist, and ask for confirmation).
2. Propose a short plan.
3. Write the test, see it fail, implement until it passes.
4. Run all tests, lint and type-check.
5. Summarise in simulation terms.
6. Small commit per working step; one branch per feature.

## Related

[[A03 Monorepo and Stack]] · [[T03 Time, Event Ordering and Determinism]]
