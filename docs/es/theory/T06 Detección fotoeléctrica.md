---
title: T06 Detección fotoeléctrica
lang: es
tags: [theory, sensors, des, physics]
updated: 2026-10-29
---

🌐 [[T06 Photoelectric Sensing|EN]] · **ES** · [[T06 Fotoelektrische Erfassung|DE]] · [[Inicio (ES)|Índice]]

# T06 Detección fotoeléctrica

## 1. Tipos de sensor y su modelo de simulación

| Tipo    | Dispositivo real                                                   | Modelo de simulación                                 |
| ------- | ------------------------------------------------------------------ | ---------------------------------------------------- |
| Difusa  | Emisor y receptor en la misma carcasa; luz reflejada por el objeto | Rayo desde el sensor en una dirección, hasta `range` |
| Reflex  | Reflector enfrente; el objeto **interrumpe** el haz                | Segmento fotocélula → reflector                      |
| Barrera | Emisor y receptor separados                                        | Segmento emisor → receptor                           |

El MVP ignora color, reflectividad y luz ambiente. La salida es una **señal booleana** con polaridad seleccionable.

## 2. Geometría: un haz entre dos puntos elegidos por el usuario

El usuario coloca la **fotocélula** en $E = (s_E, y_E, h_E)$ y el **reflector/receptor** en $R = (s_R, y_R, h_R)$, en el marco del transportador anfitrión ($s$ en el avance, $y$ transversal, $h$ altura sobre la superficie de transporte). El haz es el segmento

$$P(u) = E + u\,(R - E), \qquad u \in [0, 1]$$

Lo habitual es un haz transversal y horizontal ($s_E = s_R$, $h_E = h_R$), pero se admiten haces oblicuos o inclinados ([[S06 Fotocélula]]). La altura es el parámetro que más importa, porque un pallet no es una caja maciza.

## 3. El perfil de ocupación (modo 2D)

Cada tipo de pallet describe, por **banda de altura**, los intervalos macizos en su sentido de avance (desde el frente), por separado para las dos orientaciones: `alongLength` (lado largo delante, mesas de rodillos) y `alongWidth` (lado corto delante, mesas de cadenas). Tablas completas del EUR: [[S10 Tipos de pallet]]. Resumen:

| Banda EUR (mm)            | Lado largo delante (rodillos) | Lado corto delante (cadenas) |
| ------------------------- | ----------------------------- | ---------------------------- |
| 0–22 tablas inferiores    | continua                      | huecos de 227,5 mm           |
| 22–100 tacos              | huecos de 382,5 mm            | huecos de 227,5 mm           |
| 100–122 tablas de unión   | huecos de 382,5 mm            | **continua**                 |
| 122–144 tablas superiores | **continua**                  | huecos de ≈ 41 mm            |

Cada banda × intervalo es una caja que se mueve con el pallet: extensión transversal = todo el ancho del pallet, altura = la banda, en el avance = $[s_{frente} - b_k,\ s_{frente} - a_k]$.

## 4. Predecir los flancos para cualquier haz

Para una caja maciza $k$ con rango transversal $[y_0, y_1]$ y de altura $[h_0, h_1]$ (fijos mientras el pallet avanza) e intervalo en el avance $[a_k, b_k]$ desde el frente:

1. **Recortar el segmento** con las franjas transversal y de altura: los valores de $u$ con $y(u) \in [y_0, y_1]$ y $h(u) \in [h_0, h_1]$ forman un intervalo $[u_0, u_1]$ (si está vacío, esa caja nunca tapa el haz).
2. La parte recortada del haz ocupa en el avance $[x_{min}, x_{max}]$ = el rango de $s(u)$ para $u \in [u_0, u_1]$.
3. La caja tapa el haz justo cuando se solapan los dos intervalos en el avance:

$$s_{frente} \in [\,x_{min} + a_k,\; x_{max} + b_k\,]$$

