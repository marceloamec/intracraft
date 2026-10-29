---
title: S07 Tope mecánico
lang: es
tags: [spec, mechanics, stop]
library-id: mechanics.stop
status: spec
---

🌐 [[S07 Mechanical Stop|EN]] · **ES** · [[S07 Mechanischer Anschlag|DE]] · [[Inicio (ES)|Índice]]

# S07 Tope mecánico (`mechanics.stop`)

Teoría: [[T14 Topes, impactos y alineación]].

## 1. Parámetros

| Clave             | Clave del plan      | Unidad  | Defecto   | Rango           | Descripción                                                      |
| ----------------- | ------------------- | ------- | --------- | --------------- | ---------------------------------------------------------------- |
| `type`            | `tipo`              | enum    | retráctil | fijo, retráctil | El fijo está siempre arriba; el retráctil lleva actuador         |
| `host`, `s`       | `anfitrion`, `s`    | ref, mm | —         | —               | Posición del plano de contacto a lo largo del transportador      |
| `stopWidth`       | `anchoTope`         | mm      | 100       | 20 … 400        | Ancho transversal; uno ancho o dos topes alinean mejor el frente |
| `stopHeight`      | `alturaTope`        | mm      | 100       | 30 … 300        | Altura sobre la superficie de transporte                         |
| `raiseTime`       | `tiempoSubida`      | s       | 0,3       | 0,05 … 2        | Tiempo para elevarse                                             |
| `lowerTime`       | `tiempoBajada`      | s       | 0,3       | 0,05 … 2        | Tiempo para bajar                                                |
| `initialState`    | `estadoInicial`     | enum    | arriba    | arriba, abajo   | Estado al iniciar                                                |
| `restitution`     | `restitucion`       | —       | 0,1       | 0 … 0,5         | Rebote del pallet al impactar                                    |
| `stopMu`          | `muTope`            | —       | 0,3       | 0,1 … 0,6       | Rozamiento con el pallet                                         |
| `maxImpactEnergy` | `energiaMaxImpacto` | J       | 200       | 10 … 2000       | Avisa si la energía cinética de impacto lo supera                |
| `commandSignal`   | `señalMando`        | texto   | auto      | —               | Entrada booleana (arriba o abajo)                                |
| `positionSignals` | `señalPosicion`     | bool    | no        | —               | Publica finales de carrera de arriba y abajo                     |

## 2. Energía de impacto

$E = \tfrac12 m v^2$: 1025 kg (pallet + carga) → **32 J** a 0,25 m/s y 128 J a 0,5 m/s (T5). El simulador registra velocidad y energía de cada impacto.

## 3. Comportamiento

**2D.** Límite de posición del frente del pallet; los tiempos de subida y bajada retrasan el efecto; sobre cadenas el pallet se detiene en `stopContactTime` ([[S02 Mesa de cadenas]]).
**3D.** Cuerpo cinemático con colisionador; admite rebote y desalineación. Hay que configurar el umbral de restitución y la holgura del motor ([[A07 Envoltorio del motor físico]]).

## 4. Enclavamiento

No subir bajo un pallet (la zona sobre el tope debe estar libre).

## 5. Pruebas

| Prueba    | Criterio                                                    | Estado         |
| --------- | ----------------------------------------------------------- | -------------- |
| T5        | 1025 kg a 0,25 m/s → 32 J                                   | ✅ fórmula     |
| P1 / S1-c | Frente ±5 mm, rebote < 10 mm                                | ⏳ S1 / fase 4 |
| P2        | Velocidad y energía registradas; aviso al superar el máximo | ⏳ fase 4      |

## Relacionado

[[T14 Topes, impactos y alineación]] · [[T11 Contacto, rozamiento y restitución]]
