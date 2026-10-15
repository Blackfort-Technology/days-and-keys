# Formate von Teil A, bytegenau (#63)

Stand 2026-10-14, Formatversion 1, Suite `dak-s1` (Kennung 1). Gilt für `docs/protokoll.md` v1.0.
Maßgeblich ist der Code in `core/crates/dak-core`; jede Abweichung zwischen dieser Datei und dem
Code ist ein Fehler. Die Testvektoren in `core/testvectors/` belegen die Tabellen (Abschnitt 9).

## 1. Kodierung

Deterministisches CBOR nach RFC 8949, Abschnitt 4.2.1 (`cbor.rs`):

| Regel | Durchsetzung |
|---|---|
| Ganze Zahlen in kürzester Form, positiv (Haupttyp 0) und negativ (1) | Lesen lehnt längere Formen ab |
| Bytefolgen (2), Text (3, gültiges UTF-8), Felder (4), Maps (5), `false`, `true`, `null` | alles andere wird abgelehnt: Fließkomma, Tags, `undefined`, einfache Werte |
| nur bestimmte Längen | unbestimmte Längen werden abgelehnt |
| Map-Schlüssel sind ganze Zahlen 0 bis 23, streng aufsteigend | doppelte, absteigende, größere oder Text-Schlüssel werden abgelehnt |
| unbekannte Schlüssel in einem Objekt | werden abgelehnt (`MapLeser::ende`) |
| Schachtelungstiefe höchstens 16, Objekt höchstens 64 MiB | sonst Fehler |
| keine Bytes nach dem Wert | sonst Fehler |

