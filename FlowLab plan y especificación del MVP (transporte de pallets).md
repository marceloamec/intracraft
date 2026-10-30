# IntraCraft: simulador híbrido 2D/3D de transporte de pallets

*Plan de desarrollo y especificación del MVP · Borrador v1 · 28 de octubre de 2026 · “IntraCraft” es el nombre elegido; pendiente de comprobar marcas y dominios.*

## 0. Resumen ejecutivo

**Qué se construye.** Una aplicación para diseñar y simular sistemas de transporte de pallets con *un solo layout* y *dos modos* de simulación:

- **Modo 2D discreto (DES):** basado en eventos; corre cientos o miles de veces más rápido que el tiempo real. Responde preguntas de rendimiento, acumulación, bloqueos y utilización.
- **Modo 3D con física (tiempo real):** simula el rozamiento de rodillos y cadenas, las colisiones con topes y guías laterales, la alineación de los pallets y los sensores con geometría real.

**Qué incluye el MVP.** Cinco transportadores para pallets (mesa de rodillos, mesa de cadenas, giratoria de cadenas, giratoria de rodillos y mesa de transferencia de rodillos a cadenas), sensores fotoeléctricos, topes mecánicos, guías laterales, generador y sumidero de pallets, un controlador sencillo por reglas y una **librería** de componentes parametrizables que alimenta al editor y a los dos motores de simulación.

**Dónde mirar.** Parámetros de cada transportador: sección 8. Sensores, topes y guías: sección 9. Plan por fases: sección 14. Preguntas que hay que confirmar: sección 18.

**Decisiones clave de este plan**

| # | Decisión | Por qué |
| --- | --- | --- |
| 1 | Un layout, dos fidelidades | El layout se guarda en 3D real; el modo 2D es la misma planta con lógica simplificada |
| 2 | Librería dirigida por datos | Cada componente se describe en JSON (parámetros, puertos, señales) y su comportamiento se registra en código; la interfaz de edición se genera sola |
| 3 | Mismo controlador en ambos modos | Las reglas Si/Entonces leen sensores y mandan actuadores igual en 2D y en 3D |
| 4 | TypeScript en todo, web local primero | Un solo lenguaje y resultado visual inmediato; la versión de escritorio (Tauri) viene después |
| 5 | Spike de física antes de comprometerse | La ‘superficie móvil’ de las cadenas es el mayor riesgo técnico (sección 12) |
| 6 | Un transportador a la vez, de punta a punta | Cada tipo se entrega completo: parámetros, modo 2D, modo 3D y pruebas |
| 7 | Unidades SI y eje Y hacia arriba | Coincide con Three.js y glTF |

**Siguiente acción.** Fases 0 y 1: repositorio, `AGENTS.md`, especificaciones y spikes técnicos de física y visor (sección 14).

## 1. Principios de diseño y diferenciadores

1. **El layout es la fuente de verdad.** Todo lo demás (2D, 3D, estadísticas) se deriva de él.
2. **Misma lógica de control, distinta planta.** El controlador no sabe si habla con un modelo abstracto o con física.
3. **Todo parámetro lleva unidad, rango, valor por defecto y descripción.** La interfaz de edición se genera desde ese esquema.
4. **Determinismo.** Mismos datos y misma semilla dan los mismos resultados; es imprescindible para depurar y comparar.
5. **La geometría real del pallet importa.** Los huecos entre tablas y tacos afectan a sensores y apoyos; el pallet no es una caja lisa.
6. **Validar antes de ampliar.** Cada objeto se contrasta con cálculos analíticos y con modelos conocidos de Plant Simulation.
7. **Pasos pequeños, siempre con prueba.**

**Dónde puede ser mejor** (hipótesis que hay que demostrar, no promesas):

- Frente a Plant Simulation: física 3D real, sensores y topes con geometría, formato abierto y versionable, ejecución sin interfaz y calibración entre 2D y 3D.
- Frente a realvirtual.io: simulación discreta rápida para rendimiento, sin depender de Unity, ejecución en navegador y librería abierta.

## 2. Alcance del MVP

### 2.1 Dentro del MVP

- Cinco transportadores de pallets (sección 8) con parámetros completos.
- Sensores fotoeléctricos (difusa, reflex, barrera), topes mecánicos (fijo y retráctil) y guías laterales con entrada en embudo.
- Generador (Source) y sumidero (Sink) de pallets; tipos EUR, 1200×1000 y GMA, más uno personalizado.
- Controlador por reglas Si/Entonces con señales booleanas.
- Modo 2D discreto y modo 3D con física sobre el mismo layout; el cambio de modo reinicia la simulación.
- Editor: colocar, conectar, parametrizar, guardar y abrir; reproducir, pausar y acelerar.
- Estadísticas básicas por objeto, exportación a CSV/JSON y ejecución sin interfaz (CLI).
- Librería integrada con presets y componentes propios (duplicar y editar).
- Ampliación del 30-10-2026 (ADR-0012): mesa elevadora manual para transpaleta, mesa de tijera, elevador de dos niveles y línea de rodillos por gravedad con separador (especificaciones S12–S15 en `docs/en/specs/`).

### 2.2 Fuera del MVP (post-MVP)

Curvas, elevadores de tres o más niveles, lanzaderas y desviadores; cajas y cubetas; fallos (MTBF/MTTR) y energía; scripting de usuario; traspaso de estado entre modos en mitad de la simulación; calibración automática 2D↔3D; experimentos y optimización; OPC UA/PLC; importación de CAD; trabajo en equipo.

### 2.3 Definición de “hecho” del MVP

1. Los cinco transportadores se pueden colocar, conectar y parametrizar desde la librería.
2. Un layout de referencia con los cinco tipos corre en ambos modos sin intervención.
3. En modo 2D los resultados coinciden con los cálculos analíticos de la sección 15 con error menor al 1 %.
4. En modo 3D un pallet se detiene contra un tope con su borde frontal a ±5 mm de la posición teórica y con rebote menor de 10 mm.
5. Los tiempos de tránsito y los eventos de sensores de ambos modos difieren menos del 5 % y de ±50 ms en los escenarios de referencia.
6. El mismo layout con la misma semilla produce resultados idénticos en dos ejecuciones.
7. El modo 3D mantiene 60 FPS con 20 transportadores y 10 pallets en un portátil actual.
8. Se reproducen y se comparan tres modelos conocidos de Plant Simulation.

## 3. Arquitectura

### 3.1 Capas

| Capa | Responsabilidad | Tecnología propuesta |
| --- | --- | --- |
| Modelo (`core-model`) | Layout, objetos, puertos, conexiones, unidades y validación; sin dependencias de interfaz | TypeScript + Zod (esquemas) |
| Librería (`library`) | Definiciones JSON y comportamientos registrados | JSON + TypeScript |
| Geometría (`geometry`) | Genera mallas 3D y símbolos 2D desde los parámetros | Three.js |
| Motor discreto (`sim-des`) | Cola de eventos, cinemática analítica y estadísticas | TypeScript puro |
| Motor físico (`sim-physics`) | Mundo físico, rodillos, cadenas, topes y sensores por raycast | Rapier o Jolt (WASM), a decidir en el spike |
| Controlador (`control`) | Señales y reglas Si/Entonces | TypeScript puro |
| Editor (`apps/editor`) | Interfaz, visor 2D/3D, propiedades y panel de librería | React + react-three-fiber + Zustand |
| CLI (`apps/cli`) | Ejecutar escenarios sin interfaz | Node.js |

### 3.2 Estructura del repositorio

```
intracraft/
├─ AGENTS.md              (CLAUDE.md apunta aquí)
├─ docs/
│  ├─ specs/              (una especificación por objeto)
│  └─ decisions/          (registro de decisiones)
├─ packages/
│  ├─ core-model/  library/  geometry/
│  ├─ sim-des/  sim-physics/  control/
│  └─ scenarios/          (modelos de prueba y resultados esperados)
├─ apps/
│  ├─ editor/             (web local; Tauri más adelante)
│  └─ cli/
└─ test_data/
```

Herramientas: pnpm (espacios de trabajo), Vite, Vitest, TypeScript en modo estricto, ESLint y Prettier, GitHub Actions.

### 3.3 Ejecución

- La simulación (2D o 3D) corre en un **Web Worker** para no bloquear la interfaz; el editor recibe instantáneas de estado (posiciones, señales y estadísticas).
- Modo 3D: paso fijo de física (propuesta: 1/120 s) con acumulador; el visor interpola entre pasos.
- Modo 2D: el reloj salta de evento en evento; el visor anima por interpolación analítica.

### 3.4 Convenciones

