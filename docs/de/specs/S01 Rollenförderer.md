---
title: S01 Rollenförderer
lang: de
tags: [spec, conveyor, roller]
library-id: conveyor.roller
status: definition-json
updated: 2026-10-29
---

🌐 [[S01 Roller Conveyor|EN]] · [[S01 Mesa de rodillos|ES]] · **DE** · [[Startseite (DE)|Index]]

# S01 Rollenförderer (`conveyor.roller`)

Gerader Rollenabschnitt, der die Palette auf ihren Bodenbrettern in Förderrichtung trägt. **EUR-Paletten fahren auf Rollenförderern längs (Längsseite, 1200 mm, voran)** – vom Verantwortlichen am 29.10.2026 bestätigt. Basiskomponente: Die anderen verwenden ihre Rollengruppe wieder. Definition: `packages/library/src/definitions/roller-conveyor.json`.

Standardwerte sind **Richtwerte**; angezeigt in Anzeigeeinheiten, gespeichert in SI. **Jeder Verhaltensparameter gehört zur Instanz**: Zwei Rollenförderer desselben Typs können unterschiedliche Längen, Kapazitäten und Staumodi haben.

## 1. Gemeinsame Parameter (alle Förderer)

| Schlüssel               | Plan-Schlüssel                 | Einheit | Standard                    | Bereich                           | Beschreibung                                                             |
| ----------------------- | ------------------------------ | ------- | --------------------------- | --------------------------------- | ------------------------------------------------------------------------ |
| `name`                  | `nombre`                       | Text    | auto                        | —                                 | Sichtbarer Name; die interne `id` wird erzeugt                           |
| `position`              | `posicion`                     | m       | 0, 0, 0                     | —                                 | Ursprung (Mitte des Anfangs)                                             |
| `yaw`                   | `giro`                         | °       | 0                           | −180 … 180                        | Drehung um die Hochachse                                                 |
| `surfaceHeight`         | `alturaSuperficie`             | mm      | 600                         | 200 … 1500                        | Höhe der Palettenauflage über dem Boden                                  |
| `length`                | `longitud`                     | mm      | 3000                        | 500 … 12000                       | Länge in Förderrichtung; längere Förderer fassen mehr Paletten           |
| `usableWidth`           | `anchoUtil`                    | mm      | 1000                        | 500 … 1800                        | Lichte Breite zwischen Rahmen oder Führungen                             |
| `palletSideAlongTravel` | `ladoParaleloAvance`           | Enum    | lang                        | lang, kurz                        | Lang bei Rollenförderern (EUR mit 1200 mm in Förderrichtung)             |
| `nominalSpeed`          | `velocidadNominal`             | m/s     | 0,25                        | 0,05 … 0,60                       | Fahrgeschwindigkeit                                                      |
| `approachSpeed`         | `velocidadAproximacion`        | m/s     | 0,10                        | 0,02 … 0,30                       | Schleichgeschwindigkeit vor einem Anschlag                               |
| `accel` / `decel`       | `aceleracion` / `deceleracion` | m/s²    | 0,5 / 0,5                   | 0,05 … 2,0                        | Anfahr- und Bremsrampe                                                   |
| `direction`             | `sentido`                      | Enum    | vorwärts                    | vorwärts, rückwärts, reversierbar | Förderrichtung                                                           |
| `maxPalletLoad`         | `cargaMaxPallet`               | kg      | 1000                        | 100 … 2000                        | Warnt, wenn eine Palette schwerer ist                                    |
| `minGap`                | `separacionMinima`             | mm      | 150                         | 50 … 500                          | Mindestabstand zwischen Paletten (für ZPA / Staudruck und die Kapazität) |
| `accumulationMode`      | `modoAcumulacion`              | Enum    | **none**                    | none, zpa, pressure               | Siehe §3                                                                 |
| `guides`                | `guias`                        | Gruppe  | siehe [[S08 Seitenführung]] | —                                 | Integrierte Seitenführungen                                              |
| `enabled`               | `habilitado`                   | bool    | ja                          | —                                 | Deaktivieren ohne Löschen                                                |
| `color`, `legs`         | `color`, `patas`               | —       | —                           | —                                 | Nur Darstellung                                                          |
| `failures`              | `fallos`                       | —       | aus                         | —                                 | Reserviert (MTBF/MTTR, nach dem MVP)                                     |

**Gemeinsame Signale:** Eingänge `run`, `reverse`, `enable`, je Zone `zone[i].run`; Ausgänge `running`, `occupied`, `zone[i].occupied`, `fault` (reserviert). **Zustände (2D):** gestoppt, beschleunigend, laufend, verzögernd, blockiert. **Kennzahlen:** Auslastung, Blockierzeit, Leerzeit, bearbeitete Paletten, Durchsatz (Paletten/h), mittlere Durchlaufzeit, mittlere Belegung.

## 2. Rollenparameter

Durchmesser und Teilung werden **aus einer kurzen Liste von Standardoptionen gewählt**, nicht frei eingegeben.

