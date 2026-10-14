// SPDX-License-Identifier: MPL-2.0
import Foundation

/// Zeit mit Versatz und IANA-Zone zum Zeitpunkt der Erfassung (Protokoll 6.3).
final class Systemuhr: DakUhr {
    func jetzt() -> Zeitpunkt {
        let jetzt = Date()
        let zone = TimeZone.current
        return Zeitpunkt(utcMs: Int64(jetzt.timeIntervalSince1970 * 1000), versatzMin: Int32(zone.secondsFromGMT(for: jetzt) / 60), zone: zone.identifier)
    }
}
