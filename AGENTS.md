# AGENTS.md

Guía para agentes de código (Claude Code y similares) que trabajan en el monorepo de **IntraCraft**.
`CLAUDE.md` debe apuntar a este archivo (por ejemplo, con el contenido `@AGENTS.md`).

## Resumen del proyecto

IntraCraft es un simulador híbrido de sistemas de transporte de pallets (intralogística) con **un solo layout y dos modos**:

- **Modo 2D discreto (DES):** simulación por eventos, mucho más rápida que el tiempo real. Sirve para rendimiento, acumulación, bloqueos y utilización.
- **Modo 3D con física (tiempo real):** rozamiento de rodillos y cadenas, colisiones con topes y guías laterales, alineación de pallets y sensores con geometría real.

El MVP incluye cinco transportadores: mesa de rodillos, mesa de cadenas, giratoria de cadenas, giratoria de rodillos y mesa de transferencia de rodillos a cadenas, además de sensores fotoeléctricos, topes mecánicos, guías laterales, generador y sumidero de pallets, un controlador por reglas Si/Entonces y una **librería** de componentes parametrizables.

Ampliación del 30-10-2026 (`docs/en/decisions/ADR-0012 Scope Extension Lifts Elevator and Gravity Line.md`): mesa elevadora manual para transpaleta, mesa de tijera, elevador de dos niveles (rodillos o cadenas) y línea de rodillos por gravedad con separador (especificaciones S12–S15).

Ampliación del 31-10-2026, construida el 09-11-2026 (`docs/en/decisions/ADR-0018 IntraScript Scripting Language.md` y `ADR-0027 IntraScript in Python Syntax with Its Own Interpreter.md`): lenguaje de programación propio **IntraScript**, con la sintaxis de Python, eventos como decoradores (`@on_enter(Roller3)`), esperas (`wait(2 * s)`) y unidades (`150 * mm`), ejecutado por un intérprete propio en Rust (`platform/sim/script`, sin dependencias, aislado y determinista). El código vive en el **bloque de código** (`info.code`, S21), colocado en el plano y editado en su propio editor. Referencia: `docs/en/app/A17 IntraScript Language.md`; tutorial: `A20 IntraScript Tutorial.md`. Funciona en el módulo de simulación discreta; en el físico, todavía no. Los objetos variable, tabla, lista y zona (S18–S20, S22) están especificados y sin construir.

Ampliación del 10-11-2026, construida el 11-11-2026 (`docs/en/decisions/ADR-0033 Scope Extension High-Bay Warehouse and Stacker Cranes.md`, especificación S26, asistente A22): **almacén de gran altura (HBW)** para pallets. Cada **pasillo de almacén** (`storage.aisle`) son dos filas de estantería (simple o doble profundidad) y un **transelevador** que recoge y entrega pallets con horquillas solo sobre transportadores de cadenas, en una a tres alturas. El **asistente** «Warehouse» del módulo Diseño (y `intracraft-cli warehouse`) coloca N pasillos con sus cadenas a partir del largo, el alto y el número de pasillos. Funciona en los tres módulos; ejemplos en `examples/layouts/hbw-*.intracraft.json`.

Ampliación del 10-11-2026, construida el 11-11-2026 (`docs/en/decisions/ADR-0032 Scope Extension Stretch Wrapper.md`, especificación S25): **enfardadora de mesa giratoria** (`flow.stretch_wrapper`), en línea entre dos transportadores de rodillos. Es una estación de proceso cuyo tiempo sale del programa de enfardado (vueltas según la altura de la carga, velocidad de giro, ancho y solape del film) y cuya mesa redonda gira con el pallet; el pallet sale con la marca «enfardado». Funciona en los tres módulos. El film se dibuja, no se simula, y la carga sigue siendo un cuerpo rígido hasta que existan las cargas sueltas (ADR-0031).

**Plataforma Rust (31-10-2026, `docs/en/decisions/ADR-0020 Rust Platform on BIMCraft with Three Modules.md`):** el producto se construye en Rust dentro de **BIMCraft** (aplicación BIM de código abierto, en `platform/`), con **tres módulos** sobre el mismo proyecto: **Diseño** (editor BIM: objetos de la librería como familias, tabla de atributos con heredado ✔/✖, árbol de simulación, tablas, presupuestos, plantas y 3D), **Simulación discreta** (el modo 2D) y **Simulación física** (el modo 3D). El código TypeScript (`packages/`, `apps/`) queda **congelado como referencia**: no recibe funciones nuevas y sus pruebas y escenarios son los valores esperados de la versión Rust. Plan por fases: `docs/en/app/A19 Rust Implementation Plan.md`.

