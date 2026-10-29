---
title: A13 Konventionen
lang: de
tags: [app, conventions]
---

🌐 [[A13 Conventions|EN]] · [[A13 Convenciones|ES]] · **DE** · [[Startseite (DE)|Index]]

# A13 Konventionen

## 1. Modellkonventionen

| Thema         | Regel                                                                                                                                                     |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Einheiten     | Intern **SI**: m, kg, s, rad. Die Oberfläche zeigt mm und Grad. Nur an den Rändern (UI, Dateien) umrechnen, mit `mmToM`, `degToRad` …                     |
| Achsen        | **Y nach oben**, rechtshändig (wie three.js und glTF). Förderer laufen entlang der lokalen +X-Achse; die 2D-Ansicht ist die Draufsicht mit Blick nach −Y. |
| Zeit          | Ganzzahlige **Mikrosekunden** (`SimTime`). Ereignisreihenfolge (Zeit, Priorität, Sequenz).                                                                |
| Zufall        | Immer der Projekt-Generator mit Seed (`createRng`). Nie `Math.random()` in den Engines.                                                                   |
| Determinismus | Gleiches Layout + gleicher Seed → gleiches Ergebnis. Ein fehlschlagender Determinismustest hat höchste Priorität.                                         |
| Projektdatei  | JSON `.intracraft.json` mit `schemaVersion` und expliziten Migrationen; Ergebnisse getrennt.                                                              |
| Standardwerte | Bibliotheksstandards sind **Richtwerte**, nie als Herstellerdaten darzustellen.                                                                           |

## 2. Code-Konventionen

- Überall striktes TypeScript (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`); 2 Leerzeichen Einrückung; einfache Anführungszeichen.
- Bezeichner, Parameterschlüssel und Dateinamen auf **Englisch**; Texte für Benutzer übersetzt `{ en, es, de }` ([[ADR-0003 Dreisprachige Doku und englische Bezeichner]]).
- Kein `console.log` außerhalb von Tests: `createLogger` aus `core-model` verwenden.
- Eine Datei je Bibliothekskomponente oder Befehl; gemeinsamer Zustand, Typen und Helfer in Hilfsmodulen.
- Engines (`sim-des`, `sim-physics`, `control`) hängen nicht von React oder dem DOM ab.
- Reihenfolge in jeder Datei: Importe → Konstanten → Typen (externe vor internen) → öffentliche Funktionen → private Helfer unter ihren Nutzern. Tests: Konstanten → Testfälle (in `describe`-Blöcken gruppiert) → Helfer am Ende.

## 3. Doku-Konventionen (dieser Vault)

- Drei parallele Ordner `en/`, `es/`, `de/` mit gleicher Struktur und Nummerierung (T = Theorie, A = Anwendung, S = Spezifikation, ADR = Entscheidung).
- Notiz-Dateinamen sind übersetzt und im ganzen Vault eindeutig, damit `[[Wikilinks]]` ohne Pfade aufgelöst werden.
- Jede Notiz beginnt mit einer Sprachleiste, die die beiden anderen Übersetzungen verlinkt.
- Formeln in LaTeX (`$…$`, `$$…$$`), Diagramme in Mermaid – beides stellt Obsidian direkt dar.
- Markdown-Tabellen werden mit Leerzeichen aufgefüllt, damit die Spalten im Klartext fluchten (Regel aus AGENTS.md).
- Ändert sich eine Notiz, werden ihre beiden Übersetzungen in derselben Änderung angepasst.

## 4. Clean-Room-Regeln

- Nie Code, Binärdateien, Projektdateien oder Ressourcen fremder Software (auch kommerzieller Simulatoren) lesen, dekompilieren, importieren oder kopieren.
- Keine fremden Namen, Logos oder Marken in Produkt, Code oder Oberfläche.
- Keine unlizenzierten 3D-Modelle oder Kataloge von Herstellern.
- Die Lizenz jeder neuen Abhängigkeit prüfen (aktueller Bestand in ADR-0001).
- Andere Programme nur zum Vergleich beobachtbarer Ergebnisse nennen.

## 5. Arbeitsablauf

1. Spezifikation des Objekts lesen (falls nicht vorhanden, zuerst schreiben und bestätigen lassen).
2. Kurzen Plan vorschlagen.
3. Test schreiben, scheitern sehen, implementieren, bis er besteht.
4. Alle Tests, Lint und Typprüfung ausführen.
5. In Simulationsbegriffen zusammenfassen.
6. Kleiner Commit je funktionierendem Schritt; ein Branch je Funktion.

## Verwandt

[[A03 Monorepo und Stack]] · [[T03 Zeit, Ereignisreihenfolge und Determinismus]]
