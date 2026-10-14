// SPDX-License-Identifier: MPL-2.0
import Foundation
import SwiftUI

enum Seite: Hashable {
    case sperre, einrichtung, wiederherstellung, start, verlauf, eintraege, medikamente, einstellungen, hilfe
    case formular(String, entwurf: String? = nil, praeparat: String? = nil, korrektur: String? = nil)
    case medikament(String?)
}

/// Fehlertext des Kerns ohne Inhalte (Architektur Abschnitt 7).
func meldung(_ e: Error) -> String {
    let t = "\(e)"
    if let r = t.range(of: "meldung: \""), let ende = t[r.upperBound...].lastIndex(of: "\"") { return String(t[r.upperBound..<ende]) }
    return (e as? LocalizedError)?.errorDescription ?? t
}

@MainActor
final class AppZustand: ObservableObject {
    let kern: DakKern
    @Published var seite: Seite = .sperre
    @Published var hinweis: String?
    @Published var start: Startseite?
    @Published var arbeitet = false
    @Published var krisenhinweisGesehen = false
    var ziel: String?
    private var sperrzeit: TimeInterval = 60

    init() {
        let ordner = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        try? FileManager.default.createDirectory(at: ordner, withIntermediateDirectories: true)
        var pfad = ordner.appendingPathComponent("dak.sqlite")
        // Datenschutzklasse und kein iCloud-Backup (#67): die App sichert selbst verschlüsselt.
        var w = URLResourceValues(); w.isExcludedFromBackup = true
        try? pfad.setResourceValues(w)
        try? FileManager.default.setAttributes([.protectionKey: FileProtectionType.completeUntilFirstUserAuthentication], ofItemAtPath: ordner.path)
        kern = try! DakKern.neu(pfad: pfad.path, uhr: Systemuhr())
        if case .nichtEingerichtet = (try? kern.lage()) ?? .nichtEingerichtet { seite = .einrichtung }
    }

    var entsperrt: Bool { if case .entsperrt = (try? kern.lage()) { return true }; return false }

    @discardableResult
    func versuch<T>(_ f: () throws -> T) -> T? {
        do { return try f() } catch { hinweis = meldung(error); return nil }
    }

    func entsperren() {
        guard case let .gesperrt(generation, _, _, _) = (try? kern.lage()) else { return }
        arbeitet = true
        Task.detached { [kern] in
            do {
                let (a, b) = try Schluesselbund.lesen(generation: generation)
                try kern.entsperren(kk: a, kkH: b)
                await MainActor.run { self.arbeitet = false; self.nachEntsperren() }
            } catch {
                await MainActor.run { self.arbeitet = false; self.hinweis = meldung(error) }
            }
        }
    }

    func nachEntsperren(navigieren: Bool = true) {
        if let e = versuch({ try kern.einstellungen() }) { sperrzeit = TimeInterval(e.sperrzeitS) }
        neuLaden()
        planen()
        if start?.sicherungNoetig == true { sichern() }
        guard navigieren else { return }
        let z = ziel; ziel = nil
        seite = (z != nil && z != "einnahme") ? .formular(z!) : .start
    }

    func zielSetzen(_ z: String) {
        if entsperrt { seite = z != "einnahme" ? .formular(z) : .start } else { ziel = z; seite = .sperre }
    }

    func sperren() { kern.sperren(); start = nil; seite = .sperre }
    func imHintergrund() { if entsperrt && start?.sicherungNoetig == true { sichern() } }
    func zurueck(_ abwesend: TimeInterval) { if entsperrt && abwesend > sperrzeit { sperren() } }
    func neuLaden() { start = versuch { try kern.startseite() } }
    func planen() { if let p = versuch({ try kern.benachrichtigungen(tage: 4, maxGesamt: 60) }) { Planer.setzen(p) } }
    func geaendert() { neuLaden(); planen() }

    func sichern(_ danach: ((String) -> Void)? = nil) {
        let text: String
        do {
            switch try Sicherungsordner.sichern(kern) {
            case .keinOrdner: text = "Noch kein Sicherungsordner gewählt"
            case .ersetzt: text = "Dieses Gerät wurde durch ein anderes ersetzt und sichert nicht mehr."; hinweis = text; sperren(); return
            case .gesichert: text = "Gesichert"
            }
        } catch { text = "Sicherung fehlgeschlagen: \(meldung(error))" }
        if entsperrt { neuLaden() }
        danach?(text)
    }
}
