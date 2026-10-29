---
title: T08 Estadísticas y análisis de resultados
lang: es
tags: [theory, statistics]
---

🌐 [[T08 Statistics and Output Analysis|EN]] · **ES** · [[T08 Statistik und Ergebnisanalyse|DE]] · [[Inicio (ES)|Índice]]

# T08 Estadísticas y análisis de resultados

## 1. Dos tipos de estadísticas

| Tipo                             | Ejemplos                                                | Cómo se calcula                                              |
| -------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------ |
| **Por observaciones** (muestras) | Tiempo de tránsito, tiempo de ciclo, energía de impacto | Una muestra por pallet/evento: media, mín., máx., desviación |
| **Ponderadas en el tiempo**      | Ocupación, longitud de cola, utilización                | Valor ponderado por el tiempo que duró                       |

Confundirlas es el error de estadísticas más común: la longitud media de la cola **no** es la media de las longitudes observadas en los eventos.

## 2. Medias ponderadas en el tiempo en un motor por eventos

Una variable de estado $x(t)$ (p. ej. número de pallets en un transportador) es constante a tramos entre eventos. Su media en $[0, T]$ es

$$\bar{x} = \frac{1}{T}\int_0^T x(t)\,dt = \frac{1}{T}\sum_i x_i\,\Delta t_i$$

Implementación: guardar `lastValue` y `lastChangeTime`; en cada cambio, sumar `lastValue × (ahora − lastChangeTime)` a un acumulador. Al final, cerrar el último intervalo. Con microsegundos enteros la acumulación es exacta.

## 3. Tiempos por estado y estadísticas estándar

Cada transportador está siempre en un solo estado (parado, acelerando, en marcha, deteniendo, bloqueado); el motor acumula el tiempo en cada uno. A partir de ellos:

| Estadística              | Definición                                               |
| ------------------------ | -------------------------------------------------------- |
| Utilización              | Tiempo llevando un pallet en movimiento / tiempo medido  |
| Tiempo bloqueado         | Tiempo con un pallet terminado que no puede salir        |
| Tiempo vacío             | Tiempo sin pallet                                        |
| Pallets procesados       | Contador de pallets que salieron por un puerto de salida |
| Rendimiento (pallets/h)  | Pallets procesados / tiempo medido × 3600                |
| Tiempo medio de tránsito | Media de (salida − entrada)                              |
| Ocupación media          | Número de pallets ponderado en el tiempo                 |

**Comprobación:** la ley de Little $\bar{L} = \lambda W$ debe cumplirse entre ocupación media, rendimiento y tiempo medio de tránsito ([[T05 Acumulación, capacidad y rendimiento]]).

## 4. Media y varianza numéricamente estables

Para las muestras se usa el **algoritmo de Welford** en lugar de sumar $x$ y $x^2$ (que pierde precisión):

```text
n ← n + 1
δ ← x − media
media ← media + δ / n
M2 ← M2 + δ · (x − media)
varianza = M2 / (n − 1)
```

## 5. Calentamiento (transitorio inicial)

Un modelo suele arrancar **vacío**, lo que no es representativo: los primeros pallets no encuentran colas y transitan más rápido. Las estadísticas tomadas desde $t = 0$ están sesgadas.

- Se elige un **periodo de calentamiento** tras el cual se reinician las estadísticas (MVP: un parámetro del escenario).
- Se encuentra dibujando una media móvil de la salida (p. ej. rendimiento cada 10 min) y viendo dónde se estabiliza (*método de Welch*).
- En una línea de mesas de 3 m el calentamiento es corto (lo que tarda en llenarse); en circuitos con giratorias puede ser de decenas de minutos.

## 6. Réplicas e intervalos de confianza

Con entradas **estocásticas** una ejecución es una muestra. Se ejecutan $n$ réplicas independientes (semillas distintas), se calcula el resultado $Y_j$ de cada una y se publica

$$\bar{Y} \pm t_{n-1,\,0{,}975}\,\frac{s}{\sqrt{n}}$$

Ejemplo: 5 réplicas dan 612, 598, 620, 605 y 615 pallets/h → $\bar{Y} = 610$, $s = 8{,}6$, $t_{4;0{,}975} = 2{,}776$ → **610 ± 10,7 pallets/h** (95 %). Si los intervalos de dos variantes de layout se solapan, la simulación todavía no ha demostrado una diferencia.

Un modelo **determinista** (intervalos fijos, masas fijas) necesita una sola ejecución. Las réplicas y los experimentos son post-MVP, pero el generador con semilla y los flujos por objeto ya están pensados para ellos ([[T03 Tiempo, orden de eventos y determinismo]]).

## 7. Medidas del modo físico

El modo 3D añade magnitudes sin equivalente en 2D:

- **Impactos**: velocidad y energía cinética en cada contacto con un tope; aviso por encima de `maxImpactEnergy`.
- **Pose final**: error de posición y de giro de un pallet en reposo contra un tope (calidad de la alineación).
- **Fuerzas de contacto**: p. ej. la presión de línea sobre un tope en acumulación con deslizamiento.

## 8. Registro de eventos y exportación

Cada evento procesado puede registrarse (`time`, `type`, `target`, datos extra). El registro:

- se exporta a CSV/JSON para analizarlo con otras herramientas;
- se resume en un hash para la prueba de determinismo;
- nunca se guarda dentro del archivo de proyecto (los resultados van aparte, plan §4.4).

## Relacionado

[[T05 Acumulación, capacidad y rendimiento]] · [[A11 Pruebas y validación]] · [[A10 CLI y escenarios]]
