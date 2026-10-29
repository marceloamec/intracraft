# 05 Estación, buffer y objetos de proceso

## Station (estación) [PDF + C]

Objeto genérico donde una MU **pasa un tiempo** (máquina, puesto, prueba). Es la pieza base de casi todo modelo DES.

### Atributos

| Atributo               | Tipo          | Unidad | Descripción                                                          | Fuente |
| ---------------------- | ------------- | ------ | -------------------------------------------------------------------- | ------ |
| Nombre / etiqueta      | texto         | —      | Se puede mostrar sobre el icono                                      | [PDF]  |
| Capacidad              | entero        | uds.   | MU que caben a la vez (por defecto 1)                                | [C]    |
| Tiempo de proceso      | distribución  | s      | Por defecto 1 minuto; puede ser constante o distribución estadística | [PDF]  |
| Tiempo de ciclo        | tiempo        | s      | Si es distinto del tiempo de proceso (p. ej. con pipeline)           | [C]    |
| Tiempo de recuperación | tiempo        | s      | Tras cada pieza                                                      | [C]    |
| Factor de tiempo       | real          | ×      | Multiplica los tiempos (en el tutorial, el robot pasa de 1 a 2)      | [PDF]  |
| Preparación (set-up)   | tiempo        | s      | Al cambiar de tipo de MU                                             | [PDF]  |
| Fallos                 | modelo        | —      | Averías, paros                                                       | [PDF]  |
| Estrategia de salida   | enum          | —      | Ver [02](02%20Conexiones%20y%20flujo%20de%20material.md)             | [PDF]  |
| Control de entrada     | método        | —      | Se ejecuta al entrar una MU                                          | [PDF]  |
| Control de salida      | método        | —      | Se ejecuta al salir, decide destino                                  | [PDF]  |
| Bloqueante             | booleano      | —      | Espera el sucesor elegido                                            | [PDF]  |
| Estadísticas           | —             | —      | % de trabajo, esperando, bloqueado, en fallo, en pausa               | [PDF]  |

### Tiempos [PDF]

- Los tiempos se escriben `min:seg` (1:30 = 1 min 30 s).
- **Normal** (media 0:55, desviación 0:05, mínimo 0:01, máximo 1:30): parámetros mostrados sobre el cuadro de texto.
- **Exponencial negativa** (media 1:00): para llegadas y tiempos con mucha variabilidad.
- Una estación con tiempo de proceso > intervalo del generador crea cuello de botella: la pieza anterior queda **bloqueada** (amarilla). [PDF]

### Efecto de cambiar parámetros (ejemplo del tutorial) [PDF]

- Subir el tiempo de la primera estación de 10 s a 2 min: su utilización baja al 70 %, las siguientes suben, el robot espera piezas.
- Subir el factor de tiempo del robot de 1 a 2: baja la utilización de las estaciones, el robot al 100 %, el transportador no cambia.

Este tipo de «experimento de sensibilidad» debería ser un escenario de referencia en `packages/scenarios`. [C]

## Buffer (almacén de espera) [PDF]

Acumula MU para **absorber diferencias de ritmo** entre dos objetos.

| Atributo           | Tipo       | Unidad | Descripción                                               | Fuente |
| ------------------ | ---------- | ------ | --------------------------------------------------------- | ------ |
| Capacidad          | entero     | uds.   | Por defecto 8; en el tutorial pasa a 20                   | [PDF]  |
| Disciplina         | enum       | —      | FIFO por defecto; LIFO, prioridad, aleatoria              | [C]    |
| Tiempo de proceso  | tiempo     | s      | Mínimo de estancia (normalmente 0)                        | [C]    |
| Ocupación          | histograma | —      | % de tiempo con cada nº de piezas                         | [PDF]  |

- Cuando llega a su capacidad no admite más hasta que salga una pieza. [PDF]
- El histograma de **ocupación** (gráfico Chart sobre el buffer) es el modo estándar de comprobar si el buffer está bien dimensionado. [PDF]

Equivalente en IntraCraft: una **zona de acumulación** en un transportador de rodillos (capacidad = longitud útil / (largo del pallet + separación)). [C]

## Ensamblado (AssemblyStation) [PDF]

Une MU: carga piezas sobre un contenedor.

| Atributo                        | Descripción                                                          |
| ------------------------------- | -------------------------------------------------------------------- |
| Modo de ensamblado              | «Adjuntar MU» coloca la pieza dentro del contenedor [PDF]            |
| MU principal desde predecesor n | Indica cuál de los dos predecesores aporta el contenedor [PDF]       |
| Tiempo de proceso               | Ej. 50 s [PDF]                                                       |
| Cantidad a adjuntar             | Cuántas piezas por contenedor [C]                                    |

Con dos predecesores, el orden de conexión decide cuál es el principal: error frecuente. [PDF]

## Otros objetos de proceso (breve) [C]

| Objeto genérico       | Para qué sirve                                               | Para IntraCraft |
| --------------------- | ------------------------------------------------------------ | --------------- |
| Proceso paralelo      | Varios puestos idénticos en paralelo                         | Futuro          |
| Desensamblado         | Separa la carga del contenedor                               | Futuro          |
| Clasificador (sorter) | Reparte por criterio                                         | Útil (ruteo)    |
| Almacén               | Posiciones fijas con entrada/salida                          | Fuera de MVP    |
| Ciclo                 | Recorre un bucle                                             | Prescindible    |
| Robot pick-and-place  | Pasa piezas de un punto a otro con tiempo                    | Fuera de MVP    |

El robot del tutorial coge piezas de las estaciones y las deja en el transportador; el modelo 3D se puede intercambiar por otro gráfico. [PDF]
