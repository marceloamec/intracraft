---
title: S06 Fotocélula
lang: es
tags: [spec, sensor]
library-id: sensor.photoeye
status: spec
updated: 2026-10-29
---

🌐 [[S06 Photoelectric Sensor|EN]] · **ES** · [[S06 Lichtschranke|DE]] · [[Inicio (ES)|Índice]]

# S06 Fotocélula (`sensor.photoeye`)

Teoría: [[T06 Detección fotoeléctrica]].

**Requisito del responsable (29-10-2026):** la posición **y la altura** de la fotocélula y de su reflector son parámetros de cada instancia; las elige el usuario. Por tanto el haz es el segmento entre dos puntos elegidos por el usuario, no una línea horizontal fija.

## 1. Parámetros

| Clave              | Clave del plan    | Unidad | Defecto                   | Rango                                   | Descripción                                                                         |
| ------------------ | ----------------- | ------ | ------------------------- | --------------------------------------- | ----------------------------------------------------------------------------------- |
| `type`             | `tipo`            | enum   | reflex (a confirmar)      | difusa, reflex, barrera                 | Reflex: fotocélula + reflector; barrera: emisor + receptor; difusa: solo fotocélula |
| `host`             | `anfitrion`       | ref    | —                         | —                                       | Transportador donde se monta (posiciones en su marco)                               |
| `sensorS`          | `s`               | mm     | 0                         | 0 … longitud                            | Posición de la fotocélula a lo largo del anfitrión                                  |
| `sensorLateral`    | `lateral`         | mm     | bastidor izquierdo (auto) | libre                                   | Posición transversal de la fotocélula respecto al eje                               |
| `sensorHeight`     | `altura`          | mm     | 60                        | 0 … 400                                 | Altura del haz en la fotocélula sobre la superficie                                 |
| `reflectorS`       | — (nuevo)         | mm     | = `sensorS`               | 0 … longitud                            | Posición del reflector / receptor a lo largo                                        |
| `reflectorLateral` | — (nuevo)         | mm     | bastidor derecho (auto)   | libre                                   | Posición transversal del reflector / receptor                                       |
| `reflectorHeight`  | — (nuevo)         | mm     | = `sensorHeight`          | 0 … 400                                 | Altura del reflector / receptor                                                     |
| `orientation`      | `orientacion`     | °      | 90                        | 0 … 180                                 | Solo difusa: dirección del haz respecto al avance                                   |
| `range`            | `alcance`         | mm     | 1000 (difusa) / 5000      | 50 … 5000                               | Distancia máxima de detección                                                       |
| `spotDiameter`     | `diametroSpot`    | mm     | 10                        | 1 … 50                                  | Diámetro del haz                                                                    |
| `responseTime`     | `tiempoRespuesta` | ms     | 2                         | 0 … 50                                  | Retardo propio del sensor                                                           |
| `polarity`         | `polaridad`       | enum   | activo al detectar        | activo al detectar, activo al ocultarse | Lógica de la salida                                                                 |
| `filter`           | `filtro`          | ms     | 0                         | 0 … 2000                                | Tiempo mínimo para cambiar de estado; puede salvar huecos                           |
| `cycleDelay`       | `retardoCiclo`    | ms     | 10                        | 0 … 100                                 | Muestreo del controlador (ciclo de PLC)                                             |
| `outputSignal`     | `señalSalida`     | texto  | auto                      | —                                       | Nombre de la señal booleana                                                         |
| `showBeam`         | `mostrarHaz`      | bool   | sí                        | —                                       | Dibuja el haz en 3D (verde libre, rojo detectando)                                  |

Los parámetros `reflector*` se ocultan en la difusa (`dependsOn: type≠diffuse`). El editor coloca los dos extremos con asas en la vista 3D y muestra los valores en mm.

**Derivados (solo lectura):** longitud del haz, ángulo respecto al avance (90° si `sensorS = reflectorS`) e inclinación (0° si ambas alturas son iguales).

## 2. Validaciones y avisos

- Longitud del haz ≤ `range`.
- El haz debe cruzar el espacio que ocupa un pallet sobre el anfitrión; si no, nunca detectará.
- **Aviso de huecos:** para el tipo de pallet y la orientación previstos en el anfitrión, si la altura del haz cae en una banda con huecos, se avisa con la duración esperada del hueco a velocidad nominal y se sugiere una altura en una banda continua o un valor de `filter`. Ejemplos con EUR ([[S10 Tipos de pallet]]):

| Anfitrión (orientación EUR)   | Bandas continuas (sin parpadeo)                | A 60 mm (valor del plan)               |
| ----------------------------- | ---------------------------------------------- | -------------------------------------- |
| Rodillos (lado largo delante) | 122–144 mm (tablas superiores) → p. ej. 130 mm | Huecos de 382,5 mm → 1,53 s a 0,25 m/s |
| Cadenas (lado corto delante)  | 100–122 mm (tablas de unión) → p. ej. 110 mm   | Huecos de 227,5 mm → 0,91 s a 0,25 m/s |

Por encima del pallet (> 144 mm) el haz ve la carga, si la hay.

## 3. Comportamiento

**2D.** El haz es el segmento fotocélula → reflector. Cada parte maciza del pallet (de su perfil de ocupación) produce un intervalo de posiciones del frente durante el cual el segmento está tapado; los flancos son los extremos de esos intervalos, predichos de forma analítica ([[T06 Detección fotoeléctrica]] §4).
**3D.** Raycast (o shape-cast para spots grandes) a lo largo del mismo segmento en cada paso de física, con la marca de tiempo del flanco en el punto medio del paso.

## 4. Pruebas

| Prueba | Criterio                                                                                                | Estado    |
| ------ | ------------------------------------------------------------------------------------------------------- | --------- |
| T6     | Fotocélula y reflector a 2,0 m, haz en banda continua → subida con el frente, bajada con la cola, 4,8 s | ⏳ fase 3 |
| —      | Haz a 60 mm en rodillos con EUR → tres pulsos por pallet; con `filter` de 2000 ms → uno                 | ⏳ fase 3 |
| —      | Haz oblicuo (`sensorS` y `reflectorS` distintos) → flancos iguales a los intervalos analíticos          | ⏳ fase 3 |
| P6     | Flancos 3D y 2D dentro de ±50 ms                                                                        | ⏳ fase 4 |

## Relacionado

[[T06 Detección fotoeléctrica]] · [[S10 Tipos de pallet]] · [[T07 Lógica de control y señales]] · [[Preguntas abiertas]]
