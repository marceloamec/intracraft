---
title: T09 Dinámica del sólido rígido
lang: es
tags: [theory, physics]
---

🌐 [[T09 Rigid Body Dynamics|EN]] · **ES** · [[T09 Starrkörperdynamik|DE]] · [[Inicio (ES)|Índice]]

# T09 Dinámica del sólido rígido

## 1. Estado de un sólido rígido

Un sólido rígido no se deforma; su estado completo es

| Símbolo             | Significado                  | Notas                                       |
| ------------------- | ---------------------------- | ------------------------------------------- |
| $\mathbf{x}$        | Posición del centro de masas | metros, sistema mundo (Y hacia arriba)      |
| $\mathbf{q}$        | Orientación                  | Cuaternión unitario (sin bloqueo de cardán) |
| $\mathbf{v}$        | Velocidad lineal             | m/s                                         |
| $\boldsymbol\omega$ | Velocidad angular            | rad/s                                       |

más unas **propiedades de masa** constantes: masa $m$, centro de masas (CDM) y tensor de inercia $\mathbf{I}$.

## 2. Ecuaciones del movimiento (Newton–Euler)

$$m\,\dot{\mathbf{v}} = \mathbf{F} \qquad \mathbf{I}\,\dot{\boldsymbol\omega} + \boldsymbol\omega \times (\mathbf{I}\,\boldsymbol\omega) = \boldsymbol\tau$$

$\mathbf{F}$ es la suma de fuerzas (gravedad $m\mathbf{g}$ + fuerzas de contacto) y $\boldsymbol\tau$ la suma de momentos respecto al CDM. El tensor de inercia en coordenadas mundo es $\mathbf{I}_w = \mathbf{R}\,\mathbf{I}_b\,\mathbf{R}^T$, con $\mathbf{R}$ la matriz de giro de $\mathbf{q}$. El término giroscópico $\boldsymbol\omega \times \mathbf{I}\boldsymbol\omega$ es despreciable en pallets que giran despacio, pero el motor lo incluye.

En la práctica el motor no aplica las fuerzas de contacto directamente: calcula **impulsos** que cumplen las restricciones de contacto ([[T11 Contacto, rozamiento y restitución]]).

## 3. Tres tipos de movimiento

| Tipo       | Lo mueve                | Le afectan fuerzas | Uso                                                                 |
| ---------- | ----------------------- | ------------------ | ------------------------------------------------------------------- |
| Estático   | Nada                    | No                 | Guías, bastidores, suelo                                            |
| Cinemático | Velocidad/pose impuesta | No (masa infinita) | Rodillos, cadenas, plataformas giratorias, topes, cadenas elevables |
| Dinámico   | Fuerzas y contactos     | Sí                 | Pallets                                                             |

Un rodillo cinemático «gira» con una velocidad angular impuesta; para el solver no tiene inercia, así que un pallet de 1000 kg no puede frenarlo. Se supone que el motor tiene fuerza suficiente: en el MVP el límite de par es una validación, no una simulación.

## 4. Inercia de las formas que necesitamos

| Forma                                 | Eje                                  | Momento de inercia          |
| ------------------------------------- | ------------------------------------ | --------------------------- |
| Caja maciza $a \times b \times c$     | Por el CDM, ⟂ a la cara $a \times b$ | $\frac{m}{12}(a^2 + b^2)$   |
| Cilindro macizo, radio $r$, largo $h$ | Su propio eje                        | $\frac12 m r^2$             |
| Cilindro macizo                       | Transversal, por el CDM              | $\frac{m}{12}(3r^2 + h^2)$  |
| Tubo (camisa de rodillo) $r_o, r_i$   | Su propio eje                        | $\frac12 m (r_o^2 + r_i^2)$ |

**Teorema de Steiner (ejes paralelos)** — respecto a un eje a distancia $d$ del CDM: $I = I_{cm} + m\,d^2$. Así se monta la inercia de un cuerpo compuesto a partir de sus partes.

## 5. El pallet compuesto

El pallet **no** es una caja. Es un cuerpo dinámico formado por:

- 3 cajas para las tablas inferiores (las únicas que tocan rodillos o cadenas);
- 1 caja para el bloque de carga (con su altura y su masa);
- opcionalmente las tablas superiores y los tacos para los rayos de los sensores (o el perfil de ocupación genera los colisionadores — [[S10 Tipos de pallet]]).

Los huecos entre tablas inferiores importan: una cadena que cae en un hueco no levanta el pallet, y un haz puede pasar entre tacos.

### Ejemplo: pallet EUR con 600 kg de carga y 1200 mm de altura

| Parte     | Masa (kg) | Altura del CDM (m)                                               | Inercia de giro respecto a su CDM (kg·m²)       |
| --------- | --------- | ---------------------------------------------------------------- | ----------------------------------------------- |
| Pallet    | 25        | 0,072                                                            | $25(1{,}2^2 + 0{,}8^2)/12 = 4{,}3$              |
| Carga     | 600       | $0{,}144 + 0{,}6 = 0{,}744$                                      | $600(1{,}2^2 + 0{,}8^2)/12 = 104$               |
| **Total** | **625**   | $\dfrac{25 \cdot 0{,}072 + 600 \cdot 0{,}744}{625} = $ **0,717** | **≈ 108** (ambos centrados, sin desplazamiento) |

El parámetro `centerOfMassOffset` desplaza en horizontal el CDM de la carga para modelar cargas descentradas.

## 6. Vuelco frente a deslizamiento: por qué importa la altura del CDM

Durante una aceleración $a$ en el sentido del avance, la fuerza de inercia $m a$ actúa a la altura $h$ del CDM. Tomando momentos respecto al borde trasero de la huella de apoyo (semilongitud $L/2$):

- El pallet **vuelca** si $a\,h > g\,(L/2)$, es decir $a_{vuelco} = g\,\dfrac{L/2}{h}$.
- **Desliza** antes si $\mu < \dfrac{L/2}{h}$ (el rozamiento se satura antes de volcar).

Ejemplo (lado corto de 0,8 m en el avance, $h = 0{,}717$ m): $a_{vuelco} = 9{,}81 \times 0{,}4/0{,}717 = 5{,}5$ m/s². Las rampas normales (0,5 m/s²) quedan muy lejos, pero la **deceleración en el impacto contra un tope** puede ser mucho mayor durante unos milisegundos; por eso las cargas altas necesitan velocidades de aproximación bajas ([[T14 Topes, impactos y alineación]]).

## 7. Amortiguación

Una amortiguación lineal y angular pequeña (≈ 0,01–0,05 s⁻¹) elimina vibraciones numéricas. Debe ser pequeña: la amortiguación es una fuerza artificial y cambia los tiempos de tránsito. Cualquier cambio se registra en una nota de decisión (regla de AGENTS.md).

## 8. Simplificaciones del MVP

- La carga va unida rígidamente al pallet (no se desplaza ni se deforma).
- Rodillos y cadenas son cinemáticos: sin dinámica de motor ni estiramiento de cadena.
- La madera es rígida; se ignora la flexión de las tablas entre rodillos.

## Relacionado

[[T10 Integración temporal y paso fijo]] · [[T11 Contacto, rozamiento y restitución]] · [[T12 Rodillos y cadenas en física]] · [[A07 Envoltorio del motor físico]]
