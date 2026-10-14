# Krypto- und Datenprotokoll, Teil A: Kern für M1 (Entwurf v0.7.1)

Stand 2026-10-08, Issue #3. v0.6 entstand nach der zweiten Erörterung und der Entscheidung #96;
v0.7 setzt die Befunde „vor dem Prototyp" der dritten Erörterung
(Erörterung vom 2026-10-06, nicht öffentliches Archiv; Befunde als Issues) und die Entscheidungen
#99 um; v0.7.1 korrigiert die Widersprüche aus dem Abgleich vom 2026-10-08 (nicht öffentliches
Archiv). Der Nutzungsversuch startet direkt mit Phase A, iOS wird per Cloud-Bau gebaut (#99).
**Protokoll v1.0 umfasst nur diesen Teil A:** eine persönliche, lokale App, mit der eine Person ihren Status jederzeit
festhält, ein schreibendes Gerät je Person, automatische verschlüsselte Sicherung und
Wiederherstellung ohne Server.

**Kein Freigabeformat auf Vorrat (#96):** Teil A legt nur fest, was sich später schwer ändern lässt.
Das Schlüsselformat für Freigaben (Monats- und Tagesketten, Freigabepakete) kommt mit Teil B
(`docs/protokoll-freigaben.md`, M1.5); vor der ersten Freigabe werden alle Daten lokal umgeschlüsselt
(Abschnitt 15).

v1.0 setzt voraus: Wegwerf-Prototyp (#81), bytegenaue Formattabellen mit Testvektoren
(`docs/format.md`, #63) und ein Bedrohungsmodell gegen diesen Teil (#66).

## 1. Umfang und Garantien

| Angreifer oder Lage | geschützt | nicht geschützt |
|---|---|---|
| Wer eine Sicherungsdatei hat (Ordner, Cloud, Mail) | alle Inhalte, Feldnamen, Zeiten, Zahl und Art der Einträge | Dateigröße, Zeitpunkt der Sicherung |
| Dieb mit gesperrtem Gerät | alle Inhalte (Hardware-Wrapping) | Existenz der App |
| Person, die den Geräte-Code kennt oder die Biometrie erzwingt | Inhalt und Bezeichnungen hochsensibler Einträge (App-PIN, Abschnitt 8); sie kann hochsensible Einträge nur vorläufig anlegen | alle Gesundheitsdaten (Entscheidung #99: Geräteentsperrung genügt); sie kann Einträge der Klasse Gesundheit anlegen und Sicherungen erzeugen |
| Erzwungene Herausgabe der App-PIN | nichts | alles; eine rechtliche, keine technische Frage |
| Auslesen des Geräts mit forensischen Werkzeugen | Inhalt hochsensibler Einträge, solange der Hardware-Schlüssel nicht gebrochen ist (Abschnitt 8.4) | Zahl und Zeitpunkte hochsensibler Einträge; alle Gesundheitsdaten, wenn der Geräte-Code bekannt ist oder der Hardware-Schutz gebrochen ist |
| Kompromittierter WebView-Code bei entsperrter App | Schlüssel, hochsensible Einträge ohne frische PIN, Export und Sicherung ohne native Bestätigung | alle Daten, die die Oberfläche in dieser Sitzung anzeigen darf |
| Altes Gerät nach Wiederherstellung auf einem neuen | alle Daten, die nach der Wiederherstellung entstehen (Rotation, Abschnitt 9) | was es bis dahin hatte; solange es Zugang zum Sicherungsordner hat, kann es dort Dateien schreiben und löschen (Abschnitt 10.2) |
| Manipulierter Build oder nativer Code | nichts | alles (Lieferkette, #68) |
| Einträge ohne Entsperren (Entwürfe, später Widgets) | sie zählen erst nach Durchsicht in der entsperrten App (Abschnitt 6.2) | wer das Gerät in der Hand hat, kann vorläufige Einträge anlegen |
| Verlust von Gerät und Recovery-Wörtern | nichts | alle Daten (gewollt) |

Nicht-Ziele in M1: Nachweise gegenüber Dritten, Abstreitbarkeit gegenüber Dritten (es wird nichts
geteilt), Erkennung, dass eine Sicherung nicht die neueste ist, über die im Ordner vorhandenen
Sicherungen hinaus (Abschnitt 10.3).

## 2. Bausteine und Kryptosuiten

Ausschließlich libsodium über den Rust-Kern `dak-core` (Abschnitt 13); keine Nachbauten, keine
Kryptografie in der WebView. Jedes verschlüsselte Objekt hat eine **Suite-Kennung**.

| Suite | Signatur | Asymmetrisch | Symmetrisch | Hash/KDF | Passwort |
|---|---|---|---|---|---|
| `dak-s1` (v1.0) | Ed25519 | X25519, `crypto_box_seal` bzw. `crypto_box` | XChaCha20-Poly1305 IETF, 24-Byte-Zufallsnonce | BLAKE2b, `crypto_kdf` | Argon2id (`crypto_pwhash`) |
| `dak-s2` (vorgesehen) | Ed25519 | hybrid X25519 + ML-KEM | wie s1 | wie s1 | wie s1 |

Ein Suite-Wechsel ist immer eine **Rotation** (Abschnitt 9): neue Schlüssel, nicht nur neue
Umhüllungen. Bereits abgegriffene s1-Umhüllungen öffnen dann nur alte Daten.

## 3. Kodierung, Versionierung, Erweiterung

- Deterministisches CBOR (RFC 8949, 4.2.1), ganzzahlige Map-Schlüssel, keine doppelten Schlüssel,
  keine unbestimmten Längen, Höchstgrößen je Objektart (`docs/format.md`).
- Jedes Objekt beginnt mit `typ`, `formatversion`, `suite`. **Unbekannte Typen, Felder oder Suiten
  in sicherheitsrelevanten Objekten werden abgelehnt.** Erweiterungen gibt es nur über eine neue
  `formatversion` mit definierter Migration (Abschnitt 15); reservierte Felder gibt es nicht.
- **Schlüssel werden nie für zwei Zwecke benutzt.** Aus jedem Wurzelschlüssel werden Teilschlüssel
  per `crypto_kdf` mit festem Kontext und fester Subkey-ID abgeleitet; die Wurzel selbst
  verschlüsselt nichts. Tabelle in `docs/format.md`.
- Separatoren für alles, was gehasht, signiert oder als AD gebunden wird: `dak-gruendung-v1`,
  `dak-kontokette-v1`, `dak-bund-v1`, `dak-datensatz-v1`, `dak-sicherung-v1`, `dak-kapsel-v1`,
  `dak-messdef-v1`, `dak-abfrage-v1`.

## 4. Identität und Kontokette

Ein Konto entsteht durch ein **Gründungsobjekt**, signiert mit `DK_sig` (Gerät) und `RK_sig`
(Recovery). `konto_id = BLAKE2b-256(Gründungsobjekt)`. Es enthält `DK_sig.pub`, `DK_enc.pub`,
`RK_sig.pub`, `RK_enc.pub`.

- `RK` (256 Bit, 24 Wörter) ergibt per `crypto_kdf` zwei getrennte Paare: `RK_sig` (Autorisierung)
  und `RK_enc` (Verschlüsselung der Recovery-Kapsel). Kein Gerät hält `RK` nach dem Einrichten.
- Jede spätere Änderung ist ein Glied der **Kontokette** (`dak-kontokette-v1`) mit laufender
  Nummer, Vorgänger-Hash und Signatur. Glieder in M1:

| Glied | Inhalt | signiert von |
|---|---|---|
| `rotation` | neue Schlüsselgeneration `k` (Gesundheit) und gegebenenfalls `k_h` (hochsensibel), Grund | aktuelles Gerät |
| `geraet_ersetzt` | neues Gerät, Nummer des letzten gültigen Glieds des alten Geräts | `RK_sig` und neues Gerät |
| `recovery_neu` | neue `RK_sig.pub`, `RK_enc.pub` | aktuelles Gerät nach frischer Geräteauthentifizierung und nativer Bestätigung (alte Wörter nicht nötig); ist der hochsensible Bereich aktiv, zusätzlich die App-PIN für dessen Rotation |

- Nach `geraet_ersetzt` sind Signaturen des alten Geräts mit höherer Gliednummer ungültig.
- Nach `recovery_neu` folgt immer eine Rotation; die alten Wörter öffnen danach nur noch alte
  Sicherungen (#86).
- Die Kontokette steht in jeder Sicherung und auf dem Gerät; ihr Kopf ist Teil der AD aller
  Schlüsselbund- und Sicherungsobjekte.
- **Ankerregel (Wiederherstellung):** Die aus den eingegebenen Wörtern abgeleiteten `RK_sig.pub` und
  `RK_enc.pub` müssen den am Kettenkopf gültigen Recovery-Schlüsseln entsprechen; `konto_id` wird aus
  dem Gründungsobjekt neu berechnet; die Felder der Kapsel müssen mit dem Innenteil übereinstimmen.
  Sonst bricht die Wiederherstellung vor jeder Übernahme ab (#87).
- Signaturen gibt es nur auf Gründungsobjekt und Kettengliedern.

## 5. Messdefinitionen und Formulare

- Eine **Messdefinition** (`dak-messdef-v1`) legt Frage, Feldtyp, Skala mit Ankern, Einheit,
  Polung, Bezugsart (jetzt, seit dem letzten Status, Tag, Woche) und Schutzklasse fest. Sie ist
  unveränderlich und hat eine zufällige `messdef_id`.
- Formulare ordnen Messdefinitionen und haben Anzeigenamen. Bedeutungs-, Skalen-, Polungs- oder
  Bezugsänderung erzeugt eine neue Messdefinition; die Auswertung vergleicht alte und neue Reihe
  nicht (#74).
- Messdefinitionen und Formulare der Klasse hochsensibel liegen im hochsensiblen Bereich
  (Abschnitt 8.2), einschließlich der Bezeichnungen von Substanzen.
- **Bedingungen** sind eine feste Ausdrucksform über: Antworten im selben Status, ob heute eine
  Einnahme bestätigt ist, Zeit seit dem letzten Status, Wochentag. Hochsensible Antworten dürfen
  keine Bedingung einer Frage niedrigerer Klasse sein (#6).

## 6. Status, Ereignisse, Abfragen und Zeit

### 6.1 Objekte

| Objekt | Inhalt (verschlüsselt) |
|---|---|
| **Status** | `status_id`, `zeitpunkt`, `ausloeser` (selbst, Abfrage mit `abfrage_id`, nach Ereignis), `erfasst_ueber` (App, Entwurf, später Widget oder Benachrichtigung), `vorlaeufig`, `bezug_von`, `bezug_bis`, Liste der Antworten |
| **Antwort** | `messdef_id`, `zustand` (beantwortet, übersprungen, kam nicht vor, nicht gezeigt mit Grund: Bedingung, ausgeblendet, Block zu), Wert, `aktiv_gesetzt` |
| **Ereignis** | `ereignis_id`, Art, `zeitpunkt`, Angaben (z. B. Präparat und Dosis bei Einnahme) |
| **Abfrage** | `abfrage_id`, `geplant_um`, `ausgeloest_um`, Ergebnis (beantwortet mit `status_id`, verpasst, abgelehnt, verspätet beantwortet) |
| **Rückblick** | Tagesabschluss oder Wochenrückblick als Status mit Bezug auf Lebenstag bzw. Woche |

- `bezug_von` ist das Ende des letzten Status, in dem der Block „seit dem letzten Status" geöffnet
  und abgeschlossen wurde, frühestens das letzte „Aufgewacht", höchstens zwölf Stunden zurück;
  Status aus Zufallsabfragen verschieben ihn nicht. Die Zwei-Stunden-Schwelle misst denselben
  Abstand. `bezug_von` und `bezug_bis` werden gespeichert und angezeigt; bei Kappung heißt es „in den
  letzten zwölf Stunden" (#90).
- „Kam nicht vor" zählt in Häufigkeiten nicht mit und wird als eigene Zahl gezeigt; Häufigkeiten
  erscheinen nur zusammen mit der Fensterlänge.
- Verpasste Abfragen werden gespeichert, damit die Auswertung die Antwortquote kennt (#93).

### 6.2 Einträge ohne Entsperren

- Erlaubt sind nur Flächen aus einer festen Liste. **M1a: nur Entwürfe** (beim Sperren gesichert);
  Widgets, Kurzbefehle und schreibende Benachrichtigungen erst nach Messung im Prototyp (#101).
- Solche Einträge haben `vorlaeufig = ja` und `erfasst_ueber`. Sie zählen nicht als bestätigte
  Einnahme, lösen keine Wirkungsfrage aus und erscheinen beim nächsten Öffnen gesammelt zur
  Übernahme oder zum Verwerfen (#89).
- **Medikationsereignisse nie ohne Entsperren der App.**
- `dak-core` prüft, dass eine `messdef_id` zur Klasse des Schreibschlüssels passt.
- `erfasst_ueber` und `vorlaeufig` stehen im verschlüsselten Objekt und sind damit durch AEAD
  geschützt; Übernahme nach Durchsicht ist eine neue Revision.

### 6.3 Zeit

- `zeitpunkt` als UTC-Zeit plus UTC-Versatz und IANA-Zeitzone zum Zeitpunkt der Erfassung;
  `erfasst_um` getrennt davon (Nachträge).
- **Lebenstag:** Jede Person legt ihren Tageswechsel fest (Standard 4:00 Uhr). Ein Status um 1:30
  Uhr gehört zum Vortag. Der Lebenstag wird beim Erfassen berechnet und gespeichert, damit spätere
  Änderungen der Einstellung nichts verschieben (#92).
- Kein Startdatum im Format: Nachträge und Daten beliebiger Jahre sind möglich.
- Zeitzonenwechsel und Sommerzeit verschieben gespeicherte Zeitpunkte nicht; Einnahmepläne laufen
  in Ortszeit, bei einem Zeitzonenwechsel fragt die App (#42).

## 7. Datensätze, Revisionen und Löschung

### 7.1 Datensatz

Jedes Objekt aus Abschnitt 6 ist ein **Datensatz** mit eigenem Zufallsschlüssel `dk` (32 Byte):

```
datensatz = { formatversion, suite, datensatz_id, revision_id, vorgaenger_revision_id,
              art, klasse, umhuellung, nonce, chiffrat }
klartext  = sodium_pad(cbor(objekt), block)
ad        = "dak-datensatz-v1" || konto_id || datensatz_id || revision_id
            || vorgaenger_revision_id || art || klasse || formatversion
chiffrat  = XChaCha20-Poly1305(dk, nonce, klartext, ad)
```

- **Jede Revision hat einen frischen Zufalls-`dk`**; ein `dk` verschlüsselt genau ein Chiffrat.
- **Ein Datensatz enthält nur Inhalte einer Schutzklasse.** Ein Status mit Antworten verschiedener
  Klassen wird in klassengleiche Datensätze mit gemeinsamer `status_id` zerlegt; der Teil niedrigerer
  Klasse enthält keinen Hinweis auf den höheren.
- IDs sind 128 Bit Zufall; zeitgeordnete ID-Formate sind verboten.
- `umhuellung` ist `dk`, verschlüsselt mit dem Datenschlüssel der Klasse und Schlüsselgeneration
  (Abschnitt 8) bzw. bei hochsensiblen Einträgen an den Schreibschlüssel versiegelt.
- Im Klartext stehen nur IDs, `art` (Datensatz, Korrektur, Löschung), `klasse` und `formatversion`.
  Zeit, Objektart (Status oder Ereignis), Messdefinitionen und Werte sind verschlüsselt.
- **Keine Signaturen über Werte** (#64, #96). Integrität entsteht durch AEAD und das authentisierte
  Manifest (Abschnitt 10).

### 7.2 Revisionen und Löschung

- Korrekturen sind neue Revisionen mit Verweis auf die vorige.
- **Löschen** schreibt eine Löschrevision (Tombstone) und entfernt im aktiven Bestand Chiffrat und
  Umhüllung aller Revisionen des Datensatzes. Ohne Umhüllung ist der Inhalt auch mit allen
  Kontoschlüsseln nicht mehr zu öffnen (#62).
- **Sicherungen** enthalten gelöschte Inhalte, bis sie aus der Aufbewahrung fallen (Abschnitt
  10.2). Die App sagt beim Löschen, ab wann der Inhalt auch aus den Sicherungen verschwunden ist.
- Tombstones bleiben dauerhaft; jede Wiederherstellung wendet sie an.

## 8. Schutzklassen und Schlüssel

### 8.1 Klassen

| Klasse | Standard für | Lesen | Schreiben | Arzt-PDF |
|---|---|---|---|---|
| Gesundheit | alle Felder einschließlich Freitext und neuer Felder | App entsperrt über die Geräteauthentifizierung | App entsperrt; vorläufig über Entwürfe (6.2) | ja |
| hochsensibel | Substanzen, Craving, auf Wunsch weitere | frische App-PIN | ohne PIN über den Schreibschlüssel, dann **vorläufig** bis zur Durchsicht nach PIN-Eingabe | nur nach Auswahl je Export |
| normal | nur nach ausdrücklichem Herabstufen | App entsperrt | App entsperrt | ja |

Herab- und Hochstufen braucht eine native Bestätigung (M1b). **Hochstufen** schreibt jeden
vorhandenen Datensatz des Feldes als neue Revision mit neuem `dk` in der höheren Klasse und entfernt
die alten Revisionen aus dem aktiven Bestand; alte Sicherungen enthalten sie bis zum Ende der
Aufbewahrung (#70).

### 8.2 Schlüssel je Klasse

- **Gesundheit:** symmetrischer Datenschlüssel `DS_g,k` je Schlüsselgeneration `k`, dazu ein
  Schreibschlüsselpaar `WS_g` (X25519). `WS_g.pub` erlaubt das Schreiben ohne Entsperren (in M1a
  nur Entwürfe, Abschnitt 6.2); solche Datensätze bleiben vorläufig und werden nach Durchsicht in
  der entsperrten App unter `DS_g,k` umgehüllt.
- **Hochsensibel:** Schreibschlüsselpaar `WS_h` (X25519). Jeder hochsensible Datensatz hat sein
  `dk` an `WS_h.pub` versiegelt; Schreiben braucht keine PIN (#89). `WS_h.sec`, die hochsensiblen
  Messdefinitionen und die Bezeichnungen liegen im **hochsensiblen Bund** (`KK_h`).
- Hochsensible Datensätze liegen in einem eigenen Speicherbereich; im Gesundheits-Bereich steht
  nichts über sie (#88). Indizes über hochsensible Inhalte gibt es nur unter `KK_h`. Die Startseite
  zeigt für Substanzen nur Platznummern mit selbst gewählten, neutralen Bezeichnungen; die
  eigentlichen Namen sieht man nach der PIN, im Arzt-PDF steht immer der echte Name.
- Ein hochsensibler Tipp lässt sich wenige Sekunden lang rückgängig machen. Ohne frische PIN
  angelegte hochsensible Einträge sind vorläufig; nach der PIN werden sie zuerst zur Übernahme
  oder zum Verwerfen gezeigt, mit dem echten Namen der Substanz.
- Restrisiko, ehrlich benannt: Bei forensischer Auslesung des Geräts sind Zahl und Zeitpunkte
  hochsensibler Einträge sichtbar, Inhalt und Bezeichnungen nicht.

### 8.3 Schlüsselbund und Entsperren

- Der **Bund** (`dak-bund-v1`) enthält Datenschlüssel, Schreibschlüssel, Formulare, Messdefinitionen
  und den Planungsstand. Er ist versioniert, an `konto_id` und den Kontokettenkopf gebunden und mit
  einem Teilschlüssel aus `KK` verschlüsselt; der hochsensible Bund ebenso mit `KK_h`.
- **`KK` (Klasse Gesundheit) öffnet die Geräteauthentifizierung** (Entscheidung #99): ein
  Hardware-Schlüssel, nutzbar nach Biometrie **oder** Geräte-Code (iOS: Keychain-Zugriffsregel
  „Benutzer anwesend", `WhenUnlockedThisDeviceOnly`; Android: Keystore-Schlüssel mit
  Benutzerauthentifizierung, Biometrie stark oder Geräte-Code, nicht an Biometrie-Neueinrichtung
  gebunden). Eine Neueinrichtung der Biometrie ändert nichts, weil der Geräte-Code ohnehin öffnet.
- **`KK_h` (hochsensibel) hat einen eigenen Generationszähler `k_h`.**
  - **M1a:** Der hochsensible Bereich ist leer und nicht aktiv. `KK_h` wird bei der Einrichtung
    erzeugt und wie `KK` über die Geräteauthentifizierung umhüllt; `bund_h` ist leer. So schreibt
    M1a das vollständige Format ohne PIN.
  - **Aktivierung (M1b):** Beim nativen Einrichten der App-PIN wird `KK_h` rotiert (`k_h + 1`,
    neue `WS_h`), unter den PIN-Weg (Abschnitt 8.4) gelegt und die alte Umhüllung gelöscht. Ab dann
    öffnet nur die App-PIN mit frischer Eingabe `KK_h`.
- Beide sind zusätzlich in der Recovery-Kapsel (Abschnitt 10).

### 8.4 App-PIN

- Die App-PIN schützt nur den hochsensiblen Bereich (ab M1b).
- Eingabe ausschließlich in einer nativen Ansicht, nie in der WebView; ebenso Anzeige und Eingabe
  der Recovery-Wörter (#60).
- **Konstruktion:** `x = crypto_pwhash(PIN)`, dann `kek = HMAC_hw(x)` mit einem nicht exportierbaren
  Hardware-Schlüssel (Android: HMAC im Keystore, StrongBox vor TEE, Software-Keystore wird angezeigt;
  iOS siehe unten), Zugriffsregel „nur bei entsperrtem Gerät"; `KK_h` ist mit `kek` umhüllt. Wer `x`
  raten will, braucht den Hardware-Schlüssel bei jedem Versuch.
- **iOS:** Die Secure Enclave kann nur P-256 und kein HMAC. Aus `x` wird ein P-256-Skalar
  `s = H2S(x)` abgeleitet und der öffentliche Punkt `P = s·G` berechnet; die Secure Enclave rechnet
  `z = ECDH(SE_priv, P)`, und `kek = BLAKE2b(z, Kontext)`. Weil `P` von der PIN abhängt, braucht jeder
  Rateversuch eine Operation der Secure Enclave. Ob das auf den Zielgeräten so funktioniert, misst
  der Prototyp (#101).
- Mindestens sechs Ziffern oder eine Passphrase; Argon2id-Parameter in `docs/format.md` (Ziel etwa
  eine halbe Sekunde auf einem Mittelklassegerät).
- Fehlversuche begrenzt die App mit wachsender Wartezeit. Gegen einen Angreifer, der die
  Hardware-Umhüllung gebrochen hat, schützt nur die Länge der PIN; die App sagt das beim Einrichten.
- PIN ändern verlangt die alte PIN.

## 9. Schlüsselrotation

Eine Rotation (#86) erzeugt atomar neue Schlüssel. Es gibt zwei Generationszähler: `k` für die
Klasse Gesundheit und `k_h` für den hochsensiblen Bereich.

1. neuer `KK`, neue `DS_g,k+1`, neuer `WS_g`; alle Umhüllungen von Gesundheits-Datensätzen werden neu
   geschrieben (die Datensätze bleiben, nur `dk` wird neu umhüllt); vorläufige Einträge an `WS_g`
   werden vorher zur Durchsicht gestellt und danach mit umhüllt;
2. **ist der hochsensible Bereich aktiv, rotiert er mit:** Die Rotation verlangt dann die frische
   App-PIN. Neuer `KK_h` (`k_h + 1`), neuer `WS_h`; alle hochsensiblen `dk` werden an den neuen
   `WS_h.pub` versiegelt, danach wird der alte `WS_h.sec` gelöscht. Ohne PIN findet keine Rotation
   statt, die App fragt nach;
3. neue Kapseln (`kapsel` immer, `kapsel_h` wenn `k_h` sich ändert), neues Glied `rotation`;
4. erst nach vollständigem Schreiben und Prüfen wird umgeschaltet; alte Schlüssel werden gelöscht.

Auslöser: Wiederherstellung auf einem neuen Gerät (Abschnitt 10.3), `recovery_neu`, Suite-Wechsel,
auf Wunsch („ich vermute, jemand hatte Zugriff"), Aktivierung der App-PIN (nur `k_h`). Wer alte
Schlüssel hat (altes Gerät, alte Sicherung mit alten Wörtern), liest danach keine neuen Datensätze
und keine neuen Sicherungen. Alte Sicherungen bleiben mit alten Schlüsseln lesbar, bis sie aus der
Aufbewahrung fallen.

## 10. Sicherung und Wiederherstellung

### 10.1 Sicherungsdatei

```
außen:   formatversion, suite, kapsel, kapsel_h          (feste Größen)
kapsel   = crypto_box_seal( cbor{ "dak-kapsel-v1", konto_id, generation k, KK }, RK_enc.pub )
kapsel_h = crypto_box_seal( cbor{ "dak-kapsel-h-v1", konto_id, generation_h k_h, KK_h }, RK_enc.pub )
           einmal je Generation k_h erzeugt (bei Einrichtung, Aktivierung der PIN und Rotation
           von k_h liegt KK_h vor) und unverändert in jede Sicherung dieser Generation kopiert
innen:   XChaCha20-Poly1305( KDF(KK, "daksich_"), nonce,
                             cbor{ manifest, kontokette, bund, bund_h, datensaetze, tombstones },
                             ad = "dak-sicherung-v1" || formatversion || konto_id )
manifest = { konto_id, kettenkopf, generation k, generation_h k_h, geraet_id, sicherung_nr, erstellt_um, Hashes aller Datensätze,
             Tombstones, bund, bund_h, kapsel, kapsel_h }
           mit MAC aus einem eigenen Teilschlüssel von KK (keine Signatur, #64)
```

`bund_h` und hochsensible Datensätze werden als Chiffrat kopiert; die Sicherung braucht deshalb
keine PIN. Ablauf (#87): Wörter → `RK_enc` → `kapsel` öffnen → Innenteil entschlüsseln → Ankerregel
(Abschnitt 4) → Kontokette prüfen → Manifest-MAC und Hashes prüfen → `kapsel_h` öffnen und gegen
den Hash im Manifest und `generation_h` prüfen → erst dann übernehmen. Wer den Geräte-Code eines
aktuellen Geräts hat, kann gültige Sicherungen erzeugen; das ist eine Folge von #99 (Abschnitt 1).

### 10.2 Automatische Sicherung

- Beim Einrichten wählt die Person einmal einen **Sicherungsordner** (Android: Storage Access
  Framework, iOS: Ordner in der Dateien-App mit dauerhaftem Zugriff). Er kann in einer Cloud liegen;
  die Dateien sind ohnehin verschlüsselt.
- Die App schreibt in entsperrten Sitzungen nach Änderungen, spätestens täglich beim Öffnen, eine
  neue Sicherung. `sicherung_nr` zählt je Kontokette und läuft nach einer Wiederherstellung weiter
  (#91).
- Dateinamen beginnen mit einem Kürzel der `konto_id`, dann Gerätekennung und Generation. Die
  Aufbewahrung löscht nur Dateien der eigenen Kette; Dateien von Vorgängergeräten derselben Kette
  zählen dabei als eigene, damit alte Stände wie zugesagt aus der Aufbewahrung fallen. Nach `geraet_ersetzt` liegt ein mit
  `RK_sig` signierter Ersetzungsmarker im Ordner, den jedes Gerät vor dem Sichern prüft; ein
  ersetztes Gerät sichert danach nicht mehr.
- Nach einem Gerätewechsel sollte der Zugang des alten Geräts zum Ordner beim Anbieter entzogen
  werden; die App weist darauf hin.
- Aufbewahrung: die letzten zehn Sicherungen und je eine pro Tag der letzten 30 Tage; ältere
  löscht die App.
- Schlägt die Sicherung fehl oder ist die letzte älter als drei Tage, zeigt die App das deutlich an.
- Manuelle Sicherung und Export der Sicherungsdatei brauchen eine native Bestätigung.

### 10.3 Wiederherstellung und Frische

- Wiederherstellung auf einem neuen Gerät nur über eine native Ansicht: Wörter, Ordner wählen.
- **Auswahl nach Kettenverwandtschaft vor der Nummer:** Die App wertet die `geraet_ersetzt`-Glieder
  aller Dateien und alle Ersetzungsmarker im Ordner aus. Eine Datei, deren `geraet_id` in irgendeiner
  gültigen Kette ersetzt ist und deren Kette nach dieser Ersetzung weiterläuft, ist ungültig; ebenso
  eine Datei, deren Kette Vorgängerin einer anderen ist und deren Gerät dort ersetzt wurde. Bei verzweigten Ketten
  wählt die App nichts automatisch, sondern zeigt beide Stände nativ zur Wahl. Innerhalb derselben
  Kette gilt die höchste `sicherung_nr`; Datum und Nummer werden zur Bestätigung gezeigt.
- Ist der hochsensible Bereich aktiv, richtet die Person auf dem neuen Gerät nativ eine App-PIN
  ein. Danach sofort `geraet_ersetzt` und Rotation beider Generationen (Abschnitt 9).
- **Grenze:** Fehlt die neueste Sicherung im Ordner (gelöscht, nicht synchronisiert), kann die App
  das nicht erkennen. Gegenmittel: Aufbewahrung mehrerer Stände, Anzeige des Datums.

### 10.4 Gerätewechsel

Über Sicherung und Wiederherstellung. Das alte Gerät wird danach in der App gelöscht; tut es das
nicht (verloren, defekt), sorgt die Rotation dafür, dass es nichts Neues lesen kann.

## 11. Erinnerungen und Zufallsabfragen

- Der **Planer** liegt in `dak-core` (#42, #93). Er plant immer nur die nächsten Tage im Voraus
  (iOS hält höchstens 64 geplante Benachrichtigungen) und plant bei jedem Öffnen und über die
  Hintergrundaufgaben des Systems nach.
- Der Planungsstand (Uhrzeiten, neutrale Texte, keine Präparatnamen, keine Werte) liegt in einem
  kleinen, nicht verschlüsselten Bereich, damit Erinnerungen nach einem Neustart ohne Entsperren
  funktionieren. Er verrät, dass und wann Erinnerungen eingestellt sind.
- Android: exakte Alarme werden angefragt; ohne Erlaubnis zeigt die App an, dass Erinnerungen sich
  verschieben können.
- Obergrenze für Benachrichtigungen je Tag (Standard acht); Medikamentenerinnerungen haben einen
  eigenen Kanal und Vorrang.
- Zufallsabfragen werden auf dem Gerät gewürfelt und als Abfrage-Objekte gespeichert (geplant,
  ausgelöst, Ergebnis). Eine Abfrage gilt 30 Minuten; spätere Antworten werden als verspätet
  markiert.

## 12. Entwürfe und Sitzung

- Eine Sitzung endet nach der eingestellten Sperrzeit oder beim Sperren des Geräts.
- Was gerade erfasst wird, geht dabei nicht verloren: Der Entwurf wird laufend mit dem
  Schreibschlüssel der Klasse versiegelt und beim nächsten Entsperren angeboten (#94).
- Es gibt keinen dauerhaften Klartext-Zwischenspeicher. Indizes über Gesundheitsdaten sind mit
  einem Teilschlüssel aus `KK` verschlüsselt, Indizes über hochsensible Inhalte nur mit einem
  Teilschlüssel aus `KK_h` (oder es gibt keine); entschlüsselte Inhalte leben nur im Arbeitsspeicher
  der Sitzung (#67).

## 13. Kryptokern und Oberfläche

- `dak-core` (Rust, libsodium über `libsodium-sys-stable`, `zeroize`), eingebunden in **ein**
  Mobile-Framework (#33). Geheime Schlüssel existieren nur dort.
- **Bridge:** nur fachliche Operationen; native Bestätigung, gebunden an einen Digest der konkreten
  Aktion, für Arzt-PDF und jeden Export, manuelle Sicherung, Wiederherstellung, App-PIN ändern,
  Herab- und Hochstufen, Anzeige der Recovery-Wörter, `recovery_neu`, Rotation auf Wunsch.
- **Kein Netz für die App (#59):**
  - Android: Die App wird **ohne `INTERNET`-Berechtigung** gebaut; die Pipeline prüft das Manifest.
    Ob das gewählte Framework so läuft, belegt der Prototyp (#81).
  - Framework-Funktionen mit Netzzugang (z. B. `CapacitorHttp`) sind nicht eingebunden; WebRTC und
    DNS-Prefetch in der WebView sind abgeschaltet, soweit die Plattform das erlaubt.
  - Telefonnummern und Links öffnet nur die native Hilfe-Ansicht, nie die WebView.
  - iOS hat keine Sperre auf Ebene des Betriebssystems; die Doku sagt das offen.
- **Hilfe** ist eine native Ansicht, auch **ohne Entsperren** erreichbar (Sperrbildschirm der App):
  TelefonSeelsorge 0800 111 0 111, 0800 111 0 222 und 116 123, Nummer gegen Kummer 116 111,
  ärztlicher Bereitschaftsdienst 116 117, Notruf 112, eigene Vertrauenspersonen (#71).
- Fremde oder wiederhergestellte Zeichenketten nur als Text; Prüfung in `dak-core`.

## 14. Anforderungen an die App

- **Medikation:** Einnahmen werden nie vorbelegt; die Wirkungsfrage erscheint nur, wenn heute eine
  Einnahme eines Präparats bestätigt ist, dessen Wirkung im Tagesverlauf spürbar ist.
- **Keine Vorbelegung**, Regler ohne Startposition, gespeichert wird, ob ein Wert aktiv gesetzt
  wurde (#73).
- **Sprache** nur über die Offline-Spracherkennung; nicht verfügbar heißt: Funktion aus.
  Tastatur-Diktat und Drittanbieter-Tastaturen in hochsensiblen Feldern, bei PIN und Wörtern aus.
- **Speicher:** App-Daten von Cloud-Backups des Betriebssystems und Gerät-zu-Gerät-Übertragung
  ausnehmen, auf Zielgeräten testen (#67); Vorschaubild im App-Umschalter abdecken.
- **Auswertung:** einzelne Status als Punkte, je Lebenstag Zahl der Status und Spanne; Mittelwerte
  nur mit Hinweis auf die Erfassungsdichte; selbst ausgelöste Status und Abfragen getrennt
  anzeigbar; Vorher/Nachher nur mit Angabe der Erfassungsdichte beider Zeiträume; keine
  automatischen Zusammenhänge, keine Farbwertung, keine geglättete Wirkungskurve (#74).
- **Arzt-PDF:** Freitext und Angaben über Dritte (äußere Einflüsse, Konflikte) standardmäßig
  nicht enthalten, nur nach Auswahl (#36).

## 15. Migration und Erweiterung

- Jede neue `formatversion` bringt eine Migrationsregel: neuer Bestand wird vollständig geschrieben
  und geprüft, bei Unterbrechung bleibt der alte Bestand nutzbar, umgeschaltet wird atomar;
  hochsensible Daten werden nur mit frischer PIN migriert. Testvektoren für jede Migration (#63).
- **Vor der ersten Freigabe (Teil B):** Migration in das dann geprüfte Freigabeformat. Weil in M1
  kein Schlüssel das Gerät verlässt, ist das ein lokaler Durchlauf.

- **Stabilität während des Versuchs:** M1a schreibt und liest das vollständige Teil-A-Format, auch
  wo die Oberfläche noch fehlt (`KK_h` über die Geräteauthentifizierung, `k_h`, leerer
  hochsensibler Bund, `kapsel_h`, Abfrageobjekte, Formulare). Abnahmekriterium: Eine Sicherung aus M1a öffnet der M1b-Kern ohne Migration. Jede
  Version liest alle Formatversionen innerhalb der Aufbewahrung; neue Versionen werden in zwei
  Schritten eingeführt (erst lesen, dann schreiben). In Phase A des Versuchs keine Formatänderung.

## 16. Bibliotheken und Updates

Gepinnte Versionen, `AUDITS.md`, Updates nur mit gelesenen Release- und Security-Hinweisen,
Testvektoren, Wycheproof-Fällen, Negativtests; native Abhängigkeiten, Toolchains, Trennung von Bau
und Signatur (#68).

## 17. Offen bis v1.0

| Punkt | Issue |
|---|---|
| Prototyp: Linux-Kern sofort, Plattformwege nach der Framework-Entscheidung | #100, #33, #101 |
| Bytegenaue Formattabellen, KDF-Tabelle, Größen- und Laufzeitbudgets, Testvektoren, Migrationsregel | #63 |
| Bedrohungsmodell gegen Teil A | #66 |
| Plattformmatrix (Biometriebindung, Backup-Ausschluss, Bildschirmschutz, Android ohne Netz) | #67 |
| Argon2id-Parameter, Fehlversuchsregel | #60 |
| Ausdrucksform der Bedingungen | #6 |
