# Days & Keys - Gesamtkonzept

Stand 2026-10-08 · Version 2.4 des Konzepts (Architekturkonzept ergänzt) · Protokoll Teil A Entwurf v0.7.1 ·
Konzeptphase (M0), noch kein Code. Dieses Dokument fasst zusammen; die Einzelheiten stehen in den
verlinkten Dokumenten in diesem Repo.

## 1. Zusammenfassung

Mit Days & Keys hält eine Person ihren **Status jederzeit fest**: Befinden, Konzentration,
Medikation und Substanzkonsum, so oft sie will, auch mehrmals am Tag. Die Fragen stehen in
Formularen, die jede Person selbst anpassen kann. Die App zeigt den Verlauf über Stunden, Wochen
und Monate und erstellt auf Wunsch einen Bericht für die behandelnde Ärztin.

Die erste Ausbaustufe (M1) ist **persönlich und lokal**: ein schreibendes Gerät je Person, alles auf dem Gerät verschlüsselt, eine verschlüsselte Sicherung, die sich mit den
Recovery-Wörtern auch ohne Server wiederherstellen lässt. Freigaben an andere Personen und mehrere
gleichzeitig schreibende Geräte folgen nur, wenn ein Nutzungsversuch mit einer kleinen Testgruppe zeigt, dass sie
gebraucht werden (M1.5). Das Datenformat ist so festgelegt, dass sie dann ohne Neuverschlüsselung
vorhandener Werte dazukommen können.

