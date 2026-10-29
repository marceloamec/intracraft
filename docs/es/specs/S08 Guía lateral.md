---
title: S08 Guía lateral
lang: es
tags: [spec, mechanics, guide]
library-id: mechanics.guide
status: spec
---

🌐 [[S08 Side Guide|EN]] · **ES** · [[S08 Seitenführung|DE]] · [[Inicio (ES)|Índice]]

# S08 Guía lateral (`mechanics.guide`)

Viene integrada en cada transportador (grupo `guides`) y también como componente independiente para tramos extra. Teoría: [[T14 Topes, impactos y alineación]] §6.

## 1. Parámetros

| Clave                | Clave del plan    | Unidad | Defecto | Rango                              | Descripción                              |
| -------------------- | ----------------- | ------ | ------- | ---------------------------------- | ---------------------------------------- |
| `side`               | `lado`            | enum   | ambos   | izquierda, derecha, ambos, ninguno | Lados con guía                           |
| `guideHeight`        | `alturaGuia`      | mm     | 150     | 50 … 400                           | Altura sobre la superficie de transporte |
| `clearance`          | `holgura`         | mm     | 25      | 0 … 100                            | Holgura por lado entre pallet y guía     |
| `startPct`, `endPct` | `sInicio`, `sFin` | %      | 0, 100  | 0 … 100                            | Tramo cubierto de la longitud            |
| `funnelEntry`        | `entradaEmbudo`   | bool   | no      | —                                  | Añade una entrada divergente             |
| `funnelAngle`        | `anguloEmbudo`    | °      | 10      | 3 … 30                             | Ángulo de la entrada                     |
| `funnelLength`       | `longitudEmbudo`  | mm     | 300     | 100 … 800                          | Longitud de la entrada                   |
| `guideMu`            | `muGuia`          | —      | 0,30    | 0,1 … 0,6                          | Rozamiento con el pallet                 |
| `guideRestitution`   | `restitucionGuia` | —      | 0,1     | 0 … 0,5                            | Rebote lateral                           |

## 2. Cómo se alinean los pallets

- El **tope** fija la posición longitudinal del borde frontal; con un único punto de contacto el pallet puede girar, con un tope ancho o dos topes se endereza.
- Las **guías** fijan la posición lateral; el embudo corrige entradas desalineadas (captura ≈ holgura + longitudEmbudo · tan(ánguloEmbudo) = 25 + 53 mm con los valores por defecto).
- En 2D solo se aplica la posición final ideal (las guías son una validación de holgura); en 3D se obtiene la pose real, que es la medida de la calidad de la alineación.

## 3. Comportamiento

**2D.** Solo validación de holgura. **3D.** Colisionadores estáticos con rozamiento y restitución.

## 4. Pruebas

| Prueba     | Criterio                                             | Estado                     |
| ---------- | ---------------------------------------------------- | -------------------------- |
| S1-d / P3  | Desalineación de 50 mm y 5° corregida a ±10 mm y ±1° | ⏳ S1 / fase 4             |
| Validación | Ancho útil ≥ pallet + 2 × holgura                    | ✅ `usableWidthFitsPallet` |

## Relacionado

[[T14 Topes, impactos y alineación]] · [[S07 Tope mecánico]]
