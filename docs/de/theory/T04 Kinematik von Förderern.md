---
title: T04 Kinematik von Förderern
lang: de
tags: [theory, des, kinematics]
---

🌐 [[T04 Conveyor Kinematics|EN]] · [[T04 Cinemática de transportadores|ES]] · **DE** · [[Startseite (DE)|Index]]

# T04 Kinematik von Förderern

Im 2D-Modus „spürt“ eine Palette keine Kräfte: Sie folgt vorgegebenen **Bewegungsprofilen**. Diese Notiz leitet alle Formeln her, die die DES-Engine verwendet. Code: `packages/sim-des/src/kinematics.ts`.

## 1. Bewegungsgrundformen

Bei konstanter Beschleunigung $a$ ab $t_0$ mit Position $s_0$ und Geschwindigkeit $v_0$:

$$v(t) = v_0 + a\,(t - t_0) \qquad s(t) = s_0 + v_0 (t - t_0) + \tfrac12 a (t - t_0)^2$$

Zwei nützliche Beziehungen:

- Weg, um von $v_1$ auf $v_2$ zu kommen: $\;d = \dfrac{v_2^2 - v_1^2}{2a}$
- Zeit dafür: $\;t = \dfrac{v_2 - v_1}{a}$

Eine vollständige Bewegung ist eine **stückweise** Folge solcher Segmente `{t0, s0, v0, a}`; die Engine speichert je Palette nur das aktuelle Segment.

## 2. Trapezprofil (Start und Ende im Stillstand)

Mit $a$ auf $v_{max}$ beschleunigen, konstant fahren, mit $b$ bis zum Stillstand verzögern, Gesamtweg $D$.

```text
 v
 │      ┌───────────────┐   v_max
 │     /                 \
 │    /                   \
 │   /                     \
 └──┴───────────────────────┴──── t
     t_a      t_c          t_b
```

- Rampenwege: $d_a = \dfrac{v_{max}^2}{2a}$, $\;d_b = \dfrac{v_{max}^2}{2b}$
- Falls $D \ge d_a + d_b$ (Fahrgeschwindigkeit wird erreicht):

$$t = \frac{v_{max}}{a} + \frac{D - d_a - d_b}{v_{max}} + \frac{v_{max}}{b} = \frac{D}{v_{max}} + \frac{v_{max}}{2a} + \frac{v_{max}}{2b}$$

- Sonst ist das Profil **dreieckig**: Spitzengeschwindigkeit $v_p = \sqrt{\dfrac{2 D a b}{a + b}}$ und $t = \dfrac{v_p}{a} + \dfrac{v_p}{b}$. Mit $a = b$: $t = 2\sqrt{D/a}$.

## 3. Ankunft mit Fahrgeschwindigkeit (ohne Bremsen) — Test T1

Eine Palette, die den Förderer mit voller Geschwindigkeit verlässt (Übergabe an den nächsten), hat keine Bremsphase:

$$t = \frac{v}{a} + \frac{D - v^2/2a}{v} = \frac{D}{v} + \frac{v}{2a} \qquad (D \ge v^2/2a)$$

**T1:** $D = 3$ m, $v = 0{,}25$ m/s, $a = 0{,}5$ m/s² → $t = 12 + 0{,}25 = \mathbf{12{,}25\ s}$. Die Rampe kostet genau $v/2a = 0{,}25$ s gegenüber einem Sofortstart. Falls $D < v^2/2a$: $t = \sqrt{2D/a}$.

## 4. Drehungen nutzen dieselben Formeln — Test T3

Weg durch Winkel $\theta$, Geschwindigkeit durch $\omega$, Beschleunigung durch $\alpha$ ersetzen. Mit $\alpha$ für beide Rampen:

- Dreieckig, falls $\omega^2/\alpha \ge \theta$: $\;t = 2\sqrt{\theta/\alpha}$
- Sonst trapezförmig: $\;t = \theta/\omega + \omega/\alpha$

**T3:** $\theta = 90°$, $\omega = 15$ °/s, $\alpha = 30$ °/s² → $\omega^2/\alpha = 7{,}5° < 90°$, also $t = 6 + 0{,}5 = \mathbf{6{,}5\ s}$. (Die Einheiten kürzen sich, Grad können direkt verwendet werden; der Code rechnet in Radiant.)

## 5. Zyklus des Kettenumsetzers — Test T4

$$t = t_{heben} + \left(\frac{d}{v} + \frac{v}{a}\right) + t_{senken} \;(+\ t_{beruhigen})$$

**T4:** $d = 1$ m, $v = 0{,}2$ m/s, $a = 0{,}5$ m/s², 1 s Heben, 1 s Senken → $1 + (5 + 0{,}4) + 1 = \mathbf{7{,}4\ s}$. Kontrolle: $v^2/a = 0{,}08$ m < 1 m, die Kette erreicht also ihre Fahrgeschwindigkeit.

