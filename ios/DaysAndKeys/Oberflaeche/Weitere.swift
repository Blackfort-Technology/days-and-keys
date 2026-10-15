// SPDX-License-Identifier: MPL-2.0
import Charts
import SwiftUI
import UniformTypeIdentifiers

/// Verlauf (#43, #74, #105): jeder Eintrag als Punkt, Grundniveau als gleitender Mittelwert, Lücken bleiben Lücken.
struct VerlaufSeite: View {
    @EnvironmentObject var z: AppZustand
    @State private var frage = "stimmung"
    @State private var tage: Int64 = 28
    @State private var mittel = 7

    var body: some View {
        let heute = z.versuch { try z.kern.heute() } ?? 0
        let fragen = ["status_jetzt", "tagesabschluss"].flatMap { s in (try? z.kern.formular(schluessel: s))?.gruppen.flatMap(\.fragen) ?? [] }
            .filter { [.skala, .stufen, .zahl, .dauer].contains($0.feldart) }
        let v = z.versuch { try z.kern.verlauf(schluessel: frage, vonTag: heute - tage + 1, bisTag: heute) }
        List {
            Picker("Frage", selection: $frage) { ForEach(Array(Set(fragen.map(\.schluessel))).sorted(), id: \.self) { s in Text(fragen.first { $0.schluessel == s }?.frage ?? s).tag(s) } }
            Picker("Zeitraum", selection: $tage) { Text("4 Wochen").tag(Int64(28)); Text("3 Monate").tag(Int64(91)); Text("1 Jahr").tag(Int64(365)) }.pickerStyle(.segmented)
            Picker("Mittel", selection: $mittel) { Text("7 Tage").tag(7); Text("28 Tage").tag(28) }.pickerStyle(.segmented)
            if let v {
                Section(footer: Text(v.hinweis)) {
                    Chart {
                        if v.richtung == .bipolar { RuleMark(y: .value("null", 0)).foregroundStyle(.secondary) }
                        ForEach(Array(v.punkte.enumerated()), id: \.offset) { _, p in
                            PointMark(x: .value("Tag", Double(p.lebenstag) + Double(p.uhrzeitMin) / 1440), y: .value("Wert", p.wert)).symbolSize(30)
                        }
                        ForEach((mittel == 7 ? v.gleitend7 : v.gleitend28).filter { $0.wert != nil }, id: \.lebenstag) { m in
                            LineMark(x: .value("Tag", Double(m.lebenstag) + 0.5), y: .value("Mittel", m.wert!)).foregroundStyle(.orange)
                        }
                        ForEach(Array(v.marker.enumerated()), id: \.offset) { _, m in RuleMark(x: .value("Marker", Double(m.lebenstag) + 0.5)).foregroundStyle(.gray.opacity(0.4)) }
                    }
                    .chartYScale(domain: Double(v.min ?? 0)...Double(v.max ?? 10))
                    .chartXAxis(.hidden)
                    .frame(height: 220)
                    .accessibilityLabel("Verlauf mit \(v.punkte.count) Einträgen an \(v.tageMitWert) von \(v.tageGesamt) Tagen")
                    if !v.marker.isEmpty { Text("Marker: " + Dictionary(grouping: v.marker, by: \.name).map { "\($0.key) (\($0.value.count))" }.joined(separator: ", ")).font(.caption) }
                }
                if frage == "stimmung" {
                    Section(header: Text("Grundniveau mit und ohne"), footer: Text("Rein beschreibend, mit Fallzahl. Kein Beleg für eine Ursache.")) {
                        ForEach([("tag_bewegung", "Bewegung"), ("tag_tageslicht", "Tageslicht draußen")], id: \.0) { s, n in
                            if let f = z.versuch({ try z.kern.faktorvergleich(faktorSchluessel: s, vonTag: heute - tage + 1, bisTag: heute) }) {
                                Text("\(n): an \(f.tageMit) Tagen mit \(f.mittelMit.map { String(format: "%+.1f", $0) } ?? "-"), an \(f.tageOhne) Tagen ohne \(f.mittelOhne.map { String(format: "%+.1f", $0) } ?? "-")")
                            }
                        }
                    }
                }
            }
        }
        .navigationTitle("Verlauf")
    }
}