- **Unidades internas SI** (m, kg, s, rad); la interfaz muestra mm y grados.
- **Ejes:** Y hacia arriba, mano derecha; la vista 2D es la vista cenital (mirando hacia −Y).
- **Tiempo:** enteros en microsegundos (evita errores de coma flotante); la cola de eventos se ordena por (tiempo, prioridad, secuencia).
- **Aleatoriedad:** generador con semilla (por ejemplo PCG o xoshiro); nunca `Math.random()` dentro de los motores.
- Todo archivo de proyecto lleva `schemaVersion` y migraciones explícitas.

## 4. Modelo de datos

### 4.1 Entidades

| Entidad | Contenido |
| --- | --- |
| Proyecto | Metadatos, unidades, librerías usadas y `schemaVersion` |
| Layout | Instancias, conexiones, controladores y escenarios |
| Instancia | `id`, tipo (id de librería + versión), parámetros, posición (x, y, z) y giro (yaw) |
| Puerto | Punto de entrega o recepción de pallets: posición local, dirección, altura y tolerancias |
| Conexión | Puerto de salida de A hacia puerto de entrada de B, con validación de alineación |
| Señal | Booleana (MVP) con nombre, origen y destino |
| Pallet | Tipo, masa, carga, estado y ubicación (objeto y coordenada) |
| Escenario | Layout + generadores + semilla + duración + métricas esperadas |

### 4.2 Puertos y conexiones

- Cada puerto declara **altura de superficie**, **ancho útil** y **tolerancias** de alineación (por defecto ±10 mm de altura y ±20 mm laterales).
- El editor encaja puertos cercanos (snap) y avisa si altura, ancho u orientación no coinciden.
- En las giratorias los puertos se definen en el marco de la plataforma y solo cuentan como conectados cuando el ángulo coincide con una estación.
- La separación entre transportadores es un parámetro de la conexión (por defecto 20 mm); en 3D es un hueco real que el pallet debe salvar.

### 4.3 Señales y controlador (MVP)

- Señales booleanas con nombre: de sensores (entradas) y de actuadores (salidas: marcha, tope arriba, giro a estación N, elevar cadenas…).
- **Reglas Si/Entonces** con condiciones booleanas, retardos (ms) y flancos; se evalúan por eventos en 2D y en cada ciclo de control en 3D (por defecto 10 ms).
- Plantillas incluidas: acumulación sin presión (ZPA), parada en tope con sensor, giratoria que espera pallet centrado y transferencia con secuencia completa.
- El scripting de usuario (por ejemplo QuickJS en sandbox) queda para después del MVP.

### 4.4 Archivo de proyecto

JSON legible y versionable con git (`.intracraft.json`). Los resultados de simulación no se guardan dentro del proyecto; se exportan aparte.

## 5. Los dos modos

### 5.1 Qué hace cada modo

| Elemento | Modo 2D discreto | Modo 3D con física |
| --- | --- | --- |
| Pallet | Intervalo \[cola, frente\] sobre una pista, con perfil de ocupación por alturas | Cuerpo rígido compuesto (tablas inferiores + bloque de carga) |
| Mesa de rodillos | Cinemática trapezoidal (aceleración, crucero, deceleración) y zonas de acumulación | Rodillos cinemáticos giratorios; el rozamiento mueve el pallet |
| Mesa de cadenas | Velocidad de cadena; el pallet patina si está retenido | Tiras de rodillos pequeños (o superficie móvil) por cada cadena |
| Giratorias | Perfil angular trapezoidal; el pallet viaja con la plataforma | Plataforma cinemática con velocidad angular; el pallet se sostiene por fricción |
| Transferencia | Máquina de estados con tiempos (subir, mover, bajar) | Cadenas cinemáticas que suben y se desplazan; el pallet se levanta por contacto |
| Sensor | Haz (segmento) contra el perfil de ocupación del pallet | Raycast o shape-cast contra los colisionadores reales |
| Tope | Posición límite del frente del pallet; tiempos de subida y bajada | Cuerpo cinemático con colisionador; admite rebote y desalineación |
| Guía lateral | Solo validación de holgura | Colisionador estático con rozamiento |
| Controlador | Reglas evaluadas por eventos | Las mismas reglas evaluadas cada ciclo de control |

### 5.2 Cambio de modo

- **MVP:** el cambio reinicia la simulación; layout y parámetros son los mismos.
- **Después:** traspaso de estado a mitad de simulación (convertir posiciones y velocidades entre mundos); exige que los pallets estén en una pose estable y bien definida.

### 5.3 Calibración 2D↔3D (post-MVP)

Ejecutar lotes de pruebas en 3D (variando masa, rozamiento, velocidad de entrada y desalineación) y extraer tiempos de tránsito, probabilidad de atasco y desalineación final. Esos valores alimentan perfiles de calibración para el modo 2D, que así gana realismo sin perder velocidad. Es el diferenciador principal del proyecto.

## 6. La librería

### 6.1 Concepto

- **Definición:** descripción de un tipo de componente (parámetros, puertos, señales, geometría y comportamientos). Vive en la librería.
- **Instancia:** uso concreto de una definición en un layout, con sus valores de parámetros.
- **Preset:** conjunto nombrado de valores para una definición, por ejemplo “Rodillos pesados 1000 kg, 0,25 m/s”.
- **Orígenes:** *integrada* (solo lectura), *de usuario* (editable y compartible) y *del proyecto* (se guarda con el layout).
- Las definiciones llevan versión semántica; un layout fija la versión usada y se migra de forma explícita.

### 6.2 Contenido de la librería del MVP

| Categoría | Componentes |
| --- | --- |
| Transportadores | Mesa de rodillos, mesa de cadenas, giratoria de cadenas, giratoria de rodillos, transferencia rodillos→cadenas |
| Sensores | Fotocélula difusa, reflex (retrorreflectiva), barrera emisor-receptor |
| Mecánica | Tope fijo, tope retráctil, guía lateral (con entrada en embudo) |
| Flujo | Generador de pallets (Source) y sumidero (Sink) |
| Pallets | EUR, 1200×1000, GMA y personalizado |
| Control | Controlador por reglas y plantillas de control |
| Materiales | Tabla de pares de fricción |

### 6.3 Qué contiene una definición

`id`, `version`, `categoria`, `nombre` (con traducciones), `descripcion`, `icono2D`, `parametros` (ver 6.4), `grupos` de parámetros, `puertos`, `señales` de entrada y salida, `estadisticas` que publica, `geometria` (id del generador y enlace de parámetros), `comportamientos` (ids del comportamiento 2D y del 3D), `reglasValidacion` y `presets`.

### 6.4 Qué contiene un parámetro

`clave`, `etiqueta`, `tipo` (número, entero, booleano, lista, enumerado o tabla), `unidad`, `defecto`, `min`, `max`, `paso`, `grupo`, `descripcion`, `avanzado` (oculto por defecto), `afecta` (2D, 3D, ambos o solo geometría), `derivado` (calculado, solo lectura) y `dependeDe` (visibilidad condicional).

### 6.5 Datos y código

El JSON describe *qué es* el componente; el código TypeScript implementa *cómo se comporta* y se registra por id en un registro de comportamientos. Así la interfaz y las validaciones salen de los datos, y la lógica vive en módulos con pruebas.

### 6.6 Geometría paramétrica

La geometría se genera desde los parámetros (bastidor, patas, rodillos, cadenas y guías) usando mallas instanciadas (`InstancedMesh`) para los rodillos. Se puede sustituir por un modelo glTF/GLB propio manteniendo dimensiones y puertos.

### 6.7 Validaciones automáticas (ejemplos)

- Al menos **3 rodillos** bajo el lado del pallet que apoya (paso ≤ lado / 3).
- Ancho útil ≥ ancho del pallet + 2 × holgura de guía.
- En transferencias: hueco entre rodillos (paso − diámetro) ≥ ancho de cadena + 10 mm.
- Radio de barrido de una giratoria + holgura menor que la distancia libre a los elementos vecinos.
- Velocidad de impacto contra tope dentro del límite configurado.
- Alturas de superficie de los puertos conectados dentro de tolerancia.

### 6.8 Panel de librería en el editor

Árbol por categorías, buscador, vista previa 2D/3D, arrastrar y soltar, presets, “duplicar como componente propio” e importar/exportar `.json`.

### 6.9 Datos de referencia

**Pallets (valores de partida editables)**

| Tipo | Dimensiones L×A (mm) | Altura (mm) | Tara (kg) | Notas |
| --- | --- | --- | --- | --- |
| EUR / EPAL | 1200 × 800 | 144 | 25 | 3 tablas inferiores de 1200 mm |
| 1200 × 1000 | 1200 × 1000 | ≈ 150 | 25–30 | Muy extendido (ISO 1, PBR) |
| GMA | 1219 × 1016 | ≈ 140 | 20–30 | 48 × 40 pulgadas |
| Personalizado | libre | libre | libre | Incluye perfil de huecos |

