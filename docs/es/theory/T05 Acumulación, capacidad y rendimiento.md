---
title: T05 Acumulación, capacidad y rendimiento
lang: es
tags: [theory, des, flow]
---

🌐 [[T05 Accumulation, Capacity and Throughput|EN]] · **ES** · [[T05 Stau, Kapazität und Durchsatz|DE]] · [[Inicio (ES)|Índice]]

# T05 Acumulación, capacidad y rendimiento

## 1. Magnitudes clave

| Magnitud                | Símbolo   | Definición                                                         |
| ----------------------- | --------- | ------------------------------------------------------------------ |
| Rendimiento             | $\lambda$ | Pallets que salen por unidad de tiempo (pallets/s o pallets/h)     |
| Capacidad (estática)    | $N$       | Número máximo de pallets que caben en un transportador             |
| Tiempo de tránsito      | $W$       | Tiempo desde la entrada hasta la salida de un pallet               |
| Trabajo en curso        | $\bar{L}$ | Número medio de pallets dentro de un objeto                        |
| Intervalo entre pallets | $h$       | Tiempo entre dos pallets consecutivos en un punto; $\lambda = 1/h$ |
| Utilización             | $\rho$    | Fracción del tiempo en que el objeto está ocupado                  |

## 2. Rendimiento limitado por la separación — prueba T2

En un transportador que marcha de forma continua a velocidad $v$, pallets de longitud $L$ (en el avance) separados por un hueco $g$ pasan por un punto cada $h = (L + g)/v$ segundos:

$$\lambda_{max} = \frac{v}{L + g}$$

**T2:** EUR con el lado largo en el avance ($L = 1{,}2$ m), $g = 0{,}15$ m, $v = 0{,}25$ m/s → $h = 5{,}4$ s, $\lambda_{max} = 0{,}185$ pallets/s ≈ **667 pallets/h**. Ninguna línea hecha con estos transportadores puede superarlo, sea cual sea la lógica de control.

Capacidad estática: $N = \left\lfloor \dfrac{\text{longitud}}{L + g} \right\rfloor$ → en una mesa de 3 m caben 2 pallets EUR.

## 3. Rendimiento limitado por estaciones y cuellos de botella

Las giratorias y las mesas de transferencia atienden **un pallet a la vez**. Su rendimiento es $1/t_{ciclo}$, donde el ciclo incluye todas las fases hasta que la estación puede aceptar el siguiente pallet.

Ciclo ilustrativo de una giratoria de cadenas (valores de ejemplo, no una especificación):

| Fase                                  | Estimación             | Tiempo                       |
| ------------------------------------- | ---------------------- | ---------------------------- |
| Recibir (≈ 1,4 m a 0,25 m/s + rampas) | $1{,}4/0{,}25 + 0{,}5$ | 6,1 s                        |
| Girar 90°                             | T3                     | 6,5 s                        |
| Asentar                               | parámetro              | 0,3 s                        |
| Descargar                             | igual que recibir      | 6,1 s                        |
| Volver 90° (vacía)                    | T3                     | 6,5 s                        |
| **Ciclo**                             |                        | **25,5 s** → ≈ 141 pallets/h |

El rendimiento de la línea es el mínimo de sus etapas: el **cuello de botella**. Aquí la giratoria (141/h) limita una línea cuyos transportadores podrían llevar 667/h. Mejoras típicas: evitar el giro de vuelta en vacío (servir estaciones alternas), acortar rampas, solapar descarga y recepción.

## 4. Ley de Little

Para cualquier sistema estable (medias a largo plazo, sin acumulación ilimitada):

$$\bar{L} = \lambda \cdot W$$

Ejemplo: un tramo pulmón deja pasar 120 pallets/h ($\lambda = 1/30$ s⁻¹) y cada pallet pasa en él 90 s de media → hay de media $\bar L = 3$ pallets dentro. La ley de Little es una comprobación muy útil de las estadísticas: si los tres números que publica la simulación no la cumplen, hay un error en el cálculo de estadísticas ([[T08 Estadísticas y análisis de resultados]]).

## 5. Estados, bloqueo y falta de material

Cada transportador está siempre en un solo estado: **parado, acelerando, en marcha, deteniendo, bloqueado**. Dos tipos de tiempo perdido dominan los estudios de flujo:

