# Prototyp 81a: Linux-Kern (#100)

Stand 2026-10-14. Ergebnis von #100 als Grundlage für #63 (Formate) und #3 (Protokoll v1.0).
Der Code ist nicht weggeworfen worden: Testvektoren und geprüfter Kern sind `dak-core`
(`core/crates/dak-core`), wie in #100 vorgesehen.

## Umfang

| Teil | Modul | Protokoll |
|---|---|---|
| Hülle um libsodium, geschützter Speicher (`sodium_malloc`), deterministischer Testzufall | `sodium.rs` | 2, AE-12 |
| Strenges deterministisches CBOR, eigener Codec, `ciborium` als Gegenprobe | `cbor.rs` | 3, AE-14 |
| Recovery-Wörter BIP39 (24 Wörter, Prüfsumme, Kurzform mit vier Buchstaben) | `woerter.rs` | 4, AE-35 |
| Zeitmodell: UTC, Versatz, Zone, Lebenstag mit eigenem Tageswechsel | `zeit.rs` | 6.3 |
| Messdefinitionen, Formulare, Status, Ereignisse, Präparate, Einstellungen | `modell.rs` | 5, 6 |
| Gründung, Kontokette (`rotation`, `geraet_ersetzt`, `recovery_neu`), Ersetzungsmarker | `kette.rs` | 4, 10.2 |
| Bund und hochsensibler Bund, Datensätze mit `dk` je Revision, Umhüllungen, Kapseln | `tresor.rs` | 7, 8, 10.1 |
| SQLite-Speicher, nur Chiffretext | `speicher.rs` | AE-15 |
| Sicherungsdatei (`crypto_secretstream`, Manifest-MAC), Aufbewahrung | `sicherung.rs` | 10, AE-18 |
| Einrichten, Entsperren, Schreiben, Entwürfe, Rotation, Wiederherstellung mit Ankerregel und Zweigauswahl | `kern.rs` | 6-12 |
| Feste Fragenvorlage (`templates/standard-v1.json`) | `vorlagen.rs` | #22, #75, #105 |

Uhr, Zeitzone, Tageswechsel und Zufall sind Eingaben (`Uhr`, `Zufall`); die Tests setzen beide fest.

## Negativtests (#100)

`core/crates/dak-core/tests/szenarien.rs` und die Modultests prüfen unter anderem:

- abgeschnittene, vertauschte und veränderte Kette; Generationssprung; Signatur des ersetzten Geräts nach `geraet_ersetzt`
- fremde Gründung mit eigenen Gliedern; Gerät mit fremdem `KK` öffnet den Bund nicht
- Sicherung abgeschnitten, ein Byte verändert, Sicherung eines fremden Kontos, falsche Wörter
- Sicherung eines ersetzten Geräts, das nach der Ersetzung weitergeschrieben hat, wird verworfen; zwei Wiederherstellungen aus derselben Sicherung ergeben zwei Zweige, die App wählt dann nicht selbst
- abgebrochene Rotation lässt den Bestand unverändert; nach der Rotation öffnet der alte `KK` nichts mehr
- gelöschte Einträge bleiben nach der Wiederherstellung gelöscht (Tombstones)
- Wortprobe beim Einrichten; späte Wortprobe gegen den Kettenkopf ohne gespeicherte Wörter (#79)
- Entwurf wird gesperrt versiegelt, nach dem Entsperren angeboten und nach zwölf Stunden verworfen (#94)
- CBOR: nicht kürzeste Form, unbestimmte Längen, doppelte und absteigende Schlüssel, Fließkomma, Tags, ungültiges UTF-8, Restbytes, Tiefe

## Lastprobe

Zwei Jahre mit zehn Status je Tag (7300 Status), Release-Profil, gemessen auf einem
x86-Entwicklungsrechner; die CI misst bei jedem Lauf neu (Job `kern`, Schritt Lastprobe).

| Schritt | Dauer |
|---|---|
| 7300 Status schreiben | 0,92 s |
| Entsperren (alle Datensätze entschlüsseln) | 0,40 s |
| Sicherung erzeugen und zurücklesen | 0,12 s, Datei 7,1 MB |
| Wiederherstellung auf neuem Gerät mit Rotation | 1,17 s |

Folgerung: Für M1a reichen ein Speicher ohne Verlaufsblöcke (AE-15) und das Entschlüsseln des
ganzen Bestands beim Entsperren. Messung auf den Zielgeräten folgt mit #101; liegt das Entsperren
dort über zwei Sekunden, kommen die Verlaufsblöcke je Monat.

## Abweichungen und offene Punkte

- **AE-14:** eigener CBOR-Codec statt `minicbor` (Begründung in `docs/architektur.md`).
- **Zeitzonen:** `jiff` ist nicht eingebunden. Der Kern rechnet mit dem Versatz, den die Plattform
  beim Erfassen liefert; Planer-Zeiten bleiben Ortszeit und werden von der Plattform umgerechnet.
- **Verlaufsblöcke** (AE-15) sind nicht gebaut, siehe Lastprobe.
- **Hochsensibler Bereich:** Format vollständig (`KK_h`, `k_h`, `kapsel_h`, leerer Bund), Schreiben
  und PIN-Weg kommen mit M1b (#26, #89, #60).
- **`recovery_neu`** ist im Kettenformat und in der Prüfung enthalten, der Ablauf in der App folgt mit M1b.
- **SQLite** im gebündelten `libsqlite3-sys` 0.38.2 ist 3.53.2; AE-15 nennt 3.53.4 als Ziel.
