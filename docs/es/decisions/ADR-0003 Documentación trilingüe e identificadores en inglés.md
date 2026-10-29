---
title: ADR-0003 Documentación trilingüe e identificadores en inglés
lang: es
tags: [decision, docs, i18n]
status: accepted
date: 2026-10-28
---

🌐 [[ADR-0003 Trilingual Documentation and English Identifiers|EN]] · **ES** · [[ADR-0003 Dreisprachige Doku und englische Bezeichner|DE]] · [[Inicio (ES)|Índice]]

# ADR-0003 Documentación trilingüe e identificadores en inglés

- **Estado:** Aceptada (estructura elegida por el responsable del proyecto)
- **Fecha:** 28-10-2026

## Contexto

Todo debe estar disponible en inglés, español y alemán, y poder leerse en Obsidian. El código ilustrativo del plan usa claves en español (`clave`, `parametros`, `longitud`…), y el §18 del plan deja abierto el idioma de la interfaz y recomienda preparar las traducciones desde el inicio.

## Decisión

1. **Documentación:** `docs/` es una bóveda de Obsidian con tres carpetas paralelas `en/`, `es/`, `de/`, con la misma estructura y numeración (T, A, S, ADR). Nombres de nota traducidos y únicos en la bóveda; una barra de idioma al principio de cada nota. Las especificaciones viven en `docs/<idioma>/specs/` y las decisiones en `docs/<idioma>/decisions/`.
2. **Identificadores de código en inglés:** paquetes, funciones, claves de parámetros (`rollerPitch`, `nominalSpeed`), tipos de evento y nombres de archivo. Cada especificación relaciona cada clave inglesa con la clave española del plan (columna «Clave del plan») para mantener la trazabilidad.
3. **Textos para el usuario en tres idiomas:** nombres, descripciones, etiquetas y descripciones de parámetros son objetos `{ en, es, de }` validados con `localizedTextSchema`; los textos de la interfaz del editor están en `apps/editor/src/i18n/strings.ts`, con una prueba que comprueba que los tres idiomas tienen las mismas claves.

## Alternativas consideradas

- Los tres idiomas dentro de cada nota: menos archivos, pero notas muy largas y enlaces confusos (descartado por el responsable).
- Identificadores en español como en el apéndice del plan: mezcla idiomas en el código y dificulta contribuciones externas.

## Consecuencias

- AGENTS.md actualizado: `docs/specs/` → `docs/<idioma>/specs/`, `docs/decisions/` → `docs/<idioma>/decisions/`.
- Cada cambio de documentación toca tres archivos; las revisiones deben comprobar que las traducciones siguen sincronizadas.
- Añadir un cuarto idioma = una carpeta nueva + una clave nueva en `localizedTextSchema` y `LANGUAGES`.
