// SPDX-License-Identifier: MPL-2.0
import SwiftUI
import UniformTypeIdentifiers

/// Wort eingeben: ohne Autokorrektur, Auswahl aus der Wortliste (#69).
struct WortFeld: View {
    @EnvironmentObject var z: AppZustand
    let nr: Int
    @Binding var wert: String
    var body: some View {
        VStack(alignment: .leading) {
            TextField("Wort \(nr)", text: $wert)
                .textInputAutocapitalization(.never).autocorrectionDisabled().textContentType(.oneTimeCode)
                .onChange(of: wert) { _, n in wert = n.lowercased().trimmingCharacters(in: .whitespaces) }
            let v = wert.count >= 2 ? z.kern.wortVorschlaege(anfang: wert) : []
            if !v.isEmpty && v != [wert] {
                ScrollView(.horizontal) { HStack { ForEach(v, id: \.self) { w in Button(w) { wert = w }.buttonStyle(.bordered) } } }
            }
        }
    }
}

struct EinrichtungSeite: View {
    @EnvironmentObject var z: AppZustand
    @State private var schritt = 0
    @State private var tageswechsel = 4
    @State private var einrichtung: Einrichtung?
    @State private var probe = ["", "", ""]
    @State private var ordnerWahl = false

    var body: some View {
        NavigationStack {
            Form {
                switch schritt {
                case 0:
                    Section("Willkommen bei Days & Keys") {
                        Text("Hier hältst du fest, wie es dir geht, so oft du willst: mehrmals am Tag, jederzeit. Dazu Medikamente, Ereignisse und auf Wunsch einen Tagesabschluss.")
                        Text("Alles bleibt verschlüsselt auf diesem Gerät. Die App enthält keinen Netzwerkcode und schickt nichts an einen Server.")
                        Text("Wichtig: Wer den Code dieses Geräts kennt, kann die Einträge lesen.").bold()
                        Button("Neu einrichten") { schritt = 1 }
                        Button("Aus einer Sicherung wiederherstellen") { z.seite = .wiederherstellung }
                        Button("Hilfe in einer Krise") { z.seite = .hilfe }
                    }
                case 1:
                    Section(header: Text("Wann beginnt dein Tag?"), footer: Text("Einträge vor dieser Uhrzeit zählen zum Vortag.")) {
                        Picker("Tageswechsel", selection: $tageswechsel) { ForEach([0, 2, 3, 4, 5, 6], id: \.self) { Text("\($0):00 Uhr").tag($0) } }
                        Button("Weiter") {
                            einrichtung = z.versuch { try z.kern.einrichtungBeginnen(tageswechselMin: Int32(tageswechsel * 60)) }
                            if einrichtung != nil { schritt = 2 }
                        }
                    }
                case 2:
                    Section(header: Text("Deine 24 Recovery-Wörter"), footer: Text("Schreib sie auf Papier und bewahre sie sicher auf. Nur mit ihnen lassen sich deine Einträge auf einem neuen Gerät wiederherstellen. Kein Bildschirmfoto, keine Notiz-App.")) {
                        ForEach(Array((einrichtung?.woerter ?? []).enumerated()), id: \.offset) { i, w in Text("\(i + 1). \(w)").font(.body.monospaced()) }
                        Button("Ich habe sie aufgeschrieben") { schritt = 3 }
                    }
                    .privacySensitive()
                case 3:
                    Section("Kurze Probe") {
                        ForEach(0..<3, id: \.self) { i in WortFeld(nr: Int(einrichtung?.probe[i] ?? 0) + 1, wert: $probe[i]) }
                        Button("Prüfen") {
                            if z.versuch({ try z.kern.einrichtungProbePruefen(probe: probe) }) == true { schritt = 4 }
                            else { z.hinweis = "Die Wörter passen nicht. Schau noch einmal auf deinen Zettel." }
                        }
                        Button("Wörter noch einmal zeigen") { schritt = 2 }
                    }
                case 4:
                    Section(header: Text("Mit dem Gerät schützen"), footer: Text("Days & Keys öffnet sich künftig mit Face ID oder dem Geräte-Code.")) {
                        Button("Schützen und einrichten") {
                            guard let e = einrichtung else { return }
                            do {
                                try Schluesselbund.speichern(generation: e.schluessel.generation, kk: e.schluessel.kk, kkH: e.schluessel.kkH)
                                try z.kern.einrichtungAbschliessen(probe: probe, kkUmhuellung: nil, kkHUmhuellung: nil)
                                einrichtung = nil
                                z.nachEntsperren(navigieren: false)
                                schritt = 5
                            } catch { z.hinweis = meldung(error) }
                        }
                    }
                case 5:
                    Section(header: Text("Automatische Sicherung"), footer: Text("Wähle einen Ordner in der Dateien-App, zum Beispiel in iCloud Drive. Die App schreibt dort selbst verschlüsselte Sicherungen.")) {
                        Button("Ordner wählen") { ordnerWahl = true }
                        Button("Später") { schritt = 6 }
                    }
                case 6:
                    Section(header: Text("Erinnerungen"), footer: Text("Neutrale Texte ohne Medikamentennamen. Die App ersetzt keinen Medikamentenwecker.")) {
                        Button("Benachrichtigungen erlauben") { Task { _ = await Planer.erlaubnisAnfragen(); schritt = 7 } }
                        Button("Ohne Erinnerungen") { schritt = 7 }
                    }
                default:
                    Section("Fertig") { Button("Los geht's") { z.planen(); z.seite = .start } }
                }
            }
            .navigationTitle("Einrichtung")
            .fileImporter(isPresented: $ordnerWahl, allowedContentTypes: [.folder]) { r in
                if case let .success(url) = r { z.versuch { try Sicherungsordner.uebernehmen(url) }; z.sichern { z.hinweis = $0 } }
                schritt = 6
            }
        }
    }
}

