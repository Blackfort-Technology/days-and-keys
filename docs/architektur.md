# Architekturkonzept Days & Keys

Stand 2026-10-14, Version 0.2. Issues #33, #3. Gilt für M1a und M1b; M1.5 ist nur so weit
beschrieben, wie M1 darauf Rücksicht nehmen muss.

Dieses Dokument bündelt alle Architekturentscheidungen an einem Ort. Die fachlichen Regeln stehen
weiter in `docs/vorlagen.md` (Fragen), `docs/protokoll.md` (Teil A, Kryptografie und Formate) und
`docs/protokoll-freigaben.md` (Teil B, M1.5). Wo dieses Dokument und das Protokoll sich
widersprechen, gilt das Protokoll, und der Widerspruch ist ein Fehler, der als Issue gemeldet wird.
Die Recherchen, auf denen die neuen Entscheidungen beruhen, liegen unter `docs/architektur/`
(`framework.md`, `bibliotheken.md`).

**Status der Entscheidungen**

| Status | Bedeutung |
|---|---|
| entschieden | von der Projektleitung bestätigt oder in einer Erörterung beschlossen; Quelle steht dabei |
| vorgeschlagen | Empfehlung mit Begründung; **die Projektleitung entscheidet**, bevor darauf gebaut wird |
| vorläufig | entschieden unter dem Vorbehalt einer Messung im Prototyp (#100, #101) |
| angenommen | vorgeschlagen und mit dem Start von #100 ohne Widerspruch übernommen |

## 1. Architekturtreiber

### 1.1 Qualitätsziele, nach Rang

1. **Vertraulichkeit der Gesundheitsdaten** (Art. 9 DSGVO, Tier 1 nach dem Test-Standard): Kein
   Klartext verlässt das Gerät, kein Betreiber und kein Dienst kann mitlesen, kein Generalschlüssel.
2. **Kein Datenverlust:** Wer Gerät und Recovery-Wörter hat, bekommt alles zurück; die Sicherung
   läuft von selbst.
3. **Schnelle Erfassung, auch an schlechten Tagen:** ein Status in wenigen Sekunden, Lücken ohne
   Strafe, keine Vorbelegung. Die Dauer wird gemessen (#81), nicht angenommen.
4. **Ehrliche Auswertung:** Punkte, Fallzahlen und Lücken; keine Deutungen.
5. **Prüfbarkeit:** Open Source, ein Kryptokern mit Testvektoren, gepinnte und dokumentierte
   Abhängigkeiten, reproduzierbare Builds als Ziel.
6. **Wartbarkeit durch eine kleine Besetzung:** ein kleines Team; wenig bewegliche Teile,
   wenig Versionsdruck.

### 1.2 Randbedingungen

| Randbedingung | Folge |
|---|---|
| Zielgeräte: aktuelle Android- und iOS-Geräte | zwei Plattformen ab dem ersten Tag |
| kein eigener Mac | iOS-Bau und iOS-Tests nur auf den macOS-Runnern von GitHub Actions (#8, #99) |
| Apple-Konto der Blackfort Technology UG (#10) | iOS-Verteilung hängt an diesem Konto (#10) |
| Entwicklung über GitHub und CI (GitHub Actions) nach dem Software-Entwicklungs-Standard | jede Änderung per PR, Governance-Gate, Security-Scans, Secrets aus einem Secret-Store |
| keine echten Personen- oder Gesundheitsdaten in Repo, Tests, Issues | synthetische Testdaten, Platzhalternamen |
| Nutzungsversuch mit vier bis sechs Wochen (#82) | Format in Phase A eingefroren; M1a muss das volle Teil-A-Format schreiben |

## 2. Kontext

```
                 ┌──────────────────────────────┐
  Person ───────►│  Days & Keys auf dem Gerät   │──── lokale Benachrichtigungen
                 │  (kein Netzzugang)           │
                 └──────┬──────────────┬────────┘
                        │              │
       verschlüsselte   │              │  Arzt-PDF, Export
       Sicherungsdatei  │              │  (nur nach Bestätigung, über das
                        ▼              ▼   Teilen-Menü des Systems)
          gewählter Ordner         Empfänger nach Wahl der Person
          (Gerät, Cloud-Ordner,
           USB, Nextcloud …)
```

In M1 gibt es **keinen Server und keine Netzverbindung der App**. Alles, was das Gerät verlässt,
geht über Wege, die das Betriebssystem bereitstellt und die die Person selbst auslöst oder
einmal einrichtet: den Sicherungsordner und das Teilen-Menü. Ab M1.5 kommt ein Server unter
`https://daysandkeys.wirdsichtbar.de` dazu, der nur Chiffretext sieht (AE-25).

## 3. Bausteinsicht

```
┌─ Android-App (Kotlin, Jetpack Compose) ──┐  ┌─ iOS-App (Swift, SwiftUI) ───────────────┐
│ Ansichten zeichnen Ansichtsmodelle       │  │ Ansichten zeichnen Ansichtsmodelle       │
│ native Dialoge: PIN, Wörter, Bestätigung │  │ native Dialoge: PIN, Wörter, Bestätigung │
│ Hilfe ohne Entsperren                    │  │ Hilfe ohne Entsperren                    │
├─ Plattformdienste ───────────────────────┤  ├─ Plattformdienste ───────────────────────┤
│ Keystore (StrongBox/TEE), BiometricPrompt│  │ Keychain, Secure Enclave, LocalAuth.     │
│ AlarmManager, BOOT_COMPLETED             │  │ UNUserNotificationCenter, BGTasks        │
│ Storage Access Framework                 │  │ Security-scoped Bookmarks                │
│ FLAG_SECURE, Backup-Ausschluss           │  │ Datenschutzklassen, Backup-Ausschluss    │
└──────────────────┬───────────────────────┘  └──────────────────┬───────────────────────┘
                   │      UniFFI (Kotlin- und Swift-Bindung, eine Schnittstelle)
┌──────────────────▼─────────────────────────────────────────────▼───────────────────────┐
│ dak-core (Rust)                                                                        │
│  dak-ffi      einzige exportierte Schnittstelle, Fehlercodes, Rückrufe zur Plattform   │
│  dak-view     Ansichtsmodelle: sichtbare Fragen, Bedingungen, Bezugszeitraum, Verlauf, │
│               Texte und Digest für Bestätigungen                                       │
│  dak-model    Status, Antworten, Ereignisse, Abfragen, Messdefinitionen, Formulare,    │
│               Lebenstag, Medikation                                                    │
│  dak-session  Sitzung, Sperrzeit, Entwürfe, Autorisierung sensibler Aktionen           │
│  dak-keys     Bund, Schlüsselklassen, Kontokette, Rotation, Recovery                   │
│  dak-crypto   Hülle um libsodium, geschützter Speicher, Suiten dak-s1/dak-s2           │
│  dak-format   strenges deterministisches CBOR, Versionen, Migration                    │
│  dak-store    SQLite: Datensätze, Umhüllungen, Verlaufsblöcke, Planungsstand           │
│  dak-backup   Sicherungsdatei schreiben, prüfen, wiederherstellen                      │
│  dak-plan     Planer für Erinnerungen und Zufallsabfragen                              │
│  dak-report   Arzt-PDF und Export                                                      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

**Schnittregel:** Alles, was sich ohne Gerät prüfen lässt, liegt in `dak-core` und läuft in
der CI (GitHub Actions) unter Linux. Die Plattformschicht macht nur, was nur die Plattform kann: zeichnen,
Hardware-Schlüssel benutzen, Dateien im gewählten Ordner schreiben, Benachrichtigungen planen.
Die Module sind Crates eines Cargo-Workspace; nach außen sichtbar ist nur `dak-ffi`.

**Abhängigkeitsrichtung:** `dak-ffi` → `dak-view` → `dak-session` → `dak-model`, `dak-keys` →
`dak-crypto`, `dak-format`, `dak-store`. Nur `dak-crypto` ruft libsodium; nur `dak-store` öffnet
SQLite. `dak-report` und `dak-backup` bekommen Klartext ausschließlich über `dak-session`.

**Rückrufe zur Plattform** (UniFFI-Callback-Interfaces, bewusst wenige):

| Rückruf | Zweck |
|---|---|
| `HardwareSchluessel` | `KK` und `KK_h` mit dem Hardware-Schlüssel um- und enthüllen, `HMAC_hw` bzw. Secure-Enclave-ECDH für den PIN-Weg |
| `Bestaetigung` | nativen Dialog mit Text und Digest aus `dak-view` zeigen, frische Geräteauthentifizierung oder PIN einholen |
| `Sicherungsziel` | fertige, verschlüsselte Bytes in den gewählten Ordner schreiben, Dateien auflisten und lesen |
| `Benachrichtigungen` | geplante Benachrichtigungen setzen und löschen |
| `Uhr` | aktuelle Zeit und IANA-Zeitzone (für Tests ersetzbar) |

## 4. Laufzeitsicht

### 4.1 App öffnen und Status festhalten

1. Die App startet gesperrt. Sichtbar sind nur der Entsperr-Knopf und die Hilfe (AE-20).
2. Entsperren: Plattform zeigt die Geräteauthentifizierung, `HardwareSchluessel` enthüllt `KK`,
   die Bytes gehen sofort in geschützten Speicher von `dak-core`, der Plattformpuffer wird
   überschrieben (AE-12). `dak-session` öffnet den Bund und startet die Sitzung.
3. `dak-view` liefert das Ansichtsmodell „Status jetzt": Fragen, Gruppen, Bedingungen (z. B.
   Wirkungsfrage nur nach bestätigter Einnahme), Bezugszeitraum für „seit dem letzten Status".
4. Jede Eingabe geht als Ereignis an `dak-session` und wird laufend als Entwurf versiegelt
   (Protokoll 12). Speichern erzeugt einen Datensatz mit frischem `dk`, Lebenstag wird berechnet
   und gespeichert, der Verlaufsblock des Monats wird nachgeführt.
5. Nach dem Speichern merkt `dak-backup` eine Sicherung vor (AE-17).

### 4.2 Hochsensibel eintragen ohne PIN (M1b)

Tipp auf eine Substanz-Platznummer → `dak-core` versiegelt den Datensatz an `WS_h.pub`, markiert
ihn als vorläufig und schreibt ihn in den hochsensiblen Bereich. Lesen und Übernehmen erst nach
frischer App-PIN (Protokoll 8.2).

### 4.3 Erinnerung und Zufallsabfrage

`dak-plan` berechnet die nächsten Tage (höchstens 64 Einträge auf iOS) und schreibt den
Planungsstand ohne Gesundheitsinhalte in eine kleine, unverschlüsselte Planerdatei. Die Plattform
setzt die Benachrichtigungen; nach einem Neustart liest der Empfänger nur die Planerdatei. Tippt die
Person auf die Benachrichtigung, öffnet sich die App gesperrt mit dem passenden Formular.

### 4.4 Sicherung und Wiederherstellung

Sicherung: `dak-backup` erzeugt die Sicherungsdatei (Protokoll 10), `Sicherungsziel` schreibt sie
in den Ordner, `dak-backup` liest sie zurück und prüft sie, erst dann gilt sie als gelungen. Die
App zeigt den Zeitpunkt der letzten geprüften Sicherung.
Wiederherstellung: Recovery-Wörter in nativer Ansicht → `RK` nur im Speicher von `dak-core` →
Sicherungen im Ordner nach Ankerregel und Kettenverwandtschaft auswählen → Bestand schreiben →
Rotation (Protokoll 9) → `RK` vergessen.

### 4.5 Arzt-PDF und Export

`dak-view` liefert Auswahl und Bestätigungstext mit Digest, `Bestaetigung` holt eine frische
Geräteauthentifizierung (bei hochsensiblen Inhalten die PIN), `dak-report` erzeugt das PDF in
`dak-core`, die Plattform übergibt es an das Teilen-Menü und löscht die temporäre Datei danach.

## 5. Verteilungssicht

```
Git-Repo (GitHub) ────────────────► CI (GitHub Actions, Linux)
                                     ├─ dak-core: fmt, clippy, Tests, Testvektoren, Fuzz-Kurzlauf,
                                     │  cargo-deny, cargo-audit, Coverage
                                     ├─ Android: cargo-ndk (arm64-v8a, x86_64), Gradle, Robolectric,
                                     │  Prüfung „keine INTERNET-Berechtigung", APK signieren
                                     ├─ Governance-Gate, Security-Scans
                                     └─ macOS-Runner (GitHub Actions)
                                                     XCFramework, Swift-Tests, Snapshot-Tests,
                                                     IPA signieren ─► TestFlight
Schlüssel für Signaturen: Secret-Store mit gesicherter Zweitkopie, nur zur Bauzeit übergeben (#68)
Testgeräte: Android (APK), iOS (TestFlight)
```

## 6. Repo-Struktur

```
days-and-keys/
├── core/                 Cargo-Workspace dak-core (Crates aus Abschnitt 3)
│   ├── crates/…
│   ├── fuzz/             cargo-fuzz-Ziele
│   └── testvectors/      eigene Vektoren, Wycheproof, ed25519-speccheck (Commit gepinnt)
├── android/              Gradle-Projekt, Modul app und Modul platform
├── ios/                  Xcode-Projekt, App-Ziel, später Widget-Erweiterung
├── templates/            Fragenvorlagen als Daten (CBOR-Quelle in lesbarer Form)
├── docs/                 Konzept, Protokoll, Architektur, Recherchen, Erörterungen
├── .github/workflows/    CI (GitHub Actions)
├── AUDITS.md, ROADMAP.md, README.md, CLAUDE.md, LICENSE
```

Ein Repo für alles (AE-29), damit Protokoll, Kern und beide Apps in einem PR zusammen geändert und
geprüft werden.

## 7. Querschnittliche Konzepte

| Thema | Regel |
|---|---|
| Fehler | `dak-core` liefert typisierte Fehler mit stabilen Codes; die Oberfläche zeigt verständlichen Text und den Code. Kein Fehlertext enthält Inhalte |
| Protokollierung | in Release-Builds keine Protokolle mit Inhalten, Schlüsseln, Feldnamen oder Zeitpunkten von Einträgen; Debug-Builds nur mit synthetischen Daten |
| Telemetrie, Absturzberichte | keine (AE-21) |
| Zeit | `jiff` mit IANA-Daten im Kern; UTC plus Versatz plus Zone, Lebenstag gespeichert (Protokoll 6.3) |
| Geheimnisse im Speicher | eigene Hülle auf `sodium_malloc`/`sodium_mprotect`, `zeroize` für alles andere; Geheimnisse nie als String (AE-12) |
| Sprache | Oberfläche zuerst Deutsch; Texte in den Ressourcen der Plattform, Fragenvorlagen als Daten mit Sprachfeld; Englisch vorbereitet (AE-22) |
| Barrierefreiheit | TalkBack und VoiceOver, dynamische Schriftgröße, Tipp-Reihe 0-10 mit Semantik ohne Vorauswahl, Kontraste nach WCAG 2.2 AA (AE-23) |
| Gestaltung | ruhig, keine Farbwertung von Werten, keine Belohnungsmechanik; Details `docs/gesamtkonzept.md` Abschnitt 6 |
| Tests | Tier 1: Security-Regressionstests Pflicht, Zielkorridor 80 % Abdeckung im Kern, Testvektoren für jedes Format, Negativtests, Fuzzing, echte Geräte für Hardware und Biometrie (#80) |

## 8. Entscheidungen

### Übersicht

| Nr. | Entscheidung | Status | Quelle |
|---|---|---|---|
| AE-01 | Lokal zuerst, Zero-Knowledge, kein Generalschlüssel | entschieden | Leitplanken, #3 |
| AE-02 | Open Source, Lizenz MPL-2.0 (Apps, Kern), AGPL-3.0 (Server), CC BY 4.0 (Doku) | entschieden | Projektleitung, 2026-10-14 (#106) |
| AE-03 | Native App statt Web-App | entschieden | 2026-10-06 |
| AE-04 | Oberfläche nativ: Jetpack Compose und SwiftUI, keine WebView | entschieden | Projektleitung, 2026-10-14 (#33, #106) |
| AE-05 | Rust-Kern `dak-core` mit libsodium | entschieden | #32 |
| AE-06 | Ansichtsmodelle in Rust, Plattform zeichnet nur | vorläufig, folgt aus AE-04 | #101 |
| AE-07 | Anbindung über UniFFI | vorläufig, folgt aus AE-04 | #101 |
| AE-08 | Kein Etebase, kein fremdes Sync-Framework | entschieden | #2 |
| AE-09 | Status jederzeit festhalten statt Tagebuch | entschieden | 2026-10-06 |
| AE-10 | Kryptosuiten `dak-s1`, `dak-s2` vorgesehen | entschieden | Protokoll 2 |
| AE-11 | Geräteentsperrung genügt, App-PIN nur hochsensibel | entschieden | #99 |
| AE-12 | Schlüssel nur im Kern, geschützter Speicher | entschieden | #32 |
| AE-13 | Datensatzformat: `dk` je Revision, eine Klasse je Datensatz, Manifest-MAC | entschieden | #86, Protokoll 7 |
| AE-14 | Strenges deterministisches CBOR, eigener kleiner Codec | entschieden, geändert mit #100 | #63, `bibliotheken.md` |
| AE-15 | Speicher SQLite über `rusqlite`, eine Datei, Verlaufsblöcke je Monat | angenommen mit #100 | #63, `bibliotheken.md` |
| AE-16 | Kontokette, Recovery mit getrenntem `RK_sig`/`RK_enc`, Rotation | entschieden | #86, #87 |
| AE-17 | Automatische Sicherung in einen gewählten Ordner | entschieden | #91 |
| AE-18 | Sicherungsdatei als `crypto_secretstream` in Blöcken | angenommen mit #100 | #63 |
| AE-19 | App ohne Netzzugang | entschieden | #59 |
| AE-20 | Hilfe ohne Entsperren, nativ | entschieden | #71 |
| AE-21 | Keine Telemetrie, keine Absturzberichte | angenommen mit #100 | dieses Dokument |
| AE-22 | Deutsch zuerst, Englisch vorbereitet | angenommen mit #100 | dieses Dokument |
| AE-23 | Barrierefreiheit nach WCAG 2.2 AA | angenommen mit #100 | dieses Dokument |
| AE-24 | Lokaler Planer für Erinnerungen | entschieden | #42, #93 |
| AE-25 | M1 ohne Server; ab M1.5 eigene schlanke API | entschieden | #96, #14 |
| AE-26 | Kein Freigabeformat auf Vorrat, M1a/M1b | entschieden | #96 |
| AE-27 | Beschreibende Auswertung ohne Deutung | entschieden | #74 |
| AE-28 | Arzt-PDF im Kern mit `krilla` | angenommen mit #100 | #36, `bibliotheken.md` |
| AE-29 | Monorepo | angenommen mit #100 | dieses Dokument |
| AE-30 | iOS-Bau auf macOS-Runnern von GitHub Actions, Verteilung TestFlight | entschieden | #8, #99, Projektleitung 2026-10-14 (#106) |
| AE-31 | Android: signierte APK aus der CI (GitHub Actions), später F-Droid | entschieden | #9 |
| AE-32 | Lieferkette: Pins, `AUDITS.md`, Trennung von Bau und Signatur | entschieden | #68 |
| AE-33 | Versionierung: App, Format und Suite getrennt | angenommen mit #100 | Protokoll 15 |
| AE-34 | Shamir zurückgestellt | entschieden | #34 |
| AE-35 | Recovery-Wörter mit BIP39-Listen, eigene Kodierung | angenommen mit #100 | #63, `bibliotheken.md` |
| AE-36 | Tier 1 nach dem Test-Standard | entschieden | Test-Standard (Tier 1) |
| AE-37 | Nutzungsversuch: direkt Phase A, dann Phase B | entschieden | #82, #99 |
| AE-38 | Name „Days & Keys", Adresse unter `wirdsichtbar.de` | entschieden | #14 |
| AE-39 | Projekt vollständig auf GitHub, öffentlich | entschieden | Projektleitung, 2026-10-14 (#106) |
| AE-40 | Recovery-Wörter erst mit der Sicherung, nicht in der Einrichtung | entschieden | Projektleitung, 2026-10-17 (#165) |

### AE-01 Lokal zuerst, Zero-Knowledge, kein Generalschlüssel

- **Kontext:** Gesundheitsdaten nach Art. 9 DSGVO; mehrere Personen, die einander nahestehen.
- **Entscheidung:** Daten entstehen und bleiben auf dem Gerät der Person. Was das Gerät verlässt,
  ist mit Schlüsseln verschlüsselt, die nur sie hält. Es gibt keinen Schlüssel für Eltern,
  Administratoren oder Betreiber.
- **Alternativen:** zentraler Dienst mit Serververschlüsselung (Betreiber könnte lesen);
  gemeinsames Konto mit gemeinsamem Schlüssel (widerspricht Leitplanke 2).
- **Folgen:** Wer Gerät und Recovery-Wörter verliert, verliert die Daten (gewollt, Protokoll 1).
  Freigaben brauchen später ein eigenes Format (AE-26).

### AE-02 Lizenz

- **Kontext:** Alles soll Open Source sein. Die iOS-App wird über TestFlight und später den App
  Store verteilt. UniFFI steht unter MPL-2.0. Im Repo liegt noch keine `LICENSE`.
- **Entscheidung (Projektleitung, 2026-10-14):**
  - Kern und Apps: **MPL-2.0**. Änderungen an den Dateien bleiben offen, die Lizenz verträgt
    sich mit der Verteilung über den App Store (Firefox für iOS nutzt sie) und mit UniFFI.
  - Server ab M1.5: **AGPL-3.0**, damit auch ein fremder Betrieb seinen Code offenlegen muss.
  - Dokumentation, Fragenvorlagen, Konzept: **CC BY 4.0**.
- **Alternativen:**
  - GPL-3.0 oder AGPL-3.0 für die Apps: stärkerer Schutz, aber die Bedingungen des App Store
    gelten als unverträglich mit der GPL; das geht nur, solange alle Rechte bei einer Stelle
    liegen (Beiträge nur mit Rechteübertragung, wie bei Signal).
  - Apache-2.0: einfach und mit Patentklausel, aber Abwandlungen dürfen geschlossen werden.
- **Folgen:** `LICENSE` im Wurzelverzeichnis, SPDX-Kennung in jeder Quelldatei, `cargo-deny`
  prüft die Lizenzen der Abhängigkeiten. Vor der ersten Veröffentlichung klären, ob generierter
  Bindungscode unter MPL-2.0 fällt (`AUDITS.md`).

### AE-03 Native App statt Web-App

- **Kontext:** Eine Web-App bekommt ihren Code bei jedem Aufruf vom Server.
- **Entscheidung:** installierte App für Android und iOS.
- **Begründung:** Ein kompromittierter Server könnte einer Web-App gezielt Code unterschieben, der
  Schlüssel abgreift; Hardware-Schlüssel, Planer und Dateizugriff gibt es im Browser nur
  eingeschränkt.
- **Folgen:** iOS-Bau, Signatur und Verteilung über Apple (AE-30). Desktop später, mit eigener
  Oberfläche auf demselben Kern.

### AE-04 Oberfläche nativ in Jetpack Compose und SwiftUI

- **Kontext:** Bisher war eine TypeScript-Oberfläche in einer WebView geplant (Capacitor oder Tauri
  2 Mobile, #33). Teil A verlangt ohnehin native Ansichten für PIN, Recovery-Wörter,
  Bestätigungen, Wiederherstellung und Hilfe sowie nativen Code für Keystore und Secure Enclave,
  Planer mit Neustart-Empfänger, Ordnerzugriff, Backup-Ausschluss und Bildschirmschutz. Für eine
  WebView bleiben nur Formulare, Verläufe und Einstellungen.
- **Entscheidung (Projektleitung, 2026-10-14):** Android mit Kotlin und Jetpack Compose, iOS mit Swift
  und SwiftUI, beide über UniFFI an `dak-core`. Keine WebView in der App.
- **Begründung:**
  - zwei Schichten statt drei; Bridge-Härtung, CSP, Navigationssperre und WebView-Speichertests
    entfallen;
  - etwa die Hälfte der Client-Funde der dritten Erörterung hat ohne WebView keinen Gegenstand
    (CSP-Lücken, `CapacitorHttp`, fremde URLs, WebView-Cache, Masken über der PIN, Entwurf in der
    WebView);
  - eine Bindung erreicht App, Benachrichtigungsempfänger, Planer und später Widgets;
  - belegtes Muster: Firefox und Bitwarden liefern Rust-Kerne per UniFFI an native Apps;
  - Android baut ohne `INTERNET` ohne Sonderweg.
- **Alternativen:**
  - **Flutter mit `flutter_rust_bridge`** (zweite Wahl): eine Oberfläche, keine WebView, Release
    standardmäßig ohne `INTERNET`, UI-Tests unter Linux. Nachteil: zweite Bindung für native
    Teile, Plattform-Ansichten über Channels, Engine im Paket, `flutter_rust_bridge` hängt an
    einer Hauptperson.
  - **Capacitor:** auf iOS reicht `window.open` URLs weiter, `CapacitorHttp` steckt im Kern, volle
    WebView-Angriffsfläche, Lieferkette über npm.
  - **Tauri 2 Mobile:** Mobile noch jung, Tauri 3 schon in Alpha, Widgets nur über Drittanbieter.
- **Gegenargumente:** zwei Oberflächen; die iOS-Hälfte baut und testet nur auf dem Cloud-Mac. Das
  Gegenmittel ist AE-06 (Logik in Rust, Swift-Schicht dünn, Snapshot-Tests je PR).
- **Prüfung:** Prototyp #101 baut den Bildschirm „Status jetzt" in Compose, SwiftUI und als
  Gegenprobe in Flutter. Kostet die native Doppelarbeit deutlich mehr als Flutter, kippt die
  Entscheidung zu Flutter. Lassen sich die UniFFI-Kette ohne Mac für Android, die Kette auf dem
  Cloud-Mac oder der Kernaufruf ohne Oberfläche nicht herstellen, wird neu bewertet, bevor M1a
  beginnt.
- **Folgen bei Zustimmung:** README (Zeile „Clients"), `CLAUDE.md` (Stack), Gesamtkonzept
  Abschnitte 7 und 13, Protokoll Abschnitte 1, 2, 8.4 und 13 werden umgestellt; die Zeile
  „Kompromittierter WebView-Code" in den Garantien entfällt (siehe AE-12).

### AE-05 Rust-Kern `dak-core` mit libsodium

- **Entscheidung:** Alle Kryptografie, Formate, Schlüsselverwaltung und Autorisierung in einem
  Rust-Kern; Primitive ausschließlich aus libsodium über `libsodium-sys-stable`.
- **Begründung:** eine Implementierung für beide Plattformen, prüfbar unter Linux, Speichersicherheit
  in der Hülle; libsodium ist verbreitet, auditiert und schwer falsch zu benutzen.
- **Alternativen:** OpenSSL (große Angriffsfläche, viele gefährliche Optionen,
  `docs/krypto-bibliotheken.md`), RustCrypto (gut, aber verteilt auf viele Crates mit
  unterschiedlichem Prüfstand), Plattform-Krypto je Gerät (zwei Implementierungen),
  noble/libsodium.js in der WebView (entfällt).
- **Folgen:** native Kette mit `cargo-ndk` und XCFramework; Updates nach `AUDITS.md`.

### AE-06 Ansichtsmodelle in Rust (vorläufig)

- **Entscheidung:** `dak-view` berechnet, was angezeigt wird: welche Fragen sichtbar sind, welche
  Bedingung greift, welcher Bezugszeitraum gilt, welche Punkte ein Verlauf hat, welcher Text und
  welcher Digest im Bestätigungsdialog stehen. Kotlin und Swift zeichnen diese Records nur.
- **Begründung:** testbare Logik einmal und unter Linux; beide Oberflächen verhalten sich gleich;
  Agenten schreiben zwei gleichförmige dünne Schichten.
- **Folgen:** Abnahmetests auf Ebene von `dak-view`; eine Regel in der Plattformschicht, die
  Inhalte entscheidet, ist ein Fehler.

### AE-07 Anbindung über UniFFI (vorläufig)

- **Entscheidung:** UniFFI 0.32.2 erzeugt Kotlin- und Swift-Bindungen aus einer Schnittstelle in
  `dak-ffi`.
- **Alternativen:** handgeschriebenes JNI und C-ABI (fehleranfällig), `flutter_rust_bridge` (nur
  bei Flutter).
- **Folgen:** JNA auf Android; generierter Code wird nicht von Hand geändert. Die Schnittstelle
  übergibt Geheimnisse nur über die Rückrufe in Abschnitt 3.

### AE-08 Kein Etebase, kein fremdes Sync-Framework

- **Entscheidung:** eigenes, schlankes Format; für M1.5 eine eigene API.
- **Begründung:** Etebase passt nicht zu feldgenauen Freigaben und widerrufbaren Generationen und
  ist kaum gepflegt (`docs/unterbau-etebase.md`).

### AE-09 Status jederzeit festhalten

- **Entscheidung:** Der Kern ist der Status mit Zeitpunkt, mehrmals am Tag; dazu Ereignisse mit
  einem Tipp, Tagesabschluss und Wochenrückblick als besondere Status, optional Zufallsabfragen.
- **Folgen:** Datenmodell mit `status_id`, `bezug_von`/`bezug_bis`, Antwortzuständen und
  Lebenstag (Protokoll 6); voller Fragenkatalog in `docs/vorlagen.md`; anpassbare Formulare mit
  Messdefinitionen (Protokoll 5).

### AE-10 Kryptosuiten

- **Entscheidung:** jedes verschlüsselte Objekt hat eine Suite-Kennung. `dak-s1`: Ed25519,
  X25519, XChaCha20-Poly1305, BLAKE2b, Argon2id. `dak-s2` vorgesehen mit hybridem X25519 +
  ML-KEM; Bausteine und Zeitpunkt werden mit #63 festgelegt.
- **Folgen:** Suite-Wechsel ist immer eine Rotation (AE-16).

### AE-11 Entsperren

- **Entscheidung:** Die Geräteauthentifizierung (Biometrie oder Geräte-Code) öffnet die Klasse
  Gesundheit. Eine eigene App-PIN schützt nur den hochsensiblen Bereich (ab M1b).
- **Begründung:** Ein zweites Geheimnis für jeden Status kostet an schlechten Tagen zu viel; das
  Restrisiko (wer den Geräte-Code kennt, liest Gesundheitsdaten) ist in den Garantien benannt.
- **Folgen:** Protokoll 8.3 und 8.4; `KK_h` wird bei Aktivierung der PIN rotiert.

### AE-12 Schlüssel nur im Kern

- **Entscheidung:** Geheime Schlüssel existieren im Klartext nur im Speicher von `dak-core`, in
  einer Hülle auf `sodium_malloc` mit Schutzseiten und `zeroize`. Die Plattform bekommt nie
  Schlüssel, nur Ergebnisse.
- **Bekannte Grenze:** Beim Enthüllen von `KK` liefern Keystore und Secure Enclave das Ergebnis
  kurz im Speicher von Kotlin bzw. Swift; die Plattformschicht kopiert es sofort in den Kern und
  überschreibt ihren Puffer, nie als String.
- **Folge von AE-04:** Ohne WebView gibt es in der App keine Codeschicht mehr, der wir weniger
  vertrauen als dem Kern. Die FFI-Grenze dient dann der Ordnung (ein Ort für Schlüssel, ein Ort
  für Prüfungen); die Autorisierung sensibler Aktionen beruht auf der frischen
  Geräteauthentifizierung oder PIN, die die Hardware durchsetzt.

### AE-13 Datensatzformat

- **Entscheidung:** frischer Schlüssel `dk` je Revision, eine Schutzklasse je Datensatz, keine
  Signaturen über Werte, Manifest-MAC über den Bestand (Protokoll 7).
- **Begründung:** Löschen und Hochstufen einzelner Felder ohne Umschlüsseln des Bestands;
  Freigaben in M1.5 können später einzelne `dk` weitergeben.

### AE-14 Kodierung

- **Entscheidung:** deterministisches CBOR nach RFC 8949 4.2.1, unbekannte Felder werden
  abgelehnt, Map-Schlüssel 0 bis 23. **Geändert mit #100:** statt einer Schicht auf `minicbor`
  ein eigener, kleiner Codec in `dak-core` (`cbor.rs`, rund 300 Zeilen). Er kennt nur die
  erlaubten Typen, jede Strenge-Regel steht an einer Stelle, und es fällt eine Abhängigkeit weg
  (Qualitätsziel 6). `ciborium` dient in den Tests als unabhängige Gegenprobe: es muss jede
  Ausgabe lesen und bytegleich zurückschreiben.
- **Alternativen:** `serde_cbor` (nicht mehr gepflegt), `ciborium` (keine Kanonik), `dcbor` direkt
  (eigene Profilregeln, die nicht genau RFC 4.2.1 sind).

### AE-15 Speicher auf dem Gerät

- **Entscheidung:** SQLite über `rusqlite` 0.40.2 mit `bundled` (SQLite ≥ 3.53.4), eine Datei
  im WAL-Modus, Tabellen `WITHOUT ROWID`, Zufalls-IDs, `secure_delete=ON`. Inhalte nur als
  Chiffretext. Für Verläufe verschlüsselte **Verlaufsblöcke** je Lebensmonat unter einem
  Teilschlüssel aus `KK`; hochsensible Indizes nur unter `KK_h`.
- **Alternativen:** SQLCipher (fremde Kryptografie neben libsodium), `sled` (Beta), `fjall` (löscht
  erst bei Kompaktierung), eigene Dateien je Datensatz (viele kleine Dateien, schwer atomar).
- **Grenze:** Auf Flash-Speicher bleiben gelöschte Umhüllungen forensisch auffindbar; die App
  bietet „Löschen und rotieren" an (#62, #66).
- **Folgen:** Laufzeiten (Entschlüsseln eines Jahres, Rotation von etwa 73 000 Umhüllungen in einer
  Transaktion) misst #81/#101; iOS-Datenschutzklassen für Hauptdatei, WAL und Planerdatei klärt
  #67.

### AE-16 Kontokette, Recovery und Rotation

- **Entscheidung:** Gründungsobjekt und Kontokette mit den Gliedern `rotation`, `geraet_ersetzt`,
  `recovery_neu`. Recovery-Schlüssel `RK` getrennt in `RK_sig` und `RK_enc`, kein Gerät hält `RK`.
  Rotation atomar, mit aktivem hochsensiblem Bereich nur mit frischer PIN (Protokoll 4, 9, 10).

### AE-17 Automatische Sicherung

- **Entscheidung:** Die App sichert selbst in einen einmal gewählten Ordner (Android Storage Access
  Framework, iOS Dateien-App mit Bookmark), nach Änderungen gebündelt und mindestens täglich.
  Jede Sicherung wird nach dem Schreiben gelesen und geprüft.
- **Begründung:** kein Datenverlust ohne Server; die Person wählt, wo die verschlüsselte Datei liegt.
- **Folgen:** Wiederherstellung über Ankerregel und Zweigauswahl (Protokoll 10); Restore-Test vor dem
  Nutzungsversuch.

### AE-18 Format der Sicherungsdatei

- **Entscheidung:** `crypto_secretstream` in Blöcken statt eines einzelnen AEAD über die ganze
  Datei; Kapseln `kapsel` und `kapsel_h` an `RK_enc.pub` versiegelt.
- **Begründung:** Speicherbedarf unabhängig von der Dateigröße, abgeschnittene Dateien werden
  erkannt.
- **Folgen:** Formattabelle und Testvektoren in #63.

### AE-19 App ohne Netzzugang

- **Entscheidung:** Android-Build ohne `INTERNET`-Berechtigung, die Pipeline prüft das mit
  `aapt2 dump permissions`. iOS hat keine solche Sperre; die App enthält keinen Netzwerkcode,
  und die Doku sagt das offen.
- **Folgen:** keine Telemetrie (AE-21), Spracherkennung nur offline, Links und Telefonnummern nur
  aus der Hilfe-Ansicht an das System.

### AE-20 Hilfe ohne Entsperren

- **Entscheidung:** eine native Ansicht mit Krisennummern und eigenen Vertrauenspersonen, auch
  im gesperrten Zustand der App erreichbar (Protokoll 13).

### AE-21 Keine Telemetrie, keine Absturzberichte

- **Entscheidung:** keine Analyse-, Werbe- oder Absturz-SDKs, auch nicht über die Plattform
  (Firebase, Sentry, Apple-Absturzberichte nur, wenn die Person sie im System selbst freigibt).
- **Folge:** Fehler meldet die Person selbst mit dem angezeigten Fehlercode; `cargo-deny` und
  eine Gradle-Prüfung verbieten bekannte Telemetrie-Pakete.

### AE-22 Sprache

- **Entscheidung:** Oberfläche und Vorlagen zuerst Deutsch; alle Texte in Ressourcen bzw. als
  Vorlagendaten mit Sprachfeld, keine festen Zeichenketten im Code. Englisch kommt vor einer
  öffentlichen Veröffentlichung.

### AE-23 Barrierefreiheit

- **Entscheidung:** WCAG 2.2 AA sinngemäß; Screenreader-Tests für Tipp-Reihe, PIN-Feld
  (`setAccessibilityDataSensitive`) und Hilfe im Prototyp (#101).

### AE-24 Lokaler Planer

- **Entscheidung:** `dak-plan` plant Erinnerungen und Zufallsabfragen auf dem Gerät, immer nur
  einige Tage voraus (iOS-Grenze 64), mit unverschlüsselter Planerdatei ohne Gesundheitsinhalte;
  höchstens acht Benachrichtigungen am Tag (Protokoll 11).
- **Alternative:** Push vom Server (braucht Netz und verrät Zeitpunkte an Dritte).

### AE-25 Server

- **Entscheidung:** M1 hat keinen Server. Ab M1.5 eine eigene, schlanke API unter
  `https://daysandkeys.wirdsichtbar.de`, die nur Chiffretext und unvermeidbare Metadaten sieht
  (`docs/protokoll-freigaben.md`).
- **Folgen:** Sprache und Betrieb des Servers werden mit M1.5 entschieden; Vorgabe ist AGPL-3.0
  (AE-02) und Betrieb über die Pipeline.

### AE-26 Kein Freigabeformat auf Vorrat

- **Entscheidung:** M1 hat keine Freigaben und kein Vorratsformat. Vor der ersten Freigabe migriert
  das Gerät lokal in das dann geprüfte Format. M1 ist geteilt in M1a (Gehgerüst und Versuch) und
  M1b (Ausbau während des Versuchs).

### AE-27 Beschreibende Auswertung

- **Entscheidung:** Punkte, Spannen, Fallzahlen, Lücken; keine automatischen Zusammenhänge, keine
  Farbwertung, keine geglättete Wirkungskurve, keine Empfehlungen (#74).
- **Folgen:** Diagramme mit Swift Charts und Vico 3.3.1; die Daten dafür kommen fertig aus
  `dak-view`.

### AE-28 Arzt-PDF

- **Entscheidung:** `dak-report` erzeugt das PDF im Kern mit `krilla` 0.8.2 und der Schrift Inter
  4.2, damit beide Plattformen dasselbe Dokument liefern und es unter Linux getestet wird (Rendern
  mit `hayro` im Test).
- **Alternativen:** PDF-Erzeugung der Plattform (zwei Implementierungen), `genpdf` (still seit
  2021).

### AE-29 Monorepo

- **Entscheidung:** Kern, Android, iOS, Vorlagen und Doku in einem Git-Repo
  (Struktur in Abschnitt 6).
- **Begründung:** Eine Formatänderung berührt Protokoll, Kern und beide Apps; ein PR prüft alles
  zusammen. Eine CI-Pipeline (GitHub Actions) mit Stufen je Teil.

### AE-30 iOS-Bau in der Cloud

- **Entscheidung (Projektleitung, 2026-10-14):** iOS baut auf den macOS-Runnern von GitHub Actions im
  selben Repo (AE-39); ein eigener Cloud-Mac-Dienst entfällt. Signaturdaten (App-Store-Connect-API-
  Schlüssel) liegen als verschlüsselte Secrets der Umgebung `release`, nur der Release-Lauf auf
  `main` bzw. einem Tag bekommt sie. Verteilung über TestFlight mit Notfallplan für den Ablauf nach
  90 Tagen (#8, #78). Apple-Konto der Blackfort Technology UG (#10).
- **Begründung:** ein Ort für Code, Issues und CI; keine zweite Plattform mit eigenem Zugang.
  Öffentliche Repos bekommen die macOS-Runner ohne Minutenkontingent.
- **Folgen:** Hash des IPA in der CI protokollieren; Bau nur aus signierten Tags bzw. geprüften
  PR-Ständen.

### AE-39 Projekt auf GitHub, öffentlich

- **Entscheidung (Projektleitung, 2026-10-14):** Repo, Issues, Pull Requests und CI liegen vollständig in
  `https://github.com/Blackfort-Technology/days-and-keys`, das Repo ist öffentlich. Die Issues
  #1 bis #105 sind mit gleichen Nummern aus dem früheren Tracker übernommen.
- **Begründung:** Open Source ist Ziel (AE-02); Branch-Schutz mit Pflicht-Checks und die
  macOS-Runner gibt es für öffentliche Repos ohne Zusatzkosten.
- **Folgen:** Die öffentliche Fassung beginnt mit einer neuen Historie. Die Erörterungsprotokolle,
  PDFs und das Mockup bleiben im nicht öffentlichen Archiv; ihre Befunde stehen als Issues. Keine
  echten Personen- oder Gesundheitsdaten in Repo, Tests, Issues und CI-Protokollen
  (Invariante 5). Jede Änderung läuft über einen PR mit grünen Pflicht-Checks.

### AE-31 Android-Verteilung

- **Entscheidung:** signierte APK aus der CI (GitHub Actions) für den Versuch; später F-Droid mit reproduzierbarem
  Build (#9). Der Signaturschlüssel liegt in einem Secret-Store mit gesicherter Zweitkopie, weil
  Updates ohne denselben Schlüssel nicht installierbar sind.
- **Folge:** Ohne Netz prüft die App nicht selbst auf Updates; im Versuch werden neue APKs von
  Hand installiert.

### AE-32 Lieferkette

- **Entscheidung:** alle Abhängigkeiten gepinnt (`Cargo.lock`, Gradle-Lockfiles, Swift Package
  Resolved), `AUDITS.md` für alles Sicherheitsrelevante, `cargo-deny` und `cargo-audit` in der
  Pipeline, Bau und Signatur getrennt (#68). Updates nur mit gelesenen Release- und
  Security-Hinweisen und grünen Testvektoren.

### AE-33 Versionierung

- **Entscheidung:** drei getrennte Zähler: App-Version (SemVer), `formatversion` je Objekt,
  Suite-Kennung. Neue Formatversionen kommen in zwei Schritten (erst lesen, dann schreiben);
  in Phase A des Versuchs keine Formatänderung (Protokoll 15). Das Protokoll hat eine eigene
  Dokumentversion (derzeit v0.7.1).

### AE-34 Shamir zurückgestellt

- **Entscheidung:** keine Aufteilung der Recovery-Wörter in M1. Kommt sie später, dann nativ im
  Kern und mit geprüfter Bibliothek (#34).

### AE-35 Recovery-Wörter

- **Entscheidung:** eigene, kleine BIP39-Kodierung im Kern (Entropie plus Prüfsumme auf
  Wortliste), englische Liste als Standard, deutsche Liste `de-2048` von dys2p wählbar; Listen mit
  SHA-256 in `docs/format.md`. Die Crate `bip39` nur als Orakel in Tests.
- **Begründung:** wenige Zeilen, volle Kontrolle über Speicher und Löschen; BIP39-Listen sind
  erprobt und gut lesbar.

### AE-36 Tier 1 nach dem Test-Standard

- **Entscheidung:** Days & Keys verarbeitet Gesundheitsdaten und gilt als Tier 1:
  Security-Regressionstests Pflicht, Zielkorridor 80 % Abdeckung, Testvektoren, Negativtests,
  Fuzzing, Hardware-Tests auf echten Geräten (#80).

### AE-37 Nutzungsversuch

- **Entscheidung:** mit M1a direkt Phase A (drei Wochen, Stand eingefroren), dann Phase B (drei
  Wochen, M1b-Teile an der Phasengrenze). Freigaben und Mehrgeräte nur, wenn der Versuch den Bedarf
  zeigt (#82, #99).

### AE-38 Name und Adresse

- **Entscheidung:** „Days & Keys"; Server ab M1.5 unter `https://daysandkeys.wirdsichtbar.de`
  (#14). Eigene Domains werden vorerst nicht registriert; vor einer Veröffentlichung Marke und
  Domains neu prüfen.

### AE-40 Recovery-Wörter erst mit der Sicherung

- **Entscheidung:** Die Einrichtung kommt ohne Recovery-Wörter aus. Der Recovery-Schlüssel der
  Gründung signiert nur die Gründung und wird beim Abschluss verworfen, ohne je angezeigt oder
  gespeichert zu werden (Meta `woerter_offen`). Die Wörter entstehen erst mit der Sicherung, in der
  Einrichtung („Jetzt einrichten“) oder später unter Mehr › Sicherung: Glied `recovery_neu`, vom
  Gerät signiert, `kapsel` und `kapsel_h` an den neuen `RK_enc` gebunden, Bund und Bund_h an den
  neuen Kettenkopf. Keine Rotation, weil der erste Recovery-Schlüssel nie bekannt war; bekannte
  Wörter zu ersetzen bleibt ausgeschlossen. Solange die Wörter fehlen, erzeugt der Kern keine
  Sicherung, und „Heute“ erinnert erst ab dem zweiten Tag, aufschiebbar um eine Woche.
- **Begründung:** 24 Wörter und eine Abfrage vor dem ersten Eintrag waren die größte Hürde der
  Einrichtung (Projektleitung, 2026-10-17). Ohne Sicherung nützen die Wörter nichts, ohne Wörter
  lässt sich keine Sicherung öffnen; beides gehört in einen Schritt.
- **Folge:** Geht das Gerät verloren, bevor Wörter und Sicherung eingerichtet sind, sind die
  Einträge verloren, wie beim Überspringen der Sicherung. iOS behält den bisherigen Ablauf bis zum
  Nachzug (#137); die bisherige Schnittstelle bleibt.

## 9. Offene Entscheidungen

Die drei Entscheidungen vor dem Prototyp sind getroffen (Projektleitung, 2026-10-14, #106):
Oberfläche nativ (AE-04), Lizenz MPL-2.0/AGPL-3.0/CC BY 4.0 (AE-02), iOS-Bau über GitHub Actions
(AE-30, AE-39). Die übrigen vorgeschlagenen Einträge (AE-14, 15, 18, 21, 22, 23, 28, 29, 33, 35)
gelten mit dem Start von #100 als angenommen; AE-14 wurde dabei geändert (eigener Codec).

## 10. Risiken und bekannte Schulden

| Risiko | Umgang |
|---|---|
| Zwei native Oberflächen laufen auseinander | AE-06, gemeinsame Abnahmetests im Kern, Snapshot-Tests |
| iOS-Rückmeldung nur über die macOS-Runner | iOS-Lauf bei Änderungen an `ios/` und `core/`, Snapshot-Tests |
| Kritische Bibliotheken mit wenigen Personen (`minicbor`, `krilla`, `libsodium`) | Pins, Testvektoren, Gegenprobe mit `dcbor`, Austauschbarkeit hinter eigener Hülle |
| Forensische Reste auf Flash | ehrlich benennen, „Löschen und rotieren" (#62) |
| Bedrohungsmodell bezieht sich noch auf v0.1 | Neufassung gegen Teil A und dieses Dokument (#66) |
| Mockup zeigt noch das alte Konzept 1.0 | nach der Framework-Entscheidung neu erzeugen |
