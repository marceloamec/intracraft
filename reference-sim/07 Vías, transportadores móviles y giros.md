# 07 Vías, transportadores móviles y giros

> Esta sección es casi toda [C]: el PDF solo muestra un transportador lineal. Se incluye porque el MVP de IntraCraft necesita giratorias y mesa de transferencia.

## Vía (Track) [C]

Segmento donde se mueven MU o vehículos. Diferencia con el Conveyor: la vía es el **camino**; el vehículo (Transporter) es el que se mueve.

| Atributo            | Unidad | Descripción                                               |
| ------------------- | ------ | --------------------------------------------------------- |
| Longitud            | m      | Longitud del camino                                       |
| Sentido             | enum   | Un sentido o dos                                          |
| Velocidad máxima    | m/s    | Límite para el vehículo                                   |
| Capacidad           | uds.   | Vehículos/MU a la vez                                     |
| Forma               | enum   | Recta, curva, definida por puntos                         |
| Radio               | m      | Solo curvas (fuera del MVP)                               |

## Mesa giratoria (TurnTable) [C]

Objeto que **rota la MU** sobre su eje y la entrega a un puerto distinto. En IntraCraft hay dos: de cadenas y de rodillos.

| Atributo                  | Unidad  | Descripción                                                           |
| ------------------------- | ------- | --------------------------------------------------------------------- |
| Ángulo de giro            | rad     | 90° típico                                                            |
| Velocidad de giro ω       | rad/s   | 15 °/s en T3                                                          |
| Aceleración angular α     | rad/s²  | 30 °/s² en T3                                                         |
| Tiempo de giro            | s       | `t = θ/ω + ω/α` (T3: 90°/15 + 15/30 = 6,5 s)                          |
| Puertos de entrada/salida | —       | A 0°, 90°, 180°, 270°                                                 |
| Capacidad                 | uds.    | 1 pallet                                                              |
| Estado                    | enum    | Libre, cargando, girando, descargando                                 |
| Regla de salida           | script  | Qué puerto toma el pallet                                             |

Ciclo típico: pallet entra → se detiene (tope/sensor) → gira → sale por el puerto indicado → vuelve a la posición de recepción (si está definido). Cada fase tiene su propio tiempo.

## Mesa de transferencia (rodillos a cadenas) [C]

Cambia la dirección **90°** elevando un juego de cadenas sobre rodillos (o al revés).

| Atributo             | Unidad | Descripción                               |
| -------------------- | ------ | ----------------------------------------- |
| Carrera de elevación | m      | Altura de subida                          |
| Tiempo de subida     | s      | T4: 1 s                                   |
| Tiempo de bajada     | s      | T4: 1 s                                   |
| Velocidad de salida  | m/s    | T4: 0,2 m/s                               |
| Distancia de salida  | m      | T4: 1 m                                   |
| Estado               | enum   | Abajo, subiendo, arriba, bajando          |

Ciclo T4: subida + traslado + bajada + retorno ≈ 7,4 s.

## Transporter (vehículo) [C]

Descrito en [03](03%20MU%20Tipos%20y%20atributos.md). Fuera del MVP. Conviene reservar en el modelo de datos:

- Un **mapa de vías** (grafo) con nodos y aristas.
- Un **ruteo**: camino más corto, por reglas o por script.
- **Conflictos:** dos vehículos en una misma vía → reglas de preferencia.

## Puentes y cruces [C]

Cruces de rodillos y cadenas se consideran un caso de **transferencia** con regla de exclusión mutua: solo un pallet cruza a la vez. IntraCraft podría resolverlo con la mesa de transferencia + sensores, sin un objeto nuevo.

## Elevadores y curvas [C]

Fuera del MVP (ver [AGENTS.md](../AGENTS.md)). En la referencia comercial existen como objetos independientes; aquí solo se dejan anotados.
