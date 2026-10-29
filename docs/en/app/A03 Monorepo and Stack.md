---
title: A03 Monorepo and Stack
lang: en
tags: [app, stack, tooling]
---

🌐 **EN** · [[A03 Monorepo y stack|ES]] · [[A03 Monorepo und Stack|DE]] · [[Home (EN)|Index]]

# A03 Monorepo and Stack

## 1. Repository layout (as created on 2026-10-28)

```text
./
├─ AGENTS.md · CLAUDE.md            agent instructions (CLAUDE.md → @AGENTS.md)
├─ package.json                     root scripts and dev tooling
├─ pnpm-workspace.yaml              workspaces: packages/*, apps/*
├─ tsconfig.base.json · tsconfig.json   strict TypeScript; one root type-check
├─ eslint.config.js · .prettierrc.json · .editorconfig
├─ vitest.config.ts                 runs every *.test.ts in packages/ and apps/
├─ .github/workflows/ci.yml         install → lint → test
├─ docs/                            this Obsidian vault (en/, es/, de/)
├─ packages/
│  ├─ core-model/   units, time, rng, logger, schema
│  ├─ library/      definition schema, registry, validation rules, reference data, definitions/*.json
│  ├─ geometry/     roller layout
│  ├─ sim-des/      event queue, simulator, kinematics
│  ├─ sim-physics/  backend interface, fixed-step clock, mechanics
│  ├─ control/      If/Then rules
│  └─ scenarios/    analytic scenarios (scenarios/analytic.json)
├─ apps/
│  ├─ editor/       Vite + React + react-three-fiber + Zustand
│  └─ cli/          `pnpm run scenario <id|all>`
└─ test_data/       golden results (never edited to make a test pass)
```

## 2. Toolchain and versions

| Tool / library              | Version          | Role                                                                |
| --------------------------- | ---------------- | ------------------------------------------------------------------- |
| Node.js                     | ≥ 24             | Runtime (`.nvmrc` = 24)                                             |
| pnpm                        | 10.34.9          | Package manager, workspaces                                         |
| TypeScript                  | 6.0.3            | Strict mode everywhere (7.x not yet supported by typescript-eslint) |
| Vitest                      | 5.0.5            | Unit and scenario tests                                             |
| ESLint + typescript-eslint  | 10.13.0 / 8.72.1 | Lint, engine purity rules                                           |
| Prettier                    | 3.9.10           | Formatting                                                          |
| Zod                         | 4.6.7            | Schemas and validation                                              |
| Vite + @vitejs/plugin-react | 8.3.5 / 6.1.2    | Editor dev server and build                                         |
| React / React DOM           | 19.3.1           | UI                                                                  |
| three.js                    | 0.186.1          | 3D rendering                                                        |
| @react-three/fiber          | 9.8.2            | React renderer for three.js                                         |
| Zustand                     | 5.0.16           | UI state                                                            |
| tsx                         | 4.23.15          | Runs the CLI's TypeScript directly                                  |
| Physics engine              | —                | Decided in spike S1 ([[ADR-0002 Physics Engine Candidate]])         |

Rationale and alternatives: [[ADR-0001 Toolchain and Versions]].

## 3. Commands

| Command                       | What it does                                                  |
| ----------------------------- | ------------------------------------------------------------- |
| `pnpm install`                | Install all workspace dependencies                            |
| `pnpm test`                   | Run all tests once (Vitest)                                   |
| `pnpm test:watch`             | Re-run tests on change                                        |
| `pnpm lint`                   | ESLint + TypeScript type-check of the whole repository        |
| `pnpm dev`                    | Start the editor at http://localhost:5173                     |
| `pnpm build`                  | Production build of the editor                                |
| `pnpm run scenario <id\|all>` | Run analytic scenarios headless (e.g. `pnpm run scenario t1`) |
| `pnpm format`                 | Format code with Prettier (Markdown is excluded)              |
| `pnpm format:docs`            | Align Markdown tables in docs/ (AGENTS.md rule)               |

## 4. How packages reference each other

Packages are **source-first**: each `package.json` exports `./src/index.ts` directly; there is no build step between packages. Vite, Vitest and `tsx` compile TypeScript on the fly, and the root `tsconfig.json` type-checks everything in one pass (`moduleResolution: bundler`). Internal dependencies use `"workspace:*"`.

### Adding a package

1. Create `packages/<name>/package.json` with `"name": "@intracraft/<name>"`, `"type": "module"` and `"exports": { ".": "./src/index.ts" }`.
2. Add `src/index.ts` and at least one `*.test.ts`.
3. Run `pnpm install` so the workspace links it.
4. Update [[A02 Architecture]], this note and AGENTS.md if the structure changes.

## Related

[[A02 Architecture]] · [[A13 Conventions]] · [[A14 API Reference]]
