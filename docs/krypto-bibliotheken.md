# Kryptobibliotheken für Days & Keys

> **Überholt in der Empfehlung (2026-10-06):** Die Kryptografie liegt nicht in der WebView,
> sondern ausschließlich im Rust-Kern `dak-core` mit libsodium (`docs/protokoll.md` Abschnitt 11,
> `AUDITS.md`). noble und libsodium.js sind nicht vorgesehen. Die Bewertungen der Bibliotheken
> unten gelten weiter.

Stand: 2026-10-06, alle Zahlen live erhoben (GitHub-API, npm-Registry/-Downloads, crates.io-API, OSV.dev, MDN browser-compat-data 8.1.4 vom 2026-10-01, Herstellerseiten)..

Kennzeichnung: **[B]** = belegt (Quelle verlinkt oder in den Rohdaten unten), **[S]** = Schlussfolgerung von mir.

## 0. Kurzantwort

- **Am anerkanntesten** für die hier gebrauchten Primitive (25519-Familie, XChaCha20, BLAKE2b) ist **libsodium** [S]. Für reines JavaScript ist **noble** (paulmillr) heute der Standard mit den meisten Audits und der größten Verbreitung [B].
- **Empfehlung Client (Capacitor + Tauri-WebView):** noble (`@noble/curves`, `@noble/hashes`, `@noble/ciphers`) als Basis, die libsodium-Konstruktionen `crypto_kdf`, `crypto_box_seal` und `sodium_pad` als kleine, eigene, gegen libsodium-Testvektoren geprüfte Module. libsodium.js nur, wenn man Argon2id im Client braucht oder bewusst eine einzige Implementierung auf allen Seiten will [S].
- **Tauri/Rust-Seite und Server:** native **libsodium** (Rust: `libsodium-sys-stable`; Go/andere: jeweilige libsodium-Bindung) als Referenzimplementierung. Damit ist der Server die „Wahrheit", gegen die der noble-Client in CI geprüft wird [S].
- **Hardware-Schlüssel (P-256):** nur als Wrapping-Schlüssel für die 25519-Geheimnisse, nicht als Protokollschlüssel [S].

## 1. Rangfolge „am anerkanntesten"

Kriterien: Alter und Verbreitung, Zahl und Qualität unabhängiger Audits, Träger, CVE-Historie, Stellung in der Fachwelt. Nicht: Eignung für Days & Keys (die steht in Abschnitt 3).

| Rang | Bibliothek | Kurzbegründung |
|---|---|---|
| 1 | **OpenSSL / BoringSSL** | Basis von TLS im halben Internet, FIPS-Historie, riesige Prüfdichte; aber große Angriffsfläche, viele CVEs, kein XChaCha20/BLAKE2b-Fokus. Für D&K nur Randthema [S] |
| 2 | **libsodium** | De-facto-Standard für „einfach richtig nutzbare" 25519-Kryptografie seit 2013, Audit 2017 ohne kritische Funde, erste CVE erst Ende 2025 [B] |
| 3 | **Go-Standardbibliothek + x/crypto** | Trail-of-Bits-Audit 2025 mit einem einzigen Low-Fund, FIPS-140-3-Modul [B]. Für den Server relevant |
| 4 | **AWS-LC / aws-lc-rs** | AWS-Träger, FIPS 140-3, teils formal verifiziert; 2026 mehrere CVEs, aber in X.509/PKCS7/AES-CCM, nicht in 25519 [B] |
| 5 | **noble** (curves/hashes/ciphers) | Sieben externe Audits 2021-2026 (Cure53, Trail of Bits, Kudelski), >100 Mio. Downloads/Woche (hashes), genutzt von MetaMask, ethers.js, viem, Proton Mail, Tutanota [B]; Ein-Personen-Projekt [S] |
| 6 | **Google Tink** | Starke Misuse-Resistenz, Google-Träger, aktiv in Java/C++/Go/Python/ObjC [B]; **kein JS/TS mehr** (2023 entfernt) [B] |
| 7 | **RustCrypto + dalek** | Riesige Verbreitung im Rust-Ökosystem (ed25519-dalek 229 Mio. Downloads) [B]; Audits älter (2019/2020), 2026 große Versionssprünge [B] |
| 8 | **ring** | Sehr verbreitet (782 Mio. Downloads), 2025 kurz als „unmaintained" markiert, jetzt vom rustls-Team gepflegt [B] |
| 9 | **libsignal** | Kryptografisch hochrangig (formale Analysen von PQXDH), aber AGPL und ausdrücklich nur für Signal gedacht [B] |
| 10 | **Monocypher** | Klein, sauber, Cure53-Audit 2020 ohne High-Funde [B]; geringe Verbreitung [S] |
| 11 | **age / rage** | Anerkanntes Dateiverschlüsselungsformat (FiloSottile), aber ein Format, keine Primitive-Bibliothek [S] |
| 12 | **TweetNaCl-js** | Cure53-Audit 2017 ohne Funde [B], aber seit 2020 kein Release [B] |
| 13 | **OpenMLS** | Erst 2025/26 erstmals umfassend auditiert (ein High-Fund) [B]; Gruppenprotokoll, für D&K nicht nötig [S] |

## 2. Steckbriefe

