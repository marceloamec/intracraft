---
title: T15 Recherche Physik-Engines
lang: de
tags: [theory, physics, research]
date: 2026-10-28
---

🌐 [[T15 Physics Engine Research|EN]] · [[T15 Investigación de motores de física|ES]] · **DE** · [[Startseite (DE)|Index]]

# T15 Recherche Physik-Engines

Recherche vom **28.10.2026** zur Vorbereitung von Spike S1. Methode: Die npm-Pakete wurden heruntergeladen (nicht im Projekt installiert), ihre TypeScript-Typen geprüft und die Standardeinstellungen durch Instanziieren jeder Engine in Node.js 24 ausgelesen. Aus diesen Paketen wurde nichts in IntraCraft kopiert.

## 1. Kandidaten und Versionen

| Engine | npm-Paket                                 | Geprüfte Version | Lizenz     | Sprache / Build                                       |
| ------ | ----------------------------------------- | ---------------- | ---------- | ----------------------------------------------------- |
| Rapier | `@dimforge/rapier3d-compat`               | 0.21.0           | Apache-2.0 | Rust → WASM (base64-eingebettet)                      |
| Rapier | `@dimforge/rapier3d-deterministic-compat` | 0.21.0           | Apache-2.0 | Ebenso, plattformübergreifend deterministischer Build |
| Jolt   | `jolt-physics`                            | 1.1.1            | MIT        | C++ → WASM (Emscripten), auch Multithread-Builds      |

## 2. Die entscheidende Frage: Oberflächengeschwindigkeit in JavaScript

| Fähigkeit                             | Rapier JS 0.21.0                                                        | Jolt JS 1.1.1                                                                             |
| ------------------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Hook zur Kontaktänderung              | ❌ `PhysicsHooks` hat nur `filterContactPair`, `filterIntersectionPair` | ✅ `ContactListenerJS.OnContactAdded / OnContactPersisted(…, ioSettings)`                 |
| Oberflächengeschwindigkeit je Kontakt | ❌ (der Rust-Hook `modify_solver_contacts` ist nicht gebunden)          | ✅ `ContactSettings.mRelativeLinearSurfaceVelocity` und `mRelativeAngularSurfaceVelocity` |
| Reibung je Kontakt überschreiben      | ❌ (nur Kombinationsregeln: Average, Min, Multiply, Max)                | ✅ `ContactSettings.mCombinedFriction`, `mCombinedRestitution`                            |
| Kontaktfilterung                      | ✅                                                                      | ✅ `OnContactValidate`                                                                    |
| Plan B für Ketten                     | Nicht möglich → Plan A oder D                                           | **Machbar** (in S1 zu bestätigen)                                                         |

**Offene Punkte für S1 mit Jolt:** Vorzeichen-/Bezugssystemkonvention der relativen Oberflächengeschwindigkeit (Körper 1 vs. Körper 2); Kosten eines JS-Callbacks je Kontakt und Schritt (Hunderte Kontakte × 120 Hz); ob die Callback-Reihenfolge deterministisch ist; Speicherverwaltung der Emscripten-Objekte (`Jolt.destroy`).

## 3. Gemessene Standardeinstellungen

| Einstellung                         | Rapier 0.21.0                            | Jolt 1.1.1                                 | Warum wichtig                                      |
| ----------------------------------- | ---------------------------------------- | ------------------------------------------ | -------------------------------------------------- |
| Standard-Zeitschritt                | `dt` = 1/60 s                            | je `Update`-Aufruf                         | IntraCraft nutzt 1/120 s                           |
| Solver-Iterationen                  | `numSolverIterations` = 4                | 10 Geschwindigkeits- + 2 Positionsschritte | Schwere Paletten auf vielen Rollen (S1-a)          |
| Zulässige Eindringung               | `normalizedAllowedLinearError` = 0,005 m | `mPenetrationSlop` = 0,02 m                | ±5-mm-Anschlagkriterium (P1) → muss gesenkt werden |
| Spekulativer Kontaktabstand         | —                                        | `mSpeculativeContactDistance` = 0,02 m     | Vorausschauende Kontakterkennung                   |
| Restitutionsschwelle                | —                                        | `mMinVelocityForRestitution` = 1 m/s       | Aufpralle mit 0,1–0,25 m/s prallen nicht zurück    |
| Standardreibung / Kombinationsregel | 0,5 / Average                            | geometrisches Mittel (dokumentiert)        | Exakte Tabellenwerte je Paarung                    |

