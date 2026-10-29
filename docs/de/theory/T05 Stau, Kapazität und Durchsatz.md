---
title: T05 Stau, Kapazität und Durchsatz
lang: de
tags: [theory, des, flow]
---

🌐 [[T05 Accumulation, Capacity and Throughput|EN]] · [[T05 Acumulación, capacidad y rendimiento|ES]] · **DE** · [[Startseite (DE)|Index]]

# T05 Stau, Kapazität und Durchsatz

## 1. Kenngrößen

| Größe                | Symbol    | Definition                                                      |
| -------------------- | --------- | --------------------------------------------------------------- |
| Durchsatz            | $\lambda$ | Ausgehende Paletten je Zeiteinheit (Paletten/s oder Paletten/h) |
| Kapazität (statisch) | $N$       | Höchstzahl Paletten, die auf einen Förderer passen              |
| Durchlaufzeit        | $W$       | Zeit vom Eintritt bis zum Austritt einer Palette                |
| Bestand (WIP)        | $\bar{L}$ | Mittlere Anzahl Paletten in einem Objekt                        |
| Folgezeit            | $h$       | Zeit zwischen zwei Paletten an einem Punkt; $\lambda = 1/h$     |
| Auslastung           | $\rho$    | Zeitanteil, in dem ein Objekt belegt ist                        |

## 2. Abstandsbegrenzter Durchsatz — Test T2

Läuft ein Förderer dauerhaft mit $v$, passieren Paletten der Länge $L$ (in Förderrichtung) mit Lücke $g$ einen Punkt alle $h = (L + g)/v$ Sekunden:

$$\lambda_{max} = \frac{v}{L + g}$$

**T2:** EUR längs ($L = 1{,}2$ m), $g = 0{,}15$ m, $v = 0{,}25$ m/s → $h = 5{,}4$ s, $\lambda_{max} = 0{,}185$ Paletten/s ≈ **667 Paletten/h**. Keine Linie aus diesen Förderern kann mehr schaffen, egal mit welcher Steuerung.

Statische Kapazität: $N = \left\lfloor \dfrac{\text{Länge}}{L + g} \right\rfloor$ → auf einen 3-m-Förderer passen 2 EUR-Paletten.

## 3. Stationsbegrenzter Durchsatz und Engpässe

Drehtische und Kettenumsetzer bedienen **eine Palette nach der anderen**. Ihr Durchsatz ist $1/t_{Zyklus}$; der Zyklus umfasst alle Phasen, bis die Station die nächste Palette annehmen kann.

Beispielhafter Zyklus eines Kettendrehtischs (Beispielwerte, keine Spezifikation):

| Phase                                     | Abschätzung            | Zeit                          |
| ----------------------------------------- | ---------------------- | ----------------------------- |
| Aufnehmen (≈ 1,4 m bei 0,25 m/s + Rampen) | $1{,}4/0{,}25 + 0{,}5$ | 6,1 s                         |
| 90° drehen                                | T3                     | 6,5 s                         |
| Beruhigen                                 | Parameter              | 0,3 s                         |
| Abgeben                                   | wie Aufnehmen          | 6,1 s                         |
| 90° zurück (leer)                         | T3                     | 6,5 s                         |
| **Zyklus**                                |                        | **25,5 s** → ≈ 141 Paletten/h |

Der Liniendurchsatz ist das Minimum über alle Stufen – der **Engpass**. Hier begrenzt der Drehtisch (141/h) eine Linie, deren Förderer 667/h schaffen könnten. Typische Verbesserungen: Leerdrehung vermeiden (abwechselnde Stationen bedienen), Rampen verkürzen, Abgabe und Aufnahme überlappen.

## 4. Littles Gesetz

Für jedes stabile System (Langzeitmittel, kein unbegrenztes Anwachsen):

$$\bar{L} = \lambda \cdot W$$

Beispiel: Ein Pufferabschnitt durchlaufen 120 Paletten/h ($\lambda = 1/30$ s⁻¹), jede verweilt im Mittel 90 s → im Mittel befinden sich $\bar L = 3$ Paletten darin. Littles Gesetz ist eine starke Plausibilitätsprüfung der Simulationsstatistik: Verletzen die drei gemeldeten Werte es, steckt ein Fehler im Statistikcode ([[T08 Statistik und Ergebnisanalyse]]).

## 5. Zustände, Blockieren und Hungern

Jeder Förderer ist immer in genau einem Zustand: **gestoppt, beschleunigend, laufend, verzögernd, blockiert**. Zwei Arten verlorener Zeit dominieren Materialflussstudien:

