# Etebase als Unterbau für „Days & Keys" - Bewertung

Stand der Recherche: 2026-10-06, alle Angaben live gegen GitHub-API, npm, crates.io und die Projektseiten geprüft.
Kennzeichnung: **[B]** = belegt (Quelle genannt), **[S]** = eigene Schlussfolgerung.

## Empfehlung in einem Satz

**Etebase nur teilweise:** als Speicher- und Sync-Schicht für Chiffretext-Blöcke denkbar, als Krypto- und Freigabemodell für Days & Keys **nein**. Das Kernversprechen der App (feldgenaue, befristete, widerrufbare Freigabe mit Neuverschlüsselung) liefert Etebase nicht, und die Bibliotheken sind faktisch im Wartungsmodus. Für ein neues Projekt mit wenige Nutzern ist eine eigene schlanke API mit libsodium (aktuelle Version) der bessere Weg; Etebase lohnt sich höchstens als Vorlage für Auth und Sync-Token-Design.

---

## 1. Wartungszustand

| Repo | Letzter Commit | Letzter Release/Tag | Offene Issues/PRs | Lizenz |
|---|---|---|---|---|
| `etesync/server` | 2024-07-12, `6ac54121` „Bump django version." **[B]** | Tag `v0.14.2` (Commit `386c01d8`, 2024-06-13) **[B]** | 68 offen (Issues+PRs), davon 8 PRs **[B]** | AGPL-3.0 **[B]** |
| `etesync/etebase-js` | 2023-01-06, `24e5dd20` (Dependabot) **[B]**; Repo-Push 2023-03-14 | npm `etebase@0.43.1` vom **2021-08-08** **[B]** | 27 offen, davon 17 PRs (fast alle Dependabot) **[B]** | BSD-3-Clause **[B]** |
| `etesync/etebase-rs` | 2025-05-28, `c0c74b77` „Bump version and update changelog." (OpenSSL-CVE-Fix #47) **[B]** | crates.io `etebase 0.6.1`, 2025-05-28 **[B]** | 11 offen, 1 PR **[B]** | BSD-3-Clause **[B]** |