struct WiederherstellungSeite: View {
    @EnvironmentObject var z: AppZustand
    @State private var woerter = Array(repeating: "", count: 24)
    @State private var ordnerWahl = false
    @State private var ordner: URL?
    @State private var kandidaten: [Kandidat]?

    var body: some View {
        NavigationStack {
            Form {
                if let k = kandidaten {
                    Section(header: Text("Gefundene Sicherungen")) {
                        if k.isEmpty { Text("In diesem Ordner liegt keine Sicherung.") }
                        ForEach(k.sorted { $0.erstelltMs > $1.erstelltMs }, id: \.index) { c in
                            if c.gueltig {
                                Button("Nr. \(c.sicherungNr) vom \(datum(c.erstelltMs))" + (c.empfohlen ? " (neueste)" : "")) { herstellen(c.index) }
                            } else { Text("\(c.name): \(c.grund ?? "")").foregroundStyle(.secondary) }
                        }
                        Button("Anderen Ordner wählen") { kandidaten = nil }
                    }
                } else {
                    Section(header: Text("Wiederherstellen"), footer: Text("Gib deine 24 Wörter ein und wähle dann den Ordner mit den Sicherungen.")) {
                        ForEach(0..<24, id: \.self) { i in WortFeld(nr: i + 1, wert: $woerter[i]) }
                        Button("Ordner mit Sicherungen wählen") { if z.versuch({ try z.kern.wiederherstellungBeginnen(woerter: woerter) }) != nil { ordnerWahl = true } }
                        Button("Zurück") { z.seite = .einrichtung }
                    }
                    .privacySensitive()
                }
            }
            .navigationTitle("Wiederherstellen")
            .fileImporter(isPresented: $ordnerWahl, allowedContentTypes: [.folder]) { r in
                guard case let .success(url) = r else { return }
                ordner = url
                z.versuch { try Sicherungsordner.uebernehmen(url) }
                if let (d, m) = z.versuch({ try Sicherungsordner.sicherungen(url) }) { kandidaten = z.versuch { try z.kern.kandidaten(dateien: d, marker: m) } }
            }
        }
    }

    private func herstellen(_ index: UInt32) {
        guard let u = ordner else { return }
        do {
            let (d, m) = try Sicherungsordner.sicherungen(u)
            let w = try z.kern.wiederherstellen(dateien: d, marker: m, index: index)
            try Schluesselbund.speichern(generation: w.schluessel.generation, kk: w.schluessel.kk, kkH: w.schluessel.kkH)
            try z.kern.vorbereitetesAbschliessen(kkUmhuellung: nil, kkHUmhuellung: nil)
            try Sicherungsordner.schreiben(u, name: w.markerName, daten: w.marker)
            Schluesselbund.aufraeumen(aktuell: w.schluessel.generation)
            z.nachEntsperren()
            z.hinweis = "Wiederhergestellt. Entziehe dem alten Gerät den Zugang zum Sicherungsordner, falls es noch existiert."
        } catch { z.hinweis = meldung(error) }
    }
}

func datum(_ ms: Int64) -> String {
    let f = DateFormatter(); f.dateFormat = "dd.MM.yyyy, HH:mm"
    return f.string(from: Date(timeIntervalSince1970: Double(ms) / 1000))
}

func vorText(_ ms: Int64) -> String {
    let min = Int((Date().timeIntervalSince1970 * 1000 - Double(ms)) / 60000)
    if min < 2 { return "gerade eben" }
    if min < 60 { return "vor \(min) Minuten" }
    if min < 48 * 60 { return "vor \(min / 60) Stunden" }
    return "vor \(min / 1440) Tagen"
}

func uhrzeit(_ min: Int32) -> String { String(format: "%02d:%02d", min / 60, min % 60) }
