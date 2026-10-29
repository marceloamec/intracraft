---
title: ADR-0002 Physik-Engine-Kandidat
lang: de
tags: [decision, physics]
status: proposed
date: 2026-10-28
---

🌐 [[ADR-0002 Physics Engine Candidate|EN]] · [[ADR-0002 Candidato a motor de física|ES]] · **DE** · [[Startseite (DE)|Index]]

# ADR-0002 Physik-Engine-Kandidat

- **Status:** Vorgeschlagen — wird nach Spike S1 geschlossen
- **Datum:** 28.10.2026

## Kontext

Das größte technische Risiko des MVP ist der **Kettenförderer** in 3D: Er braucht entweder einen Kontakt mit „bewegter Oberfläche“ oder viele kleine bewegte Körper ([[T12 Rollen und Ketten in der Physik]]). Der Plan nennt Rapier (Apache-2.0) und Jolt (MIT) und verlangt, nicht anzunehmen, dass die bewegte Oberfläche in den JS-Bindings existiert.

## Befunde (siehe [[T15 Recherche Physik-Engines]])

- `jolt-physics` 1.1.1: `ContactListenerJS` + `ContactSettings.mRelativeLinearSurfaceVelocity` und `mCombinedFriction` sind verfügbar → Plan B und exakte Reibpaarungen möglich.
- `@dimforge/rapier3d-compat` 0.21.0: `PhysicsHooks` filtert nur Paare; keine Änderung der Solver-Kontakte → nur Plan A.
- Gemessene Standardwerte, die in beiden Engines überschrieben werden müssen: Penetration Slop / zulässiger Fehler (0,02 m Jolt, 0,005 m Rapier), Restitutionsschwelle von Jolt 1 m/s.

## Entscheidung (vorgeschlagen)

Spike S1 **zuerst mit Jolt** durchführen, mit Plan B für die Ketten. Rückfall auf **Rapier mit Plan A**, falls Jolt die S1-Kriterien nicht erfüllt (Leistung der JS-Kontakt-Callbacks, Determinismus oder Stabilität). Plan C (Schleppkraft) und D (eigenes Rust-Modul) bleiben letzte Auswege.

## Abnahmekriterien Spike S1

| #   | Test                                        | Kriterium                                                                     |
| --- | ------------------------------------------- | ----------------------------------------------------------------------------- |
| a   | 1000-kg-Palette auf 10 kinematischen Rollen | Nenngeschwindigkeit ±3 %; bremst ohne übermäßigen Schlupf                     |
| b   | Kette (Plan B, Plan A zum Vergleich)        | Palettengeschwindigkeit = Kettengeschwindigkeit ±3 %, kein sichtbares Zittern |
| c   | Palette gegen Anschlag mit 0,25 m/s         | Vorderkante ±5 mm; Rückprall < 10 mm                                          |
| d   | Führung mit Trichter                        | 50 mm und 5° Schräglage → innerhalb ±10 mm und ±1°                            |
| e   | 90°-Drehung auf Plattform                   | Palette verschiebt sich < 5 mm zur Plattform                                  |
| f   | Leistung                                    | 20 Förderer und 10 Paletten bei 60 FPS, keine Instabilitäten                  |
| g   | Determinismus                               | Gleicher Zustands-Hash nach 60 s in zwei Läufen                               |

## Folgen

- Jeder Engine-Zugriff bleibt hinter `PhysicsBackend` ([[A07 Physik-Wrapper]]); ein Rückfall kostet einen Adapter, keine Neuentwicklung.
- Nach S1 wird dieses ADR *Angenommen* (oder *Abgelehnt* mit der Alternative) und hält Messergebnisse, Grenzen und den Kettenplan fest.
