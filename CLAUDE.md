@AGENTS.md

## Instrucciones específicas para Claude Code

- Responde siempre en español, con un tono claro y sin jerga de programación innecesaria.
- Antes de cambios que afecten a más de un archivo, usa el modo plan y espera confirmación.
- Al terminar cada tarea, ejecuta `pnpm test` y `pnpm lint` y resume el resultado en lenguaje de simulación.
- Las especificaciones de `docs/en/specs/` (referencia; las copias en español y alemán pueden estar desactualizadas) y el documento «IntraCraft: plan y especificación del MVP» mandan sobre cualquier suposición tuya; si hay contradicción, pregunta.
- Si necesitas información actual (versiones de librerías, capacidades de los motores de física), búscala en lugar de suponerla, y deja constancia en `docs/en/decisions/` (por ahora solo en inglés).

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

Rules:
- When graphify-out/graph.json exists, run `graphify query "<question>"` before your first grep, search or source-file read for a codebase question, and put this rule in every subagent prompt that explores code. Read raw files only for lines the graph or an earlier read already located. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
- A committed graphify-out/graph.json is rewritten whole on every build, so a large diff to it in a pull request is expected. Do not flag it as churn or suggest skipping the graph rebuild or its hooks.
