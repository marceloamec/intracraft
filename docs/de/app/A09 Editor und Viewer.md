---
title: A09 Editor und Viewer
lang: de
tags: [app, editor, ui]
---

🌐 [[A09 Editor and Viewer|EN]] · [[A09 Editor y visor|ES]] · **DE** · [[Startseite (DE)|Index]]

# A09 Editor und Viewer (`apps/editor`)

## 1. Was heute existiert

Start mit `pnpm dev` → http://localhost:5173.

| Datei                                   | Aufgabe                                                                                                                                                                                                            |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `src/App.tsx`                           | Aufbau: Werkzeugleiste + Ansichtsfenster                                                                                                                                                                           |
| `src/store.ts`                          | Zustand-UI-Store: Sprache (en/es/de), Ansicht (Perspektive/Draufsicht), Simulationsmodus (des/physics)                                                                                                             |
| `src/i18n/strings.ts`, `useT.ts`        | UI-Texte auf Englisch, Spanisch und Deutsch (getestet: gleiche Schlüssel in allen drei)                                                                                                                            |
| `src/components/Toolbar.tsx`            | Ansichtswechsel (Taste **V**), Auswahl von Simulationsmodus und Sprache                                                                                                                                            |
| `src/components/Viewport.tsx`           | `Canvas` von react-three-fiber; Perspektivkamera oder orthografische Draufsicht (Blick nach −Y); Orbit-Steuerung (in der Draufsicht ohne Drehen)                                                                   |
| `src/components/RollerConveyorMesh.tsx` | Parametrischer Rollenförderer aus den Bibliotheksstandards: Rollen als ein `InstancedMesh`, platziert mit `rollerLayout`, Seitenrahmen und eine EUR-Palette, animiert mit der 2D-Kinematik (Beschleunigen, Fahren) |

Die animierte Palette ist eine Vorschau auf die „3D-Animation der 2D-Simulation“: Ihre Position stammt aus `positionAt` und `timeToReach` von `sim-des`, nicht aus einer Physik-Engine.

## 2. Funktionsumfang des MVP (zu bauen)

- **Ansichten:** 3D-Perspektive und 2D-Draufsicht (orthografisch), per Taste umschaltbar; Orbit, Zoom, Auswahl einpassen.
- **Bearbeiten:** Komponenten aus der Bibliothek ziehen, mit Gizmos verschieben/drehen, 100-mm-Raster mit Einrasten an Ports und Raster, Kopieren/Einfügen, Feldwiederholung, Rückgängig/Wiederholen.
- **Verbinden:** automatisches Einrasten der Ports mit Warnungen zu Höhe, Breite und Ausrichtung.
- **Eigenschaften:** aus dem Definitionsschema erzeugter Bereich (Gruppen, Einheiten, Bereiche, abgeleitete Werte, Validierungswarnungen) — [[A05 Komponentenbibliothek]].
- **Modellbaum:** Objektliste mit Suche, Ausblenden, Sperren.
- **Ausführen:** Start, Pause, Einzelschritt, Geschwindigkeit ×1, ×10, ×100 (+ max. in 2D); Simulationsuhr; Wahl 2D/3D.
- **Live-Darstellung:** Sensorstrahlen grün/rot, Anschläge oben/unten, Paletten nach Zustand eingefärbt, Signal- und Statistikbereich.
- **Steuerung:** Wenn/Dann-Regeleditor mit Vorlagen.
- **Datei:** `.intracraft.json` speichern/öffnen, Statistik als CSV/JSON exportieren, Screenshots.
- **Werkzeuge:** Abstände messen, sichtbare Ebenen (Führungen, Sensoren, Bemaßung).

Verschoben: Hintergrundplan (Bild/DXF), Videoaufzeichnung, Berichte, Laufvergleich.

## 3. Darstellungsregeln

- Rollen und andere wiederholte Teile nutzen `InstancedMesh` (ein Draw Call je Art).
- Die Simulation läuft in einem Web Worker; der Viewer interpoliert Snapshots ([[A02 Architektur]]).
- Die Oberfläche zeigt mm und Grad; gespeichert wird alles in SI ([[A13 Konventionen]]).
- Der Viewer entscheidet nie über Simulationsergebnisse; er zeigt nur Snapshots an.

## Verwandt

[[A02 Architektur]] · [[A05 Komponentenbibliothek]] · [[A13 Konventionen]]
