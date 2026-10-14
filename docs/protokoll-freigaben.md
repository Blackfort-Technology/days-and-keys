# Freigabe- und Mehrgeräteprotokoll (Teil B, Entwurf auf Basis v0.4)

Stand 2026-10-06. **Nicht Teil von Protokoll v1.0.** Nach der Erörterung vom 2026-10-06
(nicht öffentliches Archiv; Befunde als Issues) ist der Kern für M1 in `docs/protokoll.md` (v0.5)
neu gefasst; dieses Dokument hält den bisherigen Stand v0.4 für Freigaben, Gerätekette,
Mehrgeräte-Logs und Einladungen fest. Es wird erst für M1.5 überarbeitet, mit eigener Prüfung,
und muss dann zu den Formatentscheidungen in `docs/protokoll.md` passen.

**Durch `docs/protokoll.md` (Teil A, derzeit v0.6) ersetzt:** Identitäten und Schlüsselhaltung (hier 3),
Feldketten (4), Wertformat (5.1), Schlüsselbund und Wiederherstellung (7.2, 7.5),
Client-Anforderungen (12), Kryptokern (13), Bibliotheken (14).

**Offene Befunde für diesen Teil** (Milestone M1.5): Konsistenz- und Frischemodell (#51),
Rangfolge der Befugnisse in der Gerätekette (#52), Umschläge verraten verborgene Felder und
verschweigen weggelassene (#55), Einladungslink und Browser-Fallback (#56), beidseitige
Kontakt-Authentisierung und Sicherheitswörter (#57), Bestätigungen an der Bridge (#58),
Empfänger- und Gerätebindung von Freigaben (#61), Transport-Anmeldung (#65), Freigaben in
engen Beziehungen (#76), Serverbetrieb (#14).

---

## 1. Ziele und Nicht-Ziele

**Ziele**

1. Der Server sieht keinen Klartext: keine Werte, keine Feldnamen, keine Formulare, keine Orte,
   keine genauen Daten.
2. Freigaben gelten pro Feld und lassen sich zeitlich eingrenzen („ab Datum X", „nur Zeitraum Y").
3. Ein Widerruf sperrt den Empfänger für alle künftigen Daten und auf Wunsch auch für die
   Vergangenheit, soweit sie auf dem Server liegt.
4. Empfänger erkennen, ob ein Eintrag echt vom Autor stammt und ob der Server Einträge
   unterschlägt oder verändert.
5. Es gibt keinen Generalschlüssel.

**Nicht-Ziele**

- Schutz dessen, was ein Empfänger bereits gesehen hat. Was einmal entschlüsselt wurde, kann
  kopiert worden sein. Die App sagt das beim Widerruf ausdrücklich.
- Verbergen, *wer mit wem* teilt. Der Server kennt die Freigabebeziehungen zwischen Konten
  (siehe Abschnitt 9).
- Schutz vor einem entsperrten, kompromittierten Gerät des Autors.

## 2. Bausteine

Ausschließlich libsodium, auf allen Plattformen über den Rust-Kern `dak-core` (Abschnitt 13).
Keine eigene Kryptografie und keine Nachbauten von libsodium-Funktionen.

| Zweck | Primitiv |
|---|---|
| Signatur | Ed25519 (`crypto_sign`) |
| Asymmetrisch verschlüsseln | X25519 + XSalsa20-Poly1305 (`crypto_box`, `crypto_box_seal`) |
| Symmetrisch verschlüsseln | XChaCha20-Poly1305 IETF (`crypto_aead_xchacha20poly1305_ietf`), Nonce 24 Byte zufällig |
| Schlüsselableitung | `crypto_kdf_derive_from_key` (BLAKE2b) |
| Hashkette, Log-Hash | BLAKE2b-256 (`crypto_generichash`) |
| Auffüllen | `sodium_pad`, Blockgrößen 64 / 256 / 1024 / 4096 / 16384 Byte |
| Passwort/Recovery | kein Passwort-Login; Recovery über 256-Bit-Zufallsschlüssel (Abschnitt 7) |

Alle Kontexte für `crypto_kdf` sind 8 Byte: `dakchain`, `dakfield`, `dakmeta_`, `dakkring`, `dakrecov`.

## 3. Identitäten und Geräte

Ein Konto besteht aus mehreren Geräten. Jedes Gerät hat **eigene** Schlüssel; die Zugehörigkeit
zum Konto hält eine signierte **Gerätekette** fest (Abschnitt 7). So lässt sich ein verlorenes
Gerät gezielt aussperren (Bedrohungsmodell T-13, Maßnahme M-12).

Je **Gerät**:

- **Gerätesignaturschlüssel** `DK_sig` (Ed25519): signiert Einträge, Freigaben und Glieder der
  Gerätekette und meldet das Gerät am Server an.
- **Geräteverschlüsselungsschlüssel** `DK_enc` (X25519): empfängt den Schlüsselbund-Schlüssel.

Je **Konto**:

- **Kontoverschlüsselungsschlüssel** `AK_enc` (X25519): an ihn werden Freigaben anderer Konten
  versiegelt. Alle Geräte des Kontos kennen ihn über den Schlüsselbund; er wird bei jedem
  Gerätewiderruf erneuert.
- **Schlüsselbund-Schlüssel** `KK` (32 Byte zufällig): verschlüsselt den Schlüsselbund; je Gerät
  mit dessen `DK_enc` umhüllt; bei jedem Gerätewiderruf erneuert.
- **Konto-ID** = BLAKE2b-256 des ersten Glieds der Gerätekette. Der Server adressiert Konten nur
  darüber.
- **Recovery-Schlüssel** `RK` (256 Bit, Abschnitt 7): daraus abgeleitet ein Ed25519-Schlüssel
  `RK_sig`, der als „Papiergerät" in der Gerätekette steht und Geräte hinzufügen oder entfernen
  darf.

**Schlüsselhaltung:** Geheime Schlüssel existieren nur im nativen Kryptokern `dak-core`, nie in
der WebView (Abschnitt 13). Ruhend sind sie verschlüsselt; den Schlüssel dazu hält die Hardware
(iOS Secure Enclave, Android StrongBox bzw. TEE, auf dem Desktop der Schlüsselbund des
Betriebssystems). Weil Secure Enclave und StrongBox nur P-256 können, schützt ein P-256-Schlüssel
der Hardware ein symmetrisches Wrapping; entsperrt wird per Biometrie oder Geräte-PIN. Nach dem
Entsperren liegen die Schlüssel im nativen Arbeitsspeicher, bis die Sperre greift, und werden dann
gelöscht. Das Protokoll selbst sieht davon nichts.

**Anmeldung am Server:** Challenge-Response. Der Server schickt eine Zufallszahl, das Gerät
signiert `"dak-auth-v1" || server_origin || nonce || zeit` mit `DK_sig`. Der Server lässt nur
Geräte zu, die in der aktuellen Gerätekette des Kontos stehen.

## 4. Felder, Zeiträume und Schlüsselketten

### 4.1 Feld-ID

Jedes Feld eines Formulars bekommt eine zufällige, 16 Byte lange **Feld-ID**. Name, Typ,
Einheit und Wertebereich stehen nur verschlüsselt in der Formulardefinition. Der Server sieht
nur IDs.

### 4.2 Zeitraum (Epoche)

Eine Epoche ist ein Kalendermonat, gezählt ab Januar 2026 (`e = 0`). Die Epoche eines
Eintrags ist für den Server sichtbar, das genaue Datum nicht (es steht im Chiffretext).

### 4.3 Kette

Für jedes Feld führt der Autor eine oder mehrere **Ketten**. Eine Kette beginnt bei einer
Start-Epoche `s` mit einem Zufallsschlüssel `C_s` (32 Byte) und wird vorwärts abgeleitet:

```
C_(e+1) = KDF(C_e, id = 1, ctx = "dakchain")      Kettenschlüssel
F_e     = KDF(C_e, id = 2, ctx = "dakfield")      Inhaltsschlüssel der Epoche e
M_e     = KDF(C_e, id = 3, ctx = "dakmeta_")      Schlüssel für die Feldbeschreibung
```

Daraus folgen zwei Arten von Freigabe:

- **„Ab Epoche X"**: Der Empfänger bekommt `C_X`. Er kann alle Epochen ab X dieser Kette
  ableiten, aber keine frühere (BLAKE2b ist nicht umkehrbar).
- **„Nur Epochen X bis Y"**: Der Empfänger bekommt nur `F_X … F_Y`. Damit kann er weder
  frühere noch spätere Epochen ableiten.

Jede Kette hat eine zufällige **Ketten-ID** (16 Byte). Ein Wechsel der Kette (Widerruf,
Abschnitt 6) startet eine neue Kette ab der aktuellen Epoche; Einträge nennen die Ketten-ID,
mit der sie verschlüsselt sind.

## 5. Einträge

### 5.1 Feldwert

```
klartext  = sodium_pad(cbor({wert, zeitpunkt, formular_version, quelle}), block)
ad        = "dak-val-v1" || konto_id || eintrag_id || feld_id || ketten_id || e
chiffrat  = XChaCha20-Poly1305(F_e, nonce, klartext, ad)
```

`zeitpunkt` ist der genaue Zeitpunkt, `quelle` z. B. „manuell", „wetter", „healthkit".
Die Associated Data binden den Wert an Eintrag, Feld, Kette und Epoche, sodass der Server ihn
nicht unbemerkt umhängen kann.

### 5.2 Umschlag und Log

Ein Eintrag besteht aus `eintrag_id` (16 Byte zufällig), Epoche, laufender Nummer `n` des
**schreibenden Geräts**, `vorgaenger_hash`, `geraetekette_kopf` (Hash des zum Zeitpunkt gültigen letzten Glieds
der Gerätekette), der Kennung des signierenden Geräts und der Liste der **Feld-Hashes**:

```
h_i           = BLAKE2b("dak-fh-v1" || feld_id || ketten_id || nonce || chiffrat)
feld_hashes   = sortiert(h_1 … h_k), aufgefüllt mit Zufallswerten auf genau K = 32 Einträge
umschlag_hash = BLAKE2b(cbor(umschlag ohne signatur))
signatur      = Ed25519(DK_sig, "dak-entry-v1" || umschlag_hash)
```

Die Chiffrate selbst liegen neben dem Umschlag als `(feld_id, ketten_id, nonce, chiffrat)`.
Ein Empfänger bekommt den vollständigen Umschlag und nur die Chiffrate seiner Felder. Er prüft,
dass deren Hashes in `feld_hashes` stehen. Über die übrigen Felder erfährt er nichts: Die Liste
hat immer 32 Einträge, echte und aufgefüllte Hashes sind nicht unterscheidbar (Maßnahme M-5 zu
T-5). Hat ein Formular mehr als 32 Felder, wird ein Eintrag auf mehrere Umschläge verteilt.

**Ein Log je Gerät.** `vorgaenger_hash` ist der `umschlag_hash` des vorigen Eintrags **desselben
Geräts**. Jedes Gerät führt damit ein eigenes Append-only-Log; das Tagebuch eines Kontos ist die
Vereinigung der Logs seiner Geräte laut Gerätekette, sortiert nach dem `zeitpunkt` im Klartext.
So entstehen keine Verzweigungen, wenn zwei Geräte offline gleichzeitig schreiben (ein Log je
Konto hätte dann zwei Einträge mit derselben Nummer, die wie eine Manipulation aussehen).

Empfänger prüfen je Gerätelog Signatur, Gültigkeit des Geräts laut Gerätekette, Nummernfolge und
Hashkette; fehlt ein Glied oder taucht eine abweichende Version auf, zeigt die App eine Warnung.
Das Glied `geraet_entfernt` enthält den letzten bekannten Log-Kopf des entfernten Geräts
(Nummer und Hash, aus Sicht des widerrufenden Geräts). Spätere Einträge dieses Geräts werden
abgelehnt; noch nicht synchronisierte Einträge eines verlorenen Geräts gehen verloren.

Änderungen an einem Eintrag sind neue Einträge mit Verweis auf `eintrag_id`. Ändern zwei Geräte
denselben Eintrag, gilt die Änderung mit dem späteren `zeitpunkt`; die andere bleibt im Verlauf
sichtbar. Löschen ist ein signierter Lösch-Eintrag; der Server entfernt danach die Chiffrate.

### 5.3 Formulardefinitionen

Formulare und ihre Versionen sind eigene Objekte, verschlüsselt mit einer eigenen Kette pro
Formular. Ein Empfänger braucht sie nicht: Die Beschreibung eines freigegebenen Feldes (Name,
Typ, Einheit) steht verschlüsselt mit `M_e` in der Freigabe bzw. als Feld-Metaobjekt je Kette.

## 6. Freigaben und Widerruf

### 6.1 Freigabe

```
inhalt   = cbor({freigabe_id, autor, empfaenger, feld_id, ketten_id,
                 art: "ab" | "zeitraum", schluessel: C_X | [F_X … F_Y],
                 von_epoche, bis_epoche, feldbeschreibung, erstellt})
signiert = Ed25519(DK_sig_autorgeraet, "dak-grant-v1" || BLAKE2b(inhalt)) || inhalt
blob     = crypto_box_seal(signiert, AK_enc_empfaenger)
```

Erst signieren, dann für den Empfänger versiegeln. Der Server speichert `blob` unter der
Konto-ID des Empfängers und dazu im Klartext `(autor, empfaenger, feld_id, ketten_id)`, damit
er dem Empfänger nur die freigegebenen Chiffrate ausliefert (Zugriffskontrolle als zweite
Linie, Abschnitt 9).

### 6.2 Widerruf

Widerruft der Autor die Freigabe von Feld `f` für Empfänger `R` in Epoche `w`:

1. Neue Kette für `f` ab Epoche `w` mit frischem `C'_w`. Alle Werte ab `w` werden mit dieser
   Kette verschlüsselt; bereits hochgeladene Werte der Epoche `w` werden neu verschlüsselt.
2. Allen übrigen Empfängern von `f` stellt der Client neue Freigaben für die neue Kette aus,
   im gleichen Umfang wie bisher.
3. Signierte Anweisung an den Server, die Freigabe-Einträge von `R` für `f` zu löschen. Ab dann
   liefert der Server `R` keine Chiffrate von `f` mehr aus.
4. **Optional „auch rückwirkend":** Der Client baut für alle Epochen vor `w` ebenfalls neue
   Ketten, verschlüsselt die alten Werte neu, lädt sie hoch und lässt die alten Chiffrate
   löschen. Wer sie vorher heruntergeladen hat, behält sie trotzdem; das zeigt die App vorher an.

Die Datenmengen eines Tagebuchs sind klein (wenige Kilobyte je Feld und Monat), die Neuver-
schlüsselung ist daher unkritisch.

## 7. Gerätekette, Schlüsselbund, Wiederherstellung

### 7.1 Gerätekette

Die Gerätekette ist ein eigenes, signiertes Append-only-Log je Konto. Glieder:

| Glied | Inhalt | signiert von |
|---|---|---|
| `gruendung` | erstes Gerät (`DK_sig.pub`, `DK_enc.pub`, Name), `RK_sig.pub`, `AK_enc.pub` | erstes Gerät und `RK_sig` |
| `geraet_hinzu` | neues Gerät mit Name | ein gültiges Gerät oder `RK_sig` |
| `geraet_entfernt` | Kennung des Geräts, Zeitpunkt, letzter bekannter Log-Kopf des Geräts | ein anderes gültiges Gerät oder `RK_sig` |
| `kontoschluessel_neu` | neuer `AK_enc.pub` | ein gültiges Gerät |

Jedes Glied enthält den Hash des vorigen. Andere Konten speichern den zuletzt gesehenen Kopf und
lehnen eine Kette ab, die diesen Kopf nicht fortsetzt. Wechselt der Kopf, zeigt die App beim
Kontakt einen Hinweis („Kontakt hat ein neues Gerät hinzugefügt").

### 7.2 Schlüsselbund

Alle eigenen Kettenschlüssel, empfangenen Freigaben, Formular-Ketten und `AK_enc.sec` liegen in
einem Objekt, verschlüsselt mit `KK`. Es wird bei jeder Änderung hochgeladen. `KK` liegt beim
Server je Gerät mit dessen `DK_enc` umhüllt und einmal mit dem Recovery-Schlüssel.

### 7.3 Neues Gerät

Das neue Gerät erzeugt `DK_sig` und `DK_enc` und zeigt einen QR-Code mit beiden öffentlichen
Schlüsseln und einer Prüfzahl. Ein bekanntes Gerät scannt ihn **mit der Kamera** (kein
Eintippen), beide zeigen die Prüfzahl, der Nutzer bestätigt den Vergleich aktiv. Das bekannte
Gerät hängt `geraet_hinzu` an die Gerätekette und umhüllt `KK` für das neue Gerät. Alle anderen
Geräte des Kontos melden das neue Gerät mit Name und Zeit (Maßnahme M-11 zu T-12).

### 7.4 Gerät widerrufen

Ein verlorenes oder abgegebenes Gerät widerruft ein anderes Gerät oder der Recovery-Schlüssel:

1. Glied `geraet_entfernt`. Ab dann lässt der Server das Gerät nicht mehr anmelden, und Signaturen
   dieses Geräts nach dem Widerrufszeitpunkt gelten nicht.
2. Neuer `KK`, neuer `AK_enc` (Glied `kontoschluessel_neu`), Schlüsselbund neu verschlüsselt und
   für die verbleibenden Geräte umhüllt.
3. Für **alle eigenen Felder** beginnen neue Ketten ab der aktuellen Epoche, und alle Freigaben
   werden für die neuen Ketten neu ausgestellt (wie Abschnitt 6.2, nur ohne ausgeschlossenen
   Empfänger). Optional rückwirkend.
4. Die App informiert alle Kontakte, die diesem Konto Felder freigegeben haben. Deren Apps bieten
   an, die betroffenen Ketten ebenfalls neu zu beginnen, weil das verlorene Gerät die
   empfangenen Kettenschlüssel kennt.

Was auf dem verlorenen Gerät bereits entschlüsselt lag, schützt nur die App-Sperre
(Abschnitt 12).

### 7.5 Wiederherstellung

- **Recovery-Schlüssel:** 256 Bit zufällig, angezeigt als 24 Wörter zum Ausdrucken. Daraus
  werden `RK_sig` und ein Schlüssel zum Umhüllen von `KK` abgeleitet (`crypto_kdf`, Kontext
  `dakrecov`). Mit ihm lässt sich ein neues Gerät in die Kette aufnehmen, auch wenn kein Gerät
  mehr vorhanden ist.
- **Anzeige und Eingabe der Wörter nur nativ** (Abschnitt 13.3), nie in der WebView.
- **Shamir 2 aus 3 zurückgestellt:** Eine Aufteilung in der WebView würde dort den vollständigen
  Recovery-Schlüssel offenlegen. Kommt erst mit einer separat geprüften nativen Implementierung
  in `dak-core`; dann gilt: Zwei der Vertrauenspersonen gemeinsam haben vollen Zugriff, die App
  sagt das beim Einrichten.
- **Ohne Recovery-Schlüssel und ohne Gerät sind die Daten verloren.** Das ist gewollt.

## 8. Einladungen

1. Der Autor erzeugt auf dem Gerät einen Einladungscode (128 Bit) und eine `einladung_id`.
   Beim Server liegt nur `BLAKE2b(code)` mit Ablauf nach 48 Stunden.
2. Er schickt einen Link `https://daysandkeys.wirdsichtbar.de/e/<einladung_id>#<code>` per
   Mail oder WhatsApp. Der Teil nach `#` geht nie an den Server. Der vorformulierte Text nennt
   keine Gesundheitsthemen (M-19).
3. **Der Link wird nur von der App geöffnet** (Universal Links auf iOS, App Links auf Android).
   Die Seite `/e/*` auf dem Server ist statisches HTML ohne Skript mit
   `Content-Security-Policy: script-src 'none'` und dem Hinweis, die App zu installieren. Sie
   darf nie ein Skript ausliefern, denn ein Skript könnte das Fragment lesen (T-1, M-1).
   Ersatzweg ohne Link: Die App zeigt den Code als zwölf Wörter, der Empfänger tippt sie ein.
4. Die App des Empfängers meldet die Konto-ID, den Kopf ihrer Gerätekette und `AK_enc.pub` samt
   `HMAC(code, konto_id || kopf || AK_enc.pub)` beim Server ab. Der Server kann diesen Wert nicht
   fälschen, weil er den Code nicht kennt.
5. Der Autor prüft den HMAC und sieht Name und Fingerabdruck, bevor er bestätigt. Die Einladung
   ist danach verbraucht.
6. **Sicherheitsnummer:** Beide Apps zeigen sechs Wörter aus
   `BLAKE2b(min(ID_a, ID_b) || max(ID_a, ID_b))` über die Konto-IDs. Werden sie am Telefon oder
   persönlich verglichen, gilt der Kontakt als „verifiziert".
7. **Felder mit Gesundheitsbezug** (in der Formulardefinition markiert) lassen sich nur an
   verifizierte Kontakte freigeben. Für andere Felder genügt ein unverifizierter Kontakt mit
   Warnhinweis (M-2 zu T-2 und T-20).

Wer den Chat mitliest, kann die Einladung vor dem Empfänger einlösen. Dagegen helfen die kurze
Laufzeit, die Einmaligkeit, die Bestätigung in Schritt 5 und die Sicherheitsnummer.

## 9. Was der Server sieht

| Sieht der Server | Sieht er nicht |
|---|---|
| Konto-IDs, öffentliche Schlüssel, Gerätekette (Zahl der Geräte, Zeitpunkte) | Namen der Personen (außer was im Konto-Profil verschlüsselt liegt) |
| wer wem Felder freigibt, wie viele | welche Felder das sind (Namen, Typen) |
| Epoche (Monat) je Eintrag, Upload-Zeitpunkte | genaue Zeitpunkte, Orte, Werte |
| aufgefüllte Größen | genaue Größen |
| IP-Adressen der Clients | Formulare und ihre Versionen |

Gegenmaßnahmen: Auffüllen (Abschnitt 2), gebündeltes Hochladen in festen Abständen statt
direkt nach der Eingabe, keine Logs mit IP-Adressen länger als nötig.

## 10. Versionierung des Protokolls

Alle signierten und verschlüsselten Strukturen beginnen mit einem Domain-Separator samt
Version (`dak-…-v1`). Eine neue Protokollversion bekommt neue Separatoren; alte Einträge bleiben
lesbar, bis sie neu verschlüsselt sind.

## 11. Offene Punkte

- Server-API spezifizieren (rund zehn Endpunkte: Anmeldung per Ed25519-Challenge, Log anhängen,
  Abruf ab Laufnummer, Freigabe-Umschläge je Empfänger, Gerätekette, Löschanweisungen,
  Einladungen). Anmeldung und Sync-Token nach dem Vorbild von Etebase.
- Log-Kopf zwischen Clients abgleichen, um Unterschlagen durch den Server zu erkennen (M-3, P3).
- Signierte Widerrufsliste im Log des Autors (M-9, P3).
- Freigaben hybrid versiegeln (X25519 + ML-KEM, z. B. X-Wing), weil Gesundheitsdaten lange
  schutzbedürftig sind (#37).
- Verhalten bei Uhrzeit- und Zeitzonenwechsel an Epochengrenzen.
- Sollen Einträge gegenüber Empfängern abstreitbar sein, also mit paarweisem MAC statt Signatur
  (M-16)? Entscheidung der Nutzer.
- Druck zum Entsperren durch nahestehende Personen: reicht die App-Sperre, oder braucht es Felder,
  die nur nach erneuter Biometrie sichtbar werden?
- Bedrohungsmodell nach diesen Änderungen und nach der API-Spezifikation erneut prüfen.

## 12. Anforderungen an die Clients

Aus dem Bedrohungsmodell (`docs/threat-model/`), Maßnahmen der Priorität P1 und P2:

- **App-Sperre** (M-13): eigene Sperre mit Biometrie oder PIN, unabhängig von der Gerätesperre,
  Sperrzeit einstellbar, Standard eine Minute.
- **Geschützter Cache** (M-13): entschlüsselte Werte nur verschlüsselt mit einem
  hardwaregebundenen Schlüssel ablegen; App-Daten von Cloud-Backups ausnehmen (iOS:
  `NSFileProtectionComplete` und Ausschluss aus dem iCloud-Backup, Android: `allowBackup=false`);
  Vorschaubild im App-Umschalter abdecken (`FLAG_SECURE` bzw. Abdeckung beim Wechsel).
- **Kein Nachladen von Code** (M-18): keine `server.url` in der Capacitor-Konfiguration, keine
  Live-Update-Plugins, Content-Security-Policy `script-src 'self'`; die Pipeline bricht ab, wenn
  eines davon auftaucht.
- **Kryptografie und Autorisierung nativ** (M-25): ausschließlich über `dak-core` nach
  Abschnitt 13; die WebView enthält keine Kryptobibliothek.
- **Abhängigkeiten** (M-14): Lockfiles mit Integritäts-Hashes, Updates nach Abschnitt 14,
  ausgehende Verbindungen per CSP auf den eigenen Server und den Wetterdienst beschränkt.
- **Signaturprüfung an einer Stelle** (M-15): `dak-core` öffnet Freigaben und Einträge nur nach
  Prüfung von Signatur und Gerätekette; Tests mit manipulierten Daten müssen scheitern.
- **Wetter** (M-20): Koordinaten vor dem Abruf auf eine Dezimalstelle runden (etwa 10 km),
  höchstens ein Abruf am Tag zu zufälliger Uhrzeit, ohne Kennungen.
- **Uploads bündeln** (M-6): in festen Fenstern, z. B. alle sechs Stunden, statt direkt nach der
  Eingabe.
- **Local-first** (M-8): Jedes Gerät hält eine vollständige verschlüsselte Kopie der eigenen
  Daten; Export als verschlüsselte Datei jederzeit möglich.
- **Benachrichtigungen** (M-27 zu T-30):
  - Erinnerungen werden **lokal** auf dem Gerät geplant. Der Server schickt keine Erinnerungen.
  - Push über Apple bzw. Google nur ohne Inhalt („Neue Daten verfügbar"), wenn überhaupt.
  - Texte auf dem Sperrbildschirm standardmäßig neutral („Zeit für deinen Eintrag"), ohne
    Medikamentennamen, Substanzen oder Werte. Wer will, kann genauere Texte einschalten;
    hochsensible Inhalte erscheinen nie in einer Benachrichtigung.
  - Die App ersetzt keinen Medikamentenwecker; das steht beim Einrichten der Erinnerung.
    Einnahmezeiten werden in Ortszeit geführt, Sommerzeit und Reisen verschieben sie nicht still.
- **Sprache nur auf dem Gerät** (M-28 zu T-31): Spracheingabe und Transkription ausschließlich
  mit der Offline-Spracherkennung des Betriebssystems (iOS: `requiresOnDeviceRecognition`,
  Android: On-Device-Recognizer). Keine Cloud-Dienste. Das Übertragen gesprochener Sätze in Felder
  per LLM kommt erst, wenn es vollständig lokal läuft. Audio-Notizen bis zwei Minuten werden als
  verschlüsselter Feldwert gespeichert; längere brauchen `crypto_secretstream` (offen).
- **Sicherheitsnetz bei Krisen:** Trägt jemand das Ereignis „Krise" ein oder liegt die Stimmung
  auf der niedrigsten Stufe, zeigt die App ruhig und ohne Bewertung Hilfe an: Telefonseelsorge
  0800 111 0 111 und 0800 111 0 222 (kostenlos, rund um die Uhr), ärztlicher Bereitschaftsdienst
  116 117, Notruf 112. Die Liste ist anpassbar (Land, eigene Vertrauenspersonen, behandelnde
  Ärztin). Die App meldet nichts automatisch an Dritte und wertet nicht medizinisch aus; die
  Prüfung läuft lokal.

## 13. Kryptokern und Bridge als Sicherheitsgrenze

### 13.1 Aufbau

Das gesamte Protokoll steckt in **einer** Rust-Crate `dak-core`. Sie nutzt libsodium über
`libsodium-sys-stable` und hält eine sichere Rust-Hülle darum (Längen, Rückgabewerte,
`zeroize` für Geheimnisse). Eingebunden wird sie per UniFFI in Swift (iOS) und Kotlin (Android),
dahinter ein dünnes Capacitor-Plugin, und auf dem Desktop direkt über Tauri-Commands.

- **Geheime Schlüssel existieren nur in `dak-core`.** Ruhend sind sie mit einem Hardware-Schlüssel
  umhüllt (plattformspezifische Adapter für Secure Enclave, StrongBox/TEE, Desktop-Schlüsselbund).
  Nach dem Entsperren liegen sie im nativen Arbeitsspeicher, bis die Sperre greift; danach werden
  sie mit `zeroize` gelöscht.
- **Der Server** nutzt aus derselben Crate nur die gemeinsamen Protokolltypen und die öffentlichen
  Prüf- und Parsing-Funktionen (Signaturen, Gerätekette, Log). Geheime Operationen sind dort nicht
  einkompiliert.
- Eigene Nachbauten von libsodium-Funktionen gibt es nicht.

### 13.2 Die Bridge kontrolliert Autorisierung, nicht nur Schlüssel

Native Schlüsselhaltung verhindert, dass Schlüssel abfließen. Missbrauch verhindert sie allein
nicht: Kompromittierter WebView-Code könnte über die Bridge entschlüsseln, ausleiten oder
Freigaben erzeugen. Deshalb gilt für `dak-core`:

1. **Keine generischen Funktionen.** Es gibt kein `decrypt(handle, bytes)`, kein
   `sign(handle, bytes)`, kein Herausgeben von Schlüsseln. Die API besteht aus fachlichen
   Operationen (`entry.seal`, `entry.open`, `grant.create`, `export.doctor` …), die ihre Eingaben
   selbst prüfen.
2. **Prüfungen nativ.** Kontakte, Verifikationsstatus, Schutzklassen, Freigabeberechtigungen und
   Zeiträume prüft `dak-core` selbst. Die WebView kann sie nicht übersteuern.
3. **Native Bestätigung mit konkreten Aktionsdaten** für:
   - Freigaben von Feldern der Klassen Gesundheit und hochsensibel,
   - jedes Erweitern einer bestehenden Freigabe,
   - neue Geräte und Gerätewiderruf,
   - Exporte (Arzt-PDF, verschlüsselte Datensicherung),
   - Anzeige oder Eingabe von Recovery-Wörtern.
   Die Bestätigung ist ein nativer Dialog (nicht HTML), zeigt Empfänger, Felder, Zeitraum bzw.
   Gerätename im Klartext und verlangt Biometrie oder PIN.
4. **Entsperrstatus und Ablaufzeiten nativ.** `dak-core` führt die Sitzung (entsperrt bis,
   letzte Aktivität). Handles sind keine Berechtigung; jede Operation prüft den aktuellen Zustand.
5. **Hochsensible Felder** öffnet `dak-core` nur nach frischer Biometrie (Sitzung nur für diese
   Klasse, kurz), wenn die Person das eingestellt hat.
6. **Begrenzung:** Massenoperationen (alle Einträge eines Jahres öffnen) sind nur für Auswertung
   und Export vorgesehen und laufen über eigene, bestätigte Funktionen.

**Was das gegen kompromittierten WebView-Code bewirkt (T-15):** Er kann geheime Schlüssel nicht
direkt auslesen. Er kann angezeigte Klartexte gefährden sowie Daten, deren Zugriff die native API
im aktuellen Autorisierungszustand erlaubt. Neue Freigaben, Geräte und Exporte kann er nicht ohne
eine native Bestätigung der Person auslösen.

### 13.3 Recovery-Wörter

Recovery-Wörter werden ausschließlich in einer nativen Ansicht angezeigt und eingegeben (kein
WebView-Inhalt, Bildschirmaufnahme gesperrt). Die WebView erfährt nur, ob die Recovery
eingerichtet ist.

### 13.4 Fremde Inhalte in der Oberfläche

Feldbeschreibungen in Freigaben, geteilte Formularvorlagen, Kontaktnamen und Gerätenamen stammen
von anderen Personen; auch eigene Texte können aus einer manipulierten Datensicherung kommen.
Würde die WebView sie als HTML darstellen, könnte darüber Skript laufen, also genau der
kompromittierte WebView-Code, gegen den Abschnitt 13.2 schützt (T-29, M-26).

- `dak-core` prüft jede Zeichenkette aus Daten: gültiges UTF-8, Höchstlänge je Feldart, keine
  Steuerzeichen, keine Bidi-Steuerzeichen (verhindern verdrehte Anzeige), keine Nullbytes.
- Die Oberfläche setzt solche Texte ausschließlich als Text (`textContent`, kein `innerHTML`,
  kein Markdown, keine klickbaren Links).
- Content-Security-Policy ohne `'unsafe-inline'` und `'unsafe-eval'`, dazu Trusted Types.
- Geteilte Formularvorlagen enthalten nur Daten (Feldtyp, Beschriftung, Skala), nie Code oder
  Formeln. Bedingungen („nur fragen, wenn") sind eine feste, kleine Ausdrucksform, die
  `dak-core` auswertet.

## 14. Bibliotheken, Versionen und Updates

- Versionen sind fest gepinnt (`Cargo.lock`, `package-lock.json` mit Integritäts-Hashes).
- `AUDITS.md` führt je Bibliothek: gepinnte Version, bekannte Audits (wer, wann, welche Version,
  Umfang) und den Abstand zwischen auditierter und genutzter Version. „Auditiert" gilt nur für
  die geprüfte Version und den geprüften Umfang.
- Ein Update von libsodium, `libsodium-sys-stable`, UniFFI oder einer anderen
  sicherheitsrelevanten Abhängigkeit wird nur übernommen, wenn
  1. die Release- und Security-Hinweise gelesen und im PR zusammengefasst sind,
  2. die Testvektoren (libsodium-Referenz), die anwendbaren Wycheproof-Fälle und die
     Protokoll-Negativtests (falsche Signatur, fremdes Gerät, vertauschte Felder, alte Kette,
     abgelaufene Freigabe) grün sind,
  3. `AUDITS.md` nachgezogen ist.
  Ein grüner Lauf allein genügt nicht.

## Anhang A: Schlüsselableitung für tagesgenaue Freigaben (aus Teil A v0.5)

Nach der Entscheidung #96 steht diese Ableitung nicht mehr in Teil A. Sie ist ein Vorschlag für das
Freigabeformat von Teil B und wird mit M1.5 geprüft; vor der ersten Freigabe werden die Daten aus
dem Teil-A-Format lokal umgeschlüsselt. Offene Befunde dazu: Intervallwerte („seit dem letzten
Status") gegen tagesgenaue Freigaben, Datumsbereich vor 2026, Abschluss von Generationen (#50, #53).

### A.1 Generationen, Monate und Tage

Jede Messdefinition hat Schreibgenerationen. Eine Generation beginnt mit einem Zufallsschlüssel
`G_g` und wird **geschlossen**, sobald eine neue beginnt (in M1: nach einer Wiederherstellung auf
einem neuen Gerät und bei einem Gerätewechsel; in Teil B zusätzlich bei jedem Widerruf). In eine
geschlossene Generation wird nichts mehr geschrieben.

Innerhalb einer Generation werden Schlüssel nach Monat und Tag des Beobachtungsdatums abgeleitet
(Monat `e` gezählt ab Januar 2026, Tag `d` von 1 bis 31):

```
C(g, 0)     = KDF(G_g,      1, "dakgen__")      Monatskette, Start Januar 2026
C(g, e+1)   = KDF(C(g, e),  1, "dakchain")
T(g, e, 1)  = KDF(C(g, e),  4, "daktag__")      Tageskette im Monat
T(g, e, d+1)= KDF(T(g,e,d), 1, "daktag__")
V(g, e, d)  = KDF(T(g,e,d), 2, "dakwert_")      Wertschlüssel eines Tages
```

Damit sind spätere Freigaben **tagesgenau** möglich, ohne vorhandene Werte neu zu verschlüsseln
(#54): „ab dem 20. Oktober" = `T(g, Okt, 20)` plus `C(g, Nov)`; „nur 3. bis 5. März" = die drei
Wertschlüssel `V`. Wer eine Generation nicht hat, kann ihre Werte nicht lesen. In M1 wird nichts
freigegeben; die Ableitung ist trotzdem verbindlich, damit Teil B ohne Migration auskommt.
