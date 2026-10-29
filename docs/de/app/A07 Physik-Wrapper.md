---
title: A07 Physik-Wrapper
lang: de
tags: [app, physics, sim-physics]
---

🌐 [[A07 Physics Engine Wrapper|EN]] · [[A07 Envoltorio del motor físico|ES]] · **DE** · [[Startseite (DE)|Index]]

# A07 Physik-Wrapper (`packages/sim-physics`)

Theorie: [[T09 Starrkörperdynamik]] bis [[T15 Recherche Physik-Engines]].

## 1. Warum ein Wrapper

Jeder Zugriff auf die Physik-Engine läuft über dieses Paket. Niemand sonst darf Jolt oder Rapier importieren. Vorteile: Die Engine lässt sich nach Spike S1 (oder später) austauschen, ohne Komponenten anzufassen; Versionen werden an einer Stelle festgelegt; Eigenheiten der Engine (Schlafen, Slop, Restitutionsschwelle) werden einmal behandelt.

## 2. Was heute existiert

| Modul           | Inhalt                                                                                                                                                                                                                                                                                       | Tests             |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- |
| `backend.ts`    | Schnittstelle `PhysicsBackend`: `addBody`, `removeBody`, `setKinematicVelocity`, `setSurfaceVelocity` (liefert false, wenn nicht unterstützt), `getPose`, `castRay`, `step`, `stateHash`; `BodyDesc` (statisch/kinematisch/dynamisch, Formen, Masse, Schwerpunkt, Reibung, Restitution, CCD) | — (Schnittstelle) |
| `fixed-step.ts` | `FixedStepClock` (Akkumulator, max. Schritte je Frame, Interpolations-`alpha`), `DEFAULT_PHYSICS_DT` = 1/120 s                                                                                                                                                                               | ✓                 |
| `mechanics.ts`  | `rollerAngularVelocity`, `maxTractionAccel`, `turntableRequiredFriction`, `turntableRequiredTorque`, `sweptRadius`, `boxYawInertia`                                                                                                                                                          | ✓                 |

## 3. Geplanter Weltaufbau (Phase 4)

| Bibliotheksobjekt  | Physikkörper                                                                                                                |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| Palette            | Dynamischer Verbund: 3 Bodenbrett-Quader + Ladungsquader; explizite Masse und Schwerpunkt; CCD an                           |
| Rollenförderer     | Ein kinematischer Zylinder je Rolle, nach Zonen gruppiert; ω in jedem Schritt aus dem Zonenprofil                           |
| Kettenförderer     | Plan B: ein statischer Quader je Strang + Oberflächengeschwindigkeit (Jolt) · Plan A: Streifen kleiner kinematischer Rollen |
| Drehtisch          | Kinematische Plattform + kinematische Kinder (Rollen/Ketten), aus dem Plattformwinkel positioniert                          |
| Kettenumsetzer     | Rollen + kinematische Kettenstränge, die erst vertikal, dann horizontal fahren                                              |
| Anschlag           | Kinematischer Quader zwischen unterer/oberer Stellung                                                                       |
| Führung / Trichter | Statische Quader                                                                                                            |
| Lichtschranke      | Raycast (oder Shape-Cast) in jedem Schritt; Flankenzeit = Schrittmitte                                                      |

## 4. Schrittschleife

```text
für jeden festen Schritt (dt = 1/120 s):
    falls eine 10-ms-Steuergrenze überschritten: Steuerung ausführen, Schreibvorgänge anwenden
    für jede Komponente: kinematische Ziele aktualisieren (Rollen-ω, Anschlaghöhe, Plattformwinkel …)
    Paletten wecken, die Förderer mit geänderter Geschwindigkeit berühren   # T10 §7
    backend.step(dt)
    Sensoren: Raycast, Flanken erkennen, Zeitstempel
    Aufpralle (Geschwindigkeit, Energie) und Statistik erfassen
alle N Schritte: Zustand hashen (Determinismustest)
Snapshot an die Oberfläche senden
```

## 5. Zu überschreibende Engine-Einstellungen (nach gemessenen Standards)

| Einstellung                            | Standard (Engine)                 | Geplanter Wert       | Grund                                          |
| -------------------------------------- | --------------------------------- | -------------------- | ---------------------------------------------- |
| Zeitschritt                            | 1/60 s (Rapier)                   | 1/120 s              | [[T10 Zeitintegration und fester Zeitschritt]] |
| Penetration Slop / zulässiger Fehler   | 0,02 m (Jolt) / 0,005 m (Rapier)  | ≈ 0,001 m            | ±5-mm-Anschlagkriterium                        |
| Mindestgeschwindigkeit für Restitution | 1 m/s (Jolt)                      | ≈ 0,01 m/s           | Aufpralle mit 0,1–0,25 m/s                     |
| Reibungskombination                    | geometrisches Mittel / Mittelwert | exakter Paarungswert | Reibungstabelle                                |

Jede Änderung dieser Werte wird in `docs/*/decisions/` festgehalten (Regel aus AGENTS.md).

## Verwandt

[[ADR-0002 Physik-Engine-Kandidat]] · [[T12 Rollen und Ketten in der Physik]] · [[A02 Architektur]] · [[A14 API-Referenz]]
