# 03 MU: unidades móviles (tipos y atributos)

**MU** (Mobile Unit) es lo que se mueve por el flujo: piezas, contenedores, vehículos. En IntraCraft la MU principal es el **pallet**.

## Tipos de MU [PDF + C]

| Tipo        | Qué es                                                       | Fuente | Uso en IntraCraft                  |
| ----------- | ------------------------------------------------------------ | ------ | ---------------------------------- |
| Part        | Pieza simple, no transporta otras (monitor, consola)         | [PDF]  | Carga dentro del pallet            |
| Container   | MU que **lleva** otras MU encima (caja, pallet, bandeja)     | [PDF]  | **Pallet** (Europeo, Industrial…)  |
| Transporter | MU **con movimiento propio** que lleva otras MU por vías     | [C]    | Fuera del MVP (AGV, carretillas)   |

Las clases de MU viven en una carpeta propia de la biblioteca de clases. Se crean subclases (Monitor, Consola) con nombre propio y se reparten en el generador por frecuencia. [PDF]

## Atributos comunes a toda MU [C]

| Atributo             | Tipo        | Unidad | Descripción                                                          |
| -------------------- | ----------- | ------ | -------------------------------------------------------------------- |
| Nombre               | texto       | —      | Nombre de clase o instancia (se lee en los controles: `@.cont.name`) |
| Número / id          | entero      | —      | Identificador único por instancia                                    |
| Longitud             | real        | m      | Dimensión en el sentido de avance (cuenta para acumulación)          |
| Anchura              | real        | m      | Dimensión transversal                                                |
| Altura               | real        | m      | Importante para sensores y topes en 3D                               |
| Peso                 | real        | kg     | Entrada para la física y la energía                                  |
| Tiempo de entrada    | tiempo      | s      | Cuándo entró al modelo (para tiempo de tránsito)                     |
| Atributos de usuario | mapa        | —      | Pares clave/valor (destino, tipo, prioridad…)                        |
| Estado               | enumerado   | —      | Moviéndose, esperando, bloqueado, cargando…                          |
| Contenido            | lista       | —      | Solo contenedor: MU que lleva encima                                 |
| Posición             | real        | m      | Posición sobre el recurso actual (para sensores)                     |
| Gráfico/color        | referencia  | —      | Se cambia por clase; puede colorear por estado [PDF]                 |

## Container (pallet) [PDF + C]

El contenedor se carga en una **estación de ensamblado** (modo «adjuntar MU») y viaja por el transportador llevando su carga. [PDF]

Atributos propios [C]:

| Atributo             | Unidad | Descripción                                                                   |
| -------------------- | ------ | ----------------------------------------------------------------------------- |
| Capacidad (nº de MU) | uds.   | Cuántas piezas admite                                                         |
| Peso máximo          | kg     | Carga útil                                                                    |
| Estado de carga      | enum   | Vacío / parcial / lleno                                                       |
| Posiciones de carga  | lista  | Coordenadas relativas de cada pieza dentro del contenedor                     |
| Peso vacío (tara)    | kg     | Se suma a la carga para el peso total                                         |

Para IntraCraft, además de lo anterior, la ficha de pallet debe definir lo que pide AGENTS.md: dimensiones (EUR 1,2 × 0,8 m), tablas inferiores, bloque de carga, masa y centro de masa. Ver [08 Objetos físicos.md](08%20Objetos%20físicos.md).

## Reparto de tipos en el generador [PDF]

El generador elige el tipo con tres modos:

1. **Fijo:** siempre el mismo MU.
2. **Aleatorio con tabla:** una tabla de datos con columnas MU, Frecuencia, Nombre. Ejemplo del tutorial: 70 % monitores y 30 % consolas. [PDF]
3. **Secuencial / por pedido:** según lista. [C]

## Transporter (MU móvil) [C]

No aparece en el PDF; descripción general del sector:

| Atributo               | Unidad | Descripción                                               |
| ---------------------- | ------ | --------------------------------------------------------- |
| Velocidad              | m/s    | Con carga y sin carga                                     |
| Aceleración            | m/s²   | Arranque                                                  |
| Deceleración           | m/s²   | Frenado                                                   |
| Capacidad              | uds.   | MU que puede llevar a la vez                              |
| Tiempo de carga        | s      | Colocar la MU encima                                      |
| Tiempo de descarga     | s      | Dejarla en destino                                        |
| Distancia mínima       | m      | Separación de seguridad con otros transportadores         |
| Orientación            | rad    | Dirección actual                                          |
| Estado                 | enum   | Libre, yendo a recoger, cargado, bloqueado                |
| Control de ruteo       | script | Decide el siguiente destino                               |

Es lo que se necesita para AGV/carretillas. El MVP no los incluye (ver [AGENTS.md](../AGENTS.md)), pero el modelo de datos de MU debería poder crecer hacia ahí. Ver [07 Vías, transportadores móviles y giros.md](07%20Vías,%20transportadores%20móviles%20y%20giros.md).

## Comportamientos útiles de MU [C]

- **Adjuntar/Separar:** un contenedor recoge y suelta piezas (ensamblado/desensamblado).
- **Destino:** atributo que alimenta el ruteo.
- **Tiempo de residencia:** estadística por MU, base de tiempos de ciclo.
- **Estado por color:** diagnóstico visual (bloqueada, en espera).
