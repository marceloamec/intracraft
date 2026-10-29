---
title: ADR-0004 AI-Ready Command Layer and MCP
lang: en
tags: [decision, ai, mcp, architecture]
status: accepted
date: 2026-10-29
---

🌐 **EN** · [[ADR-0004 Capa de comandos y MCP para agentes de IA|ES]] · [[ADR-0004 Befehlsschicht und MCP für KI-Agenten|DE]] · [[Home (EN)|Index]]

# ADR-0004 AI-Ready Command Layer and MCP

- **Status:** Accepted as architecture; implementation deferred (documentation only for now)
- **Date:** 2026-10-29

## Context

The project owner requires that AI agents can work with IntraCraft from the start — directly or via an MCP server — and that the app later offers an embedded chat where the user enters their own LLM gateway and API token. Retro-fitting this onto a UI-centric app is expensive: logic hidden in UI handlers cannot be called by an agent.

## Decision

1. **Command layer from phase 2.** Every state change (layout, parameters, simulation control, files) is a typed command with a Zod schema, validation, undo, and a command log with the actor. The editor UI uses only commands. Reads go through a separate query layer.
2. **Agents use the same commands.** Tool definitions (MCP and chat) are generated from the command schemas; no agent-only code paths.
3. **MCP server** as `apps/mcp-server` after phase 3, stdio first (spec revision 2026-07-28, `@modelcontextprotocol/sdk`), Streamable HTTP on localhost later.
4. **Embedded chat** post-MVP, provider-agnostic through an `LlmAdapter` interface; first adapter on the official Anthropic SDK with a configurable gateway URL; tokens stored per user, never in projects or logs.
5. Details: [[A15 AI Agent Integration]].

## Alternatives considered

- Let agents drive the UI (screen automation): fragile, slow, not deterministic.
- Agents editing the project JSON directly: bypasses validation, undo and the running simulation.
- Build the MCP server first, commands later: duplicates logic.

## Consequences

- Phase 2 grows by a small `commands` module (schemas, executor, log, undo).
- Every new feature must ship its command(s) and tests; AGENTS.md updated with this rule.
- The command log doubles as a reproducible "macro" of any session, useful for bug reports and tests.
