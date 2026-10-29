---
title: T16 Consistencia híbrida y calibración
lang: es
tags: [theory, hybrid, validation]
---

🌐 [[T16 Hybrid Consistency and Calibration|EN]] · **ES** · [[T16 Hybride Konsistenz und Kalibrierung|DE]] · [[Inicio (ES)|Índice]]

# T16 Consistencia híbrida y calibración

## 1. Por qué difieren los dos modos

El modo 2D es exacto para una planta **ideal**; el modo 3D simula una **no ideal**. Las diferencias vienen de efectos que el modelo 2D no contiene:

| Efecto                        | Hipótesis 2D                            | Realidad 3D                                                   |
| ----------------------------- | --------------------------------------- | ------------------------------------------------------------- |
| Tracción                      | Velocidad del pallet = de la superficie | Microdeslizamiento; deslizamiento con μ bajo o rampas fuertes |
| Impacto en tope               | Parada instantánea en el tope           | Rebote, asentamiento, giro                                    |
| Entrega entre transportadores | Regla (p. ej. velocidad mínima)         | Contactos en los dos transportadores y el hueco               |
| Giratoria                     | Pallet fijo a la plataforma             | Pequeño deslizamiento al acelerar                             |
| Sensores                      | Flancos analíticos                      | Raycasts cuantizados por el paso, geometría real              |
| Control                       | Por eventos (instantáneo)               | Muestreado cada 10 ms                                         |

## 2. Criterios de consistencia (MVP)

Para cada escenario de referencia se ejecutan ambos modos y se compara:

| Magnitud             | Tolerancia |
| -------------------- | ---------- |
| Tiempos de tránsito  | ±5 %       |
| Instantes de flancos | ±50 ms     |

Una diferencia mayor es un **hallazgo**, no algo que esconder: o un error, o una regla que falta en 2D, o un parámetro de calibración que falta. Nunca se ajustan rozamientos, masas ni pasos de tiempo para que cuadren los números sin una nota de decisión (AGENTS.md).

## 3. Calibración (post-MVP, el diferenciador principal)

Se ejecutan **lotes de experimentos 3D** variando masa, rozamiento, velocidad de entrada y desalineación, y se extraen estadísticas:

- corrección del tiempo de tránsito por tipo de transportador (p. ej. +0,12 s por arranque/parada con μ = 0,3);
- probabilidad de atasco o de desalineación fuera de tolerancia;
- distribución del error de posición final en los topes.

Con ellas se crean **perfiles de calibración** que aplica el modo 2D (como retardos extra, distribuciones o probabilidades de fallo). Así el modo 2D gana realismo sin perder su velocidad de 1000×.

```mermaid
flowchart LR
  A[Lotes 3D<br/>varía m, μ, v, desalineación] --> B[Extraer estadísticas]
  B --> C[Perfil de calibración<br/>por tipo de componente]
  C --> D[El modo 2D aplica correcciones]
  D --> E[Estudios de rendimiento rápidos y realistas]
```

## 4. Cambio de modo en mitad de la simulación (post-MVP)

Cambiar de modo en marcha exige convertir el estado entre mundos. Solo está bien definido cuando los pallets están en **poses estables** (en reposo o moviéndose a velocidad conocida, no en mitad de un impacto). En el MVP el cambio reinicia la simulación.

## Relacionado

[[T01 Paradigmas de simulación]] · [[T06 Detección fotoeléctrica]] · [[A11 Pruebas y validación]]
