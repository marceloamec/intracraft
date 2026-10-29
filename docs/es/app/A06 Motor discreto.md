---
title: A06 Motor discreto
lang: es
tags: [app, des, sim-des]
---

🌐 [[A06 DES Engine|EN]] · **ES** · [[A06 DES-Engine|DE]] · [[Inicio (ES)|Índice]]

# A06 Motor discreto (`packages/sim-des`)

Teoría: [[T02 Simulación de eventos discretos]], [[T03 Tiempo, orden de eventos y determinismo]], [[T04 Cinemática de transportadores]], [[T05 Acumulación, capacidad y rendimiento]].

## 1. Qué existe hoy

| Módulo           | Contenido                                                                                                                                | Pruebas                                    |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `event-queue.ts` | `EventQueue<T>`: montículo binario ordenado por (tiempo, prioridad, secuencia); solo µs enteros                                          | Orden, empates, rechazo de no enteros      |
| `simulator.ts`   | `DesSimulator`: reloj, `on(tipo, manejador)`, `schedule(retardo, evento, prioridad)`, `cancel(tique)`, `runUntil(t)`, `trace` de eventos | Saltos de reloj, cancelación, determinismo |
| `kinematics.ts`  | `profileTime`, `transferCycleTime`, `maxThroughput`, `palletCapacity`, `impactEnergy`, `positionAt`, `velocityAt`, `timeToReach`         | T1–T5, caso triangular, segmentos          |

## 2. Diseño previsto (fase 3)

### Pistas

Cada transportador es una **pista** con una lista ordenada de pallets (el primero delante). Un pallet en una pista guarda:

- su segmento cinemático actual `{t0, s0, v0, a}`;
- su longitud en el sentido del avance (depende de la orientación);
- un número de **versión**, que aumenta cada vez que cambia su movimiento, para invalidar predicciones obsoletas.

### Ciclo de predicción

Tras cualquier evento que afecte a un pallet:

1. Cerrar el segmento actual en `ahora` y abrir uno nuevo (nueva velocidad/aceleración).
2. Calcular los cruces candidatos con `timeToReach`: fin de rampa, siguiente flanco de sensor, tope, fin de zona, cola del líder − separación mínima, fin de pista.
3. Programar solo el **más temprano**; cancelar la predicción anterior.

### Tipos de evento

`pallet_enters`, `ramp_ends`, `front_reaches_stop`, `follower_reaches_leader`, `tail_clears_entry`, `sensor_edge`, `zone_released`, `turn_completed`, `lift_completed`, `transfer_completed`, `signal_changed`, `rule_fired`.

### Entrega entre pistas

En `front_reaches_end` el pallet se registra en la pista de aguas abajo (si puede aceptarlo); mientras está a caballo, su velocidad sigue la regla de entrega ([[T04 Cinemática de transportadores]] §9). Cuando la cola deja la pista de aguas arriba, se elimina de su lista.

### Estadísticas

Por objeto: tiempo por estado, contadores y muestras (tiempos de tránsito) con acumuladores ponderados en el tiempo en µs enteros ([[T08 Estadísticas y análisis de resultados]]).

## 3. Objetivo de rendimiento

≥ 1000× el tiempo real con 50 objetos y 30 pallets. Cada evento cuesta $O(\log n)$ en el montículo más un trabajo constante por pallet afectado; el visor evalúa `positionAt` por fotograma, no por evento.

## 4. Reglas para quien contribuya

- Nada de `Math.random()`, `Date.now()`, DOM ni React (lo impone ESLint).
- Todos los tiempos que entran en la cola son enteros (µs); convertir una sola vez con `secondsToSimTime`.
- Cada tipo de evento nuevo lleva una prueba que compara su instante con una fórmula analítica.

## Relacionado

[[A02 Arquitectura]] · [[A08 Controlador]] · [[A14 Referencia de API]]
