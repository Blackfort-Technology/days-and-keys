# Datenschutzerklärung für die App Days & Keys

> **ENTWURF - NICHT VERÖFFENTLICHEN.** Dieser Text ist ein Entwurf vom 2026-10-17 zu
> https://github.com/Blackfort-Technology/days-and-keys/issues/160. Er muss vor jeder
> Veröffentlichung rechtlich geprüft werden, zusammen mit der Rollenanalyse aus
> https://github.com/Blackfort-Technology/days-and-keys/issues/77. Er beschreibt nur, was der Code
> der Android-App (Stand `main` vom 2026-10-17, Version 0.1) belegt; die Belege stehen im Anhang.
> Stellen in eckigen Klammern sind vor der Veröffentlichung zu füllen oder zu entscheiden.

## Veröffentlichung (nicht Teil der Erklärung)

Google Play verlangt eine öffentlich erreichbare URL, die ohne Anmeldung als Webseite (nicht als
PDF) lesbar ist. Zwei Orte kommen in Frage:

| Ort | Vorteile | Nachteile |
|---|---|---|
| **A: Seite auf https://www.blackfort-tec.de**, etwa https://www.blackfort-tec.de/days-and-keys/datenschutz | Anbieter und Erklärung unter derselben Domain wie Impressum und Kontakt; die Adresse bleibt stabil, auch wenn das Repo umzieht; Änderungen laufen über die Pipeline der Website mit Freigabe | Seite muss angelegt und gepflegt werden; sie liegt getrennt vom Code |
| **B: GitHub Pages dieses öffentlichen Repos**, etwa https://blackfort-technology.github.io/days-and-keys/datenschutz | liegt neben dem Code, jede Änderung ist im Verlauf nachvollziehbar; kein zusätzlicher Aufwand an der Website | GitHub verarbeitet beim Aufruf Daten der Besucher (Zugriffsprotokolle) und müsste in der Erklärung selbst genannt werden; die Adresse hängt am Namen von Organisation und Repo; eine Seite unter github.io wirkt für Laien weniger verbindlich als die Seite des Anbieters |

**Empfehlung: A.** Die Erklärung ist ein rechtliches Dokument des Anbieters und gehört zu dessen
Impressum und Kontaktweg. Für die Besucher der Seite gilt dann die bestehende
Datenschutzerklärung der Website; eine zweite Stelle mit eigener Verarbeitung (GitHub) entfällt.
Die Fassung im Repo bleibt die Quelle; die Seite übernimmt sie mit Datum. Entscheidung bei
Christian (`gesundheit.md`, E8).

---

## Datenschutzerklärung

Stand: [Datum der Veröffentlichung]

### 1. Kurz gesagt

Days & Keys ist ein persönliches Tagebuch. Alles, was du einträgst, bleibt verschlüsselt auf
deinem Gerät. Die App hat keine Berechtigung für das Internet, es gibt kein Konto und keinen
Server. Wir, die Blackfort Technology UG (haftungsbeschränkt), bekommen über die App keine Daten
von dir. Nur wenn du es einrichtest, legt die App verschlüsselte Sicherungen in einen Ordner, den du
selbst auswählst.

### 2. Verantwortlicher

Blackfort Technology UG (haftungsbeschränkt)
Ernst-Robert-Curtius-Straße 8A
53117 Bonn
Deutschland

Vertreten durch den Geschäftsführer Christian Gebhardt
Registergericht: Amtsgericht Bonn, HRB 23120
E-Mail: [`<KONTAKT>`]

[Prüfen: Ein Datenschutzbeauftragter ist nicht benannt, weil keine Pflicht dazu besteht.]

