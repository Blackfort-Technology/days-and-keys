# Google Play - Gesundheits-Apps, Altersfreigabe, Zielgruppe (Entwurf)

Stand 2026-10-17 · Entwurf zu https://github.com/Blackfort-Technology/days-and-keys/issues/160 ·
Grundlage: Stand `main` vom 2026-10-17, Android-App Version 0.1.

Antworten für drei Formulare unter „App-Inhalte“ in der Play Console: Erklärung zu
Gesundheits-Apps (Health apps declaration), Einstufung des Inhalts (IARC-Fragebogen) und
Zielgruppe. Antworten, die eine rechtliche oder geschäftliche Entscheidung brauchen, sind mit
**ENTSCHEIDUNG (En)** gekennzeichnet und in Abschnitt 4 gesammelt.

Die Bezeichnungen der Console stammen aus Googles Hilfeseiten (abgerufen 2026-10-17); den
IARC-Fragebogen selbst zeigt Google erst in der Console. Vor dem Ausfüllen dort abgleichen.

## 1. Erklärung zu Gesundheits-Apps

Quellen: https://support.google.com/googleplay/android-developer/answer/14738291 und
https://support.google.com/googleplay/android-developer/answer/12261419.

### 1.1 Hat die App Gesundheitsfunktionen?

**Ja.** Die App hält Befinden, körperliche Beschwerden, Medikamente mit Einnahmeplan und
Einnahmen, Schlaf und Arzttermine fest (`templates/standard-v1.json`) und erinnert an Einnahmen
(`android/app/src/main/java/de/blackforttec/daysandkeys/plattform/Planer.kt`, Zeilen 21 bis 24).
„My app doesn't provide any health features“ wäre falsch.

### 1.2 Welche Kategorien?

Vorschlag, **ENTSCHEIDUNG (E1)**:

| Kategorie in der Console | Ankreuzen | Begründung |
|---|---|---|
| Health and fitness: Sleep Management | ja | Schlaf am Morgen, Dauer und Güte (`standard-v1.json`, Formular „Aufgewacht“) |
| Health and fitness: Stress Management, Relaxation, Mental Acuity | nein | die App bietet keine Übungen und keine Entspannung an |
| Health and fitness: Activity and Fitness | nein | Bewegung ist nur ein Ereignis zum Festhalten, kein Training und keine Messung |
| Medical: Medication and Treatment Management | ja | Präparate, Einnahmeplan, Bestätigung der Einnahme, Erinnerung (`strings.xml`, Zeilen 196 bis 217) |
| Medical: Mental and Behavioral Health | ja | Tagebuch über Stimmung, Antrieb, Konzentration und Unruhe (`standard-v1.json`, `messdefs`) |
| Medical: Emergency and First Aid | nein | die Hilfe-Seite zeigt nur Nummern und öffnet das Wählfeld (`ui/Hilfe.kt`, Zeile 40); sie leistet keine Erste Hilfe |
| Medical: Clinical Decision Support, Diseases and Conditions Management | nein | keine Auswertung, keine Empfehlung, keine Diagnose |
| Medical Device Apps | nein, siehe 1.3 | |

Abwägung: Die beiden Kategorien unter „Medical“ beschreiben die App ehrlich. Wer sie weglässt,
macht die App unauffälliger, riskiert aber eine falsche Erklärung. Ankreuzen heißt nicht, dass
die App ein Medizinprodukt ist; das fragt Google getrennt (1.3).

### 1.3 Ist die App ein reguliertes Medizinprodukt?

Vorschlag: **Nein**, „Medical Device Apps“ nicht ankreuzen. **ENTSCHEIDUNG (E2).**

