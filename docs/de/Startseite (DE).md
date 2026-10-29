---
title: Startseite (DE)
lang: de
tags: [index]
---

🌐 [[Home (EN)|EN]] · [[Inicio (ES)|ES]] · **DE** · [[Home|Alle Sprachen]]

> [!warning] Seit dem 29.10.2026 wird die Dokumentation nur noch auf Englisch gepflegt ([[Home (EN)]]). Diese deutschen Notizen bleiben erhalten, können aber veraltet sein.

# IntraCraft — Dokumentation (Deutsch)

IntraCraft simuliert Palettenfördersysteme mit **einem Layout und zwei Modi**: einem schnellen 2D-Modus mit **ereignisdiskreter Simulation** (Durchsatz, Stau, Blockierungen, Auslastung) und einem 3D-Modus **mit Physik** in Echtzeit (Reibung von Rollen und Ketten, Kollisionen mit Anschlägen und Führungen, Ausrichtung der Paletten, Sensoren mit realer Geometrie).

## Lesepfade

- **„Ich möchte verstehen, wie der 2D-Modus rechnet“** → [[T01 Simulationsparadigmen]] → [[T02 Ereignisdiskrete Simulation]] → [[T03 Zeit, Ereignisreihenfolge und Determinismus]] → [[T04 Kinematik von Förderern]] → [[T05 Stau, Kapazität und Durchsatz]].
- **„Ich möchte verstehen, wie der 3D-Modus eine Palette bewegt“** → [[T09 Starrkörperdynamik]] → [[T10 Zeitintegration und fester Zeitschritt]] → [[T11 Kontakt, Reibung und Restitution]] → [[T12 Rollen und Ketten in der Physik]].
- **„Ich möchte wissen, wie die Anwendung aufgebaut ist“** → [[A01 Produktüberblick]] → [[A02 Architektur]] → [[A03 Monorepo und Stack]] → [[A14 API-Referenz]].

## Theorie

| #   | Notiz                                               | Modus |
| --- | --------------------------------------------------- | ----- |
| T01 | [[T01 Simulationsparadigmen]]                       | beide |
| T02 | [[T02 Ereignisdiskrete Simulation]]                 | 2D    |
| T03 | [[T03 Zeit, Ereignisreihenfolge und Determinismus]] | beide |
| T04 | [[T04 Kinematik von Förderern]]                     | 2D    |
| T05 | [[T05 Stau, Kapazität und Durchsatz]]               | 2D    |
| T06 | [[T06 Fotoelektrische Erfassung]]                   | beide |
| T07 | [[T07 Steuerungslogik und Signale]]                 | beide |
| T08 | [[T08 Statistik und Ergebnisanalyse]]               | beide |
| T09 | [[T09 Starrkörperdynamik]]                          | 3D    |
| T10 | [[T10 Zeitintegration und fester Zeitschritt]]      | 3D    |
| T11 | [[T11 Kontakt, Reibung und Restitution]]            | 3D    |
| T12 | [[T12 Rollen und Ketten in der Physik]]             | 3D    |
| T13 | [[T13 Drehtische und Kettenumsetzer]]               | beide |
| T14 | [[T14 Anschläge, Aufprall und Ausrichtung]]         | beide |
| T15 | [[T15 Recherche Physik-Engines]]                    | 3D    |
| T16 | [[T16 Hybride Konsistenz und Kalibrierung]]         | beide |

## Anwendung

| #   | Notiz                                |
| --- | ------------------------------------ |
| A01 | [[A01 Produktüberblick]]             |
| A02 | [[A02 Architektur]]                  |
| A03 | [[A03 Monorepo und Stack]]           |
| A04 | [[A04 Datenmodell und Projektdatei]] |
| A05 | [[A05 Komponentenbibliothek]]        |
| A06 | [[A06 DES-Engine]]                   |
| A07 | [[A07 Physik-Wrapper]]               |
| A08 | [[A08 Steuerung]]                    |
| A09 | [[A09 Editor und Viewer]]            |
| A10 | [[A10 CLI und Szenarien]]            |
| A11 | [[A11 Tests und Validierung]]        |
| A12 | [[A12 Phasenplan]]                   |
| A13 | [[A13 Konventionen]]                 |
| A14 | [[A14 API-Referenz]]                 |
| A15 | [[A15 KI-Agenten-Integration]]       |

## Spezifikationen (Bibliotheksobjekte)

| #   | Notiz                          | Bibliotheks-Id              |
| --- | ------------------------------ | --------------------------- |
| S01 | [[S01 Rollenförderer]]         | `conveyor.roller`           |
| S02 | [[S02 Kettenförderer]]         | `conveyor.chain`            |
| S03 | [[S03 Kettendrehtisch]]        | `conveyor.chain_turntable`  |
| S04 | [[S04 Rollendrehtisch]]        | `conveyor.roller_turntable` |
| S05 | [[S05 Rollen-Ketten-Umsetzer]] | `conveyor.transfer_rc`      |
| S06 | [[S06 Lichtschranke]]          | `sensor.photoeye`           |
| S07 | [[S07 Mechanischer Anschlag]]  | `mechanics.stop`            |
| S08 | [[S08 Seitenführung]]          | `mechanics.guide`           |
| S09 | [[S09 Quelle und Senke]]       | `flow.source`, `flow.sink`  |
| S10 | [[S10 Palettentypen]]          | `pallet.*`                  |

## Entscheidungen

- [[ADR-0001 Werkzeuge und Versionen]]
- [[ADR-0002 Physik-Engine-Kandidat]]
- [[ADR-0003 Dreisprachige Doku und englische Bezeichner]]
- [[ADR-0004 Befehlsschicht und MCP für KI-Agenten]]

## Referenz

- [[Glossar]] — Begriffe auf Englisch, Spanisch und Deutsch.
- [[Offene Fragen]] — Entscheidungen, die der Verantwortliche noch treffen muss (mit Kontext und Optionen).
