---
title: T09 Starrkörperdynamik
lang: de
tags: [theory, physics]
---

🌐 [[T09 Rigid Body Dynamics|EN]] · [[T09 Dinámica del sólido rígido|ES]] · **DE** · [[Startseite (DE)|Index]]

# T09 Starrkörperdynamik

## 1. Zustand eines Starrkörpers

Ein Starrkörper verformt sich nicht; sein vollständiger Zustand ist

| Symbol              | Bedeutung             | Hinweise                              |
| ------------------- | --------------------- | ------------------------------------- |
| $\mathbf{x}$        | Lage des Schwerpunkts | Meter, Weltkoordinaten (Y nach oben)  |
| $\mathbf{q}$        | Orientierung          | Einheitsquaternion (kein Gimbal Lock) |
| $\mathbf{v}$        | Lineargeschwindigkeit | m/s                                   |
| $\boldsymbol\omega$ | Winkelgeschwindigkeit | rad/s                                 |

plus konstante **Masseneigenschaften**: Masse $m$, Schwerpunkt und Trägheitstensor $\mathbf{I}$.

## 2. Bewegungsgleichungen (Newton-Euler)

$$m\,\dot{\mathbf{v}} = \mathbf{F} \qquad \mathbf{I}\,\dot{\boldsymbol\omega} + \boldsymbol\omega \times (\mathbf{I}\,\boldsymbol\omega) = \boldsymbol\tau$$

$\mathbf{F}$ ist die Summe der Kräfte (Schwerkraft $m\mathbf{g}$ + Kontaktkräfte), $\boldsymbol\tau$ die Summe der Momente um den Schwerpunkt. Der Trägheitstensor in Weltkoordinaten ist $\mathbf{I}_w = \mathbf{R}\,\mathbf{I}_b\,\mathbf{R}^T$ mit der Drehmatrix $\mathbf{R}$ zu $\mathbf{q}$. Der Kreiselterm $\boldsymbol\omega \times \mathbf{I}\boldsymbol\omega$ ist bei langsam drehenden Paletten vernachlässigbar, die Engine berücksichtigt ihn trotzdem.

In der Praxis wendet die Engine Kontaktkräfte nicht direkt an, sondern berechnet **Impulse**, die die Kontaktbedingungen erfüllen ([[T11 Kontakt, Reibung und Restitution]]).

## 3. Drei Bewegungsarten

| Art         | Bewegt durch                     | Kräfte wirken           | Verwendet für                                              |
| ----------- | -------------------------------- | ----------------------- | ---------------------------------------------------------- |
| Statisch    | Nichts                           | Nein                    | Führungen, Rahmen, Boden                                   |
| Kinematisch | Vorgegebene Geschwindigkeit/Lage | Nein (unendliche Masse) | Rollen, Ketten, Drehtischplattformen, Anschläge, Hubketten |
| Dynamisch   | Kräfte und Kontakte              | Ja                      | Paletten                                                   |

Eine kinematische Rolle „dreht“ mit vorgegebener Winkelgeschwindigkeit; für den Solver hat sie keine Trägheit, eine 1000-kg-Palette kann sie also nicht bremsen. Man nimmt an, dass der Antrieb stark genug ist – Momentengrenzen sind im MVP eine Validierung, keine Simulation.

## 4. Trägheit der benötigten Formen

| Form                                | Achse                                        | Trägheitsmoment             |
| ----------------------------------- | -------------------------------------------- | --------------------------- |
| Vollquader $a \times b \times c$    | Durch Schwerpunkt, ⟂ zur Fläche $a \times b$ | $\frac{m}{12}(a^2 + b^2)$   |
| Vollzylinder, Radius $r$, Länge $h$ | Eigene Achse                                 | $\frac12 m r^2$             |
| Vollzylinder                        | Quer, durch Schwerpunkt                      | $\frac{m}{12}(3r^2 + h^2)$  |
| Rohr (Rollenmantel) $r_o, r_i$      | Eigene Achse                                 | $\frac12 m (r_o^2 + r_i^2)$ |

