---
title: T15 Investigación de motores de física
lang: es
tags: [theory, physics, research]
date: 2026-10-28
---

🌐 [[T15 Physics Engine Research|EN]] · **ES** · [[T15 Recherche Physik-Engines|DE]] · [[Inicio (ES)|Índice]]

# T15 Investigación de motores de física

Investigación realizada el **28-10-2026** para preparar el spike S1. Método: se descargaron los paquetes npm (sin instalarlos en el proyecto), se revisaron sus tipos TypeScript y se leyeron los ajustes por defecto creando cada motor en Node.js 24. No se ha copiado nada de estos paquetes en IntraCraft.

## 1. Candidatos y versiones

| Motor  | Paquete npm                               | Versión revisada | Licencia   | Lenguaje / compilación                               |
| ------ | ----------------------------------------- | ---------------- | ---------- | ---------------------------------------------------- |
| Rapier | `@dimforge/rapier3d-compat`               | 0.21.0           | Apache-2.0 | Rust → WASM (incrustado en base64)                   |
| Rapier | `@dimforge/rapier3d-deterministic-compat` | 0.21.0           | Apache-2.0 | Igual, compilación determinista multiplataforma      |
| Jolt   | `jolt-physics`                            | 1.1.1            | MIT        | C++ → WASM (Emscripten), también versiones multihilo |

## 2. La pregunta decisiva: superficie móvil en JavaScript

| Capacidad                          | Rapier JS 0.21.0                                                            | Jolt JS 1.1.1                                                                           |
| ---------------------------------- | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| Hook para modificar contactos      | ❌ `PhysicsHooks` solo tiene `filterContactPair` y `filterIntersectionPair` | ✅ `ContactListenerJS.OnContactAdded / OnContactPersisted(…, ioSettings)`               |
| Velocidad superficial por contacto | ❌ (el hook de Rust `modify_solver_contacts` no está expuesto)              | ✅ `ContactSettings.mRelativeLinearSurfaceVelocity` y `mRelativeAngularSurfaceVelocity` |
| Rozamiento por contacto            | ❌ (solo reglas de combinación: Average, Min, Multiply, Max)                | ✅ `ContactSettings.mCombinedFriction`, `mCombinedRestitution`                          |
| Filtrado de contactos              | ✅                                                                          | ✅ `OnContactValidate`                                                                  |
| Plan B para cadenas                | No es posible → plan A o D                                                  | **Viable** (a confirmar en S1)                                                          |

**Puntos abiertos para S1 con Jolt:** convenio de signo/marco de la velocidad superficial relativa (cuerpo 1 frente a cuerpo 2); coste de una llamada a JS por contacto y paso (cientos de contactos × 120 Hz); si el orden de las llamadas es determinista; gestión de memoria de los objetos de Emscripten (`Jolt.destroy`).

## 3. Ajustes por defecto medidos

| Ajuste                             | Rapier 0.21.0                            | Jolt 1.1.1                             | Por qué importa                                  |
| ---------------------------------- | ---------------------------------------- | -------------------------------------- | ------------------------------------------------ |
| Paso de tiempo por defecto         | `dt` = 1/60 s                            | se fija en cada `Update`               | IntraCraft usa 1/120 s                           |
| Iteraciones del solver             | `numSolverIterations` = 4                | 10 de velocidad + 2 de posición        | Pallets pesados sobre muchos rodillos (S1-a)     |
| Penetración permitida              | `normalizedAllowedLinearError` = 0,005 m | `mPenetrationSlop` = 0,02 m            | Criterio de ±5 mm en tope (P1) → hay que bajarla |
| Distancia de contacto especulativo | —                                        | `mSpeculativeContactDistance` = 0,02 m | Detección anticipada de contactos                |
| Umbral de restitución              | —                                        | `mMinVelocityForRestitution` = 1 m/s   | Impactos a 0,1–0,25 m/s no rebotarían            |
| Rozamiento por defecto / regla     | 0,5 / Average                            | media geométrica (valor documentado)   | Valores exactos de la tabla por par              |

## 4. Determinismo

- **Rapier** publica una compilación `-deterministic` que garantiza el determinismo **entre plataformas** (más lenta). La compilación estándar solo es determinista en la misma máquina.
- **Jolt** admite simulación determinista si los cuerpos se añaden en el mismo orden y se hacen las mismas llamadas; el determinismo **entre plataformas** exige compilar con `CROSS_PLATFORM_DETERMINISTIC=ON` (el README de `jolt-physics` documenta la opción). Hay que comprobar en S1 si la versión publicada en npm lo tiene activado; si no, solo se garantiza en la misma máquina.
- La prueba S1-g de IntraCraft (mismo hash tras 60 s) solo requiere determinismo en la misma máquina para el MVP.

## 5. Otras observaciones

- Ambos funcionan dentro de un Web Worker (el modelo de ejecución del plan).
- Jolt ofrece versiones WASM multihilo (necesitan `SharedArrayBuffer` → cabeceras de aislamiento de origen). No hacen falta para el objetivo del MVP (20 transportadores, 10 pallets).
- Rapier tiene una integración madura con React (`@react-three/rapier`), pero IntraCraft ejecuta la física en un worker tras su propio envoltorio, así que esa ventaja no aplica.
- three.js incluye adaptadores de ejemplo para ambos motores (`examples/jsm/physics/`), útiles solo como lectura.

## 6. Conclusión preliminar

Jolt es el **candidato preferido** para S1 porque es el único cuyo binding JS expone velocidad superficial y rozamiento por contacto, lo que permite las mesas de cadenas (plan B) y los pares de fricción exactos sin cuerpos extra. Rapier queda como alternativa con el plan A. La decisión queda registrada como *Propuesta* en [[ADR-0002 Candidato a motor de física]] y solo se cierra cuando S1 se supere.

## Fuentes

- npm: `@dimforge/rapier3d-compat` 0.21.0 (`dist/pipeline/physics_hooks.d.ts`, `package.json`), `jolt-physics` 1.1.1 (`dist/types.d.ts`, `README.md`, `package.json`).
- [Interfaz `PhysicsHooks` de Rapier JS](https://rapier.rs/javascript3d/interfaces/PhysicsHooks.html) · [Hooks activos de colisionador en Rapier (guía JS)](https://rapier.rs/docs/user_guides/javascript/collider_active_hooks/)
- [`ContactSettings` de Jolt (documentación C++)](https://jrouwe.github.io/JoltPhysics/class_contact_settings.html) · [Simulación determinista en Jolt](https://jrouwe.github.io/JoltPhysics/#deterministic-simulation)
- [README de la compilación determinista de Rapier](https://cdn.jsdelivr.net/npm/@dimforge/rapier3d-deterministic@0.19.3/README.md)

## Relacionado

[[T12 Rodillos y cadenas en física]] · [[T11 Contacto, rozamiento y restitución]] · [[ADR-0002 Candidato a motor de física]] · [[A07 Envoltorio del motor físico]]
