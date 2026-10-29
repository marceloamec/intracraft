# 09 Sensores y controles

## Sensores en el transportador [PDF]

Un transportador tiene una **lista de sensores**. Cada sensor se define con:

| Campo      | Descripción                                                                | Fuente |
| ---------- | -------------------------------------------------------------------------- | ------ |
| Posición   | Distancia desde el inicio. 0 = al principio                                | [PDF]  |
| Control    | Método que se ejecuta cuando una MU pasa por el sensor                     | [PDF]  |
| Nombre     | Etiqueta                                                                   | [C]    |
| Flanco     | Al llegar el frente / al salir la cola                                     | [C]    |

Ejemplo del tutorial: sensor en posición 0 + método `counter` que suma 1 a una variable → cuenta cuántas piezas entran. [PDF]

## Controles de objeto [PDF]

Pestaña **Controls** del objeto: se asocian métodos a eventos.

| Evento                | Cuándo ocurre                                   | Uso típico                                     |
| --------------------- | ----------------------------------------------- | ---------------------------------------------- |
| Entrada               | Entra una MU                                    | Registrar, contar                              |
| Salida                | La MU está lista para salir                     | **Decidir destino** (`mover`) [PDF]            |
| Sensor                | Paso por un sensor                              | Contar, parar, desviar                         |
| Inicio / fin de fallo | Cambia el estado de avería                      | Registrar paros [C]                            |
| Reinicio              | Se resetea la simulación                        | Limpiar tablas [PDF]                           |

Ejemplo del tutorial [PDF]: control de salida que envía monitores a un sumidero y consolas a otro, y suma contadores en una tabla.

## Estructura de una regla Si/Entonces para IntraCraft [C]

El `control` del MVP usa reglas, no scripts. Una regla tiene:

| Parte      | Contenido                                                          |
| ---------- | ------------------------------------------------------------------ |
| Disparador | Evento (flanco de sensor, fin de temporizador, cambio de señal)    |
| Condición  | Combinación booleana de señales y atributos del pallet             |
| Acción     | Escribir consigna (arrancar/parar zona, subir/bajar tope, girar)   |
| Prioridad  | Orden de evaluación                                                |
| Retardo    | Temporizador `on-delay` / `off-delay`                              |

Ciclo de evaluación: cada 10 ms (AGENTS.md); en DES, en el instante del evento.

## Plantillas de control útiles [C]

1. **Acumulación sin presión:** una zona se detiene si la siguiente está ocupada.
2. **Parada en tope:** al activarse el sensor, subir tope y parar la zona.
3. **Liberación bajo demanda:** bajar el tope cuando el sucesor está libre.
4. **Desvío por atributo:** si destino = X, girar la mesa y salir por el puerto indicado.
5. **Anti-colisión en transferencia:** exclusión mutua en el cruce.
6. **Contador de pasos:** sensor que incrementa una variable de modelo.

## Lo que NO conviene copiar [C]

- Scripts de propósito general dentro de los sensores: dificultan el determinismo y las pruebas. IntraCraft prefiere reglas declarativas (escenarios de prueba reproducibles).
- Efectos secundarios ocultos en controles de salida (el orden de ejecución importa).