Cada tipo incluye un **perfil de ocupación**: bandas de altura (por ejemplo 0–22 mm tablas inferiores, 22–100 mm tacos y huecos, 100–144 mm tablas superiores) con la posición de los huecos a lo largo del avance. Lo usan los sensores en 2D y sirve para generar los colisionadores en 3D.

**Fricción (pares iniciales, a calibrar)**

| Par de materiales | μ inicial | Rango habitual |
| --- | --- | --- |
| Madera – acero (rodillo) | 0,35 | 0,25–0,50 |
| Madera – cadena de acero | 0,30 | 0,20–0,45 |
| Plástico – acero | 0,25 | 0,15–0,40 |
| Madera – guía de acero | 0,30 | 0,20–0,45 |

Estos valores son orientativos; hay que contrastarlos con hojas de datos o ensayos antes de usar los resultados para decidir.

## 7. Parámetros comunes de todos los transportadores

Los valores por defecto son puntos de partida orientativos para pallets de hasta 1000 kg; deben ajustarse a la hoja de datos real del equipo.

| Parámetro (clave) | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `nombre` | texto | auto | — | Nombre visible; el `id` interno se genera solo |
| `posicion` (x, y, z) | m | 0, 0, 0 | — | Origen del componente (centro del inicio) |
| `giro` (yaw) | ° | 0 | −180 a 180 | Rotación alrededor del eje vertical |
| `alturaSuperficie` | mm | 600 | 200–1500 | Altura del plano de apoyo del pallet sobre el suelo |
| `longitud` | mm | 3000 | 500–12000 | Longitud en el sentido de avance |
| `anchoUtil` | mm | 1000 | 500–1800 | Ancho libre entre bastidores o guías |
| `ladoParaleloAvance` | enum | largo | largo, ancho | Lado del pallet paralelo al avance (convención habitual: rodillos con el lado largo en el avance; cadenas con el lado corto) |
| `velocidadNominal` | m/s | 0,25 | 0,05–0,60 | Velocidad de crucero |
| `velocidadAproximacion` | m/s | 0,10 | 0,02–0,30 | Velocidad lenta antes de un tope o parada |
| `aceleracion` | m/s² | 0,5 | 0,05–2,0 | Rampa de arranque |
| `deceleracion` | m/s² | 0,5 | 0,05–2,0 | Rampa de parada |
| `sentido` | enum | adelante | adelante, atrás, reversible | Sentido de marcha |
| `cargaMaxPallet` | kg | 1000 | 100–2000 | Carga máxima por pallet; avisa si se supera |
| `separacionMinima` | mm | 150 | 50–500 | Holgura mínima entre pallets en acumulación |
| `modoAcumulacion` | enum | según tipo | continuo, ZPA, con presión | Comportamiento con pallets detenidos |
| `guias` | grupo | ver 9.3 | — | Guías laterales integradas |
| `habilitado` | bool | sí | — | Desactiva el componente sin borrarlo |
| `color`, `patas` | — | — | — | Apariencia; no afecta a la simulación |
| `fallos` (reservado) | — | desactivado | — | MTBF y MTTR; previsto para después del MVP |

**Derivados (solo lectura):** capacidad en pallets = ⌊longitud / (largo del pallet + separación mínima)⌋; tiempo de tránsito ideal; rendimiento máximo ≈ velocidad / (largo del pallet + separación mínima).

## 8. Los cinco transportadores del MVP

Los parámetros específicos de cada tipo se suman a los comunes (sección 7). Todos tienen puerto de entrada A y de salida B, salvo donde se indique. Las interpretaciones marcadas como *a confirmar* están recogidas en la sección 18.

### 8.0 Señales y estadísticas comunes

- **Entradas (mando):** `marcha`, `reversa`, `habilitar` y, por zona, `zona[i].marcha`.
- **Salidas:** `en_marcha`, `ocupado`, `zona[i].ocupada` y `fallo` (reservado).
- **Estados (2D):** parado, acelerando, en marcha, deteniendo, bloqueado.
- **Estadísticas:** utilización, tiempo bloqueado, tiempo vacío, pallets procesados, rendimiento (pallets/h), tiempo medio de tránsito y ocupación media.

### 8.1 Mesa de rodillos (`transp.rodillos`)

Tramo recto de rodillos que transporta el pallet con las tablas inferiores en el sentido del avance. Es el componente base: los demás reutilizan su grupo de parámetros de rodillos.

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `diametroRodillo` | mm | 76 | 50–114 | Diámetro exterior del rodillo |
| `pasoRodillos` | mm | 133 | 75–250 | Distancia entre ejes; debe dejar al menos 3 rodillos bajo el pallet |
| `materialRodillo` | enum | acero | acero, acero recubierto, PVC | Define el par de fricción con el pallet |
| `muRodillo` | — | 0,35 | 0,1–0,8 | Rozamiento pallet–rodillo (editable sobre el par de materiales) |
| `resistenciaRodadura` | — | 0,02 | 0–0,1 | Pérdida por rodadura; relevante en rodillos libres |
| `accionamiento` | enum | cadena continua | gravedad, cadena continua, por zonas | Tren único movido por cadena, o zonas independientes (por ejemplo con motorrodillos) |
| `longitudZona` | mm | 1400 | 1000–2000 | Solo en accionamiento por zonas; ≥ largo del pallet + holgura |
| `sensorFinDeZona` | bool | sí | — | Crea un sensor virtual por zona para la acumulación ZPA |
| `inerciaRodillo` | kg·m² | auto | — | Para la física; se calcula con masa y diámetro y es editable |

**Derivados:** `numRodillos` = ⌊longitud / paso⌋, `numZonas` y `capacidadPallets`.

- **Modo 2D:** el pallet acelera, cruza y frena con perfil trapezoidal. En acumulación sin presión (ZPA) cada zona arranca solo si la siguiente está libre; en acumulación con presión los pallets se empujan hasta quedar a la separación mínima.
- **Modo 3D:** cada rodillo es un cilindro cinemático con velocidad angular ω = v / (diámetro / 2), agrupado por zona. El pallet avanza por rozamiento con sus tablas inferiores, así que puede patinar si el par de fricción es bajo o si está retenido.
- **Rodillos libres (gravedad):** sin accionamiento; el pallet solo los cruza si lo empuja un tramo motorizado (sin pendiente; la línea de gravedad con pendiente, frenos y separador es un objeto aparte: S15).
- **Validaciones:** regla de 3 rodillos, ancho útil frente al pallet y longitud de zona ≥ largo del pallet + 100 mm.

### 8.2 Mesa de cadenas (`transp.cadenas`)

Tramo recto en el que el pallet apoya sobre varios ramales de cadena sincronizados. Es el transportador típico para el avance transversal (lado corto del pallet en el avance) y tolera la acumulación porque las cadenas patinan bajo el pallet detenido.

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `numCadenas` | — | 3 | 2–6 | Número de ramales de cadena |
| `posicionesCadenas` | mm | auto | — | Posición transversal de cada ramal respecto al eje; editable para alinear con las tablas del pallet |
| `anchoApoyoCadena` | mm | 40 | 20–100 | Ancho de contacto de cada ramal |
| `tipoCadena` | enum | rodillos | simple, rodillos, doble paso | Define rozamiento y geometría de apoyo |
| `pasoCadena` | mm | 38,1 | 19,05–76,2 | Paso; afecta a la geometría y al efecto poligonal en 3D |
| `muCadena` | — | 0,30 | 0,1–0,6 | Rozamiento pallet–cadena |
| `modoAcumulacion` | enum | con deslizamiento | con deslizamiento, por zonas | Con deslizamiento la cadena patina bajo el pallet detenido; por zonas, ramales independientes |
| `longitudZona` | mm | 1400 | 1000–2000 | Solo si es por zonas |
| `tiempoDetencionTope` | s | 0,2 | 0,05–1 | Modo 2D: tiempo que tarda el pallet en detenerse al chocar con un tope |
| `fuerzaMaxArrastre` | N | auto | — | Derivado: μ × peso del pallet; informativo |

