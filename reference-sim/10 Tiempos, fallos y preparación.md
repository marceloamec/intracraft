# 10 Tiempos, fallos y preparación

## Entrada de tiempos [PDF]

- Formato `min:seg` (1:30 = 90 s). Se separa con dos puntos.
- Cualquier tiempo puede ser **constante** o una **distribución**; el selector muestra los parámetros que esa distribución exige.

## Distribuciones [PDF + C]

| Distribución         | Parámetros (en el orden del programa)                          | Uso típico                          | Fuente |
| -------------------- | -------------------------------------------------------------- | ----------------------------------- | ------ |
| Constante            | valor                                                          | Cinta, ciclo fijo                   | [PDF]  |
| Normal               | media, desviación, mínimo, máximo (ej. 0:55, 0:05, 0:01, 1:30) | Procesos manuales y de prueba       | [PDF]  |
| Exponencial negativa | media (ej. 1:00)                                               | Llegadas aleatorias                 | [PDF]  |
| Uniforme             | mínimo, máximo                                                 | Sin información, rango              | [C]    |
| Triangular           | mínimo, moda, máximo                                           | Estimaciones de expertos            | [C]    |
| Erlang / Gamma       | media, forma                                                   | Procesos con varias fases           | [C]    |
| Empírica (tabla)     | valores y frecuencias                                          | Datos medidos                       | [C]    |

Requisitos para IntraCraft: el generador aleatorio con semilla del proyecto; separar flujos por objeto (cada generador, su propio flujo) para que añadir un objeto **no cambie** el resultado de los demás. [C]

## Preparación (set-up) [PDF]

Pestaña **Set-Up**: tiempo que pierde un recurso al cambiar de tipo de MU (de monitor a consola). Atributos [C]:

| Atributo               | Descripción                                                |
| ---------------------- | ---------------------------------------------------------- |
| Tiempo de set-up       | s; constante o distribución                                |
| Matriz de cambio       | Tiempo según tipo anterior → tipo nuevo                    |
| Cuenta como            | Estado «preparando» en estadísticas                        |

En manutención de pallets el equivalente es el **cambio de orientación** o ajuste de guías: fuera del MVP.

## Fallos (averías) [PDF]

Pestaña **Failures**. Interrumpen el objeto. Atributos habituales [C]:

| Atributo               | Unidad | Descripción                                                        |
| ---------------------- | ------ | ------------------------------------------------------------------ |
| Disponibilidad         | %      | Porcentaje de tiempo operativo                                     |
| MTBF                   | s      | Tiempo medio entre fallos                                          |
| MTTR                   | s      | Tiempo medio de reparación                                         |
| Distribución           | —      | Del tiempo hasta fallo y de reparación                             |
| Comportamiento         | enum   | Parar y reanudar el proceso / perder la pieza / parar sin reanudar |
| Dependencia            | enum   | Por tiempo calendario o por tiempo de trabajo                      |

Relación: `Disponibilidad = MTBF / (MTBF + MTTR)`.

Los fallos están **fuera del MVP** de IntraCraft (ver AGENTS.md). Aquí solo queda anotado el modelo para cuando se amplíe.

## Pausas y turnos [PDF + C]

Las interrupciones del objeto incluyen **pausas** (el PDF las nombra junto a los fallos). Un **calendario de turnos** define cuándo trabaja el recurso. Atributos [C]: turnos, descansos, días libres, precedencia entre calendarios. Fuera del MVP.

## Tiempo de recuperación [C]

Tras procesar, algunos recursos necesitan un tiempo antes de aceptar la siguiente MU. Es el equivalente a la **separación mínima** que IntraCraft ya modela en los transportadores.
