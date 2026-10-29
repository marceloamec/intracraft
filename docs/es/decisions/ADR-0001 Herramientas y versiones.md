---
title: ADR-0001 Herramientas y versiones
lang: es
tags: [decision, tooling]
status: accepted
date: 2026-10-28
---

🌐 [[ADR-0001 Toolchain and Versions|EN]] · **ES** · [[ADR-0001 Werkzeuge und Versionen|DE]] · [[Inicio (ES)|Índice]]

# ADR-0001 Herramientas y versiones

- **Estado:** Aceptada
- **Fecha:** 28-10-2026

## Contexto

El plan fija el stack (TypeScript estricto, espacios de trabajo pnpm, Vite, Vitest, ESLint, Prettier, Zod, React, react-three-fiber, Zustand). AGENTS.md pide buscar las versiones actuales en lugar de suponerlas y dejar constancia aquí. Las versiones se consultaron en el registro de npm el 28-10-2026.

## Decisión

| Elemento             | Elegido                  | Última vista | Por qué no la última / notas                                                                            |
| -------------------- | ------------------------ | ------------ | ------------------------------------------------------------------------------------------------------- |
| Node.js              | ≥ 24 (24.22.0 instalado) | —            | Línea LTS disponible en el equipo                                                                       |
| pnpm                 | 10.34.9                  | 12.16.1      | Versión mayor madura; `onlyBuiltDependencies` para esbuild. Actualizar más adelante de forma deliberada |
| TypeScript           | 6.0.3                    | 7.0.2        | typescript-eslint 8.72.1 exige TypeScript < 6.1                                                         |
| typescript-eslint    | 8.72.1                   | 8.72.1       | —                                                                                                       |
| ESLint / @eslint/js  | 10.13.0 / 10.0.1         | igual        | Configuración plana                                                                                     |
| Vitest               | 5.0.5                    | igual        | —                                                                                                       |
| Vite / plugin-react  | 8.3.5 / 6.1.2            | igual        | —                                                                                                       |
| React / React DOM    | 19.3.1                   | igual        | @react-three/fiber 9.8.2 exige React < 19.4                                                             |
| three / @types/three | 0.186.1 / 0.186.0        | igual        | —                                                                                                       |
| @react-three/fiber   | 9.8.2                    | igual        | —                                                                                                       |
| @react-three/drei    | **no añadido**           | 10.7.9       | Evitar una dependencia pesada; OrbitControls sale de `three/examples`                                   |
| Zustand              | 5.0.16                   | igual        | —                                                                                                       |
| Zod                  | 4.6.7                    | igual        | —                                                                                                       |
| tsx                  | 4.23.15                  | igual        | Ejecuta el TypeScript de la CLI sin compilar                                                            |
| Motor de física      | **sin instalar**         | —            | Se decide en el spike S1 ([[ADR-0002 Candidato a motor de física]])                                     |

Todas las versiones están fijadas exactamente en los `package.json`. Los paquetes son **de código fuente directo** (`exports: ./src/index.ts`), sin paso de compilación entre paquetes.

## Alternativas consideradas

- TypeScript 7 (compilador nativo): más rápido, pero hoy rompería las herramientas de ESLint con información de tipos.
- Compilar cada paquete a `dist/`: más ceremonia y respuesta más lenta; no hace falta mientras todo pase por Vite/Vitest/tsx.

## Consecuencias

- Un único `tsc --noEmit` en la raíz comprueba todo el monorepo (`pnpm lint`).
- Pasar a TypeScript 7.x espera a que typescript-eslint lo admita.
- Licencias (`pnpm licenses list`, 28-10-2026, 194 paquetes): MIT 156, Apache-2.0 16, ISC 9, BSD-2/3-Clause 9, más tres paquetes transitivos de herramientas de compilación: `lightningcss` (MPL-2.0, copyleft débil por archivo, se usa sin modificar), `caniuse-lite` (CC-BY-4.0, datos de navegadores) y `minimatch` (BlueOak-1.0.0, permisiva). Ninguno se distribuye modificado; revisar con cada dependencia nueva.
