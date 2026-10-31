---
title: Glossary
lang: en
tags: [reference, glossary]
---

🌐 **EN** · [[Glosario|ES]] · [[Glossar|DE]] · [[Home (EN)|Index]]

# Glossary

| English                              | Español                                  | Deutsch                             | Meaning in IntraCraft                                                        |
| ------------------------------------ | ---------------------------------------- | ----------------------------------- | ---------------------------------------------------------------------------- |
| Accumulation                         | Acumulación                              | Stau / Staubildung                  | Pallets waiting one behind the other on a conveyor                           |
| Activation (code)                    | Activación (código)                      | Aktivierung (Code)                  | One run of an event handler; it can wait and resume later (A17)              |
| Area                                 | Zona                                     | Bereich                             | Rectangle on the plan that contains objects and gives them a path (S22)      |
| Bottom board                         | Tabla inferior                           | Bodenbrett                          | Lower pallet board that rests on rollers or chains                           |
| Block (pallet)                       | Taco                                     | Klotz                               | Spacer block between the bottom and top boards                               |
| Brake roller                         | Rodillo freno                            | Bremsrolle                          | Roller of a gravity line that holds the pallet at a controlled speed         |
| Chain conveyor                       | Mesa de cadenas                          | Kettenförderer                      | Conveyor where the pallet rests on parallel chain strands                    |
| Code block                           | Bloque de código                         | Codebaustein                        | Object on the plan that holds IntraScript code (S21)                         |
| Coefficient of friction (μ)          | Coeficiente de rozamiento (μ)            | Reibkoeffizient (μ)                 | Ratio of maximum tangential force to normal force                            |
| Coefficient of restitution (e)       | Coeficiente de restitución (e)           | Stoßzahl (e)                        | Ratio of separation to approach speed after an impact                        |
| Collider                             | Colisionador                             | Kollisionskörper (Collider)         | Geometric shape used by the physics engine for contacts                      |
| Continuous collision detection (CCD) | Detección continua de colisiones         | Kontinuierliche Kollisionserkennung | Prevents fast bodies tunnelling through thin obstacles                       |
| Controller                           | Controlador                              | Steuerung                           | If/Then rules reading sensors and commanding actuators                       |
| Determinism                          | Determinismo                             | Determinismus                       | Same inputs and seed → identical results                                     |
| Discrete-event simulation (DES)      | Simulación de eventos discretos          | Ereignisdiskrete Simulation         | Clock jumps from event to event; 2D mode                                     |
| Edge (signal)                        | Flanco                                   | Flanke                              | Change of a boolean signal (rising 0→1, falling 1→0)                         |
| Elevator (pallet)                    | Elevador de pallets                      | Palettenheber / Vertikalförderer    | Carriage on a mast that moves a pallet between levels                        |
| Event handler                        | Manejador de evento                      | Ereignisbehandlung                  | Code after `on …` that runs when the event occurs (A17 §9)                   |
| Event list (future event list)       | Lista de eventos futuros                 | Ereignisliste                       | Priority queue of scheduled events                                           |
| Fixed time step                      | Paso fijo                                | Fester Zeitschritt                  | Physics advances in equal increments (1/120 s)                               |
| Gravity roller line                  | Línea de rodillos por gravedad           | Schwerkraft-Rollenbahn              | Unpowered sloped roller line; pallets roll down to a pick-up position        |
| Guardrail / side guide               | Guía lateral                             | Seitenführung                       | Static rail limiting lateral pallet position                                 |
| Information object                   | Objeto de información                    | Informationsobjekt                  | Variable, table, list, code block or area: stores or computes, moves nothing |
| Interlock                            | Enclavamiento (interlock)                | Verriegelung                        | Safety condition that blocks an action                                       |
| IntraScript                          | IntraScript                              | IntraScript                         | IntraCraft's own programming language (A17, ADR-0018)                        |
| Kinematic body                       | Cuerpo cinemático                        | Kinematischer Körper                | Body moved by prescribed velocity, unaffected by forces                      |
| Layout                               | Layout                                   | Layout                              | Placed and connected components: the single source of truth                  |
| Lift table                           | Mesa elevadora                           | Hubtisch                            | Deck that changes the pallet's height between two levels                     |
| List (object)                        | Lista (objeto)                           | Liste (Objekt)                      | Typed list on the plan, used as list, queue or stack (S20)                   |
| Occupancy profile                    | Perfil de ocupación                      | Belegungsprofil                     | Solid/void bands of a pallet by height, used by sensors                      |
| Pallet jack (hand pallet truck)      | Transpaleta                              | Handhubwagen                        | Manual truck the operator uses to bring or take a pallet                     |
| Photoelectric sensor (photoeye)      | Fotocélula                               | Lichtschranke / Lichttaster         | Light-beam presence sensor                                                   |
| Port                                 | Puerto                                   | Port / Übergabepunkt                | Point where a component hands over or receives a pallet                      |
| Preset                               | Preset                                   | Voreinstellung                      | Named set of parameter values                                                |
| Raycast                              | Raycast                                  | Strahltest (Raycast)                | Intersection test of a ray against colliders                                 |
| Rigid body                           | Sólido rígido                            | Starrkörper                         | Non-deformable body with mass and inertia                                    |
| Roller conveyor                      | Mesa de rodillos                         | Rollenförderer                      | Conveyor where the pallet rests on driven rollers                            |
| Roller pitch                         | Paso de rodillos                         | Rollenteilung                       | Axis-to-axis distance between rollers                                        |
| Scissor lift table                   | Mesa de tijera                           | Scherenhubtisch                     | Lift table raised by a scissor mechanism                                     |
| Seed                                 | Semilla                                  | Startwert (Seed)                    | Initial value of the pseudo-random generator                                 |
| Separator (pallet)                   | Separador de pallets                     | Palettenvereinzeler                 | Holds the queue back so the pick-up pallet is free of pressure               |
| Sink                                 | Sumidero                                 | Senke                               | Removes pallets from the model                                               |
| Source                               | Generador                                | Quelle                              | Creates pallets                                                              |
| Stop (mechanical)                    | Tope (mecánico)                          | Anschlag (Stopper)                  | Blocks the pallet front at a defined position                                |
| Surface velocity                     | Superficie móvil / velocidad superficial | Oberflächengeschwindigkeit          | Contact property that makes a static surface act like a belt                 |
| Table (object)                       | Tabla (objeto)                           | Tabelle (Objekt)                    | Grid of typed columns on the plan (S19)                                      |
| Teleport                             | Teletransporte                           | Teleport                            | Direct move of a pallet by code; not physical, logged in both modes          |
| Throughput                           | Rendimiento                              | Durchsatz                           | Pallets per unit time                                                        |
| Transfer table (chain transfer)      | Mesa de transferencia                    | Kettenumsetzer / Hubumsetzer        | Lifts chains between rollers to move a pallet at 90°                         |
| Turntable                            | Giratoria                                | Drehtisch                           | Rotating platform that changes flow direction                                |
| Utilisation                          | Utilización                              | Auslastung                          | Fraction of time a resource is working                                       |
| Variable (object)                    | Variable (objeto)                        | Variable (Objekt)                   | Named value on the plan with statistics (S18)                                |
| Virtual commissioning                | Puesta en marcha virtual                 | Virtuelle Inbetriebnahme            | Testing control logic against a simulated plant                              |
| Zero-pressure accumulation (ZPA)     | Acumulación sin presión                  | Staudrucklose Förderung             | Zones run only when the next zone is free, so pallets never touch            |
