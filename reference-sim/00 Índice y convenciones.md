# Referencia de simulación de planta: índice y convenciones

Material de trabajo **privado**, fuera de `docs/` (no sigue la regla de tres idiomas). Sirve para comparar conceptos y resultados observables con un simulador comercial de eventos discretos de uso extendido (Tecnomatix Plant Simulation), tal como permite la regla clean-room de [AGENTS.md](../AGENTS.md): se toma la **idea**, nunca el código, los recursos ni el texto.

## Fuentes y marcas

| Marca | Significado                                                                                          |
| ----- | ---------------------------------------------------------------------------------------------------- |
| [PDF] | Está en «Introduction to Tecnomatix Plant Simulation» (v2201, tutorial de iniciación, 90 páginas).   |
| [C]   | Complemento mío, de conocimiento general del sector. **No sale del PDF; verificar antes de usarlo.** |

El PDF solo cubre un tutorial: Source, Drain, Station, Buffer, Conveyor, Connector, Interface, PickAndPlace, Container/Part, AssemblyStation, Frame, EventController, DataTable, Variable, Method, Chart. **No detalla** los atributos de MU, transportadores ni objetos físicos; esas fichas son mayormente [C].

## Reglas de uso

- Los nombres del programa comercial se citan solo en la columna «Equivalente en la referencia». En IntraCraft se usan nombres propios (ver [AGENTS.md](../AGENTS.md)).
- Los valores numéricos son orientativos, no datos de fabricante.
- Unidades de las fichas: las de IntraCraft (SI: m, kg, s, rad). El programa comercial escribe tiempos como `min:seg`.
- Si una ficha contradice una especificación de `docs/<idioma>/specs/`, manda la especificación y se pregunta.

## Mapa de archivos

| Archivo                                              | Contenido                                                     |
| ---------------------------------------------------- | ------------------------------------------------------------- |
| 01 Modelo, simulación y animación.md                 | Cómo se organiza un modelo, reloj, 2D/3D, clases y herencia   |
| 02 Conexiones y flujo de material.md                 | Conectores, interfaces, numeración de sucesores, estrategias  |
| 03 MU Tipos y atributos.md                           | Unidades móviles: pieza, contenedor, transportador            |
| 04 Generador y sumidero.md                           | Source y Drain                                                |
| 05 Estación, buffer y procesos.md                    | Station, Buffer, ensamblado y objetos de proceso              |
| 06 Transportadores de rodillos y cadenas.md          | Conveyor: longitud, acumulación, sensores                     |
| 07 Vías, transportadores móviles y giros.md          | Tracks, Transporter, mesas giratorias y de transferencia      |
| 08 Objetos físicos.md                                | Cuerpos rígidos, rodillos, topes, guías, pallet (modo 3D)     |
| 09 Sensores y controles.md                           | Sensores, controles de entrada/salida, métodos                |
| 10 Tiempos, fallos y preparación.md                  | Distribuciones, averías, set-up, turnos                       |
| 11 Estadísticas y resultados.md                      | Estados de recurso, Chart, Sankey, informe, Gantt             |
| 12 Datos e información.md                            | Tabla de datos, variable, método, lenguaje de script          |
| 13 Objetos obsoletos o prescindibles.md              | Qué no copiar y por qué                                       |
| 14 Mapa hacia IntraCraft.md                          | Qué adoptar, qué falta en el MVP y propuestas                 |
| 15 Glosario.md                                       | Equivalencias de términos                                     |
