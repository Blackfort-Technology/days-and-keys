# Days & Keys - Designvorgabe

Stand 2026-10-15 · Grundlage: Version 0.1.8 (Android 16, 360 dp, erfundene Daten), Code im Repo
https://github.com/Blackfort-Technology/days-and-keys (Stand `main` vom 2026-10-15),
`docs/gesamtkonzept.md`, `docs/architektur.md`, `docs/vorlagen.md`, `templates/standard-v1.json`.

Diese Vorgabe ändert keine Funktion des Kerns. Wo ein Text oder ein Anzeigefeld im Kern oder in der
Vorlage liegt, steht die Fundstelle dabei; dort wird geändert, nicht in der Oberfläche (AE-06,
AE-22). Bildnummern beziehen sich auf die Bildschirmfotos `01` bis `16`.

Inhalt: 1. Kurzbefund · 2. Prinzipien · 3. Designsystem · 4. Bildschirme · 5. Navigation ·
6. App-Symbol · 7. Umsetzungsreihenfolge · Anhang Textfundstellen

---

## 1. Kurzbefund: die zehn wichtigsten Probleme

| Nr. | Problem | Bild | Fundstelle |
|---|---|---|---|
| 1 | **Ein Status braucht zu viele Schritte.** „Status jetzt" hat fünf Gruppen mit bis zu 21 Fragen untereinander. „Speichern" steht erst ganz am Ende der Scrollfläche, die untere Navigation bleibt sichtbar und nimmt 80 dp weg. Wer nur die Stimmung festhalten will, muss durch alles scrollen. Das 15-Sekunden-Ziel ist so nicht erreichbar, und `vorlagen.md` verspricht „passt auf einen Bildschirm". | 04, 05, 06 | `FormularSeite.kt:106-147` (Speichern Z. 135 in der Scrollspalte), `Oberflaeche.kt:75` (Leiste auch in Formularen), `standard-v1.json` `formulare[0].gruppen` |
| 2 | **Die Tipp-Reihe ist zu klein.** Elf Felder teilen sich 296 dp: je Feld etwa 24 dp breit, 44 dp hoch, Ziffern in `labelMedium` (12 sp). Unter 48 dp, mit zitternder Hand oder Daumen schwer zu treffen. iOS ist noch kleiner (`.caption2`, 40 pt). | 04, 07 | `FormularSeite.kt:255-267`, `ios/.../Formular.swift:209-210` |
| 3 | **Der Auswahlzustand ist kaum zu sehen.** Gewählte Chips („selten", „wenig", „3 Tage") unterscheiden sich nur durch eine Füllung `#DDE3EA` auf Weiß, Kontrast 1,29:1, ohne Häkchen. WCAG 1.4.11 verlangt 3:1, 1.4.1 eine Unterscheidung über Farbe hinaus. Die Felder der Tipp-Reihe haben gegen die Karte 1,20:1. | 05, 06, 07, 14 | `Thema.kt:14-15` (`secondaryContainer`, `surfaceVariant`), `FormularSeite.kt:188-224` (FilterChip ohne `leadingIcon`), `FormularSeite.kt:258` |
| 4 | **Hinweistexte unterschreiten AA.** `secondary #6B7A8F` auf dem Grund `#F7F6F3` hat 4,04:1, auf Weiß 4,37:1, gesetzt in 12 sp. Betroffen sind alle Untertitel, Anker („sehr schlecht"), Achsen und der Satz auf der Sperre. | 02, 04, 08, 15 | `Thema.kt:14`, `Bausteine.kt:30`, `FormularSeite.kt:272`, `Sperre.kt:33-36` |
| 5 | **Die untere Leiste schrumpft Text statt ihn zu skalieren.** Fünf reine Textreiter, „Medikamente" wird per `TextAutoSize` bis auf 8 sp verkleinert und ist schon bei 100 % sichtbar kleiner. Bei 200 % Systemschrift wird alles winzig, das Gegenteil dessen, was die Einstellung will. Keine Symbole als zweites Erkennungsmerkmal. | 02 bis 14 | `Oberflaeche.kt:103-136`, besonders Z. 129 |
| 6 | **Jede Seite hat dieselbe Kopfleiste.** „Days & Keys" als Titel überall (auch für neugierige Blicke über die Schulter), kein Seitentitel, kein Zurück-Pfeil auf Unterseiten, „Hilfe" erscheint auch auf der Hilfe selbst, und „Zurück" steht als eigene Karte am Seitenende. | 02, 04, 16 | `Oberflaeche.kt:67-73`, `Hilfe.kt:55-57` |
| 7 | **Auf der Startseite wiegt alles gleich.** Datum als „15.10.2026", dann Karten gleicher Bauart für Status, Medikamente, 14 gleich große Ereignisknöpfe und Sicherung. Die wichtigste Handlung (Stimmung festhalten) hebt sich nicht ab; Erklärtexte stehen jedes Mal da. | 02, 03 | `Start.kt:38-111`, Datum aus `ansicht.rs:275` (`zeit::datum_text`) |
| 8 | **Einträge sind schwer zu überfliegen.** Die Zusammenfassung schreibt jede Frage aus („Wie viel Antrieb habe ich gerade: 6 · Wie konzentriert bin ich gerade: 5"). Ein vergangener Eintrag heißt „Status jetzt". Das Detail ist ein `AlertDialog` mit „Löschen …" direkt neben „Korrigieren". | 10, 11 | `ansicht.rs:560-567` (`zusammenfassung`), `standard-v1.json` `formulare[0].name`, `Eintraege.kt:40-82` |
| 9 | **Der Verlauf erklärt, bevor er zeigt.** Fünf Zeilen Methodentext stehen über dem Diagramm; die Fragenauswahl ist ein Klappmenü mit 20 Einträgen, das den ganzen Bildschirm verdeckt; Zeitraum und Mittelwert stehen gemischt in einer Chipreihe. Zusätzlich entscheidet die Oberfläche Inhalte (Faktorenliste fest im Code, eigener Hinweistext statt des Kerntexts), gegen AE-06. | 08, 09 | `Verlauf.kt:58-89`, `ansicht.rs:468` und `ansicht.rs:517` |
| 10 | **Lange Formulare mit gemischtem Speichern.** „Mehr" ist eine einzige Seite mit drei verschiedenen Speicherwegen („Plan speichern" mitten im Abschnitt, „Einstellungen speichern" in der Mitte, Sicherung sofort). Beim Präparat liegt „Speichern" halb unter der Leiste, Uhrzeiten werden als Text getippt, „Entfernen" klebt am Feld, Schalterbeschriftungen enthalten Erklärungen in Klammern. | 12, 13, 14 | `Einstellungen.kt:74-131`, `Medikamente.kt:70-105` |

**Außerdem:** Die Hilfe stellt sieben Nummern als gleich schwere dunkle Knöpfe untereinander, der
Hinweis darunter lässt offen, zu welchem Knopf er gehört (Bild 16, `Hilfe.kt:31-43`). Auf der
Begrüßung steht der wichtigste Datenschutzsatz („Wer den Code dieses Geräts kennt …") in der
kleinsten Schrift (Bild 01, `Einrichtung.kt:100`). Die Farbwelt (Schiefergrau `#3D4A5C`) ist kühl
und weicht vom Konzept ab (`gesamtkonzept.md` Abschnitt 6: Petrol, Inter); das Launcher-Symbol zeigt
drei Striche und einen Punkt, also ein generisches Listen-Symbol (`res/drawable/ic_vordergrund.xml`).

---

## 2. Gestaltungsprinzipien

1. **Der erste Tipp zählt.** An einem schlechten Tag reicht Energie oft für eine einzige Angabe, deshalb ist die Stimmung auf der Startseite direkt antippbar und alles andere freiwillig.
2. **Leise und gleichwertig.** Jeder Wert sieht gleich aus, ob er gut oder schlecht ist, weil Farbe, Größe oder Ton sonst eine Bewertung transportieren, die Scham auslöst (AE-27).
3. **Eine Sache pro Blick.** Wer sich schwer konzentriert, braucht pro Bildschirm eine klar erkennbare Hauptaufgabe und Erklärungen erst auf Nachfrage.
4. **Nichts geht verloren, nichts wird verlangt.** Entwürfe sichern sich selbst, Lücken bleiben ohne Kommentar, und es gibt keine Zähler, die zum Weitermachen drängen.
5. **Diskret nach außen.** Über die Schulter gesehen soll die App wie ein ruhiges Notizwerkzeug wirken: kein App-Name in jeder Leiste, keine Diagnosebegriffe, ein neutrales Symbol.
6. **Hilfe ist nie mehr als einen Tipp entfernt.** Auch gesperrt und vor der Einrichtung, weil der Bedarf genau dann kommt, wenn niemand suchen will (AE-20).

---

## 3. Designsystem

### 3.1 Farben

Warm-neutraler Papiergrund, Petrol als einzige Handlungsfarbe, Ocker nur für die
Grundniveau-Linie und das Symbol. Rot (`error`) nur für echte Fehler und zerstörende Aktionen
(Löschen), nie für Werte. Kein dynamisches Farbschema (Material You), damit Kontraste prüfbar
bleiben und die Identität gleich ist.

| Token (M3-Rolle) | Hell | Dunkel | Verwendung |
|---|---|---|---|
| `background` | `#F7F4EF` | `#131211` | Seitengrund |
| `surface` | `#FFFCF8` | `#1C1A18` | Formularflächen, Leisten |
| `surfaceContainerLowest` | `#FFFFFF` | `#0F0E0D` | |
| `surfaceContainerLow` | `#F4F0EA` | `#211F1C` | Karten auf `background`, Blätter (Bottom Sheets) |
| `surfaceContainer` | `#EDE8E1` | `#26231F` | Navigationsleiste |
| `surfaceContainerHigh` | `#E7E1D9` | `#2D2A26` | Dialoge, Menüs |
| `surfaceContainerHighest` | `#E0D9D0` | `#34302B` | Spur der Stufenleiste, Textfeldgrund |
| `onSurface` / `onBackground` | `#1F1C18` | `#ECE6DE` | Text |
| `onSurfaceVariant` | `#5A544C` | `#BDB5AA` | Hinweise, Anker, Achsen |
| `outline` | `#7F776C` | `#958C80` | Ränder von Bedienelementen, Skalenstriche |
| `outlineVariant` | `#D6CFC5` | `#47413A` | Trennlinien (nur Schmuck) |
| `primary` | `#1E5F5A` | `#8FD0C6` | Hauptknopf, gewählter Wert, Punkte im Verlauf |
| `onPrimary` | `#FFFFFF` | `#00332E` | |
| `primaryContainer` | `#CFE6E1` | `#1A4B46` | gewählte Chips, gewählter Reiter |
| `onPrimaryContainer` | `#0A2F2B` | `#CDEDE7` | |
| `secondary` | `#5E6B67` | `#B4C3BE` | selten, sekundäre Symbole |
| `secondaryContainer` | `#E6DFD3` | `#3B352E` | Tonknöpfe („Genommen", „Mehr angeben") |
| `onSecondaryContainer` | `#2A251F` | `#EFE4D6` | |
| `tertiary` | `#8A5714` | `#E3B56C` | Grundniveau-Linie, Fokus der Marke |
| `tertiaryContainer` | `#F5E3C4` | `#4A3510` | Hinweiskarte „Plan für Tiefs" |
| `onTertiaryContainer` | `#3A2600` | `#F8DFB4` | |
| `error` | `#B3261E` | `#F2B8B5` | nur Fehler und „Löschen" |
| `onError` | `#FFFFFF` | `#601410` | |
| `errorContainer` | `#F9DEDC` | `#8C1D18` | |
| `onErrorContainer` | `#410E0B` | `#F9DEDC` | |
| `scrim` | `#000000` (32 %) | `#000000` (50 %) | |

**Gemessene Kontraste (WCAG 2.2, relative Leuchtdichte):**

| Paar | Hell | Dunkel | Anforderung |
|---|---|---|---|
| `onSurface` auf `surface` | 16,6 | 14,0 | 4,5 |
| `onSurface` auf `background` | 15,5 | 15,1 | 4,5 |
| `onSurfaceVariant` auf `surface` | 7,3 | 8,6 | 4,5 |
| `onSurfaceVariant` auf `background` | 6,8 | 9,2 | 4,5 |
| `onSurfaceVariant` auf `surfaceContainer` | 6,1 | 7,7 | 4,5 |
| `onPrimary` auf `primary` | 7,4 | 7,9 | 4,5 |
| `primary` auf `surface` (Text, Marke) | 7,2 | 9,9 | 4,5 / 3,0 |
| `onPrimaryContainer` auf `primaryContainer` | 11,0 | 7,9 | 4,5 |
| `onSecondaryContainer` auf `secondaryContainer` | 11,5 | 9,7 | 4,5 |
| `outline` auf `surface` (Ränder) | 4,3 | 5,2 | 3,0 |
| `tertiary` auf `surface` (Linie) | 5,9 | 9,2 | 3,0 |
| `error` auf `surface` | 6,4 | 10,2 | 4,5 |
| `onSurface` auf `surfaceContainerHighest` | 12,1 | 10,6 | 4,5 |
| Symbol: Sand `#F3E9D8` auf Petrol `#1E5F5A` | 6,1 | | 3,0 |

Zum Vergleich heute: `secondary` auf Grund 4,04, gewählter Chip auf Fläche 1,29, Tipp-Feld auf
Karte 1,20. Der gewählte Zustand wird künftig nie nur über Füllung gezeigt: Chips bekommen ein
Häkchen, Segmente einen kräftigen Rand plus Füllung, die Stufenleiste eine Marke in `primary`
(7,2:1).

### 3.2 Typografie

**Schrift:** Atkinson Hyperlegible Next (Braille Institute of America), Lizenz SIL Open Font
License 1.1. Gebaut für Lesbarkeit bei eingeschränktem Sehen, mit klar unterscheidbaren Formen
(0/O, 1/l/I, 6/8), was bei Erschöpfung und Konzentrationsproblemen hilft, und warmer als die
Systemschrift. Mitgeliefert werden vier statische Schnitte (400, 500, 600, 700), keine
Laufzeitladung. Bezug: https://fonts.google.com/specimen/Atkinson+Hyperlegible+Next oder
https://www.brailleinstitute.org/freefont/. Die Lizenzdatei `OFL.txt` kommt nach
`android/app/src/main/assets/lizenzen/` und ins iOS-Bundle; „Über" nennt Schrift und Lizenz.
Rückfall, falls die Prüfung der Lieferkette (AE-32) die Schrift verzögert: Systemschrift mit
derselben Skala.

Zeichenabstand überall `0.sp` (die M3-Vorgabe 0,5 sp für Fließtext entfällt, die Schrift ist
schon offen gesetzt). Uhrzeiten in Listen stehen in einer Spalte fester Breite (48 dp), damit die
Ziffern unabhängig von Tabellenziffern sauber untereinander stehen.

| Rolle | M3-Slot | Größe sp | Gewicht | Zeilenhöhe sp | iOS `relativeTo` | Einsatz |
|---|---|---|---|---|---|---|
| Wert | `displaySmall` | 34 | 600 | 40 | `.largeTitle` | Wertblase der Stufenleiste beim Ziehen |
| Seitentitel | `headlineSmall` | 26 | 600 | 32 | `.title` | „Heute", „Verlauf", Sperre |
| Abschnitt groß | `titleLarge` | 20 | 600 | 26 | `.title3` | Blatttitel, Dialogtitel |
| Frage | `titleMedium` | 18 | 500 | 24 | `.body` | jede Frage in Formularen |
| Abschnittskopf | `titleSmall` | 15 | 600 | 20 | `.subheadline` | Gruppentitel über Listen |
| Fließtext | `bodyLarge` | 17 | 400 | 24 | `.body` | Texte, Listenzeilen, Plan für Tiefs |
| Sekundär | `bodyMedium` | 15 | 400 | 22 | `.subheadline` | Zusammenfassungen |
| Hinweis | `bodySmall` | 13 | 400 | 18 | `.footnote` | Erklärungen, nie kleiner |
| Knopf | `labelLarge` | 16 | 600 | 20 | `.callout` | alle Knöpfe |
| Chip, Reiter | `labelMedium` | 14 | 500 | 18 | `.subheadline` | Chips, Segmente, Reiter |
| Anker, Achse | `labelSmall` | 12 | 500 | 16 | `.caption` | Skalenenden, Achsen; Untergrenze |

Regeln: kein `TextAutoSize`, keine festen Höhen um Text (`heightIn(min = …)` statt `height(…)`),
Android 14+ skaliert große Schrift nichtlinear, das bleibt so. Bei `fontScale >= 1.5` wechseln
Segmentreihen in senkrechte Listen (siehe 4.0).

### 3.3 Abstände, Ecken, Flächen, Bewegung

**Abstände (4-dp-Raster):** `xs 4` · `s 8` · `m 12` · `l 16` · `xl 24` · `xxl 32`.
Seitenrand 16 dp, Abstand zwischen Abschnitten 24 dp, zwischen Fragen 20 dp, Innenabstand
Karten 16 dp, Listenzeile mindestens 56 dp (zweizeilig 72 dp). Unterer Aktionsbereich 16 dp
Innenabstand plus Systemleiste.

**Touchziele:** mindestens 48 x 48 dp (iOS 44 x 44 pt), sichtbare Knöpfe 56 dp hoch für die
Hauptaktion, 48 dp für alles andere, Chips 40 dp sichtbar mit 48 dp Zielfläche
(`minimumInteractiveComponentSize`, Standard in M3).

**Ecken (`Shapes`):** `extraSmall 8` (Textfelder) · `small 12` (Chips, Segmente) · `medium 16`
(Karten, Knöpfe) · `large 24` (Blätter oben, Dialoge) · `extraLarge 28`. Knöpfe bewusst mit 16 dp
statt voller Pille: ruhiger und eigenständiger als der M3-Standard.

**Flächen:** keine Schatten. Tiefe entsteht nur über Tonstufen: Seite `background`, Karte
`surfaceContainerLow`, Formulare direkt auf `surface` ohne Karten. Blätter und Dialoge
`surfaceContainerHigh` über `scrim`. Trennlinien `outlineVariant` 1 dp, nur in Listen.

**Bewegung:**

| Token | Dauer | Kurve | Einsatz |
|---|---|---|---|
| `kurz` | 120 ms | `FastOutSlowInEasing` | Auswahl, Häkchen, Marke springt |
| `mittel` | 220 ms | `EaseInOutCubic` | Auf- und Zuklappen, Snackbar |
| `lang` | 320 ms | `EmphasizedDecelerate` (0.05, 0.7, 0.1, 1.0) | Seitenwechsel als Überblendung mit 8 dp Versatz |

Keine Federn, kein Überschwingen, keine Erfolgsanimation. Ist „Animationen entfernen" aktiv
(`Settings.Global.ANIMATOR_DURATION_SCALE == 0`, iOS `accessibilityReduceMotion`), werden alle
Übergänge zu Überblendungen von 0 ms. Haptik: leichter Takt je Stufe beim Ziehen der
Stufenleiste (`HapticFeedbackType.SegmentTick`, iOS `.selection`), ein `CONFIRM` beim Festhalten,
sonst keine. Die Systemeinstellung für Haptik gilt.

### 3.4 `Thema.kt` (zum Übertragen)

```kotlin
// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

// Ruhig, warm, ohne Farbwertung von Werten (AE-27). Rot nur für Fehler und Löschen.
private val hell = lightColorScheme(
    primary = Color(0xFF1E5F5A), onPrimary = Color(0xFFFFFFFF),
    primaryContainer = Color(0xFFCFE6E1), onPrimaryContainer = Color(0xFF0A2F2B),
    secondary = Color(0xFF5E6B67), onSecondary = Color(0xFFFFFFFF),
    secondaryContainer = Color(0xFFE6DFD3), onSecondaryContainer = Color(0xFF2A251F),
    tertiary = Color(0xFF8A5714), onTertiary = Color(0xFFFFFFFF),
    tertiaryContainer = Color(0xFFF5E3C4), onTertiaryContainer = Color(0xFF3A2600),
    error = Color(0xFFB3261E), onError = Color(0xFFFFFFFF),
    errorContainer = Color(0xFFF9DEDC), onErrorContainer = Color(0xFF410E0B),
    background = Color(0xFFF7F4EF), onBackground = Color(0xFF1F1C18),
    surface = Color(0xFFFFFCF8), onSurface = Color(0xFF1F1C18),
    surfaceVariant = Color(0xFFE0D9D0), onSurfaceVariant = Color(0xFF5A544C),
    surfaceTint = Color(0xFF1E5F5A),
    surfaceContainerLowest = Color(0xFFFFFFFF), surfaceContainerLow = Color(0xFFF4F0EA),
    surfaceContainer = Color(0xFFEDE8E1), surfaceContainerHigh = Color(0xFFE7E1D9),
    surfaceContainerHighest = Color(0xFFE0D9D0),
    outline = Color(0xFF7F776C), outlineVariant = Color(0xFFD6CFC5),
    inverseSurface = Color(0xFF34302B), inverseOnSurface = Color(0xFFF4EFE8),
    inversePrimary = Color(0xFF8FD0C6), scrim = Color(0xFF000000),
)
private val dunkel = darkColorScheme(
    primary = Color(0xFF8FD0C6), onPrimary = Color(0xFF00332E),
    primaryContainer = Color(0xFF1A4B46), onPrimaryContainer = Color(0xFFCDEDE7),
    secondary = Color(0xFFB4C3BE), onSecondary = Color(0xFF1F2B28),
    secondaryContainer = Color(0xFF3B352E), onSecondaryContainer = Color(0xFFEFE4D6),
    tertiary = Color(0xFFE3B56C), onTertiary = Color(0xFF422C00),
    tertiaryContainer = Color(0xFF4A3510), onTertiaryContainer = Color(0xFFF8DFB4),
    error = Color(0xFFF2B8B5), onError = Color(0xFF601410),
    errorContainer = Color(0xFF8C1D18), onErrorContainer = Color(0xFFF9DEDC),
    background = Color(0xFF131211), onBackground = Color(0xFFECE6DE),
    surface = Color(0xFF1C1A18), onSurface = Color(0xFFECE6DE),
    surfaceVariant = Color(0xFF34302B), onSurfaceVariant = Color(0xFFBDB5AA),
    surfaceTint = Color(0xFF8FD0C6),
    surfaceContainerLowest = Color(0xFF0F0E0D), surfaceContainerLow = Color(0xFF211F1C),
    surfaceContainer = Color(0xFF26231F), surfaceContainerHigh = Color(0xFF2D2A26),
    surfaceContainerHighest = Color(0xFF34302B),
    outline = Color(0xFF958C80), outlineVariant = Color(0xFF47413A),
    inverseSurface = Color(0xFFECE6DE), inverseOnSurface = Color(0xFF34302B),
    inversePrimary = Color(0xFF1E5F5A), scrim = Color(0xFF000000),
)

// res/font/: atkinson_next_regular.ttf, _medium, _semibold, _bold (OFL 1.1)
private val Atkinson = FontFamily(
    Font(R.font.atkinson_next_regular, FontWeight.Normal),
    Font(R.font.atkinson_next_medium, FontWeight.Medium),
    Font(R.font.atkinson_next_semibold, FontWeight.SemiBold),
    Font(R.font.atkinson_next_bold, FontWeight.Bold),
)
private fun stil(sp: Int, zeile: Int, gewicht: FontWeight) =
    TextStyle(fontFamily = Atkinson, fontSize = sp.sp, lineHeight = zeile.sp, fontWeight = gewicht, letterSpacing = 0.sp)

private val schrift = Typography(
    displaySmall = stil(34, 40, FontWeight.SemiBold),
    headlineSmall = stil(26, 32, FontWeight.SemiBold),
    titleLarge = stil(20, 26, FontWeight.SemiBold),
    titleMedium = stil(18, 24, FontWeight.Medium),
    titleSmall = stil(15, 20, FontWeight.SemiBold),
    bodyLarge = stil(17, 24, FontWeight.Normal),
    bodyMedium = stil(15, 22, FontWeight.Normal),
    bodySmall = stil(13, 18, FontWeight.Normal),
    labelLarge = stil(16, 20, FontWeight.SemiBold),
    labelMedium = stil(14, 18, FontWeight.Medium),
    labelSmall = stil(12, 16, FontWeight.Medium),
)
private val formen = Shapes(
    extraSmall = RoundedCornerShape(8.dp), small = RoundedCornerShape(12.dp),
    medium = RoundedCornerShape(16.dp), large = RoundedCornerShape(24.dp), extraLarge = RoundedCornerShape(28.dp),
)

object Abstand { val xs = 4.dp; val s = 8.dp; val m = 12.dp; val l = 16.dp; val xl = 24.dp; val xxl = 32.dp
    val rand = 16.dp; val abschnitt = 24.dp; val frage = 20.dp; val hauptknopf = 56.dp; val ziel = 48.dp }
object Bewegung { const val kurz = 120; const val mittel = 220; const val lang = 320
    val betont = CubicBezierEasing(0.05f, 0.7f, 0.1f, 1.0f) }

@Composable
fun Thema(inhalt: @Composable () -> Unit) {
    MaterialTheme(colorScheme = if (isSystemInDarkTheme()) dunkel else hell, typography = schrift, shapes = formen, content = inhalt)
}
```

Dazu in `res/values/themes.xml`: `hintergrund` auf `#FFF7F4EF`, `symbol` auf `#FF1E5F5A`, und eine
`values-night/themes.xml` mit `hintergrund #FF131211`, damit der Startbildschirm im Dunkelmodus
nicht hell aufblitzt.

### 3.5 `Gestaltung.swift` (iOS, neu unter `ios/DaysAndKeys/Oberflaeche/`)

```swift
// SPDX-License-Identifier: MPL-2.0
import SwiftUI
import UIKit

private func dyn(_ hell: UInt32, _ dunkel: UInt32) -> Color {
    func c(_ v: UInt32) -> UIColor { UIColor(red: CGFloat((v >> 16) & 0xFF) / 255, green: CGFloat((v >> 8) & 0xFF) / 255, blue: CGFloat(v & 0xFF) / 255, alpha: 1) }
    return Color(UIColor { $0.userInterfaceStyle == .dark ? c(dunkel) : c(hell) })
}

enum Farbe {
    static let grund = dyn(0xF7F4EF, 0x131211)
    static let flaeche = dyn(0xFFFCF8, 0x1C1A18)
    static let karte = dyn(0xF4F0EA, 0x211F1C)
    static let spur = dyn(0xE0D9D0, 0x34302B)
    static let text = dyn(0x1F1C18, 0xECE6DE)
    static let hinweis = dyn(0x5A544C, 0xBDB5AA)
    static let rand = dyn(0x7F776C, 0x958C80)
    static let linie = dyn(0xD6CFC5, 0x47413A)
    static let haupt = dyn(0x1E5F5A, 0x8FD0C6)
    static let aufHaupt = dyn(0xFFFFFF, 0x00332E)
    static let gewaehlt = dyn(0xCFE6E1, 0x1A4B46)
    static let aufGewaehlt = dyn(0x0A2F2B, 0xCDEDE7)
    static let ton = dyn(0xE6DFD3, 0x3B352E)
    static let ocker = dyn(0x8A5714, 0xE3B56C)
    static let fehler = dyn(0xB3261E, 0xF2B8B5)
}

enum Schrift {
    private static func a(_ gewicht: String, _ pt: CGFloat, _ stil: Font.TextStyle) -> Font { .custom("AtkinsonHyperlegibleNext-\(gewicht)", size: pt, relativeTo: stil) }
    static let wert = a("SemiBold", 34, .largeTitle)
    static let seitentitel = a("SemiBold", 26, .title)
    static let abschnittGross = a("SemiBold", 20, .title3)
    static let frage = a("Medium", 18, .body)
    static let abschnitt = a("SemiBold", 15, .subheadline)
    static let text = a("Regular", 17, .body)
    static let sekundaer = a("Regular", 15, .subheadline)
    static let hinweis = a("Regular", 13, .footnote)
    static let knopf = a("SemiBold", 16, .callout)
    static let chip = a("Medium", 14, .subheadline)
    static let anker = a("Medium", 12, .caption)
}

enum Mass { static let xs: CGFloat = 4, s: CGFloat = 8, m: CGFloat = 12, l: CGFloat = 16, xl: CGFloat = 24, xxl: CGFloat = 32
    static let ziel: CGFloat = 44, hauptknopf: CGFloat = 52, ecke: CGFloat = 16, eckeChip: CGFloat = 12 }
```

Die Schrift wird in `Info.plist` unter `UIAppFonts` eingetragen, `project.yml` nimmt die vier
TTF-Dateien als Ressourcen auf. An der Wurzel (`DaysAndKeysApp.swift`) `.tint(Farbe.haupt)` und
`.background(Farbe.grund)`; Listen `listStyle(.insetGrouped)` mit `scrollContentBackground(.hidden)`.
iOS behält die HIG-Bausteine (Navigationstitel, `List`, `Toggle`, `DatePicker`) und übernimmt nur
Farbe, Schrift, Ecken und die eigenen Komponenten aus 4.0.

---

## 4. Bildschirme

### 4.0 Gemeinsame Bausteine (neu in `Bausteine.kt`, iOS in `Gestaltung.swift`)

**Stufenleiste** (ersetzt `TippReihe` für Skalen mit mehr als fünf Stufen: Stimmung -5 bis +5,
0 bis 10, Appetit -2 bis +2).

- Aufbau: Frage (`titleMedium`), darunter eine Spur über die volle Inhaltsbreite, 48 dp hoch,
  Ecken 16 dp, Füllung `surfaceContainerHighest`, Rand 1 dp `outline`. Auf der Spur gleichmäßig
  verteilte Punkte (4 dp, `outline`); bei bipolaren Skalen ist die 0 ein Ring (8 dp). Unter der
  Spur die Anker der Vorlage an ihren Positionen (`labelSmall`, `onSurfaceVariant`) und die
  Ziffern der Enden und der Mitte.
- Ohne Antwort gibt es keine Marke, also keine Startposition (`vorlagen.md`, „Regler haben keine
  Startposition"; Verhalten aus der heutigen Tipp-Reihe).
- Gewählt: runde Marke 40 dp in `primary` an der Stufe, darin der Wert („+2", `labelLarge`,
  `onPrimary`). Die Farbe ist für jeden Wert gleich.
- Bedienung: Tippen setzt die nächste Stufe, Ziehen wandert stufenweise mit Haptik-Takt und
  zeigt über dem Finger eine Wertblase (`displaySmall`), damit der Daumen den Wert nicht verdeckt.
  Ein zweiter Tipp auf die gewählte Stufe nimmt die Antwort zurück (wie heute, Issue
  https://github.com/Blackfort-Technology/days-and-keys/issues/73).
- Maße bei 360 dp: Spur 328 dp, Stufenabstand 29,8 dp; die Zielfläche ist die ganze Spur
  (48 dp hoch), Abweichungen korrigiert das Ziehen. Auf der Startseitenkarte 304 dp (Kartenrand
  12 dp), Abstand 27,6 dp.
- Barrierefreiheit: ein einziger Knoten mit `contentDescription` = Frage plus Anker („Wie ist
  meine Stimmung? Von minus 5, sehr schlecht, bis plus 5, sehr gut"), `stateDescription` =
  „plus 2" oder „nicht beantwortet", `progressBarRangeInfo` mit `steps`, `setProgress` für
  Wischgesten, `customActions` = „Antwort entfernen". Die erste Wischgeste ohne Antwort setzt die
  Mitte der Skala und sagt sie an. iOS: `accessibilityValue`, `accessibilityAdjustableAction`,
  `accessibilityAction(named: "Antwort entfernen")`.
- Compose: eigene Komponente mit `Canvas` + `pointerInput { detectTapGestures; detectHorizontalDragGestures }`,
  `Modifier.semantics(mergeDescendants = true)`. Kein M3-`Slider`, weil der immer einen Wert hat.
  SwiftUI: `GeometryReader` + `DragGesture(minimumDistance: 0)`.

**Stufenwahl** (für Stufen mit bis zu fünf Antworten: Häufigkeit, Gelingen, Stärke, ja/nein,
„leicht, mittel, stark").

- Compose: `SingleChoiceSegmentedButtonRow` mit `SegmentedButton(icon = {})`, Höhe mindestens
  48 dp, Beschriftung `labelMedium`, gewählt `primaryContainer` plus Rand 2 dp `primary`
  (Unterscheidung über Rand, nicht nur Füllung). Erneuter Tipp hebt die Wahl auf.
- Bei 360 dp: fünf Segmente zu je 65 dp, „manchmal" passt bei 14 sp. Wird es zu eng
  (`fontScale >= 1.5` oder Beschriftung länger als das Segment), zeigt dieselbe Komponente die
  Antworten als senkrechte Liste mit `RadioButton` und 48 dp hohen Zeilen.
- „kam nicht vor" steht als eigener `FilterChip` unter der Reihe, nicht als sechstes Segment.
- SwiftUI: eigene Segmentreihe aus `Button`s (das System-`Picker(.segmented)` lässt keine
  Abwahl zu), gleiche Regeln.

**Auswahlchips** (Mehrfachauswahl: Beschwerden, Einflüsse, Marker): `FilterChip` mit
`leadingIcon` Häkchen wenn gewählt, gewählt `primaryContainer` ohne Rand, offen Rand 1 dp
`outline`. `FlowRow` mit 8 dp Abstand. Bei Beschwerden mit Stärke erscheint unter dem gewählten
Chip eine Stufenwahl „leicht | mittel | stark" mit dem Namen als Überschrift (`labelMedium`).

**Zahl** (`tag_kontakte`): Wert in der Mitte, links und rechts `FilledTonalIconButton` „minus"
und „plus" (48 dp), daneben kein Textfeld; langer Druck zählt schneller. **Dauer**: zwei Felder
„Std." und „Min." mit `suffix`, Zifferntastatur, je halbe Breite.

**Aufklappgruppe**: Kopfzeile 56 dp mit Gruppentitel (`titleSmall`), darunter grau „optional ·
4 Fragen" oder, sobald etwas beantwortet ist, eine Kurzfassung („Unruhe 3 · Müdigkeit 7"), rechts
ein Pfeil. `AnimatedVisibility` mit `Bewegung.mittel`. Semantik: `Role.Button`,
`stateDescription` „zugeklappt" oder „aufgeklappt".

**Aktionsleiste unten**: In jedem Formular fest am unteren Rand über der Tastatur
(`Modifier.imePadding()`), `surface` mit oberer Trennlinie, ein `Button` „Festhalten" über die
volle Breite, 56 dp hoch. Formulare zeigen keine Navigationsleiste.

**Zeitchip**: `AssistChip` mit Uhrsymbol, Text „Jetzt" oder „Heute, 08:15" / „Gestern, 23:40".
Öffnet den bestehenden Zeitdialog (`TimePicker` plus „heute | gestern" als Stufenwahl).

**Rückmeldung**: `Snackbar` mit Text und höchstens einer Aktion, 4 s, nie mit Lob.

### 4.1 Begrüßung und Einrichtung

**Begrüßung (Bild 01).** Ganze Seite ohne Karte.

1. Oben 48 dp Abstand, das Symbol (72 dp), darunter „Willkommen" (`headlineSmall`).
2. Drei Zeilen mit Symbol links (24 dp, `primary`), Text `bodyLarge`:
   - Schloss: „Alles bleibt verschlüsselt auf diesem Gerät."
   - durchgestrichene Wolke: „Kein Internet, kein Konto, kein Server."
   - Uhr: „Halte fest, wie es dir geht, so oft du willst."
3. Hinweisfeld `tertiaryContainer`, Ecken 16 dp, Infosymbol: „Wer den Code dieses Geräts kennt,
   kann die Einträge lesen." Gleich groß wie der übrige Text, damit niemand ihn übersieht.
4. Unten im Daumenbereich: `Button` „Einrichten", `OutlinedButton` „Aus einer Sicherung
   wiederherstellen", `TextButton` „Hilfe in einer Krise".

**Einrichtung.** Jede Stufe eine Seite mit `TopAppBar` (Zurück-Pfeil ab Schritt 2), darunter
`LinearProgressIndicator` und „Schritt 2 von 7" (`labelMedium`), Titel `headlineSmall`, Text,
Hauptknopf unten in der Aktionsleiste. Der Fortschritt dient der Orientierung, nicht als Belohnung.

| Schritt | Titel | Inhalt, Texte | Hauptknopf |
|---|---|---|---|
| 1 | „Wann beginnt dein Tag?" | Stufenwahl „0 | 2 | 3 | 4 | 5 | 6 Uhr", vorgewählt nichts außer der Kernvorgabe 4 Uhr. Text: „Wenn du nach Mitternacht noch wach bist, zählt das bis zu dieser Uhrzeit zum Vortag. Spätere Änderungen gelten nur für neue Einträge." | „Weiter" |
| 2 | „Deine 24 Recovery-Wörter" | Raster zwei Spalten, Nummer `onSurfaceVariant`, Wort `bodyLarge` 600. Bei `fontScale >= 1.5` eine Spalte. Text: „Schreib sie auf Papier. Nur mit ihnen kommst du auf einem neuen Gerät wieder an deine Einträge. Kein Foto, keine Notiz-App." | „Ich habe sie aufgeschrieben" |
| 3 | „Kurze Probe" | drei Wortfelder mit Vorschlägen; Fehler als Feldfehler (`isError`, `supportingText` „Dieses Wort passt nicht. Schau noch einmal auf deinen Zettel.") statt Snackbar | „Prüfen"; `TextButton` „Wörter noch einmal zeigen" |
| 4 | „Mit dem Gerät schützen" | „Days & Keys öffnet sich mit Fingerabdruck, Gesicht oder Geräte-Code. Der Schlüssel liegt im Sicherheitschip." | „Schützen" |
| 5 | „Automatische Sicherung" | „Wähle einen Ordner, in den die App verschlüsselte Sicherungen legt, zum Beispiel einen Cloud-Ordner. Ohne Sicherung sind die Einträge weg, wenn das Gerät verloren geht." | „Ordner wählen"; `TextButton` „Später" |
| 6 | „Erinnerungen" | „Erinnerungen haben neutrale Texte ohne Medikamentennamen. Sie ersetzen keinen Medikamentenwecker." | „Benachrichtigungen erlauben"; `TextButton` „Ohne Erinnerungen" |
| 7 | „Fertig" | Samsung-Hinweis nur auf Samsung-Geräten (`Build.MANUFACTURER`), sonst nichts. Knopf „Pünktliche Erinnerungen erlauben" nur wenn nötig. | „Zur Startseite" |

Fehler: Abbruch der Geräteauthentifizierung zeigt unter dem Knopf „Abgebrochen. Du kannst es
noch einmal versuchen." Wiederherstellung: 24 Felder in vier Gruppen zu sechs mit Kopf „Wörter 1
bis 6" usw., Hauptknopf unten fest; gefundene Sicherungen als `ListItem` (Überschrift „Sicherung
Nr. 12", Zeile „vom 14.10.2026, 21:40 · neueste"), ungültige ausgegraut mit Grund.

Fällt weg: die Karte auf leerem Grund, „Los geht's".

Fundstellen der Texte: alle in `Einrichtung.kt:97-184` (Oberfläche, gehören nach AE-22 in
`strings.xml` bzw. `Localizable.xcstrings`).

### 4.2 Heute (Bild 02, 03)

**Aufbau von oben nach unten** (`LazyColumn`, Seitenrand 16 dp):

1. `TopAppBar`: Titel „Heute", darunter klein der Wochentag mit Datum „Donnerstag, 15. Oktober"
   (`bodyMedium`, `onSurfaceVariant`). Aktionen: `TextButton` „Hilfe" mit Symbol, `IconButton`
   Schloss „Sperren".
2. **Karte „Stimmung" (Schnellstatus)**, `surfaceContainerLow`, Rand 12 dp:
   - Frage aus dem Kern: erster Eintrag der ersten Gruppe von `status_jetzt` („Wie ist meine
     Stimmung?"), keine eigene Zeichenkette in der Oberfläche.
   - Stufenleiste -5 bis +5.
   - Zeile darunter je Zustand (siehe unten).
3. **Plan für Tiefs** (nur wenn die selbst gesetzte Regel greift): Karte `tertiaryContainer`,
   Titel „Dein Plan für Tiefs", Text „Du hast eingestellt, dass die App ihn dir zeigt, wenn
   deine Tageswerte eine Weile niedrig sind.", Knöpfe „Plan ansehen" (`FilledTonalButton`) und
   „Ausblenden" (`TextButton`). Kein Warnsymbol, keine Farbe aus der Rotfamilie.
4. **Angefangen** (Entwürfe): je Entwurf ein `ListItem` „Status, angefangen um 09:12 · 4
   Antworten", rechts `TextButton` „Weiter"; „Verwerfen" im Überlaufmenü der Zeile.
5. **Medikamente heute**: Abschnittskopf, darunter je Einnahme ein `ListItem`:
   - offen: führend die Uhrzeit (48 dp Spalte), Überschrift „Testmittel A 10 mg", Zeile „1
     Stück", darunter zwei Knöpfe: `FilledTonalButton` „Genommen" und `TextButton`
     „Ausgelassen".
   - erledigt: Häkchen-Symbol in `onSurfaceVariant`, Zeile „genommen um 08:05" bzw.
     „ausgelassen". Keine Farbe für den Stand.
   - Bei Bedarf: `AssistChip`s mit dem Namen; Tipp öffnet den bestehenden Dialog „Jetzt
     eingenommen?".
   - Der Satz „Einnahmen werden nie vorausgefüllt …" steht nur, solange heute noch nichts
     bestätigt ist, und lautet „Bestätige nur, was du genommen hast."
6. **Ereignis festhalten**: Abschnittskopf, Untertitel „Ein Tipp hält es mit der Uhrzeit von
   jetzt fest. Ändern kannst du sie danach." Die ersten sechs Ereignisse in der Reihenfolge der
   Vorlage als `OutlinedButton` in einem zweispaltigen Raster (je 156 dp, 48 dp hoch), danach
   `TextButton` „Alle Ereignisse" öffnet ein `ModalBottomSheet` mit der vollen Liste als
   `ListItem`s. Die Reihenfolge bestimmt die Vorlage (`reihenfolge`), nicht die Oberfläche.
7. **Sicherung**: nur bei `sicherungWarnung` eine Karte „Die letzte Sicherung ist 4 Tage alt."
   bzw. „Noch keine Sicherung eingerichtet. Geht das Gerät verloren, sind die Einträge sonst
   weg." mit Knopf „Sicherung einrichten". Sonst eine Zeile am Seitenende „Gesichert vor 2
   Stunden" in `bodySmall`.

**Schnellstatus, ausgearbeitet**

| Zustand | Anzeige | Aktion |
|---|---|---|
| leer, heute noch nichts | Stufenleiste ohne Marke, keine Zählzeile | Tipp auf die Leiste |
| leer, heute schon festgehalten | darunter „Zuletzt um 12:02" (`bodySmall`) | |
| Wert gewählt | Marke mit Wert; darunter `Button` „Festhalten" (volle Breite, 56 dp) und `TextButton` „Mehr angeben"; in einer zweiten Zeile `TextButton` „Tagesabschluss" (wenn `tagesabschlussHeute`) | „Festhalten" speichert den Status nur mit der Stimmung; „Mehr angeben" öffnet „Status" mit der Stimmung als bereits gegebener Antwort |
| gespeichert | Snackbar „Festgehalten um 14:32", Aktion „Ansehen" (öffnet das Detail); die Karte kehrt zum leeren Zustand mit „Zuletzt um 14:32" zurück | |
| Fehler | Snackbar mit der Kernmeldung und Fehlercode (AE-21), die Auswahl bleibt stehen | erneut „Festhalten" |

Ablauf an einem schlechten Tag: Entsperren (1 bis 3 s), Stimmung tippen (1 bis 2 s),
„Festhalten" (1 s). Zusammen 4 bis 7 s, mit einer Hand, ohne Scrollen. Die Messung übernimmt
Issue https://github.com/Blackfort-Technology/days-and-keys/issues/81.

Zum Kern: Der gewählte Wert ist eine aktive Antwort der Person, keine Vorbelegung; gespeichert
wird über das vorhandene `statusSpeichern` mit einer einzigen Antwort, die übrigen gelten als
übersprungen. Wer nach der Wahl die App verlässt, bekommt über `entwurfSichern` einen Entwurf
(wie heute im Formular). Kein neuer Kernaufruf nötig.

**Ereignisse:** Ereignisse ohne Fragen (in der Vorlage `gruppen: []`, etwa „Schlafen gegangen",
„Wirkung lässt nach") werden mit dem Tipp sofort gespeichert; Snackbar „Schlafen gegangen um
23:10 festgehalten", Aktion „Ändern" öffnet die Korrektur. „Krise" behält die Rückfrage als
`AlertDialog`: Titel „Krise festhalten?", Text aus `FormularSeite.kt:114`, Knöpfe „Festhalten und
Hilfe zeigen" und „Abbrechen". Ereignisse mit Fragen öffnen ein `ModalBottomSheet` mit Titel,
Zeitchip, den Fragen (Bausteine aus 4.0) und „Festhalten" unten.

**Leer** (keine Medikamente angelegt): der Abschnitt Medikamente entfällt ganz. **Voll:** mehr
als vier Einnahmen am Tag werden nach Uhrzeit gelistet, erledigte rücken nicht nach unten
(Ordnung bleibt vorhersehbar).

Fällt weg: „Days & Keys" als Titel, „15.10.2026", die Zählzeile „Heute 1 Status", der Knopf
„Status jetzt festhalten" (ersetzt durch die Stufenleiste), 14 gleich gewichtete Ereignisknöpfe,
die Sicherungskarte im Normalfall.

### 4.3 Status (Bild 04, 05, 06)

Ganze Seite ohne Navigationsleiste.

- `TopAppBar`: `navigationIcon` Schließen (X), Titel „Status", rechts der Zeitchip und ein
  Überlaufmenü mit „Entwurf verwerfen". Schließen behält den Entwurf und zeigt auf der
  Startseite „Als Entwurf behalten".
- Inhalt direkt auf `surface`, ohne Karten:
  1. Abschnittskopf „Wie geht es mir" mit Zusatz „höher = besser" in `onSurfaceVariant`.
     Stimmung, Antrieb, Konzentration als Stufenleisten, immer offen.
  2. Aufklappgruppe „Was ist gerade stark · höher = mehr" (Unruhe, Anspannung, Gereiztheit,
     Müdigkeit), zugeklappt.
  3. Aufklappgruppe „Körper" (Appetit als Stufenleiste -2 bis +2, Wirkung als Stufenwahl, nur
     wenn der Kern sie zeigt; Beschwerden als Auswahlchips mit Stärke), zugeklappt.
  4. Aufklappgruppe „Seit 10:15 Uhr" bzw. „In den letzten zwölf Stunden" (Text aus dem Kern,
     `ansicht.rs:133-148`), nur wenn der Kern `blockAnbieten` meldet; Untertitel „optional · 10
     Fragen". Öffnen setzt wie heute `blockOffen`. Häufigkeiten als Stufenwahl, Beschwerden und
     Einflüsse als Chips, Notiz als Textfeld.
  5. Zeile „Notiz hinzufügen" (Stift-Symbol), Tipp öffnet ein Textfeld `minLines = 3`.
  6. Fußnote `bodySmall`: „Was du auslässt, bleibt leer. Nichts wird aus früheren Einträgen
     übernommen."
- Aktionsleiste unten: „Festhalten".

Welche Gruppen anfangs zugeklappt sind, bestimmt die Vorlage. Vorschlag: neues Anzeigefeld
`"aufklappbar": true` an den Gruppen 2 und 3 in `standard-v1.json` `formulare[0]` (und an
„Faktoren" und „Marker" im Tagesabschluss), vom Kern unverändert als `GruppeAnsicht.aufklappbar`
weitergereicht (`ansicht.rs:124-129`). Es ändert keine Messdefinition und keine Bedeutung, also
keinen Schlüssel. Abwägung: Zugeklappte Gruppen werden seltener beantwortet; ob das für
Unruhe und Müdigkeit hinnehmbar ist, zeigt der Nutzungsversuch. Bis dahin bleibt der Vorschlag
umkehrbar, weil nur die Vorlage betroffen ist. Achtung: Phase A friert M1a ein, das Feld muss
vorher in der Vorlage stehen.

Zustände: Entwurf geladen (Antworten gesetzt, Gruppen mit Antworten aufgeklappt); Korrektur
(Titel „Status korrigieren", Knopf „Korrektur festhalten", Zeitchip zeigt die ursprüngliche
Zeit); Fehler beim Speichern (Snackbar, Seite bleibt).

Fällt weg: Karten je Gruppe, „Zugeklappt. Öffnen, wenn du etwas dazu sagen willst." samt Knopf
„Öffnen", „Verwerfen" als gleichrangiger Knopf neben „Speichern", „jetzt, Zeit ändern".

### 4.4 Tagesabschluss (Bild 07)

Gleiches Muster wie 4.3, Titel „Tagesabschluss".

1. „Stimmung heute Abend": Stimmung (Stufenleiste), „Wie war der Tag insgesamt?"
   (Stufenleiste), „War ich gegen Abend gereizt oder erschöpft?" (Stufenwahl), immer offen.
2. Aufklappgruppe „Faktoren · jede Angabe lässt sich überspringen": Schlafdauer (Dauer),
   Schlafqualität (Stufenleiste), Bewegung (Stufenwahl ja/nein, darunter Dauer erst nach „ja"),
   Gespräche (Zahl), Tageslicht (ja/nein), Handyzeit (Dauer).
3. Aufklappgruppe „Marker": „Was war heute?" als Auswahlchips (eigene Marker mit ihren Namen),
   „Was war heute besonders?" als Textfeld.

Die Abhängigkeit „Dauer nur nach ja" ist eine Inhaltsregel und gehört als Bedingung in die
Vorlage (`formulare[1].bedingungen`, analog zu `wirkung_jetzt`), nicht in die Oberfläche. Bis sie
dort steht, bleibt die Dauer sichtbar. Wochenrückblick: gleiche Bauart, eine offene Gruppe.

### 4.5 Ereignis

Siehe 4.2. Das Blatt (`ModalBottomSheet`, `sheetState` mit `skipPartiallyExpanded = false`)
zeigt: Titel (Name aus der Vorlage), Zeitchip, bei Einnahme-Ereignissen „Präparat" als
Stufenwahl bzw. Chips (heute `FilterChip`, `FormularSeite.kt:116-122`), die Fragen, „Festhalten".
Leer: „Noch kein Präparat angelegt. Du kannst es unter Medikamente anlegen." mit `TextButton`
„Zu den Medikamenten". iOS: `.sheet` mit `presentationDetents([.medium, .large])`.

### 4.6 Medikamente (Bild 12, 13)

**Liste.** `TopAppBar` „Medikamente". Je Präparat ein `ListItem`: Überschrift „Testmittel A
10 mg", Zeile „08:00 · 20:00" bzw. „bei Bedarf" bzw. „pausiert", rechts Pfeil. Tipp öffnet die
Bearbeitung. `ExtendedFloatingActionButton` „Präparat hinzufügen" unten rechts.
Leer: Tablettensymbol in `onSurfaceVariant`, „Noch keine Medikamente eingetragen. Du kannst die
App auch ganz ohne nutzen." und der Knopf. Der Erklärtext „Ein Dosis- oder Präparatwechsel ist
zusätzlich ein Ereignis …" wandert in die Bearbeitung neben „Wechsel festhalten".

**Bearbeiten.** Ganze Seite ohne Navigationsleiste, `TopAppBar` mit Schließen (X), Titel
„Präparat bearbeiten" bzw. „Neues Präparat", Aktion `TextButton` „Speichern" (und zusätzlich
die Aktionsleiste unten, damit es mit dem Daumen erreichbar bleibt).

| Abschnitt | Inhalt |
|---|---|
| Präparat | `OutlinedTextField` „Name", „Wirkstoff (optional)", in einer Zeile „Stärke" und „Form" |
| Einnahme | Stufenwahl „Nach Plan | Bei Bedarf". Bei „Nach Plan": je Uhrzeit eine Zeile „08:00 · Dosis 1" mit Tipp auf die Zeit (öffnet `TimePicker`-Dialog, kein Tippen von „08:00"), Dosisfeld rechts, Entfernen als `IconButton` Papierkorb mit `contentDescription` „Uhrzeit 08:00 entfernen". Darunter `TextButton` „Uhrzeit hinzufügen" |
| Erinnerung | `Switch` „An die Einnahme erinnern", Zeile darunter „Die Benachrichtigung nennt kein Medikament." Feld „Status-Erinnerung danach" mit `suffix` „Min.", Hinweis „leer lassen für keine" |
| Im Status | `Switch` „Nach der Wirkung fragen", Zeile „Der Status fragt dann, ob du die Wirkung gerade spürst." |
| Stand | `Switch` „Nehme ich derzeit" (ersetzt „Aktiv") |
| Wechsel | `OutlinedButton` „Dosis- oder Präparatwechsel festhalten" mit dem Erklärsatz, nur bei vorhandenem Präparat |

Fehler: Pflichtfeld Name leer → Feldfehler „Bitte einen Namen eingeben."; Speicherfehler als
Snackbar. Verlassen mit Änderungen fragt: „Änderungen verwerfen?" mit „Verwerfen" und
„Weiter bearbeiten".

Fundstellen: alle Beschriftungen in `Medikamente.kt:39-102`.

### 4.7 Verlauf (Bild 08, 09)

Reiter „Verlauf" mit `SingleChoiceSegmentedButtonRow` oben: „Diagramm | Einträge".

**Diagramm.**

1. Fragenwahl als volle Zeile (`ListItem` mit Pfeil): „Wie ist meine Stimmung?". Tipp öffnet ein
   `ModalBottomSheet` mit den Fragen gruppiert nach Bezug („Jetzt", „Seit dem letzten Status",
   „Tagesabschluss"), Stimmung oben. Kurzname, sobald die Vorlage ihn liefert (siehe 4.8).
2. Zeitraum als Stufenwahl „4 Wochen | 3 Monate | 1 Jahr".
3. Diagramm (Höhe 240 dp, bei Querformat 60 % der Höhe): Punkte `primary` 3 dp Radius,
   Grundniveau `tertiary` 2,5 dp, Nulllinie `outline` 1,5 dp, Hilfslinien `outlineVariant`.
   Achsen `labelSmall` `onSurfaceVariant`. Lücken bleiben Lücken. Keine Flächen, keine
   Farbverläufe.
4. Unter dem Diagramm die Legende in einer Zeile und eine Stufenwahl „Grundniveau 7 Tage | 28
   Tage".
5. Faktenzeile als Text `bodyMedium`: „An 13 von 28 Tagen gibt es einen Wert · 19 Einnahmen".
6. Aufklappgruppe „So liest du das Diagramm" mit dem Methodentext (Tageswert, Mindesttage).
7. Aufklappgruppe „Tageswerte als Liste": Datum, Tageswert, Zahl der Einträge; dient
   Screenreadern als Alternative zum Diagramm und allen, die Zahlen lieber lesen.
8. Abschnitt „Grundniveau mit und ohne", Hinweis aus dem Kern, je Faktor eine Zeile „Bewegung:
   an 9 Tagen +0,8, an 4 Tagen ohne -0,3".

Zum Kern: Der heutige Hinweis (`ansicht.rs:468`) wird in zwei Felder geteilt: `hinweis` kurz
(„An {} von {} Tagen gibt es einen Wert.") und `erklaerung` lang. Der Text „Rein beschreibend …"
kommt aus `ansicht.rs:517` statt aus `Verlauf.kt:81`, und welche Faktoren verglichen werden,
liefert der Kern (heute fest in `Verlauf.kt:82`). Beides ist Pflicht aus AE-06.

Zustände: keine Werte im Zeitraum → statt des Diagramms „In diesem Zeitraum gibt es noch keine
Werte für diese Frage." (`bodyLarge`, mittig, ohne Bild); Fehler → Textzeile mit Fehlercode und
„Erneut laden".

Fällt weg: Abschnitt „Verlauf" als Karte, Klappmenü mit 20 Fragen, „▾" im Knopftext, gemischte
Chipreihe.

### 4.8 Einträge und Detail (Bild 10, 11)

**Liste.** `LazyColumn` mit `stickyHeader` je Tag: „Heute", „Gestern", sonst „Mittwoch, 14.
Oktober" (`titleSmall`, Grund `background`). Je Eintrag ein `ListItem`:
- führend die Uhrzeit (Spalte 48 dp, `labelLarge`) und darunter ein kleines Art-Symbol in
  `onSurfaceVariant` (Status: Kreis, Tagesabschluss: Mond, Einnahme: Tablette, Ereignis:
  Markierung), damit die Art auch ohne Lesen erkennbar ist;
- Überschrift: Formularname („Status", „Tagesabschluss", „Einnahme", „Bewegung");
- Zeile: Kurzfassung, höchstens zwei Zeilen: „Stimmung +1 · Antrieb 6 · Konzentration 5 · 3
  weitere".
Unten `TextButton` „Ältere Einträge laden" (erweitert den Zeitraum um 60 Tage, der Kernaufruf
`eintraege(von, bis)` gibt das her; heute fest 60 Tage, `Eintraege.kt:35`).

Leer: „Noch keine Einträge. Was du festhältst, erscheint hier." Fehler: Zeile mit Fehlercode und
„Erneut laden".

**Detail.** Eigene Seite statt `AlertDialog`: `TopAppBar` Zurück, Titel „Status", Untertitel
„Donnerstag, 15. Oktober, 08:15". Inhalt: je Antwort eine Zeile mit Frage (`bodyMedium`,
`onSurfaceVariant`) und Antwort darunter (`bodyLarge`), Freitexte im Wortlaut. Aktionsleiste
unten: `FilledTonalButton` „Korrigieren". „Löschen" im Überlaufmenü oben; Bestätigung als
`AlertDialog`: Titel „Eintrag löschen?", Text „Er verschwindet sofort aus der App. In älteren
Sicherungen bleibt er höchstens 30 Tage erhalten.", Knöpfe „Löschen" (Text in `error`) und
„Behalten". „Ohne Angaben" bleibt für leere Einträge.

Zum Kern und zur Vorlage:
- neues Anzeigefeld `"kurz"` je Messdefinition in `standard-v1.json` (`"Stimmung"`, `"Antrieb"`,
  `"Konzentration"`, `"Unruhe"`, `"Anspannung"`, `"Gereiztheit"`, `"Müdigkeit"`, `"Appetit"`,
  `"Wirkung"`, `"Beschwerden"`, `"Notiz"` usw.); `zusammenfassung` (`ansicht.rs:560-567`) nutzt
  es, die Detailansicht weiter die volle Frage. Es ändert keine Bedeutung, also keinen Schlüssel.
- `formulare[0].name` „Status jetzt" → „Status"; `formulare[3].name` „Tablette genommen" →
  „Einnahme" (passt auch zu Tropfen und Spray). **Vorher** `Eintraege.kt:73` umbauen, das den
  Formularschlüssel über den Anzeigenamen „Tablette genommen" sucht; der Kern soll den Schlüssel
  in `EintragZeile` mitliefern.
- Optional für die Zeilenform: `alleAntworten` als Paare (Frage, Antwort) statt fertiger
  Zeichenketten. Ohne diese Änderung zeigt das Detail die heutigen Zeilen „Frage: Antwort".

### 4.9 Mehr (Einstellungen, Bild 14)

Eine Übersichtsliste, jede Zeile führt auf eine Unterseite. Änderungen gelten sofort (Aufruf
`einstellungenSpeichern` nach jeder Änderung, 600 ms entprellt wie bei den Entwürfen) und werden
mit einer kurzen Snackbar „Gespeichert" bestätigt. Es gibt keinen Speicherknopf mehr.

| Gruppe | Zeile | Unterseite |
|---|---|---|
| Für schwere Momente | Mein Plan für Tiefs | Textfeld groß (`minLines = 8`), oben Hinweisfeld `tertiaryContainer`: „Dieser Text ist über die Hilfe auch gesperrt lesbar und liegt dafür unverschlüsselt auf dem Gerät. In die Sicherung kommt er verschlüsselt mit." (Wortlaut seit Issue https://github.com/Blackfort-Technology/days-and-keys/issues/131, `Einstellungen.kt:109`) Speichern beim Verlassen und nach Tipp-Pause |
| | Plan automatisch zeigen | Stufenwahl „aus | 3 | 5 | 7 Tage", darunter (nur wenn an) „unter -3 | -2 | -1 | 0". Text: „Selbst eingestellt, keine Diagnose." |
| | Vertrauenspersonen | Liste mit Name und Telefon, Hinzufügen als Zeile, Bearbeiten im Blatt. „Erscheinen in der Hilfe, aber nur entsperrt. Die App benachrichtigt niemanden." |
| Alltag | Erinnerungen | Schalter je Erinnerung, Uhrzeit als Zeile mit `TimePicker` (statt fester Chips 18 bis 22 Uhr); Status-Erinnerungen als Liste von Uhrzeiten mit „Uhrzeit hinzufügen"; oben „Neutrale Texte, höchstens 8 am Tag." |
| | Tag und Sperre | „Der Tag beginnt um" (Stufenwahl), „Sperren nach" (Stufenwahl „30 s | 1 min | 5 min") |
| | Eigene Tagesmarker | drei Textfelder |
| Daten | Sicherung | Stand („Letzte Sicherung vor 2 Stunden", Ordner gewählt ja/nein), „Jetzt sichern", „Ordner ändern" |
| | Recovery-Wörter prüfen | 24 Felder in Gruppen zu sechs, Aktionsleiste „Prüfen" |
| | Schlüssel erneuern | Erklärung, Knopf; Bestätigung über die Geräteauthentifizierung (wie heute) |
| | Alles exportieren | Erklärung „Alle Einträge als Tabelle (CSV) im Klartext …", Knopf |
| Über | Days & Keys | Version, Lizenz MPL-2.0, Quelltext https://github.com/Blackfort-Technology/days-and-keys, Schrift Atkinson Hyperlegible Next (OFL 1.1), „Keine Diagnosen, keine Empfehlungen …", Schlüsselspeicher |

Compose: `ListItem` mit `headlineContent`, `supportingContent` (aktueller Wert, etwa „3 Tage
unter -2"), `trailingContent` Pfeil; Gruppenköpfe `titleSmall`. Navigation in die Unterseiten mit
Zurück-Pfeil, Navigationsleiste bleibt sichtbar (es sind keine Formulare im Sinn von 4.0).
iOS: `List` `.insetGrouped` mit `Section` und `NavigationLink`.

Fundstellen: `Einstellungen.kt:74-172`.

### 4.10 Sperre (Bild 15)

- Mitte oben (40 % Höhe): Symbol 72 dp, darunter „Days & Keys" (`headlineSmall`) und „Gesperrt"
  (`bodyMedium`, `onSurfaceVariant`).
- Unteres Drittel, im Daumenbereich: `Button` „Entsperren" (56 dp, volle Breite),
  `OutlinedButton` „Hilfe in einer Krise" (48 dp, gleiche Breite), darunter „Zum Entsperren
  genügt Fingerabdruck, Gesicht oder der Geräte-Code." (`bodySmall`, jetzt 6,8:1).
- Die Geräteauthentifizierung startet von selbst, wenn die App aus einer Benachrichtigung oder
  aus dem Hintergrund kommt (heute nur mit `ziel`, `Sperre.kt:23`).
- Abbruch: Zeile über dem Knopf „Abgebrochen. Tippe auf Entsperren, um es noch einmal zu
  versuchen."
- iOS: Text „Zum Entsperren genügt Face ID oder der Geräte-Code." bleibt.

### 4.11 Hilfe (Bild 16, auch gesperrt)

Reihenfolge nach dem Muster eines persönlichen Krisenplans: sofortige Gefahr, eigene Schritte,
vertraute Menschen, Beratung.

1. `TopAppBar` „Hilfe" mit Zurück-Pfeil (gesperrt: zurück zur Sperre, vor der Einrichtung zur
   Begrüßung). Kein „Hilfe"-Knopf auf dieser Seite, kein „Sperren".
2. Zeile „Notruf 112 · bei akuter Gefahr" als einziger `Button` in `primary`, 56 dp, volle
   Breite, Telefonsymbol.
3. „Mein Plan für Tiefs" (wenn vorhanden): Karte `tertiaryContainer`, Text `bodyLarge` mit
   großzügiger Zeilenhöhe. Ohne Plan, entsperrt: „Unter Mehr › Mein Plan für Tiefs kannst du
   eigene Schritte aufschreiben. Sie stehen dann hier, auch ohne Entsperren."
4. „Menschen, denen ich vertraue": je Person ein `ListItem` mit Name, Telefon und Wählsymbol.
   Gesperrt: „Deine Vertrauenspersonen siehst du nach dem Entsperren."
5. „Rund um die Uhr, kostenfrei, anonym": eine Gruppe „TelefonSeelsorge" mit drei Zeilen
   (0800 111 0 111, 0800 111 0 222, 116 123), jede Zeile ein `ListItem` mit der Nummer groß
   (`titleMedium`) und Wählsymbol rechts. Aufeinanderfolgende Einträge mit gleichem Namen fasst
   die Oberfläche zusammen; Inhalt und Reihenfolge kommen aus dem Kern (`ansicht.rs:775-780`).
6. „Für Kinder und Jugendliche": Nummer gegen Kummer 116 111. „Wenn die Praxis geschlossen ist":
   Ärztlicher Bereitschaftsdienst 116 117. Die Zusatzzeile ist jeweils die Überschrift der
   Gruppe, damit kein Hinweis mehr zwischen zwei Knöpfen hängt.
7. Am Ende `bodyMedium`: „Die App benachrichtigt niemanden. Die Nummern öffnen nur das Wählfeld,
   angerufen wird erst, wenn du es dort bestätigst."

Der Kernhinweis (`ansicht.rs:787`) wird geteilt: der Teil „Wenn es dir schlecht geht, ruf eine
der Nummern an oder sprich mit einer Person, der du vertraust." steht als Einleitung unter dem
Titel, „Die App benachrichtigt niemanden." am Ende. Jede Zeile hat `Role.Button` und die
Beschreibung „TelefonSeelsorge, 0800 111 0 111, Wählfeld öffnen".

Fällt weg: sieben gleich schwere dunkle Knöpfe, die Karte mit „Zurück".

---

## 5. Navigation und Informationsarchitektur

**Reiter (4 statt 5):**

| Reiter | Android-Symbol (Material Symbols Rounded) | iOS (SF Symbols) | Inhalt |
|---|---|---|---|
| Heute | `wb_twilight` | `sun.horizon` | Schnellstatus, Plan-Hinweis, Entwürfe, Medikamente heute, Ereignisse, Sicherung |
| Verlauf | `timeline` | `chart.dots.scatter` | Diagramm und Einträge (Umschalter oben), Detail |
| Medikamente | `medication` | `pills` | Liste, Bearbeitung |
| Mehr | `more_horiz` | `ellipsis.circle` | Einstellungen nach 4.9 |

Compose: `NavigationBar` mit `NavigationBarItem` (Symbol plus Text, `labelMedium`, gewählt
`primaryContainer`). Ab `fontScale >= 1.5` `alwaysShowLabel = false`, dann zeigt nur der gewählte
Reiter seinen Namen; jeder Reiter behält `contentDescription`. Die Symbole werden als
Vektor-Ressourcen mitgeliefert (Material Symbols, Apache 2.0), nicht über
`material-icons-extended` und nie aus dem Netz. iOS: `TabView` mit denselben vier Reitern,
`.tint(Farbe.haupt)`.

**Ebenen:**

```
Sperre ──► Heute ─┬─► Status (ganze Seite) ─► Zeitdialog
   │              ├─► Tagesabschluss / Wochenrückblick (ganze Seite)
   │              ├─► Ereignis (Blatt) / Alle Ereignisse (Blatt)
   │              └─► Hilfe
   │       Verlauf ─┬─ Diagramm ─► Fragenwahl (Blatt)
   │                └─ Einträge ─► Detail ─► Korrigieren (Formular)
   │       Medikamente ─► Präparat (ganze Seite)
   │       Mehr ─► Unterseiten
   └─► Hilfe (gesperrt, ohne Vertrauenspersonen)
Begrüßung ─► Einrichtung (7 Schritte) / Wiederherstellung ─► Heute
```

**Regeln:**
- Hilfe ist von jeder Seite mit Kopfleiste über die Aktion „Hilfe" erreichbar, dazu auf Sperre
  und Begrüßung als eigener Knopf.
- Formulare (Status, Tagesabschluss, Wochenrückblick, Präparat, Korrektur) sind ganze Seiten
  ohne Navigationsleiste mit X links; Zurück behält Entwürfe.
- Unterseiten (Detail, Mehr-Unterseiten) haben einen Zurück-Pfeil, die Leiste bleibt.
- Die Systemzurückgeste folgt derselben Ordnung (heute in `Oberflaeche.kt:53-63` von Hand,
  künftig mit `navigation-compose` und typisierten Zielen; `Seite` aus `Zustand.kt:22-34`
  bleibt als Zielbeschreibung).
- Benachrichtigungen führen nach dem Entsperren direkt ins Ziel (heute `z.ziel`).
- Kopfleisten zeigen den Seitentitel, den App-Namen nur auf der Sperre und der Begrüßung.

---

## 6. App-Symbol

### 6.1 Konzept

**„Tagesanbruch":** eine warme Scheibe, durch einen Horizont geteilt, darüber eine ockerfarbene
halbe Sonne. Es steht für einen neuen Tag und den Neuanfang nach einem schweren Tag, ohne
Pathos. Das Symbol ist absichtlich unverdächtig: kein Herz, kein Gehirn, keine Pille, kein
Schloss, also nichts, was auf dem Startbildschirm eine Erkrankung verrät. Die Flächen sind groß
und haben keine dünnen Linien, damit es auch mit 32 px erkennbar bleibt.

Farben: Grund Petrol `#1E5F5A`, Scheibe Sand `#F3E9D8`, Sonne Ocker `#E3B56C`. Kontrast Sand auf
Petrol 6,1:1, Ocker auf Petrol 3,9:1.

### 6.2 Erzeugung und Auswahl

Erzeugt mit `gpt-image-2.5-flare` (gegen `/v1/models` geprüft, Snapshot
`gpt-image-2.5-flare-2026-09-08`), 1024 x 1024, Qualität `high`, auf einer Werkstatt-VM; Schlüssel
nur für den Lauf in einer Datei mit Rechten 600, danach gelöscht. Skripte:
`design/symbol/werkzeug/`.

| Datei | Motiv | Urteil |
|---|---|---|
| `symbol/symbol-a-tagesbogen.png` | Sandscheibe mit ausgeschnittenem Bogen, Linie, Ockerpunkt | klein gut lesbar, wirkt aber wie eine Servierglocke |
| `symbol/symbol-b-schluessel.png` | Schlüssel, im Griff eine kleine Sonne, dünner Ring | greift „Keys" auf, dünne Linien und Bart zerfallen unter 48 px, Schlüssel wirkt nach Sicherheits-App |
| `symbol/symbol-c-punktreihe.png` | fünf Punkte auf einer Welle im dünnen Ring | ruhig, liest sich aber als Diagramm (Bewertungsnähe) und ist klein kaum zu sehen |
| `symbol/symbol-a2-horizont-1.png`, `-2.png` | Weiterentwicklung von A: Horizont über die ganze Scheibe, Sonne in Ocker | **gewählt: `-1`**, ruhig, eindeutig ein Tagesbeginn, klein stabil |

Aus `symbol-a2-horizont-1.png` wurde die Geometrie gemessen (Scheibe 53 % der Kante, Sonne 48 %
des Scheibenradius, Horizont knapp unter der Mitte) und als Vektor neu aufgebaut, mit etwas
kleinerer Scheibe (50 %) für den Sicherheitsrand und breiterem Horizont (2,5 dp statt 1,8 dp),
damit er bei 32 px nicht verschwindet. `symbol/vorschau.png` zeigt das Ergebnis in drei Masken,
vier Größen, auf hellem und dunklem Grund, die Themenvariante und alle Varianten im Vergleich.

### 6.3 Lieferung

| Datei | Zweck |
|---|---|
| `symbol/vordergrund-freigestellt.png` | Vordergrund, 1024 x 1024, transparent, entspricht der 108-dp-Ebene (Motiv 54 dp) |
| **Hintergrundfarbe** | **`#1E5F5A`** (`@color/symbol`) |
| `symbol/android/ic_vordergrund.xml` | Vordergrund als Vector Drawable (ersetzt `res/drawable/ic_vordergrund.xml`) |
| `symbol/android/ic_monochrom.xml` | Monochrome Ebene für Themensymbole ab Android 13, die Sonne ist durch einen Spalt von der Scheibe getrennt |
| `symbol/android/ic_launcher.xml` | `adaptive-icon` mit `background`, `foreground`, `monochrome` (ersetzt `res/mipmap-anydpi-v26/ic_launcher.xml`) |
| `symbol/android/play-store-512.png` | Grafik für Store-Einträge (F-Droid, Play) |
| `symbol/monochrom.png` | Monochrome Ebene als PNG (weiß, transparent) |
| `symbol/symbol-gewaehlt-1024.png` | Vordergrund auf Petrol, Gesamtansicht |
| `symbol/ios/AppIcon-1024.png` | iOS-Standard, ohne Alpha, Motiv auf 64 % vergrößert (iOS hat keinen Sicherheitsrand, das System rundet) |
| `symbol/ios/AppIcon-dunkel-1024.png` | iOS-Variante „Dunkel": transparenter Grund, das System legt den dunklen Grund darunter |
| `symbol/ios/AppIcon-getoent-1024.png` | iOS-Variante „Getönt": Graustufen auf Schwarz |

**Android Adaptive Icon:** Die Ebenen sind 108 x 108 dp, sichtbar sind 72 dp, die Maske kann bis
auf einen Kreis von 66 dp schneiden. Die Scheibe hat 54 dp Durchmesser und liegt damit ganz in
der sicheren Zone; die Bewegung der Launcher (Parallaxe bis 18 dp) schneidet nie ins Motiv. Weil
`minSdk = 33`, reicht `mipmap-anydpi-v26/ic_launcher.xml` ohne PNG-Mipmaps. In
`res/values/themes.xml` `symbol` auf `#FF1E5F5A` setzen. Pfade aus `ic_vordergrund.xml`:

```
Scheibe oben  M27.001,54.25A27,27 0,1 1,80.999,54.25Z      (#F3E9D8)
Scheibe unten M80.86,56.75A27,27 0,0 1,27.14,56.75Z        (#F3E9D8)
Sonne         M41,54.25A13,13 0,0 1,67,54.25Z              (#E3B56C)
```

**iOS:** In `ios/DaysAndKeys/Assets.xcassets/AppIcon.appiconset/` die drei 1024er-Dateien als
Einzelgröße mit den Varianten „Any", „Dark", „Tinted" eintragen (`Contents.json` mit
`"platform": "ios", "size": "1024x1024"` und `appearances` `luminosity: dark` bzw. `tinted`), in
`project.yml` den Katalog aufnehmen und `ASSETCATALOG_COMPILER_APPICON_NAME: AppIcon` setzen.
Eine Fassung im Icon Composer (Ebenen Grund, Scheibe, Sonne) braucht einen Mac und kann später
aus denselben drei Flächen entstehen; der Cloud-Bau (AE-30) kommt mit dem Katalog aus.

Erneut bauen: `python3 design/symbol/werkzeug/rendern.py` (Pillow, ohne Netz).

---

## 7. Umsetzungsreihenfolge

Jeder Schritt ist ein eigener PR mit Issue, einzeln prüfbar, in dieser Reihenfolge. Aufwand in
Personentagen (PT) für eine Person mit Kenntnis des Codes; Android zuerst, iOS je Schritt in
einem zweiten PR, außer wo vermerkt.

| Nr. | Schritt | Prüfung | Aufwand |
|---|---|---|---|
| 1 | Tokens: `Thema.kt` und `Gestaltung.swift` nach 3.4/3.5, Schrift einbinden, `values-night`, Lizenzdatei | Unit-Test rechnet die Kontrastpaare aus 3.1 nach (Tabelle als Testdaten, Schwelle 4,5 bzw. 3,0); Bildschirmfotos hell und dunkel | 1,0 |
| 2 | App-Symbol nach 6.3 (beide Plattformen) | Launcher auf Emulator mit Kreis- und Squircle-Maske, Themensymbol an; iOS-Simulator hell, dunkel, getönt | 0,5 |
| 3 | Texte der Oberfläche nach `strings.xml` bzw. `Localizable.xcstrings` (AE-22), ohne Wortlautänderung | Lint `HardcodedText` ohne Befund, Oberfläche unverändert | 1,0 |
| 4 | Navigation: vier Reiter mit Symbolen, Seitentitel, Formulare ohne Leiste, Zurück-Regeln, `navigation-compose` | TalkBack-Reihenfolge je Seite, Systemzurück auf allen Wegen aus 5, 200 % Schrift bei 360 dp | 1,5 |
| 5 | Bausteine: Stufenleiste, Stufenwahl, Auswahlchips mit Häkchen, Zahl, Aufklappgruppe, Aktionsleiste | Compose-UI-Tests: `assertTouchHeightIsAtLeast(48.dp)`, Semantik (`stateDescription`, `setProgress`, Aktion „Antwort entfernen"), kein Wert ohne Tipp; Bildschirmfoto-Tests 100 % und 200 % | 2,0 |
| 6 | Status, Tagesabschluss, Wochenrückblick auf die Bausteine; Vorlage `aufklappbar`, Kern reicht es durch | Kerntest für das neue Feld; Entwurf und Korrektur unverändert (vorhandene Tests grün); Zeitmessung nach Issue https://github.com/Blackfort-Technology/days-and-keys/issues/81 | 1,5 |
| 7 | Heute mit Schnellstatus, Ereignisse sofort bzw. im Blatt, Medikamente als Liste | Schnellstatus mit einer Antwort gespeichert und im Verlauf sichtbar; Entwurf nach Verlassen; Zeit von Entsperren bis Festgehalten unter 15 s an fünf Durchläufen | 2,0 |
| 8 | Einträge: Vorlage `kurz`, Kern-Zusammenfassung, Schlüssel in `EintragZeile`, Liste mit Tagesköpfen, Detailseite; danach Umbenennung „Status", „Einnahme" | Kerntests für Zusammenfassung; Korrektur einer Einnahme nach der Umbenennung funktioniert (Negativprobe: ohne den Umbau von `Eintraege.kt:73` schlägt sie fehl) | 1,5 |
| 9 | Verlauf: Umschalter, Fragenwahl im Blatt, Stufenwahlen, Hinweis geteilt im Kern, Faktoren aus dem Kern, Tageswerte als Liste | Kerntest für `hinweis`/`erklaerung`; keine Zeichenkette aus `ansicht.rs` mehr in `Verlauf.kt` (grep) | 1,5 |
| 10 | Medikamente: Liste, Bearbeitung mit Zeitdialog, Rückfrage bei ungespeicherten Änderungen | Plan mit zwei Uhrzeiten anlegen, ändern, entfernen; Erinnerung kommt mit neutralem Text | 1,0 |
| 11 | Mehr: Übersicht, Unterseiten, sofortiges Speichern | jede Einstellung nach Neustart erhalten; Plan für Tiefs gesperrt in der Hilfe lesbar | 1,5 |
| 12 | Sperre, Hilfe, Begrüßung und Einrichtung | Hilfe gesperrt und vor der Einrichtung erreichbar und verlassbar (Issue https://github.com/Blackfort-Technology/days-and-keys/issues/121 bleibt behoben); TalkBack liest Nummern als Ziffern | 1,5 |
| 13 | iOS-Nachzug der Schritte 4 bis 12 (über die macOS-Runner, AE-30) | Snapshot-Tests je Seite hell, dunkel, größte Dynamic-Type-Stufe; VoiceOver auf Stufenleiste und Hilfe | 4,0 |
| 14 | Abnahme mit 360 dp, 200 % Schrift, TalkBack und VoiceOver, Dunkelmodus, „Animationen entfernen" | Prüfliste je Bildschirm aus Abschnitt 4, neue Bildschirmfotos `01` bis `16` | 1,0 |
| | **Summe** | | **etwa 21,5 PT** |

Abhängigkeiten: 1 vor allem; 5 vor 6, 7, 9, 10; 8 braucht Kern und Vorlage zuerst. Die
Vorlagenfelder `aufklappbar` und `kurz` müssen vor Phase A des Nutzungsversuchs stehen, weil M1a
dann eingefroren ist (AE-37).

---

## Anhang: Textfundstellen

| Ort | Heute | Neu | Fundstelle |
|---|---|---|---|
| Heute, Titel | „Days & Keys" | „Heute" + Wochentag mit Datum | `Oberflaeche.kt:68`; Datum `ansicht.rs:275` (`zeit::datum_text`, neue Langform) |
| Heute, Statuskarte | „Wie geht es dir gerade?" | Frage aus der Vorlage („Wie ist meine Stimmung?") | `Start.kt:40`; `standard-v1.json` `messdefs[stimmung].frage` |
| Heute, Zählzeile | „Heute 1 Status, zuletzt vor 3 Stunden" / „Heute noch kein Status …" | „Zuletzt um 12:02", bei null nichts | `Start.kt:41-43` |
| Heute, Hauptknopf | „Status jetzt festhalten" | „Festhalten" / „Mehr angeben" | `Start.kt:45` |
| Heute, Medikamente | „Einnahmen werden nie vorausgefüllt. Bestätige nur, was du genommen hast." | „Bestätige nur, was du genommen hast." (nur bis zur ersten Bestätigung) | `Start.kt:72` |
| Heute, Ereignisse | „Ein Tipp, die Uhrzeit ist jetzt und lässt sich ändern." | „Ein Tipp hält es mit der Uhrzeit von jetzt fest. Ändern kannst du sie danach." | `Start.kt:100` |
| Heute, Sicherung | „Letzte Sicherung vor … Bitte prüfen." | „Die letzte Sicherung ist 4 Tage alt." | `Start.kt:105-108` |
| Formular, Zeit | „jetzt, Zeit ändern" | Zeitchip „Jetzt" / „Heute, 08:15" | `FormularSeite.kt:110` |
| Formular, Block | „Zugeklappt. Öffnen, wenn du etwas dazu sagen willst." / „Öffnen" | „optional · 10 Fragen" in der Kopfzeile | `FormularSeite.kt:127-128` |
| Formular, Knöpfe | „Speichern" / „Verwerfen" | „Festhalten" / Menü „Entwurf verwerfen" | `FormularSeite.kt:136-140` |
| Formular, Fußnote | „Unbeantwortete Fragen gelten als übersprungen. Nichts wird aus früheren Einträgen übernommen." | „Was du auslässt, bleibt leer. Nichts wird aus früheren Einträgen übernommen." | `FormularSeite.kt:143` |
| Formular, Meldung | „Festgehalten" | „Festgehalten um 14:32" | `FormularSeite.kt:101` |
| Formularnamen | „Status jetzt", „Tablette genommen" | „Status", „Einnahme" | `standard-v1.json` `formulare[0].name`, `formulare[3].name` (vorher `Eintraege.kt:73`) |
| Einträge, Zusammenfassung | „Wie ist meine Stimmung: +1 · …" | „Stimmung +1 · Antrieb 6 · …" | `ansicht.rs:560-567`, neues Feld `messdefs[].kurz` |
| Einträge, Titel | „Einträge der letzten 60 Tage" | Reiter „Verlauf › Einträge", Fuß „Ältere Einträge laden" | `Eintraege.kt:38` |
| Detail, Löschen | „Wirklich löschen? Aus dem Bestand ist der Eintrag sofort weg; …" | „Eintrag löschen?" / „Er verschwindet sofort aus der App. In älteren Sicherungen bleibt er höchstens 30 Tage erhalten." | `Eintraege.kt:62` |
| Verlauf, Hinweis | ein Absatz | `hinweis` kurz + `erklaerung` lang | `ansicht.rs:468` |
| Verlauf, Faktoren | eigener Text in der Oberfläche | Text aus dem Kern | `Verlauf.kt:81` → `ansicht.rs:517` |
| Hilfe, Hinweis | ein Satz für alles | Einleitung + Schlusssatz | `ansicht.rs:787` |
| Präparat, Schalter | „Aktiv", „Wirkung im Tagesverlauf spürbar (dann fragt der Status danach)", „Erinnern (neutraler Text ohne Namen)" | „Nehme ich derzeit", „Nach der Wirkung fragen" + Zeile, „An die Einnahme erinnern" + Zeile | `Medikamente.kt:78-81` |
| Präparat, Feld | „Status-Erinnerung nach Einnahme (Minuten, optional)" | „Status-Erinnerung danach", Suffix „Min." | `Medikamente.kt:95` |
| Einrichtung | Texte nach 4.1 | | `Einrichtung.kt:97-184` |
| Sperre | unverändert | | `Sperre.kt:29-36` |
