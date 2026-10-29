---
title: A01 Produktüberblick
lang: de
tags: [app, overview]
---

🌐 [[A01 Product Overview|EN]] · [[A01 Visión del producto|ES]] · **DE** · [[Startseite (DE)|Index]]

# A01 Produktüberblick

## Was IntraCraft ist

Eine Anwendung zum **Planen und Simulieren von Palettenfördersystemen** mit einem Layout und zwei Simulationsmodi:

| Modus            | Engine        | Geschwindigkeit   | Beantwortet                                                   |
| ---------------- | ------------- | ----------------- | ------------------------------------------------------------- |
| 2D diskret (DES) | `sim-des`     | ≥ 1000× Echtzeit  | Durchsatz, Stau, Blockierungen, Auslastung                    |
| 3D mit Physik    | `sim-physics` | Echtzeit (60 FPS) | Reibung, Aufprall, Ausrichtung, Sensoren mit realer Geometrie |

Beide Modi lesen dasselbe Layout und werden von derselben Wenn/Dann-Steuerung bedient. Theorie: [[T01 Simulationsparadigmen]].

## Gestaltungsprinzipien

1. Das Layout ist die Quelle der Wahrheit; 2D, 3D und Statistik werden daraus abgeleitet.
2. Gleiche Steuerungslogik, andere Anlage.
3. Jeder Parameter hat Einheit, Bereich, Standardwert und Beschreibung; die Bearbeitungsoberfläche wird aus diesem Schema erzeugt.
4. Determinismus: gleiche Daten + gleicher Seed → gleiche Ergebnisse.
5. Die reale Palettengeometrie zählt (Lücken zwischen Brettern und Klötzen).
6. Erst validieren, dann erweitern: analytische Tests und Vergleich mit bekannten Referenzmodellen.
7. Kleine Schritte, immer mit Test.

## Umfang des MVP

**Enthalten:** fünf Förderer ([[S01 Rollenförderer]], [[S02 Kettenförderer]], [[S03 Kettendrehtisch]], [[S04 Rollendrehtisch]], [[S05 Rollen-Ketten-Umsetzer]]); Lichtschranken (Taster, Reflexion, Einweg); feste und versenkbare Anschläge; Seitenführungen mit Trichter; Quelle und Senke; Paletten EUR, 1200 × 1000, GMA und benutzerdefiniert; Wenn/Dann-Steuerung mit booleschen Signalen; beide Modi auf demselben Layout (Wechsel startet neu); Editor (platzieren, verbinden, parametrieren, speichern, öffnen; abspielen, pausieren, beschleunigen); Basisstatistik, CSV/JSON-Export, CLI ohne Oberfläche; integrierte Bibliothek mit Voreinstellungen und eigenen Komponenten.

**Nicht enthalten (nach dem MVP):** Kurven, Heber, Shuttles, Weichen; Kartons und Behälter; Störungen (MTBF/MTTR) und Energie; Benutzerskripte; Moduswechsel im Lauf; automatische 2D↔3D-Kalibrierung; Experimente und Optimierung; OPC UA / SPS; CAD-Import; Teamarbeit.

## Definition of Done des MVP

1. Die fünf Förderer lassen sich aus der Bibliothek platzieren, verbinden und parametrieren.
2. Ein Referenzlayout mit allen fünf läuft in beiden Modi ohne Eingriff.
3. 2D-Ergebnisse stimmen mit den analytischen Tests auf < 1 % überein ([[A11 Tests und Validierung]]).
4. In 3D hält eine Palette am Anschlag innerhalb ±5 mm, Rückprall < 10 mm.
5. Durchlaufzeiten und Sensorereignisse unterscheiden sich zwischen den Modi um < 5 % und ±50 ms.
6. Gleiches Layout + gleicher Seed → identische Ergebnisse in zwei Läufen.
7. 3D hält 60 FPS mit 20 Förderern und 10 Paletten auf einem aktuellen Laptop.
8. Drei bekannte Referenzmodelle werden nachgebaut und verglichen.

## Aktueller Stand (28.10.2026)

Das Grundgerüst der Phase 0 steht: Monorepo, Pakete, CI-Workflow, analytische Kinematik (T1–T5 bestanden), Ereigniswarteschlange, Seed-Generator, Projektschema, Definition des Rollenförderers, Regelauswerter und ein Viewer mit einem parametrischen Rollenförderer. Siehe [[A12 Phasenplan]].

## Verwandt

[[A02 Architektur]] · [[A12 Phasenplan]] · [[Glossar]]
