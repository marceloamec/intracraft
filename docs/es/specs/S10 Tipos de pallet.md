---
title: S10 Tipos de pallet
lang: es
tags: [spec, pallet, reference-data]
library-id: pallet.*
status: reference-json
updated: 2026-10-29
---

🌐 [[S10 Pallet Types|EN]] · **ES** · [[S10 Palettentypen|DE]] · [[Inicio (ES)|Índice]]

# S10 Tipos de pallet y pares de fricción

Datos: `packages/library/src/definitions/pallet-types.json` y `friction-pairs.json` (se validan al cargar y tienen pruebas). **El proyecto trabaja con pallets EUR** (responsable, 29-10-2026): longitudinal en rodillos, transversal en cadenas. Los valores son **orientativos**; contrastarlos con la norma o con pallets reales antes de confiar en los resultados de los sensores.

## 1. Tipos de pallet

| Tipo          | L × A (mm)  | Altura (mm) | Tara (kg) | Estado del perfil | Notas                                     |
| ------------- | ----------- | ----------- | --------- | ----------------- | ----------------------------------------- |
| EUR / EPAL    | 1200 × 800  | 144         | 25        | orientativo       | Tipo principal del proyecto               |
| 1200 × 1000   | 1200 × 1000 | ≈ 150       | 28        | aproximado        | Perfil transversal simplificado           |
| GMA           | 1219 × 1016 | ≈ 140       | 25        | aproximado        | Pallet de largueros; perfil provisional   |
| Personalizado | libre       | libre       | libre     | usuario           | Perfil propio en el editor (más adelante) |

Una instancia de pallet tiene además masa y altura de carga, `centerOfMassOffset` y atributos personalizados clave–valor.

## 2. Construcción EUR usada para los perfiles

| Capa (altura, mm)           | Piezas                                                                                                                  |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Tablas inferiores (0–22)    | 3 tablas de 1200 mm, repartidas en el lado de 800 mm en 0–100, 327,5–472,5 y 700–800 mm (anchos 100 / 145 / 100)        |
| Tacos (22–100)              | 9 tacos; en el lado de 1200 mm en 0–145, 527,5–672,5 y 1055–1200 mm; en el de 800 mm en 0–100, 327,5–472,5 y 700–800 mm |
| Tablas de unión (100–122)   | 3 tablas de 800 mm y 145 mm de ancho sobre las filas de tacos                                                           |
| Tablas superiores (122–144) | 5 tablas de 1200 mm (anchos 145 / 100 / 145 / 100 / 145) con huecos de ≈ 41 mm                                          |

## 3. Perfiles de ocupación (lo que ve un haz horizontal perpendicular al avance)

Intervalos macizos medidos desde el frente del pallet, en m.

**Lado largo delante — `alongLength` (mesas de rodillos):**

| Banda (mm) | Intervalos macizos                         | Comportamiento del haz           |
| ---------- | ------------------------------------------ | -------------------------------- |
| 0–22       | [0, 1,2]                                   | Continuo                         |
| 22–100     | [0, 0,145], [0,5275, 0,6725], [1,055, 1,2] | Huecos de 382,5 mm               |
| 100–122    | [0, 0,145], [0,5275, 0,6725], [1,055, 1,2] | Huecos de 382,5 mm               |
| 122–144    | [0, 1,2]                                   | **Continuo** (tablas superiores) |

**Lado corto delante — `alongWidth` (mesas de cadenas):**

| Banda (mm) | Intervalos macizos                                                                 | Comportamiento del haz         |
| ---------- | ---------------------------------------------------------------------------------- | ------------------------------ |
| 0–22       | [0, 0,1], [0,3275, 0,4725], [0,7, 0,8]                                             | Huecos de 227,5 mm             |
| 22–100     | [0, 0,1], [0,3275, 0,4725], [0,7, 0,8]                                             | Huecos de 227,5 mm             |
| 100–122    | [0, 0,8]                                                                           | **Continuo** (tablas de unión) |
| 122–144    | [0, 0,145], [0,18625, 0,28625], [0,3275, 0,4725], [0,51375, 0,61375], [0,655, 0,8] | Huecos de ≈ 41 mm              |

Consecuencia: la mejor altura del haz depende de la orientación: ≈ 130 mm en rodillos y ≈ 110 mm en cadenas ([[S06 Fotocélula]]). Los colisionadores 3D se generan a partir de la misma construcción.

## 4. Pares de fricción

| Par                      | μ inicial | Rango habitual | Id en el JSON       |
| ------------------------ | --------- | -------------- | ------------------- |
| Madera – acero (rodillo) | 0,35      | 0,25 – 0,50    | `wood-steel-roller` |
| Madera – cadena de acero | 0,30      | 0,20 – 0,45    | `wood-steel-chain`  |
| Plástico – acero         | 0,25      | 0,15 – 0,40    | `plastic-steel`     |
| Madera – guía de acero   | 0,30      | 0,20 – 0,45    | `wood-steel-guide`  |

## Relacionado

[[T06 Detección fotoeléctrica]] · [[S06 Fotocélula]] · [[T11 Contacto, rozamiento y restitución]] · [[S09 Generador y sumidero]] · [[Preguntas abiertas]]
