// SPDX-License-Identifier: MPL-2.0
import SwiftUI
import UserNotifications

@main
struct DaysAndKeysApp: App {
    @StateObject private var zustand = AppZustand()
    @Environment(\.scenePhase) private var phase
    @State private var imHintergrundSeit: Date?
    private let mitteilungen = Mitteilungen()

    init() {
        UNUserNotificationCenter.current().delegate = mitteilungen
    }

    var body: some Scene {
        WindowGroup {
            Hauptansicht()
                .environmentObject(zustand)
                .onAppear { mitteilungen.zustand = zustand }
                // Vorschaubild im App-Umschalter abdecken (Protokoll 14).
                .overlay { if phase != .active && zustand.entsperrt { Color(.systemBackground).ignoresSafeArea() } }
        }
        .onChange(of: phase) { _, neu in
            switch neu {
            case .background:
                imHintergrundSeit = Date()
                zustand.imHintergrund()
            case .active:
                if let s = imHintergrundSeit { zustand.zurueck(Date().timeIntervalSince(s)) }
                imHintergrundSeit = nil
            default: break
            }
        }
    }
}

final class Mitteilungen: NSObject, UNUserNotificationCenterDelegate {
    weak var zustand: AppZustand?
    func userNotificationCenter(_ c: UNUserNotificationCenter, didReceive r: UNNotificationResponse) async {
        let ziel = r.notification.request.content.userInfo["ziel"] as? String
        await MainActor.run { if let ziel { zustand?.zielSetzen(ziel) } }
    }
    func userNotificationCenter(_ c: UNUserNotificationCenter, willPresent n: UNNotification) async -> UNNotificationPresentationOptions { [.banner, .sound] }
}
