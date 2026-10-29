---
title: Preguntas abiertas
lang: es
tags: [questions, reference]
updated: 2026-10-29
---

🌐 [[Open Questions|EN]] · **ES** · [[Offene Fragen|DE]] · [[Inicio (ES)|Índice]]

# Preguntas abiertas

Cada pregunta indica **qué hay que decidir**, **por qué importa** (qué resultado cambia), **opciones** y **qué se supone mientras tanto**. Las respuestas se pasan a la especificación correspondiente y se marcan como respondidas.

## Respondidas (29-10-2026)

| Tema                    | Respuesta                                                                                                                                                            | Dónde se aplicó                                                                               |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Acumulación por defecto | Ninguna: cada pallet sigue el movimiento de la cadena o del rodillo; si entra otro pallet, el que ya está se mueve; el comportamiento es parámetro de cada instancia | [[S01 Mesa de rodillos]] §3, [[S02 Mesa de cadenas]]                                          |
| Fotocélula              | Altura y posición de la fotocélula y del reflector son parámetros de la instancia                                                                                    | [[S06 Fotocélula]]                                                                            |
| Orientación del pallet  | Longitudinal en rodillos, transversal en cadenas                                                                                                                     | S01, S02, [[T06 Detección fotoeléctrica]]                                                     |
| Tipo de pallet          | Europeo (EUR / EPAL)                                                                                                                                                 | [[S10 Tipos de pallet]]                                                                       |
| Rodillos                | Ø 89 mm, paso 206,3 mm; diámetro y paso se eligen entre pocas opciones                                                                                               | S01 §2, `roller-conveyor.json`                                                                |
| Agentes de IA           | Pensada desde el principio para agentes (directo y MCP); gateway y token propios; solo documentar                                                                    | [[A15 Integración con agentes de IA]], [[ADR-0004 Capa de comandos y MCP para agentes de IA]] |

## Abiertas

