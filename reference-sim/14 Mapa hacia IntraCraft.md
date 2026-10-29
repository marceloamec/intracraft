# 14 Mapa hacia IntraCraft

Cómo se traducen los conceptos de la referencia al MVP. Los nombres de la izquierda son genéricos; los de IntraCraft se fijan en las especificaciones (`docs/<idioma>/specs/`). Ante cualquier contradicción, mandan las especificaciones.

## Equivalencias

| Concepto de referencia        | IntraCraft                                                      | Estado                             |
| ----------------------------- | --------------------------------------------------------------- | ---------------------------------- |
| Source                        | Generador de pallets                                            | En MVP                             |
| Drain                         | Sumidero de pallets                                             | En MVP                             |
| Conveyor (rodillos)           | Mesa de rodillos                                                | En MVP                             |
| Conveyor (cadenas)            | Mesa de cadenas                                                 | En MVP                             |
| TurnTable                     | Giratoria de cadenas / de rodillos                              | En MVP                             |
| Estación de transferencia     | Mesa de transferencia de rodillos a cadenas                     | En MVP                             |
| Sensor de conveyor            | Sensor fotoeléctrico                                            | En MVP                             |
| Parada (bloqueo)              | Tope mecánico                                                   | En MVP                             |
| Guía                          | Guía lateral                                                    | En MVP                             |
| Controles / métodos           | Controlador por reglas Si/Entonces                              | En MVP                             |
| Container                     | Pallet                                                          | En MVP                             |
| Part                          | Carga del pallet                                                | En MVP (como bloque de carga)      |
| Station genérica              | —                                                               | No necesaria                       |
| Buffer                        | Zona de acumulación del transportador                           | Implícito                          |
| Transporter, Track            | AGV / carretilla                                                | Fuera de MVP                       |
| Fallos, turnos, set-up        | —                                                               | Fuera de MVP                       |
| Chart, Sankey, informe        | Panel de resultados                                             | Pendiente de especificar           |
| Frame jerárquico              | Subesquemas                                                     | Pendiente de decidir               |
| Clase + instancia             | Definición JSON + instancia en layout                           | Alineado                           |

## Qué adoptar de la referencia

1. **Pestañas por tema** en los diálogos (Atributos, Tiempos, Fallos, Controles, Salida, Estadísticas): encaja con el campo `grupo` de cada parámetro.
2. **Estados de utilización** estándar (trabajando, esperando, bloqueado, en fallo, en pausa) como base de la estadística.
3. **Coloración por estado** en 3D (bloqueado = amarillo) para depuración visual.
4. **Histograma de ocupación** del buffer/zona como gráfico estándar.
5. **Diagrama de flujos (Sankey)** para ver qué ramas toman más tráfico.
6. **Igualdad de resultados con y sin animación** como prueba automática.
7. **Parámetros mostrados por distribución** al elegirla (UI generada desde el esquema).
8. **Reinicio que limpia resultados** y valores iniciales de variables.

## Qué mejorar frente a la referencia

1. Prioridad de sucesores **explícita**, no implícita en el orden de conexión.
2. Nada de scripts libres: reglas declarativas con plantillas.
3. Semillas por objeto para que añadir un objeto no altere el resto.
4. Física real con criterios medibles (±5 mm, <10 mm de rebote, ±50 ms 2D/3D).
5. Todo parámetro con unidad, límite y descripción (regla del repositorio).

## Pruebas de comparación con la referencia (observables)

Se permite comparar **resultados**, nunca implementación. Sugerencias:

| Escenario                                        | Magnitud comparada              |
| ------------------------------------------------ | ------------------------------- |
| Generador → transportador → sumidero (3 m)       | Tiempo de tránsito (T1)         |
| Línea saturada con separación 0,15 m             | Rendimiento (T2)                |
| Giro de 90°                                      | Tiempo de ciclo (T3)            |
| Transferencia con subida y bajada de 1 s         | Tiempo de ciclo (T4)            |
| Reparto 60 / 40 entre dos ramas                  | Distribución final de pallets   |
| Tiempo de proceso > intervalo de llegada         | Bloqueo aguas arriba            |

## Siguientes pasos propuestos (a confirmar)

1. Revisar las fichas [C] que te interesen y decirme cuáles son correctas para tu experiencia con simuladores.
2. Decidir si IntraCraft tendrá subesquemas (jerarquía) y reglas de ruteo *pull*.
3. Convertir las fichas aprobadas en especificaciones de `docs/<idioma>/specs/` en los tres idiomas (con confirmación previa, según AGENTS.md).
