// SPDX-License-Identifier: MPL-2.0
import SwiftUI

private func antwort(_ f: FrageAnsicht, zahl: Int64? = nil, text: String? = nil, auswahl: [Auswahl] = []) -> AntwortEingabe {
    AntwortEingabe(messdefId: f.messdefId, zustand: .beantwortet, zahl: zahl, text: text, auswahl: auswahl, aktivGesetzt: true)
}

/// Status, Rückblicke und Ereignisse. Keine Vorbelegung, ein zweiter Tipp nimmt eine Antwort zurück (#73).
struct FormularSeite: View {
    @EnvironmentObject var z: AppZustand
    let schluessel: String
    var entwurf: String?
    var praeparat: String?
    var korrektur: String?
    @State private var f: FormularAnsicht?
    @State private var antworten: [String: AntwortEingabe] = [:]
    @State private var blockOffen = false
    @State private var entwurfId: String?
    @State private var gewaehltesPraeparat: String?
    @State private var zeitpunkt: Zeitpunkt?
    @State private var zeit = Date()
    @State private var praeparate: [PraeparatDaten] = []
    @State private var geladen = false

    var body: some View {
        Form {
            if let f {
                Section {
                    DatePicker("Zeitpunkt", selection: $zeit, in: ...Date(), displayedComponents: [.date, .hourAndMinute])
                        .onChange(of: zeit) { _, d in
                            let zone = TimeZone.current
                            zeitpunkt = Zeitpunkt(utcMs: Int64(d.timeIntervalSince1970 * 1000), versatzMin: Int32(zone.secondsFromGMT(for: d) / 60), zone: zone.identifier)
                        }
                }
                if f.schluessel == "krise" { Section { Text("Danach zeigt die App die Hilfe-Seite mit Nummern. Die App benachrichtigt niemanden.") } }
                if f.brauchtPraeparat {
                    Section("Präparat") {
                        if praeparate.isEmpty { Text("Noch kein Präparat angelegt (unter Medikamente).") }
                        Picker("Präparat", selection: $gewaehltesPraeparat) {
                            Text("bitte wählen").tag(String?.none)
                            ForEach(praeparate, id: \.id) { p in Text(p.handelsname + " " + (p.staerke ?? "")).tag(Optional(p.id)) }
                        }
                    }
                }
                ForEach(Array(f.gruppen.enumerated()), id: \.offset) { _, g in
                    if g.zugeklappt {
                        if f.blockAnbieten || blockOffen {
                            Section(header: Text(g.titel + (f.bezugText.map { " (\($0))" } ?? ""))) {
                                if blockOffen { ForEach(g.fragen, id: \.messdefId) { q in FrageView(q: q, antworten: $antworten) } }
                                else { Button("Öffnen") { blockOffen = true } }
                            }
                        }
                    } else {
                        Section(header: Text(g.titel), footer: Text(g.hinweis ?? "")) { ForEach(g.fragen, id: \.messdefId) { q in FrageView(q: q, antworten: $antworten) } }
                    }
                }
                Section(footer: Text("Unbeantwortete Fragen gelten als übersprungen. Nichts wird aus früheren Einträgen übernommen.")) {
                    Button("Speichern") { speichern(f) }.bold()
                    Button("Verwerfen", role: .destructive) { if let e = entwurfId { z.versuch { try z.kern.entwurfVerwerfen(entwurfId: e) } }; z.seite = .start }
                }
            }
        }
        .navigationTitle(f?.name ?? "")
        .onAppear(perform: laden)
        .onChange(of: antworten) { _, _ in entwurfSichern() }
        .onChange(of: blockOffen) { _, _ in entwurfSichern() }
    }

    private func laden() {
        guard !geladen else { return }
        geladen = true
        f = z.versuch { try z.kern.formular(schluessel: schluessel) }
        gewaehltesPraeparat = praeparat
        entwurfId = entwurf
        if f?.brauchtPraeparat == true { praeparate = (z.versuch { try z.kern.praeparate() } ?? []).filter { $0.aktiv } }
        if let e = entwurf, let s = z.versuch({ try z.kern.entwurfLaden(entwurfId: e) }) { s.antworten.forEach { antworten[$0.messdefId] = $0 }; blockOffen = s.blockOffen }
        if let k = korrektur, let d = z.versuch({ try z.kern.eintrag(datensatzId: k) }) {
            d.antworten.forEach { antworten[$0.messdefId] = $0 }; blockOffen = d.blockOffen
            zeitpunkt = d.zeitpunkt; zeit = Date(timeIntervalSince1970: Double(d.zeitpunkt.utcMs) / 1000)
            gewaehltesPraeparat = d.praeparatId
        }
    }

    /// Laufend als Entwurf versiegeln (#94); bleibt auch nach dem Sperren erhalten.
    private func entwurfSichern() {
        guard let f, f.art != .ereignis, korrektur == nil, !antworten.isEmpty else { return }
        entwurfId = z.versuch { try z.kern.entwurfSichern(entwurfId: entwurfId, formularId: f.id, antworten: Array(antworten.values), blockOffen: blockOffen) } ?? entwurfId
    }

    private func speichern(_ f: FormularAnsicht) {
        let ok: String?
        if f.art == .ereignis {
            ok = z.versuch { try z.kern.ereignisErfassen(e: EreignisEingabe(formularId: f.id, zeitpunkt: zeitpunkt, antworten: Array(antworten.values), praeparatId: gewaehltesPraeparat, korrekturVon: korrektur)) }
        } else {
            ok = z.versuch { try z.kern.statusSpeichern(e: StatusEingabe(formularId: f.id, zeitpunkt: zeitpunkt, antworten: Array(antworten.values), blockOffen: blockOffen, ausEntwurf: entwurfId, korrekturVon: korrektur)) }
        }
        if ok != nil { z.geaendert(); z.seite = f.schluessel == "krise" ? .hilfe : .start }
    }
}

