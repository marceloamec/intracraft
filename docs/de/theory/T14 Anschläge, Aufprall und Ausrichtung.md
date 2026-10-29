---
title: T14 Anschläge, Aufprall und Ausrichtung
lang: de
tags: [theory, physics, stops, guides]
---

🌐 [[T14 Stops, Impacts and Alignment|EN]] · [[T14 Topes, impactos y alineación|ES]] · **DE** · [[Startseite (DE)|Index]]

# T14 Anschläge, Aufprall und Ausrichtung

## 1. Aufprallenergie — Test T5

$$E = \tfrac12\,m\,v^2$$

| Masse (Palette + Ladung) | 0,10 m/s (Schleichfahrt) | 0,25 m/s | 0,50 m/s |
| ------------------------ | ------------------------ | -------- | -------- |
| 1025 kg                  | 5,1 J                    | **32 J** | 128 J    |

Die Energie wächst mit dem **Quadrat** der Geschwindigkeit: Schleichfahrt (0,10 m/s) senkt die Aufprallenergie gegenüber 0,25 m/s auf ein Sechstel. Dafür gibt es `approachSpeed` ([[T04 Kinematik von Förderern]] §6). Der Simulator protokolliert Geschwindigkeit und Energie jedes Aufpralls und warnt oberhalb von `maxImpactEnergy` (Standard 200 J).

## 2. Impuls, Kontaktdauer und Spitzenkraft

Der vom Anschlag übertragene Impuls ist $J = m\,v\,(1 + e)$. Die mittlere Kraft hängt von der Kontaktdauer ab:

$$\bar{F} = \frac{J}{\Delta t_{Kontakt}}, \qquad \bar{a} = \frac{v(1+e)}{\Delta t_{Kontakt}}$$

Für 1025 kg bei 0,25 m/s mit $e = 0{,}1$: $J = 282$ N·s. Bei 10 ms Kontakt: $\bar F \approx 28$ kN und $\bar a \approx 27{,}5$ m/s².

**Vorsicht bei Starrkörper-Engines:** Ein starrer Anschlag stoppt die Palette in etwa **einem Physikschritt** (8 ms); Spitzenkräfte sind daher ein Artefakt des Zeitschritts, keine Vorhersage. Reale Anschläge haben Gummipuffer oder Dämpfer. Für Entscheidungen die **Energie** verwenden, nicht die simulierte Spitzenkraft.

### Kippt eine hohe Ladung am Anschlag?

Schnelltest über die Energie: Die Palette kippt um ihre vordere Unterkante, wenn ihre kinetische Energie den Schwerpunkt über diese Kante heben kann:

$$\Delta E_{kipp} = m\,g\,\Big(\sqrt{h^2 + (L/2)^2} - h\Big)$$

EUR mit 600 kg Ladung ($h = 0{,}717$ m, $L/2 = 0{,}4$ m quer fahrend): $\Delta E_{kipp} = 625 \cdot 9{,}81 \cdot (0{,}821 - 0{,}717) \approx 638$ J ≫ 32 J → kein Kippen. (Grob: Nur ein Teil der Energie geht in die Drehung; als Vorprüfung verwenden.)

## 3. Rückprall

Mit Stoßzahl $e$ verlässt die Palette den Anschlag mit $e\,v$. Die weiter vorwärts treibenden Rollen holen sie innerhalb von $\approx (e v)^2 / (2\mu g)$ zurück – 0,09 mm bei Standardwerten ([[T11 Kontakt, Reibung und Restitution]] §6). Kriterium P1: Vorderkante innerhalb **±5 mm** der Sollposition und Rückprall **< 10 mm**.

## 4. 2D-Modell eines Anschlags