- **Falta de material (starved)** — listo para trabajar, pero no llega nada (problema aguas arriba). Se muestra como «tiempo vacío».
- **Bloqueado** — tiene un pallet listo para salir, pero el siguiente objeto no lo acepta (problema aguas abajo). Los transportadores se comportan como **pulmones finitos con bloqueo tras el servicio**: un pallet que ya terminó su transporte espera en el transportador ocupando espacio.

Utilización $\rho = t_{trabajando}/t_{total}$; un cuello de botella tiene $\rho \to 1$, sus vecinos aguas arriba muestran tiempo bloqueado y los de aguas abajo, tiempo vacío.

## 6. Modos de acumulación

### 6.1 Sin acumulación (`none`, por defecto)

Todo el transportador marcha o para como una unidad y cada pallet sigue a los rodillos o a las cadenas. Cuando entra un pallet nuevo, el transportador tiene que marchar, así que los que ya están se mueven también; las separaciones entre pallets se mantienen. Cuando el pallet delantero llega a la salida y el siguiente transportador no puede recibirlo, se para todo el transportador (bloqueado) y no recibe más. Un transportador más largo simplemente admite más pallets en fila. Es el comportamiento por defecto de todos los transportadores y, como todo comportamiento, un parámetro de cada instancia ([[S01 Mesa de rodillos]] §3).

### 6.2 Acumulación sin presión (ZPA)

El transportador se divide en **zonas** (1,4 m por defecto ≥ largo del pallet + holgura), cada una con su accionamiento y su sensor de fin de zona. Regla básica:

> La zona *i* marcha si la zona *i + 1* está libre (o en marcha); si no, se detiene cuando su propio sensor está ocupado.

Así los pallets se paran uno por zona y **nunca se tocan**. Dos modos de liberación:

- **Singulación** — cada zona arranca solo cuando la de delante ha quedado libre. Los pallets salen más separados: el intervalo es aproximadamente el tiempo que tarda un pallet en dejar una zona desde parado, $h \approx L_z/v + v/2a = 1{,}4/0{,}25 + 0{,}25 = 5{,}85$ s (≈ 615/h, frente a 667/h en flujo continuo). Este «efecto acordeón» es una pérdida real de rendimiento que el modo 2D debe reproducir.
- **Liberación en tren** — todas las zonas de una cola arrancan a la vez y mantienen la separación mínima; es más rápido, pero necesita espacio aguas abajo.

La plantilla Si/Entonces de ZPA está en `control/src/rules.test.ts` y en [[T07 Lógica de control y señales]].

### 6.3 Acumulación con presión (cadenas que patinan)

En las mesas de cadenas las cadenas siguen girando bajo los pallets retenidos y **patinan** bajo ellos. Cada pallet retenido es empujado hacia delante por la fuerza de rozamiento cinético $F = \mu\,m\,g$, y las fuerzas se suman a lo largo de la cola:

$$F_{tope} = \sum_{k=1}^{n} \mu\,m_k\,g$$

Ejemplo: 3 pallets de 625 kg sobre cadenas con $\mu = 0{,}3$ → $F_{tope} = 3 \times 0{,}3 \times 625 \times 9{,}81 \approx 5{,}5$ kN sobre el primer tope (y sobre las tablas del primer pallet). En 2D los pallets simplemente se detienen en contacto con la separación mínima (0 por defecto en este modo) tras `stopContactTime`; el modo 3D calcula las fuerzas de contacto reales.

## 7. Consecuencias prácticas para el motor 2D

1. Un pallet seguidor nunca puede pasar de la cola de su líder menos la separación mínima; es un **evento predicho** (`follower_reaches_leader`), no una comprobación en cada paso.
2. Cada arranque o parada cuesta $v/2a$ (arranque) y $v/2b$ (parada) frente a un movimiento instantáneo ([[T04 Cinemática de transportadores]] §10); las colas que arrancan y paran mucho pierden rendimiento.
3. El rendimiento se mide tras el **periodo de calentamiento**, cuando las colas han alcanzado su patrón estable ([[T08 Estadísticas y análisis de resultados]]).

## Relacionado

[[T04 Cinemática de transportadores]] · [[T07 Lógica de control y señales]] · [[T08 Estadísticas y análisis de resultados]] · [[S01 Mesa de rodillos]] · [[S02 Mesa de cadenas]]
