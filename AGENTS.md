# AGENTS.md

Guía para agentes de código (Claude Code y similares) que trabajan en el monorepo de **IntraCraft**.
`CLAUDE.md` debe apuntar a este archivo (por ejemplo, con el contenido `@AGENTS.md`).

## Resumen del proyecto

IntraCraft es un simulador híbrido de sistemas de transporte de pallets (intralogística) con **un solo layout y dos modos**:

- **Modo 2D discreto (DES):** simulación por eventos, mucho más rápida que el tiempo real. Sirve para rendimiento, acumulación, bloqueos y utilización.
- **Modo 3D con física (tiempo real):** rozamiento de rodillos y cadenas, colisiones con topes y guías laterales, alineación de pallets y sensores con geometría real.

El MVP incluye cinco transportadores: mesa de rodillos, mesa de cadenas, giratoria de cadenas, giratoria de rodillos y mesa de transferencia de rodillos a cadenas, además de sensores fotoeléctricos, topes mecánicos, guías laterales, generador y sumidero de pallets, un controlador por reglas Si/Entonces y una **librería** de componentes parametrizables.

Ampliación del 30-10-2026 (`docs/en/decisions/ADR-0012 Scope Extension Lifts Elevator and Gravity Line.md`): mesa elevadora manual para transpaleta, mesa de tijera, elevador de dos niveles (rodillos o cadenas) y línea de rodillos por gravedad con separador (especificaciones S12–S15).

Ampliación del 31-10-2026 (`docs/en/decisions/ADR-0018 IntraScript Scripting Language.md`, propuesta): lenguaje de programación propio **IntraScript** (`docs/en/app/A17 IntraScript Language.md`) y objetos de información en el plano 2D: variable, tabla, lista, bloque de código y zona (especificaciones S18–S22). No se escribe código del lenguaje hasta que la responsable confirme esas notas.

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
└─ test_data/             (resultados guardados y datos de prueba)
```

Si la estructura real difiere, actualizar este archivo en el mismo cambio.

## Stack

- TypeScript en modo estricto en todo el repositorio; pnpm (espacios de trabajo); Vite; Vitest; ESLint y Prettier.
- Esquemas y validación: Zod.
- Interfaz: React, react-three-fiber (Three.js) y Zustand.
- Física: Rapier o Jolt (WASM), a decidir en el spike S1 (ver «Física» y `docs/en/decisions/ADR-0002 Physics Engine Candidate.md`; candidato preferido: Jolt). Todo acceso al motor pasa por `sim-physics`.
- Versiones fijadas y motivos: `docs/en/decisions/ADR-0001 Toolchain and Versions.md` (TypeScript 6.0 porque typescript-eslint aún no admite la 7).
- Los motores de simulación corren en un Web Worker; la interfaz solo recibe instantáneas de estado.

## Convenciones del modelo

- **Unidades internas SI:** metros, kilogramos, segundos, radianes. La interfaz muestra mm y grados. Convertir solo en los bordes (interfaz y archivos).
- **Ejes:** Y hacia arriba, sistema de mano derecha. La vista 2D es la vista cenital (mirando hacia −Y).
- **Tiempo:** enteros en microsegundos. La cola de eventos se ordena por (tiempo, prioridad, secuencia).
- **Aleatoriedad:** siempre con el generador con semilla del proyecto. Nunca `Math.random()` dentro de los motores.
- **Determinismo:** mismo layout y misma semilla deben producir el mismo resultado. Si una prueba de determinismo falla, es un error de prioridad máxima.
- **Archivos de proyecto:** JSON (`.intracraft.json`) con `schemaVersion` y migraciones explícitas. Los resultados de simulación no se guardan dentro del proyecto.

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

- No ampliar el alcance del MVP: curvas, elevadores de tres o más niveles, cajas, cubetas, fallos, energía y OPC UA están fuera hasta nuevo aviso (la ampliación del 30-10-2026 está en ADR-0012; el lenguaje IntraScript, en ADR-0018).
- No tocar `test_data/` ni los resultados esperados para que una prueba pase.
- No añadir dependencias pesadas sin preguntar.
- No dejar código muerto, comentarios obsoletos ni funciones sin pruebas.
