---
title: T10 Integración temporal y paso fijo
lang: es
tags: [theory, physics, numerics]
---

🌐 [[T10 Time Integration and Fixed Step|EN]] · **ES** · [[T10 Zeitintegration und fester Zeitschritt|DE]] · [[Inicio (ES)|Índice]]

# T10 Integración temporal y paso fijo

## 1. De las ecuaciones a los pasos

El motor de física no puede resolver de forma cerrada las ecuaciones de Newton–Euler ([[T09 Dinámica del sólido rígido]]) en cuanto hay contactos. Avanza el estado en pasos pequeños $\Delta t$.

| Integrador              | Actualización                                                      | Comportamiento                                               |
| ----------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------ |
| Euler explícito         | $x \mathrel{+}= v\,\Delta t$; después $v \mathrel{+}= a\,\Delta t$ | Gana energía; los osciladores explotan                       |
| **Euler semiimplícito** | $v \mathrel{+}= a\,\Delta t$; después $x \mathrel{+}= v\,\Delta t$ | Simpléctico: la energía queda acotada; lo usan Rapier y Jolt |
| Runge–Kutta 4           | Cuatro evaluaciones por paso                                       | Preciso en EDO suaves, inútil con contactos                  |

Euler semiimplícito es de primer orden pero **estable** para un oscilador armónico mientras $\omega_n\,\Delta t < 2$, y con contactos la estabilidad importa mucho más que el orden.

## 2. Por qué el paso debe ser fijo

1. **Determinismo** — otro $\Delta t$ da otros impulsos de contacto y, por tanto, otros resultados. Un paso ligado a los fotogramas haría única cada ejecución.
2. **Estabilidad** — los parámetros del solver (rigidez, tratamiento de la restitución) se ajustan para un $\Delta t$ concreto.
3. **Pruebas reproducibles** — S1-g exige el mismo hash de estado tras 60 s en dos ejecuciones.

## 3. El patrón del acumulador

La pantalla se refresca a su propio ritmo (60 Hz, 144 Hz o de forma irregular). La simulación se desacopla de ella:

```text
acumulador += tiempoFotograma
pasos = floor(acumulador / dt)
si pasos > maxPasos:              # protección «espiral de la muerte»
    pasos = maxPasos; acumulador = 0
si no:
    acumulador -= pasos · dt
repetir pasos veces: mundo.step(dt)
alfa = acumulador / dt            # 0 ≤ alfa < 1
estado dibujado = lerp(anterior, actual, alfa)   # slerp para giros
```

Implementado como `FixedStepClock` en `packages/sim-physics/src/fixed-step.ts` (con pruebas). La **interpolación** elimina los tirones que aparecen cuando el número de pasos por fotograma alterna (p. ej. 2, 2, 3, 2…). El dibujo va como mucho un paso por detrás de la física: imperceptible a 120 Hz.

Si el ordenador no da abasto, la protección descarta el retraso acumulado: el tiempo simulado avanza más despacio que el real, pero sigue siendo correcto.

## 4. Por qué $\Delta t = 1/120$ s

Regla práctica: lo que se desplaza un cuerpo en un paso debe ser mucho menor que el detalle geométrico más pequeño con el que interactúa.

| Velocidad               | Desplazamiento por paso | Comparar con                                            |
| ----------------------- | ----------------------- | ------------------------------------------------------- |
| 0,10 m/s (aproximación) | 0,83 mm                 | Tolerancia en tope ±5 mm                                |
| 0,25 m/s                | 2,1 mm                  | Tabla inferior de 22 mm, cadena de 40 mm                |
| 0,60 m/s (máx.)         | 5,0 mm                  | Hueco entre rodillos de 57 mm, cara del tope — usar CCD |

A 0,25 m/s un pallet cruza un paso de rodillos (133 mm) en 64 pasos de física: el patrón de contactos cambia de forma suave. Una tabla que se acerca a un tope avanza 0,83–2,1 mm por paso, coherente con el criterio de ±5 mm de S1-c.

## 5. Iteraciones del solver y subpasos

En cada paso el solver de restricciones recorre todos los contactos varias veces ([[T11 Contacto, rozamiento y restitución]]):

- **Jolt**: iteraciones de velocidad (10 por defecto) y de posición (2 por defecto) por paso.
- **Rapier**: solver iterativo con subpasos internos (estilo TGS-soft) y número de iteraciones configurable.

Más iteraciones → contactos más rígidos y precisos (pallets pesados sobre muchos rodillos) con coste lineal. El spike S1 mide cuántas hacen falta para la prueba a (1000 kg sobre 10 rodillos).

## 6. Ciclo del controlador frente a paso de física

El controlador se ejecuta cada 10 ms; la física, cada 8,33 ms. No son múltiplos, así que el controlador se ejecuta en el primer paso tras cada frontera de 10 ms, con hasta 8,3 ms de desfase. Opciones (decisión abierta del S1):

1. Aceptar el desfase (lo cubre el presupuesto de ±50 ms de los sensores, [[T06 Detección fotoeléctrica]]).
2. Usar $\Delta t = 1/100$ s o $1/200$ s para que los ciclos de control coincidan con pasos.

## 7. Cuerpos dormidos: una trampa clásica

Los motores **duermen** los cuerpos en reposo para ahorrar cálculo. Un pallet que espera sobre un transportador parado se dormirá. Cuando el transportador arranca, los rodillos cinemáticos cambian de velocidad, pero un pallet dormido no siempre se despierta. **El envoltorio debe despertar todos los pallets en contacto con un transportador cuya velocidad cambia.** Si se olvida, aparecen pallets que «se niegan» a moverse.

## Relacionado

[[T09 Dinámica del sólido rígido]] · [[T11 Contacto, rozamiento y restitución]] · [[T03 Tiempo, orden de eventos y determinismo]] · [[A07 Envoltorio del motor físico]]
