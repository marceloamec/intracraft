---
title: ADR-0004 Befehlsschicht und MCP für KI-Agenten
lang: de
tags: [decision, ai, mcp, architecture]
status: accepted
date: 2026-10-29
---

🌐 [[ADR-0004 AI-Ready Command Layer and MCP|EN]] · [[ADR-0004 Capa de comandos y MCP para agentes de IA|ES]] · **DE** · [[Startseite (DE)|Index]]

# ADR-0004 Befehlsschicht und MCP für KI-Agenten

- **Status:** Als Architektur angenommen; Umsetzung verschoben (vorerst nur Dokumentation)
- **Datum:** 29.10.2026

## Kontext

Der Projektverantwortliche verlangt, dass KI-Agenten von Anfang an mit IntraCraft arbeiten können – direkt oder über einen MCP-Server – und dass die Anwendung später einen integrierten Chat bietet, in dem der Benutzer eigenes LLM-Gateway und eigenen API-Token einträgt. Das nachträglich in eine oberflächenzentrierte Anwendung einzubauen ist teuer: In UI-Handlern versteckte Logik kann kein Agent aufrufen.

## Entscheidung

1. **Befehlsschicht ab Phase 2.** Jede Zustandsänderung (Layout, Parameter, Simulationssteuerung, Dateien) ist ein typisierter Befehl mit Zod-Schema, Validierung, Rückgängig und einem Befehlsprotokoll mit Akteur. Die Editor-Oberfläche nutzt nur Befehle. Lesezugriffe laufen über eine eigene Abfrageschicht.
2. **Agenten nutzen dieselben Befehle.** Werkzeugdefinitionen (MCP und Chat) werden aus den Befehlsschemas erzeugt; keine Codepfade nur für Agenten.
3. **MCP-Server** als `apps/mcp-server` nach Phase 3, zuerst über stdio (Spezifikationsrevision 2026-07-28, `@modelcontextprotocol/sdk`), später Streamable HTTP auf localhost.
4. **Integrierter Chat** nach dem MVP, anbieterunabhängig über eine Schnittstelle `LlmAdapter`; erster Adapter auf dem offiziellen Anthropic-SDK mit einstellbarer Gateway-URL; Tokens je Benutzer gespeichert, nie in Projekten oder Logs.
5. Details: [[A15 KI-Agenten-Integration]].

## Betrachtete Alternativen

- Agenten bedienen die Oberfläche (Bildschirmautomatisierung): fragil, langsam, nicht deterministisch.
- Agenten bearbeiten das Projekt-JSON direkt: umgeht Validierung, Rückgängig und die laufende Simulation.
- Erst den MCP-Server, später die Befehle: doppelte Logik.

## Folgen

- Phase 2 wächst um ein kleines Modul `commands` (Schemas, Ausführer, Protokoll, Rückgängig).
- Jede neue Funktion muss ihre Befehle und Tests mitbringen; AGENTS.md um diese Regel ergänzt.
- Das Befehlsprotokoll dient zugleich als reproduzierbares „Makro“ jeder Sitzung – nützlich für Fehlerberichte und Tests.
