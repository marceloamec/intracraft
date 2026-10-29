---
title: T03 Zeit, Ereignisreihenfolge und Determinismus
lang: de
tags: [theory, des, determinism, rng]
---

🌐 [[T03 Time, Event Ordering and Determinism|EN]] · [[T03 Tiempo, orden de eventos y determinismo|ES]] · **DE** · [[Startseite (DE)|Index]]

# T03 Zeit, Ereignisreihenfolge und Determinismus

**Determinismus** heißt: gleiches Layout + gleicher Seed ⇒ identische Ergebnisse, Ereignis für Ereignis. Nur so ist Fehlersuche möglich („noch einmal laufen lassen und bei 812,4 s anhalten“), und nur so ist der Vergleich zweier Layoutvarianten aussagekräftig. In IntraCraft ist ein fehlschlagender Determinismustest ein Fehler höchster Priorität.

## 1. Warum die Zeit eine Ganzzahl ist (Mikrosekunden)

Gleitkommazahlen sind nicht exakt: $0{,}1 + 0{,}2 = 0{,}30000000000000004$ und im Allgemeinen $(a + b) + c \ne a + (b + c)$. In einer DES-Engine führt das zu zwei Problemen:

1. **Drift** — viele kleine Dauern aufzusummieren häuft Fehler an; nach 10 000 Zyklen steht die Uhr nicht dort, wo sie sein sollte.
2. **Falsche Gleichstände / falsche Reihenfolgen** — zwei eigentlich gleichzeitige Ereignisse liegen ein paar ulps auseinander, und ihre Reihenfolge hängt vom Rechenweg ab.

IntraCraft speichert die Zeit als **ganzzahlige Mikrosekunden** (`SimTime`). Ganzzahlen addieren sich exakt. JavaScript stellt Ganzzahlen bis $2^{53}$ exakt dar, also $9 \times 10^{15}$ µs ≈ **285 Jahre** Simulationszeit – mehr als genug.

**Rundungsregel.** Kinematische Vorhersagen werden in Gleitkomma-Sekunden berechnet (sie enthalten Wurzeln) und genau einmal mit `secondsToSimTime(t) = round(t · 10⁶)` umgerechnet. Der Fehler beträgt höchstens 0,5 µs je Ereignis – fünf Größenordnungen unter der Sensortoleranz von ±50 ms. `EventQueue.push` weist nicht ganzzahlige Zeiten zurück.

## 2. Eine totale Ordnung der Ereignisse

Die Ereignisliste ist nach dem Schlüssel **(Zeit, Priorität, Sequenz)** geordnet:

| Schlüssel  | Bedeutung                          | Warum                                  |
| ---------- | ---------------------------------- | -------------------------------------- |
| `time`     | Wann das Ereignis eintritt (µs)    | Kausalität                             |
| `priority` | Kleiner zuerst, bei gleicher Zeit  | Kodiert, „was logisch zuerst passiert“ |
| `seq`      | Aufsteigender Zähler beim Einfügen | Macht jeden Gleichstand reproduzierbar |

Da `seq` eindeutig ist, ist der Schlüssel eine **totale Ordnung**: Keine zwei Ereignisse sind gleich, die Abarbeitungsreihenfolge hängt also nie von Heap-Interna ab. (Ein Binär-Heap ist *nicht* stabil; ohne `seq` könnten gleiche Schlüssel in beliebiger Reihenfolge herauskommen.)

**Vorgeschlagene Prioritätsbänder** (in `docs/*/decisions` zu bestätigen):

| Band | Ereignisse                                           |
| ---- | ---------------------------------------------------- |
| 0    | Kinematisch/physikalisch: Ankünfte, Halte, Übergaben |
| 10   | Sensorflanken                                        |
| 20   | Signaländerungen                                     |
| 30   | Auswertung der Steuerungsregeln                      |
| 40   | Statistikabtastung                                   |

Mit diesen Bändern sieht eine Steuerung, die in derselben Mikrosekunde wie eine Sensorflanke auslöst, immer schon den neuen Sensorwert.

## 3. Der Binär-Heap

