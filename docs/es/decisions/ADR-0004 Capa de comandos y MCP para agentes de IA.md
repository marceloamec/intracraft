---
title: ADR-0004 Capa de comandos y MCP para agentes de IA
lang: es
tags: [decision, ai, mcp, architecture]
status: accepted
date: 2026-10-29
---

🌐 [[ADR-0004 AI-Ready Command Layer and MCP|EN]] · **ES** · [[ADR-0004 Befehlsschicht und MCP für KI-Agenten|DE]] · [[Inicio (ES)|Índice]]

# ADR-0004 Capa de comandos y MCP para agentes de IA

- **Estado:** Aceptada como arquitectura; implementación aplazada (por ahora solo documentación)
- **Fecha:** 29-10-2026

## Contexto

El responsable del proyecto exige que los agentes de IA puedan trabajar con IntraCraft desde el principio —directamente o mediante un servidor MCP— y que más adelante la aplicación ofrezca un chat integrado donde el usuario introduzca su propio gateway de LLM y su token de API. Añadir esto a posteriori a una aplicación centrada en la interfaz es caro: la lógica escondida en los manejadores de la interfaz no la puede llamar un agente.

## Decisión

1. **Capa de comandos desde la fase 2.** Todo cambio de estado (layout, parámetros, control de la simulación, archivos) es un comando tipado con esquema Zod, validación, deshacer y un registro de comandos con el actor. La interfaz del editor solo usa comandos. Las lecturas van por una capa de consultas aparte.
2. **Los agentes usan los mismos comandos.** Las definiciones de herramientas (MCP y chat) se generan desde los esquemas de los comandos; no hay caminos de código solo para agentes.
3. **Servidor MCP** como `apps/mcp-server` tras la fase 3, primero por stdio (revisión de la especificación 2026-07-28, `@modelcontextprotocol/sdk`) y después Streamable HTTP en localhost.
4. **Chat integrado** post-MVP, independiente del proveedor mediante una interfaz `LlmAdapter`; primer adaptador sobre el SDK oficial de Anthropic con URL de gateway configurable; tokens guardados por usuario, nunca en proyectos ni logs.
5. Detalles: [[A15 Integración con agentes de IA]].

## Alternativas consideradas

- Que los agentes manejen la interfaz (automatización de pantalla): frágil, lento y no determinista.
- Que los agentes editen directamente el JSON del proyecto: se salta la validación, el deshacer y la simulación en curso.
- Hacer primero el servidor MCP y después los comandos: duplica lógica.

## Consecuencias

- La fase 2 crece con un pequeño módulo `commands` (esquemas, ejecutor, registro, deshacer).
- Cada función nueva debe incluir su(s) comando(s) y pruebas; AGENTS.md actualizado con esta regla.
- El registro de comandos sirve además como «macro» reproducible de cualquier sesión, útil para informes de errores y pruebas.
