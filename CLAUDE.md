# CLAUDE.md - days-and-keys

> Dieses Repo folgt einem festen Software-Entwicklungsstandard. Verbindliche Kurzregeln:
>
> - **Jede Änderung über die CI-Pipeline (GitHub Actions)** - nie manuell/ad-hoc deployen.
> - **Issues & Bugs → GitHub-Issues** in DIESEM Repo.
> - **Roadmap → `ROADMAP.md` + GitHub-Milestones** - keine Insel-Roadmap im Fließtext.
> - **Jeder Commit referenziert ein Issue** (`… (#42)` bzw. `fixes #42`).
> - Secrets nie in Git.

## Was dieses Repo ist

App, mit der eine Person ihren Status jederzeit festhält, mit selbst bearbeitbaren Formularen, automatischen Datenquellen,
feldgenauen Freigaben und Mustererkennung. Stack (`docs/architektur.md`): Rust-Kern `dak-core`
mit libsodium, UniFFI, Android mit Kotlin und Jetpack Compose, iOS mit Swift und SwiftUI, keine
WebView (AE-04). M1 hält den Status lokal fest,
ohne Server; Etebase ist verworfen (#2). Protokoll: `docs/protokoll.md` (Teil A, M1) und
`docs/protokoll-freigaben.md` (Teil B, M1.5).
Noch kein Deploy-Ziel. Konzept und Leitplanken: `README.md`.

## Invarianten (bei jeder Änderung prüfen)

1. **Der Server sieht keinen Klartext**, auch keine Feldnamen, Formulardefinitionen oder Orte.
   Wer etwas serverseitig auswerten will, hat das Konzept verlassen.
2. **Kein Generalschlüssel**, keine Hintertür für Eltern, Admins oder Betreiber.
3. **Der Client wird nicht vom Datenserver nachgeladen.** Keine Änderung, die Code zur Laufzeit
   vom Server holt.
4. **Krypto nur aus geprüften Bausteinen** (libsodium), Protokolländerungen erst in
   `docs/protokoll.md` bzw. `docs/protokoll-freigaben.md` (#3), dann im Code.
5. **Keine echten Personen- oder Gesundheitsdaten in Repo, Tests oder Issues.** Testdaten sind
   erfunden.

## Pipeline

Die CI (GitHub Actions) führt bei jedem Push ein Governance-Gate und die Sicherheitsprüfungen
aus (`SECURITY.md`). `main` ist geschützt: kein direkter Push, Merge nur mit grüner Pipeline und
grünem Governance-Gate. Änderungen laufen also immer über einen Branch und einen PR.
