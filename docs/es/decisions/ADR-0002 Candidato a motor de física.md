---
title: ADR-0002 Candidato a motor de física
lang: es
tags: [decision, physics]
status: proposed
date: 2026-10-28
---

🌐 [[ADR-0002 Physics Engine Candidate|EN]] · **ES** · [[ADR-0002 Physik-Engine-Kandidat|DE]] · [[Inicio (ES)|Índice]]

# ADR-0002 Candidato a motor de física

- **Estado:** Propuesta — se cierra tras el spike S1
- **Fecha:** 28-10-2026

## Contexto

El mayor riesgo técnico del MVP es la **mesa de cadenas** en 3D: necesita un contacto con «superficie móvil» o muchos cuerpos pequeños en movimiento ([[T12 Rodillos y cadenas en física]]). El plan propone Rapier (Apache-2.0) y Jolt (MIT) y pide no suponer que la superficie móvil existe en los bindings JS.

## Hallazgos (ver [[T15 Investigación de motores de física]])

- `jolt-physics` 1.1.1: expone `ContactListenerJS` + `ContactSettings.mRelativeLinearSurfaceVelocity` y `mCombinedFriction` → posible el plan B y pares de fricción exactos.
- `@dimforge/rapier3d-compat` 0.21.0: `PhysicsHooks` solo filtra pares; no permite modificar contactos del solver → solo plan A.
- Valores por defecto medidos que habrá que cambiar en cualquiera de los dos: holgura de penetración / error permitido (0,02 m en Jolt, 0,005 m en Rapier) y umbral de restitución de Jolt de 1 m/s.

## Decisión (propuesta)

Hacer el spike S1 **primero con Jolt**, con el plan B para las cadenas. Volver a **Rapier con el plan A** si Jolt no cumple los criterios de S1 (rendimiento de las llamadas de contacto en JS, determinismo o estabilidad). Los planes C (fuerza de arrastre) y D (módulo propio en Rust) quedan como último recurso.

## Criterios de aprobación del spike S1

| #   | Prueba                                          | Criterio                                                        |
| --- | ----------------------------------------------- | --------------------------------------------------------------- |
| a   | Pallet de 1000 kg sobre 10 rodillos cinemáticos | Velocidad nominal ±3 %; frena sin deslizamiento excesivo        |
| b   | Cadena (plan B, y plan A para comparar)         | Velocidad del pallet = de la cadena ±3 %, sin vibración visible |
| c   | Pallet contra un tope a 0,25 m/s                | Frente a ±5 mm; rebote < 10 mm                                  |
| d   | Guía con embudo                                 | Desalineación de 50 mm y 5° → dentro de ±10 mm y ±1°            |
| e   | Giro de 90° sobre plataforma                    | El pallet se desplaza < 5 mm respecto a la plataforma           |
| f   | Rendimiento                                     | 20 transportadores y 10 pallets a 60 FPS, sin inestabilidades   |
| g   | Determinismo                                    | Mismo hash de estado tras 60 s en dos ejecuciones               |

## Consecuencias

- Todo acceso al motor queda tras `PhysicsBackend` ([[A07 Envoltorio del motor físico]]), así que cambiar de motor cuesta un adaptador, no reescribir.
- Tras S1 este ADR pasa a *Aceptada* (o *Rechazada* con la alternativa) y registra resultados medidos, limitaciones y el plan para las cadenas.
