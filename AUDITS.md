# Sicherheitsrelevante Abhängigkeiten und ihre Audits

„Auditiert" gilt nur für die geprüfte Version und den geprüften Umfang. Ein Update folgt
`docs/protokoll.md` Abschnitt 14: Release- und Security-Hinweise lesen, Testvektoren,
Wycheproof-Fälle und Protokoll-Negativtests grün, diese Datei nachziehen.

Stand 2026-10-14. Gepinnt über `core/Cargo.lock` und exakte Versionen in `core/Cargo.toml` (#100).

| Bibliothek | Rolle | gepinnt (geplant) | Audits | Abstand / Hinweise |
|---|---|---|---|---|
| libsodium | alle Kryptoprimitive in `dak-core` | 1.0.22 stable, gebündelt in libsodium-sys-stable 1.24.0, Signatur des Quellarchivs beim Bau geprüft (minisign) | Private Internet Access / Matthew Green, 2017, Versionen 1.0.12 und 1.0.13, keine kritischen Funde | Audit liegt neun Jahre und zehn Minor-Versionen zurück. Erste veröffentlichte Schwachstelle Ende 2025 (https://00f.net/2025/12/30/libsodium-vulnerability), Korrektur in 1.0.22 enthalten |
| libsodium-sys-stable | Rust-Anbindung an libsodium | 1.24.0 (2026-04-09) | keine eigenen; reine FFI-Anbindung | Low-Level-Schnittstelle: Speicher, Längen und Rückgabewerte sichert die Hülle in `dak-core` |
| uniffi | Bindings Swift/Kotlin | 0.32.2 (2026-09-23) | keine bekannt | kein Krypto, aber Teil der Sicherheitsgrenze (Bridge) |
| zeroize | Löschen von Geheimnissen im Speicher | 1.9.0 (2026-06-12) | keine eigenen bekannt | Teil von RustCrypto |
| swift-sodium | nur falls ein Swift-Weg ohne UniFFI nötig wird | 0.11.0 | keine eigenen bekannt | derzeit nicht vorgesehen |

**Nicht mehr vorgesehen** (Entscheidung 2026-10-06, `docs/krypto-bibliotheken.md`): noble und
libsodium.js in der WebView, Privys `shamir-secret-sharing` in der WebView (Shamir kommt nativ, #34).

## Weitere Abhängigkeiten aus dem Architekturkonzept

Im Kern eingesetzt (#100): rusqlite 0.40.2, thiserror 2.0.21, serde 1.0.229 und serde_json 1.0.151
(nur zum Lesen der Fragenvorlage), uniffi 0.32.2 (nur mit Feature `ffi`), in Tests ciborium 0.2.2 als
unabhängige CBOR-Gegenprobe. `jiff`, `krilla`, `bip39`, `dcbor` sind noch nicht eingebunden.

### Vorgeschlagen

Stand 2026-10-08, aus `docs/architektur/bibliotheken.md`; gilt erst mit den Entscheidungen in
`docs/architektur.md` (AE-14, AE-15, AE-28, AE-35) und wird mit #32 bestätigt.

| Bibliothek | Rolle | Version zum Pinnen | Lizenz | Audit-Stand |
|---|---|---|---|---|
| rusqlite | Speicher | 0.40.2 (2026-08-08) | MIT | kein externes Audit gefunden; Prüfung des `unsafe`-Codes durch die Maintainer führte 2020 zu 0.23.0 mit mehreren Speichersicherheits-Korrekturen [B: Release-Notes 0.23.0]; RUSTSEC-2020-0014, RUSTSEC-2021-0128 in alten Versionen |
| libsqlite3-sys (`bundled`) | SQLite 3.53.2 | 0.38.2 (2026-08-08), gepinnt; Ziel SQLite ≥ 3.53.4 offen | MIT, SQLite gemeinfrei | kein externes Audit; 100 % Branch- und MC/DC-Abdeckung, OSS-Fuzz, dbsqlfuzz, Absturztests [B: sqlite.org/testing.html]; RUSTSEC-2022-0090 (CVE-2022-35737) historisch |
| minicbor | nicht verwendet: eigener Codec (AE-14, #100) | - | - | - |
| dcbor | nur Tests: Differenzial-Fuzzing | 0.25.2 (2026-03-16) | BSD-2-Clause-Patent | kein Audit gefunden; OSV leer |
| bip39 | nur Tests: Orakel für Wortkodierung | 3.0.0 (2026-09-17) | CC0-1.0 | kein Audit gefunden; OSV leer |
| Wortlisten | BIP39 Englisch, dys2p `de-2048` | Dateien mit SHA-256 in `docs/format.md` | BIP39: Teil des BIP-Repos; dys2p: Unlicense OR CC0 OR BSD-3 | keine Audits (Daten) |
| krilla | Arzt-PDF | 0.8.2 (2026-06-04) | MIT OR Apache-2.0 | kein Audit gefunden; OSV leer |
| Inter | Schrift im PDF | 4.2 | OFL-1.1 | entfällt |
| hayro | nur Tests: PDF rendern | 0.8.0 (2026-10-04) | Apache-2.0 OR MIT | kein Audit gefunden |
| jiff | Zeit, Zeitzonen, Lebenstag | 0.2.38 (2026-10-06), IANA 2026e | Unlicense OR MIT | kein Audit gefunden; OSV leer |
| thiserror | Fehlertypen | 2.0.21 (2026-09-23) | MIT OR Apache-2.0 | kein Audit gefunden; OSV leer |
| proptest | nur Tests | 1.11.0 (2026-03-24) | MIT OR Apache-2.0 | entfällt (Testwerkzeug) |
| cargo-fuzz, libfuzzer-sys, arbitrary | nur Fuzzing | 0.13.2, 0.4.13, 1.4.2 | MIT OR Apache-2.0; libfuzzer-sys zusätzlich NCSA | entfällt (Testwerkzeug) |
| Wycheproof | Testvektoren | `C2SP/wycheproof` `testvectors_v1`, Commit pinnen | Apache-2.0 | entfällt (Daten) |
| ed25519-speccheck | Randfälle Ed25519 | `cases.json`, Commit pinnen | Apache-2.0 [S, beim Kopieren prüfen] | entfällt (Daten) |
| Swift Charts / Vico | Diagramme nativ | iOS-SDK / 3.3.1 | Apple SDK / Apache-2.0 | keine bekannt |

**Nicht vorgesehen:** `serde_cbor` (RUSTSEC-2021-0127, nicht mehr gepflegt), `ciborium` (keine
Kanonik nach RFC 8949 4.2.1), `sled` (Beta), `fjall` (löscht erst bei Kompaktierung), `genpdf`
(seit 2021 still), `secrecy` (durch eigene Hülle ersetzt), `chrono-tz` (ältere Zeitzonendaten),
SQLCipher (fremde Kryptografie neben libsodium).