- **Modo 2D:** el pallet viaja a la velocidad de la cadena; retenido por un tope o por otro pallet, se detiene y la cadena sigue girando bajo él.
- **Modo 3D:** cada ramal se modela como una tira de rodillos pequeños cinemáticos (por ejemplo Ø 30 mm cada 40 mm) cuya velocidad superficial es la de la cadena, o como superficie móvil si el motor de física lo permite (sección 12).
- **Convención de orientación (a confirmar):** en rodillos el pallet avanza con las tablas inferiores paralelas al avance (lado largo en el avance); en cadenas, con las tablas inferiores perpendiculares a las cadenas (lado corto en el avance). Por eso la transferencia rodillos→cadenas cambia el sentido de avance 90° sin girar el pallet.
- **Validaciones:** al menos 2 cadenas en contacto con las tablas inferiores y ancho útil frente al pallet.

### 8.3 Giratoria de cadenas (`transp.giratoria_cadenas`)

Plataforma giratoria con un tramo de cadenas encima. Recibe un pallet por cadenas, gira a otra estación (normalmente 90° o 180°) y lo entrega a otra línea de cadenas. Cambia la dirección del flujo y su ciclo suele condicionar el rendimiento del cruce.

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `largoPlataforma` | mm | 1400 | 1000–1800 | Dimensión de la plataforma en el sentido de avance |
| `anchoPlataforma` | mm | 1200 | 800–1500 | Dimensión transversal |
| `centroGiro` (dx, dz) | mm | 0, 0 | ±300 | Desplazamiento del eje de giro respecto al centro de la plataforma |
| `modoGiro` | enum | estaciones | estaciones, libre | En *estaciones* solo se detiene en ángulos definidos |
| `estaciones` | ° | 0, 90, 180, 270 | lista | Ángulos de parada; cada una publica un puerto |
| `rangoAngular` | ° | ilimitado | −720 a 720 | Límite mecánico (por ejemplo por cables); si es limitado, el giro usa el camino permitido |
| `caminoMasCorto` | bool | sí | — | Elige el sentido de giro de menor ángulo |
| `velocidadAngularMax` | °/s | 15 | 2–45 | Velocidad angular de crucero |
| `aceleracionAngular` | °/s² | 30 | 5–120 | Rampa de arranque |
| `deceleracionAngular` | °/s² | 30 | 5–120 | Rampa de parada |
| `tiempoAsentamiento` | s | 0,3 | 0–2 | Pausa tras llegar a la estación antes de habilitar la descarga |
| `precisionAngular` | ° | 0,2 | 0,05–2 | Error de posición al detenerse |
| `holguraBarrido` | mm | 50 | 0–300 | Margen entre el círculo de barrido y los elementos vecinos |
| `cargaMax` | kg | 1500 | 200–3000 | Pallet más carga |
| `requierePalletCentrado` | bool | sí | — | Interlock: no gira sin pallet centrado según los sensores |
| `toleranciaCentrado` | mm | ±20 | 5–50 | Desviación admisible del pallet respecto al centro |
| `inerciaPlataforma` | kg·m² | auto | — | Física: se calcula con masa y dimensiones |
| `parMotor` | N·m | 1500 | 100–10000 | Física: par máximo del accionamiento de giro |
| `parFriccionCorona` | N·m | 20 | 0–200 | Física: fricción del rodamiento de giro |
| Grupo de cadenas | — | — | — | `numCadenas`, `posicionesCadenas`, `anchoApoyoCadena`, `muCadena`, velocidad y rampas, como en 8.2 |
| `radioBarrido` (derivado) | mm | — | — | ½ · √(largo² + ancho²) + holgura de barrido |
| `tiempoGiro` (derivado) | s | — | — | Ver fórmula |

**Tiempo de giro.** Para un giro θ con velocidad máxima ω y aceleración α (igual a la deceleración): si ω²/α ≥ θ, el perfil es triangular y t = 2·√(θ/α); si no, es trapezoidal y t = θ/ω + ω/α. Ejemplo: θ = 90°, ω = 15 °/s, α = 30 °/s² → t = 6 + 0,5 = 6,5 s.

**Comprobación de fricción.** Para que el pallet no deslice al girar hace falta μ ≥ √(a\_t² + a\_c²) / g, con a\_t = α·r y a\_c = ω²·r, donde r es el radio de la esquina más alejada del pallet (0,72 m para un EUR centrado). Con los valores por defecto se necesita μ ≈ 0,04, un margen muy holgado que crece con α y con r.

**Secuencia:** esperando → recibiendo (cadenas en marcha) → centrado → girando → asentando → descargando → esperando. **Puertos:** uno por estación, definidos en el marco de la plataforma.

- **Modo 2D:** el pallet viaja con la plataforma; el giro dura lo que da el perfil angular y la descarga se habilita al terminar el asentamiento.
- **Modo 3D:** la plataforma es un cuerpo cinemático con velocidad angular en torno al eje vertical; las tiras de cadena van encima; el pallet es dinámico y se sostiene por fricción.
- **Validaciones:** radio de barrido frente a vecinos, par motor suficiente (par = inercia total × α + fricción de corona) y fricción necesaria frente a la disponible.

### 8.4 Giratoria de rodillos (`transp.giratoria_rodillos`)

Misma plataforma giratoria con un tramo de rodillos encima. Es la opción habitual cuando las líneas de entrada y de salida son de rodillos. Usa el grupo *Plataforma y giro* de 8.3 y el grupo de rodillos de 8.1, más:

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `bloqueoRodillosEnGiro` | bool | sí | — | Rodillos frenados durante el giro para que el pallet no ruede |
| `retencionEnGiro` | enum | ninguna | ninguna, topes de plataforma | Topes que retienen el pallet durante el giro |
| `velocidadCarga` | m/s | 0,25 | 0,05–0,6 | Velocidad de los rodillos al recibir y descargar |

- **Modo 3D:** con los rodillos bloqueados (ω = 0) el contacto es de deslizamiento con μ, no de rodadura; con rodillos libres el pallet rodaría y se desplazaría durante la aceleración angular, por eso el bloqueo es la opción por defecto.
- **Validaciones:** si `bloqueoRodillosEnGiro` está desactivado y no hay retención, avisa del riesgo de desplazamiento.

### 8.5 Mesa de transferencia rodillos→cadenas (`transp.transferencia_rc`)

**Interpretación (a confirmar):** estación única con dos sistemas de transporte ortogonales. Un tramo de rodillos recibe el pallet en sentido longitudinal; entre los rodillos hay cadenas elevables orientadas en sentido transversal. Al subir, las cadenas levantan el pallet de los rodillos, lo desplazan 90° hacia la línea de cadenas contigua y bajan. El pallet cambia de sentido de avance sin girar sobre sí mismo.

**Puertos:** A y B (rodillos, longitudinal), C1 y C2 (cadenas, laterales izquierdo y derecho).

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| Grupo de rodillos | — | — | — | Parámetros de 8.1; `longitud` ≈ largo del pallet + 200 mm |
| `numCadenasTransfer` | — | 3 | 2–5 | Número de cadenas elevables |
| `posicionesCadenasTransfer` | mm | auto | — | Posición longitudinal de cada cadena, entre rodillos y alineada con las tablas del pallet |
| `anchoCadenaTransfer` | mm | 40 | 20–80 | Ancho de cada cadena |
| `carreraElevacion` | mm | 40 | 20–100 | Altura que sobresalen las cadenas sobre los rodillos al subir |
| `holguraBajada` | mm | 10 | 0–30 | Distancia bajo la superficie de rodillos con las cadenas abajo |
| `tiempoSubida` | s | 1,0 | 0,3–5 | Tiempo de elevación |
| `tiempoBajada` | s | 1,0 | 0,3–5 | Tiempo de descenso |
| `actuador` | enum | neumático | neumático, eléctrico | Neumático: movimiento rápido con tope final; eléctrico: con rampas |
| `velocidadTransferencia` | m/s | 0,20 | 0,05–0,5 | Velocidad de las cadenas durante la transferencia |
| `aceleracionTransferencia` | m/s² | 0,5 | 0,1–2 | Rampa de las cadenas |
| `distanciaTransferencia` | mm | auto | — | Recorrido de la cadena (≈ ancho de la mesa); editable |
| `direccionTransferencia` | enum | derecha | izquierda, derecha, bidireccional | Lado hacia el que se transfiere |
| `modoOperacion` | enum | rodillos→cadenas | rodillos→cadenas, cadenas→rodillos, bidireccional | Sentido del flujo |
| `toleranciaPosicion` | mm | ±20 | 5–50 | Desviación admisible del pallet para permitir la elevación |
| `requiereTope` | bool | sí | — | Incluye un tope retráctil de posicionamiento en el tramo de rodillos |
| `muCadenaTransfer` | — | 0,30 | 0,1–0,6 | Rozamiento pallet–cadena |

**Secuencia:** reposo (cadenas abajo) → pallet en posición (tope y sensores) → subiendo → elevado → transfiriendo → descargado → bajando → reposo.