- Der Anschlag ist eine **Positionsgrenze** der Vorderkante: Das vorhergesagte Ereignis `front_reaches_stop` beendet das Bewegungssegment der Palette bei $s_{Anschlag}$.
- Rollen: Die Palette hält an der Anschlagposition (im Idealmodell sofort).
- Ketten: Die Palette verzögert über `stopContactTime` (Standard 0,2 s), die Ketten gleiten darunter weiter.
- `raiseTime` / `lowerTime` (0,3 s) verzögern die Wirkung eines Befehls. Ein Anschlag darf **nicht unter einer Palette hochfahren**: Verriegelung „Anschlag nur heben, wenn der Bereich darüber frei ist“.

## 5. Ausrichtung mit Anschlägen

Ein Anschlag legt die **Längslage** der Vorderkante fest. Die Kontaktgeometrie entscheidet, ob er auch die **Verdrehung** festlegt:

- **Ein schmaler Kontakt außerhalb der Palettenachse:** Der Stoßimpuls $J$ im seitlichen Abstand $d$ vom Schwerpunkt erzeugt einen Drehimpuls $J d$; die Palette dreht sich ($I\,\Delta\omega = J d$), bis ein zweiter Kontakt oder eine Führung sie stoppt. Die Rollenreibung wirkt der Drehung entgegen, der Endwinkel hängt also von μ, Masse und Trägheit ab – ein reines 3D-Ergebnis.
- **Ein breiter Anschlag oder zwei Anschläge:** Kontakte beidseits des Schwerpunkts gleichen das Moment aus, die Vorderkante wird am Anschlag rechtwinklig ausgerichtet. Deshalb zählt `stopWidth` (Standard 100 mm; breiter richtet besser aus).

In 2D wird nur die ideale Endlage angesetzt; der 3D-Modus liefert die reale Lage (Position + Winkel) – das Maß der Ausrichtungsqualität.

## 6. Seitenführungen und Einlauftrichter

### 6.1 Freiraum

Nutzbreite ≥ Palettenbreite + 2 × Freiraum: EUR 800 mm + 2 × 25 mm = 850 mm ≤ 1000 mm ✓ (`usableWidthFitsPallet`, getestet).

### 6.2 Fangbereich des Trichters

Ein Trichter der Länge $L_f$ mit Winkel $\theta_f$ weitet den Einlauf je Seite um $L_f \tan\theta_f$: 300 mm bei 10° → 53 mm. Er fängt Paletten mit bis zu Freiraum + 53 mm ≈ 78 mm Versatz (Kriterium S1-d / P3: 50 mm und 5° werden auf ±10 mm und ±1° korrigiert).

### 6.3 Der Keil: Vortriebskraft wird zu Seitenschub

Die Rollen schieben die Palette mit der Kraft $F_d$ gegen die schräge Führung. Mit Führungsreibung $\mu_g$ muss die Normalkraft $N$ den Vortrieb ausgleichen: $F_d = N(\sin\theta + \mu_g\cos\theta)$; der Seitenschub ist etwa $N\cos\theta$:

$$F_{seit} \approx \frac{F_d}{\tan\theta + \mu_g}$$

Bei 10° und $\mu_g = 0{,}3$: $F_{seit} \approx 2{,}1\,F_d$. Die Palette muss außerdem **seitlich auf den Rollen rutschen** (Rollen rollen nur in Förderrichtung), was bis zu $\mu_r m g$ kostet. Da die Rollenreibung ein einziger **Reibkegel** ist, teilen sich Vortrieb und seitliches Rutschen dasselbe Budget $\mu_r m g$ – die Palette bewegt sich schräg. Steile Trichter (großes θ) oder hohe Führungsreibung verringern $F_{seit}$ und können die Palette **verklemmen**. Das Ergebnis ist ein 3D-Test, keine Formel; kleine Trichterwinkel (≤ 10°) sind der sichere Standard.

## Verwandt

[[S07 Mechanischer Anschlag]] · [[S08 Seitenführung]] · [[T11 Kontakt, Reibung und Restitution]] · [[T09 Starrkörperdynamik]]