[Prüfen (#77): Ob die UG für die Verarbeitung auf dem Gerät überhaupt Verantwortlicher ist, ist
offen. Sie stellt die App bereit, verarbeitet aber selbst keine Daten aus ihr.]

### 3. Welche Daten die App auf deinem Gerät verarbeitet

Die App speichert nur, was du selbst einträgst oder einstellst:

- Angaben zu deinem Befinden, etwa Stimmung, Antrieb, Konzentration, Unruhe, Anspannung,
  Müdigkeit, Appetit und körperliche Beschwerden, jeweils mit Zeitpunkt;
- Ereignisse, etwa Einnahmen, Mahlzeiten, Bewegung, Schlaf, Termine, Belastungen und Krisen;
- Medikamente mit Name, Wirkstoff, Stärke, Einnahmeplan und bestätigten Einnahmen;
- Tagesabschlüsse, Wochenrückblicke und Notizen;
- deinen Plan für Tiefs und deine Vertrauenspersonen mit Name und Telefonnummer;
- Einstellungen wie Tageswechsel, Sperrzeit, Erinnerungszeiten und eigene Tagesmarker.

Viele dieser Angaben sind Gesundheitsdaten im Sinne von Art. 9 DSGVO. Die App wertet sie nicht
aus. Sie zeigt sie dir als Liste und Verlauf, mit Tageswerten und Mittelwerten über mehrere Tage.

**Verschlüsselung.** Die Einträge liegen verschlüsselt im privaten Speicher der App. Der Schlüssel
ist mit einem Schlüssel im Schlüsselspeicher deines Geräts (Android Keystore) geschützt, der sich
nur nach Fingerabdruck, Gesicht oder Geräte-Code verwenden lässt. Die Prüfung von Fingerabdruck
oder Gesicht übernimmt Android; die App erhält nur das Ergebnis, keine biometrischen Daten.

**Was unverschlüsselt auf dem Gerät liegt:**

- dein Plan für Tiefs, damit die Hilfe-Seite ihn auch gesperrt zeigen kann (in der Sicherung ist er
  verschlüsselt);
- der Erinnerungsplan mit Uhrzeiten und neutralen Texten, damit Erinnerungen nach einem Neustart
  wieder gesetzt werden; er enthält keine Medikamentennamen und keine Werte;
- die Adresse des Sicherungsordners.

Wer den Code deines Geräts kennt, kann die App öffnen und die Einträge lesen.

### 4. Keine Übermittlung an uns oder an Dritte

Die App fordert keine Berechtigung für den Internetzugang an und kann deshalb selbst keine Daten
über ein Netz senden. Sie enthält keine Werbung, keine Analyse- oder Statistikdienste und keine
Dienste für Absturzberichte. Wir erhalten keine Einträge, keine Kennungen und keine
Nutzungsdaten. Die automatische Android-Sicherung in dein Google-Konto und die Übertragung beim
Gerätewechsel sind für die App abgeschaltet.

### 5. Sicherung in einen Ordner deiner Wahl

Wenn du einen Sicherungsordner wählst, legt die App dort automatisch Sicherungsdateien ab. Den
Ordner wählst du über den Dateidialog von Android; er kann auf dem Gerät, auf einer Speicherkarte
oder bei einem Cloud-Anbieter liegen.

- Die Sicherungen sind verschlüsselt. Öffnen lassen sie sich nur mit deinem Schlüssel, den du mit
  deinen 24 Recovery-Wörtern wiederherstellen kannst. Wir haben diesen Schlüssel nicht.
- Liegt der Ordner bei einem Cloud-Anbieter, überträgt dessen App die Dateien. Der Anbieter sieht
  nur verschlüsselte Daten, dazu die Größe, den Zeitpunkt und den Dateinamen. Der Dateiname
  enthält zufällige Kurzkennungen deines Kontos in der App und deines Geräts sowie eine laufende
  Nummer, aber keinen Namen und keine Gesundheitsangaben. Für die Verarbeitung beim Anbieter
  gelten dessen Bedingungen und Datenschutzhinweise; wir haben darauf keinen Einfluss.
- Die App behält die letzten zehn Sicherungen und je eine pro Tag der letzten 30 Tage und löscht
  ältere Sicherungen im Ordner selbst.

### 6. Export

Auf deinen Wunsch und nach Bestätigung mit Fingerabdruck, Gesicht oder Geräte-Code erstellt die
App eine Tabelle aller Einträge (CSV). Diese Datei ist **nicht verschlüsselt**. Du wählst, wo sie
gespeichert wird. Was danach mit ihr geschieht, etwa wenn du sie weitergibst, liegt bei dir.

### 7. Benachrichtigungen

Erinnerungen plant die App auf deinem Gerät zu den Uhrzeiten, die du einstellst. Sie laufen nicht
über einen Server. Die Texte sind neutral und nennen kein Medikament. Auf dem Sperrbildschirm
zeigt Android sie als privat an, je nach deinen Einstellungen also ohne Inhalt.

### 8. Telefonnummern in der Hilfe

Tippst du in der Hilfe auf eine Nummer, öffnet die App das Wählfeld deines Telefons mit dieser
Nummer. Angerufen wird erst, wenn du es dort bestätigst. Die App benachrichtigt niemanden.

### 9. Google Play

Wenn du die App über Google Play lädst, verarbeitet Google dabei Daten in eigener Verantwortung,
etwa zu deinem Google-Konto und zum Download. Dafür gelten die Datenschutzhinweise von Google.
[Prüfen: Google stellt Anbietern in der Play Console zusammengefasste Statistiken zu Installationen
bereit und, wenn du das in Android erlaubt hast, Absturzberichte. Ob und in welcher Form das hier
zu nennen ist, ist vor der Veröffentlichung zu klären. Die App selbst sendet nichts.]

### 10. Rechtsgrundlage und Speicherdauer

Wir verarbeiten über die App keine personenbezogenen Daten. Die Daten auf deinem Gerät verarbeitest
du selbst; sie bleiben dort, bis du sie löschst. [Prüfen: Rechtsgrundlage für Anfragen per E-Mail,
Art. 6 Abs. 1 lit. b oder f DSGVO.]

Wenn du uns schreibst, verwenden wir deine Nachricht und deine Adresse nur, um zu antworten, und
löschen sie, wenn die Anfrage erledigt ist und keine Aufbewahrungspflicht besteht.

### 11. Löschen

- Einzelne Einträge löschst du in der App. Sie verschwinden sofort; in älteren Sicherungen bleiben
  sie höchstens 30 Tage erhalten.
- Alle Daten auf dem Gerät löschst du, indem du die App deinstallierst oder in den
  Android-Einstellungen ihren Speicher löschst.
- Die Sicherungen löschst du, indem du die Dateien im Sicherungsordner löschst (Dateien, deren Name
  mit `dak-` beginnt).

### 12. Deine Rechte

Du hast nach der DSGVO das Recht auf Auskunft (Art. 15), Berichtigung (Art. 16), Löschung (Art. 17),
Einschränkung der Verarbeitung (Art. 18), Datenübertragbarkeit (Art. 20) und Widerspruch (Art. 21).
Da wir keine Daten aus der App haben, kannst du die meisten dieser Rechte direkt in der App
ausüben: Einträge ansehen, korrigieren, löschen und als Tabelle exportieren. Für Fragen erreichst
du uns unter [`<KONTAKT>`].

Du kannst dich bei einer Datenschutz-Aufsichtsbehörde beschweren, zum Beispiel bei der für uns
zuständigen Landesbeauftragten für Datenschutz und Informationsfreiheit Nordrhein-Westfalen.

### 13. Alter

Die App ist für Erwachsene ab 18 Jahren bestimmt.

### 14. Änderungen

Ändert sich die App so, dass sich an der Verarbeitung etwas ändert, passen wir diese Erklärung an
und nennen oben das neue Datum. Frühere Fassungen stehen im öffentlichen Quelltext unter
https://github.com/Blackfort-Technology/days-and-keys

---

## Anhang: Belege im Code (nicht Teil der Erklärung)

Pfade relativ zum Repo; Kotlin-Pfade unter
`android/app/src/main/java/de/blackforttec/daysandkeys/`. Ausführlich in `datensicherheit.md`.

| Abschnitt | Aussage | Beleg |
|---|---|---|
| 2 | Anschrift, HRB, Geschäftsführer | Knowledge Base `02_organizations/blackfort-technology.md` (nicht im Repo) |
| 3 | Datenarten | `templates/standard-v1.json`; Vertrauenspersonen `core/crates/dak-core/src/modell.rs`, Zeile 948; Einstellungen `strings.xml` Zeilen 303 bis 340 |
| 3 | Mittelwerte | `core/crates/dak-core/src/ansicht.rs`, Zeilen 481 bis 489 |
| 3 | Verschlüsselt, Keystore, Geräteauthentifizierung | `DaysAndKeysApp.kt`, Zeilen 23 bis 25; `plattform/Schluesselspeicher.kt`, Zeilen 22 bis 26, 43 bis 51 und 65 bis 73 (`BiometricPrompt`, die App erhält nur den Erfolg) |
| 3 | Unverschlüsselt: Plan, Erinnerungsplan, Ordneradresse | `core/crates/dak-core/src/kern.rs`, Zeilen 540 und 541; `plattform/Planer.kt`, Zeilen 22 bis 24; `plattform/Sicherungsordner.kt`, Zeilen 17 bis 21; Hinweis in der App `strings.xml`, Zeile 325 |
| 3 | Code des Geräts öffnet | `strings.xml`, Zeile 290 |
| 4 | keine Internet-Berechtigung | `android/app/src/main/AndroidManifest.xml`, Zeilen 3 und 6 bis 10; `.github/workflows/ci.yml`, Zeilen 157 bis 161 |
| 4 | keine Fremdbibliotheken für Werbung, Analyse, Absturzberichte | `android/app/build.gradle.kts`, Zeilen 61 bis 71; `core/crates/dak-core/Cargo.toml`, Zeilen 12 bis 19 |
| 4 | Android-Sicherung aus | `AndroidManifest.xml`, Zeilen 14 bis 16; `android/app/src/main/res/xml/datenauszug.xml`, Zeilen 3 bis 17 |
| 5 | Ordnerwahl, automatisch, verschlüsselt | `plattform/Sicherungsordner.kt`, Zeilen 12 bis 15 und 46 bis 67; `ui/Zustand.kt`, Zeilen 109 und 127; `core/crates/dak-core/src/sicherung.rs`, Zeilen 4 bis 12 |
| 5 | Dateiname | `sicherung.rs`, Zeilen 378 bis 387 |
| 5 | Aufbewahrung 10 und 30 Tage | `sicherung.rs`, Zeilen 399 bis 402; Löschen im Ordner `plattform/Sicherungsordner.kt`, Zeilen 63 bis 66 |
| 6 | Export im Klartext nach Bestätigung | `ui/Einstellungen.kt`, Zeilen 68 bis 74 und 222 bis 227; `strings.xml`, Zeile 347 |
| 7 | Benachrichtigungen lokal, neutral, privat | `plattform/Planer.kt`, Zeilen 21 bis 24, 33, 36 und 94; `strings.xml`, Zeile 208 |
| 8 | Wählfeld | `ui/Hilfe.kt`, Zeile 40 (`ACTION_DIAL`); `strings.xml`, Zeile 99 |
| 11 | Einzelne Einträge löschen, 30 Tage in Sicherungen | `ui/Eintraege.kt`, Zeilen 101 bis 107; `kern.rs`, Zeilen 921 bis 929; `strings.xml`, Zeile 156 |
| 11 | Dateien `dak-` | `sicherung.rs`, Zeile 381; `plattform/Sicherungsordner.kt`, Zeile 36 (`.daks`, `.dakm`) |
| 13 | ab 18 | `gesundheit.md`, Abschnitt 3 (Entscheidung E4 offen) |