**Interlocks:** no subir sin pallet centrado; rodillos detenidos mientras las cadenas están arriba; no bajar hasta que el pallet haya dejado la zona de cadenas; en modo bidireccional no aceptar pallets por los dos lados a la vez.

**Ciclo ideal:** t = tSubida + (d/v + v/a) + tBajada (+ asentamiento). Ejemplo: d = 1000 mm, v = 0,2 m/s, a = 0,5 m/s² → 1,0 + 5,4 + 1,0 = 7,4 s.

**Validación del hueco:** con Ø 76 mm y paso 133 mm el hueco entre rodillos es 57 mm, mayor que el ancho de cadena (40 mm) + 10 mm, así que cumple.

- **Modo 2D:** máquina de estados con las duraciones anteriores; durante la elevación el tramo de rodillos queda bloqueado.
- **Modo 3D:** las cadenas son cuerpos cinemáticos que suben (movimiento vertical) y luego se desplazan en horizontal; el pallet se levanta por contacto. El simulador avisa si una cadena cae en un hueco del pallet y no lo toca.

### 8.6 Resumen comparativo

| Tipo | Puertos | Acumulación | Elementos móviles | Punto crítico |
| --- | --- | --- | --- | --- |
| Mesa de rodillos | A, B | ZPA, continuo o con presión | Rodillos | Fricción y paso respecto a las tablas |
| Mesa de cadenas | A, B | Con deslizamiento o por zonas | Cadenas | Superficie móvil en 3D |
| Giratoria de cadenas | Uno por estación | No (de uno en uno) | Plataforma + cadenas | Perfil angular y sujeción por fricción |
| Giratoria de rodillos | Uno por estación | No (de uno en uno) | Plataforma + rodillos | Bloqueo de rodillos en el giro |
| Transferencia rodillos→cadenas | A, B, C1, C2 | No | Rodillos + cadenas elevables | Secuencia, interlocks y hueco entre rodillos |

## 9. Sensores, topes y guías

### 9.1 Sensores fotoeléctricos

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `tipo` | enum | difusa | difusa, reflex, barrera | La barrera usa dos posiciones (emisor y receptor) |
| `anfitrion` | ref | — | — | Transportador donde se monta |
| `s` | mm | 0 | 0 a longitud | Posición a lo largo del anfitrión |
| `lateral` | mm | 0 | ± anchoUtil / 2 | Desplazamiento transversal |
| `altura` | mm | 60 | 0–400 | Altura del haz sobre la superficie de transporte |
| `orientacion` | ° | 90 | 0–180 | Dirección del haz respecto al avance (90° = transversal) |
| `alcance` | mm | 1000 | 50–5000 | Distancia máxima de detección |
| `diametroSpot` | mm | 10 | 1–50 | Diámetro del haz |
| `tiempoRespuesta` | ms | 2 | 0–50 | Retardo propio del sensor |
| `polaridad` | enum | activo al detectar | activo al detectar, activo al ocultarse | Lógica de la salida |
| `filtro` | ms | 0 | 0–2000 | Tiempo mínimo para cambiar de estado; ignora huecos del pallet |
| `retardoCiclo` | ms | 10 | 0–100 | Muestreo por el controlador (ciclo de PLC) |
| `señalSalida` | texto | auto | — | Nombre de la señal booleana |
| `mostrarHaz` | bool | sí | — | Dibuja el haz en 3D (verde libre, rojo detectando) |

**Atención a los huecos del pallet.** A 60 mm sobre la superficie, un haz transversal atraviesa la capa de tacos y tablas de unión: ve unas pocas bandas ocupadas (del orden de 150 mm) separadas por huecos de unos 380 mm, valores aproximados que dependen del pallet. A 0,25 m/s esos huecos duran cerca de 1,5 s y la señal parpadea. Soluciones: subir el haz a la altura de las tablas superiores (≈ 125 mm en un EUR) o usar un `filtro` mayor que hueco / velocidad. El perfil real se define en cada tipo de pallet.

- **Modo 2D:** el haz es un segmento que se cruza con el perfil de ocupación del pallet a esa altura; los flancos se calculan de forma analítica.
- **Modo 3D:** raycast (o shape-cast para spots grandes) contra los colisionadores reales en cada paso de física, con marca de tiempo del flanco.

### 9.2 Topes mecánicos

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `tipo` | enum | retráctil | fijo, retráctil | El fijo está siempre arriba; el retráctil lleva actuador |
| `anfitrion`, `s` | ref, mm | — | — | Posición del plano de contacto a lo largo del transportador |
| `anchoTope` | mm | 100 | 20–400 | Ancho transversal; uno ancho o dos topes alinean mejor el borde frontal |
| `alturaTope` | mm | 100 | 30–300 | Altura sobre la superficie de transporte |
| `tiempoSubida` | s | 0,3 | 0,05–2 | Tiempo para elevarse |
| `tiempoBajada` | s | 0,3 | 0,05–2 | Tiempo para bajar |
| `estadoInicial` | enum | arriba | arriba, abajo | Estado al iniciar |
| `restitucion` | — | 0,1 | 0–0,5 | Rebote del pallet al impactar |
| `muTope` | — | 0,3 | 0,1–0,6 | Rozamiento con el pallet |
| `energiaMaxImpacto` | J | 200 | 10–2000 | Avisa si la energía cinética de impacto lo supera |
| `señalMando` | texto | auto | — | Entrada booleana (arriba o abajo) |
| `señalPosicion` | bool | no | — | Publica finales de carrera de arriba y abajo |

**Energía de impacto:** E = ½·m·v². Para 1025 kg (pallet + carga): a 0,25 m/s son 32 J y a 0,5 m/s son 128 J. El simulador registra velocidad y energía de cada impacto.

### 9.3 Guías laterales (guardrails)

Vienen integradas en cada transportador (grupo `guias`) y también como componente independiente para tramos extra.

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `lado` | enum | ambos | izquierda, derecha, ambos, ninguno | Lados con guía |
| `alturaGuia` | mm | 150 | 50–400 | Altura sobre la superficie de transporte |
| `holgura` | mm | 25 | 0–100 | Holgura por lado entre el pallet y la guía |
| `sInicio`, `sFin` | % | 0, 100 | 0–100 | Tramo cubierto de la longitud |
| `entradaEmbudo` | bool | no | — | Añade una entrada divergente |
| `anguloEmbudo` | ° | 10 | 3–30 | Ángulo de la entrada |
| `longitudEmbudo` | mm | 300 | 100–800 | Longitud de la entrada |
| `muGuia` | — | 0,30 | 0,1–0,6 | Rozamiento con el pallet |
| `restitucionGuia` | — | 0,1 | 0–0,5 | Rebote lateral |

### 9.4 Cómo se alinean los pallets

- El **tope** fija la posición longitudinal del borde frontal. Con un único punto de contacto el pallet puede girar; con un tope ancho o dos topes se endereza.
- Las **guías** fijan la posición lateral; el embudo corrige entradas desalineadas.
- En 2D solo se aplica la posición final ideal; en 3D se obtiene la pose real (posición y ángulo), que es la medida de calidad de la alineación.

## 10. Objetos auxiliares del MVP

### 10.1 Generador de pallets (Source)

| Parámetro | Unidad | Defecto | Rango | Descripción |
| --- | --- | --- | --- | --- |
| `tipoPallet` | ref | EUR | — | Tipo de pallet de la librería |
| `masaCarga` | kg | 600 | 0–1500 | Fija o distribución (uniforme, normal, tabla) |
| `alturaCarga` | mm | 1200 | 0–2200 | Altura de la carga sobre el pallet |
| `orientacion` | enum | largo en el avance | largo, ancho | Orientación al generarse |
| `modoGeneracion` | enum | intervalo fijo | intervalo fijo, distribución, bajo demanda (señal), lista | Regla de llegada |
| `intervalo` | s | 20 | 1–3600 | Para intervalo fijo |
| `maxPallets` | — | ilimitado | — | Límite de pallets generados |
| `bloquearSiOcupado` | bool | sí | — | Espera si no hay espacio libre al inicio |
| `semilla` | entero | global | — | Para distribuciones |

### 10.2 Sumidero (Sink)

`modoRetirada` (inmediata, tras un tiempo o por señal) y `tiempoRetirada` (s, por defecto 0). Publica pallets recibidos, rendimiento y tiempo medio de ciclo.

### 10.3 Pallet

Tipo de la librería, masa de tara y de carga, altura de carga, `centroMasaOffset`, perfil de ocupación y atributos personalizados (clave–valor) para reglas futuras.

## 11. Motor discreto (diseño)