La especificación completa está en el documento «IntraCraft: plan y especificación del MVP» (archivo `FlowLab plan y especificación del MVP (transporte de pallets).md` en la raíz). Las especificaciones por objeto viven en `docs/en/specs/` (versión de referencia; las copias en `docs/es` y `docs/de` se conservan pero pueden quedar desactualizadas). Si algo de este archivo contradice una especificación, se pregunta antes de decidir.

## Quién usa este repositorio

La persona responsable **no es programadora**, pero conoce a fondo los programas de simulación de planta. Por eso:

- Explica los cambios en **términos de simulación** (qué hace el objeto, qué resultado cambia), no de código.
- Trabaja en **pasos pequeños** y, al terminar cada uno, ejecuta las pruebas y resume el resultado.
- Antes de tomar una decisión de arquitectura o de cambiar un comportamiento ya especificado, **pregunta** y ofrece dos alternativas con sus consecuencias.
- Si algo falla dos veces, para y explica por qué falla antes de un tercer intento.

## Estructura del repositorio

```
intracraft/
├─ AGENTS.md
├─ docs/                  (bóveda de Obsidian; Home.md es la portada)
│  ├─ en/ · es/ · de/     (en/ es la referencia; es/ y de/ son copias del 29-10-2026)
│  │  ├─ theory/          (T: teoría de eventos discretos y física)
│  │  ├─ app/             (A: arquitectura, funciones y referencia de API)
│  │  ├─ specs/           (S: una especificación por objeto de la librería)
│  │  └─ decisions/       (ADR: registro de decisiones técnicas)
├─ scripts/               (utilidades del repositorio, p. ej. alinear tablas Markdown)
├─ brand/                 (marca: logo A «rail» elegido, colores, guía; export/ se genera con `cargo run -p intracraft -- --write-brand`)
├─ packages/
│  ├─ core-model/         (layout, puertos, conexiones, unidades, validación)
│  ├─ library/            (definiciones JSON y comportamientos registrados)
│  ├─ commands/           (capa de comandos: sesión, deshacer, registro; ADR-0006)
│  ├─ asset-catalog/      (catálogo de modelos 3D GLB de terceros: manifiesto, formas de colisión y consultas)
│  ├─ geometry/           (geometría paramétrica 3D y símbolos 2D)
│  ├─ sim-des/            (motor de eventos discretos)
│  ├─ sim-physics/        (envoltorio del motor de física y ejecución 3D del layout, ADR-0016)
│  ├─ control/            (señales y reglas Si/Entonces)
│  └─ scenarios/          (escenarios de prueba y resultados esperados)
├─ apps/
│  ├─ editor/             (React + react-three-fiber; web local, Tauri más adelante)
│  └─ cli/                (ejecución de escenarios sin interfaz)
├─ platform/             (BIMCraft como git subtree: espacio de trabajo Cargo; ADR-0020)
│  ├─ crates/             (BIMCraft: core, ui-egui, i18n, update, rvt, dwg; cambios listados en INTRACRAFT-CHANGES.md)
│  ├─ sim/                (crates de IntraCraft: library, model, des, control, physics, avian, assets, scene, script, scenarios, ui)
│  ├─ apps/               (bimcraft y bimcraft-cli de BIMCraft; intracraft, intracraft-cli e intracraft-viewer, los nuestros)
│  └─ vendor/             (código de todas las librerías de Rust: se compila sin red)
├─ spikes/               (pruebas técnicas desechables; r5b-viewer: visores 3D en Rust, ADR-0021)
├─ examples/            (instalaciones de ejemplo: examples/layouts/*.intracraft.json)
└─ test_data/             (resultados guardados y datos de prueba)
```

Si la estructura real difiere, actualizar este archivo en el mismo cambio.

## Stack

**Producto (desde el 31-10-2026): Rust en `platform/`.**

