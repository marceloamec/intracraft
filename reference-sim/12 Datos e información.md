# 12 Datos e información

Son los objetos de «flujo de información»: no mueven material, guardan y calculan.

## Tabla de datos (DataTable) [PDF]

Matriz con filas y columnas, tipada (entero, real, texto, objeto…).

| Uso en el tutorial                                                       | Fuente |
| ------------------------------------------------------------------------ | ------ |
| **Tabla de reparto** para el generador (columnas MU, Frecuencia, Nombre) | [PDF]  |
| **Resultados:** contadores por tipo de pieza, con índice de columnas     | [PDF]  |
| Origen de un gráfico (arrastrar la tabla al Chart)                       | [PDF]  |

Atributos: [C]

| Atributo              | Descripción                                             |
| --------------------- | ------------------------------------------------------- |
| Formato por celda     | Tipo de dato                                            |
| Índice de columna     | Nombres en la fila 0 para acceder por nombre            |
| Índice de fila        | Nombres para acceder por clave                          |
| Valor inicial         | Se aplica al empezar la simulación                      |
| Borrado en reset      | Se hace con un método `reset` [PDF]                     |

Para IntraCraft, las tablas deberían ser **datos de escenario** (JSON), no objetos del layout: una tabla de pedidos o de reparto de pallets.

## Variable [PDF]

Contenedor de un valor (entero, real, texto…).

| Atributo        | Descripción                                                          |
| --------------- | -------------------------------------------------------------------- |
| Nombre          | Referenciable desde controles                                        |
| Tipo            | Entero, real, booleano, texto, tiempo                                |
| Valor inicial   | «Initial value»: se restablece al arrancar la simulación [PDF]       |
| Mostrar         | Tamaño de fuente, color para visualizar el valor en el modelo [PDF]  |

Ejemplo: `NumberOfDevices := NumberOfDevices + 1` ejecutado al pasar una pieza por el sensor. [PDF]

## Método (Method) [PDF]

Objeto con **código de script** para comportamientos que no se pueden fijar con ajustes. Lenguaje propietario (SimTalk en la referencia).

- Se asocia a eventos (entrada, salida, sensor, reinicio).
- Dentro, `@` representa el objeto que lo llama. `@.cont` es la MU contenida; `@.move(destino)` mueve la MU. [PDF]
- Un método llamado `reset` se ejecuta al reiniciar. [PDF]
- Se aplica con un botón «Aplicar cambios» antes de cerrarlo. [PDF]

Ejemplo [PDF]:

```
si @.cont.name = "Monitor"
   @.move(Monitores)
   Resultados["Monitores",1] += 1
si no
   @.move(Consolas)
   Resultados["Consolas",1] += 1
fin
```

(Traducido al español; el original está en inglés.)

## Otros objetos de información [C]

| Objeto                 | Para qué                                                | IntraCraft          |
| ---------------------- | ------------------------------------------------------- | ------------------- |
| Lista / cola           | Estructuras de datos                                    | Interno al motor    |
| Generador de números   | Distribución con semilla                                | Sí, interno         |
| Temporizador (Trigger) | Lanza un método en un instante                          | Reglas con retardo  |
| Importador/Exportador  | Leer/escribir datos externos (Excel, CSV)               | CSV/JSON en CLI     |
| Interfaz de usuario    | Mostrar valores, botones                                | Panel de la UI      |

## Decisión para IntraCraft [C]

Los scripts libres están **fuera de alcance** hasta nuevo aviso. Un escenario puede incluir: tablas JSON de pedidos, semilla, duración y reglas Si/Entonces. Esto cubre lo que el tutorial hace con métodos (contar, ruteo por tipo, registrar resultados).
