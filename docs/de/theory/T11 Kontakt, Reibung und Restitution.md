---
title: T11 Kontakt, Reibung und Restitution
lang: de
tags: [theory, physics, contact]
---

🌐 [[T11 Contact, Friction and Restitution|EN]] · [[T11 Contacto, rozamiento y restitución|ES]] · **DE** · [[Startseite (DE)|Index]]

# T11 Kontakt, Reibung und Restitution

In den Kontakten spielt sich die Förderphysik ab: Rollen bewegen die Palette nur über die Reibung in ihren Kontaktpunkten, Anschläge und Führungen wirken nur über Kontakte.

## 1. Die Kollisionspipeline (in jedem Schritt)

1. **Broad Phase** — günstige Hüllquadertests (Hüllkörperhierarchien oder Sweep-and-Prune) finden Paare, die sich berühren *könnten*.
2. **Narrow Phase** — exakte Tests (GJK/EPA, SAT für Quader) berechnen eine **Kontaktmannigfaltigkeit**: bis zu 4 Punkte, eine Normale $\mathbf{n}$ und Eindringtiefen. Ein Brett (Quader) auf einer Rolle (Zylinder) ergibt einen Linienkontakt, meist 2 Punkte.
3. **Constraint-Lösung** — Impulse werden so berechnet, dass sich Körper nicht durchdringen, die Reibung eingehalten und die Restitution angewendet wird.
4. **Integration** — Geschwindigkeiten und Positionen werden aktualisiert ([[T10 Zeitintegration und fester Zeitschritt]]).

## 2. Nichtdurchdringung

In jedem Kontaktpunkt mit relativer Normalgeschwindigkeit $v_n$ und Normalimpuls $\lambda_n$:

$$v_n \ge 0, \qquad \lambda_n \ge 0, \qquad \lambda_n\,v_n = 0$$

(die Körper trennen sich oder bleiben in Kontakt; der Kontakt kann drücken, nie ziehen; er drückt nur, solange Berührung besteht). Dieses *Komplementaritätsproblem* wird iterativ gelöst.

## 3. Coulombsche Reibung

Die Tangentialkraft ist durch die Normalkraft begrenzt:

$$\|\mathbf{F}_t\| \le \mu\,F_n$$

- **Haften** — kann die Reibung die relative Tangentialgeschwindigkeit innerhalb dieser Grenze aufheben, bewegen sich die Flächen gemeinsam.
- **Gleiten** — sonst hat die Reibkraft den Betrag $\mu F_n$ und wirkt der Gleitgeschwindigkeit entgegen.

Die zulässigen Kontaktkräfte bilden den **Reibkegel** (halber Öffnungswinkel $\arctan\mu$); Engines nähern ihn durch eine Pyramide oder durch Begrenzen der beiden Tangentialimpulse an.

**Kernpunkt für Förderer:** Die Reibung wirkt auf die **Relativ**geschwindigkeit zwischen Palette und *Oberfläche* im Kontaktpunkt. Die Oberfläche einer kinematischen Rolle mit $\omega$ bewegt sich oben mit $\omega r$; die Reibung zieht das Brett auf diese Geschwindigkeit. Das ist der ganze Mechanismus eines angetriebenen Rollenförderers ([[T12 Rollen und Ketten in der Physik]]).

**Haft- vs. Gleitreibung.** Reale Werkstoffe haben $\mu_s > \mu_k$. Rapier und Jolt verwenden einen einzigen Koeffizienten je Kollisionskörper. Die Reibungstabelle von IntraCraft liefert einen Wert je Werkstoffpaarung ([[S10 Palettentypen]]).

### Kombination der Koeffizienten

Jeder Kollisionskörper hat seine eigene Reibung; die Engine kombiniert beide:

| Engine | Standardkombination                           | So erhält man genau den Tabellenwert $\mu_{Paar}$                           |
| ------ | --------------------------------------------- | --------------------------------------------------------------------------- |
| Rapier | Mittelwert (einstellbar: Min., Max., Produkt) | Beiden Kollisionskörpern $\mu_{Paar}$ geben oder die Regel je Körper wählen |
| Jolt   | Geometrisches Mittel $\sqrt{\mu_1\mu_2}$      | `mCombinedFriction` je Kontakt im Contact-Listener überschreiben            |

