---
title: A01 Visión del producto
lang: es
tags: [app, overview]
---

🌐 [[A01 Product Overview|EN]] · **ES** · [[A01 Produktüberblick|DE]] · [[Inicio (ES)|Índice]]

# A01 Visión del producto

## Qué es IntraCraft

Una aplicación para **diseñar y simular sistemas de transporte de pallets** con un solo layout y dos modos de simulación:

| Modo              | Motor         | Velocidad            | Responde a                                                    |
| ----------------- | ------------- | -------------------- | ------------------------------------------------------------- |
| 2D discreto (DES) | `sim-des`     | ≥ 1000× tiempo real  | Rendimiento, acumulación, bloqueos, utilización               |
| 3D con física     | `sim-physics` | Tiempo real (60 FPS) | Rozamiento, impactos, alineación, sensores con geometría real |

Ambos modos leen el mismo layout y los manda el mismo controlador Si/Entonces. Teoría: [[T01 Paradigmas de simulación]].

## Principios de diseño

1. El layout es la fuente de verdad; 2D, 3D y estadísticas se derivan de él.
2. Misma lógica de control, distinta planta.
3. Todo parámetro tiene unidad, rango, valor por defecto y descripción; la interfaz de edición se genera desde ese esquema.
4. Determinismo: mismos datos + misma semilla → mismos resultados.
5. La geometría real del pallet importa (huecos entre tablas y tacos).
6. Validar antes de ampliar: pruebas analíticas y comparación con modelos de referencia conocidos.
7. Pasos pequeños, siempre con prueba.

## Alcance del MVP

**Dentro:** cinco transportadores ([[S01 Mesa de rodillos]], [[S02 Mesa de cadenas]], [[S03 Giratoria de cadenas]], [[S04 Giratoria de rodillos]], [[S05 Transferencia rodillos-cadenas]]); fotocélulas (difusa, reflex, barrera); topes fijos y retráctiles; guías laterales con embudo; generador y sumidero; pallets EUR, 1200 × 1000, GMA y personalizado; controlador Si/Entonces con señales booleanas; ambos modos sobre el mismo layout (el cambio reinicia); editor (colocar, conectar, parametrizar, guardar, abrir; reproducir, pausar, acelerar); estadísticas básicas, exportación CSV/JSON y CLI sin interfaz; librería integrada con presets y componentes propios.

**Fuera (post-MVP):** curvas, elevadores, lanzaderas, desviadores; cajas y cubetas; fallos (MTBF/MTTR) y energía; scripting de usuario; cambio de modo en marcha; calibración automática 2D↔3D; experimentos y optimización; OPC UA / PLC; importación de CAD; trabajo en equipo.

## Definición de «hecho» del MVP

1. Los cinco transportadores se pueden colocar, conectar y parametrizar desde la librería.
2. Un layout de referencia con los cinco corre en ambos modos sin intervención.
3. Los resultados 2D coinciden con las pruebas analíticas con error < 1 % ([[A11 Pruebas y validación]]).
4. En 3D un pallet se detiene contra un tope a ±5 mm, con rebote < 10 mm.
5. Tiempos de tránsito y eventos de sensores difieren entre modos < 5 % y ±50 ms.
6. Mismo layout + misma semilla → resultados idénticos en dos ejecuciones.
7. El 3D mantiene 60 FPS con 20 transportadores y 10 pallets en un portátil actual.
8. Se reproducen y comparan tres modelos de referencia conocidos.

## Estado actual (28-10-2026)

El andamiaje de la fase 0 está listo: monorepo, paquetes, flujo de CI, cinemática analítica (T1–T5 pasan), cola de eventos, generador con semilla, esquema del proyecto, definición de la mesa de rodillos, evaluador de reglas y un visor con una mesa de rodillos paramétrica. Ver [[A12 Hoja de ruta]].

## Relacionado

[[A02 Arquitectura]] · [[A12 Hoja de ruta]] · [[Glosario]]
