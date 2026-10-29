# 11 Estadísticas y resultados

## Estados de un recurso [PDF]

Cada objeto de flujo clasifica su tiempo en estados. En el tutorial se citan (nombres traducidos):

| Estado         | Significado                                                                  | Fuente |
| -------------- | ---------------------------------------------------------------------------- | ------ |
| Trabajando     | Procesa una MU                                                               | [PDF]  |
| Esperando      | Vacío, sin MU que procesar (falta de material)                               | [PDF]  |
| Bloqueado      | Terminó pero el sucesor no acepta (se colorea en amarillo)                   | [PDF]  |
| En preparación | Set-up                                                                       | [PDF]  |
| En fallo       | Averiado                                                                     | [PDF]  |
| En pausa       | Parada planificada                                                           | [PDF]  |

Ejemplo del tutorial [PDF]: con los valores de fábrica la estación principal trabaja 100 % del tiempo y las dos siguientes 50 % cada una; al subir el tiempo de la primera estación, baja su porcentaje de trabajo y el robot pasa a esperar.

Las estadísticas se leen en la pestaña **Statistics** de cada objeto o en gráficos.

## Objetos de resultados [PDF]

| Objeto          | Muestra                                                                                    |
| --------------- | ------------------------------------------------------------------------------------------ |
| Chart           | Gráficos: histograma de ocupación, barras de estadísticas de recurso, valores de una tabla |
| SankeyDiagram   | Flujos de piezas con flechas de anchura proporcional (qué ruta toma más tráfico)           |
| HtmlReport      | Información del modelo + estadísticas de estados y del sumidero                            |
| GanttChart      | Qué hace cada recurso en el tiempo (piezas en recursos)                                    |
| Display         | Muestra un valor de una variable en el modelo                                              |

Los **Chart** se crean arrastrando un objeto (o varios) sobre el icono del Chart y eligiendo el tipo: *Ocupación* o *Estadística de recurso*. [PDF]

## Qué medir en IntraCraft [C]

| Magnitud                 | Dónde se mide              | Cálculo                                           |
| ------------------------ | -------------------------- | ------------------------------------------------- |
| Rendimiento (pallets/h)  | Sumidero o sensor          | Nº de pallets / tiempo                            |
| Tiempo de tránsito       | Generador → sumidero       | Salida − entrada                                  |
| Utilización de zona      | Transportador / zona       | % de tiempo moviendo pallet                       |
| Bloqueo                  | Cada objeto                | % de tiempo con pallet que no puede avanzar       |
| Ocupación de acumulación | Zona de buffer             | Histograma de nº de pallets                       |
| Cuello de botella        | Comparar utilizaciones     | Mayor utilización sostenida                       |
| Ciclo de mesa            | Mesa giratoria/transfer.   | Duración entre inicio y fin de cada ciclo         |
| Energía de impacto       | Tope (T5)                  | ½·m·v² = 32 J con 1025 kg a 0,25 m/s              |

## Reglas de buena práctica [PDF + C]

- Con y sin animación, **los resultados deben ser idénticos**. [PDF]
- Reiniciar la simulación debe **limpiar** las tablas de resultados (método `reset`). [PDF]
- Duración de las pruebas: fijar tiempo final (p. ej. 24 h) para comparar. [PDF]
- Las estadísticas no se guardan dentro del archivo de proyecto (regla de IntraCraft). [C]
- Ejecutar varias **observaciones** con semillas distintas y dar media e intervalo de confianza. [C]

## Experimentos [C]

La referencia comercial incluye un gestor de experimentos y optimización por algoritmos genéticos. Fuera del MVP; el equivalente mínimo es una ejecución en lote por CLI (`pnpm run scenario`) con variaciones de un parámetro.
