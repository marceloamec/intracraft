---
title: T04 Cinemática de transportadores
lang: es
tags: [theory, des, kinematics]
---

🌐 [[T04 Conveyor Kinematics|EN]] · **ES** · [[T04 Kinematik von Förderern|DE]] · [[Inicio (ES)|Índice]]

# T04 Cinemática de transportadores

En el modo 2D un pallet nunca «siente» fuerzas: sigue **perfiles de movimiento** impuestos. Esta nota deduce todas las fórmulas que usa el motor DES. Código: `packages/sim-des/src/kinematics.ts`.

## 1. Movimientos elementales

Con aceleración constante $a$, desde $t_0$, con posición $s_0$ y velocidad $v_0$:

$$v(t) = v_0 + a\,(t - t_0) \qquad s(t) = s_0 + v_0 (t - t_0) + \tfrac12 a (t - t_0)^2$$

Dos identidades útiles:

- Distancia para pasar de $v_1$ a $v_2$: $\;d = \dfrac{v_2^2 - v_1^2}{2a}$
- Tiempo para pasar de $v_1$ a $v_2$: $\;t = \dfrac{v_2 - v_1}{a}$

Un movimiento completo es una secuencia **por tramos** de segmentos `{t0, s0, v0, a}`; el motor solo guarda el segmento actual de cada pallet.

## 2. Perfil trapezoidal (arranca y termina en reposo)

Acelera con $a$ hasta $v_{max}$, mantiene la velocidad y decelera con $b$ hasta parar en una distancia $D$.

```text
 v
 │      ┌───────────────┐   v_max
 │     /                 \
 │    /                   \
 │   /                     \
 └──┴───────────────────────┴──── t
     t_a      t_c          t_b
```

- Distancias de rampa: $d_a = \dfrac{v_{max}^2}{2a}$, $\;d_b = \dfrac{v_{max}^2}{2b}$
- Si $D \ge d_a + d_b$ (se alcanza la velocidad de crucero):

$$t = \frac{v_{max}}{a} + \frac{D - d_a - d_b}{v_{max}} + \frac{v_{max}}{b} = \frac{D}{v_{max}} + \frac{v_{max}}{2a} + \frac{v_{max}}{2b}$$

- Si no, el perfil es **triangular**: la velocidad pico es $v_p = \sqrt{\dfrac{2 D a b}{a + b}}$ y $t = \dfrac{v_p}{a} + \dfrac{v_p}{b}$. Con $a = b$: $t = 2\sqrt{D/a}$.

## 3. Llegar a velocidad de crucero (sin frenar) — prueba T1

Un pallet que sale del transportador a plena velocidad (entrega al siguiente) no tiene fase de frenado:

$$t = \frac{v}{a} + \frac{D - v^2/2a}{v} = \frac{D}{v} + \frac{v}{2a} \qquad (D \ge v^2/2a)$$

**T1:** $D = 3$ m, $v = 0{,}25$ m/s, $a = 0{,}5$ m/s² → $t = 12 + 0{,}25 = \mathbf{12{,}25\ s}$. La rampa cuesta exactamente $v/2a = 0{,}25$ s frente a un arranque instantáneo. Si $D < v^2/2a$: $t = \sqrt{2D/a}$.

## 4. Los giros usan las mismas fórmulas — prueba T3

Se cambia distancia por ángulo $\theta$, velocidad por $\omega$ y aceleración por $\alpha$. Con $\alpha$ en las dos rampas:

- Triangular si $\omega^2/\alpha \ge \theta$: $\;t = 2\sqrt{\theta/\alpha}$
- Trapezoidal en otro caso: $\;t = \theta/\omega + \omega/\alpha$

**T3:** $\theta = 90°$, $\omega = 15$ °/s, $\alpha = 30$ °/s² → $\omega^2/\alpha = 7{,}5° < 90°$, así que $t = 6 + 0{,}5 = \mathbf{6{,}5\ s}$. (Las unidades se cancelan y se puede trabajar en grados; el código trabaja en radianes.)

## 5. Ciclo de la mesa de transferencia — prueba T4

$$t = t_{subida} + \left(\frac{d}{v} + \frac{v}{a}\right) + t_{bajada} \;(+\ t_{asentamiento})$$

