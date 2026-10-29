---
title: T13 Giratorias y mesas de transferencia
lang: es
tags: [theory, des, physics, turntable, transfer]
---

🌐 [[T13 Turntables and Transfer Lifts|EN]] · **ES** · [[T13 Drehtische und Kettenumsetzer|DE]] · [[Inicio (ES)|Índice]]

# T13 Giratorias y mesas de transferencia

Ambos componentes cambian la dirección del flujo 90° (o 180°), atienden **un pallet a la vez** y suelen ser los cuellos de botella del layout ([[T05 Acumulación, capacidad y rendimiento]] §3).

## Parte 1 — Giratorias

### 1.1 Perfil angular

Girar $\theta$ con velocidad máxima $\omega$ y rampas $\alpha$ usa las fórmulas trapezoidal/triangular de [[T04 Cinemática de transportadores]] §4: 90° a 15 °/s con 30 °/s² tarda **6,5 s** (T3).

**Elegir el sentido.** Con `shortestPath`, el ángulo con signo es
$$\Delta\theta = \big((\theta_{destino} - \theta_{actual} + 180°) \bmod 360°\big) - 180°$$
Con un `angularRange` limitado (cables), la giratoria debe tomar el sentido permitido aunque sea más largo.

### 1.2 Estaciones y puertos

Los puertos se definen en el **marco de la plataforma**. Un puerto solo cuenta como conectado cuando el ángulo de la plataforma coincide con el de una estación dentro de `angularPrecision` (0,2° por defecto). En 2D el pallet simplemente viaja con la plataforma; en `turn_completed` + `settleTime` se habilita el puerto de la estación para descargar.

### 1.3 Radio de barrido

$$R_{barrido} = \tfrac12\sqrt{L_p^2 + W_p^2} + \text{holgura}$$

1400 × 1200 mm con 50 mm de holgura → **972 mm** (`sweptRadius`, con prueba). La validación comprueba que ningún vecino quede dentro de ese círculo.

### 1.4 ¿Deslizará el pallet durante el giro?

Un punto a radio $r$ de una plataforma que gira tiene aceleración tangencial $a_t = \alpha r$ y centrípeta $a_c = \omega^2 r$. El rozamiento debe aportar ambas:

$$\mu \ge \frac{\sqrt{a_t^2 + a_c^2}}{g}$$

Se toma $r$ en la esquina más alejada del pallet: EUR centrado, $r = \tfrac12\sqrt{1{,}2^2 + 0{,}8^2} = 0{,}72$ m. Con los valores por defecto ($\alpha = 30$ °/s² = 0,524 rad/s², $\omega = 15$ °/s = 0,262 rad/s): $a_t = 0{,}377$ m/s², $a_c = 0{,}049$ m/s² → **μ ≥ 0,039** (`turntableRequiredFriction`, con prueba). El μ disponible es 0,30–0,35: un margen amplio, que se reduce con rampas más fuertes, pallets más grandes y posiciones descentradas (tolerancia de centrado ±20 mm).

(Es conservador: $a_t$ es máxima durante la rampa y $a_c$ a plena velocidad; combinar los dos picos sobrestima la necesidad.)

### 1.5 Par de accionamiento

$$\tau = I_{total}\,\alpha + \tau_{rodamiento}$$

Ejemplo: plataforma de 400 kg y 1,4 × 1,2 m → $I = 400(1{,}4^2 + 1{,}2^2)/12 = 113$ kg·m²; EUR cargado (625 kg) ≈ 108 kg·m² ([[T09 Dinámica del sólido rígido]]); total ≈ 221 kg·m². Con $\alpha = 0{,}524$ rad/s² y 20 N·m de fricción de corona: $\tau \approx 136$ N·m, muy por debajo de los 1500 N·m por defecto (`turntableRequiredTorque`). Un pallet descentrado añade $m d^2$ (teorema de Steiner).

### 1.6 Modelo físico

