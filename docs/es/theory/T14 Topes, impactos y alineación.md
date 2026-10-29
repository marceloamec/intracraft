---
title: T14 Topes, impactos y alineación
lang: es
tags: [theory, physics, stops, guides]
---

🌐 [[T14 Stops, Impacts and Alignment|EN]] · **ES** · [[T14 Anschläge, Aufprall und Ausrichtung|DE]] · [[Inicio (ES)|Índice]]

# T14 Topes, impactos y alineación

## 1. Energía de impacto — prueba T5

$$E = \tfrac12\,m\,v^2$$

| Masa (pallet + carga) | 0,10 m/s (aproximación) | 0,25 m/s | 0,50 m/s |
| --------------------- | ----------------------- | -------- | -------- |
| 1025 kg               | 5,1 J                   | **32 J** | 128 J    |

La energía crece con el **cuadrado** de la velocidad: llegar a velocidad de aproximación (0,10 m/s) divide la energía de impacto por seis respecto a 0,25 m/s. Por eso existe `approachSpeed` ([[T04 Cinemática de transportadores]] §6). El simulador registra la velocidad y la energía de cada impacto y avisa por encima de `maxImpactEnergy` (200 J por defecto).

## 2. Impulso, tiempo de contacto y fuerza máxima

El impulso que entrega el tope es $J = m\,v\,(1 + e)$. La fuerza media depende de cuánto dura el contacto:

$$\bar{F} = \frac{J}{\Delta t_{contacto}}, \qquad \bar{a} = \frac{v(1+e)}{\Delta t_{contacto}}$$

Para 1025 kg a 0,25 m/s con $e = 0{,}1$: $J = 282$ N·s. Con un contacto de 10 ms: $\bar F \approx 28$ kN y $\bar a \approx 27{,}5$ m/s².

**Atención con los motores de sólidos rígidos:** un tope rígido detiene el pallet en aproximadamente **un paso de física** (8 ms), así que las fuerzas máximas son un artefacto del paso de tiempo, no una predicción. Los topes reales llevan topes de goma o amortiguadores. Para decidir, usar la **energía**, no la fuerza máxima simulada.

### ¿Volcará una carga alta en el tope?

Prueba rápida por energía: el pallet vuelca sobre su arista inferior delantera si su energía cinética puede elevar el CDM por encima de esa arista:

$$\Delta E_{vuelco} = m\,g\,\Big(\sqrt{h^2 + (L/2)^2} - h\Big)$$

EUR con 600 kg de carga ($h = 0{,}717$ m, $L/2 = 0{,}4$ m con el lado corto en el avance): $\Delta E_{vuelco} = 625 \cdot 9{,}81 \cdot (0{,}821 - 0{,}717) \approx 638$ J ≫ 32 J → no vuelca. (Aproximado: solo parte de la energía se convierte en giro; usarlo como criterio de cribado.)

## 3. Rebote

Con restitución $e$ el pallet sale del tope a $e\,v$. Los rodillos, que siguen empujando hacia delante, lo devuelven en $\approx (e v)^2 / (2\mu g)$: 0,09 mm con los valores por defecto ([[T11 Contacto, rozamiento y restitución]] §6). Criterio P1: frente a **±5 mm** de la posición teórica y rebote **< 10 mm**.

## 4. Modelo 2D de un tope

- El tope es un **límite de posición** del frente: el evento predicho `front_reaches_stop` termina el segmento de movimiento del pallet en $s_{tope}$.
- Rodillos: el pallet se detiene en la posición del tope (instantáneo en el modelo ideal).
- Cadenas: el pallet decelera durante `stopContactTime` (0,2 s por defecto) y las cadenas siguen deslizando por debajo.
- `raiseTime` / `lowerTime` (0,3 s) retrasan el efecto de una orden. Un tope **no debe subir bajo un pallet**: enclavamiento «subir el tope solo si la zona sobre el tope está libre».

## 5. Alineación con topes

Un tope fija la posición **longitudinal** del borde frontal. La geometría del contacto decide si también fija el **giro**:

- **Un contacto estrecho fuera del eje del pallet:** el impulso de impacto $J$ aplicado a una distancia lateral $d$ del CDM produce un impulso angular $J d$; el pallet gira ($I\,\Delta\omega = J d$) hasta que un segundo contacto o una guía lo frena. El rozamiento con los rodillos se opone al giro, así que el ángulo final depende de μ, masa e inercia: un resultado solo del 3D.
- **Un tope ancho o dos topes:** los contactos a ambos lados del CDM equilibran el momento y el borde frontal queda escuadrado contra el tope. Por eso importa `stopWidth` (100 mm por defecto; más ancho alinea mejor).

En 2D solo se aplica la posición final ideal; el modo 3D da la pose real (posición + ángulo), que es la medida de la calidad de la alineación.

## 6. Guías laterales y embudos

### 6.1 Holgura

Ancho útil ≥ ancho del pallet + 2 × holgura: EUR 800 mm + 2 × 25 mm = 850 mm ≤ 1000 mm ✓ (`usableWidthFitsPallet`, con prueba).

### 6.2 Rango de captura del embudo

Un embudo de longitud $L_f$ y ángulo $\theta_f$ ensancha la entrada por cada lado $L_f \tan\theta_f$: 300 mm a 10° → 53 mm. Captura pallets desplazados hasta holgura + 53 mm ≈ 78 mm (criterio S1-d / P3: 50 mm y 5° corregidos a ±10 mm y ±1°).

### 6.3 La cuña: la fuerza de avance se convierte en empuje lateral

Los rodillos empujan el pallet hacia delante con fuerza $F_d$ contra la guía inclinada. Con rozamiento de guía $\mu_g$, la fuerza normal $N$ debe equilibrar el empuje: $F_d = N(\sin\theta + \mu_g\cos\theta)$, y el empuje lateral es aproximadamente $N\cos\theta$:

$$F_{lat} \approx \frac{F_d}{\tan\theta + \mu_g}$$

A 10° con $\mu_g = 0{,}3$: $F_{lat} \approx 2{,}1\,F_d$. El pallet también debe **deslizar de lado sobre los rodillos** (los rodillos solo ruedan en el avance), lo que cuesta hasta $\mu_r m g$. Como el rozamiento con los rodillos es un único **cono de rozamiento**, el avance y el deslizamiento lateral comparten el mismo presupuesto $\mu_r m g$: el pallet se mueve en diagonal. Embudos muy abiertos (θ grande) o mucho rozamiento en la guía reducen $F_{lat}$ y pueden **atascar** el pallet. El resultado es una prueba 3D, no una fórmula; los ángulos pequeños (≤ 10°) son el valor seguro por defecto.

## Relacionado

[[S07 Tope mecánico]] · [[S08 Guía lateral]] · [[T11 Contacto, rozamiento y restitución]] · [[T09 Dinámica del sólido rígido]]
