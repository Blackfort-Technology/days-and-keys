// SPDX-License-Identifier: MPL-2.0
import Foundation
import LocalAuthentication
import Security

/// Hält KK und KK_h im Schlüsselbund des Geräts, nur lesbar nach Face ID, Touch ID oder
/// Geräte-Code („Benutzer anwesend", nur dieses Gerät, Protokoll 8.3). Der Schlüsselbund ist durch
/// die Secure Enclave geschützt. Grenze (AE-12): Nach dem Lesen liegen die Bytes kurz in Swift.
enum Schluesselbund {
    struct Fehler: Error, LocalizedError {
        let text: String
        var errorDescription: String? { text }
    }

    private static func konto(_ generation: UInt64) -> String { "dak_kk_g\(generation)" }
    private static let dienst = "de.blackforttec.daysandkeys.kk"

    static func speichern(generation: UInt64, kk: Data, kkH: Data) throws {
        var fehler: Unmanaged<CFError>?
        guard let zugriff = SecAccessControlCreateWithFlags(nil, kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly, .userPresence, &fehler) else {
            throw Fehler(text: "Schutz im Schlüsselbund nicht verfügbar. Ist ein Geräte-Code gesetzt?")
        }
        let basis: [String: Any] = [kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: dienst, kSecAttrAccount as String: konto(generation)]
        SecItemDelete(basis as CFDictionary)
        var eintrag = basis
        eintrag[kSecValueData as String] = kk + kkH
        eintrag[kSecAttrAccessControl as String] = zugriff
        let s = SecItemAdd(eintrag as CFDictionary, nil)
        guard s == errSecSuccess else { throw Fehler(text: "Schlüssel ließ sich nicht sichern (\(s)).") }
    }

    static func lesen(generation: UInt64) throws -> (Data, Data) {
        let kontext = LAContext()
        kontext.localizedReason = "Days & Keys entsperren"
        let anfrage: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: dienst, kSecAttrAccount as String: konto(generation),
            kSecReturnData as String: true, kSecUseAuthenticationContext as String: kontext,
        ]
        var wert: CFTypeRef?
        let s = SecItemCopyMatching(anfrage as CFDictionary, &wert)
        guard s == errSecSuccess, var d = wert as? Data, d.count == 64 else {
            throw Fehler(text: s == errSecUserCanceled ? "Abgebrochen" : "Entsperren nicht möglich (\(s)).")
        }
        defer { d.resetBytes(in: 0..<d.count) }
        return (d.subdata(in: 0..<32), d.subdata(in: 32..<64))
    }

    static func aufraeumen(aktuell: UInt64) {
        for g in 1..<max(aktuell, 1) {
            SecItemDelete([kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: dienst, kSecAttrAccount as String: konto(g)] as CFDictionary)
        }
    }

    /// Frische Geräteauthentifizierung für eine sensible Aktion (Protokoll 13).
    static func bestaetigen(_ grund: String) async throws {
        let k = LAContext()
        do {
            _ = try await k.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: grund)
        } catch {
            throw Fehler(text: "Abgebrochen")
        }
    }
}
