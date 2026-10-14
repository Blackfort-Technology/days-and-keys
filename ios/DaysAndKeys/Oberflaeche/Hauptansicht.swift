// SPDX-License-Identifier: MPL-2.0
import SwiftUI

struct Hauptansicht: View {
    @EnvironmentObject var z: AppZustand

    var body: some View {
        ZStack {
            switch z.seite {
            case .sperre: SperrSeite()
            case .einrichtung: EinrichtungSeite()
            case .wiederherstellung: WiederherstellungSeite()
            case .hilfe where !z.entsperrt: NavigationStack { HilfeSeite() }
            default: Reiter()
            }
            if z.arbeitet { ProgressView().padding().background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12)) }
        }
        .alert(z.hinweis ?? "", isPresented: Binding(get: { z.hinweis != nil }, set: { if !$0 { z.hinweis = nil } })) { Button("OK", role: .cancel) {} }
    }
}

struct Reiter: View {
    @EnvironmentObject var z: AppZustand
    var body: some View {
        TabView(selection: Binding(get: { tab(z.seite) }, set: { z.seite = $0 })) {
            NavigationStack { inhalt(.start) }.tabItem { Label("Heute", systemImage: "sun.max") }.tag(Seite.start)
            NavigationStack { VerlaufSeite() }.tabItem { Label("Verlauf", systemImage: "chart.xyaxis.line") }.tag(Seite.verlauf)
            NavigationStack { EintraegeSeite() }.tabItem { Label("Einträge", systemImage: "list.bullet") }.tag(Seite.eintraege)
            NavigationStack { inhalt(.medikamente) }.tabItem { Label("Medikamente", systemImage: "pills") }.tag(Seite.medikamente)
            NavigationStack { EinstellungenSeite() }.tabItem { Label("Mehr", systemImage: "gearshape") }.tag(Seite.einstellungen)
        }
    }

    private func tab(_ s: Seite) -> Seite {
        switch s {
        case .verlauf, .eintraege, .einstellungen: return s
        case .medikamente, .medikament: return .medikamente
        default: return .start
        }
    }

    @ViewBuilder private func inhalt(_ basis: Seite) -> some View {
        Group {
            switch z.seite {
            case let .formular(s, e, p, k) where basis == .start: FormularSeite(schluessel: s, entwurf: e, praeparat: p, korrektur: k)
            case .hilfe where basis == .start: HilfeSeite()
            case let .medikament(id) where basis == .medikamente: MedikamentSeite(id: id)
            default: if basis == .start { StartSeite() } else { MedikamenteSeite() }
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarLeading) { Button("Hilfe") { z.seite = .hilfe } }
            ToolbarItem(placement: .topBarTrailing) { Button("Sperren") { z.sperren() } }
        }
    }
}

struct SperrSeite: View {
    @EnvironmentObject var z: AppZustand
    var body: some View {
        VStack(spacing: 18) {
            Spacer()
            Text("Days & Keys").font(.largeTitle)
            Text("Gesperrt").foregroundStyle(.secondary)
            Button("Entsperren") { z.entsperren() }.buttonStyle(.borderedProminent).controlSize(.large)
            Button("Hilfe in einer Krise") { z.seite = .hilfe }.buttonStyle(.bordered)
            Text("Zum Entsperren genügt Face ID oder der Geräte-Code.").font(.footnote).foregroundStyle(.secondary)
            Spacer()
        }
        .padding()
        .onAppear { if z.ziel != nil { z.entsperren() } }
    }
}

struct HilfeSeite: View {
    @EnvironmentObject var z: AppZustand
    @Environment(\.openURL) private var oeffnen
    var body: some View {
        let h = z.versuch { try z.kern.hilfe() }
        List {
            Section(footer: Text(h?.hinweis ?? "")) {
                ForEach(h?.nummern ?? [], id: \.nummer) { n in
                    Button { oeffnen(URL(string: "tel:" + n.nummer.replacingOccurrences(of: " ", with: ""))!) } label: {
                        VStack(alignment: .leading) { Text("\(n.name): \(n.nummer)"); Text(n.hinweis).font(.caption).foregroundStyle(.secondary) }
                    }
                }
            }
            if let p = h?.krisenplan { Section("Mein Plan für Tiefs") { Text(p) } }
            if let v = h?.vertrauenspersonen, !v.isEmpty {
                Section("Meine Vertrauenspersonen") {
                    ForEach(v, id: \.telefon) { p in Button("\(p.name): \(p.telefon)") { oeffnen(URL(string: "tel:" + p.telefon.filter { $0.isNumber || $0 == "+" })!) } }
                }
            }
            if !z.entsperrt { Button("Zurück") { z.seite = (try? z.kern.lage()).map { if case .nichtEingerichtet = $0 { return Seite.einrichtung } else { return Seite.sperre } } ?? .sperre } }
        }
        .navigationTitle("Hilfe")
    }
}
