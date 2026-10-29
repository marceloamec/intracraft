---
title: A08 Controlador
lang: es
tags: [app, control]
---

🌐 [[A08 Controller|EN]] · **ES** · [[A08 Steuerung|DE]] · [[Inicio (ES)|Índice]]

# A08 Controlador (`packages/control`)

Teoría: [[T07 Lógica de control y señales]].

## 1. Qué existe hoy

`rules.ts` — el modelo de reglas Si/Entonces y un evaluador **puro**:

```ts
type Condition = { signal: string; is: boolean } | { signal: string; edge: 'rising' | 'falling' };
interface Rule { id: string; when: Condition[]; then: { signal: string; value: boolean; delayMs?: number }[] }
evaluateRules(rules, previous, current) → SignalWrite[]   // { signal, value, delayMs, ruleId }
```

- Todas las condiciones de una regla se combinan con **Y**. (O = varias reglas.)
- Las señales que faltan se leen como `false`.
- El evaluador nunca lee un reloj ni toca la planta.

Probado con una plantilla ZPA de dos zonas y casos de flancos y retardos (`rules.test.ts`).

## 2. Cómo lo usa cada modo

| Paso                    | 2D (DES)                                                       | 3D (física)                                  |
| ----------------------- | -------------------------------------------------------------- | -------------------------------------------- |
| Cuándo evaluar          | En cada evento `signal_changed`                                | En cada ciclo de control (10 ms por defecto) |
| Instantánea `previous`  | Valores antes del cambio                                       | Valores del ciclo anterior                   |
| Aplicar una escritura   | Al instante, o programar `signal_changed` en ahora + retardo   | En el primer ciclo ≥ ahora + retardo         |
| Conflictos de escritura | Gana la última regla según el orden de la lista (determinista) | Igual                                        |

## 3. Ampliaciones previstas

- Editor de reglas en la interfaz con plantillas (ZPA, tope con sensor, giratoria que espera pallet centrado, secuencia completa de transferencia).
- Semántica de cancelación de escrituras retardadas (temporizador a la conexión frente a pulso): especificar antes de la fase 3.
- Registro de señales con la convención `<instancia>.<señal>` y validación de que existe toda señal referenciada.
- Los enclavamientos **no** son reglas: viven dentro de los comportamientos de los componentes y registran un aviso cuando bloquean una orden.
- El scripting de usuario (en sandbox) es post-MVP.

## Relacionado

[[T07 Lógica de control y señales]] · [[A06 Motor discreto]] · [[A07 Envoltorio del motor físico]]