Weitere Befunde:
- `etebase-js` pinnt `libsodium-wrappers` auf **0.7.6 vom 2019-10-20**; aktuell ist 0.8.4 vom 2026-04-19 **[B]** (npm-Registry). `argon2-webworker` 0.1.1, zuletzt geändert 2022-04-11 **[B]**.
- `etebase-rs` hängt an `sodiumoxide`, das seit 2022 **archiviert** ist **[B]** (https://github.com/sodiumoxide/sodiumoxide). Offenes Issue #49 (2026-07-15): `openssl-sys` 0.9.108 baut nicht mit OpenSSL 4.0 **[B]**.
- Neue Server-Issues werden noch gemeldet (#205 Dependency-Update 2026-07-06, #204 Signup-Lücke 2026-04-19, #201 kein Undelete 2025-11-23) **[B]**. Maintainer `tasn` antwortete zuletzt 2025-10-25 auf #200 **[B]**. Org-Aktivität 2026 nur in `android` (2026-02-27), `etesync-web` (2026-04-15), `etebase-java` (2026-04-24) **[B]**.
- Ein Dritter (SilentSuite, Blogpost 2026-05-19) nennt EteSync „mostly abandoned" und zitiert den Maintainer (Dezember 2023): „I have much less time than before so the development of the apps has slowed down." SilentSuite forkt den Server ab 0.14.2 (AGPL-3.0) **[B]** (https://silentsuite.io/blog/etesync-fork-silentsuite).
- `etebase.com` und `api.etebase.com/docs` antworten mit HTTP 200 **[B]**. Die Protokoll-Doku ist unvollständig: `invitations.md`, `items.md` und der Abschnitt „Collection structure" enthalten nur „TBD" **[B]** (https://github.com/etesync/etebase-docs/tree/master/docs/protocol-specs).

**Einordnung [S]:** nicht aufgegeben, aber Wartungsmodus mit einer Person und Sicherheits-Bumps im Abstand von Monaten bis Jahren. Der JS-Client, also genau die Bibliothek, die Capacitor und Tauri bräuchten, ist seit fünf Jahren ohne Release.

## 2. Krypto-Modell

Belegt aus Protokoll-Doku (`protocol-specs/introduction.md`, `authentication.md`) und Quelltext `etebase-js/src/Crypto.ts`, `EncryptedModels.ts`:

- **Bibliothek:** libsodium für alles **[B]**.
- **KDF:** Argon2id. In `etebase-js` mit `OPSLIMIT_SENSITIVE` (Standard „Hard") und `MEMLIMIT_MODERATE` (256 MiB), primär über `argon2-webworker` (WASM), Fallback `crypto_pwhash` **[B]** (`Crypto.ts`, `deriveKey`).
- **Symmetrisch:** XChaCha20-Poly1305 IETF (AEAD) **[B]**. Unterschlüssel per `crypto_kdf_derive_from_key` mit Kontexten „Main", „Acct", „Col", „ColItem", „Stor" **[B]**.
- **Hash/MAC:** BLAKE2b (`crypto_generichash`) **[B]**.
- **Asymmetrisch:** `crypto_box` (X25519/XSalsa20-Poly1305) für Einladungen, Ed25519 für Signaturen und Login **[B]**.
- **Deterministische Verschlüsselung** der Collection-Typen, damit der Server nach Typ filtern kann, ohne ihn zu kennen **[B]**.
- **Schlüsselhierarchie [B]:** Passwort + Salt → `mainKey` (Argon2id) → verschlüsselt `accountKey` und `identityKeypair`. `authenticationKeypair` deterministisch aus `mainKey`. Collection-Schlüssel zufällig, je Mitglied mit dessen Account-Schlüssel verschlüsselt in `CollectionMember.encryptionKey` abgelegt. Item-Schlüssel: entweder eigener, mit dem Collection-Schlüssel verschlüsselter Schlüssel oder, im Normalfall, **abgeleitet** als `deriveSubkey(item.uid)` aus dem Collection-Schlüssel (`EncryptedModels.ts`, `getCryptoManager`).
- **Authentifizierung:** Challenge-Response, Client signiert `{username, challenge, host, action}` mit dem Ed25519-Login-Schlüssel; kein Passwort geht an den Server **[B]**.
- **Lokale Ablage:** `Account.save()` verschlüsselt den Sitzungszustand mit einem übergebenen Schlüssel, fehlt er, mit **32 Null-Bytes** **[B]** (`Etebase.ts`, Zeile 218). Die App muss also selbst einen Schlüssel aus Keychain/Keystore liefern.
- **Audit:** Kein unabhängiges Audit gefunden. Die Seiten verweisen nur darauf, dass libsodium auditiert sei; die 2020 angekündigte formale Verifikation mit Verifpal und eine formale Spezifikation sind nicht auffindbar veröffentlicht **[B für Ankündigung]** (https://blog.etesync.com/coming-soon-etesync-2-0/), **[S]** für „nicht erfolgt" (keine Fundstelle in Repos, Blog, FAQ, NLnet-Projektseite). Förderung: NLnet NGI0 PET, 2020 **[B]** (https://nlnet.nl/project/EteSyncEnhancements/).

**Einordnung [S]:** solide, konservative Primitive. Kein Forward Secrecy, keine Schlüsselrotation, kein Konzept für zeitliche Schlüssel.

## 3. Teilen, Widerruf, Metadaten

**Teilen [B]** (`guides/collection_sharing.mdx`, `EncryptedModels.ts#createInvitation`): Der Einladende holt den öffentlichen Schlüssel des Empfängers vom Server, soll den Fingerprint out-of-band prüfen und schickt den **Collection-Schlüssel** samt Typ in einer `crypto_box` an ihn. Zugriffsstufen: `Admin`, `ReadWrite`, `ReadOnly`. Nur Admins sehen und ändern die Mitgliederliste. Bekanntes Problem: Issue #70 in etebase-js „Fingerprints are not unique" (2025-02-05, offen) **[B]**.

**Widerruf [B]:** `DELETE /collection/{uid}/member/{username}/` ruft `CollectionMember.revoke()` auf, das nur die Mitgliedszeile löscht und einen `CollectionMemberRemoved`-Eintrag anlegt (`etebase_server/django/models.py`, `fastapi/routers/member.py`). Der Client (`OnlineManagers.ts#remove`) schickt ausschließlich diesen DELETE. **Es gibt keine Schlüsselrotation und keine Neuverschlüsselung.** Die EteSync-FAQ sagt das ausdrücklich: „a malicious user, using specialised software, could potentially keep the encryption key for a journal even after access has been revoked" (https://www.etesync.com/faq/).

**Folge [S]:** Der Widerruf ist reine Server-ACL. Wer den Collection-Schlüssel einmal hatte, kann alle alten **und künftigen** Items dieser Collection entschlüsseln, sobald er an Chiffretext kommt (Server-Kompromittierung, Backup, Cache). Da Item-Schlüssel aus Collection-Schlüssel + Item-UID abgeleitet werden, hilft auch kein Item-weiser Tausch. Rotation ginge nur durch eine neue Collection plus Umkopieren; die Revisionshistorie der alten Collection bleibt dabei liegen, bis sie gelöscht wird.

**Was der Server sieht [B, aus `models.py`]:**
- Klartext: Benutzername, E-Mail, öffentliche Schlüssel, Salt.
- Je Collection: Eigentümer, UID, **Mitgliederliste mit Zugriffsstufen**, entfernte Mitglieder, offene Einladungen (von wem an wen, welche Stufe), Typ-Token (deterministisch, also gleiche Typen erkennbar).
- Je Item: UID, Zugehörigkeit zur Collection, **Anzahl und Reihenfolge der Revisionen**, gelöscht-Flag, Chunk-Dateien mit **Größe** (Inhalt gepaddet, aber Größenklassen bleiben sichtbar).
- Zeitstempel: keine expliziten Zeitspalten in Revision/Stoken, aber die monoton steigende Stoken-ID verrät die Reihenfolge, und der Server sieht natürlich den Zeitpunkt jedes Requests und die Dateizeiten der Chunks **[S]**.
- Namen, Beschreibung, `mtime` und Item-Typ liegen in den verschlüsselten Metadaten **[B]** (`overview.md`).

## 4. Passung „ein Feld = eine Collection" (je Zeitraum)

Rechnung **[S]:** n Personen x ca. 8 Felder x 12 Monate, bei vier Personen rund 380 neue Collections pro Jahr, jede mit einem Collection-Schlüssel je Mitglied und eigener Einladung je Empfänger.

- Technische Grenzen im Server: keine harten Obergrenzen gefunden; Listen sind mit `limit=50` paginiert, `fetch_updates` mit `item_limit = 200` **[B]** (`fastapi/routers/collection.py`). Die Doku empfiehlt dagegen: „Most applications will only use one collection type and only one or a potentially few collections." **[B]** (`overview.md`).
- **Transaktionen gelten nur innerhalb einer Collection** (`/collection/{uid}/item/transaction/`) **[B]** (`fastapi/main.py`, Router-Präfix). Ein Tagebucheintrag mit 8 Feldern würde in 8 getrennten, nicht atomaren Schreibvorgängen landen **[S]**.
- Jeder Monatswechsel bedeutet: neue Collections anlegen, an alle berechtigten Empfänger einladen, die müssen **annehmen** (Einladungen erfordern Interaktion oder Auto-Accept-Logik) **[B für Einladungsfluss, S für Aufwand]**.
- Der Server sieht dann, wer welche Feld-Collection mit wem teilt; die Freigabestruktur (z. B. „B sieht 3 von 8 Feldern") wird zu Klartext-Metadaten **[S]**.
- „Ab Datum X" lässt sich nur grob über Monats-Collections abbilden; eine Freigabe ab dem 17. eines Monats braucht eine weitere Collection oder eigene Krypto **[S]**.

Alternative Abbildung **[S]:** Etebase nur als Blob-Speicher nutzen (eine Collection je Autor, Items = verschlüsselte Log-Einträge), und die Feld-Schlüssel, Vorwärtsableitung und Freigaben komplett in eigener Krypto oberhalb von Etebase führen. Dann nutzt man von Etebase nur Auth, Sync (Stoken) und Speicher, also wenig von dem, was es auszeichnet.

## 5. Client-Tauglichkeit (Capacitor, Tauri)

- `etebase-js` liefert UMD/ESM/CJS-Builds; der UMD-Build ist 773 KB (267 KB gzip) **[B]** (npm-Tarball 0.43.1), dazu libsodium.js 0.7.6 mit 496 KB (183 KB gzip) **[B]**. Issue #6 „Build a smaller libsodium.js dep" seit 2020 offen **[B]**.
- React Native ist nur optional: `react-native-sodium` ist devDependency und wird per `_setRnSodium` eingehängt **[B]** (`package.json`, `Crypto.ts`). In einem WebView läuft also der reine JS/WASM-Pfad **[S]**.
- Laufzeit-Abhängigkeiten `node-fetch` 2, `ws` 7, `isomorphic-ws` müssen im Bundler auf Browser-Varianten gemappt werden **[B für Abhängigkeiten, S für Bundler-Aufwand]**. Issue #44 (Importmaps ohne Bundler) und #15 (Kompilierfehler mit `strict: true`) offen **[B]**.
- Keine Issues zu Capacitor, Tauri, WKWebView oder Safari gefunden **[B]** (GitHub-Suche). Es gibt also auch keinen Nachweis, dass es dort läuft.
- **iOS-Risiko [S]:** Argon2id mit 256 MiB Speicher in einem Web Worker im WKWebView ist auf älteren iPhones knapp; WebContent-Prozesse werden bei Speicherdruck beendet. Muss früh auf echtem Gerät gemessen werden. Die RN-Issues #42 (lange Sodium-Aufrufe auf iOS) und #33 (Hängen bei `sodium.ready`) zeigen, dass Init und Laufzeit auf Mobilgeräten schon früher Probleme machten **[B]**.
- **Tauri [S]:** im WebView wie oben; alternativ `etebase-rs` im Rust-Backend, aber das hängt am archivierten `sodiumoxide` und an OpenSSL (Issue #49), was auf Desktop-Zielen Pflegeaufwand bedeutet.

## 6. Passt das Protokollentwurf-Konzept auf Etebase?

Entwurf: Feld-Ketten mit Vorwärtsableitung, signiertes Append-only-Log je Autor, Neuverschlüsselung beim Widerruf.

| Baustein | Etebase bietet | Passung |
|---|---|---|
| Feld-Ketten mit Vorwärtsableitung (Schlüssel_n+1 = KDF(Schlüssel_n)) | Statische Collection-Schlüssel, Item-Schlüssel aus Collection-Schlüssel abgeleitet, keine Epochen **[B]** | Nein, komplett eigene Krypto **[S]** |
| Befristung „ab Datum X" | Nichts **[B]** | Ergibt sich aus der Kette (Empfänger erhält Schlüssel_X), eigener Code **[S]** |
| Signiertes Append-only-Log je Autor | Items sind signiert/MAC-gesichert, Revisionen werden aufbewahrt, Stoken gibt Reihenfolge **[B]**; aber Items sind veränderbar, es gibt keine Hash-Verkettung über Einträge und keinen Schutz gegen Weglassen durch den Server **[S]** | Teilweise; Verkettung, Signatur je Eintrag und Lücken-Erkennung selbst bauen **[S]** |
| Widerruf mit Neuverschlüsselung | Nur ACL-Löschung, ausdrücklich ohne Schlüsselwechsel **[B]** | Nein, eigener Code; zudem behält Etebase alte Revisionen, die man aktiv löschen muss **[S]** |
| „Server re-verschlüsselt alte Daten" | Server ist bewusst dumm, sieht keine Schlüssel **[B]** | Widerspricht Zero-Knowledge, wenn wörtlich gemeint; realistisch macht das der Autor-Client und lädt neu hoch **[S]** |
| Schlüsselverteilung an Empfänger | Einladungen per `crypto_box` an den Identitätsschlüssel **[B]** | Wiederverwendbar als Transportweg für Feld-Ketten-Schlüssel, aber nur auf Collection-Ebene **[S]** |

**Ergebnis [S]:** Der Protokollentwurf lässt sich nicht auf Etebase-Konzepte abbilden. Etebase taugt nur als Speicher- und Sync-Schicht, auf die eigene Krypto opak aufsetzt. Dann sind die Etebase-Abstraktionen (Collections, Einladungen, Zugriffsstufen) eher im Weg, und die Abhängigkeit von einem kaum gepflegten JS-Client bringt Risiko ohne Gegenwert.

## 7. Alternativen kurz eingeordnet

- **Eigene schlanke API + libsodium [S, Versionen B]:** libsodium.js 0.8.4 (2026-04-19) ist aktuell gepflegt (https://github.com/jedisct1/libsodium.js/releases/tag/0.8.4). Ein Server mit ca. 10 Endpunkten (Auth per Ed25519-Challenge wie bei Etebase, Log-Append, Abruf ab Sequenznummer, Schlüssel-Umschläge je Empfänger) ist für wenige Nutzer überschaubar und bildet den Protokollentwurf direkt ab. Etebase-Design (Auth, Stoken, Padding) kann als Vorlage dienen.
- **OpenMLS [B]:** aktiv, `openmls-v0.9.0` vom 2026-08-25, MIT (https://github.com/openmls/openmls/releases/tag/openmls-v0.9.0). MLS liefert Gruppen-Epochen mit Forward Secrecy und Post-Compromise Security beim Entfernen von Mitgliedern **[S]**, ist aber für Nachrichten gebaut; „eine MLS-Gruppe je Feld und Empfängerkreis" wäre möglich, bringt jedoch Delivery-Service, Rust/WASM-Bindung und viel Komplexität für wenige Personen. Eher Ideengeber für die Rotation als Unterbau.
- **Matrix mit E2EE [B]:** Synapse v1.162.0 (2026-09-29, AGPL-3.0), `matrix-sdk-crypto-wasm` v18.9.0 (2026-09-21). Megolm-Sessions lassen sich rotieren und neuen Mitgliedern ab einem Zeitpunkt geben **[S]**, aber Feld-Granularität hieße ein Raum je Feld, und der Homeserver sieht Raumstruktur und Mitglieder. Für ein Tagebuch überdimensioniert.
- **Standard Notes Server [B]:** aktiv (Commits 2026-10-05, GPL-3.0). Ein-Nutzer-Notizmodell, Teilen ist kein Kernkonzept **[S]**; passt nicht zu feldgenauen Freigaben.
- **CryptPad [B]:** aktiv, Release 2026.5.1 (2026-05-26, AGPL-3.0). Dokumentzentriert, Zugriff über Schlüssel im Link, Widerruf ebenfalls nur durch Neuanlegen **[S]**; kein Baukasten für eigene Apps.
- **Proton-artige Ansätze [S]:** Proton-Server sind nicht offen; als Muster (Schlüssel je Objekt, mit Empfänger-Schlüssel umhüllt, signiert) entspricht das ohnehin dem Weg „eigene API + libsodium".

## 8. Lücken, die bei Etebase durch eigenen Code zu schließen wären

1. Feld-Schlüsselketten mit Vorwärtsableitung und Epochen (eigene Krypto über Etebase-Items).
2. Befristete Freigabe „ab Datum X" (Herausgabe eines Kettenglieds statt Collection-Schlüssel).
3. Echter Widerruf: neuer Kettenzweig, Neuverschlüsselung der betroffenen Altdaten durch den Autor-Client, Löschen alter Revisionen (Etebase bewahrt sie auf).
4. Signiertes, hash-verkettetes Append-only-Log je Autor mit Erkennung weggelassener oder umsortierter Einträge.
5. Atomare Einträge über mehrere Felder (Etebase-Transaktionen nur je Collection).
6. Metadaten-Minimierung: Freigabestruktur nicht als Collection-Mitgliedschaften sichtbar machen.
7. Lokale Schlüsselablage in Keychain/Keystore (Default von `Account.save()` ist ein Null-Schlüssel).
8. Fingerprint-Verifikation mit eindeutigen Fingerprints (Issue #70).
9. Pflege des JS-Clients: libsodium 0.7.6 → 0.8.x, Browser-Mapping für `node-fetch`/`ws`, Bundle-Verkleinerung, Test auf WKWebView; faktisch ein Fork.
10. Für Tauri/Rust: Ersatz für archiviertes `sodiumoxide`, OpenSSL-4-Kompatibilität.
11. Server-Betrieb: Django/FastAPI-Abhängigkeiten selbst aktuell halten (offene PR #205), Signup-Härtung (#204).

## Quellen

- https://github.com/etesync/server (Commit `6ac54121`, Tag `v0.14.2`, `etebase_server/django/models.py`, `etebase_server/fastapi/routers/member.py`, `.../collection.py`, `.../main.py`)
- https://github.com/etesync/etebase-js (`src/Crypto.ts`, `src/EncryptedModels.ts`, `src/OnlineManagers.ts`, `src/Etebase.ts`, Issues #6, #15, #33, #42, #44, #70)
- https://www.npmjs.com/package/etebase (0.43.1, 2021-08-08)
- https://github.com/etesync/etebase-rs (Commit `c0c74b77`, Issue #49), https://crates.io/crates/etebase
- https://github.com/etesync/etebase-docs/tree/master/docs/protocol-specs
- https://www.etesync.com/faq/ (Schlüssel nach Widerruf)
- https://blog.etesync.com/coming-soon-etesync-2-0/ (angekündigte Verifpal-Verifikation)
- https://nlnet.nl/project/EteSyncEnhancements/
- https://silentsuite.io/blog/etesync-fork-silentsuite (2026-05-19)
- https://github.com/sodiumoxide/sodiumoxide (archiviert)
- https://github.com/jedisct1/libsodium.js/releases/tag/0.8.4
- https://github.com/openmls/openmls/releases/tag/openmls-v0.9.0
- https://github.com/element-hq/synapse/releases/tag/v1.162.0, https://github.com/matrix-org/matrix-sdk-crypto-wasm/releases/tag/v18.9.0
- https://github.com/standardnotes/server, https://github.com/cryptpad/cryptpad/releases/tag/2026.5.1
