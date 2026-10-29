---
title: T12 Rollen und Ketten in der Physik
lang: de
tags: [theory, physics, conveyors]
---

🌐 [[T12 Rollers and Chains in Physics|EN]] · [[T12 Rodillos y cadenas en física|ES]] · **DE** · [[Startseite (DE)|Index]]

# T12 Rollen und Ketten in der Physik

## 1. Angetriebene Rollen

Eine angetriebene Rolle ist ein **kinematischer Zylinder**, der um seine Achse dreht mit

$$\omega = \frac{v}{r} = \frac{v}{D/2}$$

Ø89 mm (Projektstandard) bei 0,25 m/s → $\omega = 0{,}25/0{,}0445 = 5{,}62$ rad/s; Ø76 mm → 6,58 rad/s (`rollerAngularVelocity`, getestet). Die Rollenoberseite bewegt sich mit $\omega r = v$; die Reibung im Kontakt Brett–Rolle zieht die Palette auf diese Geschwindigkeit ([[T11 Kontakt, Reibung und Restitution]]).

Rampen erzeugt der **Wrapper**, nicht die Engine: In jedem Schritt setzt er $\omega(t)$ jeder Zone nach demselben Trapezprofil wie der 2D-Modus ([[T04 Kinematik von Förderern]]).

### Traktionsgrenze

Sind alle tragenden Rollen angetrieben, kann die Reibung die Palette höchstens mit

$$a_{max} = \mu\,g$$

beschleunigen: $\mu = 0{,}35$ → 3,43 m/s² (`maxTractionAccel`, getestet). Die Standardrampe von 0,5 m/s² liegt 7-fach darunter; die Palette folgt den Rollen mit minimalem Mikroschlupf. Schlupf entsteht, wenn μ niedrig ist (Nässe, Kunststoffpaletten), nur ein Teil der Rollen angetrieben ist, die Palette gehalten wird (Anschlag, Stau) oder die Rampe zu steil ist.

### Auflage und 3-Rollen-Regel

Eine Palettenseite der Länge $L$ liegt auf $\lfloor L/p \rfloor$ Rollen (Teilung $p$): 1200 mm bei 206,3 mm → 5 Rollen (bei 133 mm: 9). Unter jeder tragenden Seite müssen immer mindestens **3** liegen, damit die Palette nie in eine Lücke kippt und den Spalt zwischen Förderern überbrückt. Die Lastverteilung auf die Rollen ist statisch unbestimmt; die Engine löst sie, für die Reibung zählt nur die Summe.

### Nicht angetriebene Rollen (Schwerkraft)

Ohne Gefälle (MVP) lassen freie Rollen nur eine geschobene Palette passieren. Jede als freien dynamischen Körper mit Drehgelenk zu simulieren ist teuer. **Vorgeschlagenes MVP-Modell:** freie Rollen werden zu einer statischen Auflage mit niedriger Reibung gleich dem Rollwiderstandsbeiwert ($C_{rr} \approx 0{,}02$): Widerstandskraft $C_{rr}\,m\,g$.

### Blockierte Rollen (Drehtische)

Auf einem Rollendrehtisch sind die Rollen während der Drehung blockiert ($\omega = 0$). Der Kontakt ist dann **Gleiten**, nicht Rollen, mit vollem $\mu$ – genau das hält die Palette ([[T13 Drehtische und Kettenumsetzer]]). Freie Rollen würden die Palette bei der Winkelbeschleunigung wegrollen lassen.

## 2. Ketten: der schwierige Teil

Ein realer Kettenförderer hat 2–6 Stränge; jedes Trum gleitet auf einer Gleitleiste, die Palette liegt auf den Laschen oder Rollen der Kette. EUR-Paletten fahren auf Ketten **quer** (Schmalseite voran, vom Verantwortlichen bestätigt), die Bodenbretter liegen also *quer* zu den Ketten, und jedes Brett berührt jeden Strang über die Kettenbreite (40 mm).

Die Schwierigkeit: Die Kontaktfläche bewegt sich (die Kette), der Körper darunter (die Schiene) nicht. Eine Physik-Engine braucht entweder viele bewegte Körper oder einen Kontakt, der seine Oberflächengeschwindigkeit „kennt“.

### Die vier Pläne