- **Hungern (starved)** — bereit, aber es kommt nichts (Problem stromaufwärts). Wird als „Leerzeit“ angezeigt.
- **Blockiert** — eine Palette ist fertig, aber das nächste Objekt nimmt sie nicht an (Problem stromabwärts). Förderer verhalten sich wie **endliche Puffer mit Blockierung nach der Bedienung**: Eine fertig transportierte Palette wartet auf dem Förderer und belegt Platz.

Auslastung $\rho = t_{arbeitend}/t_{gesamt}$; ein Engpass hat $\rho \to 1$, seine Vorgänger zeigen Blockierzeit, seine Nachfolger Leerzeit.

## 6. Staumodi

### 6.1 Ohne Stau (`none`, Standard)

Der ganze Förderer läuft oder steht als Einheit, und jede Palette folgt den Rollen oder Ketten. Fährt eine neue Palette ein, muss der Förderer laufen, die vorhandenen Paletten bewegen sich also mit; die Abstände zwischen den Paletten bleiben erhalten. Erreicht die vorderste Palette den Auslauf und kann der nächste Förderer sie nicht übernehmen, steht der ganze Förderer (blockiert) und nimmt nichts mehr auf. Ein längerer Förderer fasst einfach mehr Paletten hintereinander. Das ist der Standard aller Förderer und, wie jedes Verhalten, ein Parameter der Instanz ([[S01 Rollenförderer]] §3).

### 6.2 Staudrucklose Förderung (ZPA)

Der Förderer ist in **Zonen** geteilt (Standard 1,4 m ≥ Palettenlänge + Abstand), jede mit eigenem Antrieb und Zonenend-Sensor. Grundregel:

> Zone *i* läuft, wenn Zone *i + 1* frei ist (oder läuft); sonst stoppt sie, sobald ihr eigener Sensor belegt ist.

So halten die Paletten je Zone einzeln und **berühren sich nie**. Zwei Abfahrmodi:

- **Vereinzelung** — jede Zone startet erst, wenn die vordere frei geworden ist. Die Paletten fahren mit größerem Abstand ab: Die Folgezeit entspricht etwa der Zeit, in der eine Palette aus dem Stand eine Zone räumt, $h \approx L_z/v + v/2a = 1{,}4/0{,}25 + 0{,}25 = 5{,}85$ s (≈ 615/h gegenüber 667/h bei Fließbetrieb). Dieser „Ziehharmonika-Effekt“ ist ein echter Durchsatzverlust, den der 2D-Modus abbilden muss.
- **Blockabzug** — alle Zonen eines Staus starten gemeinsam und halten den Mindestabstand; schneller, braucht aber Platz stromabwärts.

Die Wenn/Dann-Vorlage für ZPA steht in `control/src/rules.test.ts` und in [[T07 Steuerungslogik und Signale]].

### 6.3 Stau mit Staudruck (Ketten mit Schlupf)

Auf Kettenförderern laufen die Ketten unter gehaltenen Paletten weiter und **rutschen** darunter durch. Jede gehaltene Palette wird von der Gleitreibungskraft $F = \mu\,m\,g$ nach vorn gedrückt, und die Kräfte addieren sich entlang des Staus:

$$F_{Anschlag} = \sum_{k=1}^{n} \mu\,m_k\,g$$

Beispiel: 3 Paletten à 625 kg auf Ketten mit $\mu = 0{,}3$ → $F_{Anschlag} = 3 \times 0{,}3 \times 625 \times 9{,}81 \approx 5{,}5$ kN auf den ersten Anschlag (und auf die Bretter der ersten Palette). In 2D halten die Paletten nach `stopContactTime` einfach auf Kontakt mit dem Mindestabstand (in diesem Modus Standard 0); der 3D-Modus berechnet die realen Kontaktkräfte.

## 7. Praktische Folgen für die 2D-Engine

1. Eine Folgepalette kann nie über die Hinterkante ihrer Vorgängerin minus Mindestabstand hinaus – das ist ein **vorhergesagtes Ereignis** (`follower_reaches_leader`), keine Prüfung in jedem Schritt.
2. Jeder Anlauf/Halt kostet $v/2a$ bzw. $v/2b$ gegenüber einer Sofortbewegung ([[T04 Kinematik von Förderern]] §10); Staus mit häufigem Anfahren verlieren Durchsatz.
3. Durchsatz wird nach der **Einschwingphase** gemessen, wenn die Staus ihr stationäres Muster erreicht haben ([[T08 Statistik und Ergebnisanalyse]]).

## Verwandt

[[T04 Kinematik von Förderern]] · [[T07 Steuerungslogik und Signale]] · [[T08 Statistik und Ergebnisanalyse]] · [[S01 Rollenförderer]] · [[S02 Kettenförderer]]