Así cada caja da un intervalo de posiciones del frente, con un flanco de subida y otro de bajada, y el haz está tapado en la **unión** de esos intervalos (los que se solapan se funden). El motor DES programa los flancos con `timeToReach` ([[T04 Cinemática de transportadores]]). Para un haz transversal horizontal $x_{min} = x_{max} = s_{sens}$ y la regla queda $s_{frente} \in [s_{sens} + a_k,\ s_{sens} + b_k]$.

Por qué siempre es un único intervalo: el conjunto de traslaciones para las que dos formas convexas se cortan es convexo; en un solo eje, es un intervalo.

### Prueba T6

Haz en una banda continua en $s = 2{,}0$ m, $v = 0{,}25$ m/s, EUR con el lado largo delante: subida cuando el frente pasa por 2,0 m y bajada cuando pasa la cola; duración $1{,}2/0{,}25$ = **4,8 s**.

## 5. El problema de los huecos del pallet

| Caso (EUR, 0,25 m/s)                      | Hueco    | Señal apagada durante         |
| ----------------------------------------- | -------- | ----------------------------- |
| Rodillos, haz a 60 mm (capa de tacos)     | 382,5 mm | 1,53 s (dos veces por pallet) |
| Cadenas, haz a 60 mm (capa de tacos)      | 227,5 mm | 0,91 s (dos veces por pallet) |
| Cadenas, haz a 130 mm (tablas superiores) | ≈ 41 mm  | ≈ 0,16 s (cuatro veces)       |

Un controlador que cuente pallets contaría tres (o cinco) por pallet. Remedios:

1. **Elegir la altura del haz** en una banda continua para la orientación de ese transportador: ≈ 130 mm en rodillos y ≈ 110 mm en cadenas (o por encima del pallet, sobre la carga).
2. **Filtro:** exigir que el nuevo estado dure `filter` ms; para salvar un hueco $t_{filtro} > \text{hueco}/v$. El filtro también retrasa ese mismo tiempo el flanco de bajada **real**, lo que desplaza la liberación de zonas y puede costar rendimiento.

El editor avisa cuando la altura elegida cae en una banda con huecos ([[S06 Fotocélula]] §2).

## 6. Tamaño del spot

Un haz de diámetro $d$ empieza a taparse antes de que un borde llegue a su eje. El 2D trata el haz como una línea; el 3D puede usar un **shape-cast** para spots grandes, que ensancha cada intervalo macizo unos $d/2$ por cada lado.

## 7. Tiempos: respuesta del sensor y ciclo del PLC

$$t_{visto} = t_{geométrico} + t_{respuesta} + t_{filtro} + t_{muestreo}$$

`responseTime` (2 ms por defecto) se aplica en ambos modos; el ciclo del controlador (10 ms por defecto) da en 3D un retraso uniforme en $[0, 10)$ ms, que el 2D puede reproducir o aproximar por su media.

## 8. Modo físico: raycasts

En cada paso de física se lanza el segmento $E \to R$ contra los colisionadores (excluyendo rodillos, bastidores y guías del propio transportador). Publicar el flanco en el punto medio del paso acota el error a $\Delta t/2 = 4{,}2$ ms a 120 Hz.

## 9. Presupuesto de error 2D ↔ 3D para flancos (objetivo ±50 ms)

| Fuente                                            | Tamaño típico |
| ------------------------------------------------- | ------------- |
| Cuantización del paso de física (punto medio)     | ≤ 4,2 ms      |
| Muestreo del controlador (si no se replica en 2D) | ≤ 10 ms       |
| Deslizamiento del pallet (3D)                     | 0 – 20 ms     |
| Velocidad distinta durante la entrega             | 0 – 20 ms     |
| **Total (peor caso, no correlacionado)**          | < 50 ms       |

## Relacionado

[[S06 Fotocélula]] · [[S10 Tipos de pallet]] · [[T07 Lógica de control y señales]] · [[T11 Contacto, rozamiento y restitución]]