## 6. Schleichgeschwindigkeit vor einem Anschlag

Reale Förderer verzögern vor einem Anschlag auf `approachSpeed`, um die Aufprallenergie zu begrenzen ([[T14 Anschläge, Aufprall und Ausrichtung]]). Von Fahrgeschwindigkeit $v$ auf Schleichgeschwindigkeit $v_c$:

$$d_{brems} = \frac{v^2 - v_c^2}{2b}, \qquad t_{brems} = \frac{v - v_c}{b}$$

Die Verzögerung muss bei $s_{Anschlag} - d_{brems} - d_{schleich}$ beginnen; $d_{schleich}$ ist der mit Schleichgeschwindigkeit gefahrene Weg (meist von einem Vorsensor ausgelöst). Mit Standardwerten ($v = 0{,}25$, $v_c = 0{,}10$, $b = 0{,}5$): $d_{brems} = 0{,}0525$ m, $t_{brems} = 0{,}3$ s.

## 7. Überfahrten vorhersagen: `timeToReach`

Um „Vorderkante erreicht Sensor bei $s^{\star}$“ zu planen, wird $s(t) = s^{\star}$ nach dem frühesten $t \ge t_0$ gelöst:

$$\tfrac12 a\,\Delta t^2 + v_0\,\Delta t - (s^{\star} - s_0) = 0$$

- $a = 0$: $\Delta t = (s^{\star} - s_0)/v_0$, falls $v_0 \ne 0$ und das Ergebnis $\ge 0$ ist.
- $a \ne 0$: Diskriminante $\Delta = v_0^2 + 2a(s^{\star} - s_0)$. Bei $\Delta < 0$ wird das Ziel nie erreicht (eine bremsende Palette bleibt vorher stehen). Sonst die kleinste nichtnegative Wurzel von $\Delta t = (-v_0 \pm \sqrt{\Delta})/a$.

**Achtung:** Ein Bremssegment gilt nur bis $v = 0$. Die Engine muss es bei $t_0 + v_0/|a|$ beenden (Ereignis „gestoppt“), damit die Parabel nie rückwärts extrapoliert wird.

**Numerischer Hinweis:** Für $v_0^2 \gg |2a\Delta s|$ verliert $(-v_0 + \sqrt{\Delta})/a$ an Genauigkeit (Differenz fast gleicher Zahlen). Die gleichwertige Form $\Delta t = \dfrac{2\,\Delta s}{v_0 + \sqrt{\Delta}}$ ist stabil; sie sollte übernommen werden, falls ein Test Rundungsprobleme zeigt.

## 8. Vorderkante, Hinterkante und Ausrichtung

Die Engine verfolgt die **Vorderkante**; die Hinterkante ist $s_{hinten} = s_{vorn} - L$, wobei $L$ die Palettenabmessung *in Förderrichtung* ist: 1,2 m bei einer EUR-Palette längs, 0,8 m quer. EUR-Paletten fahren auf Rollenförderern längs und auf Kettenförderern quer (vom Verantwortlichen am 29.10.2026 bestätigt).

## 9. Übergabe zwischen Förderern

Solange eine Palette auf A → B aufliegt, tragen sie beide. Vorgeschlagene 2D-Regel (zu bestätigen in [[S01 Rollenförderer]]):

- Laufen beide, bewegt sich die Palette mit $\min(v_A, v_B)$ (die langsamere Fläche dominiert über die Reibung).
- Steht B oder ist B voll, hält die Palette mit der Vorderkante am Einlauf von B (minus Spalt) – **Blockierung**.
- Der physische Spalt zwischen Förderern (Standard 20 mm) wird in 2D sofort überbrückt; in 3D ist es ein echter Spalt, den die Bretter überfahren müssen.

## 10. Kurzreferenz mit Standardwerten

| Größe                                 | Formel              | Ergebnis (Standard) |
| ------------------------------------- | ------------------- | ------------------- |
| Rampenzeit auf 0,25 m/s               | $v/a$               | 0,5 s               |
| Rampenweg                             | $v^2/2a$            | 62,5 mm             |
| Zusatzzeit eines Anlaufs (vs. sofort) | $v/2a$              | 0,25 s              |
| Zusatzzeit Anlauf + Halt              | $v/2a + v/2b$       | 0,5 s               |
| Durchlauf 3 m ohne Halt               | $D/v + v/2a$        | 12,25 s             |
| Durchlauf 3 m mit Anlauf und Halt     | $D/v + v/2a + v/2b$ | 12,5 s              |

## Verwandt

[[T02 Ereignisdiskrete Simulation]] · [[T05 Stau, Kapazität und Durchsatz]] · [[T13 Drehtische und Kettenumsetzer]] · [[A14 API-Referenz]]
