---
title: Offene Fragen
lang: de
tags: [questions, reference]
updated: 2026-10-29
---

🌐 [[Open Questions|EN]] · [[Preguntas abiertas|ES]] · **DE** · [[Startseite (DE)|Index]]

# Offene Fragen

Jede Frage nennt, **was zu entscheiden ist**, **warum es wichtig ist** (welches Ergebnis sich ändert), **Optionen** und **was bis dahin angenommen wird**. Antworten wandern in die zugehörige Spezifikation und werden als beantwortet markiert.

## Beantwortet (29.10.2026)

| Thema               | Antwort                                                                                                                                                    | Umgesetzt in                                                                       |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Standard-Staumodus  | Keiner: Jede Palette folgt der Ketten- bzw. Rollenbewegung; fährt eine weitere ein, bewegt sich die vorhandene mit; das Verhalten ist ein Instanzparameter | [[S01 Rollenförderer]] §3, [[S02 Kettenförderer]]                                  |
| Lichtschranke       | Höhe und Position von Lichtschranke und Reflektor sind Instanzparameter                                                                                    | [[S06 Lichtschranke]]                                                              |
| Palettenausrichtung | Längs auf Rollen, quer auf Ketten                                                                                                                          | S01, S02, [[T06 Fotoelektrische Erfassung]]                                        |
| Palettentyp         | Europalette (EUR / EPAL)                                                                                                                                   | [[S10 Palettentypen]]                                                              |
| Rollen              | Ø 89 mm, Teilung 206,3 mm; Durchmesser und Teilung aus wenigen Optionen wählbar                                                                            | S01 §2, `roller-conveyor.json`                                                     |
| KI-Agenten          | Von Anfang an für Agenten ausgelegt (direkt und MCP); eigenes Gateway + Token; nur dokumentieren                                                           | [[A15 KI-Agenten-Integration]], [[ADR-0004 Befehlsschicht und MCP für KI-Agenten]] |

## Offen

