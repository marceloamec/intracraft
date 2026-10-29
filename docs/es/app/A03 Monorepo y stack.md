---
title: A03 Monorepo y stack
lang: es
tags: [app, stack, tooling]
---

🌐 [[A03 Monorepo and Stack|EN]] · **ES** · [[A03 Monorepo und Stack|DE]] · [[Inicio (ES)|Índice]]

# A03 Monorepo y stack

## 1. Estructura del repositorio (creada el 28-10-2026)

```text
./
├─ AGENTS.md · CLAUDE.md            instrucciones para agentes (CLAUDE.md → @AGENTS.md)
├─ package.json                     scripts raíz y herramientas de desarrollo
├─ pnpm-workspace.yaml              espacios de trabajo: packages/*, apps/*
├─ tsconfig.base.json · tsconfig.json   TypeScript estricto; una única comprobación de tipos
├─ eslint.config.js · .prettierrc.json · .editorconfig
├─ vitest.config.ts                 ejecuta todos los *.test.ts de packages/ y apps/
├─ .github/workflows/ci.yml         instalar → lint → pruebas
├─ docs/                            esta bóveda de Obsidian (en/, es/, de/)
├─ packages/
│  ├─ core-model/   unidades, tiempo, rng, logger, esquema
│  ├─ library/      esquema de definiciones, registro, reglas de validación, datos de referencia, definitions/*.json
│  ├─ geometry/     disposición de rodillos
│  ├─ sim-des/      cola de eventos, simulador, cinemática
│  ├─ sim-physics/  interfaz de backend, reloj de paso fijo, mecánica
│  ├─ control/      reglas Si/Entonces
│  └─ scenarios/    escenarios analíticos (scenarios/analytic.json)
├─ apps/
│  ├─ editor/       Vite + React + react-three-fiber + Zustand
│  └─ cli/          `pnpm run scenario <id|all>`
└─ test_data/       resultados de referencia (nunca se editan para que pase una prueba)
```

## 2. Herramientas y versiones

| Herramienta / librería      | Versión          | Función                                                             |
| --------------------------- | ---------------- | ------------------------------------------------------------------- |
| Node.js                     | ≥ 24             | Entorno de ejecución (`.nvmrc` = 24)                                |
| pnpm                        | 10.34.9          | Gestor de paquetes, espacios de trabajo                             |
| TypeScript                  | 6.0.3            | Modo estricto en todo (typescript-eslint aún no admite la 7.x)      |
| Vitest                      | 5.0.5            | Pruebas unitarias y de escenarios                                   |
| ESLint + typescript-eslint  | 10.13.0 / 8.72.1 | Linter, reglas de pureza de los motores                             |
| Prettier                    | 3.9.10           | Formato                                                             |
| Zod                         | 4.6.7            | Esquemas y validación                                               |
| Vite + @vitejs/plugin-react | 8.3.5 / 6.1.2    | Servidor de desarrollo y compilación del editor                     |
| React / React DOM           | 19.3.1           | Interfaz                                                            |
| three.js                    | 0.186.1          | Dibujo 3D                                                           |
| @react-three/fiber          | 9.8.2            | Renderizador React para three.js                                    |
| Zustand                     | 5.0.16           | Estado de la interfaz                                               |
| tsx                         | 4.23.15          | Ejecuta directamente el TypeScript de la CLI                        |
| Motor de física             | —                | Se decide en el spike S1 ([[ADR-0002 Candidato a motor de física]]) |

Motivos y alternativas: [[ADR-0001 Herramientas y versiones]].

## 3. Comandos

| Comando                       | Qué hace                                                                   |
| ----------------------------- | -------------------------------------------------------------------------- |
| `pnpm install`                | Instala todas las dependencias del espacio de trabajo                      |
| `pnpm test`                   | Ejecuta todas las pruebas una vez (Vitest)                                 |
| `pnpm test:watch`             | Repite las pruebas al cambiar archivos                                     |
| `pnpm lint`                   | ESLint + comprobación de tipos de todo el repositorio                      |
| `pnpm dev`                    | Arranca el editor en http://localhost:5173                                 |
| `pnpm build`                  | Compilación de producción del editor                                       |
| `pnpm run scenario <id\|all>` | Ejecuta escenarios analíticos sin interfaz (p. ej. `pnpm run scenario t1`) |
| `pnpm format`                 | Formatea el código con Prettier (el Markdown queda excluido)               |
| `pnpm format:docs`            | Alinea las tablas Markdown de docs/ (regla de AGENTS.md)                   |

## 4. Cómo se referencian los paquetes

Los paquetes son **de código fuente directo**: cada `package.json` exporta `./src/index.ts`; no hay paso de compilación entre paquetes. Vite, Vitest y `tsx` compilan TypeScript al vuelo, y el `tsconfig.json` raíz comprueba los tipos de todo en una pasada (`moduleResolution: bundler`). Las dependencias internas usan `"workspace:*"`.

### Añadir un paquete

1. Crear `packages/<nombre>/package.json` con `"name": "@intracraft/<nombre>"`, `"type": "module"` y `"exports": { ".": "./src/index.ts" }`.
2. Añadir `src/index.ts` y al menos un `*.test.ts`.
3. Ejecutar `pnpm install` para que el espacio de trabajo lo enlace.
4. Actualizar [[A02 Arquitectura]], esta nota y AGENTS.md si cambia la estructura.

## Relacionado

[[A02 Arquitectura]] · [[A13 Convenciones]] · [[A14 Referencia de API]]
