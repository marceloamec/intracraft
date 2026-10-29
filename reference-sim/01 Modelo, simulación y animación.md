# 01 Modelo, simulación y animación

## Idea general [PDF]

Simulación y animación **corren a la vez** pero son componentes separados:

- **Simulación:** calcula el flujo de material y recoge estadísticas en los objetos.
- **Animación:** solo dibuja lo que la simulación decidió. Se puede apagar para ir más rápido.

Resultado clave [PDF]: con y sin animación los resultados son **idénticos** (misma estadística en el sumidero). Es un criterio de aceptación útil para IntraCraft: el modo rápido y el modo visual no pueden diferir.

Objetivo declarado de la simulación de flujo de material: hallar el óptimo de capacidades, utilización de recursos y tamaño de buffers; detectar cuellos de botella; dimensionar la manutención.

## Contenedor de modelo: Frame [PDF]

Un **Frame** es el contenedor donde se construye el modelo. Contiene su propio controlador de eventos.

| Tipo de Frame | Qué muestra hacia fuera                                  |
| ------------- | -------------------------------------------------------- |
| Model Frame   | Un icono; el contenido solo se ve al abrirlo             |
| Glass Box     | Su contenido, sin icono ni rótulo propio                 |

- Un Frame se puede **insertar dentro de otro** (jerarquía). Los Frames hijos se comportan como un objeto más del flujo.
- Un sub-Frame con **un solo objeto** de flujo puede conectarse directamente, sin interfaces; solo funciona **un nivel** de profundidad. [PDF]
- Los objetos de información (gráficos, tablas) no se muestran dentro de un sub-Frame por defecto, salvo que se desmarque «excluir de mostrar contenido». [PDF]

## Clases y herencia [PDF]

- Existe una **biblioteca de clases** (Class Library) con carpetas, entre ellas las de unidades móviles.
- Cada objeto del modelo es una **instancia** de una clase. Editar la clase **propaga** el cambio a todas las instancias (ejemplo: borrar Source/Drain en la clase Test los borra en Test1 y Test2).
- Se puede derivar una instancia con Ctrl+arrastrar.

Traslado a IntraCraft: es exactamente el modelo de «definición JSON + instancias en el layout». Falta decidir si habrá **subesquemas** (grupos reutilizables de layout). [C]

## Controlador de eventos [PDF]

Objeto único por modelo. Controla: iniciar/parar, **reiniciar**, velocidad (más rápido/lento), tiempo final (p. ej. 24 h), tiempo real activado/desactivado, y avance rápido sin animación.

Se recomienda activar «tiempo real» en modelos 3D porque un modelo solo-3D no calcula los eventos de animación 2D. [PDF]

Atributos habituales del controlador: [C]

| Atributo            | Tipo     | Unidad | Nota                                                |
| ------------------- | -------- | ------ | --------------------------------------------------- |
| Tiempo final        | tiempo   | s      | 0 = sin final                                       |
| Tiempo de inicio    | fecha    | —      | Reloj de calendario del modelo                      |
| Factor de velocidad | real     | ×      | Relación tiempo simulado / tiempo real              |
| Tiempo real         | booleano | —      | Sincroniza con reloj pared                          |
| Semilla             | entero   | —      | Generador de números aleatorios; en IntraCraft fija |
| Nº de observaciones | entero   | —      | Repeticiones para experimentos                      |

## 2D y 3D [PDF]

- Se recomienda modelar en 3D; el 2D (vista cenital, «vista de planificación») es ideal si importa solo la estadística.
- Un mismo modelo se ve en ambas vistas; para colocar objetos es más cómodo la vista cenital.
- Propiedades de animación 3D por objeto: posición/rotación/escala (transformación), apariencia de MU, apariencia de objetos longitudinales (transportadores), material (color), animación de MU, autoanimación, brazo robot, poses y articulaciones.
- Los estados pueden colorearse (p. ej. pieza amarilla = bloqueada). [PDF]

Traslado a IntraCraft: el MVP ya separa DES 2D y física 3D con **un solo layout**. La diferencia es que aquí el 3D es solo animación y en IntraCraft es simulación física real: el criterio de coherencia (±50 ms en eventos de sensor) es más exigente. [C]

## Pestañas de propiedades de simulación [PDF]

Cada objeto de flujo ofrece (nombres traducidos): **Atributos, Tiempos, Preparación (set-up), Fallos, Controles, Salida (estrategia), Estadísticas**. Esa estructura de pestañas es una buena plantilla para la UI generada desde el esquema de parámetros (campo `grupo`).

Los valores se pueden cambiar de tres formas: diálogo del objeto, ventana de atributos y métodos, o asignación en el lenguaje de script. [PDF]
