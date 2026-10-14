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
        let lo = q.min ?? 0
        let hi = q.max ?? 10
        let werte: [Int64] = Array(lo...hi)
        return VStack(spacing: 2) {
            HStack(spacing: 3) {
                ForEach(werte, id: \.self) { w in
                    TippFeld(wert: w, bipolar: lo < 0, gewaehlt: a?.zahl == w) { setzen(zahl: w) }
                }
            }
            AnkerZeile(anker: q.anker)
        }
    }

    private func chips(_ werte: [(Int64, String)], gewaehlt: Int64?, tipp: @escaping (Int64) -> Void) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack {
                ForEach(werte, id: \.0) { w in
                    WahlKnopf(text: w.1, an: gewaehlt == w.0) { tipp(w.0) }
                }
            }
        }
    }

    private func umschalten(_ id: UInt16) {
        var neu: [Auswahl] = a?.auswahl ?? []
        if neu.contains(where: { $0.option == id }) {
            neu.removeAll { $0.option == id }
        } else if q.feldart == .einfachauswahl {
            neu = [Auswahl(option: id, staerke: nil)]
        } else {
            neu.append(Auswahl(option: id, staerke: nil))
        }
        antworten[q.messdefId] = neu.isEmpty ? nil : antwort(q, auswahl: neu)
    }

    private func staerkeSetzen(_ option: UInt16, _ w: Int64) {
        let alt: [Auswahl] = a?.auswahl ?? []
        let neu: [Auswahl] = alt.map { x in
            guard x.option == option else { return x }
            return Auswahl(option: x.option, staerke: x.staerke == w ? nil : w)
        }
        antworten[q.messdefId] = antwort(q, auswahl: neu)
    }

    @ViewBuilder private var auswahl: some View {
        let gewaehlt: [Auswahl] = a?.auswahl ?? []
        ScrollView(.horizontal, showsIndicators: false) {
            HStack {
                ForEach(q.optionen, id: \.id) { o in
                    WahlKnopf(text: o.text, an: gewaehlt.contains { $0.option == o.id }) { umschalten(o.id) }
                }
            }
        }
        if q.mitStaerke {
            ForEach(gewaehlt.filter { $0.option != 99 }, id: \.option) { x in
                StaerkeZeile(name: q.optionen.first { $0.id == x.option }?.text ?? "", staerke: x.staerke) { staerkeSetzen(x.option, $0) }
            }
        }
    }
}

/// Ein Feld der Tipp-Reihe; ohne Vorauswahl, die Nulllinie bipolarer Skalen ist umrandet.
struct TippFeld: View {
    let wert: Int64
    let bipolar: Bool
    let gewaehlt: Bool
    let tipp: () -> Void

    var body: some View {
        let text: String = (bipolar && wert > 0) ? "+\(wert)" : "\(wert)"
        let hintergrund: Color = gewaehlt ? Color.accentColor : Color.secondary.opacity(0.15)
        let rand: Color = (bipolar && wert == 0) ? Color.secondary : Color.clear
        return Text(text)
            .font(.caption2)
            .frame(maxWidth: .infinity, minHeight: 40)
            .background(RoundedRectangle(cornerRadius: 7).fill(hintergrund))
            .overlay(RoundedRectangle(cornerRadius: 7).stroke(rand))
            .foregroundStyle(gewaehlt ? Color.white : Color.primary)
            .contentShape(Rectangle())
            .onTapGesture(perform: tipp)
            .accessibilityLabel(text)
            .accessibilityAddTraits(gewaehlt ? [.isSelected, .isButton] : [.isButton])
    }
}

struct AnkerZeile: View {
    let anker: [Anker]
    var body: some View {
        HStack {
            ForEach(Array(anker.enumerated()), id: \.offset) { i, an in
                if i > 0 { Spacer() }
                Text(an.text).font(.caption2).foregroundStyle(.secondary)
            }
        }
    }
}

struct WahlKnopf: View {
    let text: String
    let an: Bool
    let tipp: () -> Void
    var body: some View {
        let farbe: Color = an ? Color.accentColor : Color.secondary
        return Button(text, action: tipp).buttonStyle(.bordered).tint(farbe)
            .accessibilityAddTraits(an ? .isSelected : [])
    }
}

struct StaerkeZeile: View {
    let name: String
    let staerke: Int64?
    let setzen: (Int64) -> Void
    private let stufen: [(Int64, String)] = [(2, "leicht"), (5, "mittel"), (8, "stark")]
    var body: some View {
        HStack {
            Text(name).font(.caption)
            ForEach(stufen, id: \.0) { s in
                WahlKnopf(text: s.1, an: staerke == s.0) { setzen(s.0) }.controlSize(.small)
            }
        }
    }
}
