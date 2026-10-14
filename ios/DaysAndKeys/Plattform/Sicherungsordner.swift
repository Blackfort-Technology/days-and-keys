// SPDX-License-Identifier: MPL-2.0
import Foundation

/// Automatische Sicherung in einen einmal gewählten Ordner der Dateien-App mit dauerhaftem Zugriff
/// über ein Security-scoped Bookmark (Protokoll 10.2, AE-17).
enum Sicherungsordner {
    private static let schluessel = "sicherungsordner"

    static var gewaehlt: Bool { UserDefaults.standard.data(forKey: schluessel) != nil }

    static func uebernehmen(_ url: URL) throws {
        guard url.startAccessingSecurityScopedResource() else { throw Schluesselbund.Fehler(text: "Kein Zugriff auf den Ordner") }
        defer { url.stopAccessingSecurityScopedResource() }
        let b = try url.bookmarkData(options: [], includingResourceValuesForKeys: nil, relativeTo: nil)
        UserDefaults.standard.set(b, forKey: schluessel)
    }

    static func mitOrdner<T>(_ url: URL? = nil, _ f: (URL) throws -> T) throws -> T {
        var veraltet = false
        let ziel: URL
        if let url { ziel = url } else {
            guard let b = UserDefaults.standard.data(forKey: schluessel) else { throw Schluesselbund.Fehler(text: "Noch kein Sicherungsordner gewählt") }
            ziel = try URL(resolvingBookmarkData: b, bookmarkDataIsStale: &veraltet)
        }
        let ok = ziel.startAccessingSecurityScopedResource()
        defer { if ok { ziel.stopAccessingSecurityScopedResource() } }
        if veraltet { try? uebernehmen(ziel) }
        return try f(ziel)
    }

    struct Datei { let name: String; let url: URL; let geaendert: Date }

    static func dateien(_ ordner: URL) throws -> [Datei] {
        let inhalt = try FileManager.default.contentsOfDirectory(at: ordner, includingPropertiesForKeys: [.contentModificationDateKey])
        return inhalt.filter { $0.pathExtension == "daks" || $0.pathExtension == "dakm" }.map {
            Datei(name: $0.lastPathComponent, url: $0, geaendert: (try? $0.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate) ?? Date())
        }
    }

    enum Ergebnis { case keinOrdner, ersetzt, gesichert(String) }

    /// Sichert, liest zurück, lässt den Kern prüfen, räumt nach der Aufbewahrungsregel auf.
    static func sichern(_ kern: DakKern) throws -> Ergebnis {
        guard gewaehlt else { return .keinOrdner }
        return try mitOrdner { ordner in
            let vorhanden = try dateien(ordner)
            let marker = try vorhanden.filter { $0.name.hasSuffix(".dakm") }.map { try Data(contentsOf: $0.url) }
            if try !kern.markerPruefen(marker: marker) { return .ersetzt }
            let s = try kern.sicherungErstellen()
            let ziel = ordner.appendingPathComponent(s.name)
            try s.bytes.write(to: ziel, options: [.atomic])
            let zurueck = try Data(contentsOf: ziel)
            _ = try kern.sicherungBestaetigen(gelesen: zurueck)
            let tag = { (d: Date) -> Int64 in Int64(floor(d.timeIntervalSince1970 / 86400)) }
            let alle = try dateien(ordner).filter { $0.name.hasSuffix(".daks") }
            let weg = try kern.aufbewahrung(dateien: alle.map { DateiInfo(name: $0.name, tag: tag($0.geaendert)) }, heuteTag: tag(Date()))
            for n in weg { if let d = alle.first(where: { $0.name == n }) { try? FileManager.default.removeItem(at: d.url) } }
            return .gesichert(s.name)
        }
    }

    static func sicherungen(_ ordner: URL) throws -> ([SicherungDatei], [Data]) {
        try mitOrdner(ordner) { o in
            let d = try dateien(o)
            return (try d.filter { $0.name.hasSuffix(".daks") }.map { SicherungDatei(name: $0.name, bytes: try Data(contentsOf: $0.url)) },
                    try d.filter { $0.name.hasSuffix(".dakm") }.map { try Data(contentsOf: $0.url) })
        }
    }

    static func schreiben(_ ordner: URL, name: String, daten: Data) throws {
        try mitOrdner(ordner) { o in try daten.write(to: o.appendingPathComponent(name), options: [.atomic]) }
    }
}