Rohdaten (Stand 2026-10-06): GitHub `pushed_at`, letztes Release, npm-Downloads der KW 2026-09-28 bis 2026-10-04, crates.io Gesamt-/90-Tage-Downloads.

### 2.1 libsodium (C)

- Lizenz: ISC [B] https://github.com/jedisct1/libsodium/blob/master/LICENSE
- Träger: Frank Denis (jedisct1), Einzelmaintainer mit kommerziellem Support-Angebot [B] https://libsodium.gitbook.io/doc/commercial_support
- Letztes Release: **1.0.22 am 2026-04-09** (ML-KEM768, X-Wing, SHA-3, NEON-Argon2) [B] https://github.com/jedisct1/libsodium/releases/tag/1.0.22-RELEASE ; letzter Push 2026-09-28 [B]
- Audit: Matthew Green (Cryptography Engineering), v1.0.12/1.0.13, 2017, finanziert von Private Internet Access; keine kritischen Schwachstellen, einige Low-Funde zur API-Nutzbarkeit und zum anpassbaren RNG [B] https://www.privateinternetaccess.com/blog/libsodium-v1-0-12-and-v1-0-13-security-assessment/ . Ein neueres Voll-Audit habe ich nicht gefunden [B: Suche ohne Treffer].
- CVE: **CVE-2025-69277** (Dezember 2025), `crypto_core_ed25519_is_valid_point` prüfte nur X statt auch Y-Z nach Cofaktor-Multiplikation; betrifft nur untypische Low-Level-Nutzung, Fix ad3004e [B] https://00f.net/2025/12/30/libsodium-vulnerability , https://nvd.nist.gov/vuln/detail/CVE-2025-69277 . Laut Berichterstattung die erste CVE nach 13 Jahren [B] https://hackingpassion.com/libsodium-first-cve-13-years-ed25519/ . D&K nutzt diese Funktion nicht [S].
- Verbreitung: 13 976 GitHub-Sterne; Nutzer u. a. Discord, Wire, Zcash, Stellar, WordPress [B] https://doc.libsodium.org/libsodium_users
- Primitive: **alle** gebrauchten: Ed25519 (`crypto_sign`), X25519 (`crypto_scalarmult`, `crypto_kx`), XChaCha20-Poly1305 (`crypto_aead_xchacha20poly1305_ietf`, `crypto_secretstream`), BLAKE2b (`crypto_generichash`), `crypto_kdf`, `crypto_box_seal`, `sodium_pad`, HMAC-SHA-256/512/512-256, Argon2id (`crypto_pwhash`). Kein Shamir [B].
- Tauri: über Rust-Bindung `libsodium-sys-stable` 1.24.0 (2026-04-09, vom libsodium-Autor) [B] https://crates.io/crates/libsodium-sys-stable . `sodiumoxide` ist seit 2021 ohne Release und nicht mehr zu nehmen [B] https://crates.io/crates/sodiumoxide

### 2.2 libsodium.js / libsodium-wrappers

- Lizenz: ISC [B] https://www.npmjs.com/package/libsodium-wrappers ; Träger: derselbe Maintainer wie libsodium [B]
- Letztes Release **0.8.4 am 2026-04-19**, letzter Push 2026-09-25 [B] https://github.com/jedisct1/libsodium.js
- Downloads: `libsodium-wrappers` 5,9 Mio./Woche, `-sumo` 0,49 Mio./Woche [B]
- Audit: **kein eigenes Audit** der Emscripten-Übersetzung gefunden [B: Suche]. Es erbt den C-Code [S], aber Build-Kette und Wrapper sind ungeprüft [S].
- CVE: keine im OSV für `libsodium`/`libsodium-wrappers` [B] (OSV-Abfrage 2026-10-06)
- Primitive (selbst im Paket nachgesehen, 0.8.4): Die **Standard-Variante** enthält `crypto_sign`, `crypto_box_seal`, `crypto_kdf_derive_from_key`, `crypto_aead_xchacha20poly1305_ietf_*`, `crypto_generichash`, `sodium_pad`, `crypto_auth` (HMAC-SHA-512-256). **Nur in sumo:** `crypto_pwhash` (Argon2id) und `crypto_auth_hmacsha256/512` [B]. Falle: die `.d.ts` der Standard-Variante deklariert `crypto_pwhash` trotzdem, zur Laufzeit fehlt es [B].
- Größe: ~290 KB (Standard) bzw. ~375 KB (sumo), minifiziert+gzip, WASM + JS-Fallback [B] (README)
- WebView: läuft in jedem Browser mit WASM; asynchrone Initialisierung (`await sodium.ready`), kein Code-Splitting [B] (serenity-kit-README) [S]. Kein Bedarf an Secure Context, also auch unter `capacitor://` unkritisch [S].
- Bemerkenswert: Frank Denis selbst schrieb 2023-11-04: „for crypto in JavaScript today, I'd rather use WebCrypto when possible, and Noble cryptography for everything else." [B] https://github.com/jedisct1/libsodium.js/issues/327#issuecomment-1793419292

### 2.3 noble (@noble/curves, @noble/hashes, @noble/ciphers)

