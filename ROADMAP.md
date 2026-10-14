# Roadmap - Days & Keys

Der mittelfristige Bogen dieses Repos. Operative Aufgaben leben als **GitHub-Issues**;
hier stehen nur die größeren Linien mit Verweis auf Issues und Milestones.

Neu geschnitten am 2026-10-06 nach der Erörterung des Gesamtkonzepts
(nicht öffentliches Archiv; Befunde als Issues): erst eine schlanke, lokale App zum Festhalten des Status und ein
Nutzungsversuch, Freigaben und Mehrgeräte nur, wenn der Versuch den Bedarf zeigt.

## Jetzt: M0 Konzept bis Protokoll v1.0 (Teil A)

- [x] Projektgrundlage, Pipeline, Branch-Schutz (#1, #5)
- [x] Etebase geprüft und verworfen (#2)
- [x] Bedrohungsmodell zu v0.1 (#4), Erörterung mit ChatGPT und adversarischen Agenten
- [x] Entscheidungen nach der dritten Erörterung: Geräteentsperrung genügt, M1a bleibt, Versuch
      direkt mit Phase A, iOS per Cloud-Bau (#99)
- [x] Architekturkonzept mit allen Entscheidungen (`docs/architektur.md`, #33)
- [x] Entscheidungen (2026-10-14): Oberfläche nativ (AE-04, #33), Lizenz MPL-2.0 (AE-02),
      iOS-Bau und das ganze Projekt auf GitHub, öffentlich (AE-30, AE-39, #106)
- [x] Prototyp Linux-Kern mit Sicherung, Restore, Rotation, Testvektoren und Lastprobe (#100,
      `docs/prototyp-kern.md`), CI mit GitHub Actions
- [ ] Plattformwege mit Compose und SwiftUI (#101)
- [x] Entscheidung: kein Freigabeformat auf Vorrat, M1 in M1a und M1b geteilt (#96)
- [x] Formate bytegenau mit Testvektoren (`docs/format.md`, `core/testvectors/v1/`, #63)
- [x] Teil A v1.0, Format für M1a eingefroren (`docs/protokoll.md`, #3): Rotation (#86),
      Wiederherstellung (#87), Statusmodell (#90), Zeitmodell (#92), Löschung (#62), Formate (#63)
- [ ] nach v1.0: hochsensible Metadaten und Schreibweg (#88, #89) und App-PIN (#60) mit M1b,
      Formular-Engine (#6), Plattformmatrix (#67), Tests auf Hardware (#80), Apple-Konto (#10),
      Issue-Pflege (#95)
- [ ] Bedrohungsmodell gegen Teil A (#66), Begleitdokumente angleichen

## Jetzt: M1a Gehgerüst und Nutzungsversuch

Status jederzeit festhalten, beliebig oft am Tag (Entscheidung 2026-10-06). Ein schreibendes Gerät
je Person, Android (Compose) und iOS (SwiftUI) über UniFFI am selben Kern, keine Freigaben.

- [x] `dak-core` mit fachlicher Schnittstelle (#32, `dak_core::api`), App ohne Netz (#59),
      Entsperren über die Geräteauthentifizierung, Hilfe ohne Entsperren mit eigenem Krisenplan (#71)
- [x] voller Fragenkatalog als feste Vorlage (`templates/standard-v1.json`: Status jetzt, seit dem
      letzten Status, Ereignisse ohne Substanzen, Tagesabschluss mit Stimmungsverlauf, Wochenrückblick;
      #22, #75, #105), Medikament und Einnahme ohne Vorbelegung (#21, #73)
- [x] Verlauf mit Einzelpunkten, Tageswerten, Grundniveau und Lücken (#43, #74), Entwürfe (#94)
- [x] automatische Sicherung und Restore (#91, #79), Erinnerungen (#42), Client-Härtung (#69)
- [x] Android-APK aus der CI, signiert im Release-Lauf (#9)
- [ ] iOS über TestFlight (#8, #78): wartet auf das Apple-Konto der UG (#10)
- [ ] interne Sicherheitsprüfungen des Lab-Jenkins als Pflicht-Check (#109)
- [ ] Lieferkette: Gradle-Lockfile, Prüfsummen (#68); Schutzklassen-Feinschliff (#70)
- [ ] Messungen auf den Zielgeräten: Entsperrzeit, Erinnerungen nach Neustart, Sicherungsordner (#80, #67)
- [ ] Rechtliche Einordnung der Rollen (#77)
- [ ] **Nutzungsversuch mit einer Testgruppe (#82):** direkt Phase A (drei Wochen, M1a eingefroren),
      dann Phase B (drei Wochen, M1b-Teile an der Phasengrenze)

## Während des Versuchs: M1b Ausbau

Formulareditor, Substanzen hochsensibel mit Schreibweg (#26, #89), Arzt-PDF (#36),
Zufallsabfragen (#93), Vorher/Nachher (#23), Export und Konto löschen (#39). Danach Entscheidung
über M1.5.

## Danach, nur bei Bedarf: M1.5 Freigaben und Mehrgeräte

Eigenes Paket mit eigener Prüfung (`docs/protokoll-freigaben.md`): Konsistenzmodell (#51),
Gerätekette (#52), Logs und verborgene Felder (#55), Einladungen und Kontaktprüfung (#56, #57),
Bestätigungen (#58), Freigabebindung (#61), Transport (#65), Freigaben in engen Beziehungen (#76, #41),
neue Felder nachfragen (#40), Server und Kontoanlage (#14, #38).

## Später

- **M2 Erweiterter Alltag:** Wetter (#7), allgemeine Mustererkennung (#13), weitere Vorlagen
- **M3 Anbindung:** Standort (#11), Health Connect und HealthKit (#12), Kalender, Ernährung,
  Shamir-Recovery (#34)
- Vor einer Veröffentlichung: Medizinprodukt (#24), Markenprüfung
- Ideen: Desktop-App (Tauri), Formularvorlagen teilen, F-Droid offiziell