**Satz von Steiner** — um eine Achse im Abstand $d$ vom Schwerpunkt: $I = I_{cm} + m\,d^2$. So setzt man die Trägheit eines zusammengesetzten Körpers aus seinen Teilen zusammen.

## 5. Die zusammengesetzte Palette

Die Palette ist **kein** einzelner Quader. Sie ist ein dynamischer Körper aus:

- 3 Quadern für die Bodenbretter (die einzigen Teile, die Rollen oder Ketten berühren);
- 1 Quader für den Ladungsblock (mit Ladehöhe und Masse);
- optional Deckbrettern und Klötzen für Sensorstrahlen (oder das Belegungsprofil erzeugt die Kollisionskörper — [[S10 Palettentypen]]).

Die Lücken zwischen den Bodenbrettern sind wichtig: Eine Kette, die in eine Lücke fällt, hebt die Palette nicht an, und ein Sensorstrahl kann zwischen Klötzen hindurchgehen.

### Rechenbeispiel: EUR-Palette mit 600 kg Ladung, 1200 mm hoch

| Teil       | Masse (kg) | Schwerpunkthöhe (m)                                              | Trägheit um die Hochachse (kg·m²)      |
| ---------- | ---------- | ---------------------------------------------------------------- | -------------------------------------- |
| Palette    | 25         | 0,072                                                            | $25(1{,}2^2 + 0{,}8^2)/12 = 4{,}3$     |
| Ladung     | 600        | $0{,}144 + 0{,}6 = 0{,}744$                                      | $600(1{,}2^2 + 0{,}8^2)/12 = 104$      |
| **Gesamt** | **625**    | $\dfrac{25 \cdot 0{,}072 + 600 \cdot 0{,}744}{625} = $ **0,717** | **≈ 108** (beide mittig, kein Versatz) |

Der Parameter `centerOfMassOffset` verschiebt den Ladungsschwerpunkt horizontal, um außermittige Ladungen abzubilden.

## 6. Kippen oder Rutschen – warum die Schwerpunkthöhe zählt

Bei einer Beschleunigung $a$ in Förderrichtung greift die Trägheitskraft $m a$ in Schwerpunkthöhe $h$ an. Momente um die hintere Kante der Auflagefläche (halbe Länge $L/2$):

- Die Palette **kippt**, wenn $a\,h > g\,(L/2)$, also $a_{kipp} = g\,\dfrac{L/2}{h}$.
- Sie **rutscht** vorher, wenn $\mu < \dfrac{L/2}{h}$ (die Reibung ist erschöpft, bevor sie kippt).

Beispiel (Schmalseite 0,8 m in Förderrichtung, $h = 0{,}717$ m): $a_{kipp} = 9{,}81 \times 0{,}4/0{,}717 = 5{,}5$ m/s². Normale Rampen (0,5 m/s²) liegen weit darunter, aber die **Verzögerung beim Aufprall auf einen Anschlag** kann für einige Millisekunden viel größer sein; hohe Ladungen brauchen daher niedrige Schleichgeschwindigkeiten ([[T14 Anschläge, Aufprall und Ausrichtung]]).

## 7. Dämpfung

Kleine lineare und Winkeldämpfung (≈ 0,01–0,05 s⁻¹) entfernt numerisches Zittern. Sie muss klein bleiben: Dämpfung ist eine künstliche Kraft und verändert Durchlaufzeiten. Jede Änderung wird in einer Entscheidungsnotiz festgehalten (Regel aus AGENTS.md).

## 8. Vereinfachungen im MVP

- Die Ladung ist starr mit der Palette verbunden (kein Verrutschen, keine Verformung).
- Rollen und Ketten sind kinematisch: keine Antriebsdynamik, keine Kettenlängung.
- Holz ist starr; die Durchbiegung der Bretter zwischen Rollen wird ignoriert.

## Verwandt

[[T10 Zeitintegration und fester Zeitschritt]] · [[T11 Kontakt, Reibung und Restitution]] · [[T12 Rollen und Ketten in der Physik]] · [[A07 Physik-Wrapper]]
