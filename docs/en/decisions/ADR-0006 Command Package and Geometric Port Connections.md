---
title: ADR-0006 Command Package and Geometric Port Connections
lang: en
tags: [decision, architecture, editor, ports]
status: accepted
date: 2026-10-29
---

# ADR-0006 Command Package and Geometric Port Connections

- **Status:** Accepted (approved by the project owner with the editor plan, 2026-10-29)
- **Date:** 2026-10-29

## Context

The editor gained a component panel (drag into the 3D or 2D view), port snapping and a properties panel. [[ADR-0004 AI-Ready Command Layer and MCP]] requires that every change goes through typed commands, and that agents and the UI use the same ones. The commands need the library definitions, both to validate parameters and to find ports, so they cannot live in `core-model`.

## Decision

1. **New package `packages/commands`** on top of `core-model` and `library`:
   - `Session` holds the current layout, undo/redo and the command log (with actor `user`, `cli`, `chat` or `mcp:<client>`).
   - The commands are `layout.addInstance`, `layout.move`, `layout.setParams`, `layout.setAttributes`, `layout.remove` and `layout.clear`. Each has a Zod input schema and one file.
   - A command is atomic. A rejected command changes nothing and is still logged.
   - Undo keeps the previous layout snapshots (layouts are immutable).
   - The editor changes the layout only through `Session.execute`.
2. **Connections are derived from the geometry.** After every edit, two ports are connected when all of the following hold:
   - they face each other;
   - they are `PORT_GAP` = 20 mm apart, ±10 mm;
   - their directions are compatible (out/inout → in/inout).

   There is no separate "connect" step, so placing objects is enough, for the user and for an agent.
3. **Ports have a facing and simple formulas.**
   - `facing` is `+x | -x | +z | -z` in the local frame.
   - A coordinate is a number, a parameter key, or a list of terms `{param, factor}` / numbers that are added up. For example, `−platformLength/2` or `surfaceHeight + liftStroke`. No text is evaluated.
4. **Snapping rule (editor).**
   - A dragged component snaps when one of its ports faces a free port within 300 mm. The pose is chosen so that the two ports end up 20 mm apart.
   - Otherwise the component goes on the 100 mm grid.
   - The yaw is never changed automatically; the user rotates with R.
   - Warnings: support heights differ by more than 10 mm, or both ports have the same direction (out→out, in→in).
5. **Turntable ports.** The four station ports (P0, P90, P180, P270) sit at ±`platformLength`/2, so a port exists for every station orientation of the platform.
6. **Instance attributes.**
   - `instance.attributes` is a free key → number | boolean | text map, for example zone = A.
   - It is optional with default `{}`, so `schemaVersion` stays 1 and needs no migration.

## Alternatives considered

- Commands inside `core-model`: impossible without a dependency loop with `library`.
- An explicit `layout.connect` command: it can contradict the geometry (connected but 2 m apart). It may return later for non-adjacent logical links.
- Inverse operations per command for undo, instead of snapshots: more code for no benefit at MVP layout sizes.
- Automatic rotation while snapping: less predictable. The user keeps control with R.

## Consequences

- AGENTS.md structure lists `packages/commands`.
- [[A04 Data Model and Project File]], [[A05 Component Library]], [[A09 Editor and Viewer]] and [[A14 API Reference]] are updated.
- Future MCP and chat tools ([[A15 AI Agent Integration]]) are generated from `COMMANDS` in `packages/commands/src/session.ts`.
- Tolerances (300 mm snap radius, 10 mm height) are indicative values, adjustable later.
