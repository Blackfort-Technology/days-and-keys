# Google Play - Abschnitt „Datensicherheit“ (Entwurf)

Stand 2026-10-17 · Entwurf zu https://github.com/Blackfort-Technology/days-and-keys/issues/160 ·
Grundlage: Stand `main` vom 2026-10-17, Android-App Version 0.1.

Antworten für das Formular „Datensicherheit“ (Data safety) unter „App-Inhalte“ in der Play
Console, Frage für Frage. Jede Antwort hat einen Beleg im Code; Pfade sind relativ zum Repo,
Zeilen beziehen sich auf den genannten Stand. Stellen, an denen die Auslegung von Googles Begriffen
nicht sicher ist, sind mit **OFFEN** gekennzeichnet.

## 0. Googles Begriffe

Aus https://support.google.com/googleplay/android-developer/answer/10787469 (abgerufen 2026-10-17):

- **Erheben (collect):** „transmitting data from your app off a user's device“, also Daten aus
  der App vom Gerät wegschicken.
- **Teilen (share):** Daten, die die App erhoben hat, an Dritte weitergeben.
- **Ausnahmen:** Daten, die nur auf dem Gerät verarbeitet werden, gelten nicht als erhoben.
  Daten, die Ende-zu-Ende verschlüsselt sind und die außer Absender und Empfänger niemand lesen
  kann, müssen nicht als erhoben angegeben werden. Eine Weitergabe auf eine ausdrückliche Handlung
  der Person hin, bei der sie die Weitergabe erwartet, gilt nicht als Teilen.
- Auch Apps, die nichts erheben, müssen das Formular ausfüllen und eine Datenschutzerklärung
  verlinken.

## 1. Was verlässt das Gerät?

Kurz: Die App selbst schickt nichts über ein Netz, weil sie keine Netzberechtigung hat. Sie
schreibt nur in Ordner und Dateien, die die Person selbst auswählt.

| Weg | Inhalt | Wann | Verschlüsselt | Beleg |
|---|---|---|---|---|
| Netzwerk | nichts | nie | entfällt | `android/app/src/main/AndroidManifest.xml`, Zeilen 3 und 6 bis 10 (keine `INTERNET`-Berechtigung); die CI bricht ab, wenn die fertige APK sie verlangt: `.github/workflows/ci.yml`, Zeilen 157 bis 161 |
| Android-Sicherung (Google-Konto) und Geräteumzug | nichts | nie | entfällt | `AndroidManifest.xml`, Zeilen 14 bis 16 (`allowBackup="false"`, `dataExtractionRules`); `android/app/src/main/res/xml/datenauszug.xml`, Zeilen 3 bis 17 (alle Bereiche ausgeschlossen) |
| Sicherungsordner | alle Einträge, Präparate, Einstellungen, Vertrauenspersonen, Plan für Tiefs | nur wenn die Person einen Ordner gewählt hat; danach automatisch beim Entsperren und beim Wechsel in den Hintergrund, wenn eine Sicherung fällig ist, sowie auf Tipp „Jetzt sichern“ | ja, `crypto_secretstream` aus libsodium mit einem Schlüssel, den nur die Person hat (aus den Recovery-Wörtern wiederherstellbar) | Ordnerwahl: `android/app/src/main/java/de/blackforttec/daysandkeys/ui/Einrichtung.kt`, Zeilen 104 und 105, `ui/Einstellungen.kt`, Zeilen 65 und 66; Schreiben: `plattform/Sicherungsordner.kt`, Zeilen 23 bis 26 und 46 bis 67; Auslöser: `ui/Zustand.kt`, Zeilen 109, 127 und 148 bis 167; Format: `core/crates/dak-core/src/sicherung.rs`, Zeilen 4 bis 12 |
| CSV-Export | alle Einträge im Klartext | nur nach Tipp auf „Export erstellen“ und Bestätigung mit Fingerabdruck, Gesicht oder Geräte-Code; Ziel wählt die Person im Dateidialog | nein, Klartext; die App warnt davor, ihn in einen Cloud-Ordner zu legen | `ui/Einstellungen.kt`, Zeilen 68 bis 74 und 222 bis 227; `android/app/src/main/res/values/strings.xml`, Zeile 347 |
| Wählfeld | Telefonnummer der gewählten Beratungsstelle oder Vertrauensperson | nur nach Tipp; die Telefon-App öffnet sich, angerufen wird erst nach Bestätigung dort | entfällt | `ui/Hilfe.kt`, Zeile 40 (`ACTION_DIAL`, keine Berechtigung `CALL_PHONE` im Manifest) |
| Benachrichtigungen | neutrale Titel und Texte, kein Präparatname, keine Werte | zu den selbst gewählten Uhrzeiten, lokal über `AlarmManager` | entfällt (bleiben auf dem Gerät) | `plattform/Planer.kt`, Zeilen 21 bis 24, 33, 36 und 94; `strings.xml`, Zeile 208 |

