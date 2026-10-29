---
title: A05 Komponentenbibliothek
lang: de
tags: [app, library]
---

🌐 [[A05 Component Library|EN]] · [[A05 Librería de componentes|ES]] · **DE** · [[Startseite (DE)|Index]]

# A05 Komponentenbibliothek

## 1. Begriffe

| Begriff        | Bedeutung                                                                                                        |
| -------------- | ---------------------------------------------------------------------------------------------------------------- |
| Definition     | Ein Komponenten**typ**: Parameter, Ports, Signale, Geometrie, Verhalten. Liegt in der Bibliothek.                |
| Instanz        | Eine Verwendung einer Definition im Layout, mit eigenen Parameterwerten.                                         |
| Voreinstellung | Benannter Wertesatz einer Definition (z. B. „Schwer 1000 kg, 0,25 m/s“).                                         |
| Herkunft       | *integriert* (schreibgeschützt), *Benutzer* (bearbeitbar, teilbar), *Projekt* (wird mit dem Layout gespeichert). |

Definitionen tragen eine semantische Version; ein Layout legt die verwendete Version fest.

## 2. Inhalt einer Definition

Umgesetzt in `packages/library/src/definition-schema.ts`:

| Feld                  | Bedeutung                                                                   |
| --------------------- | --------------------------------------------------------------------------- |
| `id`                  | `kategorie.name`, z. B. `conveyor.roller`                                   |
| `version`             | Semantische Version `x.y.z`                                                 |
| `category`            | conveyors, sensors, mechanics, flow, pallets, control                       |
| `name`, `description` | Übersetzter Text `{ en, es, de }`                                           |
| `parameters`          | Liste der Parameterdefinitionen (§3)                                        |
| `ports`               | Id, Richtung (in/out/inout), lokale Position (Zahl oder Parameterschlüssel) |
| `signals`             | Namen der Ein- und Ausgangssignale                                          |
| `statistics`          | Namen der veröffentlichten Kennzahlen                                       |
| `geometry`            | Id des Geometriegenerators                                                  |
| `behaviors`           | Ids `{ des, physics }` der registrierten Verhalten                          |
| `validationRules`     | Ids der auszuführenden Validierungsregeln                                   |
| `presets`             | Benannte Wertesätze                                                         |

## 3. Inhalt eines Parameters

| Feld                            | Bedeutung                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------ |
| `key`                           | Bezeichner (englisches camelCase, z. B. `rollerPitch`)                         |
| `label`, `description`          | Übersetzt `{ en, es, de }`                                                     |
| `type`                          | number, integer, boolean, enum, text                                           |
| `unit`                          | **Gespeicherte** SI-Einheit (`m`, `m/s`, `kg` …)                               |
| `displayUnit`                   | Im Editor angezeigte Einheit (`mm`, `°` …)                                     |
| `default`, `min`, `max`, `step` | Wert und Bereich in der gespeicherten Einheit                                  |
| `options`                       | Werte eines Enums                                                              |
| `choices`                       | Zulässige Werte eines Zahlenparameters (Auswahlliste), z. B. Rollendurchmesser |
| `group`                         | Gruppe im Eigenschaftsbereich (Geometrie, Bewegung, Rollen …)                  |
| `advanced`                      | Standardmäßig ausgeblendet                                                     |
| `affects`                       | `2d`, `3d`, `both` oder `geometry`                                             |
| `derived`, `formula`            | Schreibgeschützter berechneter Wert, z. B. `floor(length / rollerPitch)`       |
| `dependsOn`                     | Bedingte Sichtbarkeit, z. B. `drive=zoned`                                     |

Der Eigenschaftsbereich wird aus diesem Schema **erzeugt**: keine handgeschriebenen Formulare.

## 4. Daten und Code

JSON beschreibt, **was** eine Komponente ist; per Id registrierte TypeScript-Verhalten setzen um, **wie** sie sich verhält. `createBuiltInRegistry()` lädt und validiert beim Start alle integrierten JSON-Definitionen; eine ungültige Definition schlägt deutlich fehl. `defaultParams(definition)` liefert die nicht abgeleiteten Standardwerte.

## 5. Validierungsregeln (umgesetzt)

| Regel                     | Prüfung                                           | Beispiel                 |
| ------------------------- | ------------------------------------------------- | ------------------------ |
| `min3RollersUnderPallet`  | Teilung ≤ tragende Seite / 3                      | 206,3 mm unter 1200 mm ✓ |
| `usableWidthFitsPallet`   | Nutzbreite ≥ Palettenbreite + 2 × Führungsabstand | 1000 ≥ 800 + 50 ✓        |
| `chainFitsBetweenRollers` | Teilung − Durchmesser ≥ Kettenbreite + 10 mm      | 117,3 ≥ 40 + 10 ✓        |

Geplant: Schwenkradius von Drehtischen gegen Nachbarn, Aufprallgeschwindigkeit gegen Grenzwert, Höhentoleranz der Ports, Zonenlänge ≥ Palette + 100 mm.

## 6. Bibliotheksumfang des MVP und Stand

| Kategorie  | Komponenten                                                              | Stand (28.10.2026)                                  |
| ---------- | ------------------------------------------------------------------------ | --------------------------------------------------- |
| Förderer   | Rollen, Ketten, Kettendrehtisch, Rollendrehtisch, Rollen→Ketten-Umsetzer | Rollen: Definitions-JSON; übrige: nur Spezifikation |
| Sensoren   | Lichttaster, Reflexions- und Einweglichtschranke                         | Nur Spezifikation                                   |
| Mechanik   | Fester Anschlag, versenkbarer Anschlag, Seitenführung (mit Trichter)     | Nur Spezifikation                                   |
| Fluss      | Quelle, Senke                                                            | Nur Spezifikation                                   |
| Paletten   | EUR, 1200 × 1000, GMA, benutzerdefiniert                                 | EUR, 1200 × 1000, GMA in `pallet-types.json`        |
| Werkstoffe | Tabelle der Reibpaarungen                                                | `friction-pairs.json`                               |
| Steuerung  | Regelsteuerung und Vorlagen                                              | Regelauswerter + ZPA-Testvorlage                    |

## 7. Definition of Done eines Bibliotheksobjekts

1. Spezifikation in `docs/<sprache>/specs/` (drei Sprachen).
2. Definitions-JSON mit Parametern, Bereichen, abgeleiteten Werten und Validierungen.
3. Parametrische Geometrie und 2D-Symbol.
4. 2D-Verhalten mit analytischen Tests.
5. 3D-Verhalten mit Physiktests.
6. Steuerungsvorlage.
7. Referenzszenario in `packages/scenarios`.
8. Aus der Definition erzeugte Parameterdokumentation.

## Verwandt

[[A04 Datenmodell und Projektdatei]] · [[S01 Rollenförderer]] · [[S10 Palettentypen]] · [[ADR-0003 Dreisprachige Doku und englische Bezeichner]]