## 4. Determinismus

- **Rapier** bietet einen eigenen `-deterministic`-Build mit **plattformübergreifender** Determinismusgarantie (langsamer). Der Standard-Build ist nur auf demselben Rechner deterministisch.
- **Jolt** simuliert deterministisch, wenn Körper in derselben Reihenfolge angelegt und dieselben API-Aufrufe gemacht werden; **plattformübergreifender** Determinismus erfordert einen Build mit `CROSS_PLATFORM_DETERMINISTIC=ON` (im README von `jolt-physics` beschrieben). Ob der npm-Build das aktiviert hat, ist in S1 zu prüfen; sonst gilt die Garantie nur auf demselben Rechner.
- Der IntraCraft-Test S1-g (gleicher Hash nach 60 s) braucht für den MVP nur Determinismus auf demselben Rechner.

## 5. Weitere Beobachtungen

- Beide laufen in einem Web Worker (Ausführungsmodell des Plans).
- Jolt bietet Multithread-WASM-Builds (brauchen `SharedArrayBuffer` → Cross-Origin-Isolation-Header). Für das MVP-Ziel (20 Förderer, 10 Paletten) nicht nötig.
- Rapier hat eine ausgereifte React-Integration (`@react-three/rapier`), aber IntraCraft rechnet die Physik in einem Worker hinter einem eigenen Wrapper – dieser Vorteil entfällt.
- three.js liefert Beispieladapter für beide Engines (`examples/jsm/physics/`), nur als Lesematerial nützlich.

## 6. Vorläufiges Fazit

Jolt ist der **bevorzugte Kandidat** für S1, weil nur sein JS-Binding Oberflächengeschwindigkeit und Reibung je Kontakt freigibt; damit sind Kettenförderer (Plan B) und exakte Reibpaarungen ohne zusätzliche Körper möglich. Rapier bleibt die Rückfallebene mit Plan A. Die Entscheidung ist als *Vorgeschlagen* in [[ADR-0002 Physik-Engine-Kandidat]] festgehalten und wird erst nach bestandenem S1 geschlossen.

## Quellen

- npm: `@dimforge/rapier3d-compat` 0.21.0 (`dist/pipeline/physics_hooks.d.ts`, `package.json`), `jolt-physics` 1.1.1 (`dist/types.d.ts`, `README.md`, `package.json`).
- [Rapier-JS-Schnittstelle `PhysicsHooks`](https://rapier.rs/javascript3d/interfaces/PhysicsHooks.html) · [Rapier Collider Active Hooks (JS-Leitfaden)](https://rapier.rs/docs/user_guides/javascript/collider_active_hooks/)
- [Jolt `ContactSettings` (C++-Doku)](https://jrouwe.github.io/JoltPhysics/class_contact_settings.html) · [Jolt deterministische Simulation](https://jrouwe.github.io/JoltPhysics/#deterministic-simulation)
- [README des deterministischen Rapier-Builds](https://cdn.jsdelivr.net/npm/@dimforge/rapier3d-deterministic@0.19.3/README.md)

## Verwandt

[[T12 Rollen und Ketten in der Physik]] · [[T11 Kontakt, Reibung und Restitution]] · [[ADR-0002 Physik-Engine-Kandidat]] · [[A07 Physik-Wrapper]]
