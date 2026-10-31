---
title: ADR-0001 Toolchain and Versions
lang: en
tags: [decision, tooling]
status: accepted
date: 2026-10-28
---

🌐 **EN** · [[ADR-0001 Herramientas y versiones|ES]] · [[ADR-0001 Werkzeuge und Versionen|DE]] · [[Home (EN)|Index]]

# ADR-0001 Toolchain and Versions

- **Status:** Accepted **Rust platform (2026-10-31):** superseded for the product by [[ADR-0020 Rust Platform on BIMCraft with Three Modules]]; it still describes the TypeScript reference, which is frozen.
- **Date:** 2026-10-28

## Context

The plan fixes the stack (TypeScript strict, pnpm workspaces, Vite, Vitest, ESLint, Prettier, Zod, React, react-three-fiber, Zustand). AGENTS.md asks to look up current versions instead of assuming them and to record the result here. Versions were queried from the npm registry on 2026-10-28.

## Decision

| Item                 | Chosen                   | Latest seen | Why not latest / notes                                                             |
| -------------------- | ------------------------ | ----------- | ---------------------------------------------------------------------------------- |
| Node.js              | ≥ 24 (24.22.0 installed) | —           | LTS line available on the machine                                                  |
| pnpm                 | 10.34.9                  | 12.16.1     | Mature major; `onlyBuiltDependencies` used for esbuild. Upgrade later deliberately |
| TypeScript           | 6.0.3                    | 7.0.2       | typescript-eslint 8.72.1 requires TypeScript < 6.1                                 |
| typescript-eslint    | 8.72.1                   | 8.72.1      | —                                                                                  |
| ESLint / @eslint/js  | 10.13.0 / 10.0.1         | same        | Flat config                                                                        |
| Vitest               | 5.0.5                    | same        | —                                                                                  |
| Vite / plugin-react  | 8.3.5 / 6.1.2            | same        | —                                                                                  |
| React / React DOM    | 19.3.1                   | same        | @react-three/fiber 9.8.2 requires React < 19.4                                     |
| three / @types/three | 0.186.1 / 0.186.0        | same        | —                                                                                  |
| @react-three/fiber   | 9.8.2                    | same        | —                                                                                  |
| @react-three/drei    | **not added**            | 10.7.9      | Avoid a heavy dependency; OrbitControls comes from `three/examples`                |
| Zustand              | 5.0.16                   | same        | —                                                                                  |
| Zod                  | 4.6.7                    | same        | —                                                                                  |
| tsx                  | 4.23.15                  | same        | Runs the CLI TypeScript without a build step                                       |
| Physics engine       | **not installed**        | —           | Decided in spike S1 ([[ADR-0002 Physics Engine Candidate]])                        |

All versions are pinned exactly in `package.json` files. Packages are **source-first** (`exports: ./src/index.ts`), so there is no build step between packages.

## Alternatives considered

- TypeScript 7 (native compiler): faster, but would break the ESLint type-aware tooling today.
- Building each package to `dist/`: more ceremony, slower feedback; not needed while everything runs through Vite/Vitest/tsx.

## Consequences

- One root `tsc --noEmit` checks the whole monorepo (`pnpm lint`).
- Upgrading TypeScript to 7.x waits for typescript-eslint support.
- Licences (`pnpm licenses list`, 2026-10-28, 194 packages): MIT 156, Apache-2.0 16, ISC 9, BSD-2/3-Clause 9, plus three transitive build-tool packages — `lightningcss` (MPL-2.0, weak file-level copyleft, used unmodified), `caniuse-lite` (CC-BY-4.0, browser data) and `minimatch` (BlueOak-1.0.0, permissive). None is shipped modified; re-check on every new dependency.