Begründung: Die App speichert und zeigt an, was die Person einträgt. Sie stellt keine Diagnose,
gibt keine Therapie- oder Dosisempfehlung und wertet Werte nicht automatisch medizinisch aus
(`docs/gesamtkonzept.md`, Abschnitt 2, Leitplanke 4; `strings.xml`, Zeile 351). Software, die
Daten nur speichert, anzeigt und überträgt, ist nach MDCG 2019-11 in der Regel kein
Medizinprodukt (https://github.com/Blackfort-Technology/days-and-keys/issues/24). Die
Zweckbestimmung „Tagebuch, keine automatische Bewertung“ ist aber noch nicht schriftlich
festgelegt, und die regulatorische Prüfung vor einer Veröffentlichung steht aus (#24, #77). Bis
dahin ist „Nein“ eine Annahme, keine geprüfte Aussage.

Wer „Medical Device Apps“ ankreuzt, muss für den EWR nach MDR unter anderem Hersteller, SRN,
Zweckbestimmung, elektronische Gebrauchsanweisung und UDI-DI angeben und auf Anfrage die
Konformität belegen (Hilfeseite 14738291).

### 1.4 Pflichten für Gesundheits-Apps ohne Zulassung

- **Hinweis in der Beschreibung**, dass die App kein Medizinprodukt ist und nichts diagnostiziert,
  behandelt, heilt oder verhindert, und der Rat, sich an Fachleute zu wenden: steht im Abschnitt
  „Wichtiger Hinweis“ der ausführlichen Beschreibung (`eintrag.md`). In der App steht unter Mehr ›
  Über „Keine Diagnosen, keine Empfehlungen, keine automatischen Zusammenhänge. Die App ersetzt
  keine ärztliche Beratung.“ (`strings.xml`, Zeile 351). Der Satz „kein Medizinprodukt“ hängt an
  E2.
- **Datenschutzerklärung**, die den Umgang mit Gesundheitsdaten beschreibt:
  `datenschutzerklaerung.md`.

### 1.5 Zweckbestimmung und die Stellen, an denen die App rechnet

Die Zweckbestimmung „Tagebuch, keine automatische Bewertung“ verlangt, dass die App nichts
deutet. An drei Stellen rechnet sie aus den Werten; alle drei sind beschreibend und von der
Person ausgelöst oder eingestellt. **ENTSCHEIDUNG (E3)**, ob das so in die Zweckbestimmung passt:

| Funktion | Was sie tut | Beleg |
|---|---|---|
| Grundniveau | Mittelwert der Tageswerte über 7 und 28 Tage, nur bei genügend Tagen; Lücken bleiben Lücken | `core/crates/dak-core/src/ansicht.rs`, Zeilen 481 bis 489 |
| Grundniveau mit und ohne | Mittelwert der Stimmung an Tagen mit und ohne eine Ja/Nein-Angabe, jeweils mit Fallzahl, Hinweis „Rein beschreibend … Kein Beleg für eine Ursache.“ | `ansicht.rs`, Zeilen 503 bis 547 |
| Plan automatisch zeigen | zeigt den eigenen Plan für Tiefs, wenn die Stimmung an einer selbst gewählten Zahl von Tagen unter einer selbst gewählten Schwelle liegt; Standard aus; Text „Selbst eingestellt, keine Diagnose.“ | `strings.xml`, Zeilen 115 und 327; `ansicht.rs`, Zeilen 778 und 779 |

Es gibt keine Summenwerte über Fragen, keine Grenzwerte, keine Hinweise wie „auffällig“ und
keine Empfehlungen (Forderung aus #77). Die Nähe einzelner Fragen zu standardisierten
Fragebögen ist in #77 und #22 offen.

## 2. Einstufung des Inhalts (IARC-Fragebogen)

Quelle für die Stellen, die Google veröffentlicht:
https://support.google.com/googleplay/android-developer/answer/9859655. Die Fragen selbst stehen
erst in der Console; die Liste unten folgt dem üblichen Aufbau und ist dort abzugleichen.

| Frage | Antwort | Begründung, Beleg |
|---|---|---|
| Kategorie der App | Dienstprogramm, Produktivität, Kommunikation oder Sonstiges | Tagebuch; kein Spiel, kein soziales Netzwerk, kein Nachschlagewerk |
| Gewalt, Blut, Angst | Nein | keine Darstellungen; die App zeigt nur Fragen, Skalen und Verläufe |
| Sexuelle Inhalte, Nacktheit | Nein | |
| Vulgäre Sprache | Nein | alle Texte stammen aus `strings.xml` und `standard-v1.json`; Freitexte schreibt nur die Person selbst und nur für sich |
| Drogen, Alkohol, Tabak | Nein | Version 0.1 hat keine Erfassung von Substanzen; die Vorlage ist „Ereignisse ohne Substanzen“ (`ROADMAP.md`, M1a). Medikamente trägt die Person selbst ein; die App nennt keine Präparate. Mit der Substanzvorlage (https://github.com/Blackfort-Technology/days-and-keys/issues/26) neu beantworten |
| Glücksspiel, simuliertes Glücksspiel | Nein | |
| Verweise auf Suizid oder Selbstverletzung (falls gefragt) | **ENTSCHEIDUNG (E6)** | die App hat ein Ereignis „Krise“ (`standard-v1.json`) und eine Hilfe-Seite mit Krisennummern (`ansicht.rs`, Zeilen 850 bis 855). Sie zeigt nichts davon bildlich und erwähnt Suizid nicht ausdrücklich. Vorschlag: Nein, mit Blick auf die genaue Frage in der Console |
| Können Nutzer miteinander kommunizieren oder Inhalte teilen? | Nein | keine Netzberechtigung (`AndroidManifest.xml`, Zeilen 3 und 6 bis 10), keine Freigaben in M1 |
| Teilt die App den Standort mit anderen? | Nein | keine Standortberechtigung im Manifest |
| Digitale Käufe | Nein | keine Käufe, keine Abrechnungsbibliothek (`android/app/build.gradle.kts`, Zeilen 61 bis 71) |
| Uneingeschränkter Internetzugang (Browser) | Nein | keine WebView, kein Netz (`CLAUDE.md`, AE-04) |

Erwartete Einstufung: niedrigste Stufe (etwa USK ab 0, PEGI 3). Die Altersfreigabe sagt nur, ob
der Inhalt für Kinder ungeeignet ist; die Zielgruppe ab 18 legt Abschnitt 3 fest.

## 3. Zielgruppe und Inhalte

Quelle: https://support.google.com/googleplay/android-developer/answer/9867159.

**Zielgruppe: nur „18 und älter“.** **ENTSCHEIDUNG (E4).**

Begründung:

1. Die App verarbeitet Gesundheitsdaten nach Art. 9 DSGVO, darunter Befinden, Medikation und
   Krisen. Ob und wie Minderjährige in die Verarbeitung solcher Daten wirksam selbst einwilligen
   können und welche Rolle Eltern hätten, ist nicht geklärt (#77: „Minderjährige“).
2. Für Minderjährige fehlt ein Konzept; die Gefahr, dass Eltern die App zur Kontrolle nutzen, ist
   als Risiko benannt (https://github.com/Blackfort-Technology/days-and-keys/issues/76,
   `docs/gesamtkonzept.md`, Abschnitt 14, letzte Zeile).
3. Der Nutzungsversuch ist mit Erwachsenen geplant (#82).
4. Mit einer Zielgruppe ab 18 gilt die Richtlinie für Familien nicht.

Zusätzlich vorschlagen: **„Zugriff für Minderjährige einschränken“ (Restrict Minor Access)
einschalten.** Dann können Konten unter 18 die App nicht suchen und nicht installieren. Das
passt zur Begründung, schließt aber auch Jugendliche aus, die die App selbst suchen; Teil von E4.

Weitere Fragen im Formular:

- Spricht die App Kinder an (Gestaltung, Figuren)? **Nein.** Ruhige Oberfläche ohne Figuren
  (`docs/design/designvorgabe.md`, Abschnitt 2).
- Werbung: **Die App enthält keine Werbung.** Keine Werbebibliothek, kein Netz.

## 4. Entscheidungen von Christian

| Nr. | Frage | Vorschlag | Art |
|---|---|---|---|
| E1 | Welche Kategorien in der Erklärung zu Gesundheits-Apps? | Sleep Management, Medication and Treatment Management, Mental and Behavioral Health | rechtlich, Außenwirkung |
| E2 | „Medical Device Apps“ und der Satz „kein Medizinprodukt“ in der Beschreibung | Nein bzw. Satz bleibt, erst nach schriftlicher Zweckbestimmung und MDR-Prüfung (#24, #77) | rechtlich |
| E3 | Passen Grundniveau, „mit und ohne“ und „Plan automatisch zeigen“ zur Zweckbestimmung? | ja, beschreibend und selbst eingestellt; in der Zweckbestimmung ausdrücklich nennen | rechtlich (MDR) |
| E4 | Zielgruppe nur ab 18, Zugriff für Minderjährige einschränken | ja und ja | rechtlich (DSGVO, Minderjährige) |
| E5 | Data safety: Sicherung in Cloud-Ordner (O1) und CSV-Export (O2) als „nicht erhoben“ | Nein in Frage 1 (`datensicherheit.md`) | rechtlich, Auslegung Google |
| E6 | IARC: Krise und Krisennummern als Verweis auf Suizid oder Selbstverletzung? | Nein, Wortlaut der Frage abwarten | Auslegung |
| E7 | Öffentliche Kontakt-E-Mail im Eintrag (`<KONTAKT>`) | eigene Adresse für die App unter blackfort-tec.de statt der allgemeinen info@blackfort-tec.de; die Adresse wird öffentlich angezeigt | Konto, Kontaktadresse |
| E8 | Ort der Datenschutzerklärung | Seite auf https://www.blackfort-tec.de (`datenschutzerklaerung.md`, Abschnitt „Veröffentlichung“) | Ort, rechtlich |
| E9 | Play-Konto | Organisationskonto auf die Blackfort Technology UG (haftungsbeschränkt) mit der D-U-N-S-Nummer der UG; Dienstkonto für die Pipeline (#159). Es gibt noch kein Konto | Konto |
| E10 | Datenschutzerklärung rechtlich prüfen lassen | vor der Veröffentlichung, zusammen mit der Rollenanalyse aus #77 (Verantwortlicher, Anbieter, Hersteller) | rechtlich |
