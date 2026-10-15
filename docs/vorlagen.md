# Fragen und Erfassung (Entwurf)

Stand 2026-10-14. Issues #6, #21, #22, #26, #74, #75, #105. Maßgeblich für M1a ist die Datei
`templates/standard-v1.json`; diese Seite beschreibt sie.

**Grundprinzip: den Status jederzeit festhalten.** Die App ist kein Tagebuch mit einem Eintrag am
Abend. Eine Person hält ihren Zustand fest, wann immer sie will, auch mehrmals am Tag: morgens,
mittags, nachmittags, nach einem Konflikt. Dazu
kommen Ereignisse mit einem Tipp (Einnahme, Substanz, Mahlzeit) und auf Wunsch ein Tages- und ein
Wochenrückblick. Alle Fragen sind **Vorlagen**: Jede Person kann sie umformulieren, ausblenden,
ergänzen oder ersetzen. Die Formulierungen sind eigene, keine standardisierten Fragebögen.

## Grundsätze

- **Drei Arten von Fragen:**
  - **jetzt**: Zustand im Moment der Erfassung (Stimmung jetzt, Unruhe jetzt)
  - **seit dem letzten Status**: was in der Zwischenzeit passiert ist (vergessen, unterbrochen,
    aufgeschoben); bezieht sich auf den Zeitraum seit dem vorigen Status, höchstens die letzten
    zwölf Stunden
  - **allgemein**: nur im Wochenrückblick, damit sie den Verlauf nicht verwischen
- **Skalen:** Zustände jetzt auf 0-10 (0 = gar nicht, 10 = sehr stark), Häufigkeiten seit dem
  letzten Status auf der Fünferskala nie, selten, manchmal, oft, sehr oft. Symptomfragen sind
  gleich gepolt (höher = mehr Beschwerden), Befindensfragen umgekehrt (höher = besser) und in der
  Oberfläche deutlich getrennt.