struct EintraegeSeite: View {
    @EnvironmentObject var z: AppZustand
    @State private var stand = 0
    @State private var loeschen: EintragZeile?
    var body: some View {
        let heute = z.versuch { try z.kern.heute() } ?? 0
        let liste = (z.versuch { try z.kern.eintraege(vonTag: heute - 60, bisTag: heute) } ?? []).filter { _ in stand >= 0 }
        List {
            if liste.isEmpty { Text("Noch keine Einträge.") }
            ForEach(Array(Dictionary(grouping: liste, by: \.datum).sorted { $0.value[0].utcMs > $1.value[0].utcMs }), id: \.key) { tag, eintraege in
                Section(tag) {
                    ForEach(eintraege, id: \.datensatzId) { e in
                        VStack(alignment: .leading) {
                            Text("\(e.uhrzeit)  \(e.name)")
                            if !e.zusammenfassung.isEmpty { Text(e.zusammenfassung).font(.caption).foregroundStyle(.secondary) }
                        }
                        .swipeActions {
                            Button("Löschen", role: .destructive) { loeschen = e }
                            Button("Korrigieren") {
                                let schl = e.istEreignis ? ((try? z.kern.startseite())?.ereignisse.first { $0.formularId == e.formularId }?.schluessel ?? (e.name == "Tablette genommen" ? "einnahme" : nil)) : formularSchluessel(z, e.formularId)
                                if let schl { z.seite = .formular(schl, korrektur: e.datensatzId) }
                            }
                        }
                    }
                }
            }
        }
        .navigationTitle("Einträge")
        .confirmationDialog("Wirklich löschen? In älteren Sicherungen steht der Eintrag noch, bis sie nach höchstens 30 Tagen aus der Aufbewahrung fallen.", isPresented: Binding(get: { loeschen != nil }, set: { if !$0 { loeschen = nil } }), titleVisibility: .visible) {
            Button("Löschen", role: .destructive) { if let e = loeschen { z.versuch { try z.kern.eintragLoeschen(datensatzId: e.datensatzId) }; z.geaendert(); stand += 1 }; loeschen = nil }
        }
    }
}

struct MedikamenteSeite: View {
    @EnvironmentObject var z: AppZustand
    var body: some View {
        let liste = z.versuch { try z.kern.praeparate() } ?? []
        List {
            Section(footer: Text("Ein Dosis- oder Präparatwechsel ist zusätzlich ein Ereignis, damit er im Verlauf sichtbar ist.")) {
                Button("Präparat hinzufügen") { z.seite = .medikament(nil) }
            }
            ForEach(liste, id: \.id) { p in
                Section(p.handelsname + " " + (p.staerke ?? "")) {
                    Text((p.aktiv ? "" : "pausiert · ") + (p.bedarf ? "bei Bedarf · " : "") + (p.plan.isEmpty ? "ohne festen Plan" : p.plan.map { "\(uhrzeit($0.uhrzeitMin)) \($0.dosis)" }.joined(separator: ", "))).font(.caption)
                    Button("Bearbeiten") { z.seite = .medikament(p.id) }
                    Button("Wechsel festhalten") { z.seite = .formular("dosiswechsel", praeparat: p.id) }
                }
            }
        }
        .navigationTitle("Medikamente")
    }
}

