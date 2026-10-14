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
- [ ] Teil A auf v1.0 (`docs/protokoll.md`, derzeit v0.7.1, #3): Rotation (#86), Wiederherstellung
      (#87, #79), hochsensible Metadaten und Schreibweg (#88, #89), Statusmodell (#90), Zeitmodell
      (#92), App-PIN und Biometrie (#60), Löschung (#62), Formate (#63), Formular-Engine (#6),
      Plattformmatrix (#67), Tests auf Hardware (#80), Apple-Konto (#10), Issue-Pflege (#95)
- [ ] Bedrohungsmodell gegen Teil A (#66), Begleitdokumente angleichen

## Als Nächstes: M1a Gehgerüst und Nutzungsversuch

Status jederzeit festhalten statt Tagebuch (Entscheidung 2026-10-06). Ein schreibendes Gerät je
Person, Android und iOS über ein Framework, keine Freigaben.

- `dak-core` mit Bridge-Regeln (#32), App ohne Netz (#59), Entsperren über die Geräteauthentifizierung,
  Hilfe ohne Entsperren (#71)
- voller Fragenkatalog als feste Vorlage (Status jetzt, seit dem letzten Status, Ereignisse ohne
  Substanzen, Tagesabschluss, Wochenrückblick; #22, #75), Medikament und Einnahme ohne Vorbelegung
  (#21, #73), Schutzklassen (#70)
- Verlauf mit Punkten und Lebenstagen (#43, #74), Entwürfe beim Sperren (#94)
- automatische Sicherung und Restore (#91, #79), Erinnerungen (#42), Client-Härtung (#69)
- iOS-Bau und TestFlight mit Notfallplan (#8, #78), Android-APK (#9), Lieferkette (#68)
- Rechtliche Einordnung der Rollen (#77)
- **Nutzungsversuch mit einer Testgruppe (#82):** direkt Phase A (drei Wochen, M1a eingefroren), dann Phase B
  (drei Wochen, M1b-Teile an der Phasengrenze)

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
