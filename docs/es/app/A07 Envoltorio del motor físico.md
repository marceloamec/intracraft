---
title: A07 Envoltorio del motor físico
lang: es
tags: [app, physics, sim-physics]
---

🌐 [[A07 Physics Engine Wrapper|EN]] · **ES** · [[A07 Physik-Wrapper|DE]] · [[Inicio (ES)|Índice]]

# A07 Envoltorio del motor físico (`packages/sim-physics`)

Teoría: de [[T09 Dinámica del sólido rígido]] a [[T15 Investigación de motores de física]].

## 1. Por qué un envoltorio

Todo acceso al motor de física pasa por este paquete. Nadie más puede importar Jolt o Rapier. Ventajas: el motor puede cambiarse tras el spike S1 (o más adelante) sin tocar los componentes; las versiones se fijan en un solo sitio; las rarezas del motor (cuerpos dormidos, holgura, umbral de restitución) se resuelven una sola vez.

## 2. Qué existe hoy

| Módulo          | Contenido                                                                                                                                                                                                                                                                | Pruebas      |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ |
| `backend.ts`    | Interfaz `PhysicsBackend`: `addBody`, `removeBody`, `setKinematicVelocity`, `setSurfaceVelocity` (devuelve false si no se admite), `getPose`, `castRay`, `step`, `stateHash`; `BodyDesc` (estático/cinemático/dinámico, formas, masa, CDM, rozamiento, restitución, CCD) | — (interfaz) |
| `fixed-step.ts` | `FixedStepClock` (acumulador, máximo de pasos por fotograma, `alpha` de interpolación), `DEFAULT_PHYSICS_DT` = 1/120 s                                                                                                                                                   | ✓            |
| `mechanics.ts`  | `rollerAngularVelocity`, `maxTractionAccel`, `turntableRequiredFriction`, `turntableRequiredTorque`, `sweptRadius`, `boxYawInertia`                                                                                                                                      | ✓            |

## 3. Construcción del mundo prevista (fase 4)

| Objeto de la librería | Cuerpos físicos                                                                                                |
| --------------------- | -------------------------------------------------------------------------------------------------------------- |
| Pallet                | Compuesto dinámico: 3 cajas de tablas inferiores + caja de carga; masa y CDM explícitos; CCD activado          |
| Mesa de rodillos      | Un cilindro cinemático por rodillo, agrupados por zona; ω fijada en cada paso desde el perfil de la zona       |
| Mesa de cadenas       | Plan B: una caja estática por ramal + superficie móvil (Jolt) · Plan A: tiras de rodillos cinemáticos pequeños |
| Giratoria             | Plataforma cinemática + hijos cinemáticos (rodillos/cadenas) colocados según el ángulo                         |
| Transferencia         | Rodillos + ramales de cadena cinemáticos que se mueven en vertical y luego en horizontal                       |
| Tope                  | Caja cinemática que se mueve entre las posiciones abajo/arriba                                                 |
| Guía / embudo         | Cajas estáticas                                                                                                |
| Fotocélula            | Raycast (o shape-cast) en cada paso; instante del flanco = punto medio del paso                                |

## 4. Bucle de pasos

```text
por cada paso fijo (dt = 1/120 s):
    si se cruzó una frontera de control de 10 ms: ejecutar controlador, aplicar escrituras
    por cada componente: actualizar objetivos cinemáticos (ω de rodillos, altura de topes, ángulo de plataforma…)
    despertar pallets en contacto con transportadores cuya velocidad cambió   # T10 §7
    backend.step(dt)
    sensores: raycast, detectar flancos, marcar tiempo
    registrar impactos (velocidad, energía), estadísticas
cada N pasos: hash del estado (prueba de determinismo)
publicar instantánea a la interfaz
```

## 5. Ajustes del motor que hay que cambiar (según los valores medidos)

| Ajuste                          | Por defecto (motor)              | Valor previsto       | Motivo                                   |
| ------------------------------- | -------------------------------- | -------------------- | ---------------------------------------- |
| Paso de tiempo                  | 1/60 s (Rapier)                  | 1/120 s              | [[T10 Integración temporal y paso fijo]] |
| Holgura / error permitido       | 0,02 m (Jolt) / 0,005 m (Rapier) | ≈ 0,001 m            | Criterio de ±5 mm en el tope             |
| Velocidad mínima de restitución | 1 m/s (Jolt)                     | ≈ 0,01 m/s           | Impactos a 0,1–0,25 m/s                  |
| Combinación de rozamiento       | media geométrica / media         | valor exacto del par | Tabla de fricción                        |

Cualquier cambio de estos valores se registra en `docs/*/decisions/` (regla de AGENTS.md).

## Relacionado

[[ADR-0002 Candidato a motor de física]] · [[T12 Rodillos y cadenas en física]] · [[A02 Arquitectura]] · [[A14 Referencia de API]]
