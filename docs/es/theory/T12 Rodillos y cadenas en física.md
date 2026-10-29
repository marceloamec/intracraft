---
title: T12 Rodillos y cadenas en física
lang: es
tags: [theory, physics, conveyors]
---

🌐 [[T12 Rollers and Chains in Physics|EN]] · **ES** · [[T12 Rollen und Ketten in der Physik|DE]] · [[Inicio (ES)|Índice]]

# T12 Rodillos y cadenas en física

## 1. Rodillos motorizados

Un rodillo motorizado es un **cilindro cinemático** que gira sobre su eje a

$$\omega = \frac{v}{r} = \frac{v}{D/2}$$

Ø89 mm (valor del proyecto) a 0,25 m/s → $\omega = 0{,}25/0{,}0445 = 5{,}62$ rad/s; Ø76 mm → 6,58 rad/s (`rollerAngularVelocity`, con prueba). La parte superior del rodillo se mueve a $\omega r = v$; el rozamiento en el contacto tabla–rodillo lleva al pallet hacia esa velocidad ([[T11 Contacto, rozamiento y restitución]]).

Las rampas las genera el **envoltorio**, no el motor: en cada paso fija la $\omega(t)$ de cada zona con el mismo perfil trapezoidal que usa el modo 2D ([[T04 Cinemática de transportadores]]).

### Límite de tracción

Con todos los rodillos de apoyo motorizados, el rozamiento puede acelerar el pallet como mucho

$$a_{max} = \mu\,g$$

$\mu = 0{,}35$ → 3,43 m/s² (`maxTractionAccel`, con prueba). La rampa por defecto de 0,5 m/s² está 7 veces por debajo, así que el pallet sigue a los rodillos con un microdeslizamiento mínimo. Hay deslizamiento cuando: μ es bajo (humedad, pallets de plástico), solo parte de los rodillos está motorizada, el pallet está retenido (tope, cola) o la rampa es demasiado fuerte.

### Apoyo y regla de los 3 rodillos

Un lado del pallet de longitud $L$ apoya sobre $\lfloor L/p \rfloor$ rodillos (paso $p$): 1200 mm con 206,3 mm → 5 rodillos (con 133 mm: 9). Siempre debe haber al menos **3** bajo cada lado apoyado, para que el pallet nunca caiga en un hueco y salve el hueco entre transportadores. El reparto de carga entre rodillos es hiperestático; el motor lo resuelve y para el rozamiento solo importa el total.

### Rodillos libres (gravedad)

Sin pendiente (MVP), los rodillos libres solo dejan pasar un pallet empujado. Simular cada uno como un cuerpo dinámico libre con bisagra es caro. **Modelo propuesto para el MVP:** los rodillos libres son un apoyo estático con un rozamiento bajo igual al coeficiente de resistencia a la rodadura ($C_{rr} \approx 0{,}02$): fuerza resistente $C_{rr}\,m\,g$.

### Rodillos bloqueados (giratorias)

En una giratoria de rodillos, los rodillos se bloquean durante el giro ($\omega = 0$). El contacto pasa a ser de **deslizamiento**, no de rodadura, con $\mu$ completo, y eso es lo que sujeta el pallet ([[T13 Giratorias y mesas de transferencia]]). Con rodillos libres el pallet rodaría durante la aceleración angular.

## 2. Cadenas: la parte difícil

Una mesa de cadenas real tiene de 2 a 6 ramales; cada ramal portante desliza sobre una guía de desgaste y el pallet apoya en los eslabones o rodillos de la cadena. Los pallets EUR avanzan **con el lado corto delante** en cadenas (confirmado por el responsable), así que las tablas inferiores quedan *perpendiculares* a las cadenas y cada tabla toca cada ramal en el ancho de la cadena (40 mm).

La dificultad: la superficie de contacto se mueve (la cadena), pero el cuerpo de debajo (la guía) no. Un motor de física necesita o muchos cuerpos en movimiento o un contacto que «sepa» la velocidad de su superficie.

### Los cuatro planes