struct MedikamentSeite: View {
    @EnvironmentObject var z: AppZustand
    let id: String?
    @State private var p = PraeparatDaten(id: "", handelsname: "", wirkstoff: nil, staerke: nil, form: nil, plan: [], bedarf: false, wirkungSpuerbar: false, aktiv: true, erinnerung: true, statusNachMin: nil)
    @State private var geladen = false
    var body: some View {
        Form {
            Section {
                TextField("Name (Handelsname)", text: $p.handelsname)
                TextField("Wirkstoff (optional)", text: Binding(get: { p.wirkstoff ?? "" }, set: { p.wirkstoff = $0 }))
                TextField("Stärke", text: Binding(get: { p.staerke ?? "" }, set: { p.staerke = $0 }))
                TextField("Form", text: Binding(get: { p.form ?? "" }, set: { p.form = $0 }))
                Toggle("Bei Bedarf statt nach Plan", isOn: $p.bedarf)
                Toggle("Wirkung im Tagesverlauf spürbar", isOn: $p.wirkungSpuerbar)
                Toggle("Aktiv", isOn: $p.aktiv)
                Toggle("Erinnern (neutraler Text ohne Namen)", isOn: $p.erinnerung)
            }
            Section("Einnahmeplan") {
                ForEach(p.plan.indices, id: \.self) { i in
                    HStack {
                        DatePicker("", selection: Binding(get: { Calendar.current.date(bySettingHour: Int(p.plan[i].uhrzeitMin) / 60, minute: Int(p.plan[i].uhrzeitMin) % 60, second: 0, of: Date())! },
                                                          set: { d in let c = Calendar.current.dateComponents([.hour, .minute], from: d); p.plan[i].uhrzeitMin = Int32(c.hour! * 60 + c.minute!) }), displayedComponents: .hourAndMinute).labelsHidden()
                        TextField("Dosis", text: $p.plan[i].dosis)
                    }
                }
                .onDelete { p.plan.remove(atOffsets: $0) }
                Button("Uhrzeit hinzufügen") { p.plan.append(Planzeit(uhrzeitMin: 8 * 60, dosis: "1", wochentage: 0x7f)) }
                TextField("Status-Erinnerung nach Einnahme (Minuten, optional)", text: Binding(get: { p.statusNachMin.map(String.init) ?? "" }, set: { p.statusNachMin = Int64($0) })).keyboardType(.numberPad)
            }
            Button("Speichern") { if z.versuch({ try z.kern.praeparatSpeichern(p: p) }) != nil { z.geaendert(); z.seite = .medikamente } }
        }
        .navigationTitle(id == nil ? "Neues Präparat" : "Präparat")
        .onAppear { if !geladen, let id, let alt = (try? z.kern.praeparate())?.first(where: { $0.id == id }) { p = alt }; geladen = true }
    }
}

struct CsvDatei: FileDocument {
    static var readableContentTypes: [UTType] { [.commaSeparatedText] }
    var text: String
    init(text: String) { self.text = text }
    init(configuration: ReadConfiguration) throws { text = "" }
    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper { FileWrapper(regularFileWithContents: Data(text.utf8)) }
}

struct EinstellungenSeite: View {
    @EnvironmentObject var z: AppZustand
    @State private var e: EinstellungenDaten?
    @State private var plan = ""
    @State private var ordnerWahl = false
    @State private var export: CsvDatei?
    @State private var probe = Array(repeating: "", count: 24)
    @State private var probeOffen = false