Jedes Objekt beginnt mit `0: typ` und `1: formatversion` (= 1). Objekte, die außerhalb eines
Chiffrats stehen oder signiert werden, tragen zusätzlich `2: suite` (= 1). Eine unbekannte
Formatversion führt zum Fehler 700 („bitte die App aktualisieren"); verworfen wird nichts.

**Verkettung für AD und Hashes:** `lp(x)` = Länge als u32 big-endian, dann die Bytes. Feste
Ganzzahlen gehen als u64 big-endian ein, Klassen und Arten als ein Byte.

## 2. Typkennungen

| typ | Objekt | | typ | Objekt |
|---|---|---|---|---|
| 1 | Gründung | | 12 | Abfrage (M1b) |
| 2 | Glied der Kontokette | | 13 | Messdefinition |
| 3 | Bund (Gesundheit), Klartext | | 14 | Formular |
| 4 | hochsensibler Bund, Klartext | | 15 | Präparat |
| 5 | Datensatz | | 16 | Einstellungen |
| 6 | Kapsel | | 17 | Entwurf |
| 7 | Kapsel_h | | 19 | Umhüllung |
| 8 | Innenteil der Sicherung | | 20 | Kopf der Sicherungsdatei |
| 9 | Manifest | | 21 | Ersetzungsmarker |
| 10 | Status | | 22 | verschlüsselter Bund (Behälter) |
| 11 | Ereignis | | | |

## 3. Schlüssel und Ableitungen

| Schlüssel | Länge | Herkunft | Zweck |
|---|---|---|---|
| `RK` | 32 | Zufall, als 24 Wörter (BIP39 Englisch, SHA-256 der Liste `2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda`) | nur Wurzel; kein Gerät hält `RK` |
| `RK_sig` | Ed25519 | `crypto_sign_seed_keypair(kdf(RK, "dakrecov", 1))` | Gründung, `geraet_ersetzt`, Ersetzungsmarker |
| `RK_enc` | X25519 | `crypto_box_seed_keypair(kdf(RK, "dakrecov", 2))` | Kapseln |
| `KK` | 32 | Zufall je Generation `k` | Wurzel der Klasse Gesundheit, von der Plattform umhüllt |
| `KK_h` | 32 | Zufall je Generation `k_h` | Wurzel des hochsensiblen Bereichs |
| Bundschlüssel | 32 | `kdf(KK, "dakbund_", 1)` | AEAD des Bundes |
| hochsensibler Bundschlüssel | 32 | `kdf(KK_h, "dakbundh", 1)` | AEAD des hochsensiblen Bundes |
| Sicherungsschlüssel | 32 | `kdf(KK, "daksich_", 1)` | `crypto_secretstream` der Sicherungsdatei |
| Manifestschlüssel | 32 | `kdf(KK, "dakmanif", 1)` | BLAKE2b-256 mit Schlüssel über das Manifest |
| `DS_g,k` | 32 | Zufall, im Bund | umhüllt `dk` der Klasse Gesundheit |
| `WS_g` | X25519 | `crypto_box_seed_keypair(saat)`, Saat im Bund | Schreiben ohne Entsperren (Entwürfe, vorläufige Einträge) |
| `WS_h` | X25519 | wie oben, Saat im hochsensiblen Bund | hochsensibles Schreiben (M1b) |
| `DK_sig`, `DK_enc` | Ed25519, X25519 | Saaten im Bund | Gerät: Glieder signieren; `DK_enc` für Teil B |
| `dk` | 32 | Zufall je Revision | genau ein Chiffrat |

`kdf` ist `crypto_kdf_derive_from_key` mit 8-Byte-Kontext und Teilschlüssel-Nummer. Kein
Schlüssel dient zwei Zwecken; die Wurzeln `RK`, `KK`, `KK_h` verschlüsseln selbst nichts.

## 4. Identität und Kontokette

**Gründung** (`typ` 1), Inhalt: `2 suite`, `3 geraet_id (16)`, `4 DK_sig.pub (32)`,
`5 DK_enc.pub (32)`, `6 RK_sig.pub (32)`, `7 RK_enc.pub (32)`, `8 erstellt_um (ms UTC)`.

**Hülle** um signierte Objekte: `{0: inhalt (Bytes, kodiertes Objekt), 1: sig1 (64), 2: sig2 (64, optional)}`.
Signiert wird `trenner || inhalt` mit `crypto_sign_detached`.

| Objekt | Trenner | sig1 | sig2 |
|---|---|---|---|
| Gründung | `dak-gruendung-v1` | `DK_sig` | `RK_sig` (Pflicht) |
| Glied `rotation`, `recovery_neu` | `dak-kontokette-v1` | `DK_sig` des aktuellen Geräts | keine |
| Glied `geraet_ersetzt` | `dak-kontokette-v1` | `RK_sig` am Kettenkopf | `DK_sig` des neuen Geräts |
| Ersetzungsmarker | `dak-ersetzung-v1` | `RK_sig` am Kettenkopf | keine |

`konto_id = BLAKE2b-256("dak-gruendung-v1" || Hülle der Gründung)`.
Glied-Hash `= BLAKE2b-256("dak-kontokette-v1" || Hülle des Glieds)`; der Kettenkopf ist der Hash
des letzten Glieds, ohne Glieder die `konto_id`.

**Glied** (`typ` 2): `2 suite`, `3 nr` (ab 1, lückenlos), `4 vorgaenger (32)`, `5 art`
(1 `rotation`, 2 `geraet_ersetzt`, 3 `recovery_neu`), `6 geraet_id`, `7 erstellt_um`, dazu je Art:

| Art | Felder |
|---|---|
| `rotation` | `8 generation k`, `9 generation_h k_h`, `10 grund` (1 Wiederherstellung, 2 neue Wörter, 3 auf Wunsch, 4 PIN aktiviert, 5 Suite-Wechsel). Mindestens ein Zähler steigt um genau 1, keiner sinkt. |
| `geraet_ersetzt` | `11 neues geraet_id` (= Feld 6), `12 DK_sig.pub`, `13 DK_enc.pub`, `14 letztes_glied_alt` (≤ nr - 1) |
| `recovery_neu` | `15 RK_sig.pub`, `16 RK_enc.pub` |

Start: `k = k_h = 1`. Nach `geraet_ersetzt` sind Signaturen des alten Geräts ungültig.

**Ersetzungsmarker** (`typ` 21): `2 suite`, `3 konto_id`, `4 ersetztes geraet_id`,
`5 neues geraet_id`, `6 glied_nr`. Dateiname `dak-<konto8>-ersetzt-<geraet8>.dakm`.

## 5. Bund

Klartext Bund (`typ` 3): `2 suite`, `3 konto_id`, `4 version`, `5 generation k`, `6 DS_g,k (32)`,
`7 Saat WS_g`, `8 Saat DK_sig`, `9 Saat DK_enc`, `10 Messdefinitionen`, `11 Formulare`,
`12 Präparate`, `13 Einstellungen`.

Klartext hochsensibler Bund (`typ` 4): `2 suite`, `3 konto_id`, `4 version`, `5 generation k_h`,
`6 Saat WS_h`, `7 Messdefinitionen`, `8 Bezeichnungen [{0 platz, 1 text}]`. In M1a leer.

Behälter (`typ` 22): `2 suite`, `3 klasse` (1 Gesundheit, 2 hochsensibel), `4 version`,
`5 kettenkopf (32)`, `6 nonce (24)`, `7 chiffrat`.
`chiffrat = XChaCha20-Poly1305-IETF(Bundschlüssel, nonce, cbor(Klartext), ad)`,
`ad = lp("dak-bund-v1") || lp(konto_id) || lp(kettenkopf) || u64(version) || klasse`.
Beim Lesen müssen Klasse, Kettenkopf, `konto_id`, Version und Generation passen.

## 6. Datensätze

**Datensatz** (`typ` 5, unverschlüsselter Rahmen): `2 suite`, `3 datensatz_id (16)`,
`4 revision_id (16)`, `5 vorgaenger_revision_id (16, optional)`, `6 art` (1 Inhalt, 2 Korrektur),
`7 klasse` (0 normal, 1 Gesundheit, 2 hochsensibel), `8 Umhüllung (Bytes)`, `9 nonce (24)`,
`10 chiffrat`.

```
klartext = sodium_pad(cbor(Objekt), 256)
ad       = lp("dak-datensatz-v1") || lp(konto_id) || lp(datensatz_id) || lp(revision_id)
           || lp(vorgaenger oder leer) || art || klasse || u64(formatversion)
chiffrat = XChaCha20-Poly1305-IETF(dk, nonce, klartext, ad)
```

**Umhüllung** (`typ` 19): `2 suite`, `3 weg`, `4 generation`, `5 nonce (nur Weg 1)`, `6 chiffrat`.

| Weg | Bedeutung | chiffrat |
|---|---|---|
| 1 | Datenschlüssel `DS_g,k` | `AEAD(DS_g,k, nonce, dk, lp("dak-umhuellung-v1") \|\| lp(konto_id) \|\| lp(datensatz_id) \|\| lp(revision_id) \|\| klasse \|\| u64(generation))` |
| 2 | Schreibschlüssel `WS_g` (vorläufig) | `crypto_box_seal(cbor{0 datensatz_id, 1 revision_id, 2 dk}, WS_g.pub)` |
| 3 | Schreibschlüssel `WS_h` (M1b) | wie Weg 2 an `WS_h.pub` |

IDs sind 128 Bit Zufall. Ein Datensatz enthält nur eine Schutzklasse; Antworten auf
Messdefinitionen einer anderen Klasse werden abgelehnt.

**Löschen:** alle Revisionen werden aus dem Bestand entfernt, die `datensatz_id` kommt in die
Tombstone-Liste (bleibt dauerhaft und wird bei jeder Wiederherstellung angewendet).

## 7. Inhalte

Inhalte tragen `0 typ`, `1 formatversion`, aber keine Suite (die steht am Datensatz).

**Zeitpunkt:** `{0 utc_ms (i64), 1 versatz_min (-1080 bis 1080), 2 zone (IANA, ≤ 64 Byte)}`.
Lebenstag = `floor((utc_ms/60000 + versatz_min - tageswechsel_min) / 1440)`, Tage seit 1970-01-01.

**Status** (`typ` 10): `3 status_id`, `4 zeitpunkt`, `5 erfasst_um`, `6 ausloeser` (0 selbst,
1 Abfrage, 2 nach Ereignis), `7 abfrage_id (optional)`, `8 erfasst_ueber` (0 App, 1 Entwurf),
`9 vorlaeufig`, `10 bezug_von (optional)`, `11 bezug_bis (optional)`, `12 Antworten`,
`13 lebenstag`, `14 formular_id`, `15 art` (0 Status jetzt, 1 Tagesabschluss, 2 Wochenrückblick).
Beliebig viele Status je Lebenstag.

**Ereignis** (`typ` 11): `3 ereignis_id`, `4 art` (Schlüssel des Ereignisformulars, z. B.
`einnahme`), `5 formular_id`, `6 zeitpunkt`, `7 erfasst_um`, `8 lebenstag`, `9 Antworten`,
`10 praeparat_id (optional)`, `11 vorlaeufig`.

**Antwort:** `{0 messdef_id, 1 zustand, 2 wert (nur bei beantwortet), 3 aktiv_gesetzt}`;
Zustände 0 beantwortet, 1 übersprungen, 2 kam nicht vor, 3 nicht gezeigt (Bedingung),
4 ausgeblendet, 5 Block zu. Wert: `{0 zahl (optional), 1 text (optional), 2 [{0 option, 1 stärke 0-10 (optional)}]}`.

**Messdefinition** (`typ` 13): `3 messdef_id`, `4 schluessel`, `5 frage`, `6 feldtyp`
(0 Skala, 1 Stufen, 2 Einfachauswahl, 3 Mehrfachauswahl, 4 Text, 5 Dauer in Minuten, 6 Uhrzeit,
7 Zahl, 8 ja/nein), `7 min`, `8 max`, `9 Anker [{0 wert, 1 text}]`, `10 Optionen [{0 id, 1 text}]`,
`11 Polung` (0 keine, 1 höher besser, 2 höher mehr, 3 bipolar), `12 Bezug` (0 jetzt, 1 seit dem
letzten Status, 2 Tag, 3 Woche, 4 Ereignis), `13 klasse`, `14 kam_nicht_vor`, `15 mit_staerke`,
`16 einheit`, `17 hinweis`. Die IDs der mitgelieferten Vorlage sind
`BLAKE2b-128("dak-messdef-v1" || schluessel)`, Formulare `BLAKE2b-128("dak-formular-v1" || schluessel)`.

**Formular** (`typ` 14): `3 id`, `4 schluessel`, `5 name`, `6 art` (0-2 wie Status, 3 Ereignis),
`7 Gruppen [{0 titel, 1 hinweis, 2 messdef_ids, 3 zugeklappt}]`, `8 eingeschaltet`,
`9 Bedingungen [{0 messdef_id, 1 art}]` (1 Einnahme mit spürbarer Wirkung heute bestätigt,
2 zwei Stunden seit dem letzten Status), `10 reihenfolge`.

**Präparat** (`typ` 15): `3 id`, `4 handelsname`, `5 wirkstoff`, `6 staerke`, `7 form`,
`8 Plan [{0 uhrzeit_min, 1 dosis, 2 wochentage (Bit 0 Montag)}]`, `9 bedarf`,
`10 wirkung_spuerbar`, `11 aktiv`, `12 erinnerung`, `13 status_nach_min`.

**Einstellungen** (`typ` 16): `3 tageswechsel_min` (0-720, Standard 240), `4 sperrzeit_s`,
`5 Erinnerungen [{0 art (1 Status, 2 Tagesabschluss, 3 Wochenrückblick), 1 uhrzeit_min,
2 wochentage, 3 aktiv}]`, `6 max_benachrichtigungen` (Standard 8), `7 Vertrauenspersonen
[{0 name, 1 telefon}]`, `8 Krisenregel {0 tage, 1 unter}`, `9 ausgeblendete messdef_ids`,
`10 Namen der eigenen Marker` (höchstens drei), `11 krisenplan` (Text, höchstens 8000 Byte,
fehlt ohne Plan). Feld 11 ist mit #131 dazugekommen; ein Bund ohne es bleibt gültig. Die
Klartextkopie für die Hilfe ohne Entsperren liegt in den lokalen Metadaten und wird beim
Entsperren aus Feld 11 hergestellt; ein Plan, der nur dort steht (vor #131), wandert dabei einmalig
in den Bund.

**Entwurf** (`typ` 17), an `WS_g.pub` versiegelt nach `sodium_pad(…, 256)`: `3 entwurf_id`,
`4 formular_id`, `5 erstellt (Zeitpunkt)`, `6 block_offen`, `7 Antworten`. Älter als zwölf
Stunden wird er beim Entsperren verworfen.

## 8. Sicherung

```
Datei = "DAKSICH1" || u32(len(kopf)) || kopf || ( u32(len(stück)) || stück )*
kopf  (typ 20) = {2 suite, 3 kapsel, 4 kapsel_h, 5 stromkopf (24)}        len(kopf) ≤ 4096
stücke         = crypto_secretstream_xchacha20poly1305 mit dem Sicherungsschlüssel über
                 cbor(Innenteil), in Stücken zu 64 KiB, letztes mit TAG_FINAL,
                 AD jedes Stücks = lp("dak-sicherung-v1") || u64(formatversion) || lp(konto_id)
Innenteil (typ 8) = {3 Gründung (Hülle), 4 Kette [Hüllen], 5 Bund (Behälter),
                     6 hochsensibler Bund (Behälter), 7 Datensätze [alle Revisionen],
                     8 Tombstones [datensatz_id], 9 Manifest (Bytes), 10 Manifest-MAC (32)}
Manifest (typ 9) = {2 suite, 3 konto_id, 4 kettenkopf, 5 k, 6 k_h, 7 geraet_id, 8 sicherung_nr,
                    9 erstellt_um, 10 [BLAKE2b-256 je Datensatz, sortiert], 11 Tombstones,
                    12 Hash Bund, 13 Hash hochsensibler Bund, 14 Hash kapsel, 15 Hash kapsel_h,
                    16 Hash über lp(Gründung) || lp(Glied)*}
MAC   = BLAKE2b-256(Manifestschlüssel, Manifest)
kapsel   = crypto_box_seal(cbor{0: 6, 1: 1, 2: "dak-kapsel-v1",   3 konto_id, 4 k,   5 KK},   RK_enc.pub)
kapsel_h = crypto_box_seal(cbor{0: 7, 1: 1, 2: "dak-kapsel-h-v1", 3 konto_id, 4 k_h, 5 KK_h}, RK_enc.pub)
```

`kapsel` entsteht je Generation `k`, `kapsel_h` je Generation `k_h`, beide werden unverändert in
jede Sicherung der Generation kopiert.

**Prüfreihenfolge der Wiederherstellung:** Kopf → `kapsel` mit `RK_enc` öffnen (`konto_id`, `k`,
`KK`) → Strom vollständig und in Reihenfolge entschlüsseln (abgeschnitten: Fehler) → MAC → Kette
prüfen → `konto_id` aus der Gründung = `konto_id` der Kapsel → Ankerregel (Schlüssel aus den
Wörtern = Recovery-Schlüssel am Kettenkopf) → `k` der Kapsel = `k` der Kette → alle Hashes im
Manifest → `kapsel_h` öffnen, `k_h` und Hash prüfen → Auswahl nach Kettenverwandtschaft
(Protokoll 10.3) → neues Gerät, `geraet_ersetzt`, Rotation.

**Dateiname:** `dak-<konto8>-<geraet8>-g<k>-<nr, 6 Stellen>.daks` (`konto8`, `geraet8`: erste acht
Hex-Zeichen). **Aufbewahrung:** die letzten zehn Sicherungen und je eine pro Tag der letzten 30 Tage,
nur Dateien mit eigenem Kontokürzel.

## 9. Testvektoren

`core/testvectors/v1/` enthält eine Sicherungsdatei, die zugehörigen Recovery-Wörter und den
erwarteten Inhalt. Der Test `testvektoren` in `core/crates/dak-core/tests/testvektoren.rs` liest
sie mit dem aktuellen Kern und muss genau diesen Inhalt ergeben. Damit ist belegt, dass jede
spätere Version Sicherungen der Formatversion 1 lesen kann (Protokoll 15). Die Bytes der
Vektoren werden nie neu erzeugt; eine neue Formatversion bekommt einen neuen Ordner.

Dazu kommen bytegenaue Vektoren für CBOR (RFC 8949, Anhang A), BIP39 (Trezor-Vektoren),
SHA-256 („abc") und die festen Vorlagen-IDs in den Modultests.

## 10. Größen

| Objekt | Grenze |
|---|---|
| Text einer Antwort | 20 000 Byte, Notizen in anderen Feldtypen 2000 Byte |
| Präparate | 30, je Präparat 12 Planzeiten |
| Entwurf | 200 Antworten |
| Sicherungsdatei | 512 MiB |
| Krisenplan | 8000 Byte |
