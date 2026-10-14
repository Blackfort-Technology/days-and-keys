// SPDX-License-Identifier: MPL-2.0
import SwiftUI

struct StartSeite: View {
    @EnvironmentObject var z: AppZustand
    @State private var bedarf: Knopf?

    var body: some View {
        List {
            if let s = z.start {
                Section(header: Text(s.heute), footer: Text(statusText(s))) {
                    Button { z.seite = .formular("status_jetzt") } label: { Text("Status jetzt festhalten").font(.headline).frame(maxWidth: .infinity, minHeight: 44) }
                        .buttonStyle(.borderedProminent)
                    if s.tagesabschlussHeute { Button("Tagesabschluss") { z.seite = .formular("tagesabschluss") } }
                    if s.wochenrueckblickHeute { Button("Wochenrückblick") { z.seite = .formular("wochenrueckblick") } }
                }
                if s.krisenplanHinweis && !z.krisenhinweisGesehen {
                    Section(footer: Text("Du hast eingestellt, dass die App ihn dir zeigt, wenn deine Tageswerte eine Weile niedrig sind.")) {
                        Button("Mein Plan für Tiefs und Hilfe") { z.seite = .hilfe }
                        Button("Ausblenden") { z.krisenhinweisGesehen = true }
                    }
                }
                ForEach(s.entwuerfe, id: \.id) { e in
                    Section("Angefangen: \(e.formularName)") {
                        Button("Weiter ausfüllen (\(vorText(e.erstelltMs)))") {
                            if let schl = formularSchluessel(z, e.formularId) { z.seite = .formular(schl, entwurf: e.id) }
                        }
                        Button("Verwerfen", role: .destructive) { z.versuch { try z.kern.entwurfVerwerfen(entwurfId: e.id) }; z.neuLaden() }
                    }
                }
                if !s.einnahmenHeute.isEmpty || !s.bedarf.isEmpty {
                    Section(header: Text("Medikamente heute"), footer: Text("Einnahmen werden nie vorausgefüllt.")) {
                        ForEach(Array(s.einnahmenHeute.enumerated()), id: \.offset) { _, e in
                            HStack {
                                VStack(alignment: .leading) {
                                    Text("\(uhrzeit(e.uhrzeitMin))  \(e.name) \(e.staerke ?? "")")
                                    Text("\(e.dosis)  \(standText(e.stand))").font(.caption).foregroundStyle(.secondary)
                                }
                                Spacer()
                                if e.stand == .offen {
                                    Button("Genommen") { z.versuch { try z.kern.einnahmeBestaetigen(praeparatId: e.praeparatId, zeitpunkt: nil) }; z.geaendert() }.buttonStyle(.bordered)
                                    Button("Ausgelassen") { z.seite = .formular("einnahme_ausgelassen", praeparat: e.praeparatId) }.buttonStyle(.borderless)
                                }
                            }
                        }
                        ForEach(s.bedarf, id: \.formularId) { b in Button("Bei Bedarf: \(b.name)") { bedarf = b } }
                    }
                }
                Section("Ereignis festhalten") {
                    ForEach(s.ereignisse, id: \.schluessel) { k in Button(k.name) { z.seite = .formular(k.schluessel) } }
                }
                Section(footer: Text(sicherungText(s))) { if s.sicherungWarnung { Button("Sicherung einrichten") { z.seite = .einstellungen } } }
            }
        }
        .navigationTitle("Heute")
        .onAppear { z.neuLaden() }
        .confirmationDialog(bedarf?.name ?? "", isPresented: Binding(get: { bedarf != nil }, set: { if !$0 { bedarf = nil } })) {
            Button("Jetzt genommen") { if let b = bedarf { z.versuch { try z.kern.einnahmeBestaetigen(praeparatId: b.formularId, zeitpunkt: nil) }; z.geaendert() }; bedarf = nil }
        }
    }

    private func statusText(_ s: Startseite) -> String {
        if s.statusHeute == 0 { return "Heute noch kein Status. Halte ihn fest, so oft du willst." }
        return "Heute \(s.statusHeute) Status" + (s.letzterStatusMs.map { ", zuletzt \(vorText($0))" } ?? "")
    }
    private func standText(_ s: EinnahmeStand) -> String { switch s { case .genommen: "genommen"; case .ausgelassen: "ausgelassen"; case .offen: "offen" } }
    private func sicherungText(_ s: Startseite) -> String {
        guard let t = s.letzteSicherungMs else { return "Noch keine Sicherung. Ohne Sicherung gehen bei Verlust des Geräts alle Einträge verloren." }
        return "Letzte Sicherung \(vorText(t))" + (s.sicherungWarnung ? ". Bitte prüfen." : "")
    }
}

func formularSchluessel(_ z: AppZustand, _ id: String) -> String? {
    ["status_jetzt", "tagesabschluss", "wochenrueckblick"].first { (try? z.kern.formular(schluessel: $0))?.id == id }
}
