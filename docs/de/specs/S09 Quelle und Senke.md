---
title: S09 Quelle und Senke
lang: de
tags: [spec, flow]
library-id: flow.source, flow.sink
status: spec
---

🌐 [[S09 Source and Sink|EN]] · [[S09 Generador y sumidero|ES]] · **DE** · [[Startseite (DE)|Index]]

# S09 Quelle und Senke (`flow.source`, `flow.sink`)

## 1. Quelle (Palettenerzeuger)

| Schlüssel         | Plan-Schlüssel      | Einheit  | Standard         | Bereich                                                       | Beschreibung                                                                                     |
| ----------------- | ------------------- | -------- | ---------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `palletType`      | `tipoPallet`        | Ref      | EUR              | —                                                             | Palettentyp aus der Bibliothek ([[S10 Palettentypen]])                                           |
| `loadMass`        | `masaCarga`         | kg       | 600              | 0 … 1500                                                      | Fest oder Verteilung (gleich, normal, Tabelle)                                                   |
| `loadHeight`      | `alturaCarga`       | mm       | 1200             | 0 … 2200                                                      | Ladehöhe über der Palette                                                                        |
| `orientation`     | `orientacion`       | Enum     | längs            | längs, quer                                                   | Ausrichtung beim Erzeugen                                                                        |
| `generationMode`  | `modoGeneracion`    | Enum     | festes Intervall | festes Intervall, Verteilung, auf Anforderung (Signal), Liste | Ankunftsregel                                                                                    |
| `interval`        | `intervalo`         | s        | 20               | 1 … 3600                                                      | Für festes Intervall                                                                             |
| `maxPallets`      | `maxPallets`        | —        | unbegrenzt       | —                                                             | Höchstzahl erzeugter Paletten                                                                    |
| `blockIfOccupied` | `bloquearSiOcupado` | bool     | ja               | —                                                             | Wartet, wenn am Anfang kein Platz frei ist                                                       |
| `seed`            | `semilla`           | Ganzzahl | global           | —                                                             | Für Verteilungen (eigener Stream je Objekt, [[T03 Zeit, Ereignisreihenfolge und Determinismus]]) |

**2D.** Plant Ereignisse `pallet_enters`: festes Intervall → alle `interval`; Verteilung → Zwischenankunftszeit aus dem Seed-Generator; auf Anforderung → bei steigendem Signal. Mit `blockIfOccupied` wartet eine Erzeugung, die den Einlauf belegt vorfindet (diese „verlorene“ Zeit wird gemeldet).
**3D.** Legt den Palettenverbundkörper am Ausgangsport an, ruhend und am Förderer ausgerichtet.

## 2. Senke

| Schlüssel     | Plan-Schlüssel   | Einheit | Standard | Bereich                             | Beschreibung                      |
| ------------- | ---------------- | ------- | -------- | ----------------------------------- | --------------------------------- |
| `removalMode` | `modoRetirada`   | Enum    | sofort   | sofort, nach einer Zeit, per Signal | Wie Paletten das Modell verlassen |
| `removalTime` | `tiempoRetirada` | s       | 0        | ≥ 0                                 | Für „nach einer Zeit“             |

Veröffentlicht empfangene Paletten, Durchsatz und mittlere Zykluszeit (Eintritt ins Modell → Ankunft an der Senke).

## Verwandt

[[S10 Palettentypen]] · [[T08 Statistik und Ergebnisanalyse]] · [[T03 Zeit, Ereignisreihenfolge und Determinismus]]
