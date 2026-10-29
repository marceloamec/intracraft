---
title: A11 Tests und Validierung
lang: de
tags: [app, testing, validation]
---

🌐 [[A11 Testing and Validation|EN]] · [[A11 Pruebas y validación|ES]] · **DE** · [[Startseite (DE)|Index]]

# A11 Tests und Validierung

**Erst der Test, dann die Umsetzung.** Erwartete Ergebnisse stammen aus der Spezifikation, nie aus der Implementierung.

## 1. Testebenen

1. **Unit** — Einheiten, Kinematik, Ereigniswarteschlange, Parametervalidierung.
2. **2D-Szenarien mit analytischem Ergebnis** (§2).
3. **Physikszenarien** (§3).
4. **Konsistenz zwischen den Modi** (§4).
5. **Regression mit Referenzergebnissen** und **Determinismus** (§5).
6. **Vergleich mit bekannten Referenzmodellen** (§6).

## 2. Analytische Tests des 2D-Modus (Fehler < 1 %)

| Id  | Szenario                                                          | Erwartet                                                 | Stand                 |
| --- | ----------------------------------------------------------------- | -------------------------------------------------------- | --------------------- |
| T1  | Eine Palette, 3 m aus dem Stand, 0,25 m/s, 0,5 m/s², ohne Bremsen | $D/v + v/2a$ = 12,25 s                                   | ✅ Formel             |
| T2  | Sättigung, EUR längs (1,2 m), Lücke 0,15 m                        | $v/(L+g)$ ≈ 0,185 Paletten/s ≈ 667/h                     | ✅ Formel             |
| T3  | 90°-Drehung, 15 °/s, 30 °/s²                                      | 6,5 s                                                    | ✅ Formel             |
| T4  | Umsetzen: 1 m, 0,2 m/s, 0,5 m/s², je 1 s Heben und Senken         | 7,4 s                                                    | ✅ Formel             |
| T5  | Aufprallenergie 1025 kg bei 0,25 m/s                              | 32 J                                                     | ✅ Formel             |
| T6  | Sensor bei 2,0 m, hoher Strahl                                    | Steigend mit Vorderkante, fallend mit Hinterkante; 4,8 s | ⏳ braucht DES-Bahnen |

„✅ Formel“ = die geschlossenen Funktionen bestehen; der vollständige DES-Lauf jedes Layouts folgt in den Phasen 3–8.

## 3. Physikszenarien

| Id  | Szenario             | Kriterium                                                           |
| --- | -------------------- | ------------------------------------------------------------------- |
| P1  | Anschlag             | Vorderkante ±5 mm; Rückprall < 10 mm                                |
| P2  | Aufprall             | Geschwindigkeit und Energie erfasst; Warnung über `maxImpactEnergy` |
| P3  | Führung mit Trichter | 50 mm und 5° auf ±10 mm und ±1° korrigiert                          |
| P4  | 90°-Drehung          | Palettenversatz < 5 mm; kein Rutschen bei μ ≥ 0,04                  |
| P5  | Umsetzen             | 40 ± 5 mm gehoben, 1 m ± 10 mm verfahren, < 1° verdreht             |
| P6  | Sensoren             | 3D- und 2D-Flanken innerhalb ±50 ms                                 |

Die Kriterien a–g des Spikes S1 stehen in [[T15 Recherche Physik-Engines]] und [[ADR-0002 Physik-Engine-Kandidat]].

## 4. Konsistenz zwischen den Modi

Gleiches Szenario in beiden Modi: Durchlaufzeiten ±5 %, Sensorflanken ±50 ms. Größere Abweichungen sind Befunde ([[T16 Hybride Konsistenz und Kalibrierung]]).

## 5. Regression und Determinismus

- Referenzergebnisse je Szenario in `test_data/`. Jede Änderung muss in Commit-Nachricht und Änderungsprotokoll begründet werden. Nie ändern, nur damit ein Test besteht.
- Zwei Läufe mit gleichem Seed müssen denselben Hash des Ereignisprotokolls (2D) und der Körperzustände (3D) liefern. Heute bereits getestet: identische Ereignisspuren (`simulator.test.ts`) und identische Zufallsfolgen (`rng.test.ts`).

## 6. Vergleich mit Referenzmodellen

Drei Modelle wählen, die das Team gut kennt (einfacher Fluss, Stau, Drehtisch/Umsetzer), in 2D nachbauen und Durchsatz, mittlere Zykluszeit und Belegung vergleichen. Abweichungen dokumentieren statt erzwingen; verglichen werden nur beobachtbare Ergebnisse (Clean-Room-Regel).

## 7. Aktueller Testbestand (28.10.2026)

`pnpm test` → **11 Dateien, 62 Tests, alle grün** (29.10.2026); `pnpm lint` (ESLint + Typprüfung) fehlerfrei; `pnpm run scenario all` → 5/5.

| Paket       | Testdatei(en)                                    |
| ----------- | ------------------------------------------------ |
| core-model  | `units.test.ts`, `rng.test.ts`, `schema.test.ts` |
| sim-des     | `kinematics.test.ts`, `simulator.test.ts`        |
| sim-physics | `mechanics.test.ts`                              |
| control     | `rules.test.ts`                                  |
| geometry    | `rollers.test.ts`                                |
| library     | `library.test.ts`                                |
| scenarios   | `analytic.test.ts`                               |
| editor      | `i18n/strings.test.ts`                           |

## Verwandt

[[A10 CLI und Szenarien]] · [[A12 Phasenplan]] · [[T08 Statistik und Ergebnisanalyse]]
