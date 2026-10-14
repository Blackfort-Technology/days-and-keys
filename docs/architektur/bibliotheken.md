# Bibliotheken und Speicheraufbau für `dak-core`

Stand 2026-10-08. Grundlage: `docs/protokoll.md` Teil A v0.7.1 (Abschnitte 3, 6, 7, 8, 9, 10, 12,
13, 15, 16), `AUDITS.md` (Stand 2026-10-06), `docs/krypto-bibliotheken.md`, `docs/vorlagen.md`
im Repo `days-and-keys`. Alle Versionen, Daten und Lizenzen sind am 2026-10-08 live erhoben:
crates.io-API (`/api/v1/crates/<name>`), Quelltext der veröffentlichten Crates
(`static.crates.io`, nur gelesen), GitHub-API und `raw.githubusercontent.com`, npm-Registry,
Maven Central, OSV.dev (`/v1/query`, Ökosystem crates.io), Herstellerdoku.

Kennzeichnung: **[B]** belegt (Quelle in der Zeile oder im Quellenverzeichnis), **[S]**
Schlussfolgerung oder Schätzung, die der Prototyp (#81, #100, #101) bestätigen muss.

Bezug zur Framework-Bewertung: `architektur/framework.md` (gleicher Tag) empfiehlt native
Oberflächen (Compose und SwiftUI) mit `dak-core` über UniFFI, ohne WebView. Die Abschnitte 5 und 6
nennen deshalb beide Fälle; die Empfehlung folgt der nativen Variante, solange #33 nicht anders
entschieden ist.

## 0. Kurzfassung

| Thema | Empfehlung | Hauptrisiko in einem Satz |
|---|---|---|
| 1. Speicher | **SQLite über `rusqlite` 0.40.2 (`bundled`)**, eine Datenbankdatei, Tabellen ohne `rowid`, nur Chiffrate und Zufalls-IDs; Verläufe über verschlüsselte **Verlaufsblöcke** je Lebensmonat | Gelöschte Umhüllungen können physisch im Flash überleben, solange der Schlüssel darüber lebt; echtes Löschen gegen Forensik braucht Rotation oder Neuschreiben der Datei |
| 2. CBOR | **eigene strenge Kodierschicht** auf dem Low-Level-API von `minicbor` 2.3.0 (ohne Derive), Prüfung per Neu-Kodieren und Bytevergleich; `dcbor` 0.25.2 als Gegenprobe im Fuzzing | Der Kanonik-Check ist Eigenbau und damit so gut wie seine Tests |
| 3. Recovery-Wörter | **eigene BIP39-Kodierung** (Entropie zu Wörtern, Prüfsumme mit libsodium SHA-256), englische BIP39-Liste als Standard, deutsche Liste `de-2048` von dys2p als Option; `bip39` 3.0.0 nur als Testorakel | Eine nicht offizielle deutsche Liste bindet uns dauerhaft an eine eingefrorene Kopie; ändert sich die Liste, sind alte Wörter ungültig |
| 4. Arzt-PDF | **`krilla` 0.8.2** im Kern, Schrift Inter 4.2 eingebettet und unterteilt, Diagramme als Vektorpfade, PDF/A-2 und PDF/UA-1 | Seitenumbruch, Tabellen und Zeilenfall sind Handarbeit; wächst das Layout, wird `typst` als Bibliothek nötig und der Kern deutlich größer |
| 5. Hilfsbibliotheken | `zeroize` 1.9.1, eigene Geheimnis-Hülle auf `sodium_malloc`, `uniffi` 0.32.2, `jiff` 0.2.38 mit eingebauter Zeitzonendatenbank, `thiserror` 2.0.21, `proptest` 1.11.0, `cargo-fuzz` 0.13.2, Wycheproof aus `C2SP/wycheproof` (`testvectors_v1`) | UniFFI und Jiff sind beide vor 1.0; jedes Update kann die Schnittstelle ändern |
| 6. Diagramme in der Oberfläche | nativ: **Swift Charts** (iOS 16+) und **Vico** 3.3.1 (Compose); falls doch WebView: **uPlot** 1.6.32 | Bei nativen Diagrammen entstehen zwei Darstellungen derselben Auswertung; die Datenaufbereitung muss deshalb vollständig in `dak-core` liegen |

## 1. Speicher auf dem Gerät

### 1.1 Mengengerüst [S]

Annahme als obere Last: zehn Status je Tag, dazu etwa acht Ereignisse und zwei
Abfragen, also rund 20 Datensätze je Tag oder 7 300 je Jahr. Ein Status hat bis zu 20 Antworten
(Vorlagen Abschnitte 1 und 2).

| Objekt | Klartext (CBOR) | nach `sodium_pad` (Block 256) | Hülle (IDs, `umhuellung` 72 B, Nonce, Tag, Kopf) | gesamt |
|---|---|---|---|---|
| Status mit 20 Antworten | ca. 600 B (Antwort ca. 26 B: `messdef_id` 16 B, Zustand, Wert, `aktiv_gesetzt`, Map-Kopf) | 768 B | ca. 180 B | ca. 950 B |
| Ereignis, Abfrage | ca. 150 B | 256 B | ca. 180 B | ca. 440 B |

- Je Tag rund 14 KB, je Jahr rund 5 MB Nutzdaten; mit Seiten- und Indexaufwand von SQLite und
  einigen Revisionen grob **7 MB je Jahr, 70 MB und 73 000 Datensätze nach zehn Jahren**.
  Realistisch (drei bis vier Status am Tag) ist es ein Drittel davon.
- Antworten nach zehn Jahren bei voller Last: 10 × 20 × 3 650 = **730 000 Messpunkte**.

### 1.2 Kandidaten

| Kriterium | SQLite über `rusqlite` | eigenes Append-only-Format | `redb` | `fjall` | `sled` |
|---|---|---|---|---|---|
| Version, Datum [B] | rusqlite 0.40.2 (2026-08-08), libsqlite3-sys 0.38.2 bündelt SQLite **3.53.2**; aktuell ist SQLite 3.53.4 (2026-07-24) | - | 4.3.0 (2026-09-15), Rückport 2.6.4 (2026-10-02) | 3.1.12 (2026-10-03), MSRV 1.90 | stabil 0.34.7 (2021-09-12), sonst 1.0.0-alpha.124 (2024-10-11) |
| Lizenz [B] | MIT (Crate), SQLite gemeinfrei | eigene | MIT OR Apache-2.0 | MIT OR Apache-2.0 | MIT/Apache-2.0 |
| Pflege [B] | Push 2026-09-25, 4 431 Sterne | wir selbst | Push 2026-10-05, 3 offene Issues | Push 2026-10-05 | letzte stabile Version vor fünf Jahren; README: „if reliability is your primary constraint, use SQLite. sled is beta." |
| Atomarität | Transaktionen über beliebig viele Tabellen einer Datei; über mehrere angehängte Dateien im WAL-Modus **nicht** atomar [B: sqlite.org/wal.html] | selbst zu bauen (Schreibreihenfolge, `fsync`, Umbenennen) [S] | ACID, MVCC, Savepoints [B] | Batches und Transaktionen; Standard schreibt nur in OS-Puffer, nicht auf die Platte [B: README „Durability"] | Transaktionen, Beta |
| Absturzsicherheit | Absturz- und Stromausfalltests mit umsortierten und beschädigten Schreibzugriffen, 100 % Branch- und MC/DC-Abdeckung, OSS-Fuzz und dbsqlfuzz [B: sqlite.org/testing.html] | eigene Crash-Tests nötig, iOS braucht `F_FULLFSYNC` [S] | „Crash-safe by default", Copy-on-Write-B+-Bäume [B] | LSM mit Journal, Haltbarkeit muss per `persist` angefordert werden [B] | nicht belastbar belegt |
| Löschbarkeit | `PRAGMA secure_delete` überschreibt gelöschte Inhalte mit Nullen; `FAST` lässt Spuren auf Freelist-Seiten [B]; WAL wird nach Checkpoint überschrieben, nicht gekürzt, außer mit `journal_size_limit` [B] | Löschen heißt Kompaktieren, also die ganze Datei neu schreiben [S] | freigegebene Seiten werden nach der Design-Doku nicht genullt [B: `docs/design.md` beschreibt nur Freigabezustände, kein Überschreiben] | gelöschte Werte leben bis zur Kompaktierung in älteren Segmenten weiter (LSM-Prinzip) [S] | wie fjall [S] |
| Größe im Binary | SQLite-Amalgamation, grob 1-1,5 MB [S] | klein | klein, reines Rust [S] | mittel [S] | mittel [S] |
| Plattformen | Android, iOS, Desktop, Linux-CI; `bundled` baut dieselbe Version überall [B: Feature `bundled`] | überall | überall (reines Rust) [B] | überall; nicht aus mehreren Prozessen gleichzeitig [B] | überall |
| Testbarkeit | In-Memory-Datenbank, `PRAGMA integrity_check`, Werkzeuge zum Ansehen der Datei | alles selbst | gut (In-Memory-Backend) [S] | gut [S] | - |
| Formatstabilität | Dateiformat seit 2004 abwärtskompatibel [S, allgemein bekannt; im Prototyp nicht relevant] | wir selbst | README: „The file format is stable"; trotzdem Formatwechsel v1→v2 (redb 2.0) und v2→v3 (redb 3.0) mit Migrationspflicht [B: CHANGELOG] | „Future breaking changes will result in a major version bump" [B] | Formatänderungen vor 1.0 angekündigt [B] |
| Bekannte Advisories (OSV) [B] | rusqlite: RUSTSEC-2020-0014, RUSTSEC-2021-0128 und GHSA 2021/2022 (Use-after-free, Data Races in Feature-APIs), alle in alten Versionen; libsqlite3-sys: RUSTSEC-2022-0090 (CVE-2022-35737) | - | keine | keine | keine |

### 1.3 Empfehlung Speicher

**SQLite über `rusqlite` mit Feature `bundled`, eine Datei, WAL-Modus** [S].

Begründung: Rotation (Protokoll Abschnitt 9) und Migration (Abschnitt 15) verlangen, dass
Tausende Umhüllungen, Bund, Kapseln und Kettenglied in einem Schritt umschalten. Das leistet eine
SQLite-Transaktion ohne Eigenbau, und die Absturztests von SQLite sind die umfangreichsten, die
eine eingebettete Datenbank vorweisen kann [B: testing.html]. `secure_delete` ist die einzige
eingebaute Löschfunktion unter den Kandidaten [B]. Ein eigenes Append-only-Format verliert genau an
den zwei Stellen, die das Protokoll am stärksten fordert (atomares Umschalten, Löschen), und die
Key-Value-Speicher bringen entweder kein Überschreiben (redb), Reste bis zur Kompaktierung (fjall)
oder keinen stabilen Stand (sled) mit [S]. Zweite Wahl ist `redb`, falls SQLite im Prototyp an
Größe oder Bau scheitert [S].

**Aufbau** [S]:

- **Eine Datei für beide Bereiche.** Hochsensible Datensätze in eigenen Tabellen (`hs_*`), deren
  Inhalt nur unter `KK_h` bzw. `WS_h` lesbar ist. Grund: Die Rotation beider Generationen muss
  atomar sein, und Transaktionen über zwei angehängte Dateien sind im WAL-Modus nicht atomar [B].
  Was Forensik dabei sieht (Zahl und Zeitpunkte hochsensibler Einträge), nimmt Protokoll Abschnitt 1
  bereits in Kauf [B]. Wer zwei Dateien will, muss den Rollback-Journal-Modus nehmen.
- **Tabellen `WITHOUT ROWID` mit der zufälligen `revision_id` bzw. `datensatz_id` als
  Primärschlüssel.** Eine `rowid` würde die Einfügereihenfolge offenlegen und damit genau die
  zeitliche Ordnung, die zufällige 128-Bit-IDs verbergen sollen (Protokoll 7.1). Restrisiko:
  Seitenzuteilung und WAL-Rahmen spiegeln die Schreibreihenfolge grob wider [S].
- **Klartextspalten nur wie in Protokoll 7.1:** IDs, `art`, `klasse`, `formatversion`, `suite`;
  dazu `umhuellung`, `nonce`, `chiffrat` als BLOB. Keine Zeit, keine Objektart, keine
  `messdef_id`.
- Weitere Tabellen: `tombstone`, `bund`, `bund_h`, `kontokette` (Gliednummer als Schlüssel),
  `kapsel`, `kapsel_h`, `verlauf_block`, `hs_verlauf_block`, `meta` (Formatversion).
- **Planungsstand** (Protokoll Abschnitt 11, unverschlüsselt) in einer **eigenen kleinen Datei**,
  weil er vor dem Entsperren lesbar sein muss und auf iOS eine andere Datenschutzklasse braucht als
  die Hauptdatei [S, Prüfpunkt #67].
- Kompilieroptionen über `LIBSQLITE3_FLAGS` (die Build-Datei von libsqlite3-sys nimmt `-D` und
  `-U` an [B]): `-DSQLITE_SECURE_DELETE`, `-USQLITE_ENABLE_LOAD_EXTENSION`, FTS und JSON
  abschalten, soweit nicht gebraucht (das `bundled`-Profil schaltet `LOAD_EXTENSION`, FTS3/5, JSON1,
  RTREE ein [B: build.rs Zeilen 152-167]).
- Pragmas beim Öffnen: `secure_delete=ON` (nicht `FAST`), `journal_mode=WAL`,
  `journal_size_limit=0`, `temp_store=MEMORY`, `cell_size_check=ON`, `trusted_schema=OFF`,
  `foreign_keys=ON`; nach Löschung, Hochstufung und Rotation `wal_checkpoint(TRUNCATE)` [B für die
  Wirkung der Pragmas: sqlite.org/pragma.html; Auswahl S].
- SQLite-Version: 3.53.3 und 3.53.4 beheben Fehler aus 3.53.0 bis 3.53.2 [B: sqlite.org/changes.html].
  Vor #32 prüfen, ob ein libsqlite3-sys mit 3.53.4 erschienen ist; sonst diese Abweichung in
  `AUDITS.md` vermerken [S].
- SQLCipher (`bundled-sqlcipher`) scheidet aus: verschlüsselt Seiten mit OpenSSL bzw.
  CommonCrypto und verstößt gegen „ausschließlich libsodium" [S].

### 1.4 Löschen: was SQLite leistet und was nicht

- Logisch: Tombstone schreiben, Chiffrat und Umhüllung aller Revisionen entfernen (Protokoll 7.2);
  mit `secure_delete=ON` sind die Bytes in Datenbankseiten und Freelist genullt [B]. Im WAL stehen
  alte Seitenbilder, bis der Checkpoint mit `TRUNCATE` die Datei kürzt [B: wal.html, pragma.html].
- Physisch: Flash-Speicher überschreibt nicht an Ort und Stelle; alte Blöcke können auf dem Chip
  bleiben [S]. Solange `DS_g,k` lebt und der Angreifer den Gerätecode hat, kann eine solche
  Restkopie der Umhüllung den Inhalt wieder öffnen. Die Zusage in Protokoll 7.2 („auch mit allen
  Kontoschlüsseln nicht mehr zu öffnen") gilt damit für den aktiven Bestand, gegen
  forensische Auslesung erst nach einer Rotation von `k` [S].
- Vorschlag für #62 und #66 [S]: (a) Löschen dokumentiert diese Grenze; (b) optional „Löschen und
  rotieren" als eigener Knopf; (c) nach größeren Löschungen `VACUUM INTO` eine neue Datei und die
  alte löschen; ob das auf iOS (Schlüssel je Datei) und Android (fscrypt) die alte Datei
  kryptografisch unlesbar macht, prüft #67.

### 1.5 Verlaufsabfragen ohne Klartext-Index

**Alles im RAM entschlüsseln (Schätzung)** [S]:

- Eigene Messung als Größenordnung [B, auf einem Entwicklungsrechner]: OpenSSL 3.0.13
  ChaCha20-Poly1305: 2,76 GB/s bei 1-KB-Blöcken, 608 MB/s bei 64-B-Blöcken (etwa 0,1 µs
  je kleiner Operation). Das ist weder libsodium noch ein Telefon.
- Annahme Mittelklasse-Telefon: fünf- bis zehnmal langsamer [S]. Je Datensatz zwei AEAD-Öffnungen
  (Umhüllung 32 B, Inhalt rund 800 B) etwa 1-3 µs, CBOR-Prüfung 2-5 µs, Zeile lesen 2-5 µs [S].
- Ein Jahr (7 300 Datensätze): **unter 0,1 s**. Zehn Jahre (73 000): **0,5-1,5 s**, dazu 70 MB von
  Flash lesen [S]. Arbeitsspeicher: als kompakte Punkte (Zeit 8 B, Messreihe 2 B, Wert 2 B) für
  730 000 Antworten rund 9 MB; als Rust-Objekte mit `Vec` und `String` leicht das Zehnfache [S].
- Ergebnis: Für Tages- und Monatsansichten reicht Entschlüsseln im RAM. Für Verläufe über Jahre und
  für den Arzt-PDF-Zeitraum ist es grenzwertig und skaliert linear mit der Nutzungsdauer [S].

**Empfehlung: verschlüsselte Verlaufsblöcke** [S] (passt zu Protokoll Abschnitt 12: Indizes mit
Teilschlüssel aus `KK`, hochsensibel nur unter `KK_h`):

- Je Lebensmonat und Klasse ein Block mit kompakten Spalten: Datensatz-ID, Revision, Lebenstag,
  `zeitpunkt`, Objektart, Herkunft (selbst, Abfrage), dazu je Antwort Messreihen-Nummer (Tabelle der
  `messdef_id` im Blockkopf), Zustand und Wert. Größe bei voller Last rund 35-50 KB je Monat.
- Verschlüsselt mit XChaCha20-Poly1305 unter einem Teilschlüssel `KDF(KK, "dakindex")` (Kontext und
  Subkey-ID in `docs/format.md`, #63). Die Block-ID ist ein Keyed-BLAKE2b über Monat und Klasse mit
  einem zweiten Teilschlüssel, damit der Monat nicht im Klartext steht.
- Blöcke mit `sodium_pad` auf feste Stufen (etwa 16 KB) auffüllen, sonst verrät die Blockgröße die
  Aktivität je Monat.
- Ein Verlauf über zehn Jahre öffnet 120 Blöcke (rund 6 MB): **10-50 ms** statt einer Sekunde [S].
- Die Blöcke sind abgeleitete Daten: Sie kommen nicht in die Sicherung, werden nach
  Wiederherstellung und Rotation aus den Datensätzen neu gebaut (einmalig rund eine Sekunde bei zehn
  Jahren) und bei jeder Änderung in derselben Transaktion wie der Datensatz neu geschrieben.
- Löschen schreibt den betroffenen Block neu; der alte Block enthält den gelöschten Wert. Für ihn
  gilt dasselbe wie für Umhüllungen (1.4): `secure_delete` und Checkpoint sind Pflicht, physische
  Reste fallen erst mit einer Rotation weg [S].

### 1.6 Befund zur Sicherungsdatei (für #63)

Protokoll 10.1 verschlüsselt den Innenteil der Sicherung als **ein** XChaCha20-Poly1305-Chiffrat.
Bei 70 MB nach zehn Jahren heißt das: 70 MB Klartext beim Schreiben und Lesen am Stück im
Arbeitsspeicher, auf iOS auch in Hintergrundaufgaben mit engem Speicherlimit [S]. Vorschlag:
Innenteil mit `crypto_secretstream_xchacha20poly1305` in Blöcken (etwa 64 KB) und das Manifest als
ersten Block; libsodium bringt das mit [B: `docs/krypto-bibliotheken.md` Abschnitt 2.1]. Das ist
eine Formatfrage und gehört vor v1.0 entschieden, weil eine spätere Änderung eine neue
`formatversion` braucht.

**Hauptrisiko Speicher:** Gelöschte Inhalte und alte Verlaufsblöcke können physisch auf dem Flash
überleben und sind mit dem noch lebenden Klassenschlüssel lesbar. Das Protokoll muss diese Grenze
benennen und eine Rotation nach dem Löschen anbieten, sonst verspricht es mehr, als SQLite und das
Gerät leisten.

## 2. Deterministisches CBOR

### 2.1 Kandidaten

| Crate | Version, Datum [B] | Lizenz [B] | Pflege [B] | kanonisch kodieren | streng parsen |
|---|---|---|---|---|---|
| `ciborium` | 0.2.2 (2024-01-24) | Apache-2.0 | Push 2026-09-09, seit 2024 kein Release | nur im `Value`-Typ (`value/canonical.rs`), Sortierung **zuerst nach Länge** (RFC 7049 bzw. RFC 8949 4.2.3), nicht bytelexikografisch nach 4.2.1 [B: Quelltext] | keine Ablehnung von Duplikaten oder unbestimmten Längen gefunden [B: Quelltextsuche]; serde-Derive kennt keine ganzzahligen Map-Schlüssel [S] |
| `minicbor` | 2.3.0 (2026-07-23) | BlueOak-1.0.0 | Push 2026-09-27, 12 offene Issues | Low-Level-`Encoder` schreibt immer die kürzeste Ganzzahl; Map-Reihenfolge bestimmt der Aufrufer [S, aus Quelltext] | `Decoder` nimmt unbestimmte Längen und nicht minimale Ganzzahlen an (`u32()` akzeptiert `0x1a`/`0x1b` für kleine Werte) [B: decoder.rs]; Derive mit `#[cbor(map)]` und `#[n(k)]` erzeugt ganzzahlige Schlüssel, **ignoriert aber unbekannte Felder** [B: minicbor-derive 0.19.5, Doku Zeile 20] |
| `cbor4ii` | 1.2.3 (2026-09-07) | MIT | Push 2026-09-07, 89 Sterne | kein Kanonik-Modus gefunden [B: Quelltextsuche ohne Treffer] | kennt unbestimmte Längen als Fehlerart, keine Duplikatprüfung gefunden [B] |
| `serde_cbor` | 0.11.2 (2021-08-15) | MIT/Apache-2.0 | Repo archiviert, **RUSTSEC-2021-0127 „unmaintained"**, dazu RUSTSEC-2019-0025 (Stapelüberlauf) [B] | - | - |
| `serde_cbor_2` (kanidm-Fork) | 0.13.0 (2025-06-14) | MIT OR Apache-2.0 | ein Release seit 2022 [B] | kein Kanonik-Modus gefunden [B: nur `value/mod.rs` erwähnt „canonical"] | nicht geprüft |
| `dcbor` (Blockchain Commons) | 0.25.2 (2026-03-16) | BSD-2-Clause-Patent | README: „production-ready" [B] | CBOR Common Deterministic Encoding: kürzeste Zahlen, bytelexikografische Schlüssel, keine unbestimmten Längen, dazu numerische Reduktion, NFC-Pflicht für Texte [B: README] | lehnt `MisorderedMapKey`, `DuplicateMapKey`, `NonCanonicalNumeric`, `NonCanonicalString` ab [B: error.rs]; eigener Wertebaum, neun Abhängigkeiten (u. a. chrono, num-bigint, hashbrown, spin) [B: Cargo.toml] |
| `coset` (Google, COSE) | 0.4.2 (2026-03-02) | Apache-2.0 | aktiv | für COSE-Strukturen, nicht für eigene Objekte | - |

Für keinen dieser Crates habe ich ein öffentliches Audit gefunden (Websuche 2026-10-08) [B: Suche
ohne Treffer]. OSV meldet nur für `serde_cbor` Einträge [B].

### 2.2 Empfehlung CBOR

**Eigene strenge Kodierschicht in `dak-core` auf dem Low-Level-API von `minicbor`, ohne Derive**
[S]:

1. Je Objektart eine handgeschriebene `encode`/`decode`-Funktion mit fester Schlüsselmenge;
   unbekannter Schlüssel, fehlender Pflichtschlüssel oder falscher Typ ist ein Fehler (Protokoll
   Abschnitt 3).
2. **Kanonik-Check per Rundlauf:** nach dem Dekodieren neu kodieren und Byte für Byte mit der
   Eingabe vergleichen. Das fängt nicht minimale Ganzzahlen, unbestimmte Längen, falsche
   Schlüsselreihenfolge und Duplikate (bei strikt steigender Reihenfolge unmöglich) mit einer
   einzigen Regel ab.
3. Vor dem Parsen: Höchstgröße je Objektart, Verschachtelungstiefe begrenzen, keine Floats, keine
   Tags, keine Simple Values außer `true`/`false`/`null`, falls nicht ausdrücklich gebraucht.
4. **Map-Schlüssel nur 0 bis 23.** Dann sind sie ein Byte lang, und die Reihenfolge nach RFC 8949
   4.2.1 (bytelexikografisch) und nach 4.2.3 (Länge zuerst) fallen zusammen [S]; ein Fehler bei
   der Sortierregel kann so nicht entstehen. Mehr als 24 Felder in einem Objekt deuten ohnehin auf
   ein Objekt hin, das geteilt gehört.
5. **Gegenprobe:** `dcbor` als reine Testabhängigkeit; ein Fuzz-Ziel prüft, dass beide Decoder
   dieselben Eingaben annehmen und ablehnen (Differenzial-Fuzzing). Abweichungen sind entweder
   dCBOR-Zusatzregeln (NFC, numerische Reduktion) oder ein Fehler in unserer Schicht.

Warum nicht `dcbor` direkt: Er erfüllt die Strenge schon, bringt aber einen eigenen Wertebaum mit
Allokationen je Knoten, neun Abhängigkeiten und Zusatzregeln über 4.2.1 hinaus mit, die das
Protokoll nicht verlangt [S]. Für ein Format mit festen Feldern ist eine kleine eigene Schicht
besser prüfbar. Warum `minicbor`: kleinster Encoder/Decoder ohne Abhängigkeiten (nur optional
`half`) [B: Cargo.toml], aktiv gepflegt, `no_std`-fähig. Lizenz BlueOak-1.0.0 ist freizügig, wird
aber nicht von jedem Lizenzscanner erkannt; in `cargo-deny` freigeben [S].

**Hauptrisiko CBOR:** Die Strenge liegt in unserem eigenen Code, nicht in einer geprüften
Bibliothek. Ohne Negativtests aus `docs/format.md` (#63) und Fuzzing gegen `dcbor` bleibt offen, ob
der Parser wirklich alles ablehnt, was er ablehnen soll.

## 3. Recovery-Wörter

### 3.1 Befunde

- `bip39` (rust-bitcoin) 3.0.0 vom 2026-09-17, CC0-1.0, Push 2026-09-17; Sprachen: Englisch,
  Chinesisch (vereinfacht, traditionell), Tschechisch, Französisch, Italienisch, Japanisch,
  Koreanisch, Portugiesisch, Spanisch; **kein Deutsch** [B: Cargo-Features, `src/language/`].
  Abhängigkeiten: `bitcoin_hashes`, `unicode-normalization`, `zeroize`, optional `rand` [B].
- `tiny-bip39` 2.0.0 vom 2024-10-07, MIT AND (MIT OR Apache-2.0), seit zwei Jahren ohne Release,
  ebenfalls ohne Deutsch [B].
- Die offiziellen BIP39-Wortlisten sind die zehn oben genannten; Deutsch gibt es nicht [B:
  `bitcoin/bips` `bip-0039/bip-0039-wordlists.md`]. Deutsche Listen wurden mehrfach vorgeschlagen
  (PR #721 2018, #942 2020, #1071 2021); #1922 wurde am 2025-08-12 mit der Begründung geschlossen,
  die Autoren unterstützten keine weiteren Listen über Englisch hinaus [B: Suchergebnis, Spiegel
  mirror.b10c.me].
- Gepflegte deutsche Liste außerhalb von BIP39: **dys2p/wordlists-de**, Liste `de-2048` mit 2 048
  Wörtern, 4-8 Buchstaben, **ohne ä, ö, ü, ß**, eindeutig ab dem vierten Buchstaben, nur
  Grundformen, ohne Namen und negativ besetzte Wörter; wahlweise Unlicense, CC0 oder BSD-3 [B:
  Repo-README].
- Days & Keys braucht keine Kompatibilität zu Wallets: `RK` sind 256 Bit Entropie, daraus leitet
  `crypto_kdf` die Paare ab (Protokoll Abschnitt 4). Die PBKDF2-Ableitung von BIP39 (Seed aus
  Wörtern) wird nicht gebraucht [S].

### 3.2 Empfehlung Recovery-Wörter

**Eigene kleine BIP39-Kodierung in `dak-core`** [S]: 256 Bit Entropie plus 8 Bit Prüfsumme (erstes
Byte von SHA-256 über die Entropie, `crypto_hash_sha256` aus libsodium) ergeben 24 Indizes zu je
11 Bit. Das sind rund 60 Zeilen ohne neue Abhängigkeit; die Wortlisten sind eingebettete Daten mit
festgehaltenem SHA-256 in `docs/format.md`.

- **Standard: englische BIP39-Liste** (offiziell, eingefroren, gut geprüft). **Option: `de-2048`**
  von dys2p, weil die Wörter ohne Umlaute auskommen und ab vier Buchstaben eindeutig sind; das passt
  zur Eingabe in einer nativen Ansicht mit eigener Vervollständigung [S].
- Die gewählte Liste wird im Gründungsobjekt nicht gespeichert (das verriete nichts, nützt aber
  auch nichts); die Eingabe erkennt die Sprache am ersten Wort, sofern sich die Listen nicht
  überschneiden. Überschneidung prüfen, bevor `de-2048` aufgenommen wird [S].
- **Keine eigene Wortliste erfinden.** Die Arbeit (Levenshtein-Abstände, keine Homophone, keine
  belasteten Wörter) ist bei dys2p schon gemacht und frei lizenziert [S].
- `bip39` 3.0.0 nur als Entwicklungsabhängigkeit: Vergleich unserer Kodierung mit der englischen
  Liste über alle Testvektoren (`vectors.json` von Trezor) und Property-Tests [S].

**Hauptrisiko Recovery-Wörter:** Eine nicht offizielle deutsche Liste ist ein Stück Format, das wir
selbst für immer stabil halten müssen; jede Änderung an Liste, Normalisierung oder Prüfsumme macht
ausgegebene Wörter ungültig. Deshalb Liste als Datei mit Hash ins Format aufnehmen und nie ändern,
neue Listen nur als neue Variante.

## 4. PDF für die Ärztin

### 4.1 Kandidaten

| Crate | Version, Datum [B] | Lizenz [B] | Befund |
|---|---|---|---|
| `krilla` | 0.8.2 (2026-06-04), MSRV 1.92, Push 2026-10-08 | MIT OR Apache-2.0 | High-Level-API über `pdf-writer` (Pfade, Füllungen, Glyphen, Bilder) [B: README]; Prüfprofile **PDF/A-1 bis A-4 und PDF/UA-1**, Fehlerarten wie `MissingTagging`, `NoDocumentLanguage` [B: `src/configure/validate.rs`]; Schriftunterteilung über `subsetter` [B: Cargo.toml]; Grundlage von `typst-pdf` [B: Abhängigkeiten von typst-pdf 0.15.1] |
| `typst` als Bibliothek | 0.15.1 (2026-07-17), MSRV 1.92 | Apache-2.0 | voller Satz mit Tabellen, Seitenumbruch, Silbentrennung; `typst-pdf` nutzt `krilla` und `krilla-svg` [B]; braucht eine eigene `World`-Implementierung, viele Unter-Crates (`typst-library`, `typst-layout`, `typst-assets` …) [B]; Binärgröße mehrere MB [S] |
| `printpdf` | 0.12.8 (2026-09-05), MSRV 1.88 | MIT | Schriften, SVG, Unterteilung; HTML-Layout „still evolving" [B: README]; viele Abhängigkeiten (azul-layout, allsorts, lopdf, image, rust-fontconfig, web-sys) [B: Cargo.toml] |
| `genpdf` | 0.2.0 (2021-06-17) | Apache-2.0 OR MIT | seit über fünf Jahren ohne Release [B]; nicht nehmen |
| `pdf-writer` | 0.15.0 (2026-05-27) | MIT OR Apache-2.0 | Low-Level, Basis von krilla [B] |
| `hayro` | 0.8.0 (2026-10-04) | Apache-2.0 OR MIT | PDF-Leser und -Renderer desselben Autors [B]; nützlich für Tests (Seite rendern, Text prüfen) [S] |

Für keinen dieser Crates habe ich ein Audit gefunden [B: Suche ohne Treffer]; OSV meldet nichts [B].

### 4.2 Empfehlung PDF

**`krilla` 0.8.2 im Kern** [S]. Das PDF entsteht in `dak-core` aus den entschlüsselten Datensätzen;
Klartext verlässt den Kern nur als fertige Datei an das native Teilen (nach nativer Bestätigung,
Protokoll Abschnitt 13). In der nativen Variante gibt es keine WebView; in einer WebView-Variante
bliebe so das Arzt-PDF ohnehin außerhalb der Oberfläche.

- **Schrift:** Inter 4.2 (SIL OFL 1.1) [B: rsms/inter `LICENSE.txt`, `version.txt`], dieselbe
  Schrift wie die App (`docs/gesamtkonzept.md` Abschnitt 6), per `include_bytes!` eingebettet,
  automatisch unterteilt. Umlaute und ß sind in Inter enthalten [S, beim ersten Test-PDF prüfen].
  Keine Systemschriften laden, damit das PDF auf allen Geräten gleich aussieht.
- **Diagramme:** als Vektorpfade direkt mit krilla (`PathBuilder`, Linien, Punkte, Achsen): einzelne
  Status als Punkte, je Lebenstag Spanne und Zahl der Status, Einnahmezeiten als Marken; keine
  Glättung, keine Farbwertung (Protokoll 14, Vorlagen Abschnitt 8). Eine eigene Plot-Bibliothek ist
  dafür nicht nötig; `plotters` 0.3.7 ist seit 2024-09 ohne Release [B].
- **Barrierefreiheit und Archiv:** Profil **PDF/A-2a oder PDF/UA-1** mit Tagging, Dokumentsprache
  `de`, Titel; Diagramme mit Alternativtext und daneben die Werte als Tabelle [S]. Viele
  Praxisverwaltungen archivieren PDF/A [S].
- **Metadaten:** keine Gerätenamen, keine App-Version mit Build-Hash, Erstellungsdatum nur als Tag
  [S].
- **Vorbehalt typst:** Wird das PDF umfangreich (mehrseitige Tabellen, Fließtext mit Umbruch über
  Seiten), ist `typst` als Bibliothek die nächste Stufe. Dann Daten nur über `sys.inputs` bzw. eine
  JSON-Datei in der `World` übergeben, **nie** Freitext in Typst-Quelltext einsetzen, sonst führt
  ein `#` im Freitext Code aus [S].
- Tests: Golden-PDFs mit festem Zeitstempel, Rendern mit `hayro` und Pixelvergleich, Textauszug auf
  verbotene Felder prüfen (Freitext und Angaben über Dritte nur nach Auswahl, Protokoll 14) [S].

**Hauptrisiko PDF:** krilla ist 0.x, und Seitenumbruch, Tabellen und Zeilenfall sind eigener Code.
Wächst der Inhalt, steigen Aufwand und Fehlerquellen, bis `typst` nötig wird, das den Kern um
mehrere Megabyte vergrößert und eine eigene Eingabeschnittstelle braucht.

## 5. Hilfsbibliotheken

| Thema | Empfehlung | Belege und Begründung |
|---|---|---|
| Speicherlöschung | `zeroize` 1.9.1 (2026-10-06, Apache-2.0 OR MIT) | Teil von RustCrypto, nutzt `write_volatile` und Compiler-Barrieren [B: README]. 1.9.1 behebt u. a. ein mögliches Lesen von uninitialisiertem Speicher im Fallback von `optimization_barrier` und ersetzt das `Debug` von `Zeroizing` durch eine undurchsichtige Ausgabe [B: CHANGELOG]. Release ist erst zwei Tage alt; vor #32 Changelog und Issues erneut prüfen [S] |
| Geheimnis-Typen | **eigene Hülle** auf `sodium_malloc`/`sodium_mprotect_noaccess`/`sodium_free` für langlebige Schlüssel (`KK`, `DS_g,k`, `WS_*`), `zeroize::Zeroizing` für kurzlebige Puffer | libsodium bringt Schutzseiten, `mlock` und Kanarienwerte mit [S, libsodium-Doku „Secure memory"]; `secrecy` 0.10.3 (2024-10-09, Apache-2.0 OR MIT) bietet nur `SecretBox` mit `zeroize` und kein `mlock` [B: README]; daher nicht nötig [S]. `sodium_malloc` ist langsam und durch `RLIMIT_MEMLOCK` begrenzt, also nur für wenige Schlüssel [S] |
| Bindings | `uniffi` 0.32.2 (2026-09-23, MPL-2.0) | „ready for production use, but … a long way from a 1.0 release"; genutzt in Firefox mobil und Desktop, Kotlin und Swift [B: README]. MPL-2.0 ist Datei-Copyleft, für eine App unkritisch [S]. Fehler als `enum` mit `#[derive(uniffi::Error)]`, ohne Datenfelder, die Klartext enthalten könnten [S] |
| Zeit, Zeitzonen | `jiff` 0.2.38 (2026-10-06, Unlicense OR MIT) mit Feature `tzdb-bundle-always` | Bündelt IANA 2026e [B: CHANGELOG], liest auf Android die verkettete Zeitzonendatenbank [B: PLATFORM.md]; iOS ist dort nicht eigens erwähnt [B]. Eingebettete Datenbank macht Lebenstag-Berechnung und Planer auf allen Geräten gleich; die Zonen-ID liefert die native Schicht [S]. Kein 1.0-Termin: „I don't currently have a timeline for a Jiff 1.0 release" [B: README]. `chrono-tz` 0.10.4 (2025-07-11) bündelt ältere Zeitzonendaten [B: Datum]; `time` 0.3.55 hat keine IANA-Datenbank [S] und zuletzt RUSTSEC-2026-0009 (Stapelüberlauf, 2026-02) [B: OSV] |
| Zufall | libsodium `randombytes_buf` | nutzt auf Linux `getrandom`, auf Windows `RtlGenRandom`, sonst `/dev/urandom`; Android und Apple nennt die Doku nicht ausdrücklich [B: libsodium-Doku]. Nach VM-Snapshots kann sich die Ausgabe wiederholen [B]; für Telefone ohne Belang [S]. `sodium_init()` einmal aufrufen und Rückgabewert prüfen; keine zweite Zufallsquelle im Kern [S] |
| Fehler | `thiserror` 2.0.21 (2026-09-23) für die Fehlertypen des Kerns; `anyhow` 1.0.104 nur in Werkzeugen und Tests | Fehler ohne `Debug`-Ausgabe von Inhalten; Fehlercodes stabil, damit die Oberfläche sie übersetzen kann [S] |
| Fuzzing | `cargo-fuzz` 0.13.2 (2026-06-09), `libfuzzer-sys` 0.4.13, `arbitrary` 1.4.2 | Ziele: CBOR-Decoder je Objektart, Datensatz-Hülle, Sicherungsdatei (außen, Kapsel, Manifest), Kontokette, Migration, Recovery-Wort-Eingabe, Differenzial gegen `dcbor` [S]. `arbitrary` 1.5.0 erschien erst am 2026-10-06; bis zur Prüfung 1.4.2 (2025-08-14) pinnen [S]. cargo-fuzz braucht Nightly; nur im CI-Job, nicht im Release-Bau [S] |
| Property-Tests | `proptest` 1.11.0 (2026-03-24, MIT OR Apache-2.0) | Rundlauf CBOR, Kanonik-Eigenschaft, Revisionen und Tombstones (Wiederherstellung wendet alle an), Lebenstag über Sommerzeit und Zonenwechsel, Rotation bricht an beliebiger Stelle ab und der alte Bestand bleibt nutzbar [S] |
| Wycheproof | `C2SP/wycheproof`, Ordner `testvectors_v1` | `xchacha20_poly1305_test.json` (315 Fälle, Algorithmus XCHACHA20-POLY1305), `x25519_test.json` (518, XDH), `ed25519_test.json` (151, EDDSA) [B: Dateien am 2026-10-08 gelesen]; das alte Repo `google/wycheproof` leitet um [B: GitHub-API „Moved Permanently"]. Commit-Hash pinnen und Dateien ins Repo kopieren, statt sie im CI zu laden [S]. Ergänzend die Randfälle `ed25519-speccheck` (`cases.json`) für Small-Order- und nicht kanonische Punkte [B: Datei erreichbar]. Der Crate `wycheproof` 0.7.0 (2026-09-10, Apache-2.0) ist ein bequemer Lader, aber nicht nötig [S] |

`crypto_kdf`, `crypto_box_seal`, `sodium_pad`, Argon2id und `crypto_secretstream` haben in
Wycheproof keine Vektoren [S]; dafür eigene Known-Answer-Tests aus libsodium selbst erzeugen und
in `docs/format.md` (#63) festhalten.

**Hauptrisiko Hilfsbibliotheken:** UniFFI und Jiff sind beide vor 1.0 und ändern ihre Schnittstellen
noch. Updates nur gezielt und mit gelesenem Changelog (Protokoll Abschnitt 16), sonst bricht die
Bridge oder verschiebt sich eine Zeitberechnung unbemerkt.

## 6. Diagramme in der Oberfläche

Welche Bibliothek in Frage kommt, hängt an #33 [S]. Die Datenaufbereitung (Punkte, Spannen je Lebenstag, Zahl
der Status, Trennung selbst und Abfrage, Vorher/Nachher mit Erfassungsdichte) liegt in jedem Fall
in `dak-core`; die Oberfläche zeichnet nur, was der Kern liefert [S].

**Native Oberflächen (Empfehlung aus `framework.md`):**

| Bibliothek | Stand [B] | Lizenz | Hinweise |
|---|---|---|---|
| Swift Charts (Apple) | Systemframework ab iOS 16.0, macOS 13 [B: developer.apple.com Doku-Metadaten] | Teil des SDK | kein Fremdcode, kein Netz; VoiceOver-Unterstützung und Audio Graphs über `AXChartDescriptor` [S, im Prototyp prüfen] |
| Vico (Compose) | `com.patrykandpatrick.vico:compose` **3.3.1** (Maven Central, 2026-08-28), Push 2026-10-08, 3 189 Sterne, 6 offene Issues | Apache-2.0 | „extensible chart library for Compose Multiplatform" [B: README]; TalkBack-Beschreibungen selbst setzen [S] |

**Falls doch eine WebView kommt:**

| Bibliothek | Version, Datum [B] | Lizenz [B] | Größe min. / gzip [B, am 2026-10-08 gemessen] | Barrierefreiheit |
|---|---|---|---|---|
| **uPlot** | 1.6.32 (2025-03-14), Push 2026-10-05 | MIT | 51 KB / 22 KB | Canvas ohne eigene Hilfen [S]; Werte als Tabelle daneben nötig |
| Chart.js | 4.5.1 (2025-10-13) | MIT | 209 KB / 71 KB | Canvas; zugänglich nur über `role="img"`, `aria-label` und Fallback-Inhalt, „up to the user" [B: chartjs.org Accessibility] |
| Apache ECharts | 6.1.0 (2026-05-19) | Apache-2.0 | 1 122 KB / 369 KB (voll), 500 KB / 169 KB (simple) | `aria`-Modul erzeugt Beschreibungen und Muster statt Farbe, standardmäßig aus [B: ECharts-Handbuch] |

Empfehlung WebView-Fall: **uPlot** [S], weil es mit Hunderttausenden Punkten schnell bleibt, am
kleinsten ist und keine Abhängigkeiten hat; Barrierefreiheit über eine Textzusammenfassung und
eine Wertetabelle aus `dak-core`, die ohnehin für das PDF entsteht. Keine der drei lädt von sich aus
etwas aus dem Netz, solange sie lokal gebündelt werden [S].

**Hauptrisiko Diagramme:** Mit zwei nativen Oberflächen gibt es zwei Zeichnungen derselben
Auswertung, die auseinanderlaufen können. Gegenmittel: Kern liefert fertige Diagrammmodelle
(Achsen, Punkte, Beschriftungen), und Screenshot-Tests vergleichen beide Plattformen mit dem PDF.

## 7. Tabelle für `AUDITS.md`

Vorschlag zum Nachtragen; „Version zum Pinnen" ist der Stand vom 2026-10-08 und wird mit #32
bestätigt. Zeilen für libsodium und libsodium-sys-stable stehen bereits in `AUDITS.md` und bleiben
unverändert.

| Bibliothek | Rolle | Version zum Pinnen | Lizenz | Audit-Stand |
|---|---|---|---|---|
| rusqlite | Speicher | 0.40.2 (2026-08-08) | MIT | kein externes Audit gefunden; Prüfung des `unsafe`-Codes durch die Maintainer führte 2020 zu 0.23.0 mit mehreren Speichersicherheits-Korrekturen [B: Release-Notes 0.23.0]; RUSTSEC-2020-0014, RUSTSEC-2021-0128 in alten Versionen |
| libsqlite3-sys (`bundled`) | SQLite 3.53.2 | 0.38.2 (2026-08-08); Ziel SQLite ≥ 3.53.4 | MIT, SQLite gemeinfrei | kein externes Audit; 100 % Branch- und MC/DC-Abdeckung, OSS-Fuzz, dbsqlfuzz, Absturztests [B: sqlite.org/testing.html]; RUSTSEC-2022-0090 (CVE-2022-35737) historisch |
| minicbor | CBOR-Kodierung (Low-Level) | 2.3.0 (2026-07-23) | BlueOak-1.0.0 | kein Audit gefunden; OSV leer |
| dcbor | nur Tests: Differenzial-Fuzzing | 0.25.2 (2026-03-16) | BSD-2-Clause-Patent | kein Audit gefunden; OSV leer |
| bip39 | nur Tests: Orakel für Wortkodierung | 3.0.0 (2026-09-17) | CC0-1.0 | kein Audit gefunden; OSV leer |
| Wortlisten | BIP39 Englisch, dys2p `de-2048` | Dateien mit SHA-256 in `docs/format.md` | BIP39: Teil des BIP-Repos; dys2p: Unlicense OR CC0 OR BSD-3 | keine Audits (Daten) |
| krilla | Arzt-PDF | 0.8.2 (2026-06-04) | MIT OR Apache-2.0 | kein Audit gefunden; OSV leer |
| Inter | Schrift im PDF | 4.2 | OFL-1.1 | entfällt |
| hayro | nur Tests: PDF rendern | 0.8.0 (2026-10-04) | Apache-2.0 OR MIT | kein Audit gefunden |
| uniffi | Bindings Kotlin/Swift | 0.32.2 (2026-09-23) | MPL-2.0 | keine bekannt (wie bisher) |
| zeroize | Löschen von Geheimnissen | 1.9.1 (2026-10-06) | Apache-2.0 OR MIT | keine eigenen bekannt (wie bisher) |
| jiff | Zeit, Zeitzonen, Lebenstag | 0.2.38 (2026-10-06), IANA 2026e | Unlicense OR MIT | kein Audit gefunden; OSV leer |
| thiserror | Fehlertypen | 2.0.21 (2026-09-23) | MIT OR Apache-2.0 | kein Audit gefunden; OSV leer |
| proptest | nur Tests | 1.11.0 (2026-03-24) | MIT OR Apache-2.0 | entfällt (Testwerkzeug) |
| cargo-fuzz, libfuzzer-sys, arbitrary | nur Fuzzing | 0.13.2, 0.4.13, 1.4.2 | MIT OR Apache-2.0; libfuzzer-sys zusätzlich NCSA | entfällt (Testwerkzeug) |
| Wycheproof | Testvektoren | `C2SP/wycheproof` `testvectors_v1`, Commit pinnen | Apache-2.0 | entfällt (Daten) |
| ed25519-speccheck | Randfälle Ed25519 | `cases.json`, Commit pinnen | Apache-2.0 [S, beim Kopieren prüfen] | entfällt (Daten) |
| Swift Charts / Vico | Diagramme nativ | iOS-SDK / 3.3.1 | Apple SDK / Apache-2.0 | keine bekannt |
| uPlot | nur bei WebView | 1.6.32 | MIT | keine bekannt |
| **nicht vorgesehen** | | | | `serde_cbor` (RUSTSEC-2021-0127 unmaintained), `ciborium` (keine Kanonik nach 4.2.1), `sled` (Beta), `fjall` (Löschen erst bei Kompaktierung), `genpdf` (seit 2021 still), `secrecy` (durch eigene Hülle ersetzt), `chrono-tz` (ältere Zeitzonendaten), SQLCipher (fremde Kryptografie) |

## 8. Offene Punkte für Protokoll und Prototyp

| Punkt | Wohin |
|---|---|
| Grenze des Löschens gegen Forensik benennen, „Löschen und rotieren" anbieten | #62, #66 |
| Sicherung als `crypto_secretstream` in Blöcken statt eines einzigen AEAD | #63 |
| Verlaufsblöcke: Teilschlüssel, Block-ID, Auffüllstufen in die KDF- und Formattabelle | #63 |
| Map-Schlüssel 0 bis 23 als Formatregel | #63 |
| Laufzeiten aus 1.5 auf Zielgeräten messen (Entschlüsseln, Rotation von 73 000 Umhüllungen in einer Transaktion) | #81, #101 |
| iOS-Datenschutzklassen für Hauptdatei, Planerdatei und WAL; Wirkung von `VACUUM INTO` plus Löschen | #67 |

## Quellen

- crates.io-API: https://crates.io/api/v1/crates/rusqlite , …/libsqlite3-sys , …/redb , …/fjall ,
  …/sled , …/minicbor , …/cbor4ii , …/ciborium , …/serde_cbor , …/dcbor , …/bip39 , …/krilla ,
  …/typst , …/printpdf , …/genpdf , …/uniffi , …/zeroize , …/secrecy , …/jiff , …/chrono-tz ,
  …/thiserror , …/proptest , …/cargo-fuzz , …/arbitrary , …/wycheproof (je 2026-10-08)
- Crate-Quelltext (gelesen): https://static.crates.io/crates/libsqlite3-sys/libsqlite3-sys-0.38.2.crate
  (`sqlite3/sqlite3.h`, `build.rs`), …/minicbor-2.3.0.crate (`decode/decoder.rs`),
  …/minicbor-derive-0.19.5.crate (`src/lib.rs`), …/ciborium-0.2.2.crate (`value/canonical.rs`),
  …/dcbor-0.25.2.crate (`src/error.rs`, README), …/bip39-3.0.0.crate (`Cargo.toml`),
  …/redb-4.3.0.crate (README, CHANGELOG, `docs/design.md`), …/krilla-0.8.2.crate
  (`src/configure/validate.rs`), …/jiff-0.2.38.crate (`PLATFORM.md`, README)
- SQLite: https://www.sqlite.org/pragma.html , https://www.sqlite.org/wal.html ,
  https://www.sqlite.org/testing.html , https://www.sqlite.org/chronology.html ,
  https://www.sqlite.org/changes.html
- sled-Warnung: https://github.com/spacejam/sled#known-issues-warnings ; fjall-Haltbarkeit:
  https://github.com/fjall-rs/fjall#durability
- OSV: https://api.osv.dev/v1/query (Ökosystem crates.io, 2026-10-08)
- rusqlite 0.23.0: https://newreleases.io/project/cargo/rusqlite/release/0.23.0
- BIP39-Wortlisten: https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md ;
  geschlossener Vorschlag: https://mirror.b10c.me/bitcoin-bips/1922/ ; frühere Vorschläge:
  https://mirror.b10c.me/bitcoin-bips/721/ , https://mirror.b10c.me/bitcoin-bips/942/ ,
  https://mirror.b10c.me/bitcoin-bips/1071/ ; deutsche Liste: https://github.com/dys2p/wordlists-de
- rust-bip39: https://github.com/rust-bitcoin/rust-bip39
- krilla: https://github.com/LaurenzV/krilla ; typst: https://github.com/typst/typst ; printpdf:
  https://github.com/fschutt/printpdf ; hayro: https://github.com/LaurenzV/hayro
- Inter: https://github.com/rsms/inter (`LICENSE.txt`, `version.txt`)
- UniFFI: https://github.com/mozilla/uniffi-rs ; zeroize: https://github.com/RustCrypto/utils/tree/master/zeroize
  (CHANGELOG); secrecy: https://github.com/iqlusioninc/crates/tree/main/secrecy ; jiff:
  https://github.com/BurntSushi/jiff (CHANGELOG, PLATFORM.md)
- libsodium Zufall: https://doc.libsodium.org/generating_random_data
- Wycheproof: https://github.com/C2SP/wycheproof/tree/main/testvectors_v1 ;
  ed25519-speccheck: https://github.com/novifinancial/ed25519-speccheck
- Diagramme: https://github.com/leeoniya/uPlot , https://www.chartjs.org/docs/latest/general/accessibility.html ,
  https://echarts.apache.org/handbook/en/best-practices/aria/ , https://github.com/patrykandpatrick/vico ,
  https://repo1.maven.org/maven2/com/patrykandpatrick/vico/compose/maven-metadata.xml ,
  https://developer.apple.com/documentation/charts ; Größen gemessen an
  https://cdn.jsdelivr.net/npm/uplot@1.6.32/dist/uPlot.iife.min.js ,
  https://cdn.jsdelivr.net/npm/chart.js@4.5.1/dist/chart.umd.min.js ,
  https://cdn.jsdelivr.net/npm/echarts@6.1.0/dist/echarts.min.js