- Rust estable (edición 2024), espacio de trabajo Cargo de BIMCraft; interfaz egui/eframe 0.36 con wgpu; física con Avian 0.8 (`avian3d`, la física de Bevy, con velocidad de superficie para las cadenas) detrás del contrato común `PhysicsBench`; Rapier (`rapier3d`) se conserva congelado como referencia y sus resultados son los esperados para Avian. La simulación 3D de los dos módulos se ve junto a la planta dentro de la ventana principal (**2D + 3D**, `apps/intracraft/src/view3d.rs`, wgpu propio) o en una ventana Bevy 0.20 aparte con sombras (**View in 3D**, `apps/intracraft-viewer`); la escena sale de `sim/scene` y los modelos propios de pallets y cargas de `sim/assets`. Las pestañas MEP de BIMCraft (Plumbing, Electrical, Lightning Protection, Data) están ocultas en IntraCraft. En el módulo Diseño los objetos se mueven con snap entre puertos en la planta y en la 3D integrada; el clic derecho abre su menú de acciones y el doble clic su hoja de propiedades (flotante o acoplada, por secciones, con un control por tipo de valor). Ver `docs/en/decisions/ADR-0026 Bevy and Avian for the Physics Simulation Module.md` y `ADR-0028 Object Editing Snap Action Menu and Property Sheet.md`.
- Todas las librerías están copiadas en `platform/vendor/` y se compila sin red. Una dependencia nueva se añade con `cargo vendor --versioned-dirs vendor` y se sube en el mismo commit (preguntar antes si es pesada).
- Reglas de BIMCraft que también son nuestras (ver `platform/CLAUDE.md`): nunca romper (sin `unwrap`, `expect`, `panic!` ni `unsafe` fuera de las pruebas; las entradas son hostiles), **todo cambio es un comando** que pasa por `Session::execute` (deshacer y vuelta atrás si falla), la validación vive en un solo sitio (`Model::check` y las comprobaciones registradas), el núcleo no depende de la interfaz y la interfaz solo cambia el modelo con comandos, archivos leídos con tolerancia y comandos con rigor (`FORMAT` y migraciones).
- Los crates propios van en `platform/sim/`, no en `platform/crates/` (así no se tocan las comprobaciones propias de BIMCraft). Cada archivo de BIMCraft que se cambie se anota en `platform/INTRACRAFT-CHANGES.md`.
- En Windows, el «Control inteligente de aplicaciones» bloquea los programas que Cargo genera al compilar; en el portátil de la responsable está desactivado.

**Referencia TypeScript (congelada):**

- TypeScript en modo estricto en todo el repositorio; pnpm (espacios de trabajo); Vite; Vitest; ESLint y Prettier.
- Esquemas y validación: Zod.
- Interfaz: React, react-three-fiber (Three.js) y Zustand.
- Física: Rapier o Jolt (WASM), a decidir en el spike S1 (ver «Física» y `docs/en/decisions/ADR-0002 Physics Engine Candidate.md`; candidato preferido: Jolt). Todo acceso al motor pasa por `sim-physics`.
- Versiones fijadas y motivos: `docs/en/decisions/ADR-0001 Toolchain and Versions.md` (TypeScript 6.0 porque typescript-eslint aún no admite la 7).
- Los motores de simulación corren en un Web Worker; la interfaz solo recibe instantáneas de estado.

## Convenciones del modelo

- **Plataforma Rust:** el modelo BIM (módulo Diseño) guarda milímetros con x este, y norte, z arriba, como BIMCraft; los motores trabajan en SI con Y arriba. `intracraft-model` convierte una sola vez al construir la ejecución.
- **Unidades internas SI:** metros, kilogramos, segundos, radianes. La interfaz muestra mm y grados. Convertir solo en los bordes (interfaz y archivos).
- **Ejes:** Y hacia arriba, sistema de mano derecha. La vista 2D es la vista cenital (mirando hacia −Y).
- **Tiempo:** enteros en microsegundos. La cola de eventos se ordena por (tiempo, prioridad, secuencia).
- **Aleatoriedad:** siempre con el generador con semilla del proyecto. Nunca `Math.random()` dentro de los motores.
- **Determinismo:** mismo layout y misma semilla deben producir el mismo resultado. Si una prueba de determinismo falla, es un error de prioridad máxima.
- **Archivos de proyecto:** en la plataforma Rust, `.intracraft` (un proyecto de BIMCraft con el nombre de IntraCraft; también se abren `.bimcraft` e IFC; ADR-0030). En la referencia TypeScript, JSON (`.intracraft.json`) con `schemaVersion` y migraciones explícitas. Los resultados de simulación no se guardan dentro del proyecto.