struct FrageView: View {
    let q: FrageAnsicht
    @Binding var antworten: [String: AntwortEingabe]
    var a: AntwortEingabe? { antworten[q.messdefId] }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(q.frage)
            if let h = q.hinweis { Text(h).font(.caption).foregroundStyle(.secondary) }
            switch q.feldart {
            case .skala: tippReihe
            case .stufen: chips(q.anker.map { ($0.wert, $0.text) }, gewaehlt: a?.zahl) { setzen(zahl: $0) }
            case .jaNein: chips([(1, "ja"), (0, "nein")], gewaehlt: a?.zahl) { setzen(zahl: $0) }
            case .einfachauswahl, .mehrfachauswahl: auswahl
            case .text:
                TextField("", text: Binding(get: { a?.text ?? "" }, set: { t in if t.isEmpty { antworten[q.messdefId] = nil } else { antworten[q.messdefId] = antwort(q, text: String(t.prefix(20000))) } }), axis: .vertical)
            case .dauer, .zahl, .uhrzeit:
                TextField(q.feldart == .dauer ? "Minuten" : "Zahl", text: Binding(get: { a?.zahl.map(String.init) ?? "" }, set: { t in
                    if let w = Int64(t.filter(\.isNumber).prefix(4)) { antworten[q.messdefId] = antwort(q, zahl: min(max(w, q.min ?? 0), q.max ?? 9999)) } else { antworten[q.messdefId] = nil }
                })).keyboardType(.numberPad)
            }
            if q.kamNichtVor {
                Toggle("kam nicht vor", isOn: Binding(get: { a?.zustand == .kamNichtVor }, set: { an in
                    antworten[q.messdefId] = an ? AntwortEingabe(messdefId: q.messdefId, zustand: .kamNichtVor, zahl: nil, text: nil, auswahl: [], aktivGesetzt: true) : nil
                })).font(.caption)
            }
        }
        .padding(.vertical, 4)
    }

    private func setzen(zahl: Int64) {
        antworten[q.messdefId] = a?.zahl == zahl ? nil : antwort(q, zahl: zahl)
    }

    private var tippReihe: some View {
        let lo = q.min ?? 0, hi = q.max ?? 10
        return VStack(spacing: 2) {
            HStack(spacing: 3) {
                ForEach(Array(lo...hi), id: \.self) { w in
                    let gewaehlt = a?.zahl == w
                    Text(w > 0 && lo < 0 ? "+\(w)" : "\(w)").font(.caption2)
                        .frame(maxWidth: .infinity, minHeight: 40)
                        .background(gewaehlt ? Color.accentColor : Color.secondary.opacity(0.15), in: RoundedRectangle(cornerRadius: 7))
                        .foregroundStyle(gewaehlt ? Color.white : Color.primary)
                        .overlay(RoundedRectangle(cornerRadius: 7).stroke(w == 0 && lo < 0 ? Color.secondary : .clear))
                        .onTapGesture { setzen(zahl: w) }
                        .accessibilityLabel("\(w)").accessibilityAddTraits(gewaehlt ? .isSelected : [])
                }
            }
            HStack { ForEach(Array(q.anker.enumerated()), id: \.offset) { i, an in if i > 0 { Spacer() }; Text(an.text).font(.caption2).foregroundStyle(.secondary) } }
        }
        .buttonStyle(.plain)
    }

    private func chips(_ werte: [(Int64, String)], gewaehlt: Int64?, tipp: @escaping (Int64) -> Void) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack { ForEach(werte, id: \.0) { w in Button(w.1) { tipp(w.0) }.buttonStyle(.bordered).tint(gewaehlt == w.0 ? .accentColor : .secondary) } }
        }
    }

    private var auswahl: some View {
        let gewaehlt = Set((a?.auswahl ?? []).map(\.option))
        return VStack(alignment: .leading) {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack {
                    ForEach(q.optionen, id: \.id) { o in
                        Button(o.text) {
                            var neu = a?.auswahl ?? []
                            if gewaehlt.contains(o.id) { neu.removeAll { $0.option == o.id } } else if q.feldart == .einfachauswahl { neu = [Auswahl(option: o.id, staerke: nil)] } else { neu.append(Auswahl(option: o.id, staerke: nil)) }
                            antworten[q.messdefId] = neu.isEmpty ? nil : antwort(q, auswahl: neu)
                        }.buttonStyle(.bordered).tint(gewaehlt.contains(o.id) ? .accentColor : .secondary)
                    }
                }
            }
            if q.mitStaerke {
                ForEach((a?.auswahl ?? []).filter { $0.option != 99 }, id: \.option) { x in
                    HStack {
                        Text(q.optionen.first { $0.id == x.option }?.text ?? "").font(.caption)
                        ForEach([(Int64(2), "leicht"), (5, "mittel"), (8, "stark")], id: \.0) { s in
                            Button(s.1) {
                                let neu = (a?.auswahl ?? []).map { $0.option == x.option ? Auswahl(option: $0.option, staerke: $0.staerke == s.0 ? nil : s.0) : $0 }
                                antworten[q.messdefId] = antwort(q, auswahl: neu)
                            }.buttonStyle(.bordered).controlSize(.small).tint(x.staerke == s.0 ? .accentColor : .secondary)
                        }
                    }
                }
            }
        }
    }
}
