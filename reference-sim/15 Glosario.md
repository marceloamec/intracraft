# 15 Glosario

| Término de la referencia    | Español usado aquí                 | Notas                                                   |
| --------------------------- | ---------------------------------- | ------------------------------------------------------- |
| Material flow object        | Objeto de flujo de material        | Source, Drain, Station, Buffer, Conveyor…               |
| Information flow object     | Objeto de información              | Tabla, variable, método                                 |
| User interface object       | Objeto de interfaz                 | Chart, Sankey, informe, Display                         |
| MU (mobile unit)            | Unidad móvil                       | Pieza, contenedor, vehículo                             |
| Part                        | Pieza                              | Sin carga encima                                        |
| Container                   | Contenedor                         | Lleva otras MU. En IntraCraft: pallet                   |
| Transporter                 | Transportador móvil / vehículo     | AGV, carretilla                                         |
| Source                      | Generador                          | Crea MU                                                 |
| Drain                       | Sumidero                           | Retira MU                                               |
| Station                     | Estación                           | Pasa tiempo de proceso                                  |
| Buffer                      | Almacén de espera                  | FIFO por defecto                                        |
| Conveyor                    | Transportador                      | Movimiento lineal                                       |
| Track                       | Vía                                | Camino de vehículos                                     |
| TurnTable                   | Mesa giratoria                     | Rota la MU                                              |
| Connector                   | Conector                           | Define el flujo                                         |
| Interface                   | Interfaz                           | Entrada/salida de un Frame                              |
| Frame                       | Marco / contenedor de modelo       | Se puede anidar                                         |
| Class Library               | Biblioteca de clases               | Contiene MU y objetos reutilizables                     |
| Instance / derived          | Instancia / derivada               | Hereda cambios de su clase                              |
| EventController             | Controlador de eventos             | Reloj, inicio/parada, velocidad                         |
| Exit strategy               | Estrategia de salida               | Reparto entre sucesores                                 |
| Blocking                    | Bloqueante                         | Espera al sucesor elegido                               |
| Entrance / exit control     | Control de entrada / salida        | Método disparado por el evento                          |
| Sensor                      | Sensor                             | Posición + control                                      |
| Set-up                      | Preparación                        | Cambio de tipo                                          |
| Failure                     | Fallo / avería                     | MTBF, MTTR                                              |
| Throughput                  | Rendimiento                        | Pallets por hora                                        |
| Utilization                 | Utilización                        | % en trabajo                                            |
| Working / waiting / blocked | Trabajando / esperando / bloqueado | Estados estándar                                        |
| Chart                       | Gráfico                            | Histograma, barras                                      |
| SankeyDiagram               | Diagrama de Sankey                 | Flujos con anchura proporcional                         |
| HtmlReport                  | Informe HTML                       | Resumen de modelo y estadísticas                        |
| GanttChart                  | Diagrama de Gantt                  | Ocupación en el tiempo                                  |
| DataTable                   | Tabla de datos                     | Matriz tipada                                           |
| Variable                    | Variable                           | Valor con inicial                                       |
| Method                      | Método                             | Código de script                                        |
| SimTalk                     | Lenguaje de script                 | Propietario; no se replica                              |
| DES                         | Simulación de eventos discretos    | Modo 2D de IntraCraft                                   |
| Planning view               | Vista de planificación             | Cenital 2D                                              |