- **Representación:** cada transportador es una *pista* con una lista ordenada de pallets; cada pallet tiene un segmento cinemático (t0, s0, v0, a) con el que se calcula su posición en cualquier instante.
- **Eventos principales:** `pallet_entra`, `frente_alcanza_tope`, `cola_libera_entrada`, `sensor_flanco`, `zona_liberada`, `giro_completado`, `elevacion_completada`, `transferencia_completada`, `senal_cambia` y `regla_dispara`.
- **Predicción:** tras cada evento se recalcula el siguiente evento de los pallets afectados; los eventos obsoletos se invalidan con un número de versión.
- **Acumulación:** el pallet que sigue no puede avanzar más allá de la cola del anterior menos la separación mínima; en ZPA, la zona arranca solo cuando la siguiente queda libre; en acumulación con presión se aplica el tiempo de detención por contacto.
- **Sensores:** sus flancos se calculan analíticamente cruzando el haz con el perfil de ocupación del pallet en movimiento.
- **Controlador:** las reglas se evalúan cuando cambia una señal; los retardos se programan como eventos.
- **Estadísticas:** tiempos acumulados por estado de cada objeto, contadores y registro de eventos exportable (CSV/JSON).
- **Determinismo:** orden de eventos por (tiempo, prioridad, secuencia) y generador de números aleatorios con semilla.
- **Objetivo de rendimiento:** al menos 1000× el tiempo real con 50 objetos y 30 pallets.

## 12. Motor físico: diseño y spike

### 12.1 Elección del motor

Candidatos: **Rapier** (bindings JS oficiales, licencia Apache-2.0) y **Jolt** (WASM, licencia MIT). Se decide en el spike S1 con los criterios de 12.4.

**Hallazgo importante: la superficie móvil.** Los motores de física modelan una cinta transportadora modificando el contacto para que la superficie tenga velocidad propia. Según la documentación consultada, Rapier lo permite en su versión Rust mediante el hook `modify_solver_contacts`, pero en los bindings JS revisados (versión 0.18) ese hook figura como no soportado todavía en JavaScript. Jolt expone `mRelativeLinearSurfaceVelocity` en los ajustes de contacto de su API C++; falta comprobar si el binding JS/WASM lo expone. Por eso el plan no depende de ello:

| Plan | Cómo se modelan las cadenas | Cuándo se usa |
| --- | --- | --- |
| A | Tiras de rodillos pequeños cinemáticos (Ø ≈ 30 mm cada ≈ 40 mm) con ω = v / r | Por defecto: no necesita hooks |
| B | Superficie móvil real por contacto | Si el spike confirma que el motor elegido lo permite |
| C | Fuerza de arrastre sobre el pallet, limitada por μ × N | Respaldo simple si A resulta inestable |
| D | Módulo físico propio en Rust compilado a WASM con el hook activado | Último recurso; añade Rust solo a ese módulo |

### 12.2 Modelo físico

- **Pallet:** cuerpo dinámico compuesto por 3 cajas para las tablas inferiores y 1 caja para el bloque de carga, con masa y centro de masa explícitos; fricción y restitución según el material; amortiguación lineal y angular pequeña; detección continua de colisiones (CCD) para los topes.
- **Rodillos:** cilindros cinemáticos controlados por velocidad, con ω = v / (diámetro / 2); el rozamiento con las tablas inferiores transmite el movimiento.
- **Cadenas:** según los planes A a D.
- **Plataformas giratorias:** cuerpo cinemático con velocidad angular; sus rodillos o cadenas son hijos de la plataforma.
- **Cadenas elevables:** cuerpos cinemáticos con movimiento vertical y luego horizontal.
- **Topes y guías:** cinemáticos (topes) y estáticos (guías), con la restitución y la fricción de sus parámetros.
- **Sensores:** raycast o shape-cast en cada paso, con marca de tiempo del flanco.

### 12.3 Numérica y ejecución

- Paso fijo de 1/120 s (propuesta), con acumulador y subpasos si hace falta; iteraciones del solver ajustables.
- El controlador se evalúa cada 10 ms y escribe consignas (velocidades de rodillo, posiciones de tope).
- Cuerpos dormidos cuando no hay pallets cerca; `InstancedMesh` para dibujar.
- Registro de impactos (velocidad y energía), pose final de cada pallet y fuerzas de contacto.
- Determinismo: mismo orden de creación de cuerpos y misma semilla; hash del estado cada N pasos para detectar divergencias.

### 12.4 Spike S1: criterios de aprobación

| # | Prueba | Criterio de aprobación |
| --- | --- | --- |
| a | Pallet de 1000 kg sobre 10 rodillos cinemáticos | Alcanza la velocidad nominal con ±3 % y frena sin deslizamiento excesivo |
| b | Tira de rodillos pequeños como cadena | Velocidad del pallet igual a la de la cadena ±3 %, sin vibración visible |
| c | Pallet contra un tope a 0,25 m/s | Frente a ±5 mm del tope y rebote menor de 10 mm |
| d | Guía con embudo | Una desalineación de 50 mm y 5° queda dentro de ±10 mm y ±1° |
| e | Giro de 90° sobre plataforma | El pallet se desplaza menos de 5 mm respecto a la plataforma |
| f | Rendimiento | 20 transportadores y 10 pallets a 60 FPS, sin inestabilidades numéricas |
| g | Determinismo | Mismo hash de estado tras 60 s en dos ejecuciones |

Resultado esperado: un registro de decisión en `docs/decisions/` con el motor elegido, el plan para las cadenas y las limitaciones encontradas.

## 13. Editor y visor del MVP

### 13.1 Funciones mínimas

- **Vistas:** 3D en perspectiva y 2D cenital (ortográfica), intercambiables con una tecla; cámara orbital, zoom y encuadre.
- **Edición:** arrastrar componentes desde la librería, mover y girar con gizmos, rejilla de 100 mm con ajuste (snap) a puertos y a la rejilla, copiar, pegar y repetir en matriz, deshacer y rehacer.
- **Conexión:** encaje automático de puertos con avisos de altura, ancho y orientación.
- **Propiedades:** panel generado desde el esquema de la definición, con grupos, unidades, rangos, valores derivados y avisos de validación.
- **Esquema del modelo:** lista de objetos con búsqueda, ocultar y bloquear.
- **Ejecución:** reproducir, pausar, paso a paso y velocidades ×1, ×10 y ×100 (más el máximo en modo 2D); reloj de simulación y selector de modo 2D/3D.
- **Visualización en vivo:** haces de sensores en verde o rojo, topes arriba o abajo, pallets coloreados por estado, panel de señales con sus valores y panel de estadísticas.
- **Controlador:** editor de reglas Si/Entonces con plantillas.
- **Archivo:** guardar y abrir `.intracraft.json`, exportar estadísticas a CSV/JSON y capturas de pantalla.
- **Herramientas:** medición de distancias y capas visibles (guías, sensores, cotas).

### 13.2 Pospuesto

Plano de fondo (imagen o DXF) como base, grabación de vídeo, informes y comparación de ejecuciones.

## 14. Plan priorizado por fases

Orden pensado para atacar primero el mayor riesgo (la física de las cadenas) y construir cada transportador de punta a punta. Tamaños orientativos, medidos en sesiones de trabajo con Claude Code: S ≈ 1–3, M ≈ 4–8, L ≈ 8–15; dependen mucho de tu ritmo y de cuántas veces haya que corregir.

### 14.1 Fases del MVP

| Fase | Objetivo | Entregables | Criterio de salida | Tamaño | Riesgo |
| --- | --- | --- | --- | --- | --- |
| 0. Preparación | Terreno listo | Repositorio, espacios de trabajo pnpm, `AGENTS.md`, carpeta `docs/specs`, glosario, 3 modelos de referencia de Plant Simulation (capturas y resultados) | Las pruebas se ejecutan (aunque vacías) y la integración continua pasa | S | Bajo |
| 1. Spikes | Reducir incertidumbre | S1 física (12.4), S2 visor 3D con rodillos instanciados y gizmos, S3 núcleo DES mínimo (Source → pista → Sink) | Criterios de S1 cumplidos y decisión de motor registrada | M | Alto |
| 2. Núcleo y librería | Base de datos del modelo | `core-model` (unidades, puertos, conexiones, validación), esquema de definiciones y parámetros, registro de comportamientos, tipos de pallet, tabla de fricción, formato de archivo | Un layout Source → Sink se guarda y se abre; las validaciones avisan | M | Medio |
| 3. Mesa de rodillos (2D) | Primer transportador completo en 2D | Definición, geometría, comportamiento DES (cinemática y ZPA), sensores, topes y guías en 2D, controlador básico, CLI | Pruebas T1 y T2 de la sección 15; animación 3D de la simulación 2D | L | Medio |
| 4. Física base | Modo 3D para rodillos | Mundo físico, rodillos cinemáticos, pallet compuesto, sensores por raycast, topes y guías | Pruebas a, c, d y g de S1 dentro del producto; consistencia 2D/3D de la mesa de rodillos | L | Alto |
| 5. Mesa de cadenas | Cadenas en ambos modos | Definición, DES con deslizamiento, física con plan A (o el elegido) | Velocidad de pallet = cadena ±3 %; acumulación con deslizamiento | M | Alto |
| 6. Giratoria de cadenas | Giro en ambos modos | Perfil angular, estaciones, interlocks, puertos por estación | T3 (6,5 s) y prueba e de S1 | M | Medio |
| 7. Giratoria de rodillos | Reutilizar lo anterior | Rodillos sobre plataforma, bloqueo en giro | Misma batería de pruebas que 6 | S–M | Bajo |
| 8. Transferencia rodillos→cadenas | Secuencia completa | Cadenas elevables, interlocks, validación de huecos, ambos modos | T4 (7,4 s) y transferencia en 3D sin pérdida de pallet | L | Alto |
| 9. Integración | Cerrar el MVP | Layout de referencia con los cinco tipos, cambio de modo, contraste con Plant Simulation, rendimiento, documentación, empaquetado | Definición de hecho de 2.3 | L | Medio |