### P1. Lista de diámetros y pasos de rodillo
- **Decidir:** qué combinaciones de diámetro y paso ofrece la lista desplegable.
- **Por qué importa:** fija cuántos rodillos hay bajo el pallet (regla de 3 rodillos), el hueco para las cadenas de la transferencia y la velocidad de giro en 3D.
- **Preguntas concretas:** (a) ¿Qué diámetros usas además de 89 mm? (b) ¿Cada diámetro va con un paso fijo (pares) o pueden combinarse libremente? (c) ¿El paso es siempre múltiplo de la cadena de accionamiento? 206,3 mm ≈ 13 × 15,875 mm (cadena de 5/8"). (d) ¿Mantengo 76/133 del plan como opción o lo quito?
- **Supuesto actual:** diámetros {76, 89}, pasos {133, 206,3}, combinables; por defecto 89/206,3.

### P2. Comportamiento exacto sin acumulación (`none`)
- **Decidir:** qué pasa cuando el pallet delantero llega al final y el siguiente transportador no está libre.
- **Por qué importa:** cambia la capacidad efectiva, el tiempo bloqueado y la posición de parada.
- **Preguntas concretas:** (a) ¿El transportador para cuando el frente llega al final del transportador, o cuando lo detecta un sensor de fin colocado X mm antes? ¿Cuánto es X? (b) ¿Frena con la deceleración normal (el pallet se pasa un poco del sensor: $v^2/2b$ = 62,5 mm a 0,25 m/s) o se para en una posición exacta? (c) Mientras está parado con un pallet al final, ¿puede entrar otro pallet? (En `none` eso obligaría a mover el delantero, así que supongo que **no**.) (d) ¿Hay siempre un sensor de fin por defecto?
- **Supuesto actual:** sensor de fin virtual en el extremo; el transportador frena con su rampa; no recibe mientras está bloqueado.

### P3. Velocidad en la entrega entre dos transportadores con velocidades distintas
- **Decidir:** a qué velocidad va un pallet que está a caballo entre A y B en el modo 2D.
- **Por qué importa:** tiempos de tránsito y consistencia 2D/3D (±5 %).
- **Opciones:** (1) la del más lento; (2) la del que recibe; (3) la del que entrega hasta que el centro de masas pasa al otro.
- **Supuesto actual:** la del más lento.

### P4. Mesa de cadenas con EUR transversal: ancho y guías
- **Decidir:** ancho útil por defecto y si hay guías laterales.
- **Por qué importa:** con 1200 mm transversales, un ancho de 1000 mm no pasa la validación.
- **Preguntas concretas:** (a) ¿El pallet va entre bastidores/guías (ancho ≥ 1250 mm) o sobresale de las cadenas sin guías? (b) ¿Cuántos ramales y a qué distancia entre ellos?
- **Supuesto actual:** ancho útil 1300 mm, 3 ramales, guías a ambos lados.

### P5. Giratoria de cadenas con EUR transversal: tamaño de plataforma
- **Decidir:** dimensiones por defecto de la plataforma.
- **Por qué importa:** el plan pone 1400 (avance) × 1200 (transversal); un EUR transversal ocupa 800 × 1200, justo el ancho. Afecta al radio de barrido y a las distancias a los vecinos.
- **Pregunta concreta:** ¿Qué medidas tienen tus giratorias de cadenas y de rodillos?
- **Supuesto actual:** las del plan.

### P6. Mesa de transferencia rodillos→cadenas
- **Decidir:** confirmar el mecanismo y sus valores.
- **Por qué importa:** es el componente de mayor riesgo (fase 8) y su ciclo limita el rendimiento.
- **Preguntas concretas:** (a) ¿Es correcto: rodillos longitudinales + cadenas elevables entre rodillos que suben, desplazan 90° y bajan? (b) Número de cadenas, carrera de elevación (40 mm) y tiempos de subida/bajada (1 s). (c) ¿La mesa de cadenas contigua está 40 mm más alta que los rodillos, o las cadenas bajan para entregar a la misma altura? (d) ¿Lleva tope de posicionamiento?
- **Supuesto actual:** interpretación del plan; con Ø89/paso 206,3 el hueco entre rodillos es 117,3 mm (caben cadenas de hasta ~107 mm).

### P7. Fotocélulas: tipo y alturas habituales
- **Decidir:** tipo por defecto y altura al colocar una fotocélula nueva.
- **Por qué importa:** con EUR, a 60 mm la señal parpadea (huecos de 382,5 mm en rodillos y 227,5 mm en cadenas); las bandas continuas son ≈ 122–144 mm en rodillos y 100–122 mm en cadenas.
- **Preguntas concretas:** (a) ¿Usas reflex (fotocélula + reflector) o barrera? (b) ¿A qué alturas las montas normalmente en rodillos y en cadenas? (c) ¿Te interesan haces oblicuos o inclinados, o siempre perpendiculares y horizontales? (d) ¿Usáis filtros/temporizaciones en el PLC para los huecos?
- **Supuesto actual:** reflex, 60 mm (plan) con aviso de huecos.

### P8. Cargas
- **Decidir:** masa y altura de carga típicas y máximas, y si son fijas o variables.
- **Por qué importa:** inercia, par de las giratorias, energía de impacto en topes, riesgo de vuelco.
- **Supuesto actual:** 600 kg y 1200 mm de alto; máximo 1000 kg (¿hasta 1500 kg?).

### P9. Velocidades y rampas reales
- **Decidir:** valores por defecto de tu equipo para rodillos, cadenas, giratorias y transferencias.
- **Supuesto actual:** 0,25 m/s y 0,5 m/s²; giratoria 15 °/s y 30 °/s²; transferencia 0,2 m/s.

### P10. Modelos de referencia para contrastar
- **Decidir:** tres modelos que conozcas bien (flujo simple, acumulación, giratoria o transferencia) con sus resultados (rendimiento, tiempo de ciclo, ocupación).
- **Por qué importa:** criterio 8 de la definición de «hecho» del MVP.

### P11. Agentes de IA
- **Preguntas concretas:** (a) ¿Qué proveedores o gateways debe admitir primero el chat integrado (Anthropic directo, gateway de empresa, otros proveedores, modelos locales)? (b) ¿Qué clientes MCP vas a usar? (c) ¿Puede un agente modificar el layout sin confirmación, o siempre pide permiso? (d) ¿Solo en local o también acceso remoto al servidor MCP?
- **Supuesto actual:** solo lectura por defecto; confirmación para borrar o sobrescribir; MCP solo en local.

### P12. Equipo para el modo 3D, licencia y nombre
- (a) ¿Portátil con GPU dedicada o también equipos modestos? (b) ¿Código abierto, uso interno o comercial? (c) ¿Comprobamos la marca IntraCraft (dominio, GitHub, npm)?

### P13. Medidas reales del pallet
- **Decidir:** si tus pallets coinciden con la construcción EPAL usada en [[S10 Tipos de pallet]] (tablas inferiores 100/145/100 mm, tacos de 78 mm, tablas de unión de 22 mm, 5 tablas superiores).
- **Por qué importa:** de ahí salen los huecos que ven las fotocélulas y los apoyos sobre cadenas.
- **Petición:** si puedes, mide un pallet real (posiciones de tacos y tablas).
