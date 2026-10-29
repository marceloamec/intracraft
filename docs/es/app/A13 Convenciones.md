---
title: A13 Convenciones
lang: es
tags: [app, conventions]
---

🌐 [[A13 Conventions|EN]] · **ES** · [[A13 Konventionen|DE]] · [[Inicio (ES)|Índice]]

# A13 Convenciones

## 1. Convenciones del modelo

| Tema                | Regla                                                                                                                                                 |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Unidades            | Internamente **SI**: m, kg, s, rad. La interfaz muestra mm y grados. Convertir solo en los bordes (interfaz, archivos) con `mmToM`, `degToRad`…       |
| Ejes                | **Y hacia arriba**, mano derecha (igual que three.js y glTF). Los transportadores avanzan según +X local; la vista 2D es la cenital mirando hacia −Y. |
| Tiempo              | **Microsegundos** enteros (`SimTime`). Orden de eventos (tiempo, prioridad, secuencia).                                                               |
| Aleatoriedad        | Siempre el generador con semilla del proyecto (`createRng`). Nunca `Math.random()` en los motores.                                                    |
| Determinismo        | Mismo layout + misma semilla → mismo resultado. Una prueba de determinismo que falla es prioridad máxima.                                             |
| Archivo de proyecto | JSON `.intracraft.json` con `schemaVersion` y migraciones explícitas; los resultados se guardan aparte.                                               |
| Valores por defecto | Los de la librería son **orientativos**; nunca se presentan como datos de fabricante.                                                                 |

## 2. Convenciones de código

- TypeScript estricto en todo (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`); indentación de 2 espacios; comillas simples.
- Identificadores, claves de parámetros y nombres de archivo en **inglés**; los textos para el usuario, traducidos `{ en, es, de }` ([[ADR-0003 Documentación trilingüe e identificadores en inglés]]).
- Nada de `console.log` fuera de pruebas: usar `createLogger` de `core-model`.
- Un archivo por componente de la librería o por comando; estado, tipos y ayudantes compartidos en módulos de apoyo.
- Los motores (`sim-des`, `sim-physics`, `control`) no dependen de React ni del DOM.
- Orden en cada archivo: imports → constantes → tipos (externos antes que internos) → funciones públicas → ayudantes privados debajo de quien los usa. Pruebas: constantes → casos (agrupados en bloques `describe`) → ayudantes al final.

## 3. Convenciones de documentación (esta bóveda)

- Tres carpetas paralelas `en/`, `es/`, `de/` con la misma estructura y numeración (T = teoría, A = aplicación, S = especificación, ADR = decisión).
- Los nombres de archivo de las notas están traducidos y son únicos en toda la bóveda, para que los `[[enlaces]]` se resuelvan sin rutas.
- Cada nota empieza con una barra de idioma que enlaza las otras dos traducciones.
- Las fórmulas usan LaTeX (`$…$`, `$$…$$`) y los diagramas Mermaid; ambos se ven directamente en Obsidian.
- Las tablas Markdown se rellenan con espacios para que las columnas queden alineadas en texto plano (regla de AGENTS.md).
- Cuando cambia una nota, se actualizan sus dos traducciones en el mismo cambio.

## 4. Reglas clean-room

- Nunca leer, descompilar, importar ni copiar código, binarios, archivos de proyecto ni recursos de software de terceros (incluidos simuladores comerciales).
- Sin nombres, logotipos ni marcas de terceros en el producto, el código o la interfaz.
- Sin modelos 3D ni catálogos de fabricantes sin licencia.
- Revisar la licencia de cada dependencia nueva (inventario actual en ADR-0001).
- Otros programas solo se citan para comparar resultados observables.

## 5. Flujo de trabajo

1. Leer la especificación del objeto (si no existe, escribirla primero y pedir confirmación).
2. Proponer un plan corto.
3. Escribir la prueba, verla fallar e implementar hasta que pase.
4. Ejecutar todas las pruebas, el linter y la comprobación de tipos.
5. Resumir en términos de simulación.
6. Un commit pequeño por cada paso que funciona; una rama por función.

## Relacionado

[[A03 Monorepo y stack]] · [[T03 Tiempo, orden de eventos y determinismo]]
