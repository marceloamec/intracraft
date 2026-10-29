# 06 Transportadores (Conveyor)

## Qué es [PDF]

El **Conveyor** mueve MU entre dos objetos. Se dibuja con dos clics (inicio y fin) y se conecta al predecesor y al sucesor. En el tutorial lleva piezas desde el generador hasta la estación de pre-prueba.

Variante de animación [PDF]: la configuración 3D permite elegir **banda** (por defecto) o **rodillos**, con patas en T arqueadas cada 1,5 m. Es solo apariencia; la simulación no cambia.

## Atributos de simulación [PDF + C]

| Atributo                               | Tipo      | Unidad | Descripción                                                                                    | Fuente |
| -------------------------------------- | --------- | ------ | ---------------------------------------------------------------------------------------------- | ------ |
| Longitud                               | real      | m      | Entre inicio y fin; se fija al dibujarlo                                                       | [PDF]  |
| Velocidad                              | real      | m/s    | Constante en la versión simple                                                                 | [C]    |
| Aceleración / frenado                  | real      | m/s²   | Si se modela (IntraCraft sí: T1 usa a = 0,5 m/s²)                                              | [C]    |
| Acumulante                             | booleano  | —      | Sí: las MU se acumulan juntas al llegar al final; no: la cinta sigue moviéndolas y se bloquean | [C]    |
| Distancia mínima entre MU              | real      | m      | Separación en acumulación (T2 usa 0,15 m)                                                      | [C]    |
| Capacidad calculada                    | entero    | uds.   | Longitud / (largo de MU + separación)                                                          | [C]    |
| Longitud de la MU en sentido de avance | real      | m      | Depende de la orientación del pallet en el transportador                                       | [C]    |
| Sentido                                | enum      | —      | Un solo sentido o reversible                                                                   | [C]    |
| Sensores                               | lista     | —      | Posición (m desde el inicio) + control a ejecutar                                              | [PDF]  |
| Control de entrada                     | método    | —      | Al entrar una MU                                                                               | [PDF]  |
| Control de salida                      | método    | —      | Al salir una MU                                                                                | [PDF]  |
| Fallos                                 | modelo    | —      | Paros del transportador                                                                        | [PDF]  |
| Estadísticas                           | —         | —      | % de tiempo trabajando, ocupación, MU transportadas                                            | [PDF]  |

## Fórmulas de referencia [C]

- **Tiempo de recorrido sin acumulación:** `t = L / v` (+ rampa de arranque si hay aceleración).
- **Con aceleración desde reposo** (T1 de IntraCraft: 3 m, v = 0,25 m/s, a = 0,5 m/s²): `t = L/v + v/(2a)` → 12,25 s.
- **Rendimiento máximo:** `Q = v / (largo MU + separación)` (T2: ≈ 0,185 pallets/s, ≈ 667/h).
- **Capacidad de acumulación:** `N = ⌊L / (largo MU + separación)⌋`.

## Estados que se observan [PDF]

En el ejemplo del tutorial, el transportador trabaja el 100 % del tiempo con el ajuste de fábrica; la utilización **no cambia** al variar tiempos aguas arriba si ya estaba saturado.

## Rodillos vs cadenas [C]

La referencia comercial no distingue la física: ambos son «transportador de longitud y velocidad». IntraCraft sí debe distinguirlos (ver [08 Objetos físicos.md](08%20Objetos%20físicos.md)):

| Característica          | Rodillos                                    | Cadenas                                         |
| ----------------------- | ------------------------------------------- | ----------------------------------------------- |
| Soporte del pallet      | Tablas inferiores apoyadas en rodillos      | Tablas en 2–3 cadenas paralelas                 |
| Acumulación sin presión | Sí, con zonas accionadas                    | Normalmente no                                  |
| Arrastre                | Giro del rodillo ω = v/r                    | Fricción cadena-madera                          |
| Giro del pallet         | Con mesas giratorias de rodillos            | Con mesas giratorias de cadenas                 |
| Uso típico              | Acumulación, entradas, salidas              | Transferencia, cambios de sentido               |

## Zonas (para acumulación) [C]

Un transportador de rodillos de acumulación se divide en **zonas** de longitud ≈ un pallet. Cada zona tiene motor, sensor y control. Atributos de zona: longitud, estado (en marcha / parada), sensor asociado, regla de parada («si hay pallet en la zona siguiente, parar»).

## Estructura de especificación sugerida para IntraCraft [C]

Para cada transportador del MVP: parámetros (longitud, velocidad, aceleración, separación, acumulación), puertos (entrada, salida), señales (ocupado, en movimiento), validaciones (separación ≥ 0, longitud ≥ largo de pallet), comportamiento 2D (eventos entrada/salida con la fórmula de arriba), comportamiento 3D (rodillos/cadenas con fricción), plantilla de control y escenario de prueba.
