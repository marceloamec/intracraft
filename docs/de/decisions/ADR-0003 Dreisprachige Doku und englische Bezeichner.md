---
title: ADR-0003 Dreisprachige Doku und englische Bezeichner
lang: de
tags: [decision, docs, i18n]
status: accepted
date: 2026-10-28
---

🌐 [[ADR-0003 Trilingual Documentation and English Identifiers|EN]] · [[ADR-0003 Documentación trilingüe e identificadores en inglés|ES]] · **DE** · [[Startseite (DE)|Index]]

# ADR-0003 Dreisprachige Doku und englische Bezeichner

- **Status:** Angenommen (Struktur vom Projektverantwortlichen gewählt)
- **Datum:** 28.10.2026

## Kontext

Alles muss auf Englisch, Spanisch und Deutsch vorliegen und in Obsidian lesbar sein. Der Beispielcode des Plans nutzt spanische Schlüssel (`clave`, `parametros`, `longitud` …), und Plan §18 lässt die UI-Sprache offen und empfiehlt, Übersetzungen von Anfang an vorzusehen.

## Entscheidung

1. **Doku:** `docs/` ist ein Obsidian-Vault mit drei parallelen Ordnern `en/`, `es/`, `de/`, gleicher Struktur und Nummerierung (T, A, S, ADR). Übersetzte, im Vault eindeutige Notiznamen; eine Sprachleiste am Anfang jeder Notiz. Spezifikationen liegen in `docs/<sprache>/specs/`, Entscheidungen in `docs/<sprache>/decisions/`.
2. **Code-Bezeichner auf Englisch:** Pakete, Funktionen, Parameterschlüssel (`rollerPitch`, `nominalSpeed`), Ereignistypen und Dateinamen. Jede Spezifikation ordnet jedem englischen Schlüssel den spanischen Plan-Schlüssel zu (Spalte „Plan-Schlüssel“), um die Nachverfolgbarkeit zu sichern.
3. **Benutzertexte in drei Sprachen:** Komponentennamen, Beschreibungen, Parameterbezeichnungen und -beschreibungen sind `{ en, es, de }`-Objekte, validiert mit `localizedTextSchema`; die UI-Texte des Editors liegen in `apps/editor/src/i18n/strings.ts`, mit einem Test, dass alle drei Sprachen dieselben Schlüssel haben.

## Betrachtete Alternativen

- Alle drei Sprachen in jeder Notiz: weniger Dateien, aber sehr lange Notizen und unübersichtliche Links (vom Verantwortlichen verworfen).
- Spanische Bezeichner wie im Anhang des Plans: mischt Sprachen im Code und erschwert externe Beiträge.

## Folgen

- AGENTS.md aktualisiert: `docs/specs/` → `docs/<sprache>/specs/`, `docs/decisions/` → `docs/<sprache>/decisions/`.
- Jede Doku-Änderung betrifft drei Dateien; Reviews sollten prüfen, dass die Übersetzungen synchron bleiben.
- Eine vierte Sprache = neuer Ordner + neuer Schlüssel in `localizedTextSchema` und `LANGUAGES`.
