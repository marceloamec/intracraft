# 08 Objetos físicos (modo 3D con física)

> Casi todo es [C]. La referencia comercial del PDF anima en 3D pero **no simula física de contacto** en el tutorial: el 3D es visualización del resultado DES. IntraCraft sí necesita física, así que esta ficha define lo que hay que modelar. Valores por defecto: orientativos, no de fabricante.

## Principios (de AGENTS.md) [C]

- Paso fijo propuesto 1/120 s; controlador cada 10 ms que escribe consignas.
- Unidades SI; ejes Y arriba, mano derecha.
- Rodillos: cilindros cinemáticos con ω = v/r. Cadenas: tiras de rodillos pequeños (plan A).
- Acceso al motor solo vía `sim-physics`.

## Cuerpo rígido (genérico) [C]

| Atributo                     | Unidad  | Descripción                                          |
| ---------------------------- | ------- | ---------------------------------------------------- |
| Tipo de cuerpo               | enum    | Estático, dinámico, cinemático                       |
| Masa                         | kg      | Solo dinámicos                                       |
| Centro de masa               | m       | Relativo al origen del cuerpo                        |
| Inercia                      | kg·m²   | Calculada de la forma o explícita                    |
| Fricción                     | —       | Coeficiente estático/dinámico por material           |
| Restitución                  | —       | Rebote (debe ser casi 0 para cumplir ±5 mm en tope)  |
| Holgura de penetración       | m       | Ajustar (los valores por defecto incumplen)          |
| Amortiguación lineal/angular | 1/s     | Disipación                                           |
| Forma de colisión            | —       | Caja, cilindro, malla convexa, compuesta             |
| Capa de colisión             | máscara | Con qué choca (pallet↔rodillo sí; pallet↔sensor no)  |

## Pallet (cuerpo dinámico compuesto) [C]

| Parámetro            | Valor de ejemplo | Nota                                           |
| -------------------- | ---------------- | ---------------------------------------------- |
| Largo × ancho        | 1,2 × 0,8 m      | EUR (lado largo en avance)                     |
| Altura de tablas     | ≈ 0,144 m        | Tablas inferiores                              |
| Masa pallet vacío    | ≈ 25 kg          | Orientativo                                    |
| Masa de carga        | hasta ≈ 1000 kg  | T5: 1025 kg                                    |
| Centro de masa       | variable         | Según carga                                    |
| Material tablas      | madera           | Fricción con rodillo y cadena                  |

Composición: tablas inferiores (formas de caja) + bloque de carga (caja con masa y centro de masa explícitos).

## Rodillo [C]

| Atributo               | Unidad | Descripción                                      |
| ---------------------- | ------ | ------------------------------------------------ |
| Radio                  | m      | p. ej. 0,025–0,05                                |
| Longitud               | m      | Ancho útil                                       |
| Paso entre rodillos    | m      | Distancia entre ejes                             |
| Velocidad tangencial   | m/s    | Consigna `v`; ω = v/r                            |
| Aceleración            | m/s²   | Rampa de la consigna                             |
| Fricción               | —      | Contra la madera                                 |
| Modo                   | enum   | Accionado, libre (loco)                          |
| Zona                   | id     | Agrupación para control                          |

## Cadena [C]

| Atributo            | Unidad | Descripción                                                        |
| ------------------- | ------ | ------------------------------------------------------------------ |
| Número de cadenas   | uds.   | 2–3                                                                |
| Separación          | m      | Entre cadenas                                                      |
| Velocidad           | m/s    | Consigna                                                           |
| Fricción            | —      | Contra la madera (mayor que el rodillo)                            |
| Modelo              | enum   | Tira de rodillos pequeños (plan A) / superficie móvil por contacto |

La superficie móvil por contacto: Jolt JS 1.1.1 la expone (`mRelativeLinearSurfaceVelocity`), Rapier JS 0.21.0 no (ver AGENTS.md).

## Tope mecánico [C]

| Atributo                | Unidad | Descripción                                   |
| ----------------------- | ------ | --------------------------------------------- |
| Posición                | m      | A lo largo del transportador                  |
| Altura                  | m      | Cuánto sobresale                              |
| Estado                  | enum   | Arriba, abajo                                 |
| Tiempo de subida/bajada | s      | Mecanismo neumático                           |
| Rigidez                 | N/m    | Evitar rebote                                 |
| Tolerancia de parada    | m      | ±5 mm criterio de IntraCraft                  |

Criterio de aceptación: el pallet queda con el frente a ±5 mm del tope y rebote < 10 mm.

## Guía lateral [C]

| Atributo           | Unidad | Descripción                        |
| ------------------ | ------ | ---------------------------------- |
| Longitud           | m      | A lo largo del transportador       |
| Altura             | m      | Contacto con el pallet             |
| Separación         | m      | Luz libre (entre guías)            |
| Fricción           | —      | Contra el costado                  |
| Chaflán de entrada | m      | Centra el pallet                   |

## Sensor fotoeléctrico (con geometría real) [C]

| Atributo         | Unidad | Descripción                                     |
| ---------------- | ------ | ----------------------------------------------- |
| Posición         | m      | A lo largo y a lo ancho                         |
| Altura           | m      | Del haz respecto al suelo                       |
| Tipo             | enum   | Barrera, difuso, reflexivo                      |
| Alcance          | m      | Distancia del haz                               |
| Filtro / retardo | s      | Anti-rebote                                     |
| Polaridad        | enum   | Activo con presencia o en reposo                |

En 3D se evalúa por **rayo** contra los cuerpos; en 2D por **posición del frente/cola** de la MU. Ambos deben coincidir dentro de ±50 ms.

## Coherencia 2D ↔ 3D [C]

| Magnitud               | Cómo se obtiene en 2D                     | Cómo se obtiene en 3D                    |
| ---------------------- | ----------------------------------------- | ---------------------------------------- |
| Tiempo de recorrido    | Fórmula analítica                         | Integración física                       |
| Parada en tope         | Instante de llegada                       | Contacto + amortiguación                 |
| Separación entre MU    | Parámetro                                 | Resultado de la física (medir)           |
| Eventos de sensor      | Cruce de posición                         | Rayo                                     |
