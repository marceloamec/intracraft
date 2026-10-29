---
title: A12 Hoja de ruta
lang: es
tags: [app, roadmap, plan]
---

🌐 [[A12 Roadmap|EN]] · **ES** · [[A12 Phasenplan|DE]] · [[Inicio (ES)|Índice]]

# A12 Hoja de ruta

Orden pensado para atacar primero el mayor riesgo (la física de las cadenas) y entregar cada transportador de punta a punta. Tamaños en sesiones de trabajo con Claude Code: S ≈ 1–3, M ≈ 4–8, L ≈ 8–15.

## Fases del MVP

| Fase | Objetivo                       | Entregables                                                                                                                                                          | Criterio de salida                                                | Tamaño | Riesgo | Estado (28-10-2026)                                                                 |
| ---- | ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- | ------ | ------ | ----------------------------------------------------------------------------------- |
| 0    | Terreno listo                  | Repositorio, espacios pnpm, AGENTS.md, documentación, glosario, 3 modelos de referencia                                                                              | Las pruebas corren y la CI pasa                                   | S      | Bajo   | 🟡 Andamiaje hecho; faltan modelos de referencia y repositorio git                  |
| 1    | Reducir incertidumbre          | Spike S1 de física, S2 visor 3D con rodillos instanciados y gizmos, S3 DES mínimo (generador → pista → sumidero)                                                     | Criterios de S1 cumplidos y decisión de motor registrada          | M      | Alto   | 🟡 Investigación hecha ([[T15 Investigación de motores de física]]); visor iniciado |
| 2    | Núcleo y librería              | core-model (unidades, puertos, conexiones, validación), esquema de definiciones, registro de comportamientos, tipos de pallet, tabla de fricción, formato de archivo | Generador → sumidero se guarda y se abre; las validaciones avisan | M      | Medio  | 🟡 Esquemas, registro, datos de pallets y fricción                                  |
| 3    | Mesa de rodillos (2D)          | Definición, geometría, comportamiento DES (cinemática, ZPA), sensores, topes y guías en 2D, controlador básico, CLI                                                  | T1, T2; animación 3D de la simulación 2D                          | L      | Medio  | 🟡 JSON de definición, cinemática, CLI                                              |
| 4    | Física base                    | Mundo, rodillos cinemáticos, pallet compuesto, sensores por raycast, topes, guías                                                                                    | S1 a, c, d, g dentro del producto; consistencia 2D/3D             | L      | Alto   | ⚪                                                                                  |
| 5    | Mesa de cadenas                | Definición, DES con deslizamiento, física con el plan elegido                                                                                                        | Velocidad del pallet = cadena ±3 %; acumulación con deslizamiento | M      | Alto   | ⚪                                                                                  |
| 6    | Giratoria de cadenas           | Perfil angular, estaciones, enclavamientos, puertos por estación                                                                                                     | T3 (6,5 s) y S1-e                                                 | M      | Medio  | ⚪ (fórmula T3 ✅)                                                                  |
| 7    | Giratoria de rodillos          | Rodillos sobre plataforma, bloqueados en el giro                                                                                                                     | Mismas pruebas que la fase 6                                      | S–M    | Bajo   | ⚪                                                                                  |
| 8    | Transferencia rodillos→cadenas | Cadenas elevables, enclavamientos, validación de huecos, ambos modos                                                                                                 | T4 (7,4 s) y transferencia 3D sin perder el pallet                | L      | Alto   | ⚪ (fórmula T4 ✅)                                                                  |
| 9    | Integración                    | Layout de referencia con los cinco, cambio de modo, contraste con modelos de referencia, rendimiento, documentación, empaquetado                                     | Definición de «hecho» del MVP ([[A01 Visión del producto]])       | L      | Medio  | ⚪                                                                                  |

🟢 hecho · 🟡 en curso · ⚪ sin empezar

**Por qué este orden:** la mesa de rodillos es la base de las demás; las cadenas concentran el riesgo físico; las giratorias reutilizan rodillos y cadenas; la transferencia depende de todo lo anterior (rodillos, cadenas, topes y sensores).

## Preparada para IA desde el principio

La fase 2 entrega también la capa de comandos y consultas (toda acción de la interfaz es un comando); tras la fase 3, el servidor MCP (stdio); tras el MVP, el chat integrado con gateway y token propios; ver [[A15 Integración con agentes de IA]].

## Próximos pasos concretos

1. `git init`, primer commit y subida a un remoto para que corra la CI.
2. Spike S1 con Jolt (`jolt-physics`): pruebas a, b (superficie móvil del plan B), c y g; si no basta, Rapier + plan A. Cerrar [[ADR-0002 Candidato a motor de física]].
3. Spike S3: generador → pista de rodillos → sumidero en el motor DES, con T6.
4. Responder las preguntas que quedan en [[Preguntas abiertas]] (orientación, tipo de pallet y rodillos respondidos el 29-10-2026).

## Después del MVP (orden sugerido)

1. Cambio de modo en marcha. 2. Calibración automática 2D↔3D. 3. Curvas, desviadores, elevadores, lanzaderas. 4. Cajas y cubetas. 5. Fallos (MTBF/MTTR) y energía. 6. Scripting de usuario. 7. Experimentos, réplicas y optimización. 8. OPC UA / PLC. 9. Modelos 3D de fabricantes con licencia e importación DXF. 10. Versión de escritorio (Tauri).

## Relacionado

[[A01 Visión del producto]] · [[A11 Pruebas y validación]]
