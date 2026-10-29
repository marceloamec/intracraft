---
title: T03 Tiempo, orden de eventos y determinismo
lang: es
tags: [theory, des, determinism, rng]
---

🌐 [[T03 Time, Event Ordering and Determinism|EN]] · **ES** · [[T03 Zeit, Ereignisreihenfolge und Determinismus|DE]] · [[Inicio (ES)|Índice]]

# T03 Tiempo, orden de eventos y determinismo

**Determinismo** significa: mismo layout + misma semilla ⇒ resultados idénticos, evento a evento. Es lo que permite depurar («ejecútalo otra vez y para en 812,4 s») y lo que da sentido a comparar dos variantes de layout. En IntraCraft, una prueba de determinismo que falla es un error de prioridad máxima.

## 1. Por qué el tiempo es un entero (microsegundos)

Los números en coma flotante no son exactos: $0{,}1 + 0{,}2 = 0{,}30000000000000004$ y, en general, $(a + b) + c \ne a + (b + c)$. En un motor DES esto causa dos problemas:

1. **Deriva** — sumar muchas duraciones pequeñas acumula error; tras 10 000 ciclos el reloj no está donde debería.
2. **Empates falsos / órdenes falsos** — dos eventos que deberían ser simultáneos quedan separados por unos pocos ulps, y su orden depende del camino aritmético que los produjo.

IntraCraft guarda el tiempo como un **número entero de microsegundos** (`SimTime`). Los enteros se suman de forma exacta. JavaScript representa enteros exactos hasta $2^{53}$, es decir $9 \times 10^{15}$ µs ≈ **285 años** de tiempo simulado: más que suficiente.

**Política de redondeo.** Las predicciones cinemáticas se calculan en segundos de coma flotante (llevan raíces cuadradas) y se convierten una sola vez con `secondsToSimTime(t) = round(t · 10⁶)`. El error máximo es 0,5 µs por evento, cinco órdenes de magnitud por debajo de la tolerancia de ±50 ms de los sensores. `EventQueue.push` rechaza tiempos no enteros.

## 2. Un orden total de los eventos

La lista de eventos futuros se ordena por la clave **(tiempo, prioridad, secuencia)**:

| Clave      | Significado                                     | Por qué                                 |
| ---------- | ----------------------------------------------- | --------------------------------------- |
| `time`     | Cuándo ocurre el evento (µs)                    | Causalidad                              |
| `priority` | Menor primero, entre eventos del mismo instante | Codifica «qué ocurre lógicamente antes» |
| `seq`      | Contador creciente asignado al insertar         | Hace reproducible cualquier empate      |

Como `seq` es único, la clave es un **orden total**: dos eventos nunca son iguales, así que el orden de proceso no depende de los detalles internos del montículo. (Un montículo binario *no* es estable; sin `seq`, claves iguales podrían salir en cualquier orden.)

**Bandas de prioridad propuestas** (a confirmar en `docs/*/decisions`):

| Banda | Eventos                                          |
| ----- | ------------------------------------------------ |
| 0     | Cinemáticos/físicos: llegadas, paradas, entregas |
| 10    | Flancos de sensores                              |
| 20    | Cambios de señal                                 |
| 30    | Evaluación de reglas del controlador             |
| 40    | Muestreo de estadísticas                         |

Con estas bandas, un controlador que se dispara en el mismo microsegundo que un flanco de sensor siempre ve el valor nuevo del sensor.

## 3. El montículo binario

La cola es un montículo binario de mínimos guardado en un array: el padre del nodo $i$ es $\lfloor (i-1)/2 \rfloor$ y los hijos son $2i+1$ y $2i+2$. `push` añade al final y *sube* el elemento; `pop` lleva el último a la raíz y lo *hunde*. Ambos son $O(\log n)$; `peek` es $O(1)$. Implementado en `packages/sim-des/src/event-queue.ts` y probado en `simulator.test.ts`.

## 4. Números pseudoaleatorios con semilla

Las entradas estocásticas (tiempos entre llegadas, masas de carga) salen de un **generador de números pseudoaleatorios** (PRNG): una función determinista que produce una secuencia que *parece* aleatoria y que queda totalmente fijada por su **semilla**.

`Math.random()` está prohibido en los motores: no admite semilla y su algoritmo cambia entre navegadores. Una regla de ESLint lo impide en `sim-des`, `sim-physics` y `control`.

### PCG32 (implementado en `core-model/src/rng.ts`)

PCG = *generador congruencial permutado*. Tiene dos etapas:

1. **Transición de estado** — un generador congruencial lineal de 64 bits: $s_{n+1} = (a \cdot s_n + c) \bmod 2^{64}$, con $a = 6364136223846793005$ y un incremento impar $c$ elegido por el número de **flujo** (stream).
2. **Permutación de salida (XSH-RR)** — desplazamiento con xor de los bits altos y rotación a la derecha según los 5 bits superiores del estado; da una salida de 32 bits de calidad estadística muy superior a la del LCG puro.

Propiedades: periodo $2^{64}$, $2^{63}$ flujos independientes, estado mínimo, rápido y supera las baterías estadísticas habituales. Se usa `BigInt` de JavaScript para la aritmética exacta de 64 bits.

### De uniformes a distribuciones

| Distribución         | Método                                                    |
| -------------------- | --------------------------------------------------------- |
| Uniforme $[a, b)$    | $a + (b - a)\,u$                                          |
| Exponencial (tasa λ) | Transformada inversa: $-\ln(1 - u)/\lambda$               |
| Normal (μ, σ)        | Box–Muller: $\mu + \sigma \sqrt{-2\ln u_1}\cos(2\pi u_2)$ |
| Tabla empírica       | Inversa de la tabla acumulada                             |

### Un flujo por objeto (propuesta)

Si todos los objetos comparten un generador, añadir un generador de pallets cambia los números aleatorios que reciben *los demás*: dos variantes de layout ya no se comparan en las mismas condiciones. La propuesta deriva un flujo para cada objeto a partir de `hash(semillaProyecto, idObjeto)`. Es la técnica clásica de **números aleatorios comunes** para comparar alternativas.

## 5. Lista de comprobación de determinismo

- Tiempo en µs enteros; eventos ordenados por (tiempo, prioridad, secuencia).
- Nada de `Math.random()`, `Date.now()` ni hora del reloj dentro de los motores.
- Recorrer colecciones en un orden definido (`Map`/`Set` de JavaScript conservan el orden de inserción: mantener la inserción determinista; ordenar por id ante la duda).
- No depender de la identidad de objetos, de códigos hash ni del orden de `for…in` con claves numéricas.
- En 3D: paso fijo, mismo orden de creación de cuerpos, mismos ajustes del solver y una compilación del motor con garantías de determinismo ([[T15 Investigación de motores de física]]).
- **Calcular un hash del resultado**: la traza de eventos 2D (`time`, `type`, `target` por evento) y los estados de los cuerpos 3D cada N pasos; dos ejecuciones deben dar el mismo hash (prueba S1-g y prueba de determinismo del DES).

## Relacionado

[[T02 Simulación de eventos discretos]] · [[T10 Integración temporal y paso fijo]] · [[A13 Convenciones]] · [[A11 Pruebas y validación]]
