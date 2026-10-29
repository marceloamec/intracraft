---
title: A08 Steuerung
lang: de
tags: [app, control]
---

🌐 [[A08 Controller|EN]] · [[A08 Controlador|ES]] · **DE** · [[Startseite (DE)|Index]]

# A08 Steuerung (`packages/control`)

Theorie: [[T07 Steuerungslogik und Signale]].

## 1. Was heute existiert

`rules.ts` — das Wenn/Dann-Regelmodell und ein **reiner** Auswerter:

```ts
type Condition = { signal: string; is: boolean } | { signal: string; edge: 'rising' | 'falling' };
interface Rule { id: string; when: Condition[]; then: { signal: string; value: boolean; delayMs?: number }[] }
evaluateRules(rules, previous, current) → SignalWrite[]   // { signal, value, delayMs, ruleId }
```

- Alle Bedingungen einer Regel werden mit **UND** verknüpft. (ODER = mehrere Regeln.)
- Fehlende Signale gelten als `false`.
- Der Auswerter liest nie eine Uhr und berührt nie die Anlage.

Getestet mit einer ZPA-Vorlage für zwei Zonen sowie Flanken- und Verzögerungsfällen (`rules.test.ts`).

## 2. Wie jeder Modus sie nutzt

| Schritt                 | 2D (DES)                                                     | 3D (Physik)                                |
| ----------------------- | ------------------------------------------------------------ | ------------------------------------------ |
| Wann auswerten          | Bei jedem Ereignis `signal_changed`                          | In jedem Steuerungszyklus (Standard 10 ms) |
| Snapshot `previous`     | Werte vor der Änderung                                       | Werte des vorigen Zyklus                   |
| Schreibvorgang anwenden | Sofort, oder `signal_changed` bei jetzt + Verzögerung planen | Im ersten Zyklus ≥ jetzt + Verzögerung     |
| Schreibkonflikte        | Letzte Regel in Listenreihenfolge gewinnt (deterministisch)  | Ebenso                                     |

## 3. Geplante Erweiterungen

- Regeleditor in der Oberfläche mit Vorlagen (ZPA, Anschlag mit Sensor, Drehtisch wartet auf zentrierte Palette, vollständiger Umsetzablauf).
- Stornierungssemantik verzögerter Schreibvorgänge (Einschaltverzögerung vs. Impuls) – vor Phase 3 festzulegen.
- Signalregister mit Namenskonvention `<instanz>.<signal>` und Prüfung, dass jedes referenzierte Signal existiert.
- Verriegelungen sind **keine** Regeln: Sie liegen im Komponentenverhalten und protokollieren eine Warnung, wenn sie einen Befehl sperren.
- Benutzerskripte (in einer Sandbox) kommen nach dem MVP.

## Verwandt

[[T07 Steuerungslogik und Signale]] · [[A06 DES-Engine]] · [[A07 Physik-Wrapper]]