## Preparado para agentes de IA (desde el principio)

- Toda acción que cambia el proyecto o la simulación es un **comando** tipado (esquema Zod, validación, deshacer, registro con el actor). La interfaz, la CLI, el futuro servidor MCP y el futuro chat integrado usan los mismos comandos; no hay lógica que solo exista en la interfaz. Las lecturas van por una capa de consultas.
- Cada función nueva incluye sus comandos y sus pruebas.
- El servidor MCP y el chat con gateway y token propios **solo están documentados**; no desarrollarlos hasta nuevo aviso. Diseño: `docs/en/app/A15 AI Agent Integration.md` y ADR-0004.
- Los tokens de API nunca se guardan en el archivo de proyecto, logs ni resultados.

## Datos confirmados por el responsable (29-10-2026)

- Pallet europeo (EUR/EPAL): longitudinal en rodillos, transversal en cadenas.
- Rodillos Ø 89 mm con paso 206,3 mm por defecto; diámetro y paso se eligen de una lista corta.
- Por defecto **sin acumulación**: cada pallet sigue el movimiento del rodillo o de la cadena; el comportamiento es parámetro de cada instancia.
- Posición y altura de fotocélula y reflector son parámetros de cada instancia.
- Preguntas pendientes: `docs/en/Open Questions.md`.

## La librería

- Cada componente se describe con una **definición** en JSON (parámetros, puertos, señales, geometría, validaciones, presets) y su **comportamiento** se implementa en TypeScript y se registra por id.
- Todo parámetro lleva: `clave`, `etiqueta`, `tipo`, `unidad`, `defecto`, `min`, `max`, `grupo`, `descripcion`, y si aplica `avanzado`, `afecta` (2D, 3D, ambos o geometría), `derivado` y `dependeDe`. La interfaz de edición se genera desde ese esquema: no se escriben formularios a mano.
- Un objeto de la librería no está terminado hasta tener: especificación, definición con validaciones, geometría y símbolo 2D, comportamiento 2D, comportamiento 3D, plantilla de control, escenario de referencia y pruebas.
- Los valores por defecto de las especificaciones son **orientativos**; no presentarlos como datos de fabricante.

## Física

- Paso fijo propuesto de 1/120 s. El controlador se evalúa cada 10 ms y escribe consignas.
- Pallet: cuerpo dinámico compuesto (tablas inferiores + bloque de carga) con masa y centro de masa explícitos.
- Rodillos: cilindros cinemáticos controlados por velocidad con ω = v / r. Cadenas: tiras de rodillos pequeños (plan A) hasta que el spike S1 decida otra cosa.
- La superficie móvil por contacto puede no estar disponible en los bindings JS del motor elegido; **no asumir que existe**, comprobarlo en el spike. Investigación del 28-10-2026: Jolt JS 1.1.1 la expone (`ContactSettings.mRelativeLinearSurfaceVelocity`); Rapier JS 0.21.0 no.
- Los valores por defecto de los motores (holgura de penetración, umbral de restitución) incumplen el criterio de ±5 mm en el tope; ver `docs/en/theory/T15 Physics Engine Research.md`.
- No cambiar masas, fricciones ni pasos de tiempo para «arreglar» una prueba sin documentarlo en `docs/en/decisions/`.

## Pruebas

- **Primero la prueba, después la implementación.** Los resultados esperados salen de la especificación.
- Pruebas analíticas de referencia (modo 2D, error menor al 1 %):
  - T1: recorrido de 3 m desde reposo con v = 0,25 m/s y a = 0,5 m/s² → 12,25 s.
  - T2: rendimiento máximo con EUR (lado largo) y separación de 0,15 m → ≈ 0,185 pallets/s (≈ 667/h).
  - T3: giro de 90° con ω = 15 °/s y α = 30 °/s² → 6,5 s.
  - T4: ciclo de transferencia (1 m, 0,2 m/s, 0,5 m/s², 1 s de subida y de bajada) → 7,4 s.
  - T5: impacto de 1025 kg a 0,25 m/s → 32 J.