**Por qué este orden:** la mesa de rodillos es la base de las demás; las cadenas concentran el riesgo físico y conviene resolverlo pronto; las giratorias reutilizan rodillos y cadenas; la transferencia depende de todo lo anterior (rodillos, cadenas, topes y sensores).

### 14.2 Definición de hecho por transportador

1. Especificación escrita en `docs/specs/<objeto>.md`.
2. Definición en la librería con parámetros, rangos, derivados y validaciones.
3. Geometría paramétrica y símbolo 2D.
4. Comportamiento 2D con pruebas analíticas.
5. Comportamiento 3D con pruebas de física.
6. Plantilla de control incluida.
7. Escenario de referencia en `packages/scenarios`.
8. Documentación de parámetros generada desde la definición.

### 14.3 Después del MVP (orden sugerido)

1. Traspaso de estado entre modos en mitad de la simulación.
2. Calibración automática 2D↔3D.
3. Curvas, desviadores, elevadores de varios niveles y lanzaderas.
4. Cajas y cubetas.
5. Fallos (MTBF/MTTR) y consumo de energía.
6. Scripting de usuario.
7. Experimentos, réplicas y optimización.
8. Conexión OPC UA/PLC.
9. Importación de modelos 3D de fabricantes (con licencia) y de planos DXF.
10. Versión de escritorio con Tauri.

## 15. Pruebas y validación

### 15.1 Capas de pruebas

1. **Unitarias:** unidades, cinemática, cola de eventos, validación de parámetros.
2. **Escenarios 2D con resultado analítico** (15.2).
3. **Escenarios de física** (15.3).
4. **Consistencia entre modos** (15.4).
5. **Regresión con resultados guardados** y **determinismo** (15.5).
6. **Contraste con Plant Simulation** (15.6).

### 15.2 Pruebas analíticas del modo 2D

| Id | Escenario | Resultado esperado |
| --- | --- | --- |
| T1 | Un pallet en una mesa de rodillos, recorre D = 3 m desde reposo con v = 0,25 m/s y a = 0,5 m/s², sin frenar al final | t = D/v + v/(2a) = 12 + 0,25 = 12,25 s |
| T2 | Saturación: Source sin límite, EUR con el lado largo (1,2 m) en el avance, separación 0,15 m | Rendimiento máximo = v / (L + separación) = 0,25 / 1,35 ≈ 0,185 pallets/s ≈ 667 pallets/h |
| T3 | Giro de 90° con ω = 15 °/s y α = 30 °/s² | 6,5 s |
| T4 | Ciclo de transferencia con d = 1 m, v = 0,2 m/s, a = 0,5 m/s² y 1 s de subida y de bajada | 7,4 s |
| T5 | Energía de impacto de 1025 kg a 0,25 m/s | 32 J (½·m·v²) |
| T6 | Sensor a 2,0 m con haz alto (sin huecos) | Flanco de subida cuando el frente cruza el haz y de bajada cuando lo cruza la cola; duración = largo / v = 4,8 s |

### 15.3 Escenarios de física

| Id | Escenario | Criterio |
| --- | --- | --- |
| P1 | Parada contra tope | Frente a ±5 mm; rebote menor de 10 mm |
| P2 | Impacto | Velocidad y energía registradas; aviso al superar `energiaMaxImpacto` |
| P3 | Guía en embudo | Desalineación de 50 mm y 5° corregida a ±10 mm y ±1° |
| P4 | Giro de 90° | Desplazamiento del pallet menor de 5 mm; sin deslizamiento con μ ≥ 0,04 |
| P5 | Transferencia | Pallet elevado 40 mm ±5, desplazado 1 m ±10 mm y depositado sin girar más de 1° |
| P6 | Sensores | Flancos de 3D y de 2D coinciden dentro de ±50 ms |

### 15.4 Consistencia entre modos

Correr el mismo escenario en ambos modos y comparar tiempos de tránsito (±5 %) y flancos de sensores (±50 ms). Una diferencia mayor indica un error o un parámetro de calibración que falta (alimenta 5.3).

### 15.5 Regresión y determinismo

- Resultados guardados (golden) en `test_data/` para cada escenario; cualquier cambio de resultado debe quedar explicado en el registro de cambios.
- Dos ejecuciones con la misma semilla deben dar el mismo hash del registro de eventos (2D) y del estado (3D).

### 15.6 Contraste con Plant Simulation

- Elegir 3 modelos que ya conozcas bien: uno simple de flujo, uno con acumulación y uno con giratoria o transferencia.
- Reproducirlos en IntraCraft (modo 2D) y comparar rendimiento, tiempo medio de ciclo y ocupación.
- Documentar las diferencias en lugar de forzarlas: los dos programas pueden modelar la acumulación de forma distinta; lo importante es entender y justificar cada diferencia.
- No hace falta leer ni importar archivos de Plant Simulation, solo comparar resultados.

## 16. Cómo trabajar con Claude Code siendo no programador

### 16.1 Ciclo de trabajo (para cada paso pequeño)

1. Describe el objetivo en lenguaje de simulación: «la mesa de rodillos debe acelerar a 0,25 m/s y detenerse contra el tope».
2. Pide que lea la especificación del objeto y proponga un plan corto antes de tocar código (modo plan).
3. Que escriba primero la prueba con el resultado esperado (sección 15).
4. Que implemente hasta que la prueba pase.
5. Que ejecute todas las pruebas y te cuente el resultado y qué cambió, sin jerga de código.
6. Tú compruebas visualmente en el editor (captura o vídeo corto).
7. Commit con un mensaje claro; si algo sale mal, se vuelve al commit anterior.

### 16.2 Reglas útiles

- Una cosa pequeña a la vez y una rama por función.
- Las especificaciones (`docs/specs`) son la fuente de verdad: si cambias de idea, cambia primero la especificación.
- Pide siempre que explique las decisiones en términos de simulación (qué hace el objeto), no de código.
- Si algo falla dos veces, pide un diagnóstico antes de otro intento: «explica por qué falla y propón dos alternativas».
- Guarda los resultados de referencia; si un resultado cambia, que lo justifique.
- Para tareas grandes, pide un plan por escrito y revísalo tú antes de continuar.

### 16.3 Primeros prompts (ejemplos)

1. «Lee AGENTS.md y crea la estructura del repositorio de IntraCraft como describe la sección 3.2 del plan. Usa pnpm, TypeScript estricto y Vitest. No implementes nada más.»
2. «Implementa el spike S1: un pallet de 1000 kg sobre 10 rodillos cinemáticos con Rapier. Aplica los criterios a y c de la tabla 12.4 y muestra los resultados medidos.»
3. «Escribe docs/specs/mesa-de-rodillos.md con los parámetros de 8.1 y las pruebas T1 y T2. Todavía no escribas código.»

### 16.4 Borrador de `AGENTS.md`

