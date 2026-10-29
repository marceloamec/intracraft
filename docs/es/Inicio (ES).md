---
title: Inicio (ES)
lang: es
tags: [index]
---

🌐 [[Home (EN)|EN]] · **ES** · [[Startseite (DE)|DE]] · [[Home|Todos los idiomas]]

> [!warning] Desde el 29-10-2026 la documentación se mantiene solo en inglés ([[Home (EN)]]). Estas notas en español se conservan, pero pueden estar desactualizadas.

# IntraCraft — documentación (español)

IntraCraft simula sistemas de transporte de pallets con **un solo layout y dos modos**: un modo 2D **de eventos discretos**, muy rápido (rendimiento, acumulación, bloqueos, utilización), y un modo 3D **con física** en tiempo real (rozamiento de rodillos y cadenas, colisiones con topes y guías, alineación de pallets, sensores con geometría real).

## Rutas de lectura

- **«Quiero saber cómo calcula el modo 2D»** → [[T01 Paradigmas de simulación]] → [[T02 Simulación de eventos discretos]] → [[T03 Tiempo, orden de eventos y determinismo]] → [[T04 Cinemática de transportadores]] → [[T05 Acumulación, capacidad y rendimiento]].
- **«Quiero saber cómo mueve un pallet el modo 3D»** → [[T09 Dinámica del sólido rígido]] → [[T10 Integración temporal y paso fijo]] → [[T11 Contacto, rozamiento y restitución]] → [[T12 Rodillos y cadenas en física]].
- **«Quiero saber cómo está construida la aplicación»** → [[A01 Visión del producto]] → [[A02 Arquitectura]] → [[A03 Monorepo y stack]] → [[A14 Referencia de API]].

## Teoría

| #   | Nota                                            | Modo  |
| --- | ----------------------------------------------- | ----- |
| T01 | [[T01 Paradigmas de simulación]]                | ambos |
| T02 | [[T02 Simulación de eventos discretos]]         | 2D    |
| T03 | [[T03 Tiempo, orden de eventos y determinismo]] | ambos |
| T04 | [[T04 Cinemática de transportadores]]           | 2D    |
| T05 | [[T05 Acumulación, capacidad y rendimiento]]    | 2D    |
| T06 | [[T06 Detección fotoeléctrica]]                 | ambos |
| T07 | [[T07 Lógica de control y señales]]             | ambos |
| T08 | [[T08 Estadísticas y análisis de resultados]]   | ambos |
| T09 | [[T09 Dinámica del sólido rígido]]              | 3D    |
| T10 | [[T10 Integración temporal y paso fijo]]        | 3D    |
| T11 | [[T11 Contacto, rozamiento y restitución]]      | 3D    |
| T12 | [[T12 Rodillos y cadenas en física]]            | 3D    |
| T13 | [[T13 Giratorias y mesas de transferencia]]     | ambos |
| T14 | [[T14 Topes, impactos y alineación]]            | ambos |
| T15 | [[T15 Investigación de motores de física]]      | 3D    |
| T16 | [[T16 Consistencia híbrida y calibración]]      | ambos |

## Aplicación

| #   | Nota                                          |
| --- | --------------------------------------------- |
| A01 | [[A01 Visión del producto]]                   |
| A02 | [[A02 Arquitectura]]                          |
| A03 | [[A03 Monorepo y stack]]                      |
| A04 | [[A04 Modelo de datos y archivo de proyecto]] |
| A05 | [[A05 Librería de componentes]]               |
| A06 | [[A06 Motor discreto]]                        |
| A07 | [[A07 Envoltorio del motor físico]]           |
| A08 | [[A08 Controlador]]                           |
| A09 | [[A09 Editor y visor]]                        |
| A10 | [[A10 CLI y escenarios]]                      |
| A11 | [[A11 Pruebas y validación]]                  |
| A12 | [[A12 Hoja de ruta]]                          |
| A13 | [[A13 Convenciones]]                          |
| A14 | [[A14 Referencia de API]]                     |
| A15 | [[A15 Integración con agentes de IA]]         |

## Especificaciones (objetos de la librería)

| #   | Nota                                   | Id en la librería           |
| --- | -------------------------------------- | --------------------------- |
| S01 | [[S01 Mesa de rodillos]]               | `conveyor.roller`           |
| S02 | [[S02 Mesa de cadenas]]                | `conveyor.chain`            |
| S03 | [[S03 Giratoria de cadenas]]           | `conveyor.chain_turntable`  |
| S04 | [[S04 Giratoria de rodillos]]          | `conveyor.roller_turntable` |
| S05 | [[S05 Transferencia rodillos-cadenas]] | `conveyor.transfer_rc`      |
| S06 | [[S06 Fotocélula]]                     | `sensor.photoeye`           |
| S07 | [[S07 Tope mecánico]]                  | `mechanics.stop`            |
| S08 | [[S08 Guía lateral]]                   | `mechanics.guide`           |
| S09 | [[S09 Generador y sumidero]]           | `flow.source`, `flow.sink`  |
| S10 | [[S10 Tipos de pallet]]                | `pallet.*`                  |

## Decisiones

- [[ADR-0001 Herramientas y versiones]]
- [[ADR-0002 Candidato a motor de física]]
- [[ADR-0003 Documentación trilingüe e identificadores en inglés]]
- [[ADR-0004 Capa de comandos y MCP para agentes de IA]]

## Referencia

- [[Glosario]] — términos en inglés, español y alemán.
- [[Preguntas abiertas]] — decisiones pendientes del responsable (con contexto y opciones).