(Standardwerte am 28.10.2026 geprüft mit `@dimforge/rapier3d-compat` 0.21.0 — `frictionCombineRule` = Average, Standardreibung 0,5 — und `jolt-physics` 1.1.1; siehe [[T15 Recherche Physik-Engines]].)

## 4. So arbeitet der Solver: sequentielle Impulse

Die meisten Echtzeit-Engines nutzen **projiziertes Gauß-Seidel / sequentielle Impulse**: Sie durchlaufen alle Kontakte mehrmals, berechnen je Kontakt den Impuls, der den Geschwindigkeitsfehler korrigieren würde, addieren ihn zu einem **akkumulierten** Impuls und begrenzen diesen ($\lambda_n \ge 0$, $|\lambda_t| \le \mu\lambda_n$). **Warm Starting** übernimmt die Impulse des letzten Schritts, sodass eine auf Rollen ruhende Palette in wenigen Iterationen konvergiert. Schwere Körper mit vielen Kontakten brauchen mehr Iterationen – gemessen in S1-a.

## 5. Eindringkorrektur

Numerische Fehler lassen Körper leicht überlappen. Engines drücken sie sanft auseinander (Baumgarte-Terme, Split-Impulse oder weiche Constraints) und erlauben einen kleinen **Slop** (tolerierte Eindringtiefe), um Zittern zu vermeiden.

> ⚠️ Der Slop beeinflusst direkt das ±5-mm-Kriterium am Anschlag. Gemessene Standardwerte: **Jolt** `mPenetrationSlop` = 0,02 m (und `mSpeculativeContactDistance` = 0,02 m); **Rapier** `normalizedAllowedLinearError` = 0,005 m (× `lengthUnit` = 1). Beide liegen auf oder über der Toleranz; IntraCraft muss sie auf ≈ 1 mm senken oder kompensieren. Rapier bietet zusätzlich je Kollisionskörper eine **Contact Skin**.

## 6. Restitution (Rückprall)

Die Stoßzahl verknüpft Trenn- und Annäherungsgeschwindigkeit entlang der Normalen:

$$e = -\frac{v_n^{nachher}}{v_n^{vorher}}, \qquad 0 \le e \le 1$$

$e = 0$: vollplastisch (kein Rückprall); $e = 1$: vollelastisch. Anschläge haben standardmäßig $e = 0{,}1$.

> ⚠️ **Geschwindigkeitsschwelle.** Viele Engines ignorieren Restitution unterhalb einer Mindestannäherungsgeschwindigkeit (Jolt: `mMinVelocityForRestitution`, Standard **1 m/s**, gemessen), um endlose Mikrostöße zu vermeiden. Paletten treffen mit 0,1–0,25 m/s auf, die Restitution würde also stillschweigend ignoriert, wenn die Schwelle nicht gesenkt wird.

**Erwarteter Rückprall.** Mit $e = 0{,}1$ bei 0,25 m/s prallt die Palette mit 0,025 m/s zurück; die weiter vorwärts treibenden Rollen stoppen sie innerhalb von $v^2/(2\mu g) = 0{,}025^2/(2 \cdot 0{,}35 \cdot 9{,}81) \approx 0{,}09$ mm. Das < 10-mm-Kriterium ist leicht erfüllt, außer bei sehr niedriger Reibung.

## 7. Kontinuierliche Kollisionserkennung (CCD)

Ein Körper **tunnelt** durch ein Hindernis, wenn er sich in einem Schritt weiter bewegt, als das Hindernis dick ist. Bei 0,6 m/s und 120 Hz sind das 5 mm – weniger als eine Anschlagfläche, aber CCD ist bei Paletten als Sicherheitsnetz aktiviert, wie der Plan vorsieht.

## 8. Leistung: Kontakte zählen

Eine Palette mit 3 Bodenbrettern über 9 Rollen erzeugt bis zu 27 Paare Brett–Rolle mit je etwa 2 Punkten. Zehn Paletten → etwa 500 Kontaktpunkte je Schritt, 60 000 pro Sekunde bei 120 Hz. Deshalb werden Rollen gruppiert, ruhende Körper schlafen gelegt, und das Kriterium 20 Förderer / 10 Paletten / 60 FPS (S1-f) ist ein echter Test.

## Verwandt

[[T12 Rollen und Ketten in der Physik]] · [[T14 Anschläge, Aufprall und Ausrichtung]] · [[T15 Recherche Physik-Engines]]