```
# AGENTS.md — IntraCraft

Simulador híbrido (2D discreto y 3D con física) de transporte de pallets.
Monorepo TypeScript (pnpm). Documentos de diseño en docs/specs.

## Reglas
- Unidades SI internas (m, kg, s, rad); la interfaz muestra mm y grados.
- Eje Y hacia arriba. Tiempo en enteros (microsegundos).
- Nunca usar Math.random() en los motores; usar el generador con semilla.
- Los motores (sim-des, sim-physics, control) no dependen de React ni del DOM.
- Cada objeto de la librería tiene: definición, geometría, comportamiento 2D,
  comportamiento 3D y pruebas.
- No se lee, importa ni copia código ni archivos de software de terceros (clean-room).
- Antes de cambiar un resultado guardado en test_data, explicar por qué cambia.

## Estilo
- TypeScript estricto; dos espacios de indentación; sin console.log fuera de pruebas.
- Orden en cada archivo: constantes, tipos, funciones públicas, funciones privadas.

## Comandos
- pnpm install · pnpm test · pnpm lint · pnpm dev (editor)
- pnpm run scenario <nombre>  (ejecuta un escenario sin interfaz)

## Flujo de trabajo
1. Leer la especificación del objeto en docs/specs.
2. Proponer un plan corto.
3. Escribir primero la prueba y después la implementación.
4. Ejecutar todas las pruebas y resumir los resultados en lenguaje de simulación.
```

## 17. Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
| --- | --- | --- |
| La superficie móvil no está disponible en el motor JS elegido (cadenas) | Alto | Planes A a D de 12.1 y spike S1 antes de construir |
| Inestabilidad física con cargas pesadas y topes (pallets que vibran, atraviesan o se disparan) | Alto | Paso fijo pequeño, CCD, masas y fricciones realistas, pruebas P1 a P6, escenarios simples primero |
| El alcance crece sin control | Alto | MVP cerrado en 2.1 y 2.2; toda idea nueva va a la lista post-MVP |
| Los resultados de 2D y 3D no coinciden | Medio | Mismo controlador, pruebas de consistencia (15.4) y calibración 2D↔3D después del MVP |
| Deriva del código generado por IA | Medio | `AGENTS.md`, especificaciones por objeto, pruebas primero, commits pequeños y revisión de resultados |
| Parámetros de fricción y masa poco fiables | Medio | Presets marcados como orientativos, calibración con hojas de datos y análisis de sensibilidad más adelante |
| Rendimiento del modo 3D con muchos rodillos | Medio | Mallas instanciadas, cuerpos dormidos, rodillos agrupados por zona y nivel de detalle reducido |
| Propiedad intelectual: marcas, modelos de fabricantes y software de terceros | Medio | Clean-room; no usar nombres de Siemens ni de otros productos; no importar CAD de fabricantes sin licencia; revisar licencias de dependencias |
| Cambios en el modelo de datos rompen archivos antiguos | Medio | `schemaVersion` y migraciones desde el primer día |
| Dependencia de un motor de física externo | Bajo | Versiones fijadas y envoltorio propio (`sim-physics`) que aísla el motor |
| Los archivos de Plant Simulation no se pueden abrir | Informativo | No se intenta; el contraste se hace comparando resultados (15.6) y, más adelante, con intercambio en formatos abiertos |

## 18. Decisiones abiertas y preguntas por confirmar

1. **Mesa de transferencia (8.5):** ¿es correcta la interpretación de rodillos longitudinales con cadenas elevables transversales, o es otro mecanismo (por ejemplo un carro transferidor)?
2. **Orientación del pallet (8.2):** ¿se confirma la convención de rodillos con el lado largo en el avance y cadenas con el lado corto?
3. **Pallets habituales:** dimensiones, pesos máximos (¿hasta 1500 kg?) y tipos que más usas.
4. **Velocidades y fabricantes de referencia:** rango real de velocidades de tus transportadores y hojas de datos disponibles para crear presets realistas.
5. **Fallos y energía:** ¿deben entrar en el MVP (MTBF/MTTR, consumo) o quedan para después?
6. **Idioma:** interfaz y librería en español, en inglés o en ambos (conviene preparar las traducciones desde el inicio).
7. **Modelos de contraste:** qué tres modelos de Plant Simulation vas a usar para validar (15.6).
8. **Equipo objetivo del modo 3D:** portátil con GPU dedicada o también equipos modestos (afecta al nivel de detalle).
9. **Licencia y uso:** código abierto, uso interno o comercial; y comprobar el nombre IntraCraft (marca, dominio, GitHub, npm).
10. **Decisión técnica pendiente:** motor de física y plan para las cadenas, que se cierra en el spike S1 (12.1).

**Supuestos mientras tanto:** se usan los valores por defecto de las secciones 6, 7, 8 y 9, el tamaño de los pallets de 6.9, el MVP sin fallos ni energía, y la interfaz en español con claves preparadas para traducción.

## 19. Apéndices

### A. Ejemplo ilustrativo de definición de componente (mesa de rodillos, abreviada)

```
export const mesaDeRodillos: ComponentDefinition = {
  id: 'transp.rodillos',
  version: '0.1.0',
  categoria: 'transportadores',
  nombre: { es: 'Mesa de rodillos', en: 'Roller conveyor' },
  parametros: [
    { clave: 'longitud', tipo: 'numero', unidad: 'mm', defecto: 3000,
      min: 500, max: 12000, grupo: 'geometria', afecta: 'ambos',
      descripcion: 'Longitud en el sentido de avance' },
    { clave: 'diametroRodillo', tipo: 'numero', unidad: 'mm', defecto: 76,
      min: 50, max: 114, grupo: 'rodillos', afecta: 'ambos' },
    { clave: 'pasoRodillos', tipo: 'numero', unidad: 'mm', defecto: 133,
      min: 75, max: 250, grupo: 'rodillos', afecta: 'ambos' },
    { clave: 'numRodillos', tipo: 'entero', derivado: true,
      formula: 'floor(longitud / pasoRodillos)' },
  ],
  puertos: [
    { id: 'A', tipo: 'entrada', local: { x: 0, y: 0, z: 0 } },
    { id: 'B', tipo: 'salida', local: { x: 'longitud', y: 0, z: 0 } },
  ],
  senales: {
    entradas: ['marcha', 'reversa', 'habilitar'],
    salidas: ['en_marcha', 'ocupado'],
  },
  geometria: 'geom.transportador_rodillos',
  comportamientos: { des: 'des.rodillos', fisica: 'fis.rodillos' },
  reglasValidacion: ['min3RodillosBajoPallet', 'anchoUtilMayorQuePallet'],
  presets: [{ nombre: 'Pesado 1000 kg, 0,25 m/s', valores: { velocidadNominal: 0.25 } }],
};
```

### B. Contrato de comportamiento (esquema ilustrativo)

```
interface ComponentBehavior<P> {
  readonly id: string;
  init(ctx: SimContext, params: P): void;
  onEvent?(ctx: DesContext, evento: SimEvent): void;   // modo 2D
  step?(ctx: PhysicsContext, dt: number): void;         // modo 3D
  leerSenales(): Record<string, boolean>;               // salidas hacia el controlador
  escribirSenales(entradas: Record<string, boolean>): void;
  estadisticas(): Record<string, number>;
}
```

### C. Glosario español – inglés

| Español | English |
| --- | --- |
| Mesa de rodillos | Roller conveyor |
| Mesa de cadenas | Chain conveyor |
| Giratoria (de cadenas o de rodillos) | Turntable (chain or roller) |
| Mesa de transferencia | Transfer table (chain transfer) |
| Tope | Mechanical stop |
| Guía lateral | Guardrail, side guide |
| Fotocélula | Photoelectric sensor (photoeye) |
| Acumulación sin presión | Zero-pressure accumulation (ZPA) |
| Paso de rodillos | Roller pitch |
| Simulación de eventos discretos | Discrete-event simulation (DES) |
| Puesta en marcha virtual | Virtual commissioning |
| Gemelo digital | Digital twin |
| Superficie móvil | Surface velocity (conveyor-belt effect) |

### D. Fuentes consultadas

- realvirtual.io: [física](https://doc.realvirtual.io/basics/physics), [soporte de AGX](https://doc.realvirtual.io/extensions/agx-physics), [sistema de rutas](https://doc.realvirtual.io/extensions/page-4/path-system) y [página principal](https://www.realvirtual.io/en/).
- Rapier: [hooks de física (Rust)](https://docs.rs/crate/rapier3d/0.32.0/source/src/pipeline/physics_hooks.rs) y [hooks en los bindings JS 0.18](https://app.unpkg.com/@dimforge/rapier3d@0.18.0/files/pipeline/physics_hooks.js).
- Jolt Physics: [ContactSettings y velocidad superficial](https://jrouwe.github.io/JoltPhysics/class_contact_settings.html).
- ArtCraft (storytold): [repositorio](https://github.com/storytold/artcraft) y su [AGENTS.md](https://github.com/storytold/artcraft/blob/main/AGENTS.md).

Las dimensiones de pallets, los coeficientes de fricción y los valores por defecto de los transportadores son conocimiento general y puntos de partida: deben contrastarse con normas, hojas de datos y ensayos.