Alle Kotlin-Pfade liegen unter `android/app/src/main/java/de/blackforttec/daysandkeys/`.

Weitere Belege, dass nichts Fremdes mitläuft: Die App bindet nur Compose, Activity, JNA und
Coroutines ein, keine Bibliothek für Analyse, Absturzberichte oder Werbung
(`android/app/build.gradle.kts`, Zeilen 61 bis 71). Der Kern hängt nur an libsodium, zeroize,
rusqlite, thiserror, serde und uniffi (`core/crates/dak-core/Cargo.toml`, Zeilen 12 bis 19); er
hat keine Netzabhängigkeit (`CLAUDE.md`, Abschnitt „Bauen“).

Auf dem Gerät liegen die Daten in `noBackupFilesDir` (`DaysAndKeysApp.kt`, Zeilen 23 bis 25),
verschlüsselt; der Schlüssel ist mit einem nicht exportierbaren Schlüssel im Android Keystore
umhüllt, der eine frische Geräteauthentifizierung verlangt (`plattform/Schluesselspeicher.kt`,
Zeilen 22 bis 26 und 43 bis 51). Unverschlüsselt auf dem Gerät liegen nur: der Plan für Tiefs, damit
die Hilfe ihn gesperrt zeigen kann (`core/crates/dak-core/src/kern.rs`, Zeilen 540 und 541), der
Erinnerungsplan mit Uhrzeiten und neutralen Texten (`plattform/Planer.kt`, Zeilen 22 bis 24) und
die Adresse des Sicherungsordners (`plattform/Sicherungsordner.kt`, Zeilen 17 bis 21). Für das
Formular zählt das nicht, weil es das Gerät nicht verlässt; in der Datenschutzerklärung steht es.

## 2. Antworten im Formular

### Frage 1: Erhebt oder teilt deine App die erforderlichen Nutzerdatentypen?

**Antwort: Nein.**

Begründung: Die App sendet nichts vom Gerät weg (Abschnitt 1, Zeile „Netzwerk“). Die Sicherung
ist Ende-zu-Ende verschlüsselt und nur mit dem Schlüssel der Person lesbar; Absender und Empfänger
sind dieselbe Person. Blackfort Technology erhält keine Daten, es gibt keinen Server.

**OFFEN (O1), Sicherung in einen Cloud-Ordner.** Die App schreibt über das Storage Access
Framework in einen Ordner, den die Person wählt. Ist das ein Cloud-Anbieter (etwa Google Drive,
Nextcloud, OneDrive), lädt dessen App die Datei hoch, nicht Days & Keys. Drei Gründe sprechen für
„Nein“: Days & Keys überträgt nichts selbst; die Datei ist Ende-zu-Ende verschlüsselt
(Ausnahme „end-to-end encryption“); die Person wählt den Ordner selbst. Google nennt diesen Fall
aber nicht ausdrücklich. Sichtbar bleiben für den Anbieter Dateigröße, Zeitpunkt und der
Dateiname mit je acht Hexadezimalzeichen aus Konto- und Gerätekennung, Generation und laufender
Nummer (`core/crates/dak-core/src/sicherung.rs`, Zeilen 378 bis 387). Diese Kennungen sind
zufällig und keiner Person zugeordnet, Blackfort sieht sie nie.

**OFFEN (O2), CSV-Export.** Der Export ist Klartext und geht an den Ort, den die Person im
Dateidialog wählt, auch in eine Cloud. Die App sendet ihn nicht selbst; er entsteht nur auf eine
ausdrückliche Handlung mit Bestätigung hin (Ausnahme „user-initiated action“). Auch das nennt Google
nicht ausdrücklich. Wir halten „Nein“ für richtig, solange die App den Export nicht selbst an
einen Dienst schickt.

Wer O1 oder O2 anders auslegt, beantwortet Frage 1 mit „Ja“ und füllt die Abschnitte 3 und 4
unten aus. Entscheidung bei Christian (siehe `gesundheit.md`, E5).

### Frage 2: Werden alle von deiner App erhobenen Nutzerdaten bei der Übertragung verschlüsselt?

Wird bei „Nein“ in Frage 1 nicht gestellt. Falls doch „Ja“ (O1): **Ja.** Die Sicherungsdatei ist
vor dem Schreiben mit `crypto_secretstream` verschlüsselt (`sicherung.rs`, Zeilen 4 bis 12); wie
der Cloud-Anbieter sie überträgt, hängt von ihm ab, ändert aber nichts daran, dass er nur
Chiffretext sieht. Für den CSV-Export (O2) wäre die Antwort **Nein**, weil er Klartext ist; dann
müsste der Export als erhoben angegeben werden, und „alle Daten verschlüsselt“ wäre falsch.

### Frage 3: Bietest du eine Möglichkeit, das Löschen der Daten zu beantragen?