Diese Fassung ersetzt Version 1.0. Anlass war die Erörterung mit ChatGPT und adversarischen
Claude-Agenten (Erörterung vom 2026-10-06, nicht öffentliches Archiv; Befunde als Issues): 75 Befunde, keiner widerlegt; die
wichtigsten Folgen sind ein deutlich kleineres M1 und ein neu gefasstes Protokoll. Version 2.1
ersetzt das Tagebuch mit Abend-Eintrag durch das jederzeitige Festhalten des Status und nimmt den
vollen Fragenkatalog wieder auf (Entscheidung Projektleitung, 2026-10-06). Version 2.2 setzt die zweite
Erörterung vom 2026-10-06 um (nicht öffentliches Archiv): Teil A legt kein Freigabeformat auf
Vorrat fest, Schlüssel werden bei Wiederherstellung und neuen Recovery-Wörtern rotiert, M1 ist in
M1a und M1b geteilt (#96).

## 2. Ziel und Leitplanken

**Ziel:** Veränderungen im Verlauf einer psychischen Erkrankung und ihrer Behandlung sichtbar
machen, besonders in Einstellungsphasen einer Medikation, mit wenig Aufwand und ohne dass die
Daten in fremde Hände geraten.

**Leitplanken** (aus `README.md`):

1. Kein Generalschlüssel, weder für Eltern noch für Administratoren noch für den Betreiber.
2. Jede Person hält ihren eigenen Status fest; Freigaben gehen nur von ihr aus.
3. Gesundheitsdaten verlassen das Gerät nur verschlüsselt oder nach ausdrücklicher Bestätigung.
4. Ehrlich zu den Grenzen: keine Empfehlungen, keine Diagnosen, keine automatischen Deutungen.
5. Keine selbst erfundene Kryptografie.
6. Keine Kontrollfunktionen: keine Lesebestätigungen, kein „zuletzt angesehen", kein „Zugriff
   anfordern".
7. Keine Umwege: Erinnerungen lokal und neutral, Spracherkennung nur offline, keine
   Cloud-Auswertung.
8. Erst Nutzen belegen, dann Komplexität bauen.

## 3. Nutzung und Rollen

| Rolle | Wer | Was sie tut |
|---|---|---|
| Person, die die App nutzt | jede Person, die die App nutzt | erfasst, passt Formulare an, sieht Verläufe, erstellt Arzt-PDF und Sicherung |
| Ärztin | ohne App | erhält das PDF, das die Person selbst erstellt und weitergibt |
| Betreiber | Blackfort Technology | Pipeline, Verteilung; in M1 kein Server mit Nutzerdaten |
| Empfänger von Freigaben | erst ab M1.5 | sieht nur freigegebene Felder im freigegebenen Zeitraum |

Zunächst nutzt eine kleine Testgruppe die App auf Android und iOS. Die
rechtliche Einordnung der Rollen wird vor der ersten Nutzung schriftlich geprüft (#77).

## 4. Ausbaustufen

| Stufe | Inhalt | Stand |
|---|---|---|
| **M0 Konzept** | Protokoll Teil A bis v1.0, Framework-Entscheidung, Wegwerf-Prototyp, Bedrohungsmodell gegen Teil A | Pipeline steht; Etebase verworfen; drei Erörterungen abgeschlossen; Teil A Entwurf v0.7 |
| **M1a Gehgerüst und Nutzungsversuch** | Status jederzeit mit vollem Katalog, Ereignisse ohne Substanzen, Medikation, Verlauf, Hilfe, automatische Sicherung; Start des Nutzungsversuchs (#82) | geplant |
| **M1b Ausbau während des Versuchs** | Formulareditor, Substanzen hochsensibel, Arzt-PDF, Zufallsabfragen, Vorher/Nachher, Export | geplant |
| **M1.5 Freigaben und Mehrgeräte** | Protokoll Teil B mit eigener Prüfung | nur bei Bedarf nach dem Nutzungsversuch |
| **M2 Erweiterter Alltag** | Wetter, allgemeine Mustererkennung, weitere Vorlagen | später |
| **M3 Anbindung** | Standort, Health Connect / HealthKit, Kalender, Shamir-Recovery | später |

## 5. Funktionen in M1

### 5.1 Status jederzeit festhalten

Die App ist kein Tagebuch mit einem Eintrag am Abend. Ein Status lässt sich jederzeit festhalten,
auch mehrmals am Tag; jeder Status ist ein Messpunkt mit Zeitpunkt. Der volle Fragenkatalog steht
in `docs/vorlagen.md`:

| Baustein | Inhalt | Hinweis |
|---|---|---|
| **Status jetzt** (ein Bildschirm) | Stimmung, Antrieb, Konzentration, innere Unruhe, Anspannung, Gereiztheit, Müdigkeit, Appetit, Wirkung der Medikation gerade (nur wenn passend), körperliche Beschwerden jetzt, Notiz | zwei Gruppen, Tipp-Reihe ohne Vorauswahl; Dauer wird im Prototyp gemessen |
| **Seit dem letzten Status** (zugeklappt, ab zwei Stunden Abstand) | vieles gleichzeitig angefangen, vergessen woran ich gearbeitet habe, etwas vergessen, am Stück gearbeitet, zuhören, unterbrochen, aufgeschoben, Stimmungsschwankungen, Beschwerden zwischendurch, äußere Einflüsse | Zeitraum steht in der Überschrift und wird gespeichert; „kam nicht vor" möglich |
| **Ereignisse** (ein Tipp) | Tablette genommen oder ausgelassen (mit Grund), Wirkung lässt nach, Substanz, Craving, Mahlzeit, Bewegung, schlafen gegangen, aufgewacht (mit Schlafqualität), Termin pünktlich, Belastung, Krise, Arzttermin, Dosiswechsel | Substanzen ohne PIN eintragbar (M1b) |
| **Tagesabschluss** (optional) | Handyzeit, Tag insgesamt, gegen Abend gereizt oder erschöpft, Besonderes | Lebenstag mit eigenem Tageswechsel |
| **Wochenrückblick** | allgemein vergesslich, angefangen und nicht beendet, Zurechtkommen mit der Medikation, Notiz fürs Arztgespräch | sonntags |
| **Zufällige Abfragen** (optional, Standard aus) | zwei bis vier kurze Status am Tag zu zufälligen Zeiten in eigenen Zeitfenstern, gegen die Verzerrung, nur in schlechten Momenten einzutragen | verpasste Abfragen werden gespeichert; M1b |

Zustände jetzt auf 0-10, Häufigkeiten seit dem letzten Status auf der Fünferskala. Nichts wird mit
früheren Werten vorbelegt; Einnahmen zählen nur, wenn sie bestätigt sind.

### 5.2 Anpassbare Formulare

Fragen ein- und ausblenden, ergänzen, umbenennen, umsortieren (#6). Jede Frage beruht auf einer
**Messdefinition** (Frage, Skala, Einheit, Polung, Schutzklasse). Ändert sich nur der Anzeigename,
bleibt die Messreihe; ändert sich die Bedeutung oder Skala, beginnt eine neue Reihe, und die
Auswertung vergleicht beide nicht.

### 5.3 Medikation und Substanzen

- Präparat und Wirkstoff, Stärke, Einnahmeplan, Bedarfsmedikation (#21). Bei unklarem Stand
  zeigt die App „Stand unbekannt".
- Substanzkonsum mit Getränke-Buttons und frei erweiterbarer Substanzliste (#26), **hochsensibel**:
  Eintragen geht mit einem Tipp ohne PIN, Lesen nur nach frischer App-PIN; auf der Startseite
  stehen nur selbst gewählte, neutrale Bezeichnungen; im Arzt-PDF nur nach ausdrücklicher Auswahl.
  Kommt in M1b.

### 5.4 Verläufe

Jeder Status ist ein Punkt im Verlauf. Die App zeigt die einzelnen Punkte, je Tag Mittelwert,
Spanne und Zahl der Status, Tage ohne Status bleiben leer. Eine Tagesansicht ordnet die Punkte
nach Uhrzeit und markiert die Einnahmen, ohne eine Wirkungskurve zu glätten. Status aus zufälligen
Abfragen lassen sich getrennt ansehen. Medikation, Ereignisse und Konsum stehen auf derselben
Zeitachse; Dosiswechsel als neutrale Gegenüberstellung zweier gleich langer Zeiträume ohne
Farbwertung (#23, #74). **Nicht in M1:** automatische Zusammenhänge, geglättete Wirkungskurven,
Rebound-Erkennung, Hinweise wie „auffällig".

### 5.5 Arzt-PDF, Sicherung, Hilfe

- Arzt-PDF über einen wählbaren Zeitraum, nur nach nativer Bestätigung; Hinweis, dass es die
  Verschlüsselung verlässt (#36).
- **Automatische** verschlüsselte Sicherung in einen einmal gewählten Ordner (auch in einer Cloud),
  mehrere Stände, Warnung bei Ausfall; Wiederherstellung auf einem neuen Gerät mit den 24
  Recovery-Wörtern, ohne Server (#91, #79). Restore-Test vor dem Nutzungsversuch ist Pflicht.
- Hilfe jederzeit erreichbar, auch ohne Entsperren: TelefonSeelsorge 0800 111 0 111,
  0800 111 0 222 und 116 123, Nummer gegen Kummer 116 111, ärztlicher Bereitschaftsdienst 116 117, Notruf 112, eigene Vertrauenspersonen
  (#71). Keine Überwachung, keine Meldung an Dritte.

## 6. Gestaltung

- Status jetzt auf einem Bildschirm, Ereignisse mit einem Tipp von der Startseite, große
  Bedienelemente, Lücken ohne Strafe, keine Streaks.
- Ruhiger, sachlicher Ton, keine Bewertung von Konsum oder ausgelassenen Einnahmen.
- Weißer Grund, Petrol (#0E7C7B), Schrift Inter. Das Mockup (nicht öffentlich) zeigt noch den
  größeren Umfang von Version 1.0 und wird mit dem Prototyp überarbeitet.

## 7. Architektur

```
┌──────────────── Oberfläche (TypeScript in der WebView) ──────────────────┐
│ Formulare, Verläufe · sieht nie Schlüssel · kein Netzzugang              │
│ fremde Texte nur als Text · CSP ohne inline/eval                         │
└───────────────┬──────────────────────────────────────────────────────────┘
                │ ein Mobile-Framework: Capacitor oder Tauri 2 Mobile (#33)
┌───────────────▼──────────────────────────────────────────────────────────┐
│ dak-core (Rust) · libsodium                                              │
│ Protokoll, Schlüssel, Autorisierung, native Bestätigungen, Sitzung       │
│ Hardware-Wrapping: Secure Enclave · StrongBox/TEE                        │
└───────────────┬──────────────────────────────────────────────────────────┘
                │ verschlüsselte Sicherungsdatei (Gerät, Cloud oder Server)
┌───────────────▼──────────────────────────────────────────────────────────┐
│ Speicher nach Wahl · sieht nur Chiffretext und die Dateigröße            │
└──────────────────────────────────────────────────────────────────────────┘
```

In M1 gibt es keinen eigenen Server. Ab M1.5 kommt eine schlanke API dazu, die nur Chiffretext
sieht (Etebase wurde geprüft und verworfen, `docs/unterbau-etebase.md`).

## 8. Kryptokonzept (Teil A)

Vollständig in `docs/protokoll.md` (v0.6); hier die wichtigsten Punkte.

- **Bausteine:** libsodium im Rust-Kern, Suite-Kennung je Objekt, deterministisches CBOR,
  unbekannte Felder werden abgelehnt, jeder Schlüssel hat genau einen Zweck.
- **Kein Freigabeformat auf Vorrat:** Was Freigaben brauchen, kommt mit Teil B; vor der ersten
  Freigabe werden die Daten lokal umgeschlüsselt (#96).
- **Identität:** Gründungsobjekt und eine signierte Kontokette (Rotation, Gerät ersetzt, neue
  Recovery-Wörter). Recovery mit getrenntem Signatur- und Verschlüsselungsschlüssel; kein Gerät hält
  den Recovery-Schlüssel.
- **Daten:** Jeder Status, jedes Ereignis, jede Abfrage ist ein Datensatz mit eigenem Schlüssel,
  umhüllt je Schutzklasse und Schlüsselgeneration. Zeit, Art und Werte sind verschlüsselt; keine
  Signaturen über Werte. Löschen entfernt die Umhüllung, damit ist der Inhalt im aktiven Bestand
  nicht mehr zu öffnen; Sicherungen folgen nach Ablauf der Aufbewahrung.
- **Rotation:** Bei Wiederherstellung, neuen Recovery-Wörtern, Suite-Wechsel und auf Wunsch entstehen
  neue Schlüssel; alte Geräte und alte Wörter öffnen danach nichts Neues.
- **Schutzklassen:** Gesundheit als Standard; hochsensibel in eigenem Bereich, auch Art, Tag,
  Häufigkeit und Bezeichnungen. Schreiben über einen Schreibschlüssel ohne PIN, Lesen nur mit
  frischer App-PIN.
- **Entsperren:** Geräteauthentifizierung (Biometrie oder Geräte-Code) für die Gesundheitsdaten
  (#99); die App-PIN schützt nur den hochsensiblen Bereich. PIN und Recovery-Wörter nur in
  nativen Ansichten.
- **Statusmodell im Format:** `status_id`, gespeicherter Bezugszeitraum, Antwortzustände
  (beantwortet, übersprungen, kam nicht vor), Abfrage-Protokoll, Lebenstag mit eigenem Tageswechsel.
- **Bridge:** fachliche Operationen, native Bestätigungen für Export, Sicherung, Restore, PIN,
  Stufen und Recovery; Android-App ohne Netzberechtigung, Hilfe ohne Entsperren.

## 9. Sicherheit und ehrliche Grenzen

| Lage | geschützt | nicht geschützt |
|---|---|---|
| Sicherungsdatei in fremden Händen | alle Inhalte | Größe, Zeitpunkt |
| Gesperrtes Gerät gestohlen | alle Inhalte | Existenz der App |
| Jemand kennt den Geräte-Code oder erzwingt die Biometrie | Inhalt und Bezeichnungen hochsensibler Einträge | alle Gesundheitsdaten (#99) |
| Erzwungene Herausgabe der App-PIN | nichts | alles (rechtliche Frage) |
| Kompromittierter WebView-Code bei entsperrter App | Schlüssel, hochsensible Felder ohne frische PIN, Export und Sicherung | angezeigte Daten |
| Manipulierter Build oder nativer Code | nichts | alles (Lieferkette, #68) |
| Altes Gerät nach Wiederherstellung | alles Neue (Rotation) | was es bis dahin hatte |
| Gerät und Recovery-Wörter verloren | nichts | alle Daten (gewollt) |

Das Bedrohungsmodell (`docs/threat-model/`) bezieht sich noch auf Protokoll v0.1 und wird gegen
Teil A neu erstellt (#66), bevor v1.0 freigegeben wird.

## 10. Datenschutz und Recht

- Gesundheitsdaten (Art. 9 DSGVO) bleiben auf dem Gerät oder verschlüsselt; keine Auswertung bei
  Dritten.
- Rollen und Verantwortlichkeiten (#77) werden vor der ersten Nutzung schriftlich geprüft;
  Altersgrenzen und Einwilligung prüfen (#77).
- Medizinprodukt: Zweckbestimmung und Funktionen (nicht nur ein Hinweis „keine Empfehlungen")
  werden vor einer Nutzung über den Testkreis hinaus geprüft (#24).
- Fragen sind eigene Formulierungen; Nähe zu standardisierten Fragebögen wird vor dem Mitliefern
  geprüft.
- Die Medikamentenerinnerung ersetzt keinen Medikamentenwecker; die App sagt das (#42).
- Angaben zu illegalen Substanzen schützt die App-PIN gegen erzwungene Biometrie; gegen erzwungene
  Herausgabe der PIN schützt sie nicht.

## 11. Entwicklung und Betrieb

| Thema | Umsetzung |
|---|---|
| Repo, Pipeline | dieses Repo, CI (GitHub Actions) mit Governance-Gate und Security-Scans, `main` geschützt, alles per PR |
| Abhängigkeiten | gepinnt, `AUDITS.md`, Updates nach festen Regeln; native Abhängigkeiten, Toolchains, Trennung von Bau und Signatur (#68) |
| iOS | macOS-Cloud-Dienst aus der CI (GitHub Actions), Signatur aus einem Secret-Store, TestFlight mit Notfallplan für den Ablauf nach 90 Tagen (#8, #78); Apple-Konto (#10) |
| Android | signierte APK aus der CI (GitHub Actions) (#9) |
| Tests | Testvektoren, Negativtests, Bridge-Tests, echte Geräte für Hardware und Biometrie (#80) |
| Server | in M1 keiner; ab M1.5 `daysandkeys.wirdsichtbar.de` (#14) |

## 12. Nächste Schritte

1. Linux-Kern des Prototyps sofort (#100), parallel Framework-Entscheidung (#33) und Apple-Konto (#10).
2. Plattformwege auf iOS (Cloud-Bau, kein Mac) und Android (#101).
3. Teil A auf v1.0: Formate und Testvektoren (#63), Bedrohungsmodell gegen Teil A (#66).
4. M1a bauen, Restore testen, Nutzungsversuch: direkt Phase A (drei Wochen, eingefroren), dann
   Phase B (drei Wochen, M1b-Teile an der Phasengrenze) (#82).
5. Danach entscheiden, ob M1.5 kommt.

## 13. Offene Entscheidungen

| Frage | Bezug |
|---|---|
| Oberfläche: nativ mit Compose und SwiftUI (Vorschlag), Flutter oder WebView | #33, `docs/architektur.md` AE-04 |
| Lizenz: MPL-2.0 für Kern und Apps, AGPL-3.0 für den Server, CC BY 4.0 für die Doku (Vorschlag) | `docs/architektur.md` AE-02 |
| Argon2id-Parameter und Fehlversuchsregel der App-PIN | #60 |
| Rollen und Verantwortlichkeiten | #77 |
| Freigabeformat für M1.5 (Monats- und Tagesketten, Metadaten am Server) | Teil B, #55 |

## 14. Risiken

| Risiko | Umgang |
|---|---|
| Nutzen zeigt sich im Alltag nicht oder die Erfassung wird zu viel | Dauer im Prototyp messen, alles abschaltbar, Nutzungsversuch mit M1a vor weiterer Komplexität |
| Verzerrung, weil vor allem in schlechten Momenten erfasst wird | optionale Zufallsabfragen, getrennte Anzeige, Zahl der Status je Tag sichtbar |
| Kritische Bibliotheken hängen an wenigen Personen | gepinnte Versionen, Audits dokumentiert, Testvektoren |
| iOS-Builds nicht reproduzierbar, TestFlight läuft ab | Hash-Abgleich, Notfallplan, Apple-Konto (#10) |
| Datenverlust | Sicherung ohne Server, Restore-Test, gedruckte Recovery-Wörter |
| Fehldeutung der Verläufe | keine automatischen Deutungen, Fallzahl und Lücken sichtbar |
| App wird als Kontrollinstrument empfunden | keine Freigaben in M1, Leitplanke 6 |

## 15. Dokumente

| Dokument | Inhalt |
|---|---|
| `README.md`, `ROADMAP.md` | Überblick, Ausbaustufen, Leitplanken, Milestones |
| `docs/architektur.md` | Architekturkonzept: Bausteine, Abläufe, Verteilung, alle Architekturentscheidungen; Recherchen in `docs/architektur/` |
| `docs/protokoll.md` | Teil A: Kern für M1 (Entwurf v0.7.1) |
| `docs/protokoll-freigaben.md` | Teil B: Freigaben und Mehrgeräte (Entwurf, M1.5) |
| Erörterungen vom 2026-10-06 (nicht öffentliches Archiv) | erste und zweite Erörterung: Einzelberichte, Schiedsrichter, Ergebnis; Befunde als Issues |
| `docs/vorlagen.md` | Fragen und Erfassung: Status jetzt, seit dem letzten Status, Ereignisse, Rückblicke |
| `docs/threat-model/` | Bedrohungsmodell (Stand v0.1, Neufassung #66) |
| Mockup (nicht öffentlich) | Stand Version 1.0 des Konzepts |
| `docs/unterbau-etebase.md`, `docs/krypto-bibliotheken.md` | Bewertungen |
| `AUDITS.md`, `CLAUDE.md`, `SECURITY.md` | Abhängigkeiten, Regeln für Mitarbeit und Sicherheit |
