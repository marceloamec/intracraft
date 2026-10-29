---
title: A03 Monorepo und Stack
lang: de
tags: [app, stack, tooling]
---

🌐 [[A03 Monorepo and Stack|EN]] · [[A03 Monorepo y stack|ES]] · **DE** · [[Startseite (DE)|Index]]

# A03 Monorepo und Stack

## 1. Repository-Struktur (angelegt am 28.10.2026)

```text
./
├─ AGENTS.md · CLAUDE.md            Anweisungen für Agenten (CLAUDE.md → @AGENTS.md)
├─ package.json                     Root-Skripte und Entwicklungswerkzeuge
├─ pnpm-workspace.yaml              Workspaces: packages/*, apps/*
├─ tsconfig.base.json · tsconfig.json   striktes TypeScript; eine gemeinsame Typprüfung
├─ eslint.config.js · .prettierrc.json · .editorconfig
├─ vitest.config.ts                 führt alle *.test.ts in packages/ und apps/ aus
├─ .github/workflows/ci.yml         installieren → Lint → Tests
├─ docs/                            dieser Obsidian-Vault (en/, es/, de/)
├─ packages/
│  ├─ core-model/   Einheiten, Zeit, RNG, Logger, Schema
│  ├─ library/      Definitionsschema, Registry, Validierungsregeln, Referenzdaten, definitions/*.json
│  ├─ geometry/     Rollenanordnung
│  ├─ sim-des/      Ereigniswarteschlange, Simulator, Kinematik
│  ├─ sim-physics/  Backend-Schnittstelle, Festschritt-Uhr, Mechanik
│  ├─ control/      Wenn/Dann-Regeln
│  └─ scenarios/    analytische Szenarien (scenarios/analytic.json)
├─ apps/
│  ├─ editor/       Vite + React + react-three-fiber + Zustand
│  └─ cli/          `pnpm run scenario <id|all>`
└─ test_data/       Referenzergebnisse (nie ändern, nur damit ein Test besteht)
```

## 2. Werkzeuge und Versionen

| Werkzeug / Bibliothek       | Version          | Aufgabe                                                                |
| --------------------------- | ---------------- | ---------------------------------------------------------------------- |
| Node.js                     | ≥ 24             | Laufzeit (`.nvmrc` = 24)                                               |
| pnpm                        | 10.34.9          | Paketmanager, Workspaces                                               |
| TypeScript                  | 6.0.3            | Überall strikt (7.x wird von typescript-eslint noch nicht unterstützt) |
| Vitest                      | 5.0.5            | Unit- und Szenariotests                                                |
| ESLint + typescript-eslint  | 10.13.0 / 8.72.1 | Lint, Reinheitsregeln der Engines                                      |
| Prettier                    | 3.9.10           | Formatierung                                                           |
| Zod                         | 4.6.7            | Schemas und Validierung                                                |
| Vite + @vitejs/plugin-react | 8.3.5 / 6.1.2    | Dev-Server und Build des Editors                                       |
| React / React DOM           | 19.3.1           | Oberfläche                                                             |
| three.js                    | 0.186.1          | 3D-Darstellung                                                         |
| @react-three/fiber          | 9.8.2            | React-Renderer für three.js                                            |
| Zustand                     | 5.0.16           | UI-Zustand                                                             |
| tsx                         | 4.23.15          | Führt das TypeScript der CLI direkt aus                                |
| Physik-Engine               | —                | Entscheidung in Spike S1 ([[ADR-0002 Physik-Engine-Kandidat]])         |

Begründung und Alternativen: [[ADR-0001 Werkzeuge und Versionen]].

## 3. Befehle

| Befehl                        | Was er tut                                                           |
| ----------------------------- | -------------------------------------------------------------------- |
| `pnpm install`                | Installiert alle Workspace-Abhängigkeiten                            |
| `pnpm test`                   | Führt alle Tests einmal aus (Vitest)                                 |
| `pnpm test:watch`             | Wiederholt Tests bei Änderungen                                      |
| `pnpm lint`                   | ESLint + TypeScript-Typprüfung des ganzen Repositorys                |
| `pnpm dev`                    | Startet den Editor unter http://localhost:5173                       |
| `pnpm build`                  | Produktions-Build des Editors                                        |
| `pnpm run scenario <id\|all>` | Analytische Szenarien ohne Oberfläche (z. B. `pnpm run scenario t1`) |
| `pnpm format`                 | Formatiert Code mit Prettier (Markdown ausgenommen)                  |
| `pnpm format:docs`            | Richtet die Markdown-Tabellen in docs/ aus (Regel aus AGENTS.md)     |

## 4. Wie Pakete sich referenzieren

Pakete sind **quellcodebasiert**: Jede `package.json` exportiert direkt `./src/index.ts`; zwischen Paketen gibt es keinen Build-Schritt. Vite, Vitest und `tsx` übersetzen TypeScript im Flug, und die Root-`tsconfig.json` prüft alle Typen in einem Durchlauf (`moduleResolution: bundler`). Interne Abhängigkeiten nutzen `"workspace:*"`.

### Ein Paket hinzufügen

1. `packages/<name>/package.json` mit `"name": "@intracraft/<name>"`, `"type": "module"` und `"exports": { ".": "./src/index.ts" }` anlegen.
2. `src/index.ts` und mindestens eine `*.test.ts` hinzufügen.
3. `pnpm install` ausführen, damit der Workspace es verknüpft.
4. [[A02 Architektur]], diese Notiz und AGENTS.md aktualisieren, wenn sich die Struktur ändert.

## Verwandt

[[A02 Architektur]] · [[A13 Konventionen]] · [[A14 API-Referenz]]
