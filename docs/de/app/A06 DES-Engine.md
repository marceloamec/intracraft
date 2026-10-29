---
title: A06 DES-Engine
lang: de
tags: [app, des, sim-des]
---

🌐 [[A06 DES Engine|EN]] · [[A06 Motor discreto|ES]] · **DE** · [[Startseite (DE)|Index]]

# A06 DES-Engine (`packages/sim-des`)

Theorie: [[T02 Ereignisdiskrete Simulation]], [[T03 Zeit, Ereignisreihenfolge und Determinismus]], [[T04 Kinematik von Förderern]], [[T05 Stau, Kapazität und Durchsatz]].

## 1. Was heute existiert

| Modul            | Inhalt                                                                                                                                   | Tests                                                          |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `event-queue.ts` | `EventQueue<T>`: Binär-Min-Heap nach (Zeit, Priorität, Sequenz); nur ganzzahlige µs                                                      | Reihenfolge, Gleichstände, Ablehnung nicht ganzzahliger Zeiten |
| `simulator.ts`   | `DesSimulator`: Uhr, `on(typ, handler)`, `schedule(verzögerung, ereignis, priorität)`, `cancel(ticket)`, `runUntil(t)`, Ereignis-`trace` | Uhrensprünge, Stornierung, Determinismus                       |
| `kinematics.ts`  | `profileTime`, `transferCycleTime`, `maxThroughput`, `palletCapacity`, `impactEnergy`, `positionAt`, `velocityAt`, `timeToReach`         | T1–T5, Dreiecksfall, Segmente                                  |

## 2. Geplantes Design (Phase 3)

### Bahnen

Jeder Förderer ist eine **Bahn** mit einer geordneten Palettenliste (vorderste zuerst). Eine Palette auf einer Bahn speichert:

- ihr aktuelles kinematisches Segment `{t0, s0, v0, a}`;
- ihre Länge in Förderrichtung (abhängig von der Ausrichtung);
- eine **Versionsnummer**, die bei jeder Bewegungsänderung steigt, um veraltete Vorhersagen zu entwerten.

### Vorhersagezyklus

Nach jedem Ereignis, das eine Palette betrifft:

1. Aktuelles Segment bei `jetzt` schließen und ein neues öffnen (neue Geschwindigkeit/Beschleunigung).
2. Kandidaten-Überfahrten mit `timeToReach` berechnen: Rampenende, nächste Sensorflanke, Anschlag, Zonenende, Hinterkante der Vorgängerin − Mindestabstand, Bahnende.
3. Nur die **früheste** planen; die vorherige Vorhersage stornieren.

### Ereignistypen

`pallet_enters`, `ramp_ends`, `front_reaches_stop`, `follower_reaches_leader`, `tail_clears_entry`, `sensor_edge`, `zone_released`, `turn_completed`, `lift_completed`, `transfer_completed`, `signal_changed`, `rule_fired`.

### Übergabe zwischen Bahnen

Bei `front_reaches_end` wird die Palette auf der nachfolgenden Bahn registriert (falls diese sie annehmen kann); solange sie auf beiden liegt, folgt ihre Geschwindigkeit der Übergaberegel ([[T04 Kinematik von Förderern]] §9). Hat die Hinterkante die vorherige Bahn verlassen, wird sie aus deren Liste entfernt.

### Statistik

Je Objekt: Zeit je Zustand, Zähler, Stichproben (Durchlaufzeiten) mit zeitgewichteten Akkumulatoren in ganzzahligen µs ([[T08 Statistik und Ergebnisanalyse]]).

## 3. Leistungsziel

≥ 1000× Echtzeit mit 50 Objekten und 30 Paletten. Jedes Ereignis kostet $O(\log n)$ im Heap plus konstanten Aufwand je betroffener Palette; der Viewer wertet `positionAt` je Frame aus, nicht je Ereignis.

## 4. Regeln für Beitragende

- Kein `Math.random()`, kein `Date.now()`, kein DOM, kein React (per ESLint erzwungen).
- Alle Zeiten in der Warteschlange sind ganzzahlig (µs); einmalig mit `secondsToSimTime` umrechnen.
- Jeder neue Ereignistyp bekommt einen Test, der seinen Zeitpunkt mit einer analytischen Formel vergleicht.

## Verwandt

[[A02 Architektur]] · [[A08 Steuerung]] · [[A14 API-Referenz]]
