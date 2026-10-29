# 04 Generador y sumidero (Source y Drain)

## Source (generador) [PDF + C]

Produce las MU que entran en el sistema. Puede representar una máquina de producción o el muelle de recepción.

### Atributos

| Atributo            | Tipo         | Unidad | Descripción                                                              | Fuente |
| ------------------- | ------------ | ------ | ------------------------------------------------------------------------ | ------ |
| MU                  | referencia   | —      | Clase de MU que produce (pieza, contenedor…)                             | [PDF]  |
| Selección de MU     | enum         | —      | Fijo, aleatorio con tabla, secuencia                                     | [PDF]  |
| Tabla de reparto    | tabla        | —      | Columnas MU, Frecuencia (%), Nombre                                      | [PDF]  |
| Intervalo           | distribución | s      | Tiempo entre piezas. Ej.: exponencial negativa con media 1:00            | [PDF]  |
| Hora de inicio      | tiempo       | s      | Primera pieza                                                            | [C]    |
| Hora de parada      | tiempo       | s      | Última pieza                                                             | [C]    |
| Cantidad máxima     | entero       | uds.   | Para tras N piezas                                                       | [C]    |
| Criterio de entrega | enum         | —      | Por intervalo, por cantidad, por demanda del sucesor (pull), por lista   | [C]    |
| Bloqueo             | booleano     | —      | Si el sucesor está lleno: espera o pierde la pieza                       | [C]    |
| Tipo de llegada     | enum         | —      | Una a una o por lotes                                                    | [C]    |
| Estadísticas        | —            | —      | Piezas creadas, % trabajando / bloqueado                                 | [PDF]  |

### Observaciones

- Una distribución exponencial negativa para el intervalo da llegadas de tipo Poisson (aleatorias, sin memoria). Es una fuente típica de bloqueos aguas abajo. [PDF]
- Con semilla fija, mismas llegadas: obligatorio en IntraCraft (nunca `Math.random()`).
- Un generador **a intervalo fijo** debe respetar el espacio físico del puerto de salida: en 3D no se pueden crear dos pallets solapados. [C]

## Drain (sumidero) [PDF + C]

Retira las MU del modelo: equivale al muelle de expedición.

### Atributos

| Atributo              | Tipo     | Unidad | Descripción                                                                              | Fuente    |
| --------------------- | -------- | ------ | ---------------------------------------------------------------------------------------- | --------- |
| Tiempo de proceso     | tiempo   | s      | Retraso antes de retirar (0 = inmediato)                                                 | [C]       |
| Capacidad             | entero   | uds.   | Normalmente ilimitada                                                                    | [C]       |
| Estadísticas          | —        | —      | Número de MU entradas, rendimiento (uds./h), tiempo en el sistema, tiempo de ciclo medio | [PDF + C] |
| Registrar por tipo    | booleano | —      | Estadística separada por clase de MU                                                     | [C]       |

### Observaciones

- El sumidero es el sitio natural para leer el **rendimiento (throughput)** y el **tiempo de tránsito**.
- Tutorial: dos sumideros distintos (Monitores y Consolas) separaron el flujo con un control de salida. [PDF]
- El informe HTML del tutorial incluye estadísticas del sumidero; los resultados con y sin animación coinciden. [PDF]

## Pruebas de aceptación sugeridas para IntraCraft [C]

1. Con intervalo constante y capacidad sobrada, el sumidero recibe `N = tiempo / intervalo` pallets.
2. Con la misma semilla, el sumidero registra los mismos tiempos de llegada en cada ejecución.
3. Si el generador está bloqueado más de un tiempo definido, se marca como estado «bloqueado» y se contabiliza en estadísticas.