Wird bei „Nein“ in Frage 1 nicht gestellt. Es gibt nichts zu beantragen, weil Blackfort keine
Daten hat. Die Person löscht selbst:

- einzelne Einträge in der App (`ui/Eintraege.kt`, Zeilen 101 bis 107; `kern.rs`, Zeilen 921 bis
  929). In älteren Sicherungen bleibt ein gelöschter Eintrag höchstens 30 Tage
  (`strings.xml`, Zeile 156; Aufbewahrung `sicherung.rs`, Zeilen 399 bis 402: die letzten zehn
  Sicherungen und je eine pro Tag der letzten 30 Tage);
- alles auf dem Gerät durch Deinstallieren oder „Speicher löschen“ in den Android-Einstellungen;
  eine Kopie bei Google gibt es nicht (`datenauszug.xml`);
- die Sicherungen durch Löschen der Dateien `dak-*.daks` und `dak-*.dakm` im Sicherungsordner.

Eine Schaltfläche „Alles löschen“ in der App gibt es noch nicht; sie ist für M1b geplant
(https://github.com/Blackfort-Technology/days-and-keys/issues/39). Falls Frage 1 „Ja“ wird,
verlangt Google zusätzlich einen Link, über den man das Löschen anfordern kann; dann reicht ein
Abschnitt in der Datenschutzerklärung mit dieser Anleitung.

### Frage 4: Unabhängige Sicherheitsprüfung (optional)

**Nicht angeben.** Es gibt keine unabhängige Prüfung nach dem Standard, den Google dafür
anerkennt. Die Scans in der CI und im Lab-Jenkins (`CLAUDE.md`, Abschnitt „Pipeline“) sind eigene
Prüfungen.

### Frage 5: Entspricht die App der Richtlinie für Familien?

Entfällt; die Zielgruppe ist ab 18 (`gesundheit.md`, Abschnitt 3).

## 3. Datentypen

Für den Fall „Nein“ in Frage 1 wird hier nichts angegeben. Die Tabelle zeigt, welche von Googles
Datentypen die App auf dem Gerät verarbeitet, damit die Antwort nachvollziehbar ist und bei
„Ja“ sofort ausgefüllt werden kann.

| Googles Datentyp | Was die App dazu verarbeitet | Verlässt das Gerät | Angabe |
|---|---|---|---|
| Gesundheit und Fitness: Gesundheitsinformationen | Befinden, körperliche Beschwerden, Medikamente und Einnahmen, Schlaf, Arzttermine, Krisen-Ereignisse (`templates/standard-v1.json`) | nur verschlüsselt im Sicherungsordner (O1), im Klartext im Export (O2) | keine; bei „Ja“: erhoben, nicht geteilt, Zweck „App-Funktionen“, optional |
| Gesundheit und Fitness: Fitnessinformationen | Bewegung und Dauer (`standard-v1.json`, Formular „Bewegung“) | wie oben | wie oben |
| Personenbezogene Daten: Name, Telefonnummer | Vertrauenspersonen mit Name und Telefon, von Hand eingetragen (`core/crates/dak-core/src/modell.rs`, Zeile 948) | nur verschlüsselt im Sicherungsordner | keine; bei „Ja“: erhoben, nicht geteilt, „App-Funktionen“, optional |
| Kontakte | keine; die App liest das Adressbuch nicht (keine Berechtigung `READ_CONTACTS` im Manifest) | nein | keine |
| App-Aktivitäten: Sonstige nutzergenerierte Inhalte | Notizen, Plan für Tiefs, eigene Tagesmarker | wie Gesundheitsinformationen | wie Gesundheitsinformationen |
| Dateien und Dokumente | keine Dateien der Person; die App liest im Sicherungsordner nur ihre eigenen Sicherungen (`Sicherungsordner.kt`, Zeile 36) | nein | keine |
| App-Informationen und -Leistung (Absturzberichte, Diagnose) | keine | nein | keine |
| Geräte-IDs und andere IDs | keine Werbe- oder Gerätekennung; die Konto- und Gerätekennung im Dateinamen ist zufällig und nur in der Sicherung | nur als Teil des Dateinamens (O1) | keine |
| Standort, Nachrichten, Fotos, Audio, Kalender, Finanzen, Browserverlauf | keine | nein | keine |

## 4. Sonstige Hinweise für die Console

- **Datenschutzerklärung:** Pflichtfeld, auch bei „Nein“. Entwurf in `datenschutzerklaerung.md`.
- **Berechtigungen:** `SCHEDULE_EXACT_ALARM` (Manifest, Zeile 8) dient pünktlichen Erinnerungen.
  Ab Android 14 muss die Person sie selbst erlauben; die App fragt nur, wenn nötig
  (`strings.xml`, Zeile 272). Für `USE_EXACT_ALARM` gäbe es eine Erklärungspflicht; die App nutzt
  sie nicht.
- **Health Connect:** nicht genutzt, daher keine Erklärung für Health-Connect-Berechtigungen.
