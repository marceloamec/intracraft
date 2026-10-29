---
title: T08 Statistik und Ergebnisanalyse
lang: de
tags: [theory, statistics]
---

🌐 [[T08 Statistics and Output Analysis|EN]] · [[T08 Estadísticas y análisis de resultados|ES]] · **DE** · [[Startseite (DE)|Index]]

# T08 Statistik und Ergebnisanalyse

## 1. Zwei Arten von Statistik

| Art                                   | Beispiele                                  | Berechnung                                                  |
| ------------------------------------- | ------------------------------------------ | ----------------------------------------------------------- |
| **Beobachtungsbasiert** (Stichproben) | Durchlaufzeit, Zykluszeit, Aufprallenergie | Ein Wert je Palette/Ereignis: Mittel, Min., Max., Std.-Abw. |
| **Zeitgewichtet**                     | Belegung, Staulänge, Auslastung            | Wert gewichtet mit seiner Dauer                             |

Sie zu verwechseln ist der häufigste Statistikfehler: Die mittlere Staulänge ist **nicht** der Mittelwert der bei Ereignissen beobachteten Staulängen.

## 2. Zeitgewichtete Mittelwerte in einer ereignisgesteuerten Engine

Eine Zustandsvariable $x(t)$ (z. B. Anzahl Paletten auf einem Förderer) ist zwischen Ereignissen stückweise konstant. Ihr Zeitmittel über $[0, T]$ ist

$$\bar{x} = \frac{1}{T}\int_0^T x(t)\,dt = \frac{1}{T}\sum_i x_i\,\Delta t_i$$

Umsetzung: `lastValue` und `lastChangeTime` merken; bei jeder Änderung `lastValue × (jetzt − lastChangeTime)` aufaddieren. Am Ende das letzte Intervall schließen. Mit ganzzahligen Mikrosekunden ist die Summation exakt.

## 3. Zustandszeiten und Standardstatistiken

Jeder Förderer ist immer in genau einem Zustand (gestoppt, beschleunigend, laufend, verzögernd, blockiert); die Engine summiert die Zeit je Zustand. Daraus:

| Kennzahl               | Definition                                              |
| ---------------------- | ------------------------------------------------------- |
| Auslastung             | Zeit mit bewegter Palette / Messzeit                    |
| Blockierzeit           | Zeit mit fertiger Palette, die nicht weiter kann        |
| Leerzeit               | Zeit ohne Palette                                       |
| Bearbeitete Paletten   | Zähler der Paletten, die über einen Ausgangsport gingen |
| Durchsatz (Paletten/h) | Bearbeitete Paletten / Messzeit × 3600                  |
| Mittlere Durchlaufzeit | Mittel von (Austritt − Eintritt)                        |
| Mittlere Belegung      | Zeitgewichtete Palettenanzahl                           |

**Plausibilitätsprüfung:** Littles Gesetz $\bar{L} = \lambda W$ muss zwischen mittlerer Belegung, Durchsatz und mittlerer Durchlaufzeit gelten ([[T05 Stau, Kapazität und Durchsatz]]).

## 4. Numerisch stabiler Mittelwert und Varianz

Für Stichproben wird der **Welford-Algorithmus** verwendet, statt $x$ und $x^2$ aufzusummieren (was Genauigkeit kostet):

```text
n ← n + 1
δ ← x − mittel
mittel ← mittel + δ / n
M2 ← M2 + δ · (x − mittel)
varianz = M2 / (n − 1)
```

## 5. Einschwingphase (Anlaufverhalten)

Ein Modell startet meist **leer**, was nicht repräsentativ ist: Die ersten Paletten finden keinen Stau vor und sind schneller. Ab $t = 0$ erfasste Statistik ist verzerrt.

- Eine **Einschwingdauer** wählen, nach der die Statistik zurückgesetzt wird (MVP: Szenarioparameter).
- Sie wird gefunden, indem man einen gleitenden Mittelwert der Ausgabe (z. B. Durchsatz je 10 min) aufträgt und schaut, wo er flach wird (*Welch-Verfahren*).
- Bei einer Linie aus 3-m-Förderern ist die Einschwingphase kurz (Füllzeit der Linie); bei Kreisläufen mit Drehtischen können es Dutzende Minuten sein.

## 6. Replikationen und Konfidenzintervalle

Bei **stochastischen** Eingaben ist ein Lauf eine Stichprobe. Man führt $n$ unabhängige Replikationen aus (verschiedene Seeds), berechnet je Lauf das Ergebnis $Y_j$ und meldet

$$\bar{Y} \pm t_{n-1;\,0{,}975}\,\frac{s}{\sqrt{n}}$$

Beispiel: 5 Replikationen liefern 612, 598, 620, 605, 615 Paletten/h → $\bar{Y} = 610$, $s = 8{,}6$, $t_{4;0{,}975} = 2{,}776$ → **610 ± 10,7 Paletten/h** (95 %). Überlappen sich die Intervalle zweier Layoutvarianten, hat die Simulation noch keinen Unterschied gezeigt.

Ein **deterministisches** Modell (feste Intervalle, feste Massen) braucht einen Lauf. Replikationen und Experimente kommen nach dem MVP, aber Seed-Generator und Streams je Objekt sind bereits dafür ausgelegt ([[T03 Zeit, Ereignisreihenfolge und Determinismus]]).

## 7. Messgrößen des Physikmodus

Der 3D-Modus liefert Größen ohne 2D-Gegenstück:

- **Aufpralle**: Geschwindigkeit und kinetische Energie bei jedem Anschlagkontakt, Warnung oberhalb `maxImpactEnergy`.
- **Endlage**: Positions- und Winkelfehler einer am Anschlag ruhenden Palette (Ausrichtungsqualität).
- **Kontaktkräfte**: z. B. Staudruck auf einen Anschlag bei Stau mit Schlupf.

## 8. Ereignisprotokoll und Export

Jedes verarbeitete Ereignis kann protokolliert werden (`time`, `type`, `target`, Zusatzdaten). Das Protokoll wird:

- als CSV/JSON exportiert, um es mit anderen Werkzeugen auszuwerten;
- für den Determinismustest gehasht;
- nie in der Projektdatei gespeichert (Ergebnisse liegen getrennt, Plan §4.4).

## Verwandt

[[T05 Stau, Kapazität und Durchsatz]] · [[A11 Tests und Validierung]] · [[A10 CLI und Szenarien]]
