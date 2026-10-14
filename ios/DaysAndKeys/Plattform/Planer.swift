// SPDX-License-Identifier: MPL-2.0
import Foundation
import UserNotifications

/// Setzt die vom Kern geplanten lokalen Benachrichtigungen (Protokoll 11). iOS hält höchstens 64;
/// neutrale Texte ohne Präparatnamen und Werte (#69).
enum Planer {
    static func erlaubnisAnfragen() async -> Bool {
        (try? await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])) ?? false
    }

    static func setzen(_ plan: [Benachrichtigung]) {
        let c = UNUserNotificationCenter.current()
        c.removeAllPendingNotificationRequests()
        let fmt = DateFormatter()
        fmt.dateFormat = "yyyy-MM-dd"
        fmt.timeZone = .current
        for b in plan.prefix(60) {
            guard let tag = fmt.date(from: b.datumIso) else { continue }
            var k = Calendar.current.dateComponents([.year, .month, .day], from: tag)
            k.hour = Int(b.uhrzeitMin) / 60
            k.minute = Int(b.uhrzeitMin) % 60
            let inhalt = UNMutableNotificationContent()
            inhalt.title = b.titel
            inhalt.body = b.text
            inhalt.userInfo = ["ziel": b.ziel]
            inhalt.interruptionLevel = b.kanal == "medikament" ? .timeSensitive : .active
            c.add(UNNotificationRequest(identifier: b.schluessel, content: inhalt, trigger: UNCalendarNotificationTrigger(dateMatching: k, repeats: false)))
        }
    }
}
