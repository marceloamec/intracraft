---
title: A05 Librería de componentes
lang: es
tags: [app, library]
---

🌐 [[A05 Component Library|EN]] · **ES** · [[A05 Komponentenbibliothek|DE]] · [[Inicio (ES)|Índice]]

# A05 Librería de componentes

## 1. Conceptos

| Concepto   | Significado                                                                                                  |
| ---------- | ------------------------------------------------------------------------------------------------------------ |
| Definición | Un **tipo** de componente: parámetros, puertos, señales, geometría, comportamientos. Vive en la librería.    |
| Instancia  | Un uso concreto de una definición en un layout, con sus propios valores.                                     |
| Preset     | Conjunto de valores con nombre para una definición (p. ej. «Pesado 1000 kg, 0,25 m/s»).                      |
| Origen     | *integrada* (solo lectura), *de usuario* (editable y compartible), *del proyecto* (se guarda con el layout). |

Las definiciones llevan versión semántica; un layout fija la versión que usa.

## 2. Qué contiene una definición

Implementado en `packages/library/src/definition-schema.ts`:

| Campo                 | Significado                                                                |
| --------------------- | -------------------------------------------------------------------------- |
| `id`                  | `categoria.nombre`, p. ej. `conveyor.roller`                               |
| `version`             | Versión semántica `x.y.z`                                                  |
| `category`            | conveyors, sensors, mechanics, flow, pallets, control                      |
| `name`, `description` | Texto traducido `{ en, es, de }`                                           |
| `parameters`          | Lista de definiciones de parámetros (§3)                                   |
| `ports`               | Id, dirección (in/out/inout), posición local (número o clave de parámetro) |
| `signals`             | Nombres de señales de entrada y salida                                     |
| `statistics`          | Nombres de las estadísticas que publica                                    |
| `geometry`            | Id del generador de geometría                                              |
| `behaviors`           | Ids `{ des, physics }` de los comportamientos registrados                  |
| `validationRules`     | Ids de las reglas de validación                                            |
| `presets`             | Conjuntos de valores con nombre                                            |

## 3. Qué contiene un parámetro

| Campo                           | Significado                                                                            |
| ------------------------------- | -------------------------------------------------------------------------------------- |
| `key`                           | Identificador (camelCase en inglés, p. ej. `rollerPitch`)                              |
| `label`, `description`          | Traducidos `{ en, es, de }`                                                            |
| `type`                          | number, integer, boolean, enum, text                                                   |
| `unit`                          | Unidad SI **guardada** (`m`, `m/s`, `kg`…)                                             |
| `displayUnit`                   | Unidad que muestra el editor (`mm`, `°`…)                                              |
| `default`, `min`, `max`, `step` | Valor y rango, en la unidad guardada                                                   |
| `options`                       | Valores de un enumerado                                                                |
| `choices`                       | Valores permitidos de un parámetro numérico (desplegable), p. ej. diámetros de rodillo |
| `group`                         | Grupo del panel de propiedades (geometría, movimiento, rodillos…)                      |
| `advanced`                      | Oculto por defecto                                                                     |
| `affects`                       | `2d`, `3d`, `both` o `geometry`                                                        |
| `derived`, `formula`            | Valor calculado de solo lectura, p. ej. `floor(length / rollerPitch)`                  |
| `dependsOn`                     | Visibilidad condicional, p. ej. `drive=zoned`                                          |

El panel de propiedades se **genera** desde este esquema: no se escriben formularios a mano.

## 4. Datos y código

El JSON describe **qué es** un componente; los comportamientos TypeScript registrados por id implementan **cómo se comporta**. `createBuiltInRegistry()` carga y valida al arrancar todas las definiciones JSON integradas; una definición inválida falla de forma visible. `defaultParams(definition)` devuelve los valores por defecto no derivados.

## 5. Reglas de validación (implementadas)

| Regla                     | Comprobación                                        | Ejemplo                 |
| ------------------------- | --------------------------------------------------- | ----------------------- |
| `min3RollersUnderPallet`  | paso ≤ lado apoyado / 3                             | 206,3 mm bajo 1200 mm ✓ |
| `usableWidthFitsPallet`   | ancho útil ≥ ancho del pallet + 2 × holgura de guía | 1000 ≥ 800 + 50 ✓       |
| `chainFitsBetweenRollers` | paso − diámetro ≥ ancho de cadena + 10 mm           | 117,3 ≥ 40 + 10 ✓       |

Previstas: radio de barrido de giratorias frente a vecinos, velocidad de impacto frente al límite, tolerancia de altura de puertos, longitud de zona ≥ pallet + 100 mm.

## 6. Contenido de la librería del MVP y estado

| Categoría       | Componentes                                                                                    | Estado (28-10-2026)                                      |
| --------------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| Transportadores | Rodillos, cadenas, giratoria de cadenas, giratoria de rodillos, transferencia rodillos→cadenas | Rodillos: JSON de definición; resto: solo especificación |
| Sensores        | Fotocélula difusa, reflex y barrera                                                            | Solo especificación                                      |
| Mecánica        | Tope fijo, tope retráctil, guía lateral (con embudo)                                           | Solo especificación                                      |
| Flujo           | Generador, sumidero                                                                            | Solo especificación                                      |
| Pallets         | EUR, 1200 × 1000, GMA, personalizado                                                           | EUR, 1200 × 1000 y GMA en `pallet-types.json`            |
| Materiales      | Tabla de pares de fricción                                                                     | `friction-pairs.json`                                    |
| Control         | Controlador por reglas y plantillas                                                            | Evaluador de reglas + plantilla ZPA de prueba            |

## 7. Definición de «hecho» de un objeto de la librería

1. Especificación en `docs/<idioma>/specs/` (tres idiomas).
2. JSON de definición con parámetros, rangos, derivados y validaciones.
3. Geometría paramétrica y símbolo 2D.
4. Comportamiento 2D con pruebas analíticas.
5. Comportamiento 3D con pruebas de física.
6. Plantilla de control.
7. Escenario de referencia en `packages/scenarios`.
8. Documentación de parámetros generada desde la definición.

## Relacionado

[[A04 Modelo de datos y archivo de proyecto]] · [[S01 Mesa de rodillos]] · [[S10 Tipos de pallet]] · [[ADR-0003 Documentación trilingüe e identificadores en inglés]]
