---
title: A12 Phasenplan
lang: de
tags: [app, roadmap, plan]
---

🌐 [[A12 Roadmap|EN]] · [[A12 Hoja de ruta|ES]] · **DE** · [[Startseite (DE)|Index]]

# A12 Phasenplan

Die Reihenfolge greift zuerst das größte Risiko an (Kettenphysik) und liefert jeden Förderer vollständig aus. Größen in Arbeitssitzungen mit Claude Code: S ≈ 1–3, M ≈ 4–8, L ≈ 8–15.

## MVP-Phasen

| Phase | Ziel                   | Ergebnisse                                                                                                                                   | Abschlusskriterium                                        | Größe | Risiko  | Stand (28.10.2026)                                                      |
| ----- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- | ----- | ------- | ----------------------------------------------------------------------- |
| 0     | Grundlage              | Repository, pnpm-Workspaces, AGENTS.md, Doku, Glossar, 3 Referenzmodelle                                                                     | Tests laufen, CI grün                                     | S     | Niedrig | 🟡 Gerüst fertig; Referenzmodelle und git-Repository offen              |
| 1     | Unsicherheit senken    | Physik-Spike S1, S2 3D-Viewer mit instanzierten Rollen und Gizmos, S3 minimale DES (Quelle → Bahn → Senke)                                   | S1-Kriterien erfüllt, Engine-Entscheidung festgehalten    | M     | Hoch    | 🟡 Recherche fertig ([[T15 Recherche Physik-Engines]]); Viewer begonnen |
| 2     | Kern und Bibliothek    | core-model (Einheiten, Ports, Verbindungen, Validierung), Definitionsschema, Verhaltensregistry, Palettentypen, Reibungstabelle, Dateiformat | Quelle → Senke speichern und öffnen; Validierungen warnen | M     | Mittel  | 🟡 Schemas, Registry, Paletten- und Reibungsdaten vorhanden             |
| 3     | Rollenförderer (2D)    | Definition, Geometrie, DES-Verhalten (Kinematik, ZPA), Sensoren, Anschläge, Führungen in 2D, Basissteuerung, CLI                             | T1, T2; 3D-Animation des 2D-Laufs                         | L     | Mittel  | 🟡 Definitions-JSON, Kinematik, CLI                                     |
| 4     | Physikbasis            | Welt, kinematische Rollen, Verbundpalette, Raycast-Sensoren, Anschläge, Führungen                                                            | S1 a, c, d, g im Produkt; 2D/3D-Konsistenz                | L     | Hoch    | ⚪                                                                      |
| 5     | Kettenförderer         | Definition, DES mit Schlupf, Physik mit gewähltem Plan                                                                                       | Palettengeschwindigkeit = Kette ±3 %; Stau mit Schlupf    | M     | Hoch    | ⚪                                                                      |
| 6     | Kettendrehtisch        | Winkelprofil, Stationen, Verriegelungen, Ports je Station                                                                                    | T3 (6,5 s) und S1-e                                       | M     | Mittel  | ⚪ (Formel T3 ✅)                                                       |
| 7     | Rollendrehtisch        | Rollen auf Plattform, beim Drehen blockiert                                                                                                  | Gleiche Tests wie Phase 6                                 | S–M   | Niedrig | ⚪                                                                      |
| 8     | Rollen→Ketten-Umsetzer | Hubketten, Verriegelungen, Lückenprüfung, beide Modi                                                                                         | T4 (7,4 s) und 3D-Umsetzen ohne Palettenverlust           | L     | Hoch    | ⚪ (Formel T4 ✅)                                                       |
| 9     | Integration            | Referenzlayout mit allen fünf, Moduswechsel, Vergleich mit Referenzmodellen, Leistung, Doku, Paketierung                                     | Definition of Done des MVP ([[A01 Produktüberblick]])     | L     | Mittel  | ⚪                                                                      |

🟢 fertig · 🟡 in Arbeit · ⚪ nicht begonnen

**Warum diese Reihenfolge:** Der Rollenförderer ist die Basis der anderen; Ketten bündeln das Physikrisiko; Drehtische verwenden Rollen und Ketten wieder; der Umsetzer hängt von allem Vorherigen ab (Rollen, Ketten, Anschläge, Sensoren).

## Von Anfang an KI-fähig

Phase 2 liefert zusätzlich die Befehls- und Abfrageschicht (jede UI-Aktion ist ein Befehl); nach Phase 3 den MCP-Server (stdio); nach dem MVP den integrierten Chat mit eigenem Gateway und Token – siehe [[A15 KI-Agenten-Integration]].

## Nächste konkrete Schritte

1. `git init`, erster Commit, Push auf ein Remote, damit die CI läuft.
2. Spike S1 mit Jolt (`jolt-physics`): Tests a, b (Oberflächengeschwindigkeit, Plan B), c, g; falls nötig Rückfall auf Rapier + Plan A. [[ADR-0002 Physik-Engine-Kandidat]] schließen.
3. Spike S3: Quelle → Rollenbahn → Senke in der DES-Engine mit T6.
4. Die übrigen Fragen in [[Offene Fragen]] beantworten (Ausrichtung, Palettentyp und Rollen am 29.10.2026 beantwortet).

## Nach dem MVP (vorgeschlagene Reihenfolge)

1. Moduswechsel im Lauf. 2. Automatische 2D↔3D-Kalibrierung. 3. Kurven, Weichen, Heber, Shuttles. 4. Kartons und Behälter. 5. Störungen (MTBF/MTTR) und Energie. 6. Benutzerskripte. 7. Experimente, Replikationen, Optimierung. 8. OPC UA / SPS. 9. Lizenzierte Herstellermodelle und DXF-Import. 10. Desktop-Version (Tauri).

## Verwandt

[[A01 Produktüberblick]] · [[A11 Tests und Validierung]]
