---
title: ADR-0003 Trilingual Documentation and English Identifiers
lang: en
tags: [decision, docs, i18n]
status: accepted
date: 2026-10-28
---

🌐 **EN** · [[ADR-0003 Documentación trilingüe e identificadores en inglés|ES]] · [[ADR-0003 Dreisprachige Doku und englische Bezeichner|DE]] · [[Home (EN)|Index]]

# ADR-0003 Trilingual Documentation and English Identifiers

- **Status:** Accepted (structure chosen by the project owner) — **amended 2026-10-29: English only for now**
- **Date:** 2026-10-28

## Context

Everything must be available in English, Spanish and German, readable in Obsidian. The plan's illustrative code uses Spanish keys (`clave`, `parametros`, `longitud`…), and plan §18 leaves the UI language open, suggesting translations from the start.

## Decision

1. **Docs:** `docs/` is an Obsidian vault with three parallel folders `en/`, `es/`, `de/`, identical structure and numbering (T, A, S, ADR). Translated, vault-unique note names; a language bar at the top of every note. Specifications live in `docs/<lang>/specs/`, decisions in `docs/<lang>/decisions/`.
2. **Code identifiers in English:** packages, functions, parameter keys (`rollerPitch`, `nominalSpeed`), event types and file names. Each spec maps every English key to the plan's Spanish key ("Plan key" column) for traceability.
3. **User-facing text in three languages:** component names, descriptions, parameter labels and descriptions are `{ en, es, de }` objects validated by `localizedTextSchema`; editor UI strings live in `apps/editor/src/i18n/strings.ts`, with a test that all three languages have the same keys.

## Alternatives considered

- All three languages inside each note: fewer files but very long notes and noisy links (rejected by the owner).
- Spanish identifiers as in the plan's appendix: mixes languages in code and makes outside contributions harder.

## Consequences

- AGENTS.md updated: `docs/specs/` → `docs/<lang>/specs/`, `docs/decisions/` → `docs/<lang>/decisions/`.
- Every documentation change touches three files; reviews should check that translations stay in sync.
- Adding a fourth language = a new folder + a new key in `localizedTextSchema` and `LANGUAGES`.

## Amendment (2026-10-29): English is enough for now

Decided by the project owner: from now on new notes and changes are written **only in English** (`docs/en`). The Spanish and German notes created up to 2026-10-29 are **kept as they are** — not deleted — but may become outdated; the English version is always the reference. They will be translated again only when the owner asks.

In code, user-facing text requires `en`; `es` and `de` are optional and fall back to English (`localizedTextSchema`, `localize()`, editor `translate()`). Existing translations are kept.
