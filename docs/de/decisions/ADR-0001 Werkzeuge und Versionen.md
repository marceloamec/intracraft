---
title: ADR-0001 Werkzeuge und Versionen
lang: de
tags: [decision, tooling]
status: accepted
date: 2026-10-28
---

🌐 [[ADR-0001 Toolchain and Versions|EN]] · [[ADR-0001 Herramientas y versiones|ES]] · **DE** · [[Startseite (DE)|Index]]

# ADR-0001 Werkzeuge und Versionen

- **Status:** Angenommen
- **Datum:** 28.10.2026

## Kontext

Der Plan legt den Stack fest (striktes TypeScript, pnpm-Workspaces, Vite, Vitest, ESLint, Prettier, Zod, React, react-three-fiber, Zustand). AGENTS.md verlangt, aktuelle Versionen nachzuschlagen statt anzunehmen und das Ergebnis hier festzuhalten. Die Versionen wurden am 28.10.2026 in der npm-Registry abgefragt.

## Entscheidung

| Element              | Gewählt                    | Neueste gesehen | Warum nicht die neueste / Hinweise                                                          |
| -------------------- | -------------------------- | --------------- | ------------------------------------------------------------------------------------------- |
| Node.js              | ≥ 24 (24.22.0 installiert) | —               | Auf dem Rechner verfügbare LTS-Linie                                                        |
| pnpm                 | 10.34.9                    | 12.16.1         | Ausgereifte Hauptversion; `onlyBuiltDependencies` für esbuild. Später gezielt aktualisieren |
| TypeScript           | 6.0.3                      | 7.0.2           | typescript-eslint 8.72.1 verlangt TypeScript < 6.1                                          |
| typescript-eslint    | 8.72.1                     | 8.72.1          | —                                                                                           |
| ESLint / @eslint/js  | 10.13.0 / 10.0.1           | gleich          | Flat Config                                                                                 |
| Vitest               | 5.0.5                      | gleich          | —                                                                                           |
| Vite / plugin-react  | 8.3.5 / 6.1.2              | gleich          | —                                                                                           |
| React / React DOM    | 19.3.1                     | gleich          | @react-three/fiber 9.8.2 verlangt React < 19.4                                              |
| three / @types/three | 0.186.1 / 0.186.0          | gleich          | —                                                                                           |
| @react-three/fiber   | 9.8.2                      | gleich          | —                                                                                           |
| @react-three/drei    | **nicht aufgenommen**      | 10.7.9          | Schwere Abhängigkeit vermeiden; OrbitControls kommt aus `three/examples`                    |
| Zustand              | 5.0.16                     | gleich          | —                                                                                           |
| Zod                  | 4.6.7                      | gleich          | —                                                                                           |
| tsx                  | 4.23.15                    | gleich          | Führt das CLI-TypeScript ohne Build aus                                                     |
| Physik-Engine        | **nicht installiert**      | —               | Entscheidung in Spike S1 ([[ADR-0002 Physik-Engine-Kandidat]])                              |

Alle Versionen sind in den `package.json`-Dateien exakt festgelegt. Pakete sind **quellcodebasiert** (`exports: ./src/index.ts`), ohne Build-Schritt zwischen Paketen.

## Betrachtete Alternativen

- TypeScript 7 (nativer Compiler): schneller, würde aber heute die typbasierten ESLint-Werkzeuge brechen.
- Jedes Paket nach `dist/` bauen: mehr Aufwand, langsameres Feedback; unnötig, solange alles über Vite/Vitest/tsx läuft.

## Folgen

- Ein einziges `tsc --noEmit` im Root prüft das ganze Monorepo (`pnpm lint`).
- Der Umstieg auf TypeScript 7.x wartet auf Unterstützung durch typescript-eslint.
- Lizenzen (`pnpm licenses list`, 28.10.2026, 194 Pakete): MIT 156, Apache-2.0 16, ISC 9, BSD-2/3-Clause 9, dazu drei transitive Build-Werkzeuge – `lightningcss` (MPL-2.0, schwaches Copyleft auf Dateiebene, unverändert genutzt), `caniuse-lite` (CC-BY-4.0, Browserdaten) und `minimatch` (BlueOak-1.0.0, permissiv). Keines wird verändert ausgeliefert; bei jeder neuen Abhängigkeit erneut prüfen.