Die Warteschlange ist ein Min-Heap in einem Array: Elternknoten von $i$ ist $\lfloor (i-1)/2 \rfloor$, Kinder sind $2i+1$ und $2i+2$. `push` hängt an und lässt *aufsteigen*; `pop` setzt das letzte Element an die Wurzel und lässt es *absinken*. Beides $O(\log n)$; `peek` ist $O(1)$. Umgesetzt in `packages/sim-des/src/event-queue.ts`, getestet in `simulator.test.ts`.

## 4. Pseudozufallszahlen mit Seed

Stochastische Eingaben (Zwischenankunftszeiten, Lastmassen) stammen aus einem **Pseudozufallszahlengenerator** (PRNG): einer deterministischen Funktion, die eine zufällig *aussehende* Folge erzeugt, vollständig durch ihren **Seed** bestimmt.

`Math.random()` ist in den Engines verboten: Es lässt sich nicht seeden, und der Algorithmus unterscheidet sich je Browser. Eine ESLint-Regel erzwingt das in `sim-des`, `sim-physics` und `control`.

### PCG32 (umgesetzt in `core-model/src/rng.ts`)

PCG = *permutierter Kongruenzgenerator*. Zwei Stufen:

1. **Zustandsübergang** — ein linearer 64-Bit-Kongruenzgenerator: $s_{n+1} = (a \cdot s_n + c) \bmod 2^{64}$ mit $a = 6364136223846793005$ und einem ungeraden Inkrement $c$, das die **Stream**-Nummer festlegt.
2. **Ausgabepermutation (XSH-RR)** — Xor-Shift der oberen Bits, dann Rechtsrotation um die oberen 5 Zustandsbits; ergibt 32-Bit-Werte mit weit besserer statistischer Qualität als der reine LCG.

Eigenschaften: Periode $2^{64}$, $2^{63}$ unabhängige Streams, winziger Zustand, schnell, besteht gängige Testbatterien. Für exakte 64-Bit-Arithmetik wird JavaScript-`BigInt` verwendet.

### Von Gleichverteilung zu Verteilungen

| Verteilung              | Verfahren                                                 |
| ----------------------- | --------------------------------------------------------- |
| Gleichverteilt $[a, b)$ | $a + (b - a)\,u$                                          |
| Exponential (Rate λ)    | Inversionsmethode: $-\ln(1 - u)/\lambda$                  |
| Normal (μ, σ)           | Box-Muller: $\mu + \sigma \sqrt{-2\ln u_1}\cos(2\pi u_2)$ |
| Empirische Tabelle      | Inverse der kumulierten Tabelle                           |

### Ein Stream je Objekt (Vorschlag)

Teilen sich alle Objekte einen Generator, ändert eine zusätzliche Quelle die Zufallszahlen *aller anderen* Quellen – zwei Layoutvarianten werden nicht mehr unter gleichen Bedingungen verglichen. Vorgeschlagen ist, für jedes Objekt einen Stream aus `hash(Projekt-Seed, Objekt-Id)` abzuleiten. Das ist die klassische Technik der **gemeinsamen Zufallszahlen** (common random numbers) zum Variantenvergleich.

## 5. Checkliste Determinismus

- Zeit in ganzzahligen µs; Ereignisse nach (Zeit, Priorität, Sequenz) geordnet.
- Kein `Math.random()`, `Date.now()` oder Uhrzeit in den Engines.
- Sammlungen in definierter Reihenfolge durchlaufen (`Map`/`Set` behalten die Einfügereihenfolge – Einfügen deterministisch halten; im Zweifel nach Id sortieren).
- Keine Abhängigkeit von Objektidentität, Hashcodes oder der `for…in`-Reihenfolge numerischer Schlüssel.
- In 3D: fester Zeitschritt, gleiche Reihenfolge beim Anlegen der Körper, gleiche Solver-Einstellungen, Engine-Build mit Determinismusgarantie ([[T15 Recherche Physik-Engines]]).
- **Ergebnis hashen**: Die 2D-Ereignisspur (`time`, `type`, `target` je Ereignis) und die 3D-Körperzustände alle N Schritte werden gehasht; zwei Läufe müssen denselben Hash liefern (Test S1-g und Determinismustest der DES).

## Verwandt

[[T02 Ereignisdiskrete Simulation]] · [[T10 Zeitintegration und fester Zeitschritt]] · [[A13 Konventionen]] · [[A11 Tests und Validierung]]
