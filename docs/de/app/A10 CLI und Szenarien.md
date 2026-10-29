---
title: A10 CLI und Szenarien
lang: de
tags: [app, cli, scenarios]
---

🌐 [[A10 CLI and Scenarios|EN]] · [[A10 CLI y escenarios|ES]] · **DE** · [[Startseite (DE)|Index]]

# A10 CLI und Szenarien

## 1. Szenarien (`packages/scenarios`)

Ein **Szenario** ist ein reproduzierbares Experiment mit erwartetem Ergebnis. Heute enthält das Paket die fünf **analytischen** Szenarien des Plans in `scenarios/analytic.json`:

| Id  | Art             | Eingaben (SI)                                 | Erwartet           |
| --- | --------------- | --------------------------------------------- | ------------------ |
| T1  | `profileTime`   | 3 m, 0,25 m/s, 0,5 m/s², ohne Bremsen         | 12,25 s            |
| T2  | `throughput`    | 0,25 m/s, Palette 1,2 m, Lücke 0,15 m         | 0,18519 Paletten/s |
| T3  | `profileTime`   | π/2 rad, 15 °/s, 30 °/s² (in rad)             | 6,5 s              |
| T4  | `transferCycle` | 1 s Heben, 1 s Senken, 1 m, 0,2 m/s, 0,5 m/s² | 7,4 s              |
| T5  | `impactEnergy`  | 1025 kg, 0,25 m/s                             | 32 J               |

Jedes Szenario hat einen übersetzten Titel (en/es/de), eine Einheit und eine relative Toleranz (1 %). `runScenario` berechnet das Ergebnis mit den Formeln aus `sim-des` und liefert `{ actual, expected, relativeError, passed }`.

Ab Phase 3 enthalten Szenarien auch **vollständige Layouts** (Quelle → Förderer → Senke), die die DES-Engine ausführt, mit Referenzergebnissen in `test_data/`.

## 2. CLI (`apps/cli`)

```text
pnpm run scenario all     # alle Szenarien ausführen
pnpm run scenario t3      # eines ausführen (Groß-/Kleinschreibung egal)
```

Beispielausgabe:

```text
PASS T1 Roller conveyor: 3 m from rest, 0.25 m/s, 0.5 m/s², no braking: 12.2500 s (expected 12.25 s, error 0.000 %)
…
5/5 scenarios passed
```

Exit-Codes: `0` alle bestanden · `1` mindestens eines fehlgeschlagen · `2` unbekannte Szenario-Id. Die CLI schreibt über den Projekt-Logger (kein `console.log`).

## Verwandt

[[A11 Tests und Validierung]] · [[T04 Kinematik von Förderern]] · [[A14 API-Referenz]]