| Schlüssel           | Plan-Schlüssel        | Einheit | Standard           | Optionen / Bereich                     | Beschreibung                                                      |
| ------------------- | --------------------- | ------- | ------------------ | -------------------------------------- | ----------------------------------------------------------------- |
| `rollerDiameter`    | `diametroRodillo`     | mm      | **89**             | 76, 89 (vorläufige Liste)              | Außendurchmesser der Rolle                                        |
| `rollerPitch`       | `pasoRodillos`        | mm      | **206,3**          | 133, 206,3 (vorläufige Liste)          | Achsabstand                                                       |
| `rollerMaterial`    | `materialRodillo`     | Enum    | Stahl              | Stahl, beschichteter Stahl, PVC        | Bestimmt die Reibpaarung                                          |
| `rollerMu`          | `muRodillo`           | —       | 0,35               | 0,1 … 0,8                              | Reibung Palette–Rolle                                             |
| `rollingResistance` | `resistenciaRodadura` | —       | 0,02               | 0 … 0,1                                | Rollverlust (freie Rollen)                                        |
| `drive`             | `accionamiento`       | Enum    | durchgehende Kette | Schwerkraft, durchgehende Kette, Zonen | Ein kettengetriebener Strang, unabhängige Zonen oder freie Rollen |
| `zoneLength`        | `longitudZona`        | mm      | 1400               | 1000 … 2000                            | Nur Zonenantrieb                                                  |
| `zoneEndSensor`     | `sensorFinDeZona`     | bool    | ja                 | —                                      | Virtueller Zonenend-Sensor für ZPA                                |
| `rollerInertia`     | `inerciaRodillo`      | kg·m²   | auto               | —                                      | Nur Physik                                                        |

Die Werte 76/133 stammen aus dem Plan; 89/206,3 sind die Werte deiner Anlage. Die endgültige Liste ist eine offene Frage ([[Offene Fragen]]). 206,3 mm entspricht fast genau 13 × 15,875 mm (13 Glieder einer 5/8"-Kette) – die Teilung hängt also vermutlich an der Antriebskette; zu bestätigen.

**Abgeleitet mit Standardwerten:** Rollen auf 3 m $\lfloor 3000/206{,}3 \rfloor$ = **14**; Rollen unter einer 1200-mm-Seite $\lfloor 1200/206{,}3 \rfloor$ = **5** (≥ 3 ✓); freie Lücke zwischen Rollen 206,3 − 89 = **117,3 mm**; Rollendrehzahl für 0,25 m/s $\omega = 0{,}25/0{,}0445$ = **5,62 rad/s**; Kapazität eines 3-m-Förderers $\lfloor 3000/(1200+150) \rfloor$ = **2** EUR-Paletten.

## 3. Stauverhalten (je Instanz)

| Modus               | Verhalten                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **none** (Standard) | Kein Stau. Jede Palette auf dem Förderer folgt den Rollen: Der Förderer läuft oder steht **als Einheit**. Fährt eine neue Palette ein, muss der Förderer laufen, die bereits vorhandenen Paletten **bewegen sich also mit**. Die Abstände zwischen den Paletten bleiben erhalten (niemand schließt auf). Erreicht die vorderste Palette den Auslauf und kann der nächste Förderer sie nicht übernehmen, steht der ganze Förderer (Zustand *blockiert*) und kann keine Paletten mehr aufnehmen, bis er wieder laufen kann. |
| zpa                 | Staudrucklose Förderung: erfordert `drive = zoned`; jede Zone läuft nur, wenn die nächste frei ist; Paletten berühren sich nie ([[T05 Stau, Kapazität und Durchsatz]] §6.2).                                                                                                                                                                                                                                                                                                                                              |
| pressure            | Paletten schließen hinter einer gehaltenen Palette bis `minGap` (oder Kontakt) auf.                                                                                                                                                                                                                                                                                                                                                                                                                                       |

**2D-Modell von `none`.** Alle Paletten des Förderers teilen sich sein Bewegungsprofil (ein kinematisches Segment für den ganzen Förderer, je Palette verschoben). Vorhergesagte Ereignisse: vorderste Palette erreicht den Auslauf (Übergabe oder Blockierung), Hinterkante der letzten Palette gibt den Einlauf frei, Sensorflanken jeder Palette.

## 4. Ports, Verhalten in 3D, Validierungen

- **Ports:** A (Eingang) bei (0, surfaceHeight, 0); B (Ausgang) bei (length, surfaceHeight, 0).
- **3D:** Jede Rolle ist ein kinematischer Zylinder mit $\omega = v/(D/2)$, nach Zonen gruppiert; die Palette bewegt sich durch Reibung an ihren Bodenbrettern und kann rutschen ([[T12 Rollen und Ketten in der Physik]]).
- **Validierungen:** 3-Rollen-Regel (`min3RollersUnderPallet`); Nutzbreite ≥ Palettenbreite + 2 × Führungsabstand (`usableWidthFitsPallet`; EUR längs: 800 + 50 ≤ 1000 ✓); Zonenlänge ≥ Palettenlänge + 100 mm.

## 5. Tests

| Test | Erwartet                                                                                        | Stand              |
| ---- | ----------------------------------------------------------------------------------------------- | ------------------ |
| T1   | 3 m aus dem Stand, 0,25 m/s, 0,5 m/s² → 12,25 s                                                 | ✅ Formel          |
| T2   | EUR längs + 0,15 m Lücke → ≈ 667 Paletten/h                                                     | ✅ Formel          |
| —    | Standards: Ø89, Teilung 206,3, Staumodus `none`                                                 | ✅ Bibliothekstest |
| T6   | Sensor bei 2,0 m, durchgehendes Band → 4,8 s belegt                                             | ⏳ Phase 3         |
| —    | `none`: einfahrende Palette bewegt die vorhandene; blockierter Auslauf hält den ganzen Förderer | ⏳ Phase 3         |
| S1-a | 1000 kg auf 10 kinematischen Rollen → Nenngeschwindigkeit ±3 %                                  | ⏳ Spike S1        |

## Verwandt

[[T04 Kinematik von Förderern]] · [[T05 Stau, Kapazität und Durchsatz]] · [[T12 Rollen und Ketten in der Physik]] · [[A05 Komponentenbibliothek]] · [[Offene Fragen]]