- Física: un pallet contra un tope debe quedar con el frente a ±5 mm y con rebote menor de 10 mm; los eventos de sensores de 2D y 3D deben coincidir dentro de ±50 ms.
- Los resultados guardados en `test_data/` no se modifican sin explicar por qué cambian.
- Antes de dar un paso por terminado: ejecutar todas las pruebas y resumir el resultado.

## Estilo de código

- TypeScript estricto; dos espacios de indentación.
- No usar `console.log` fuera de pruebas; usar el logger del proyecto.
- Preferir `import` a rutas completas en línea. Si dos módulos exportan el mismo nombre, renombrar el import con un sufijo.
- Un archivo por componente de la librería o por comando; el estado, los tipos y los ayudantes compartidos van en módulos de apoyo.
- Los motores (`sim-des`, `sim-physics`, `control`) no dependen de React ni del DOM.

### Orden dentro de cada archivo

Organizar para lectura de arriba abajo: lo importante primero, los detalles después.

- Constantes después de los imports.
- Tipos: los externos antes que los internos.
- Funciones públicas primero; las privadas debajo de las que las usan.
- Dentro de las pruebas: constantes, casos de prueba (agrupados en bloques si hay dos o más) y ayudantes al final.

## Documentación en Markdown

- Las tablas se escriben con celdas rellenadas con espacios para que las columnas queden alineadas en texto plano; se leen en terminales y diffs tanto como renderizadas. `pnpm format:docs` las alinea automáticamente.
- **Idioma (decisión del 29-10-2026): por ahora basta con inglés.** Las notas nuevas y los cambios se escriben solo en `docs/en`. Las versiones en español y alemán ya creadas se conservan tal cual (no se borran); pueden quedar desactualizadas y la referencia es siempre la inglesa. Traducirlas de nuevo solo cuando el responsable lo pida (ADR-0003, enmienda).
- Los nombres de nota son únicos en la bóveda para que los `[[enlaces]]` de Obsidian funcionen sin rutas. Las notas que ya tienen traducción mantienen su barra de idiomas; las nuevas solo en inglés no la necesitan.
- Identificadores de código y claves de parámetros en inglés; los textos visibles para el usuario llevan siempre `en`; `es` y `de` son opcionales (si faltan se muestra el inglés). Se conservan las traducciones existentes (ADR-0003).

## Propiedad intelectual (clean-room)

- No se lee, descompila, importa ni copia código, binarios, archivos de proyecto o recursos de software de terceros (incluidos programas de simulación comerciales).
- No usar nombres, logotipos ni marcas de terceros en el producto, el código o la interfaz.
- No incorporar modelos 3D ni catálogos de fabricantes sin licencia.
- Revisar la licencia de cada dependencia nueva antes de añadirla.
- Las referencias a otros programas se hacen solo para comparar resultados observables, nunca para reproducir su implementación.

## Flujo de trabajo

1. Leer la especificación del objeto en `docs/<idioma>/specs/` (si no existe, escribirla primero en inglés y pedir confirmación).
2. Proponer un plan corto, sobre todo en tareas de más de un archivo.
3. Escribir la prueba y verla fallar; luego implementar hasta que pase.
4. Ejecutar todas las pruebas, el linter y la comprobación de tipos.
5. Resumir qué cambió y qué resultado se obtuvo, en lenguaje de simulación.
6. Un commit pequeño con mensaje claro por cada paso que funciona; una rama por función.

## Comandos

Creados en la Fase 0 (28-10-2026); todos funcionan.

Plataforma Rust (en `platform/`):