**T4:** $d = 1$ m, $v = 0{,}2$ m/s, $a = 0{,}5$ m/s², 1 s de subida y 1 s de bajada → $1 + (5 + 0{,}4) + 1 = \mathbf{7{,}4\ s}$. Comprobación: $v^2/a = 0{,}08$ m < 1 m, luego la cadena sí alcanza la velocidad de crucero.

## 6. Velocidad de aproximación antes de un tope

Los transportadores reales reducen a `approachSpeed` antes de un tope para limitar la energía de impacto ([[T14 Topes, impactos y alineación]]). De crucero $v$ a aproximación $v_c$:

$$d_{frenado} = \frac{v^2 - v_c^2}{2b}, \qquad t_{frenado} = \frac{v - v_c}{b}$$

La reducción debe empezar en $s_{tope} - d_{frenado} - d_{aprox}$, donde $d_{aprox}$ es la distancia recorrida a velocidad lenta (normalmente la dispara un sensor previo al tope). Con los valores por defecto ($v = 0{,}25$, $v_c = 0{,}10$, $b = 0{,}5$): $d_{frenado} = 0{,}0525$ m y $t_{frenado} = 0{,}3$ s.

## 7. Predecir cruces: `timeToReach`

Para programar «el frente alcanza el sensor en $s^{\star}$» se resuelve $s(t) = s^{\star}$ buscando el menor $t \ge t_0$:

$$\tfrac12 a\,\Delta t^2 + v_0\,\Delta t - (s^{\star} - s_0) = 0$$

- $a = 0$: $\Delta t = (s^{\star} - s_0)/v_0$ si $v_0 \ne 0$ y el resultado es $\ge 0$.
- $a \ne 0$: discriminante $\Delta = v_0^2 + 2a(s^{\star} - s_0)$. Si $\Delta < 0$ nunca se alcanza (un pallet que frena se queda corto). Si no, se toma la menor raíz no negativa de $\Delta t = (-v_0 \pm \sqrt{\Delta})/a$.

**Atención:** un segmento de frenado solo es válido hasta $v = 0$. El motor debe cerrarlo en $t_0 + v_0/|a|$ (evento «detenido») para no extrapolar la parábola hacia atrás.

**Nota numérica:** cuando $v_0^2 \gg |2a\Delta s|$, la forma $(-v_0 + \sqrt{\Delta})/a$ pierde precisión (resta de dos números parecidos). La forma equivalente $\Delta t = \dfrac{2\,\Delta s}{v_0 + \sqrt{\Delta}}$ es estable; adoptarla si alguna prueba muestra problemas de redondeo.

## 8. Frente, cola y orientación

El motor sigue el **frente**; la cola es $s_{cola} = s_{frente} - L$, donde $L$ es la dimensión del pallet *en el sentido del avance*: 1,2 m para un EUR con el lado largo delante y 0,8 m con el lado corto. Los pallets EUR van con el lado largo delante en las mesas de rodillos y con el corto delante en las de cadenas (confirmado por el responsable el 29-10-2026).

## 9. Entrega entre transportadores

Mientras un pallet está a caballo entre A → B, lo llevan los dos. Regla 2D propuesta (a confirmar en [[S01 Mesa de rodillos]]):

- Si ambos están en marcha, el pallet va a $\min(v_A, v_B)$ (domina la superficie más lenta por rozamiento).
- Si B está parado o lleno, el pallet se detiene con el frente en la entrada de B (menos el hueco): **bloqueo**.
- El hueco físico entre transportadores (20 mm por defecto) se salva al instante en 2D; en 3D es un hueco real que las tablas deben cruzar.

## 10. Referencia rápida con valores por defecto

| Magnitud                                   | Fórmula             | Resultado por defecto |
| ------------------------------------------ | ------------------- | --------------------- |
| Tiempo de rampa hasta 0,25 m/s             | $v/a$               | 0,5 s                 |
| Distancia de rampa                         | $v^2/2a$            | 62,5 mm               |
| Tiempo extra de un arranque (vs. instante) | $v/2a$              | 0,25 s                |
| Tiempo extra de arranque + parada          | $v/2a + v/2b$       | 0,5 s                 |
| Tránsito de 3 m sin parar                  | $D/v + v/2a$        | 12,25 s               |
| Tránsito de 3 m arrancando y parando       | $D/v + v/2a + v/2b$ | 12,5 s                |

## Relacionado

[[T02 Simulación de eventos discretos]] · [[T05 Acumulación, capacidad y rendimiento]] · [[T13 Giratorias y mesas de transferencia]] · [[A14 Referencia de API]]
