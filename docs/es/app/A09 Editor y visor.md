---
title: A09 Editor y visor
lang: es
tags: [app, editor, ui]
---

🌐 [[A09 Editor and Viewer|EN]] · **ES** · [[A09 Editor und Viewer|DE]] · [[Inicio (ES)|Índice]]

# A09 Editor y visor (`apps/editor`)

## 1. Qué existe hoy

Se arranca con `pnpm dev` → http://localhost:5173.

| Archivo                                 | Función                                                                                                                                                                                                                                      |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/App.tsx`                           | Distribución: barra de herramientas + visor                                                                                                                                                                                                  |
| `src/store.ts`                          | Estado de interfaz con Zustand: idioma (en/es/de), vista (perspectiva/cenital), modo de simulación (des/physics)                                                                                                                             |
| `src/i18n/strings.ts`, `useT.ts`        | Textos de la interfaz en inglés, español y alemán (probado: mismas claves en los tres)                                                                                                                                                       |
| `src/components/Toolbar.tsx`            | Cambio de vista (tecla **V**), selector de modo y de idioma                                                                                                                                                                                  |
| `src/components/Viewport.tsx`           | `Canvas` de react-three-fiber; cámara en perspectiva o vista cenital ortográfica (mirando hacia −Y); controles orbitales (sin giro en vista cenital)                                                                                         |
| `src/components/RollerConveyorMesh.tsx` | Mesa de rodillos paramétrica con los valores por defecto de la librería: rodillos en un único `InstancedMesh` colocados con `rollerLayout`, bastidores laterales y un pallet EUR animado con la cinemática 2D (acelera y mantiene velocidad) |

El pallet animado es un anticipo de la «animación 3D de la simulación 2D»: su posición sale de `positionAt` y `timeToReach` de `sim-des`, no de un motor de física.

## 2. Funciones del MVP (por construir)

- **Vistas:** 3D en perspectiva y 2D cenital (ortográfica), intercambiables con una tecla; órbita, zoom y encuadre.
- **Edición:** arrastrar componentes desde la librería, mover y girar con gizmos, rejilla de 100 mm con ajuste a puertos y a la rejilla, copiar/pegar, repetir en matriz, deshacer/rehacer.
- **Conexión:** encaje automático de puertos con avisos de altura, ancho y orientación.
- **Propiedades:** panel generado desde el esquema de la definición (grupos, unidades, rangos, derivados, avisos de validación) — [[A05 Librería de componentes]].
- **Árbol del modelo:** lista de objetos con búsqueda, ocultar y bloquear.
- **Ejecución:** reproducir, pausar, paso a paso, velocidades ×1, ×10, ×100 (+ máximo en 2D); reloj de simulación; selector de modo 2D/3D.
- **Visualización en vivo:** haces en verde/rojo, topes arriba/abajo, pallets coloreados por estado, panel de señales y de estadísticas.
- **Controlador:** editor de reglas Si/Entonces con plantillas.
- **Archivo:** guardar/abrir `.intracraft.json`, exportar estadísticas CSV/JSON, capturas.
- **Herramientas:** medir distancias, capas visibles (guías, sensores, cotas).

Pospuesto: plano de fondo (imagen/DXF), grabación de vídeo, informes, comparación de ejecuciones.

## 3. Reglas de dibujo

- Rodillos y demás piezas repetidas usan `InstancedMesh` (una llamada de dibujo por tipo).
- La simulación corre en un Web Worker; el visor interpola las instantáneas ([[A02 Arquitectura]]).
- La interfaz muestra mm y grados; todo lo guardado está en SI ([[A13 Convenciones]]).
- El visor nunca decide resultados de simulación; solo muestra instantáneas.

## Relacionado

[[A02 Arquitectura]] · [[A05 Librería de componentes]] · [[A13 Convenciones]]
