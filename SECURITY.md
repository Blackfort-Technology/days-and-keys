# Security Policy - days-and-keys

## Schwachstelle melden

Über GitHub Security Advisories (privat) in diesem Repo. Vertrauliche Funde,
die nicht offen stehen sollen, alternativ an `info@blackfort-tec.de`.

## Pipeline-Gates

Jeder Pull Request durchläuft die CI (GitHub Actions, `.github/workflows/`): Governance-Prüfung
(Issue-Bezug, Pflichtdateien), Gitleaks über die Historie, `cargo-deny` (Advisories, Lizenzen,
verbotene Pakete), `cargo-audit`, Rust-Tests mit Testvektoren sowie die Android-Prüfung „keine
`INTERNET`-Berechtigung". Ein roter Check verhindert den Merge nach `main`.

## Besonderheit dieses Projekts

Days & Keys verarbeitet Gesundheitsdaten (Art. 9 DSGVO) und verspricht Zero-Knowledge.
Als Schwachstelle gilt deshalb auch alles, was dem Server Klartext, Feldnamen, Orte oder
Schlüsselmaterial zugänglich macht, und jede Möglichkeit, Clientcode zur Laufzeit vom
Datenserver nachzuladen. Solche Funde ebenfalls über GitHub Security Advisories melden.