- Lizenz: MIT [B]; Träger: Paul Miller (paulmillr), Einzelmaintainer, Sponsoren u. a. Ethereum Foundation, OpenSats, wevm, vault12 [B] https://paulmillr.com/noble/
- Letztes Release: alle drei **2.4.0 am 2026-08-27**, letzte Pushes 2026-09-08 bis 2026-09-17 [B]
- Downloads/Woche: hashes 112 Mio., ciphers 42,8 Mio., curves 39,6 Mio. [B]; Nutzer laut Projekt: MetaMask, ethers.js, viem, Proton Mail, Tutanota, polkadot.js, solana-web3.js [B] https://paulmillr.com/noble/
- Audits [B] (README „Security"-Abschnitte):
  - noble-curves: Trail of Bits Feb 2023 (v0.7.3, abstract + secp256k1), Kudelski Sep 2023 (v1.2.0, Weierstrass/Starknet), **Cure53 Sep 2024 (v1.6.0, inkl. ed25519)**, laut README **Trail of Bits Aug 2026 (v2.3.0, „everything", Initiative „Patch the Planet" mit OpenAI)** - für Letzteres liegt noch kein PDF im Repo [B: `audit/`-Ordner enthält nur 2023/2023/2024] https://github.com/paulmillr/noble-curves#security
  - noble-ciphers: **Cure53 Sep 2024 (v1.0.0, vollständig)** [B] https://cure53.de/audit-report_noble-crypto-libs.pdf
  - noble-hashes: **Cure53 Jan 2022 (v1.0.0)**, ausdrücklich **ohne argon2, blake3, sha1, sha3-addons** [B] https://github.com/paulmillr/noble-hashes#security
- CVE: keine im OSV für die vier noble-Pakete [B]
- Primitive: Ed25519 (RFC 8032, wahlweise ZIP215), X25519, P-256 [B]; XChaCha20-Poly1305, XSalsa20-Poly1305, `secretbox` [B]; BLAKE2b mit Key/Salt/Personalisierung (je bis 16 Byte), HMAC, HKDF, Argon2id, scrypt [B]. **Nicht enthalten:** `crypto_box`, `crypto_box_seal`, `crypto_kdf`, `sodium_pad`, `secretstream` [B] (README verweist für box/sealedbox auf `@serenity-kit/noble-sodium`).
- Einschränkungen: Konstante Laufzeit ist in JS nicht garantierbar, sagen die Autoren selbst [B]. Argon2 in JS etwa 5x langsamer als nativ, das Projekt rät zu scrypt [B].
- WebView: reines TS/JS, kein WASM, keine Secure-Context-Abhängigkeit, tree-shakeable [S].

### 2.4 TweetNaCl-js

- Lizenz: Unlicense [B]; Träger: Dmitry Chestnykh (dchest) [B]
- Letztes Release **1.0.3 vom 2020-02-10**, letzter Push 2025-08-15 [B] https://www.npmjs.com/package/tweetnacl
- Downloads 44,8 Mio./Woche, fast nur als transitive Altlast [S]
- Audit: Cure53, Jan/Feb 2017, finanziert von Deletype, **keine Funde** [B] https://tweetnacl.js.org/audits/cure53.pdf
- Primitive: Ed25519, X25519, XSalsa20-Poly1305 (box/secretbox), SHA-512. **Kein** XChaCha20-Poly1305, kein BLAKE2b, kein sealed box, kein Argon2 [B/S]
- Bewertung: für D&K ungeeignet, weil zentrale Primitive fehlen und die Entwicklung steht [S].

### 2.5 Google Tink

- Lizenz: Apache-2.0; Träger: Google [B]
- Status: Monorepo `tink-crypto/tink` **archiviert** (letzter Push 2024-04-17); aktiv sind die Einzel-Repos: tink-java v1.23.0 (2026-07-09), tink-cc v2.9.2 (2026-10-01), tink-go v2.8.0 (2026-08-11), tink-py v1.16.1 (2026-08-13), tink-objc (Push 2026-10-05) [B]
- **JavaScript/TypeScript wurde 2023 entfernt**, npm-Paket `tink-crypto` 0.1.1 von 2023, 247 Downloads/Woche [B] https://github.com/tink-crypto/tink/issues/689
- Audit: kein öffentliches Gesamt-Audit gefunden; CVE-2020-8929 (Ciphertext-Malleability in Tink Java) [B] https://osv.dev/vulnerability/GHSA-g5vf-v6wf-7w2r
- Primitive: Ed25519, X25519 (über HPKE), XChaCha20-Poly1305, HMAC; **kein BLAKE2b, kein libsodium-Format (kdf/sealed box/pad)**; eigenes Keyset-Format [S]
- Bewertung: für den Client mangels JS raus. Auf dem Server nur, wenn man das Protokoll auf Tink-Formate umbaut [S].

### 2.6 Web Crypto API (Stand 2026)

- Ed25519 und X25519 in Web Crypto laut MDN browser-compat-data 8.1.4 (2026-10-01) [B] https://github.com/mdn/browser-compat-data :
  - Ed25519: Chrome/Chrome Android/**Android WebView ab 137**, Safari/**iOS Safari ab 17**, Firefox ab 129
  - X25519: Chrome/Android WebView ab 133, Safari/iOS ab 17, Firefox ab 130
  - Hintergrund Chrome 137 (Mai 2025): https://blogs.igalia.com/jfernandez/2025/08/25/ed25519-support-lands-in-chrome-what-it-means-for-developers-and-the-web/
- Interop-Lücken [B] https://blogs.igalia.com/jfernandez/2025/02/28/can-i-use-secure-curves-in-the-web-platform/ : WebKit signierte Ed25519 randomisiert statt deterministisch nach RFC 8032; unterschiedliche Small-Order-Prüfungen je Engine; JWK-Streit um `alg`.
- **Fehlt in Web Crypto komplett:** XChaCha20-Poly1305, BLAKE2b, Argon2 [B: nicht im Standard]. Damit kann Web Crypto in D&K höchstens Ed25519/X25519 übernehmen [S].
- **Secure-Context-Problem:** `crypto.subtle` gibt es nur im Secure Context. Für Capacitor iOS (`capacitor://localhost`) wird berichtet, dass die Custom-Scheme-Herkunft kein Secure Context sei (aktuell z. B. ein Issue vom 2026-09-12) [B] https://github.com/water-rs/waterui/issues/592 , https://forum.ionicframework.com/t/crypto-subtle/206836 ; ein WebKit-Kommentar von 2021 sagt dagegen, Custom-Scheme-Seiten würden als Secure Context behandelt [B] https://bugs.webkit.org/show_bug.cgi?id=220184 . Widersprüchlich, also **auf dem Gerät prüfen** [S].
- Weitere Abhängigkeiten [S]: Android WebView folgt dem Play-Store-Update; Geräte ohne aktuelles WebView (ohne Google-Dienste, alte Firmware) haben kein Ed25519. Tauri unter Linux nutzt WebKitGTK in der Version der Distribution.
- Vorteil: `extractable: false` hält Schlüssel aus dem JS-Heap fern [S]. Aber ohne Hardware-Bindung bringt das gegen ein kompromittiertes WebView wenig [S].

### 2.7 RustCrypto und dalek

- Lizenz: MIT/Apache-2.0 (RustCrypto), BSD-3-Clause (dalek) [B]
- Träger: RustCrypto-Organisation (Community, u. a. Tony Arcieri), dalek-cryptography (Community) [B]
- Aktuelle Versionen [B] (crates.io): `ed25519-dalek` **3.0.0 (2026-07-06)**, `x25519-dalek` 3.0.0 (2026-07-06), `curve25519-dalek` 5.0.0 (2026-07-06), `chacha20poly1305` 0.11.0 (2026-06-28), `blake2` 0.11.0 (2026-08-26), `hmac` 0.13.0, `argon2` 0.6.0 (2026-08-27), `crypto_box` 0.9.1 (2025-10-05)
- Downloads gesamt: curve25519-dalek 276 Mio., ed25519-dalek 229 Mio., blake2 176 Mio., chacha20poly1305 91 Mio. [B]
- Audits:
  - Quarkslab 2019 (curve25519-dalek, subtle, bulletproofs voll; x25519-/ed25519-dalek nur am Rand), 30 Personentage [B] https://blog.quarkslab.com/resources/2019-08-26-audit-dalek-libraries/19-06-594-REP.pdf
  - NCC Group 2020 für MobileCoin: `aes-gcm` und `chacha20poly1305`, keine Schwachstellen, 5 Personentage [B] https://www.nccgroup.com/research/public-report-rustcrypto-aesgcm-and-chacha20pluspoly1305-implementation-review/
  - `crypto_box`: Cure53 (v0.7.1), keine wesentlichen Funde [B] https://docs.rs/crypto_box/latest/crypto_box/
  - Alle Audits betreffen deutlich ältere Versionen als die 2026er Major-Releases [S].
- CVEs [B]: CVE-2022-50237 / RUSTSEC-2022-0093 (ed25519-dalek, Double-Public-Key-Signing-Oracle), CVE-2024-58262 / RUSTSEC-2024-0344 (curve25519-dalek, Timing in `Scalar::sub`, Fix 4.1.3), CVE-2019-16143 (blake2, falsches HMAC-BLAKE2) https://rustsec.org/packages/curve25519-dalek.html
- Primitive: alle außer `sodium_pad` und `crypto_kdf` als fertige Funktion; `crypto_box` hat mit Feature `seal` libsodium-kompatible sealed boxes [B: Cargo-Features `seal = [blake2, alloc]`]. `crypto_kdf` ist mit `blake2` (Salt/Personal) nachzubauen [S].
- Pure-Rust-libsodium-Klon `dryoc` 2.0.0 (2026-09-28): libsodium-kompatibel, **kein Drittaudit** [B] https://github.com/brndnmtthws/dryoc

### 2.8 ring

- Lizenz: Apache-2.0 AND ISC (OpenSSL-Erbe) [S: GitHub meldet NOASSERTION]; Träger: Brian Smith, seit 2025 Sicherheitspflege durch das rustls-Team [B] https://rustsec.org/advisories/RUSTSEC-2025-0007.html
- Letzte Version 0.17.14 (2025-03-11), letzter Push 2026-07-23 [B]
- CVE-2025-4432 / RUSTSEC-2025-0009 (AES-Panic bei Overflow-Checks), RUSTSEC-2025-0010 (<0.17 unmaintained) [B]
- Primitive: Ed25519, X25519 (nur ephemer), ChaCha20-Poly1305 (12-Byte-Nonce), HMAC; **kein XChaCha20, kein BLAKE2b, kein Argon2** [S]. Für D&K ungeeignet [S].

### 2.9 aws-lc-rs / AWS-LC

- Lizenz: ISC AND (Apache-2.0 OR ISC) [S]; Träger: AWS [B]
- aws-lc-rs 1.18.1 (2026-09-01), AWS-LC v5.11.0 (2026-10-01) [B]
- FIPS 140-3 (AWS-LC FIPS 3.0, inkl. Ed25519 und ML-KEM), formal verifizierte Teile [B] https://aws.amazon.com/blogs/security/aws-lc-fips-3-0-first-cryptographic-library-to-include-ml-kem-in-fips-140-3-validation/
- CVEs 2026: CVE-2026-3336/-3338 (PKCS7_verify), CVE-2026-3337 (Timing AES-CCM), CVE-2026-4428 (CRL), X.509-Name-Constraints [B] (OSV `aws-lc-sys`)
- Primitive: wie ring (ring-kompatible API); **kein XChaCha20, kein BLAKE2b** [S]. Interessant nur, falls der Server FIPS braucht [S].

### 2.10 Monocypher

- Lizenz: BSD-2-Clause oder CC0 [S: GitHub NOASSERTION, Repo-Datei LICENCE.md]; Träger: Loup Vaillant [B]
- 4.0.3 (2026-06-15), Push 2026-09-30 [B]
- Audit: Cure53 Juni 2020, finanziert vom OTF, keine High/Critical-Funde [B] https://monocypher.org/quality-assurance/audit
- Primitive: X25519, EdDSA (Standard-Hash BLAKE2b, Ed25519 per Zusatzmodul), XChaCha20-Poly1305, BLAKE2b, Argon2 [B/S]. Kein sealed box, kein `crypto_kdf`-Format [S]. AEAD folgt RFC 8439 mit XChaCha20; volle Bitkompatibilität zu libsodium ist in der Doku nicht zugesagt [B] https://monocypher.org/manual/aead
- Bewertung: gute Alternative für eingebettete C-Ziele, für D&K ohne Vorteil gegenüber libsodium [S].

### 2.11 age / rage

- age v1.3.2 (2026-08-29, BSD-3-Clause, FiloSottile), rage v0.12.1 (2026-07-14, Apache/MIT, str4d) [B]
- CVE/GHSA Dez. 2024: Plugin-Name mit Pfadtrenner führt beliebiges Binary aus (GHSA-32gq-x56h-299c, GHSA-4fg7-vxc8-qx5w) [B]
- Kein öffentliches Voll-Audit gefunden [B: Suche ohne Treffer]
- Einordnung: Dateiformat (X25519 + ChaCha20-Poly1305 + HKDF-SHA-256), nicht libsodium-kompatibel [S]. Brauchbar als Vorbild oder für Export/Backup-Dateien, nicht als Primitive-Bibliothek [S].

### 2.12 OpenSSL / BoringSSL / LibreSSL

- OpenSSL 4.0.3 (2026-09-29), LibreSSL 4.3.3 (2026-10-05), BoringSSL ohne Releases, Push 2026-10-06 [B]
- Haben Ed25519/X25519/ChaCha20-Poly1305, aber kein XChaCha20, BLAKE2b nur teils (OpenSSL), keine libsodium-Konstruktionen [S]. Für D&K nur indirekt (TLS) relevant [S].

### 2.13 libsignal

- AGPL-3.0, v0.104.0 (2026-10-02), Träger Signal [B]
- Nutzung außerhalb der Signal-Apps ausdrücklich nicht unterstützt, APIs ändern sich ohne Ankündigung [B] https://github.com/signalapp/libsignal
- Formale Analysen von PQXDH (ProVerif/CryptoVerif, USENIX Security 2024) [B] https://www.usenix.org/system/files/usenixsecurity24-bhargavan.pdf
- Bewertung: AGPL plus App-Store-Problematik (Issue #677) und Signal-spezifische Typen schließen es für D&K aus [S].

### 2.14 OpenMLS

- MIT, openmls 0.9.0 (2026-08-25), Träger Phoenix R&D / Community [B]
- Audit: SRLabs, 12 Wochen ab 2025-10-16, finanziert von der Sovereign Tech Agency, 8 Funde, davon 1 High (MAC-Authentisierung); Bericht v1.2 vom 2026-03-11 [B] https://blog.openmls.tech/SRL-OpenMLS_security_assurance_assessment.pdf
- GHSA 2025/2026: falsche Tag-Validierung, Secret-Tree-Persistenz [B]
- Bewertung: MLS lohnt sich erst bei geteilten Tagebüchern mit vielen Geräten/Personen. Für den Start Überdimensionierung [S].

### 2.15 Ergänzungen (nur das wirklich Relevante)

- **@serenity-kit/noble-sodium**: libsodium-kompatible API (box, sealed box, sign) auf noble, MIT, 93 085 Downloads/Woche, 24 Sterne, letzter Push 2026-02-03, **kein Audit** [B] https://github.com/serenity-kit/noble-sodium
- **shamir-secret-sharing** (Privy): TypeScript, Apache-2.0, auditiert von **Cure53 (Feb 2023)** und **Zellic (Jul 2023)**, 93 026 Downloads/Woche [B] https://github.com/privy-io/shamir-secret-sharing
- **Go stdlib + golang.org/x/crypto** (Server-Option): Trail of Bits 2025, ein Low-Fund in Go+BoringCrypto [B] https://go.dev/blog/tob-crypto-audit ; x/crypto hat XChaCha20-Poly1305, BLAKE2b, nacl/box, Argon2 [S]

## 3. Abdeckungsmatrix der D&K-Primitive

✔ = fertig vorhanden, (✔) = mit kleinem Eigenbau aus vorhandenen Bausteinen, ✘ = fehlt

| Primitiv | libsodium(.js) | noble | WebCrypto | RustCrypto/dalek | Monocypher | Tink |
|---|---|---|---|---|---|---|
| Ed25519 | ✔ | ✔ | ✔ (iOS 17+, WebView 137+) | ✔ | (✔) Zusatzmodul | ✔ |
| X25519 | ✔ | ✔ | ✔ (iOS 17+, WebView 133+) | ✔ | ✔ | ✔ (HPKE) |
| XChaCha20-Poly1305 | ✔ | ✔ | ✘ | ✔ | ✔ | ✔ |
| BLAKE2b (Hash, keyed) | ✔ | ✔ | ✘ | ✔ | ✔ | ✘ |
| `crypto_kdf` | ✔ | (✔) | ✘ | (✔) | (✔) | ✘ |
| `crypto_box_seal` | ✔ | (✔) bzw. noble-sodium | ✘ | ✔ (`crypto_box` + `seal`) | (✔) | ✘ |
| `sodium_pad` | ✔ | (✔) | (✔) | (✔) | (✔) | (✔) |
| HMAC | ✔ (js: SHA-512-256 std, SHA-256 nur sumo) | ✔ | ✔ | ✔ | ✘ | ✔ |
| Argon2id | ✔ (js: nur sumo) | ✔ (unauditiert, langsam) | ✘ | ✔ | ✔ | ✘ |
| Shamir | ✘ | ✘ | ✘ | ✘ (`vsss-rs`) | ✘ | ✘ |
| P-256 | ✘ | ✔ | ✔ | ✔ (`p256`) | ✘ | ✔ |

## 4. Empfehlung für Days & Keys

### 4.1 Client im WebView (Capacitor iOS/Android, Tauri-Frontend)

**noble statt libsodium.js** [S]. Gründe:

1. Audits: die tatsächlich gebrauchten noble-Teile (ed25519/x25519 in curves, xchacha in ciphers, blake2b/hmac in hashes) sind von Cure53 2022/2024 und Trail of Bits 2026 (laut README) geprüft [B]; für die Emscripten-Fassung von libsodium gibt es kein eigenes Audit [B].
2. Keine WASM-Initialisierung, kein Secure-Context-Bedarf, tree-shakeable, TypeScript-Typen ohne Laufzeitlücken (vgl. die `crypto_pwhash`-Falle der libsodium.js-Standard-Variante) [B/S].
3. Der libsodium-Autor empfiehlt für JS selbst WebCrypto plus noble [B].
4. Auf Android ohne aktuelles WebView und auf iOS unter `capacitor://` ist WebCrypto-Ed25519 nicht verlässlich verfügbar [B/S]; noble ist überall gleich.

**Was woher kommt:**

| Primitiv | Client-Quelle |
|---|---|
| Ed25519 | `@noble/curves/ed25519` mit **`zip215: false`** beim Verifizieren (strenges RFC 8032) |
| X25519 | `@noble/curves/ed25519` (`x25519`), lehnt Low-Order-Punkte ab wie libsodium [B: Quelltext montgomery.ts] |
| XChaCha20-Poly1305 | `@noble/ciphers/chacha` (`xchacha20poly1305`) |
| BLAKE2b, HMAC | `@noble/hashes` |
| `crypto_kdf` | Eigenbau, ~10 Zeilen: `blake2b(msg=leer, key=master, salt=LE64(subkey_id)‖0^8, personalization=ctx(8 Byte)‖0^8, dkLen)` [B: libsodium-Quelle `kdf_blake2b.c`] |
| `crypto_box_seal` | Eigenbau oder `@serenity-kit/noble-sodium`: ephemeres X25519, Nonce = BLAKE2b-24(epk‖pk), dann `crypto_box` = HSalsa20-Schlüssel + **XSalsa20**-Poly1305 [B: libsodium-Quelle `crypto_box_seal.c`; noble exportiert `hsalsa`, `xsalsa20poly1305`] |
| `sodium_pad` | Eigenbau (ISO/IEC 7816-4: 0x80 + Nullen bis Blockgrenze), ~15 Zeilen |
| Argon2id | wenn nötig: libsodium.js-**sumo** nur dafür, lazy geladen; noble-argon2 ist unauditiert und ~5x langsamer [B]. Besser: Argon2 nativ über Capacitor-Plugin bzw. Tauri-Command [S] |
| Shamir | `shamir-secret-sharing` (Privy, 2 Audits) [B] |
| Zufall | `crypto.getRandomValues` (braucht keinen Secure Context) [S] |

**Bitkompatibilität noble zu libsodium** [S, mit Begründung]:

- XChaCha20-Poly1305: beide implementieren draft-irtf-cfrg-xchacha (HChaCha20 + IETF-ChaCha20-Poly1305) [B: noble-README verweist auf den Draft; libsodium `_ietf`-Variante]. Ergebnis sollte bitgleich sein; per Testvektor prüfen.
- Ed25519-Signaturen: beide deterministisch nach RFC 8032, gleiche Signatur bei gleichem Seed. Unterschied im Schlüsselformat: libsodium-Secret-Key = 64 Byte (Seed‖PK), noble = 32-Byte-Seed. Verifikation unterscheidet sich in Randfällen (Small-Order, nicht-kanonische Punkte); noble per Default ZIP215 (nachsichtiger), daher `zip215: false` setzen und mit Randfall-Vektoren (z. B. „Taming the many EdDSAs") gegen libsodium testen.
- `crypto_secretstream` gibt es in noble nicht; wenn das Protokoll Streaming für große Anhänge braucht, entweder eigene Chunk-Konstruktion spezifizieren oder dort libsodium nutzen.
- `crypto_auth` von libsodium ist **HMAC-SHA-512-256** (gekürzt), nicht HMAC-SHA-256. Das Protokoll muss die Variante explizit nennen.

### 4.2 Tauri/Rust-Seite

- **Erste Wahl: `libsodium-sys-stable`** (native libsodium, vom libsodium-Autor) [S]. Grund: dieselbe Implementierung wie der Server, alle Primitive inkl. Argon2id nativ schnell, `crypto_kdf`/`box_seal`/`pad` fertig.
- **Alternative pure Rust:** `ed25519-dalek` 3 / `x25519-dalek` 3 / `chacha20poly1305` 0.11 (`XChaCha20Poly1305`) / `blake2` 0.11 / `crypto_box` 0.9 mit Feature `seal` / `argon2` 0.6 [B]. Nachteil: Audits beziehen sich auf alte Versionen, 2026 große API-Sprünge [S]. `dryoc` wegen fehlendem Audit nicht [S].
- Krypto bevorzugt in Rust-Commands statt im Tauri-WebView ausführen, Schlüsselmaterial bleibt dann im Rust-Prozess [S]. Tauri-IPC-Härtung beachten (u. a. CVE-2026-42184, Origin-Verwechslung bei IPC) [B] https://osv.dev/vulnerability/GHSA-7gmj-67g7-phm9

### 4.3 Server

- Der Server sieht bei Zero-Knowledge nur Chiffrate und prüft vor allem **Ed25519-Signaturen** (Auth, Integrität von Uploads) [S].
- **Bei Sprache Rust oder Go:** Rust mit `libsodium-sys-stable` (gleiche Engine wie Tauri) oder Go mit stdlib `crypto/ed25519` + `golang.org/x/crypto` (auditiert 2025) [B/S]. Bei Go: Ed25519-Verifikation weicht in Randfällen von libsodium ab, gleiche Testvektoren wie oben [S].
- **Bei Node/TypeScript:** noble wie im Client, so dass Client und Server dieselbe Implementierung nutzen; libsodium (native Addon `sodium-native`) nur, wenn Leistung nötig [S].
- **Referenztests:** ein CI-Job, der für jedes Primitiv Vektoren aus libsodium erzeugt und gegen noble (Client) und die Server-Bibliothek prüft [S].

### 4.4 Hardware-Schlüsselspeicher (nur P-256)

- Secure Enclave und StrongBox können kein Ed25519/X25519. Die 25519-Geheimnisse daher **in Hardware einhüllen**: P-256-ECDH (SE) bzw. AES-256-GCM-Schlüssel im Keystore/StrongBox als Key-Encryption-Key, das gewrappte 25519-Material liegt im App-Speicher [S].
- Das braucht ein natives Capacitor-Plugin (Swift/Kotlin) und auf Desktop den OS-Keychain über Tauri-Plugin [S]. Die eigentlichen Protokollschlüssel bleiben 25519 und damit libsodium-kompatibel [S].
- P-256 im Protokoll selbst (z. B. für Gerätebindungs-Attestierung) ginge mit noble/WebCrypto, verdoppelt aber die Kurvenvielfalt [S].

## 5. Risiken und offene Punkte

1. **Ein-Personen-Risiko** bei noble (Paul Miller) und libsodium (Frank Denis) [S]. Gegenmittel: Versionen pinnen, Lockfile-Hashes, Updates gezielt reviewen, Abhängigkeitsbaum klein halten (noble hat keine Fremdabhängigkeiten) [S].
2. **Supply-Chain im npm-Ökosystem** (vgl. Nx-Kompromittierung 2025, GHSA-cxm3-wv7p-598c) [B]. Für die Kryptopakete: `npm ci` mit Integrity-Hashes, Provenance prüfen, keine Range-Versionen [S].
3. **Konstante Laufzeit in JS** ist nicht garantierbar (sagt noble selbst) [B]. Für ein Tagebuch auf dem eigenen Gerät ist das Timing-Angriffsmodell schwach; für Server-seitige Geheimnisoperationen trotzdem native Bibliotheken [S].
4. **Trail-of-Bits-Audit 2026 von noble-curves**: nur im README genannt, PDF noch nicht veröffentlicht [B]. Nachreichen und lesen, bevor man es als Beleg zitiert [S].
5. **Eigenbauten** (`crypto_kdf`, `crypto_box_seal`, `sodium_pad`) sind klein, aber unauditiert [S]. Pflicht: Known-Answer-Tests gegen libsodium und unabhängiges Review vor Release.
6. **Ed25519-Randfälle** zwischen noble, libsodium, dalek und Go können unterschiedlich sein [B: Igalia-Interop-Bericht für Browser; S für Bibliotheken]. Eine Verifikationsregel im Protokoll festschreiben (strenges RFC 8032 inkl. Small-Order-Ablehnung) und alle Implementierungen dagegen testen.
7. **WebCrypto im Capacitor-iOS-WebView**: Secure-Context-Status unter `capacitor://` widersprüchlich belegt [B]. Wenn WebCrypto genutzt werden soll (z. B. nicht-extrahierbare Schlüssel), auf echtem Gerät mit iOS 17+ prüfen. Mit noble als Basis entfällt die Abhängigkeit [S].
8. **Tauri unter Linux** (WebKitGTK-Version der Distribution) und **Android ohne Google-WebView-Updates**: WebCrypto-Ed25519 nicht garantiert [S]. Spricht wieder für noble bzw. Krypto im Rust-Teil.
9. **Argon2id im Client**: noble langsam und unauditiert, libsodium.js nur in sumo (+375 KB) [B]. Entscheidung offen: nativ per Plugin, sumo lazy laden, oder auf scrypt ausweichen [S].
10. **Streaming großer Anhänge**: `crypto_secretstream` fehlt außerhalb libsodium [B]. Protokollentscheidung nötig [S].
11. **Post-Quanten**: libsodium 1.0.22 bringt X-Wing/ML-KEM768 [B], noble hat `@noble/post-quantum` 0.7.1 [B]. Für ein Tagebuch mit Langzeitvertraulichkeit sollte das Protokoll eine Hybrid-Option (X25519 + ML-KEM) zumindest vorsehen [S].
12. **Rust-Major-Sprünge 2026** (dalek 3.0/5.0, RustCrypto 0.11): Audit-Abdeckung bezieht sich auf alte Versionen [S].

## 6. Quellen (Auswahl)

- libsodium Release 1.0.22: https://github.com/jedisct1/libsodium/releases/tag/1.0.22-RELEASE
- libsodium-Audit 2017: https://www.privateinternetaccess.com/blog/libsodium-v1-0-12-and-v1-0-13-security-assessment/
- CVE-2025-69277: https://00f.net/2025/12/30/libsodium-vulnerability , https://nvd.nist.gov/vuln/detail/CVE-2025-69277
- libsodium.js: https://github.com/jedisct1/libsodium.js ; Denis-Zitat: https://github.com/jedisct1/libsodium.js/issues/327#issuecomment-1793419292
- noble: https://paulmillr.com/noble/ , https://github.com/paulmillr/noble-curves#security , https://github.com/paulmillr/noble-ciphers , https://github.com/paulmillr/noble-hashes , Cure53 2024: https://cure53.de/audit-report_noble-crypto-libs.pdf , Trail of Bits 2023: https://github.com/trailofbits/publications/blob/master/reviews/2023-01-ryanshea-noblecurveslibrary-securityreview.pdf
- noble-sodium: https://github.com/serenity-kit/noble-sodium
- TweetNaCl-js-Audit: https://tweetnacl.js.org/audits/cure53.pdf
- Tink JS entfernt: https://github.com/tink-crypto/tink/issues/689
- WebCrypto: MDN BCD https://github.com/mdn/browser-compat-data , Igalia https://blogs.igalia.com/jfernandez/2025/02/28/can-i-use-secure-curves-in-the-web-platform/ , https://blogs.igalia.com/jfernandez/2025/08/25/ed25519-support-lands-in-chrome-what-it-means-for-developers-and-the-web/
- Capacitor/Secure Context: https://forum.ionicframework.com/t/crypto-subtle/206836 , https://github.com/water-rs/waterui/issues/592 , https://bugs.webkit.org/show_bug.cgi?id=220184
- dalek-Audit Quarkslab 2019: https://blog.quarkslab.com/resources/2019-08-26-audit-dalek-libraries/19-06-594-REP.pdf
- RustCrypto-AEAD-Audit NCC 2020: https://www.nccgroup.com/research/public-report-rustcrypto-aesgcm-and-chacha20pluspoly1305-implementation-review/
- RustSec: https://rustsec.org/packages/curve25519-dalek.html , https://rustsec.org/advisories/RUSTSEC-2025-0007.html , https://nvd.nist.gov/vuln/detail/cve-2025-4432
- AWS-LC FIPS: https://aws.amazon.com/blogs/security/aws-lc-fips-3-0-first-cryptographic-library-to-include-ml-kem-in-fips-140-3-validation/
- Monocypher-Audit: https://monocypher.org/quality-assurance/audit
- OpenMLS-Audit: https://blog.openmls.tech/SRL-OpenMLS_security_assurance_assessment.pdf
- libsignal: https://github.com/signalapp/libsignal , PQXDH-Analyse https://www.usenix.org/system/files/usenixsecurity24-bhargavan.pdf
- Go-Krypto-Audit: https://go.dev/blog/tob-crypto-audit
- Shamir (Privy): https://github.com/privy-io/shamir-secret-sharing
- dryoc: https://github.com/brndnmtthws/dryoc
- Vulnerability-Abfragen: https://osv.dev (API `v1/query`, 2026-10-06)
