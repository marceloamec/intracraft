---
title: T11 Contacto, rozamiento y restitución
lang: es
tags: [theory, physics, contact]
---

🌐 [[T11 Contact, Friction and Restitution|EN]] · **ES** · [[T11 Kontakt, Reibung und Restitution|DE]] · [[Inicio (ES)|Índice]]

# T11 Contacto, rozamiento y restitución

En los contactos ocurre toda la física del transporte: los rodillos solo mueven el pallet mediante el rozamiento en sus puntos de contacto, y los topes y guías solo actúan mediante contactos.

## 1. La cadena de colisiones (en cada paso)

1. **Fase amplia** — pruebas baratas con cajas envolventes (jerarquías de volúmenes o barrido y poda) encuentran los pares que *podrían* tocarse.
2. **Fase estrecha** — pruebas exactas (GJK/EPA, SAT para cajas) calculan una **variedad de contacto**: hasta 4 puntos, una normal $\mathbf{n}$ y profundidades de penetración. Una tabla (caja) sobre un rodillo (cilindro) da un contacto en línea, normalmente 2 puntos.
3. **Resolución de restricciones** — se calculan impulsos para que los cuerpos no se penetren, se respete el rozamiento y se aplique la restitución.
4. **Integración** — se actualizan velocidades y posiciones ([[T10 Integración temporal y paso fijo]]).

## 2. No penetración

En cada punto de contacto, con velocidad normal relativa $v_n$ e impulso normal $\lambda_n$:

$$v_n \ge 0, \qquad \lambda_n \ge 0, \qquad \lambda_n\,v_n = 0$$

(los cuerpos se separan o siguen en contacto; el contacto puede empujar pero nunca tirar; solo empuja mientras se tocan). Este problema de *complementariedad* se resuelve de forma iterativa.

## 3. Rozamiento de Coulomb

La fuerza tangencial está limitada por la normal:

$$\|\mathbf{F}_t\| \le \mu\,F_n$$

- **Adherencia** — si el rozamiento puede anular la velocidad tangencial relativa dentro de ese límite, las superficies se mueven juntas.
- **Deslizamiento** — si no, la fuerza de rozamiento vale $\mu F_n$ y se opone a la velocidad relativa de deslizamiento.

El conjunto de fuerzas de contacto admisibles es el **cono de rozamiento** (semiángulo $\arctan\mu$); los motores lo aproximan con una pirámide o limitando los dos impulsos tangenciales.

**Clave para transportadores:** el rozamiento actúa sobre la velocidad **relativa** entre el pallet y la *superficie* en el punto de contacto. La superficie de un rodillo cinemático que gira a $\omega$ se mueve a $\omega r$ en su parte superior; el rozamiento lleva la tabla hacia esa velocidad. Ese es todo el mecanismo de un transportador de rodillos motorizados ([[T12 Rodillos y cadenas en física]]).

**Estático frente a cinético.** Los materiales reales tienen $\mu_s > \mu_k$. Rapier y Jolt usan un único coeficiente por colisionador. La tabla de fricción de IntraCraft da un valor por par de materiales ([[S10 Tipos de pallet]]).

### Combinación de coeficientes

Cada colisionador tiene su rozamiento; el motor combina los dos:

| Motor  | Combinación por defecto                    | Cómo obtener exactamente el valor de la tabla $\mu_{par}$                  |
| ------ | ------------------------------------------ | -------------------------------------------------------------------------- |
| Rapier | Media (configurable: mín., máx., producto) | Dar $\mu_{par}$ a ambos colisionadores, o elegir la regla por colisionador |
| Jolt   | Media geométrica $\sqrt{\mu_1\mu_2}$       | Sobrescribir `mCombinedFriction` por contacto en el listener de contactos  |

(Valores por defecto comprobados el 28-10-2026 con `@dimforge/rapier3d-compat` 0.21.0 — `frictionCombineRule` = Average, rozamiento por defecto 0,5 — y con `jolt-physics` 1.1.1; ver [[T15 Investigación de motores de física]].)

## 4. Cómo trabaja el solver: impulsos secuenciales

La mayoría de motores en tiempo real usan **Gauss–Seidel proyectado / impulsos secuenciales**: recorren todos los contactos varias veces; para cada uno calculan el impulso que corregiría su error de velocidad, lo suman a un impulso **acumulado** y limitan ese acumulado ($\lambda_n \ge 0$, $|\lambda_t| \le \mu\lambda_n$). El **arranque en caliente** reutiliza los impulsos del paso anterior, de modo que un pallet apoyado en rodillos converge en pocas iteraciones. Los cuerpos pesados con muchos contactos necesitan más iteraciones; se mide en S1-a.

## 5. Corrección de penetración

Los errores numéricos dejan los cuerpos ligeramente solapados. Los motores los separan con suavidad (términos de Baumgarte, impulsos divididos o restricciones blandas) y toleran una pequeña **holgura** (penetración permitida) para evitar vibraciones.

> ⚠️ Esa holgura afecta directamente al criterio de ±5 mm en el tope. Valores por defecto medidos: **Jolt** `mPenetrationSlop` = 0,02 m (y `mSpeculativeContactDistance` = 0,02 m); **Rapier** `normalizedAllowedLinearError` = 0,005 m (× `lengthUnit` = 1). Ambos están en la tolerancia o por encima, así que IntraCraft debe bajarlos a ≈ 1 mm o compensarlos. Rapier ofrece además una **piel de contacto** por colisionador.

## 6. Restitución (rebote)

El coeficiente de restitución relaciona las velocidades de separación y de aproximación según la normal:

$$e = -\frac{v_n^{después}}{v_n^{antes}}, \qquad 0 \le e \le 1$$

$e = 0$: perfectamente plástico (sin rebote); $e = 1$: perfectamente elástico. Los topes usan $e = 0{,}1$ por defecto.

> ⚠️ **Umbral de velocidad.** Muchos motores ignoran la restitución por debajo de una velocidad de aproximación mínima (Jolt: `mMinVelocityForRestitution`, **1 m/s** por defecto, medido) para evitar microrrebotes sin fin. Los impactos de pallets ocurren a 0,1–0,25 m/s, así que la restitución se ignoraría en silencio si no se baja el umbral.

**Rebote esperado.** Con $e = 0{,}1$ a 0,25 m/s el pallet rebota a 0,025 m/s; los rodillos, que siguen empujando hacia delante, lo frenan en $v^2/(2\mu g) = 0{,}025^2/(2 \cdot 0{,}35 \cdot 9{,}81) \approx 0{,}09$ mm. El criterio de < 10 mm se cumple con facilidad salvo con rozamientos muy bajos.

## 7. Detección continua de colisiones (CCD)

Un cuerpo **atraviesa** un obstáculo si en un paso se mueve más que el espesor del obstáculo. A 0,6 m/s y 120 Hz el desplazamiento es de 5 mm, menos que la cara de un tope, pero el CCD se activa en los pallets como red de seguridad, tal como pide el plan.

## 8. Rendimiento: contar los contactos

Un pallet con 3 tablas inferiores sobre 9 rodillos crea hasta 27 pares tabla–rodillo, cada uno con unos 2 puntos. Diez pallets → unos 500 puntos de contacto por paso, 60 000 por segundo a 120 Hz. Por eso los rodillos se agrupan, los cuerpos se duermen en reposo y el criterio de 20 transportadores / 10 pallets / 60 FPS (S1-f) es una prueba real.

## Relacionado

[[T12 Rodillos y cadenas en física]] · [[T14 Topes, impactos y alineación]] · [[T15 Investigación de motores de física]]