- `cargo build -p intracraft` y `cargo run -p intracraft`: compilar y abrir la aplicación de escritorio con los tres módulos.
- Abrir la aplicación con doble clic: `scripts/intracraft.cmd` (vacía; arrastrar un `.intracraft.json` o un `.bimcraft` encima lo abre), `scripts/intracraft-demo-discrete.cmd` y `scripts/intracraft-demo-physics.cmd` (planta mixta de ejemplo en marcha, 2D + 3D), `scripts/intracraft-viewer-demo.cmd` (ventana 3D Bevy) y `scripts/intracraft-pallet-drop.cmd` (pallets que caen por un final abierto) y `scripts/intracraft-demo-warehouse.cmd` (almacén de gran altura en marcha). Desde una terminal, siempre dentro de `platform/`: `cargo run --release -p intracraft -- RUTA [--module design|discrete|physics] [--view 3d] [--play] [--split]`; una ruta relativa se busca también en las carpetas superiores. El botón «View in 3D» necesita `intracraft-viewer` compilado junto a la aplicación (`cargo build --release -p intracraft -p intracraft-viewer`).
- Ventana 3D sola: `cargo run --release -p intracraft-viewer -- RUTA [--module discrete|physics] [--play] [--speed N]`; `--frames N --screenshot ARCHIVO.png` guarda una imagen de su propia ventana y la cierra (también en `intracraft`, para comprobar vistas sin capturar la pantalla).
- `cargo run -p intracraft-assets`: vuelve a escribir los modelos glTF propios de pallets y cargas (una prueba avisa si están desactualizados).
- Instalaciones de ejemplo: `examples/layouts/*.intracraft.json` (una prueba las ejecuta en 2D y en física con Avian).
- `cargo test --workspace`: todas las pruebas, las de BIMCraft y las nuestras.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --check`: linter y formato.
- `cargo run -p intracraft-cli -- warehouse [--length M] [--height M] [--aisles N] [--deep 1|2] [--station ALTURA:in|out|both]… [--set CLAVE=VALOR]… [--ends] --out ARCHIVO.intracraft.json`: generar un almacén de gran altura (A22); `cargo run -p intracraft-cli -- run LAYOUT.intracraft.json [--seconds N]`: simular un layout en el motor discreto e imprimir avisos y estadísticas.
- `cargo run -p intracraft-cli -- code check|run LAYOUT.intracraft.json [CODIGO.py] [--seconds N]`: comprobar código IntraScript contra un layout o ejecutarlo con él en el motor discreto (imprime la consola del código).
- `cargo test -p intracraft-script`: pruebas del lenguaje (incluye que los ejemplos de A17 y A20 sean válidos).
- `cargo i18n check`: catálogo de traducciones de BIMCraft al día.
- `cargo vendor --versioned-dirs vendor`: tras añadir o actualizar una dependencia; se sube con `git add -f platform/vendor` (algunas librerías traen archivos que un `.gitignore` dejaría fuera y sus sumas de comprobación los incluyen).

Referencia TypeScript:

- `pnpm install`: instalar dependencias.
- `pnpm test`: ejecutar todas las pruebas.
- `pnpm lint`: linter y comprobación de tipos.
- `pnpm dev`: editor en modo desarrollo (http://localhost:5173).
- `pnpm build`: compilación de producción del editor.
- `pnpm run scenario <id|all>`: ejecutar escenarios sin interfaz (hoy T1–T5 analíticos).
- `pnpm format`: formatear el código con Prettier (excluye Markdown).
- `pnpm format:docs`: alinear las tablas Markdown de `docs/`.
- `pnpm assets:fetch`: descargar y verificar los modelos 3D de la librería de objetos en `library-assets/` (fuera de git; catálogo en `packages/asset-catalog`, ver `docs/en/app/A16 Asset Catalog.md`).

## Qué no hacer

- No ampliar el alcance del MVP: curvas, elevadores de tres o más niveles, cajas, cubetas, fallos, energía y OPC UA están fuera hasta nuevo aviso (la ampliación del 30-10-2026 está en ADR-0012; el lenguaje IntraScript, en ADR-0018 y ADR-0027; el almacén de gran altura, en ADR-0033).
- No tocar `test_data/` ni los resultados esperados para que una prueba pase.
- No añadir funciones nuevas a la referencia TypeScript (`packages/`, `apps/`): solo correcciones que expliquen una diferencia con la versión Rust.
- No cambiar archivos de BIMCraft (`platform/crates`, `platform/apps/bimcraft*`) sin anotarlo en `platform/INTRACRAFT-CHANGES.md`.
- No añadir dependencias pesadas sin preguntar.
- No dejar código muerto, comentarios obsoletos ni funciones sin pruebas.