    var body: some View {
        Form {
            if e != nil {
                Section("Tag und Sperre") {
                    Picker("Der Tag beginnt um", selection: Binding(get: { e!.tageswechselMin }, set: { e!.tageswechselMin = $0 })) { ForEach([0, 2, 3, 4, 5, 6], id: \.self) { Text("\($0):00").tag(Int32($0 * 60)) } }
                    Picker("Sperren nach", selection: Binding(get: { e!.sperrzeitS }, set: { e!.sperrzeitS = $0 })) { Text("30 s").tag(UInt32(30)); Text("1 min").tag(UInt32(60)); Text("5 min").tag(UInt32(300)) }
                }
                Section(header: Text("Erinnerungen")) {
                    ForEach([(UInt8(2), "Tagesabschluss", Int32(21 * 60), UInt8(0x7f)), (UInt8(3), "Wochenrückblick (Sonntag)", Int32(18 * 60), UInt8(0x40))], id: \.0) { art, name, std, tage in
                        Toggle(name, isOn: Binding(get: { e!.erinnerungen.first { $0.art == art }?.aktiv == true }, set: { an in
                            let alt = e!.erinnerungen.first { $0.art == art }
                            e!.erinnerungen.removeAll { $0.art == art }
                            e!.erinnerungen.append(ErinnerungDaten(art: art, uhrzeitMin: alt?.uhrzeitMin ?? std, wochentage: alt?.wochentage ?? tage, aktiv: an))
                        }))
                    }
                    ForEach([9, 12, 15, 18], id: \.self) { h in
                        Toggle("Status-Erinnerung \(h):00", isOn: Binding(get: { e!.erinnerungen.contains { $0.art == 1 && $0.uhrzeitMin == Int32(h * 60) } }, set: { an in
                            e!.erinnerungen.removeAll { $0.art == 1 && $0.uhrzeitMin == Int32(h * 60) }
                            if an { e!.erinnerungen.append(ErinnerungDaten(art: 1, uhrzeitMin: Int32(h * 60), wochentage: 0x7f, aktiv: true)) }
                        }))
                    }
                }
                Section(header: Text("Eigene Tagesmarker"), footer: Text("Bis zu drei Marker mit selbst gewählten Namen. Die Namen liegen nur verschlüsselt auf dem Gerät.")) {
                    ForEach(0..<3, id: \.self) { i in
                        TextField("Marker \(i + 1)", text: Binding(get: { i < e!.eigeneMarker.count ? e!.eigeneMarker[i] : "" }, set: { t in
                            var m = e!.eigeneMarker; while m.count <= i { m.append("") }; m[i] = String(t.prefix(40)); e!.eigeneMarker = m
                        }))
                    }
                }
                Section(header: Text("Mein Plan für Tiefs"), footer: Text("Über die Hilfe auch ohne Entsperren lesbar und dafür unverschlüsselt auf dem Gerät. In die Sicherung kommt er verschlüsselt mit.")) {
                    TextField("Frühwarnzeichen, was hilft, wen ich anrufe", text: $plan, axis: .vertical).lineLimit(3...8)
                    Button("Plan speichern") { if z.versuch({ try z.kern.krisenplanSetzen(text: plan.isEmpty ? nil : plan) }) != nil { z.hinweis = "Plan gespeichert" } }
                    Picker("Plan zeigen nach", selection: Binding(get: { e!.krisenregelTage ?? 0 }, set: { t in e!.krisenregelTage = t == 0 ? nil : t; e!.krisenregelUnter = t == 0 ? nil : (e!.krisenregelUnter ?? -2) })) {
                        Text("aus").tag(UInt32(0)); Text("3 Tagen").tag(UInt32(3)); Text("5 Tagen").tag(UInt32(5)); Text("7 Tagen").tag(UInt32(7))
                    }
                    if e!.krisenregelTage != nil {
                        Picker("mit Stimmung unter", selection: Binding(get: { e!.krisenregelUnter ?? -2 }, set: { e!.krisenregelUnter = $0 })) { ForEach([-3, -2, -1, 0], id: \.self) { Text("\($0)").tag(Int64($0)) } }
                    }
                }
                Section(header: Text("Vertrauenspersonen"), footer: Text("Erscheinen in der Hilfe, nur entsperrt. Die App benachrichtigt niemanden.")) {
                    ForEach(e!.vertrauenspersonen.indices, id: \.self) { i in
                        HStack { TextField("Name", text: Binding(get: { e!.vertrauenspersonen[i].name }, set: { e!.vertrauenspersonen[i].name = $0 }))
                            TextField("Telefon", text: Binding(get: { e!.vertrauenspersonen[i].telefon }, set: { e!.vertrauenspersonen[i].telefon = $0 })).keyboardType(.phonePad) }
                    }
                    if e!.vertrauenspersonen.count < 10 { Button("Person hinzufügen") { e!.vertrauenspersonen.append(Vertrauensperson(name: "", telefon: "")) } }
                }
                Button("Einstellungen speichern") {
                    var n = e!
                    n.eigeneMarker = n.eigeneMarker.filter { !$0.trimmingCharacters(in: .whitespaces).isEmpty }
                    n.vertrauenspersonen = n.vertrauenspersonen.filter { !$0.name.isEmpty && !$0.telefon.isEmpty }
                    if z.versuch({ try z.kern.einstellungenSpeichern(e: n) }) != nil { e = n; z.geaendert(); z.hinweis = "Gespeichert" }
                }
            }
            Section(header: Text("Sicherung"), footer: Text(Sicherungsordner.gewaehlt ? "Ordner gewählt." + (z.start?.letzteSicherungMs.map { " Letzte Sicherung \(vorText($0))." } ?? "") : "Noch kein Ordner gewählt.")) {
                Button("Ordner wählen") { ordnerWahl = true }
                Button("Jetzt sichern") { z.sichern { z.hinweis = $0 } }
            }
            Section(header: Text("Alles exportieren"), footer: Text("Alle Einträge als Tabelle (CSV) im Klartext.")) {
                Button("Export erstellen") {
                    Task {
                        do {
                            let b = try z.kern.bestaetigungAnfordern(aktion: .exportCsv)
                            try await Schluesselbund.bestaetigen(b.titel)
                            export = CsvDatei(text: try z.kern.exportCsv(digest: b.digest))
                        } catch { z.hinweis = meldung(error) }
                    }
                }
            }
            Section(header: Text("Recovery-Wörter prüfen"), footer: Text("Die Wörter werden nicht gespeichert.")) {
                if probeOffen {
                    ForEach(0..<24, id: \.self) { i in WortFeld(nr: i + 1, wert: $probe[i]) }
                    Button("Prüfen") { z.hinweis = z.versuch({ try z.kern.woerterPruefen(woerter: probe) }) == true ? "Die Wörter stimmen." : "Die Wörter passen nicht."; probe = Array(repeating: "", count: 24); probeOffen = false }
                } else { Button("Prüfen") { probeOffen = true } }
            }
            .privacySensitive()
            Section(header: Text("Schlüssel erneuern"), footer: Text("Bei Verdacht, dass jemand Zugriff hatte.")) {
                Button("Schlüssel erneuern") {
                    Task {
                        do {
                            let b = try z.kern.bestaetigungAnfordern(aktion: .rotationAufWunsch)
                            try await Schluesselbund.bestaetigen(b.titel)
                            let s = try z.kern.rotationBeginnen(digest: b.digest)
                            try Schluesselbund.speichern(generation: s.generation, kk: s.kk, kkH: s.kkH)
                            try z.kern.vorbereitetesAbschliessen(kkUmhuellung: nil, kkHUmhuellung: nil)
                            Schluesselbund.aufraeumen(aktuell: s.generation)
                            z.geaendert(); z.sichern { z.hinweis = "Schlüssel erneuert. \($0)" }
                        } catch { z.kern.vorbereitetesVerwerfen(); z.hinweis = meldung(error) }
                    }
                }
            }
            Section("Über") {
                Text("Days & Keys \(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "") · Open Source (MPL-2.0)")
                Text("Keine Diagnosen, keine Empfehlungen, keine automatischen Zusammenhänge. Die App ersetzt keine ärztliche Beratung.").font(.caption)
            }
        }
        .navigationTitle("Mehr")
        .onAppear { if e == nil { e = z.versuch { try z.kern.einstellungen() }; plan = z.versuch { try z.kern.hilfe() }?.krisenplan ?? "" } }
        .fileImporter(isPresented: $ordnerWahl, allowedContentTypes: [.folder]) { r in
            if case let .success(url) = r { z.versuch { try Sicherungsordner.uebernehmen(url) }; z.sichern { z.hinweis = $0 } }
        }
        .fileExporter(isPresented: Binding(get: { export != nil }, set: { if !$0 { export = nil } }), document: export, contentType: .commaSeparatedText, defaultFilename: "days-and-keys-export") { _ in export = nil }
    }
}