- Plataforma: cuerpo cinemático con velocidad angular $\omega(t)$ alrededor del eje vertical.
- Sus rodillos o cadenas son cuerpos cinemáticos aparte: en cada paso el envoltorio calcula su pose a partir del ángulo de la plataforma y fija sus objetivos cinemáticos para que sus velocidades sean coherentes (un cuerpo teletransportado sin velocidad no arrastra al pallet).
- Giratoria de rodillos: rodillos **bloqueados** durante el giro ($\omega_{rodillo} = 0$), así el pallet queda sujeto por rozamiento de deslizamiento ([[T12 Rodillos y cadenas en física]]).
- Prueba S1-e / P4: el pallet se desplaza menos de 5 mm respecto a la plataforma en un giro de 90°.

### 1.7 Secuencia

esperando → recibiendo (transportador en marcha) → centrado (sensores) → girando → asentando → descargando → esperando. Enclavamiento: no gira si el pallet no está centrado y completamente sobre la plataforma.

## Parte 2 — Mesa de transferencia rodillos→cadenas

### 2.1 Mecanismo (interpretación a confirmar, plan §18)

Un tramo de rodillos recibe el pallet en sentido **longitudinal**. Entre los rodillos hay **ramales de cadena elevables** orientados en sentido transversal. Al subir, las cadenas levantan el pallet de los rodillos, lo desplazan 90° hacia la línea de cadenas contigua y bajan. El pallet cambia de dirección sin girar sobre sí mismo.

```text
     vista superior                     vista lateral (corte a través de los rodillos)
  ┌─────────────────────┐
  │ ║ │ ║ │ ║ │ ║ │ ║ │ │  ← rodillos (│) y cadenas (║) en los huecos     pallet ▄▄▄▄▄▄
  │ ║ │ ║ │ ║ │ ║ │ ║ │ │                                        elevado: ║ ↑ 40 mm
  └─────────────────────┘                                       rodillos: ○ ○ ○ ○
     avance en rodillos →     transferencia en cadenas ↓
```

### 2.2 Comprobaciones geométricas

- **Las cadenas caben entre rodillos:** hueco $= p - D \ge a_{cadena} + 10$ mm. Ø89 con paso 206,3 mm (valores del proyecto) → 117,3 mm ≥ 40 + 10 ✓; el Ø76 con paso 133 mm del plan → 57 mm ✓ (`chainFitsBetweenRollers`, con prueba).
- **Las cadenas tocan las tablas:** con el lado largo del pallet en el sentido de los rodillos, sus tablas inferiores van en la dirección de avance y cada cadena transversal las cruza todas ✓. Si el pallet estuviera girado 90°, las tablas serían paralelas a las cadenas y una cadena podría caer **en un hueco** entre tablas: el simulador avisa.
- **Continuidad de alturas:** las cadenas elevadas quedan `liftStroke` (40 mm) por encima del plano de rodillos, así que la mesa de cadenas contigua debe tener su superficie a la altura de rodillos + 40 mm (tolerancia de puerto ±10 mm).

### 2.3 Tiempo de ciclo — prueba T4

$$t = t_{subida} + \frac{d}{v} + \frac{v}{a} + t_{bajada}$$

1 s + (5 + 0,4) s + 1 s = **7,4 s** para 1 m a 0,2 m/s.

### 2.4 Elevación por contacto

Las cadenas son cuerpos cinemáticos que suben `liftStroke` + `lowerClearance` (50 mm) en `liftTime`. Levantan el pallet empujando sus tablas. Si las cadenas se detuvieran en seco arriba, el pallet seguiría subiendo y se elevaría $v^2/2g$; a 0,05 m/s son 0,13 mm, despreciable. Un actuador neumático tiene amortiguación final; uno eléctrico, rampas (parámetro `actuator`).

Prueba P5: elevado 40 ± 5 mm, desplazado 1 m ± 10 mm y depositado girando < 1°.

### 2.5 Enclavamientos

No subir sin pallet centrado (tope + sensores, tolerancia ±20 mm) · rodillos parados mientras las cadenas están arriba · no bajar hasta que el pallet haya dejado las cadenas · en modo bidireccional nunca aceptar por los dos lados.

## Relacionado

[[T04 Cinemática de transportadores]] · [[T07 Lógica de control y señales]] · [[S03 Giratoria de cadenas]] · [[S04 Giratoria de rodillos]] · [[S05 Transferencia rodillos-cadenas]]