| Plan | Modell                                                                     | Vorteile                             | Nachteile                                                                           |
| ---- | -------------------------------------------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------- |
| A    | Streifen kleiner kinematischer Rollen (Ø 30 mm alle 40 mm), $\omega = v/r$ | In jeder Engine; keine Hooks         | Viele Körper (3 Stränge × 75 Rollen je 3 m = 225); holpriger Kontakt; mehr Kontakte |
| B    | Echte **Oberflächengeschwindigkeit** am Kontakt                            | Ein Quader je Strang; glatt; günstig | Braucht Unterstützung im JS-Binding der Engine                                      |
| C    | Schleppkraft $F = \text{clamp}(k\,m\,(v_c - v_p),\ \pm\mu N)$              | Einfach                              | $N$ muss geschätzt werden; kein echtes Kontaktmodell                                |
| D    | Eigenes Rust-Physikmodul (WASM) mit `modify_solver_contacts`               | Volle Kontrolle                      | Bringt Rust und eine Build-Kette mit; letzter Ausweg                                |

Plan A mit Ø30 mm bei 0,25 m/s: $\omega = 0{,}25/0{,}015 = 16{,}7$ rad/s.

### Rechercheergebnis (28.10.2026)

- **Jolt** (`jolt-physics` 1.1.1 auf npm): Die TypeScript-Typen bieten `ContactListenerJS` mit `OnContactAdded` / `OnContactPersisted(body1, body2, manifold, ioSettings)` und `ContactSettings.mRelativeLinearSurfaceVelocity` / `mRelativeAngularSurfaceVelocity`. **Plan B erscheint mit Jolt machbar** – der Spike muss Vorzeichenkonvention, Kosten eines JS-Callbacks je Kontakt und Determinismus bestätigen.
- **Rapier** (`@dimforge/rapier3d-compat` 0.21.0): Die JS-Schnittstelle `PhysicsHooks` hat nur `filterContactPair` und `filterIntersectionPair` (`ActiveHooks`: `FILTER_CONTACT_PAIRS`, `FILTER_INTERSECTION_PAIRS`). **Keine Änderung der Solver-Kontakte → Plan B ist nicht verfügbar**; Rapier müsste Plan A (oder D) nutzen.

Details in [[T15 Recherche Physik-Engines]] und [[ADR-0002 Physik-Engine-Kandidat]].

### So wirkt die Oberflächengeschwindigkeit (Plan B)

Normalerweise treibt der Solver die relative Tangentialgeschwindigkeit im Kontakt auf null (Haften). Mit einer Oberflächengeschwindigkeit $\mathbf{u}$ wird das Ziel $\mathbf{v}_{rel,t} = \mathbf{u}$: Die feste Schiene verhält sich, als bewegte sich ihre Oberfläche mit Kettengeschwindigkeit. Die Reibgrenze $\mu N$ gilt weiter, eine gehaltene Palette rutscht also genau wie auf einer echten Kette.

## 3. Stau mit Schlupf

Unter einer gehaltenen Palette läuft die Kette (Rollen aus Plan A oder Oberfläche aus Plan B) weiter: Die Gleitreibung drückt die Palette mit $\mu\,m\,g$ (je Palette) nach vorn, und die Kräfte addieren sich entlang des Staus ([[T05 Stau, Kapazität und Durchsatz]] §6.3). Beide Pläne bilden das natürlich ab; der 3D-Modus meldet dann die Kraft auf den Anschlag.

## 4. Polygoneffekt

Eine Kette um ein Kettenrad mit $z$ Zähnen läuft nicht mit konstanter Geschwindigkeit: Die Geschwindigkeit schwankt mit der relativen Amplitude $1 - \cos(\pi/z)$ – 1,7 % bei $z = 17$. Der Parameter `chainPitch` ist dafür reserviert; der MVP ignoriert den Effekt (konstante Kettengeschwindigkeit).

## 5. Abnahmetests (Spike S1)

| Test | Aufbau                                      | Kriterium                                                                     |
| ---- | ------------------------------------------- | ----------------------------------------------------------------------------- |
| a    | 1000-kg-Palette auf 10 kinematischen Rollen | Erreicht Nenngeschwindigkeit ±3 %, bremst ohne übermäßigen Schlupf            |
| b    | Streifen kleiner Rollen als Kette           | Palettengeschwindigkeit = Kettengeschwindigkeit ±3 %, kein sichtbares Zittern |
| f    | 20 Förderer, 10 Paletten                    | 60 FPS, keine numerischen Instabilitäten                                      |

## Verwandt

[[T11 Kontakt, Reibung und Restitution]] · [[T13 Drehtische und Kettenumsetzer]] · [[S01 Rollenförderer]] · [[S02 Kettenförderer]] · [[T15 Recherche Physik-Engines]]