| Plan | Modelo                                                                     | Ventajas                               | Inconvenientes                                                                             |
| ---- | -------------------------------------------------------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------ |
| A    | Tira de rodillos cinemáticos pequeños (Ø 30 mm cada 40 mm), $\omega = v/r$ | Funciona en cualquier motor; sin hooks | Muchos cuerpos (3 ramales × 75 rodillos cada 3 m = 225); contacto irregular; más contactos |
| B    | **Superficie móvil** real en el contacto                                   | Una caja por ramal; suave; barato      | Necesita que el binding JS del motor lo permita                                            |
| C    | Fuerza de arrastre $F = \text{clamp}(k\,m\,(v_c - v_p),\ \pm\mu N)$        | Sencillo                               | Hay que estimar $N$; no es un modelo de contacto real                                      |
| D    | Módulo físico propio en Rust (WASM) con `modify_solver_contacts`           | Control total                          | Añade Rust y una cadena de compilación; último recurso                                     |

Plan A con Ø30 mm a 0,25 m/s: $\omega = 0{,}25/0{,}015 = 16{,}7$ rad/s.

### Resultado de la investigación (28-10-2026)

- **Jolt** (`jolt-physics` 1.1.1 en npm): los tipos TypeScript exponen `ContactListenerJS` con `OnContactAdded` / `OnContactPersisted(body1, body2, manifold, ioSettings)` y `ContactSettings.mRelativeLinearSurfaceVelocity` / `mRelativeAngularSurfaceVelocity`. **El plan B parece viable con Jolt**; el spike debe confirmar el convenio de signos, el coste por contacto de una llamada a JS y el determinismo.
- **Rapier** (`@dimforge/rapier3d-compat` 0.21.0): la interfaz JS `PhysicsHooks` solo tiene `filterContactPair` y `filterIntersectionPair` (`ActiveHooks`: `FILTER_CONTACT_PAIRS`, `FILTER_INTERSECTION_PAIRS`). **No permite modificar los contactos del solver → no hay plan B**; Rapier usaría el plan A (o el D).

Detalles en [[T15 Investigación de motores de física]] y [[ADR-0002 Candidato a motor de física]].

### Cómo funciona la superficie móvil (plan B)

El solver normalmente lleva a cero la velocidad tangencial relativa en el contacto (adherencia). Con una velocidad superficial $\mathbf{u}$ el objetivo pasa a ser $\mathbf{v}_{rel,t} = \mathbf{u}$: la guía estática se comporta como si su superficie se moviera a la velocidad de la cadena. El límite de rozamiento $\mu N$ se sigue aplicando, así que un pallet retenido patina igual que sobre una cadena real.

## 3. Acumulación con deslizamiento

Bajo un pallet retenido la cadena (rodillos del plan A o superficie del plan B) sigue moviéndose: el rozamiento cinético empuja el pallet hacia delante con $\mu\,m\,g$ (por pallet), y los empujes se suman a lo largo de la cola ([[T05 Acumulación, capacidad y rendimiento]] §6.3). Ambos planes lo reproducen de forma natural; el modo 3D publica entonces la fuerza sobre el tope.

## 4. Efecto poligonal

Una cadena que envuelve un piñón de $z$ dientes no se mueve a velocidad constante: la velocidad oscila con una amplitud relativa $1 - \cos(\pi/z)$, un 1,7 % para $z = 17$. El parámetro `chainPitch` queda reservado para ello; el MVP ignora el efecto (velocidad de cadena constante).

## 5. Pruebas de aceptación (spike S1)

| Prueba | Montaje                                         | Criterio                                                               |
| ------ | ----------------------------------------------- | ---------------------------------------------------------------------- |
| a      | Pallet de 1000 kg sobre 10 rodillos cinemáticos | Alcanza la velocidad nominal ±3 %, frena sin deslizamiento excesivo    |
| b      | Tira de rodillos pequeños como cadena           | Velocidad del pallet = velocidad de cadena ±3 %, sin vibración visible |
| f      | 20 transportadores, 10 pallets                  | 60 FPS, sin inestabilidades numéricas                                  |

## Relacionado

[[T11 Contacto, rozamiento y restitución]] · [[T13 Giratorias y mesas de transferencia]] · [[S01 Mesa de rodillos]] · [[S02 Mesa de cadenas]] · [[T15 Investigación de motores de física]]