### F1. Liste der Rollendurchmesser und -teilungen
- **Entscheiden:** welche Kombinationen aus Durchmesser und Teilung die Auswahlliste anbietet.
- **Warum:** bestimmt die Rollen unter der Palette (3-Rollen-Regel), die Lücke für die Umsetzerketten und die 3D-Rollendrehzahl.
- **Konkrete Fragen:** (a) Welche Durchmesser außer 89 mm? (b) Gehört zu jedem Durchmesser eine feste Teilung (Paare), oder sind sie frei kombinierbar? (c) Ist die Teilung immer ein Vielfaches der Antriebskette? 206,3 mm ≈ 13 × 15,875 mm (5/8"-Kette). (d) 76/133 aus dem Plan als Option behalten oder entfernen?
- **Angenommen:** Durchmesser {76, 89}, Teilungen {133, 206,3}, frei kombinierbar; Standard 89/206,3.

### F2. Genaues Verhalten ohne Stau (`none`)
- **Entscheiden:** was passiert, wenn die vorderste Palette das Ende erreicht und der nächste Förderer nicht frei ist.
- **Warum:** ändert effektive Kapazität, Blockierzeit und Halteposition.
- **Konkrete Fragen:** (a) Hält der Förderer, wenn die Vorderkante das Förderende erreicht, oder wenn ein Endsensor X mm davor sie erkennt? Wie groß ist X? (b) Bremst er mit der normalen Rampe (die Palette fährt um $v^2/2b$ = 62,5 mm bei 0,25 m/s über den Sensor) oder hält er an einer exakten Position? (c) Darf, während er mit einer Palette am Ende steht, eine weitere einfahren? (Bei `none` würde das die vorderste bewegen, ich nehme also **nein** an.) (d) Gibt es immer einen Standard-Endsensor?
- **Angenommen:** virtueller Endsensor am Ende; der Förderer bremst mit seiner Rampe; er nimmt nicht auf, solange er blockiert ist.

### F3. Geschwindigkeit bei der Übergabe zwischen Förderern mit unterschiedlicher Geschwindigkeit
- **Entscheiden:** die Geschwindigkeit einer Palette, die in 2D auf A und B aufliegt.
- **Warum:** Durchlaufzeiten und 2D/3D-Konsistenz (±5 %).
- **Optionen:** (1) die langsamere; (2) die des aufnehmenden Förderers; (3) die des abgebenden, bis der Schwerpunkt übergeht.
- **Angenommen:** die langsamere.

### F4. Kettenförderer mit EUR quer: Breite und Führungen
- **Entscheiden:** Standard-Nutzbreite und ob es Seitenführungen gibt.
- **Warum:** mit 1200 mm quer besteht eine Breite von 1000 mm die Validierung nicht.
- **Konkrete Fragen:** (a) Läuft die Palette zwischen Rahmen/Führungen (Breite ≥ 1250 mm) oder steht sie ohne Führungen über die Ketten über? (b) Wie viele Stränge in welchem Abstand?
- **Angenommen:** Nutzbreite 1300 mm, 3 Stränge, Führungen auf beiden Seiten.

### F5. Kettendrehtisch mit EUR quer: Plattformgröße
- **Entscheiden:** Standardmaße der Plattform.
- **Warum:** Der Plan nennt 1400 (Förderrichtung) × 1200 (quer); eine EUR quer belegt 800 × 1200 – genau die Breite. Beeinflusst Schwenkradius und Abstände.
- **Konkrete Frage:** Welche Maße haben deine Ketten- und Rollendrehtische?
- **Angenommen:** die Werte des Plans.

### F6. Rollen-Ketten-Umsetzer
- **Entscheiden:** Mechanismus und Werte bestätigen.
- **Warum:** Komponente mit dem höchsten Risiko (Phase 8); ihr Zyklus begrenzt den Durchsatz.
- **Konkrete Fragen:** (a) Stimmt es: Längsrollen + Hubketten zwischen den Rollen, die heben, um 90° umsetzen und senken? (b) Anzahl Ketten, Hub (40 mm), Hebe-/Senkzeit (1 s). (c) Liegt der anschließende Kettenförderer 40 mm höher als die Rollen, oder senken die Ketten, um auf gleicher Höhe zu übergeben? (d) Gibt es einen Positionieranschlag?
- **Angenommen:** Interpretation des Plans; mit Ø89/206,3 ist die Rollenlücke 117,3 mm (Ketten bis ~107 mm passen).

### F7. Lichtschranken: Typ und übliche Höhen
- **Entscheiden:** Standardtyp und -höhe beim Platzieren einer neuen Lichtschranke.
- **Warum:** Mit EUR flackert das Signal in 60 mm (Lücken von 382,5 mm auf Rollen, 227,5 mm auf Ketten); durchgehende Bänder sind ≈ 122–144 mm auf Rollen und 100–122 mm auf Ketten.
- **Konkrete Fragen:** (a) Reflexionslichtschranke (Sensor + Reflektor) oder Einweglichtschranke? (b) Übliche Montagehöhen auf Rollen und auf Ketten? (c) Sind schräge oder geneigte Strahlen relevant, oder immer quer und waagerecht? (d) Nutzt ihr SPS-Filter/Zeitglieder für die Lücken?
- **Angenommen:** Reflexion, 60 mm (Plan) mit Lückenwarnung.

### F8. Ladungen
- **Entscheiden:** typische und maximale Lastmasse und -höhe; fest oder variabel.
- **Warum:** Trägheit, Drehtisch-Moment, Aufprallenergie, Kippgefahr.
- **Angenommen:** 600 kg, 1200 mm hoch; maximal 1000 kg (bis 1500 kg?).

### F9. Reale Geschwindigkeiten und Rampen
- **Entscheiden:** Standardwerte deiner Anlagen für Rollen, Ketten, Drehtische und Umsetzer.
- **Angenommen:** 0,25 m/s und 0,5 m/s²; Drehtisch 15 °/s und 30 °/s²; Umsetzer 0,2 m/s.

### F10. Referenzmodelle zum Vergleich
- **Entscheiden:** drei Modelle, die du gut kennst (einfacher Fluss, Stau, Drehtisch oder Umsetzer), mit ihren Ergebnissen (Durchsatz, Zykluszeit, Belegung).
- **Warum:** Kriterium 8 der Definition of Done des MVP.

### F11. KI-Agenten
- **Konkrete Fragen:** (a) Welche Anbieter oder Gateways soll der integrierte Chat zuerst unterstützen (Anthropic direkt, Firmen-Gateway, andere Anbieter, lokale Modelle)? (b) Welche MCP-Clients wirst du nutzen? (c) Darf ein Agent das Layout ohne Bestätigung ändern, oder fragt er immer? (d) Nur lokal oder auch Fernzugriff auf den MCP-Server?
- **Angenommen:** standardmäßig nur lesend; Bestätigung bei Löschen/Überschreiben; MCP nur lokal.

### F12. 3D-Hardware, Lizenz und Name
- (a) Laptop mit eigener Grafikkarte oder auch einfache Rechner? (b) Open Source, intern oder kommerziell? (c) Prüfen wir die Marke IntraCraft (Domain, GitHub, npm)?

### F13. Reale Palettenmaße
- **Entscheiden:** ob deine Paletten dem in [[S10 Palettentypen]] genutzten EPAL-Aufbau entsprechen (Bodenbretter 100/145/100 mm, 78-mm-Klötze, 22-mm-Querbretter, 5 Deckbretter).
- **Warum:** Daraus ergeben sich die Lücken, die Lichtschranken sehen, und die Auflage auf Ketten.
- **Bitte:** falls möglich, eine reale Palette vermessen (Klotz- und Brettpositionen).
