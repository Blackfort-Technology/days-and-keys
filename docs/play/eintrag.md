# Google Play - Store-Eintrag (Entwurf)

Stand 2026-10-17 · Entwurf zu https://github.com/Blackfort-Technology/days-and-keys/issues/160 ·
Anbieter: Blackfort Technology UG (haftungsbeschränkt), Ernst-Robert-Curtius-Straße 8A, 53117 Bonn.

Die Texte beschreiben nur, was Version 0.1 auf Android kann (Stand `main` vom 2026-10-17). Sie
müssen zur Zweckbestimmung „Tagebuch, keine automatische Bewertung“ passen, die mit der
rechtlichen Einordnung (https://github.com/Blackfort-Technology/days-and-keys/issues/77) und der
Abgrenzung Medizinprodukt (https://github.com/Blackfort-Technology/days-and-keys/issues/24)
festgelegt wird. Deshalb gilt für alle Texte:

- keine Krankheitsnamen, keine Zielgruppe nach Diagnose, keine Aussage über Wirkung oder Besserung;
- die App hält fest und zeigt an; sie bewertet, deutet und empfiehlt nichts;
- Funktionen aus späteren Stufen (Arzt-PDF, Formulareditor, Substanzen, Freigaben) kommen nicht vor;
- der Hinweis, den Google für Gesundheits-Apps ohne Zulassung verlangt („not a medical device and
  does not diagnose, treat, cure, or prevent any medical condition“ und der Rat, sich an
  Fachleute zu wenden, https://support.google.com/googleplay/android-developer/answer/12261419),
  steht auf Deutsch in der Beschreibung.

Gezählt wurde mit Python `len()` über den Text zwischen den Codezäunen, also in Unicode-Zeichen
einschließlich Leerzeichen und Zeilenumbrüchen. Grenzen nach
https://support.google.com/googleplay/android-developer/answer/9866151.

## App-Name (höchstens 30 Zeichen)

```
Days & Keys: Tagebuch
```

21 Zeichen. Alternative ohne Zusatz: `Days & Keys` (11 Zeichen). Der Zusatz hilft bei der Suche;
„Tagebuch“ ist bewusst allgemein gehalten, damit der Name auf dem Startbildschirm und in der
Liste der Apps nichts über Gesundheit verrät (Designvorgabe Abschnitt 2, Prinzip 5). Auf dem Gerät
heißt die App weiter „Days & Keys“ (`android/app/src/main/AndroidManifest.xml`, Zeile 18).

## Kurzbeschreibung (höchstens 80 Zeichen)

```
Halte fest, wie es dir geht. Verschlüsselt auf dem Gerät, ohne Konto und Netz.
```

78 Zeichen.

## Ausführliche Beschreibung (höchstens 4000 Zeichen)

```
Days & Keys ist ein persönliches Tagebuch für dein Befinden. Du hältst fest, wie es dir gerade geht, so oft du willst, auch mehrmals am Tag. Die App speichert, was du einträgst, und zeigt es dir im Verlauf. Sie bewertet nichts, deutet nichts und gibt keine Empfehlungen.

FESTHALTEN
• Stimmung mit einem Tipp direkt auf der Startseite
• Status ausführlich: Antrieb, Konzentration, innere Unruhe, Anspannung, Gereiztheit, Müdigkeit, Appetit, körperliche Beschwerden und eine Notiz. Was du auslässt, bleibt leer; nichts wird aus früheren Einträgen übernommen.
• Seit dem letzten Status: Fragen zu den Stunden dazwischen, zugeklappt und freiwillig
• Ereignisse mit einem Tipp, zum Beispiel Einnahme, Mahlzeit, Bewegung, schlafen gegangen, aufgewacht, Termin, Belastung, Arzttermin, Therapiesitzung oder Dosiswechsel. Die Uhrzeit lässt sich danach ändern.
• Schlaf am Morgen: wie lange und wie gut
• Tagesabschluss und Wochenrückblick, auf Wunsch mit einer Notiz für das nächste Arztgespräch
• Entwürfe sichern sich selbst; angefangene Einträge gehen nicht verloren.

MEDIKAMENTE
Trag deine Präparate mit Stärke und Einnahmeplan ein und bestätige, was du genommen oder ausgelassen hast. Gezählt wird nur, was du bestätigst. Die App kommt auch ganz ohne Medikamente aus.

VERLAUF
Jeder Eintrag ist ein Punkt mit Uhrzeit. Der Verlauf zeigt die einzelnen Werte über vier Wochen, drei Monate oder ein Jahr, dazu als Linie den Mittelwert der letzten sieben Tage. Tage ohne Eintrag bleiben leer. Einnahmen stehen auf derselben Zeitachse. Es gibt keine Zähler und keine Serien, die zum Weitermachen drängen. Jeder Eintrag lässt sich korrigieren oder löschen.

ERINNERUNGEN
Auf Wunsch erinnert die App zu selbst gewählten Uhrzeiten an einen Status oder an eine Einnahme. Die Benachrichtigungen haben neutrale Texte und nennen kein Medikament. Sie ersetzen keinen Medikamentenwecker.

HILFE, AUCH GESPERRT
Die Hilfe-Seite ist immer erreichbar, auch ohne Entsperren: Notruf 112, TelefonSeelsorge, Nummer gegen Kummer, ärztlicher Bereitschaftsdienst 116 117 und dein eigener Plan für Tiefs. Nach dem Entsperren stehen dort auch deine Vertrauenspersonen. Die Nummern öffnen nur das Wählfeld. Die App benachrichtigt niemanden.

DEINE DATEN BLEIBEN BEI DIR
• Alle Einträge liegen verschlüsselt auf deinem Gerät.
• Die App hat keine Berechtigung für das Internet. Es gibt kein Konto, keinen Server, keine Werbung und keine Analyse deines Verhaltens.
• Geöffnet wird die App mit Fingerabdruck, Gesicht oder dem Geräte-Code. Der Schlüssel liegt im geschützten Schlüsselspeicher des Geräts.
• Sicherung: Du wählst einmal einen Ordner, auch einen Cloud-Ordner. Die App legt dort automatisch verschlüsselte Sicherungen ab. Mit deinen 24 Recovery-Wörtern stellst du sie auf einem neuen Gerät wieder her, ohne Server.
• Export: Auf deinen Wunsch und nach Bestätigung erstellt die App eine Tabelle aller Einträge im Klartext (CSV).

Ehrlich zu den Grenzen: Wer den Code deines Geräts kennt, kann die Einträge lesen. Dein Plan für Tiefs liegt unverschlüsselt auf dem Gerät, damit er in der Hilfe auch gesperrt lesbar ist. Ohne Gerät und ohne Recovery-Wörter sind die Einträge verloren; so ist es gewollt.

WICHTIGER HINWEIS
Days & Keys ist ein Tagebuch und kein Medizinprodukt. Die App stellt keine Diagnosen und dient nicht dazu, eine Erkrankung zu erkennen, zu behandeln, zu heilen oder ihr vorzubeugen. Sie ersetzt keine ärztliche oder psychotherapeutische Beratung oder Behandlung. Für Rat, Diagnose oder Behandlung wende dich an eine Ärztin, einen Arzt oder eine Psychotherapeutin. In akuter Gefahr ruf 112 an.

Die App richtet sich an Erwachsene und läuft ab Android 13.

OFFENER QUELLTEXT
Days & Keys ist Open Source (MPL-2.0). Den Quelltext findest du unter https://github.com/Blackfort-Technology/days-and-keys

Anbieter: Blackfort Technology UG (haftungsbeschränkt), Bonn
```

3853 Zeichen.

Belege für die Aussagen der Beschreibung (alle Pfade relativ zum Repo):

| Aussage | Beleg |
|---|---|
| Stimmung mit einem Tipp, Status, Felder | `templates/standard-v1.json` (`messdefs`, `formulare[0]`), Bildschirmfoto 03 und 05 |
| Was du auslässt, bleibt leer | `android/app/src/main/res/values/strings.xml`, Zeile 231 |
| Ereignisse | `templates/standard-v1.json` `formulare` (Einnahme bis Dosis- oder Präparatwechsel); „Krise“ steht in der Vorlage, wird in der Beschreibung aber bewusst nicht aufgezählt |
| Uhrzeit danach änderbar | `strings.xml`, Zeile 135 |
| Entwürfe | `core/crates/dak-core/src/kern.rs`, Zeile 935 (`entwurf_sichern`) |
| Gezählt wird nur, was du bestätigst | `strings.xml`, Zeile 126; `docs/gesamtkonzept.md` Abschnitt 5.1 |
| Ohne Medikamente nutzbar | `strings.xml`, Zeile 192 |
| Verlauf, Grundniveau 7 Tage, Zeiträume | `strings.xml`, Zeilen 164 bis 168; `core/crates/dak-core/src/ansicht.rs`, Zeilen 481 und 489 (Mittelwert nur bei mindestens 4 von 7 Tagen) |
| Korrigieren, löschen | `strings.xml`, Zeilen 153 und 154; `android/app/src/main/java/de/blackforttec/daysandkeys/ui/Eintraege.kt`, Zeilen 101 bis 107 |
| Neutrale Erinnerungen, kein Medikamentenwecker | `strings.xml`, Zeilen 208 und 266; `android/app/src/main/java/de/blackforttec/daysandkeys/plattform/Planer.kt`, Zeilen 22 bis 24 |
| Hilfe ohne Entsperren, Nummern, Wählfeld | `android/app/src/main/java/de/blackforttec/daysandkeys/ui/Hilfe.kt`, Zeilen 35 und 40; `strings.xml`, Zeile 99 |
| Keine INTERNET-Berechtigung | `android/app/src/main/AndroidManifest.xml`, Zeilen 3 und 6 bis 10; `.github/workflows/ci.yml`, Zeilen 157 bis 161 |
| Schlüssel im Schlüsselspeicher, Entsperren | `android/app/src/main/java/de/blackforttec/daysandkeys/plattform/Schluesselspeicher.kt`, Zeilen 22 bis 26 und 43 bis 51 |
| Sicherung, Recovery-Wörter | `android/app/src/main/java/de/blackforttec/daysandkeys/plattform/Sicherungsordner.kt`, Zeilen 12 bis 15; `strings.xml`, Zeilen 252 bis 263 |
| Export im Klartext nach Bestätigung | `android/app/src/main/java/de/blackforttec/daysandkeys/ui/Einstellungen.kt`, Zeilen 224 bis 227; `strings.xml`, Zeile 347 |
| Grenzen | `strings.xml`, Zeilen 290 und 325 |
| Android ab 13 | `android/app/build.gradle.kts`, Zeile 13 (`minSdk = 33`) |
| Lizenz, Quelltext | `strings.xml`, Zeilen 349 und 350 |

Bewusst nicht in der Beschreibung: „Plan automatisch zeigen“ (eine selbst eingestellte Regel über
niedrige Tageswerte, `strings.xml`, Zeile 327) und der Vergleich „Grundniveau mit und ohne“
(Mittelwert der Stimmung an Tagen mit und ohne eine Ja/Nein-Angabe,
`core/crates/dak-core/src/ansicht.rs`, Zeilen 503 bis 547). Beides rechnet aus Werten; ob es mit
der Zweckbestimmung vereinbar ist, steht in `gesundheit.md` unter E3.

## Kategorie

**Gesundheit & Fitness** (englisch „Health and Fitness“).

Begründung: Google beschreibt „Medical“ als „Drug and clinical references, calculators, handbooks
for healthcare providers, medical journals and news“
(https://support.google.com/googleplay/android-developer/answer/9859673). Das passt nicht zu
einem persönlichen Tagebuch und rückt die App in die Nähe eines Werkzeugs für die Behandlung,
was der Zweckbestimmung widerspricht. „Lifestyle“ wäre unauffälliger, beschreibt aber Stilratgeber
und Planung und würde Nutzerinnen, die ein Befindenstagebuch suchen, in die Irre führen. Unter
„Gesundheit & Fitness“ suchen die Menschen, für die die App gedacht ist, und die Kategorie sagt
nichts über medizinische Zwecke.

Tags (falls die Console sie anbietet): Tagebuch, Stimmung, Gewohnheiten. Keine Tags mit
Krankheitsbezug.

## Kontakt

- E-Mail: `<KONTAKT>` (Pflichtfeld, öffentlich sichtbar; Entscheidung bei Christian, siehe
  `gesundheit.md`, E7)
- Website: https://www.blackfort-tec.de (optional)
- Telefon: leer lassen (optional)
- Datenschutzerklärung: öffentliche URL, siehe `datenschutzerklaerung.md`

## Bildschirmfotos (Telefon, 2 bis 8)

Quelle: `16_todos/days-and-keys-bilder/` in der Knowledge Base (nicht im Repo), Version 0.1.15,
alle Werte erfunden. Die App sperrt Bildschirmfotos (`FLAG_SECURE`,
`android/app/src/main/java/de/blackforttec/daysandkeys/MainActivity.kt`, Zeile 22); die Bilder
stammen aus dem Emulatorfenster.

Empfehlung, acht Bilder in dieser Reihenfolge:

| Nr. | Datei | Satz darunter |
|---|---|---|
| 1 | `03-schnellstatus.png` | Ein Tipp hält fest, wie es dir gerade geht. |
| 2 | `05-status.png` | Mehr angeben, wenn du willst. Was du auslässt, bleibt leer. |
| 3 | `04-heute-medikamente.png` | Einnahmen bestätigen und Ereignisse mit einem Tipp festhalten. |
| 4 | `08-schlaf.png` | Schlaf am Morgen, in ein paar Sekunden. |
| 5 | `09-verlauf.png` | Der Verlauf zeigt deine Werte. Tage ohne Eintrag bleiben leer. |
| 6 | `11-eintraege.png` | Alle Einträge nach Uhrzeit, korrigierbar und löschbar. |
| 7 | `17-hilfe.png` | Hilfe mit Nummern und deinem Plan, auch gesperrt. |
| 8 | `01-willkommen.png` | Verschlüsselt auf dem Gerät. Kein Internet, kein Konto, kein Server. |

Warum diese: Bild 1 bis 4 zeigen das Festhalten, vom kürzesten Weg zum ausführlichen; Bild 5 und
6 zeigen, was man davon hat, ohne Deutung; Bild 7 und 8 zeigen Hilfe und Datenschutz. Nicht
gewählt: `02-heute` (fast gleich wie 03), `06-status-koerper` und `07-tagesabschluss` (gleiche
Bauart wie 05), `10-fragenwahl`, `12-eintrag`, `13-medikamente`, `14-praeparat`, `15-mehr`
(Bedienung, kein Nutzen auf einen Blick), `16-gesperrt` (wenig Inhalt).

Prüfen vor dem Hochladen:

- Die Bilder haben 688 × 1371 Pixel, 24-Bit-PNG ohne Alphakanal, Seitenverhältnis 1,99. Das reicht
  für den Eintrag (mindestens 320 px, höchstens 3840 px, längere Seite höchstens doppelt so lang).
  Für Empfehlungsflächen verlangt Google mindestens vier Bilder mit mindestens 1080 px
  (https://support.google.com/googleplay/android-developer/answer/9866151). Vor der
  Veröffentlichung sollten die acht Bilder deshalb mit 1080 × 2160 Pixeln oder mehr neu
  aufgenommen werden.
- Die Sätze stehen als Bildunterschrift in diesem Dokument; ob sie ins Bild gesetzt werden oder
  nur als Alternativtext dienen, ist eine Gestaltungsfrage.
- In Bild 1 und 3 steht „Testmittel A 10 mg“, in Bild 7 ein erfundener Plan. Es dürfen keine
  echten Präparatnamen oder Personen zu sehen sein.
- Vorgeschrieben sind außerdem ein Symbol (512 × 512 px, 32-Bit-PNG) und eine Vorstellungsgrafik
  (1024 × 500 px). Beide fehlen noch; das Symbol entsteht nach Designvorgabe Abschnitt 6.
