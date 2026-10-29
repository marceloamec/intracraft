# 02 Conexiones y flujo de material

## Conector [PDF]

Un **Connector** une dos objetos de flujo y define **por dónde pueden moverse las piezas**: las piezas siguen la secuencia en que se conectan los objetos.

- Dirección: del origen al destino.
- Modo conectar: con Ctrl pulsado se encadenan varios objetos seguidos.
- Insertar un objeto *sobre* un conector existente lo intercala sin rehacer conexiones.
- Al reemplazar un objeto arrastrando uno nuevo sobre el viejo se **conservan** las conexiones (Alt+arrastrar). [PDF]

Traslado a IntraCraft: equivale a la conexión entre **puertos** del `core-model`. Un puerto tiene orientación y posición; un conector del programa comercial es más abstracto (no tiene geometría). [C]

## Numeración de predecesores y sucesores [PDF]

- Cada objeto numera sus sucesores según el **orden en que se conectaron**.
- Regla: el sucesor 1 se atiende **antes** que el 2. Si el reparto sale al revés, se borran los conectores y se reconectan en el orden correcto.
- En un objeto con dos predecesores (p. ej. ensamblado) el orden también importa: se indica «MU principal desde predecesor n».

Consecuencia de diseño: el orden de conexión es **estado oculto** que cambia el resultado. IntraCraft debería hacer esa prioridad **explícita y editable** en la UI (lista ordenable), no implícita. [C]

## Interfaz (Interface) [PDF]

Objeto que marca **dónde entra y sale el material** en un Frame. Se coloca en lugar del Source/Drain al convertir un modelo en subestructura.

- Un Frame con un solo objeto de flujo no necesita interfaces (un nivel). [PDF]
- Atributos [C]: nombre, dirección (entrada/salida), capacidad 0 (no retiene piezas), sin tiempo propio.

## Estrategias de salida [PDF]

Pestaña **Exit Strategy** de la estación. Decide a qué sucesor va la pieza que acaba de procesarse.

| Estrategia            | Efecto                                                                | Fuente |
| --------------------- | --------------------------------------------------------------------- | ------ |
| Por defecto (cíclica) | Alterna entre sucesores disponibles                                   | [PDF]  |
| Porcentaje            | Reparto fijo por lista (ej. 60 % al sucesor 1 y 40 % al 2)            | [PDF]  |
| Primero disponible    | El primer sucesor con sitio                                           | [C]    |
| Aleatorio             | Con probabilidades                                                    | [C]    |
| Por atributo / tipo   | Según el MU (p. ej. ruteo por destino)                                | [C]    |
| Control (script)      | Un método decide, con `mover(destino)`                                | [PDF]  |

Opción **Blocking** (bloqueante) [PDF]: si está marcada, la pieza espera al sucesor elegido aunque otro esté libre; si no, puede ir a otro. Esta diferencia es clave en un layout con ramas: cambia los bloqueos.

## Empuje y tirón [C]

- **Push:** el predecesor empuja la pieza cuando termina.
- **Pull:** el sucesor la pide cuando está libre.

El programa comercial lo trata con las condiciones de entrada y salida y con «bloqueo». IntraCraft ya usa topes, sensores y reglas Si/Entonces; conviene definir en la especificación del ruteo si se admite *pull* o solo *push* con topes.

## Reglas de ruteo útiles para pallets [C]

1. Ruteo por **destino** (atributo del pallet).
2. Ruteo por **estado de ocupación** del sucesor (evitar bloquear).
3. Ruteo por **porcentaje** para pruebas de carga reproducibles (con la semilla fija).
4. Ruteo por **prioridad** explícita entre ramas.