- **Schnell:** Der Status jetzt passt auf einen Bildschirm, der Block „seit dem letzten Status" ist
  zugeklappt und erscheint ab zwei Stunden Abstand; sein Zeitraum steht in der Überschrift („seit
  10:15 Uhr") und wird gespeichert. Die Dauer wird im Prototyp gemessen (#81), nicht angenommen.
- **Antworten:** Jede Frage kann beantwortet, übersprungen oder, wo sie eine Gelegenheit braucht,
  mit „kam nicht vor" beantwortet werden. Regler haben keine Startposition; gespeichert wird, ob ein
  Wert aktiv gesetzt wurde. Nichts wird mit früheren Werten gefüllt.
- **Lebenstag:** Jede Person legt ihren Tageswechsel fest (Standard 4:00 Uhr), damit späte Abende
  zum richtigen Tag zählen.
- **Jeder Status hat einen Zeitpunkt.** Wer etwas nachträgt, stellt die Zeit ein; Erfassungs- und
  Beobachtungszeit werden getrennt gespeichert (Protokoll Abschnitt 6).
- **Messdefinitionen:** Wird eine Frage umformuliert oder ihre Skala geändert, entsteht eine neue
  Messreihe; die Auswertung vergleicht alte und neue Reihe nicht (Protokoll Abschnitt 5).
- **Schutzklassen:** Gesundheit ist Standard für alle Felder einschließlich Freitext und neuer
  Felder; hochsensibel für Substanzkonsum (eigene App-PIN, im Export nur auf Wunsch); normal nur
  nach ausdrücklichem Herabstufen (Protokoll Abschnitt 8.1).

## 1. Status jetzt

Standardmäßig eingeschaltet. Ein Bildschirm, 0-10 als Tipp-Reihe ohne Vorauswahl, in zwei sichtbar
getrennten Gruppen: „Wie geht es mir" (höher = besser) und „Was ist gerade stark" (höher = mehr).

| Frage | Polung |
|---|---|
| Wie ist meine Stimmung? (bipolar -5 bis +5, 0 = neutral; dieselbe Frage im Tagesabschluss, #105) | höher = besser |
| Wie viel Antrieb habe ich gerade? | höher = besser |
| Wie konzentriert bin ich gerade? | höher = besser |
| Wie unruhig bin ich innerlich? | höher = mehr |
| Wie angespannt oder ängstlich bin ich? | höher = mehr |
| Wie gereizt bin ich? | höher = mehr |
| Wie müde bin ich? | höher = mehr |
| Wie ist mein Appetit gerade? (bipolar -2 bis +2: viel weniger als sonst bis viel mehr als sonst, #75) | bipolar |
| Spüre ich die Wirkung meiner Medikation gerade? (nur wenn heute eine Einnahme eines Präparats bestätigt ist, dessen Wirkung im Tagesverlauf spürbar ist; vierstufig: nein, etwas, deutlich, stark) | höher = stärker |
| Körperliche Beschwerden jetzt | Mehrfachauswahl: Kopfschmerzen, Herzklopfen, Magen, Verspannung, Mundtrockenheit, keine, + eigene; je Stärke. Was zwischendurch war und vorbei ist, gehört in „seit dem letzten Status" |
| Notiz | Freitext oder Sprachnotiz (nur Offline-Spracherkennung) |

## 2. Seit dem letzten Status

Standardmäßig eingeschaltet, zugeklappt. Fünferskala nie bis sehr oft, wo nicht anders angegeben;
Fragen, die eine Gelegenheit brauchen, haben zusätzlich „kam nicht vor".

| Frage | Feldtyp |
|---|---|
| Habe ich viele Dinge gleichzeitig angefangen? | Häufigkeit |
| Habe ich vergessen, woran ich zuvor gearbeitet habe? | Häufigkeit |
| Habe ich etwas vergessen (Termine, Dinge, Absprachen)? | Häufigkeit |
| Konnte ich Aufgaben am Stück bearbeiten? | Fünferskala gar nicht, kaum, teils, meist, immer; „kam nicht vor" |
| Konnte ich gut zuhören, wenn mir jemand etwas erklärt hat? | Fünferskala gar nicht, kaum, teils, meist, immer; „kam nicht vor" |
| Habe ich andere unterbrochen oder vorschnell geantwortet? | Häufigkeit; „kam nicht vor" (keine Gespräche) |
| Habe ich Wichtiges aufgeschoben? | Häufigkeit |
| Ist meine Stimmung stark geschwankt? | Häufigkeit |
| Hatte ich zwischendurch Beschwerden, die jetzt vorbei sind? | Mehrfachauswahl wie bei den Beschwerden, dazu „neu" oder „wie bisher" (keine Doppelzählung mit „jetzt") |
| Gab es äußere Einflüsse? | Mehrfachauswahl: Termindruck, Konflikt, Unterbrechungen, wenig Struktur, Reise, krank, Zyklus, + eigene; Freitext |

## 3. Ereignisse (jederzeit, ein Tipp)

Erscheinen als Knöpfe auf der Startseite; Zeitpunkt ist jetzt, änderbar.

| Ereignis | Angaben |
|---|---|
| Tablette genommen | Präparat aus dem Einnahmeplan, Uhrzeit; ausdrücklich bestätigt, nie vorbelegt |
| Einnahme ausgelassen | Grund: vergessen, bewusst ausgelassen, Pause, keine mehr da, Nebenwirkungen, anderes |
| Wirkung lässt nach | Uhrzeit; danach optional ein Status jetzt |
| Substanz | Alkohol mit Getränke-Buttons und Gramm, weitere Substanzen frei, Menge, Anlass, Craving; **hochsensibel**: Eintragen ohne PIN, Lesen nur mit App-PIN; auf der Startseite nur selbst gewählte, neutrale Bezeichnungen oder Symbole (#26, #89) |
| Craving ohne Konsum | Skala 0-10; **hochsensibel** |
| Mahlzeit | Frühstück, Mittag, Abend, Snack; optional „ohne Appetit gegessen" |
| Bewegung | Spaziergang, Sport, anderes; Dauer |
| Schlafen gegangen | Uhrzeit |
| Aufgewacht | Uhrzeit, wie lange geschlafen (Stunden und Minuten), wie gut (1 bis 5), nachts wach (0, 1, 2, 3+); morgens bis 14 Uhr bietet die Startseite ihn an (#139) |
| Termin | pünktlich, knapp, zu spät |
| Belastung oder Konflikt | Freitext optional |
| Krise | optional, mit kurzer Rückfrage gegen Fehl-Tipps; zeigt danach die Hilfe-Seite. Die Hilfe selbst ist ein eigener Knopf, immer sichtbar, auch ohne Entsperren (Protokoll Abschnitt 13) |
| Arzttermin, Therapiesitzung | Freitext optional |
| Dosis- oder Präparatwechsel | Dosis vorher und nachher, Anlass; Ankerpunkt für den Vergleich vorher/nachher (#23) |

## 4. Tagesabschluss (optional)

Für Angaben, die einmal am Tag genügen. Standardmäßig eingeschaltet, Erinnerung abends.

| Frage | Feldtyp |
|---|---|
| Handyzeit heute | Dauer; Schätzung oder aus der Bildschirmzeit abgeschrieben (automatisch erst in M3) |
| Wie war der Tag insgesamt? | 0-10, höher = besser |
| Gegen Abend gereizt oder erschöpft? | Fünferskala |
| Was war heute besonders? | Freitext |

### Stimmungsverlauf im Tagesabschluss (#105)

Der Tagesabschluss beginnt mit derselben Stimmungsfrage wie der Status jetzt; der Tageswert ist der
letzte Eintrag des Tages, also meist der Abendwert. Dazu einzeln überspringbare Faktoren: Bewegung (ja/nein, Minuten), echte Gespräche (Anzahl), Tageslicht draußen
(ja/nein), Handyzeit. Schlaf wird morgens mit „Aufgewacht“ erfasst, nicht abends (#139); die früheren
Schlaffragen bleiben für vorhandene Einträge definiert. Tagesmarker mit einem Tipp: Belastungsereignis, Therapiesitzung, besonderer
Tag und bis zu drei eigene Marker, deren Namen die Person selbst festlegt (nur verschlüsselt
gespeichert). Im Verlauf erscheint jeder Eintrag als Punkt, das Grundniveau als gleitender
Mittelwert über 7 oder 28 Tage (nur mit mindestens 4 bzw. 14 Tageswerten im Fenster, Lücken
bleiben Lücken), dazu ein beschreibender Vergleich der Tage mit und ohne Faktor samt Fallzahl.
Alkohol kommt erst mit dem hochsensiblen Bereich (M1b).

**Eigener Krisenplan:** ein selbst geschriebener Text („Mein Plan für Tiefs"), über die Hilfe auch
ohne Entsperren lesbar und deshalb unverschlüsselt gespeichert. Auf Wunsch eine selbst gesetzte
Regel „wenn ich N Tage in Folge unter X eintrage, zeig mir meinen Plan"; keine voreingestellten
Schwellen, keine Diagnose. Solange die Regel gilt, lässt sich die Stimmungsfrage nicht ausblenden.

## 5. Wochenrückblick (Sonntag)

| Frage | Feldtyp |
|---|---|
| Bin ich allgemein schnell vergesslich? | Häufigkeit |
| Habe ich Dinge angefangen und nicht beendet? | Häufigkeit |
| Wie gut kam ich mit der Medikation zurecht? | 0-10 |
| Was war diese Woche anders? | Freitext |
| Für das nächste Arztgespräch merken | Freitext |

## 6. Zufällige Abfragen (optional)

Wer nur in schlechten Momenten einen Status festhält, bekommt einen zu düsteren Verlauf. Gegen
diese Verzerrung kann die App auf Wunsch **zwei bis vier Mal am Tag zu zufälligen Zeiten** in
selbst gewählten Zeitfenstern um einen kurzen Status jetzt bitten (in der Forschung als
ambulantes Assessment oder Experience Sampling üblich). Standard: aus. Eine Abfrage gilt 30
Minuten; verpasste und abgelehnte Abfragen werden gespeichert, damit die Auswertung die
Antwortquote kennt. Statuswerte aus Abfragen werden markiert und lassen sich getrennt ansehen.
Zusammen mit allen Erinnerungen gilt eine Obergrenze von acht Benachrichtigungen am Tag.

## 7. Medikament

Präparat und Wirkstoff getrennt, Stärke, Form; Einnahmeplan mit Uhrzeiten und erlaubten Pausen;
Erinnerung (lokal, neutraler Text); optional eine Erinnerung an einen Status einige Stunden nach
der Einnahme; Bedarfsmedikation (#21). Ist der Stand einer Einnahme unklar, zeigt die App „Stand
unbekannt".

## 8. Was die Auswertung damit macht

- Jeder Status ist ein Messpunkt mit Zeitpunkt. Verläufe zeigen die einzelnen Punkte und je
  Lebenstag die Spanne und die Zahl der Status (als nüchterne Angabe, nicht als Leistung);
  Mittelwerte nur mit Hinweis, dass sie von den gewählten Erfassungszeiten abhängen. Tage ohne
  Status bleiben leer.
- **Über den Tag:** Punkte nach Uhrzeit, mit den Einnahmezeiten markiert. Keine geglättete
  Wirkungskurve und keine durchschnittliche Wirkdauer; wo „Einnahme" und „Wirkung lässt nach"
  erfasst sind, zeigt die App je Tag nur den Abstand als Rohwert.
- **Vorher/Nachher** um einen Dosiswechsel nur mit Angabe, wie viele Status in beiden Zeiträumen
  und zu welchen Tageszeiten erfasst wurden.
- Status aus zufälligen Abfragen und selbst ausgelöste Status lassen sich getrennt anzeigen.
- Keine automatischen Zusammenhänge, keine Farbwertung, keine Empfehlungen (#74).
