@AGENTS.md

## Instrucciones específicas para Claude Code

- Responde siempre en español, con un tono claro y sin jerga de programación innecesaria.
- Antes de cambios que afecten a más de un archivo, usa el modo plan y espera confirmación.
- Al terminar cada tarea, ejecuta `pnpm test` y `pnpm lint` y resume el resultado en lenguaje de simulación.
- Las especificaciones de `docs/en/specs/` (referencia; las copias en español y alemán pueden estar desactualizadas) y el documento «IntraCraft: plan y especificación del MVP» mandan sobre cualquier suposición tuya; si hay contradicción, pregunta.
- Si necesitas información actual (versiones de librerías, capacidades de los motores de física), búscala en lugar de suponerla, y deja constancia en `docs/en/decisions/` (por ahora solo en inglés).
