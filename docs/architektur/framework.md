# App-Framework für Days & Keys (Bewertung zu #33)

Stand 2026-10-08. Grundlage: `docs/protokoll.md` Teil A v0.7.1 (Abschnitte 1, 6.2, 8, 10, 11, 12,
13, 14), `docs/gesamtkonzept.md` 2.3, `docs/vorlagen.md`, Client-Prüfungen der zweiten (D-1 bis
D-14) und dritten Erörterung (K-1 bis K-15). Alle Versions- und Quelltextangaben sind am
2026-10-08 live geprüft (GitHub-API, crates.io, pub.dev, npm, Herstellerdoku, Quelltext auf `main`
bzw. `dev`).

Kennzeichnung: **[B]** belegt mit Quelle (URL in der Zeile oder im Quellenverzeichnis), **[S]**
Schlussfolgerung oder Schätzung, die der Prototyp #101 bestätigen oder widerlegen muss.

## 1. Kurzfassung

**Empfehlung: Option 3, native Oberflächen (Jetpack Compose und SwiftUI) mit `dak-core` über
UniFFI, ohne WebView.** Dazu eine Bauregel, die den Mehraufwand der zwei Oberflächen klein hält:
Zustand, Validierung, Bedingungen, Planer, Auswertung und Texte der Bestätigungsdialoge liegen als
fertige Ansichtsmodelle in `dak-core`; Kotlin und Swift zeichnen nur und rufen fachliche
Operationen auf.

Begründung in einem Satz: Gut die Hälfte dessen, was Teil A von der Oberfläche verlangt (PIN,
Recovery-Wörter, Bestätigungen, Hilfe ohne Entsperren, Planer mit Neustart-Empfänger, Sicherung
per SAF bzw. Bookmark, Schlüssel in Keystore und Secure Enclave, später Widgets und Health), muss
in jeder Option ohnehin nativ in Kotlin und Swift entstehen; die WebView-Optionen legen eine dritte
Schicht samt eigener Angriffsfläche obendrauf, Flutter eine zweite Rust-Bindung. [S]

Zweitplatzierte ist **Flutter mit `flutter_rust_bridge`** (eine Oberfläche, keine WebView, kein
`INTERNET` im Release, UI-Tests unter Linux). Capacitor und Tauri 2 Mobile sind machbar, kosten aber
dauerhaft Härtungsarbeit gegen Funde, die in Option 3 gar nicht entstehen.

## 2. Was die Anforderungen für die Wahl bedeuten

| Anforderung aus Teil A | Fundstelle | Folge für das Framework |
|---|---|---|
| Keine Kryptografie, keine Schlüssel in der Oberfläche | §2, §13 | `dak-core` hält alles; jede Option braucht eine Bindung, die auch außerhalb der Haupt-Activity bzw. im Erweiterungsprozess erreichbar ist (K-6) |
| Android ohne `INTERNET`, Pipeline prüft die Release-APK | §13, K-4 | Template und Manifest-Merge des Frameworks; WebView-Frameworks bringen `INTERNET` im Template mit |
| Keine Weitergabe fremder URLs | §13, K-4 | WebView-Frameworks reichen fremde Navigation standardmäßig an den Browser oder laden sie selbst; ohne WebView gibt es diesen Pfad nicht |
| PIN, Recovery-Wörter, Bestätigungen, Hilfe nativ | §8.4, §10.3, §13 | in allen Optionen Kotlin- und Swift-Code; bei WebView-Optionen zusätzlich Übergänge zwischen WebView und nativer Ansicht |
| Hilfe-Ansicht ohne Entsperren | §13 | muss ohne Kern-Entsperrung und ohne WebView starten |
| Planer in `dak-core`, Planungsstand ohne Entsperren, exakte Alarme, Neustart | §11, K-8, K-10 | `BroadcastReceiver` für `BOOT_COMPLETED` und Alarme ohne laufende Oberfläche; Benachrichtigungs-Plugins der Frameworks planen selbst und passen nicht zum Planer im Kern [S] |
| Entwürfe laufend versiegeln | §12, K-15 | jede Eingabe geht entprellt an den Kern; in WebView-Optionen über die Bridge |
| Kein Klartext-Zwischenspeicher | §12, D-5 | WebView-Speicher (IndexedDB, `localStorage`, Cache) müssen per Test leer bleiben; ohne WebView entfällt das |
| Sicherungsordner per SAF bzw. Dateien-App, dauerhaft | §10.2, K-11 | `ACTION_OPEN_DOCUMENT_TREE` + `takePersistableUriPermission`, iOS Security-Scoped Bookmark; in allen Optionen eigener nativer Code [S] |
| Später Widgets, Benachrichtigungsaktionen, Health Connect/HealthKit | §6.2, M3 | Widget-Erweiterung (iOS) und Glance (Android) sind immer nativ; der Kern muss dort ohne Framework-Bridge laufen (K-6) |
| Tastatur- und Diktatkontrolle, `FLAG_SECURE`, Vorschaubild | §14, K-14 | native Felder geben feinere Kontrolle als HTML-Attribute (K-14) |

## 3. Die Optionen im Einzelnen

### 3.1 Capacitor 8 + `dak-core` per UniFFI in einem nativen Plugin

- **Reife:** Capacitor 8.5.3 vom 2026-10-07, 9.0.0-alpha.8 am selben Tag; v8 aktiv seit
  2025-12-08, v7 endet im Dezember 2026; MIT. [B] https://api.github.com/repos/ionic-team/capacitor/releases,
  https://capacitorjs.com/docs/main/reference/support-policy
  Ein Major-Wechsel auf v9 fällt wahrscheinlich in den Nutzungsversuch. [S]
- **Rust-Kern:** Keine eingebaute Rust-Unterstützung; `dak-core` wird mit UniFFI zu AAR und
  XCFramework gebaut und in ein eigenes Plugin gelegt (Kotlin/Swift ruft UniFFI, JS ruft das
  Plugin). Drei Sprachgrenzen: TS → Plugin → Rust. [S]
- **Ohne `INTERNET`:** Das Android-Template setzt `INTERNET` und `allowBackup="true"`. [B]
  https://github.com/ionic-team/capacitor/blob/main/android-template/app/src/main/AndroidManifest.xml
  Ob die WebView mit Asset-Auslieferung über den lokalen Server ohne die Berechtigung startet, ist
  nicht belegt. [S, Prototyp]
- **Fremde URLs:** `Bridge.launchIntent` startet für jeden fremden Host `Intent.ACTION_VIEW`;
  vorher fragt es alle Plugins über `shouldOverrideLoad`. Ein eigenes Plugin, das für Fremdes `true`
  liefert, sperrt das auf Android. [B]
  https://github.com/ionic-team/capacitor/blob/main/android/capacitor/src/main/java/com/getcapacitor/Bridge.java
  Auf iOS gilt dasselbe für Navigationen, aber `webView(_:createWebViewWith:...)` (also
  `window.open`, `target=_blank`) ruft `UIApplication.shared.open(url)` **ohne** Plugin-Abfrage. [B]
  https://github.com/ionic-team/capacitor/blob/main/ios/Capacitor/Capacitor/WebViewDelegationHandler.swift
  Abhilfe nur durch eine eigene Unterklasse von `CAPBridgeViewController`, die den `uiDelegate`
  ersetzt. [S]
- **Netz im Kern:** `CapacitorHttp` und `CapacitorCookies` werden fest in `registerAllPlugins`
  registriert, nicht abwählbar. [B] (Bridge.java, Zeilen 659-664) Ohne `INTERNET` scheitert
  `CapacitorHttp` auf Android; auf iOS nicht. [S]
- **WebView-Angriffsfläche:** vollständig vorhanden, inklusive WebRTC, DNS-Prefetch und
  npm-Lieferkette (D-1). Die Zeile „kompromittierter WebView-Code" in §1 bleibt nötig. [B] D-1
- **Native Dialoge:** über eigene Plugins, die Activities bzw. ViewController zeigen. [B]
  https://capacitorjs.com/docs/plugins/android
- **Biometrie/Keystore:** keine offiziellen Plugins mit `CryptoObject`; eigener Code. [S]
- **Exakte Alarme:** `@capacitor/local-notifications` 8.3.1 plant mit
  `setExactAndAllowWhileIdle`, wenn `canScheduleExactAlarms()` es erlaubt [B]
  https://github.com/ionic-team/capacitor-plugins/tree/main/local-notifications ; es plant aber aus JS,
  der Planer nach §11 liegt im Kern, also eigener nativer Planer. [S]
- **Widgets:** keine Erweiterungsziele im Framework (K-6). [B] K-6
- **Ordnerzugriff:** `@capacitor/filesystem` arbeitet mit Pfaden, nicht mit SAF-Bäumen; eigener
  Code. [S]
- **Barrierefreiheit:** HTML-Semantik über die WebView, mit TalkBack/VoiceOver gut nutzbar. [S]
- **Testbarkeit ohne Mac:** Oberfläche mit Vitest/Playwright unter Linux, Android-Emulator unter
  Linux; iOS-Teil nur auf dem Cloud-Mac. [S]
- **Größe:** System-WebView, Paket klein; plus `dak-core`. [S]

### 3.2 Tauri 2 Mobile (`dak-core` direkt in Rust)

- **Reife:** tauri 2.12.1 vom 2026-09-30; parallel laufen 3.0.0-alpha.4 (2026-10-01). Hohe
  Aktivität, 1 489 offene Issues und PRs; MIT/Apache-2.0. [B]
  https://api.github.com/repos/tauri-apps/tauri/releases, https://crates.io/crates/tauri
  Mobile ist der jüngste Teil von Tauri; mit v3 im Anflug droht während des Versuchs eine Migration. [S]
- **Rust-Kern:** der große Vorteil; `dak-core` ist eine normale Abhängigkeit, Befehle sind
  Rust-Funktionen. Native Ansichten (PIN, Wörter, Hilfe) entstehen aber als Kotlin/Swift-Plugins;
  Rust ruft sie über `run_mobile_plugin`, und Plugin-Code ruft Rust „only through JNI on Android or
  C FFI on iOS, with extra boilerplate". [B] https://v2.tauri.app/develop/plugins/develop-mobile/
  Für Widget-Empfänger und Erweiterung ist die Tauri-Bridge nicht initialisiert; es braucht dort eine
  zweite Bindung (K-6). [S]
- **Bau:** iOS „requires Xcode and is only available on macOS"; das Xcode-Projekt wird per XcodeGen
  aus `project.yml` erzeugt. [B] https://v2.tauri.app/start/prerequisites/,
  https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/templates/mobile/ios/project.yml
  Erweiterungsziele müssen dort gepflegt werden, sonst gehen sie beim Neuerzeugen verloren. [S]
- **Ohne `INTERNET`:** Template setzt `INTERNET`. [B]
  https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/templates/mobile/android/app/src/main/AndroidManifest.xml
  Auslieferung über `shouldInterceptRequest` und IPC über JNI sprechen dafür, dass es ohne geht. [S, Prototyp]
- **Fremde URLs:** wry gibt jede Navigation an Rust (`Rust.shouldOverride`); ohne eigenen
  `on_navigation`-Handler ist das Ergebnis „erlauben", die fremde Seite lädt also **in der
  WebView** (keine Weitergabe an den Browser). Ein Handler in Rust sperrt alles Fremde auf beiden
  Plattformen. [B] https://github.com/tauri-apps/wry/blob/dev/src/android/binding.rs,
  https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/manager/webview.rs
  Neu in 2.12.0: `limit_navigations_to_app_bound_domains` (iOS). [B]
  https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md
- **WebView-Angriffsfläche:** vorhanden, aber mit Capability-System (Befehle je Ursprung) besser
  begrenzbar als bei Capacitor; CSP-Sonderfall `ipc:` (D-1). [B] https://v2.tauri.app/security/csp/
- **Biometrie:** `tauri-plugin-biometric` ruft nur den Prompt auf, ohne `CryptoObject`, also keine
  Schlüsselbindung; eigener Code nötig. [B]
  https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/biometric
- **Exakte Alarme:** `tauri-plugin-notification` nutzt `setExactIfPossible` mit
  `canScheduleExactAlarms()`. [B]
  https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/notification
  Gleiche Einschränkung wie bei Capacitor: Planer gehört in den Kern. [S]
- **Widgets:** nur über Drittanbieter (`tauri-plugin-widgets` 0.5.0), das Ziel wird von Hand in
  Xcode angelegt. [B] https://docs.rs/tauri-plugin-widgets/
- **Testbarkeit:** wie Capacitor; Rust-Befehle zusätzlich mit `cargo test`. [S]
- **Größe:** klein, System-WebView plus Rust-Binärdatei. [S]

### 3.3 Native Oberflächen (Compose, SwiftUI) + `dak-core` per UniFFI

- **Reife:** UniFFI 0.32.2 vom 2026-09-23, aktiv, MPL-2.0; Mozilla nutzt es in Firefox für
  Android und iOS, das Projekt nennt sich „ready for production use", aber vor 1.0 mit möglichen
  Brüchen bei fortgeschrittenen Funktionen. [B] https://github.com/mozilla/uniffi-rs,
  https://crates.io/crates/uniffi
  Bitwarden baut seine Android- und iOS-Apps nativ auf einem Rust-SDK mit UniFFI-Bindungen für
  Kotlin und Swift, also genau dieses Muster bei einer Passwort-App. [B]
  https://github.com/bitwarden/sdk-internal
  Compose und SwiftUI sind die Herstellerwege, gepflegt von Google und Apple. [B]
- **Rust-Kern:** eine Bindung für alles: Activity, `BroadcastReceiver`, WorkManager,
  Glance-Widget, iOS-Widget-Erweiterung und Benachrichtigungsaktionen laden dieselbe Bibliothek.
  Damit ist K-6 ohne zweite Bindung lösbar. [S] Android-Bau mit `cargo-ndk` unter Linux; iOS
  verlangt Xcode. `libsodium-sys-stable` 1.24.0 unterstützt `aarch64-apple-ios` und
  `aarch64-apple-ios-sim` und „automatically detects Xcode paths", baut iOS also nur auf dem Mac. [B]
  https://github.com/jedisct1/libsodium-sys-stable
  Diese Grenze gilt für jede Option, weil `dak-core` libsodium enthält. [S]
- **Ohne `INTERNET`:** Ein Compose-Projekt braucht die Berechtigung nicht; sie kommt nur über
  Bibliotheken herein (Manifest-Merge, Pipeline-Prüfung). [S]
- **Fremde URLs:** Es gibt keine WebView und keine Navigation; Links entstehen nur, wo der eigene
  Code `Intent.ACTION_VIEW`/`UIApplication.open` aufruft (Hilfe-Ansicht mit fester Nummernliste). [S]
- **WebView-Angriffsfläche:** keine. Die Zeile „kompromittierter WebView-Code" in §1 wird zu
  „kompromittierte Abhängigkeit im App-Prozess" und gilt dann für alle Optionen gleich. [S]
- **Native Dialoge:** sind die normale Oberfläche; `FLAG_SECURE`,
  `setAccessibilityDataSensitive`, eigene Zifferntastatur ohne IME direkt verfügbar. [S]
- **Biometrie/Keystore/Keychain:** `BiometricPrompt` mit `CryptoObject`, Keystore mit
  `setUnlockedDeviceRequired`, Secure-Enclave-ECDH nach §8.4 direkt in Kotlin bzw. Swift. [B]
  https://developer.android.com/privacy-and-security/keystore,
  https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave
- **Exakte Alarme:** `AlarmManager` und `USE_EXACT_ALARM` direkt (außerhalb von Google Play
  automatisch erteilt, K-10). [B] https://developer.android.com/reference/android/Manifest.permission#USE_EXACT_ALARM
- **Widgets:** Glance und WidgetKit sind die Herstellerwege. [B]
  https://developer.android.com/develop/ui/compose/glance, https://developer.apple.com/documentation/widgetkit
- **Ordnerzugriff:** SAF mit dauerhafter URI-Berechtigung; iOS Ordnerzugriff mit Bookmark. [B]
  https://developer.android.com/training/data-storage/shared/documents-files,
  https://developer.apple.com/documentation/uikit/providing-access-to-directories
- **Barrierefreiheit:** Compose-Semantik, SwiftUI-Accessibility, jeweils Referenzverhalten der
  Plattform. [B] https://developer.android.com/develop/ui/compose/accessibility
- **Testbarkeit ohne Mac:** Compose-UI-Tests laufen mit Robolectric auf der JVM in der CI (GitHub Actions) unter
  Linux. [B] https://developer.android.com/training/testing/local-tests/robolectric
  SwiftUI-Code lässt sich unter Linux weder bauen noch testen; jede iOS-Iteration geht über den
  Cloud-Mac. [S]
- **Größe:** am kleinsten; nur Compose-Laufzeit, JNA (Kotlin-Bindung von UniFFI) und
  `dak-core`. [S]

### 3.4 Flutter + `flutter_rust_bridge`

- **Reife:** Flutter stable 3.47.6 vom 2026-10-01, BSD-3-Clause, Google; `flutter_rust_bridge`
  2.13.0 (2026-08-23), 2.14.0-beta.2, MIT, im Kern von einer Person gepflegt
  (`fzyzcjy`), 35 offene Issues. [B]
  https://storage.googleapis.com/flutter_infra_release/releases/releases_linux.json,
  https://github.com/fzyzcjy/flutter_rust_bridge
- **Rust-Kern:** Dart ruft Rust direkt; Android-Bau unter Linux, iOS nur mit Xcode. Native Teile
  (PIN-Ansicht, Planer-Empfänger, Widget-Erweiterung) erreichen `dak-core` nicht über die Dart-Bridge,
  dort braucht es eine zweite Bindung (UniFFI oder C) oder den Umweg über eine Hintergrund-Engine. [S]
- **Ohne `INTERNET`:** Das Flutter-Template setzt `INTERNET` nur in den Manifesten für `debug` und
  `profile`; das `main`-Manifest hat sie nicht. Eine Release-APK ist ohne Netz der Normalfall. [B]
  https://github.com/flutter/flutter/tree/master/packages/flutter_tools/templates/app/android.tmpl/app/src
- **Fremde URLs:** keine WebView; Links nur mit `url_launcher`, das man weglassen kann. [S]
- **WebView-Angriffsfläche:** keine; dafür pub-Lieferkette und Dart-Code im selben Prozess. [S]
- **Native Dialoge:** über Platform Channels oder Platform Views; jede native Ansicht ist ein
  Übergang zwischen zwei UI-Systemen. [B] https://docs.flutter.dev/platform-integration/platform-channels
- **Biometrie/Keystore:** `local_auth` 3.0.2 nur als Prompt; Schlüsselbindung eigener Code. [B]
  https://pub.dev/packages/local_auth
- **Exakte Alarme:** `flutter_local_notifications` 22.3.1 (2026-09-13), plant aber selbst; für den
  Planer im Kern eigener nativer Teil. [B] https://pub.dev/packages/flutter_local_notifications
- **Widgets:** `home_widget` 0.10.0; Widget-Oberfläche bleibt Swift bzw. Kotlin. [B]
  https://pub.dev/packages/home_widget
- **Ordnerzugriff:** Pakete wie `saf_util` 3.1.1 für Android; iOS-Bookmarks eigener Code. [B]
  https://pub.dev/packages/saf_util
- **Barrierefreiheit:** eigener Semantikbaum, der auf TalkBack/VoiceOver abgebildet wird; gut,
  aber eine Schicht mehr als nativ. [B] https://docs.flutter.dev/ui/accessibility
- **Testbarkeit ohne Mac:** `flutter test` mit Widget- und Golden-Tests unter Linux, auch als
  Linux-Desktop-App lauffähig; nur iOS-Bau und Gerätetests auf dem Cloud-Mac. Für Agenten die
  kürzeste Rückkopplung aller Optionen. [S]
- **Größe:** Flutter-Engine kommt hinzu; das letzte offizielle Beispiel nennt 5,4 MB Download auf
  iOS (Flutter 1.17, veraltet). [B] https://docs.flutter.dev/perf/app-size Aktuell grob 5-8 MB je
  ABI mehr als nativ. [S]

### 3.5 Kurz: Compose Multiplatform + Rust, React Native + UniFFI

- **Compose Multiplatform:** iOS stabil seit 1.8.0 (Mai 2025), aktuell 1.12.1 (2026-09-22),
  Apache-2.0. [B] https://blog.jetbrains.com/kotlin/2025/05/compose-multiplatform-1-8-0-released-compose-multiplatform-for-ios-is-stable-and-production-ready/,
  https://github.com/JetBrains/compose-multiplatform/releases
  Die Rust-Anbindung für Kotlin/Native ist die Schwachstelle: Gobley (UniFFI für KMP) hatte das
  letzte Release 0.3.7 am 2025-10-08, ein Jahr alt. [B] https://crates.io/crates/gobley-uniffi-bindgen
  Ausgeschieden, solange die Bindung so dünn gepflegt ist. [S]
- **React Native + `uniffi-bindgen-react-native`:** RN 0.87.1, MIT; ubrn nur Vorabversionen
  (0.31.0-6 vom 2026-09-25), rund 550 Sterne, MPL-2.0. [B]
  https://github.com/jhugman/uniffi-bindgen-react-native
  Keine WebView, aber JS-Laufzeit und npm-Lieferkette wie bei Capacitor; Bindung noch nicht
  stabil. Ausgeschieden. [S]
- **Crux** (Rust-Kern mit Ansichtsmodellen, dünne Schalen in SwiftUI/Compose, 0.20.0 vom
  2026-08-07, Apache-2.0) beschreibt genau die Bauregel aus Abschnitt 1. [B]
  https://github.com/redbadger/crux Als Vorlage für den Schnitt lesenswert; als Abhängigkeit vor 1.0
  nicht nötig, UniFFI direkt reicht. [S]

## 4. Vergleichstabelle

Bewertung: ++ sehr gut, + gut, o mit Aufwand, - schwach.

| Kriterium | Capacitor + UniFFI | Tauri 2 Mobile | Nativ + UniFFI | Flutter + frb |
|---|---|---|---|---|
| Pflege 2026 | + (8.5.3, v9 in Alpha) [B] | o (2.12.1, v3 in Alpha, Mobile jung) [B] | ++ (Hersteller, UniFFI 0.32.2) [B] | + (Flutter 3.47.6; frb eine Hauptperson) [B] |
| Rust-Anbindung | o, drei Sprachgrenzen [S] | ++ in der App, o für native Teile [B] | ++ eine Bindung überall [S] | + in Dart, o für native Teile [S] |
| Kern in Widget/Empfänger ohne App-Oberfläche | o, zweiter Weg nötig [S] | o, zweiter Weg nötig [S] | ++ dieselbe Bindung [S] | o, zweite Bindung [S] |
| Start ohne `INTERNET` | ? Template hat es, Prototyp [B/S] | ? Template hat es, Prototyp [B/S] | ++ nicht nötig [S] | ++ Release ohne, Standard [B] |
| Fremde URLs | o, Android per Plugin-Hook, iOS `window.open` nur per Unterklasse [B] | + ein Rust-Handler, Standard lädt in der WebView [B] | ++ kein Pfad [S] | ++ kein Pfad ohne `url_launcher` [S] |
| WebView-Angriffsfläche | - voll, inkl. `CapacitorHttp` im Kern [B] | o voll, aber Capabilities [B] | ++ keine [S] | ++ keine [S] |
| Native Dialoge/Ansichten | o Plugin je Ansicht [S] | o Plugin je Ansicht [B] | ++ Normalfall [S] | o Channel/Platform View [B] |
| Keystore/Keychain mit Bindung | o eigener Code [S] | o eigener Code, Plugin ohne `CryptoObject` [B] | ++ direkt [B] | o eigener Code [S] |
| Planer, exakte Alarme, Neustart | o eigener nativer Planer [S] | o eigener nativer Planer [S] | ++ direkt [B] | o eigener nativer Planer [S] |
| Widgets/Erweiterungen (M3) | - nicht vorgesehen [B] | o Drittanbieter, Hand in Xcode [B] | ++ Glance, WidgetKit [B] | o `home_widget`, UI nativ [B] |
| SAF/Bookmark | o eigener Code [S] | o eigener Code [S] | ++ direkt [B] | + Android-Paket, iOS eigen [B] |
| Barrierefreiheit | + HTML [S] | + HTML [S] | ++ Plattform [B] | + Semantikbaum [B] |
| Oberflächen-Code | ++ einmal TS [S] | ++ einmal TS [S] | - zweimal [S] | ++ einmal Dart [S] |
| Gesamter Code inkl. Pflicht-Nativteil | o drei Schichten [S] | o drei Schichten [S] | + zwei Schichten [S] | + zwei Schichten plus zwei Bindungen [S] |
| Tests in der CI ohne Mac | ++ Web-UI unter Linux [S] | ++ Web-UI und Rust unter Linux [S] | o Android ja (Robolectric) [B], iOS nur Cloud-Mac [S] | ++ `flutter test` unter Linux [S] |
| Lieferkette der Oberfläche | - npm [S] | - npm + Cargo [S] | + Gradle/SwiftPM, wenige Pakete [S] | o pub [S] |
| Bundle-Größe | + klein [S] | ++ klein [S] | ++ am kleinsten [S] | o Engine kommt hinzu [B/S] |
| Lizenzen | MIT [B] | MIT/Apache-2.0 [B] | Plattform-SDKs; UniFFI MPL-2.0, JNA Apache-2.0/LGPL-2.1 [B] | BSD-3, MIT [B] |

Lizenzhinweis: MPL-2.0 ist eine Copyleft-Lizenz auf Dateiebene und verträgt sich mit einem
Open-Source-Projekt; ob generierter Bindungscode darunter fällt, ist vor der Veröffentlichung in
`AUDITS.md` festzuhalten. [S]

## 5. Empfehlung und Begründung

**Option 3: Compose + SwiftUI, `dak-core` per UniFFI, keine WebView.**

1. **Weniger Schichten bei gleicher Pflicht.** Teil A verlangt native Ansichten für PIN, Wörter,
   Bestätigungen, Wiederherstellung und Hilfe, dazu nativen Code für Keystore/Secure Enclave,
   Planer mit Neustart-Empfänger, SAF/Bookmark, Backup-Ausschluss und Bildschirmschutz. Der Teil,
   den eine WebView oder Flutter übernehmen würde, sind Formulare, Verläufe und Einstellungen. [S]
   Wer diesen Teil nativ schreibt, spart Bridge-Härtung, CSP, Navigationssperre, WebView-Speicher-
   Tests und die Übergänge zwischen WebView und nativen Ansichten.
2. **Die Hälfte der Client-Funde entfällt.** D-1 (CSP-Lücken, `CapacitorHttp`, WebRTC,
   DNS-Prefetch), K-4 (fremde URLs an den Browser), D-5 (WebView-Cache), D-2 (Masken über der PIN),
   K-14 (HTML-Attribute als Tastaturhinweis) und K-15 (Entwurf lebt in der WebView) haben ohne
   WebView keinen Gegenstand. Der Rest gilt in allen Optionen gleich. [S]
3. **Eine Rust-Bindung für App, Empfänger und Erweiterungen.** Widgets, Benachrichtigungsaktionen
   und der Planer laufen ohne Haupt-Oberfläche; mit UniFFI erreichen sie `dak-core` auf demselben
   Weg wie die App (K-6). Tauri und Flutter brauchen hier eine zweite Bindung. [S]
4. **Belegtes Muster.** Firefox und Bitwarden liefern Rust-Kerne per UniFFI an native Kotlin- und
   Swift-Apps aus. [B]
5. **Wenig Versionsdruck.** Weder Tauri 3 noch Capacitor 9 fallen in den Nutzungsversuch; Compose
   und SwiftUI ändern sich in kleinen Schritten. [S]

**Bauregel gegen die doppelte Oberfläche:** `dak-core` liefert Ansichtsmodelle (welche Fragen
sichtbar, welche Bedingung greift, welcher Bezugszeitraum, welche Punkte im Verlauf, welcher Text im
Bestätigungsdialog samt Digest) als UniFFI-Records; Kotlin und Swift zeichnen nur. Damit liegt die
testbare Logik in Rust und läuft in der CI (GitHub Actions) unter Linux; die Plattformschicht bleibt dünn und
gleichförmig, was Agenten das parallele Schreiben beider Oberflächen erleichtert. [S]

**Folgen für die Dokumente:** Gesamtkonzept §7 (Architekturbild „TypeScript in der WebView"),
§13 (offene Entscheidung), Protokoll §1 (Zeile „kompromittierter WebView-Code"), §2 („keine
Kryptografie in der WebView"), §13 (CSP, `CapacitorHttp`, WebRTC) sowie `README.md` Zeile 54 und
`CLAUDE.md` Zeile 15 sind nach der Entscheidung anzupassen. Die Desktop-Idee (Tauri, `ROADMAP.md`)
bleibt möglich, weil `dak-core` unverändert weiterverwendbar ist; sie bekäme aber eine eigene
Oberfläche. [S]

## 6. Die zwei stärksten Gegenargumente

1. **Zwei Oberflächen, und die iOS-Hälfte baut und testet nur auf dem Cloud-Mac.** SwiftUI lässt
   sich unter Linux nicht übersetzen. Jeder Tippfehler eines Agenten im Swift-Code wird erst im
   Cloud-Bau sichtbar; das kostet Laufzeit, Geld für Bau-Minuten und Geduld, und die Android- und
   iOS-Oberfläche können auseinanderlaufen. Bei Flutter oder einer Web-Oberfläche liefen fast alle
   UI-Tests unter Linux, und nur die nativen Pflichtteile bräuchten den Mac. [S]
   *Gegenmittel:* Ansichtsmodelle in Rust (Abschnitt 5), Swift-Schicht klein halten, ein Cloud-Lauf
   je PR mit Snapshot-Tests, gemeinsame Abnahmetests auf Ebene von `dak-core`.
2. **Flutter erreicht fast denselben Sicherheitsgewinn mit einer Oberfläche.** Keine WebView,
   Release ohne `INTERNET` als Standard [B], fremde URLs nur bei bewusst eingebundenem
   `url_launcher`. Der Vorsprung von Option 3 gegenüber Flutter ist kleiner als gegenüber den
   WebView-Optionen und besteht vor allem aus „eine Bindung statt zwei" und „native Ansichten ohne
   Übergang". Wenn der Prototyp zeigt, dass die native Doppelarbeit für Agenten deutlich teurer ist
   als erwartet, ist Flutter die bessere Wahl. [S]

## 7. Was der Prototyp #101 auf echten Geräten prüfen muss

Die Liste ergänzt „Was der Prototyp #81 zwingend messen muss" aus der dritten Client-Prüfung; dort
schon enthaltene Plattformpunkte (Biometrie-Invalidierung, StrongBox, Secure-Enclave-ECDH,
Bookmarks, 64er-Grenze, Direct Boot) bleiben unverändert.

**Annahmen, die die Empfehlung absichern**

1. **UniFFI-Kette ohne Mac für Android:** `dak-core` mit `libsodium-sys-stable` per `cargo-ndk`
   für `arm64-v8a` und `x86_64` in der CI (GitHub Actions) unter Linux bauen, AAR mit Kotlin-Bindung erzeugen, auf
   einem echten Gerät laden; Release-APK mit `aapt2 dump permissions` ohne `INTERNET`.
2. **UniFFI-Kette auf dem Cloud-Mac für iOS:** XCFramework mit libsodium für Gerät und Simulator,
   Swift-Bindung, signierter TestFlight-Bau aus der CI; gemessen werden Dauer und Kosten je Lauf.
3. **Kern ohne Oberfläche:** `dak-core` aus einem `BroadcastReceiver` (Alarm und
   `BOOT_COMPLETED`) und aus einer iOS-Widget-Erweiterung aufrufen, mit Speicherbedarf in der
   Erweiterung und Laufzeit des Versiegelns (K-6).
4. **Ansichtsmodelle aus Rust:** den Bildschirm „Status jetzt" mit Bedingungen und „seit dem letzten
   Status" einmal in Compose und einmal in SwiftUI aus demselben UniFFI-Record bauen; gemessen
   werden Zeilen je Plattform, Zahl der Cloud-Iterationen bis grün, Abweichungen im Verhalten.
5. **Erfassungsdauer:** Zeit für einen Status jetzt auf beiden Plattformen (vorlagen.md verlangt
   Messung statt Annahme).
6. **Barrierefreiheit:** TalkBack und VoiceOver über Tipp-Reihe 0-10 ohne Vorauswahl, PIN-Feld mit
   `setAccessibilityDataSensitive`, Hilfe-Ansicht ohne Entsperren.
7. **Größe:** APK je ABI und iOS-Download-Größe inklusive `dak-core`, JNA und libsodium.

**Gegenprobe, damit die Entscheidung nicht ungeprüft fällt**

8. **Flutter-Gegenprobe im selben Umfang wie Punkt 4:** derselbe Bildschirm in Flutter mit
   `flutter_rust_bridge`, Release-APK ohne `INTERNET`, Aufruf von `dak-core` aus dem
   Alarm-Empfänger. Fällt der Aufwand bei Punkt 4 deutlich höher aus als hier, kippt die
   Empfehlung zu Flutter.
9. **Nur falls eine WebView-Option doch in Betracht kommt:** Start ohne `INTERNET` auf Android,
   Navigation nach `https://`, `mailto:`, `intent:` und `window.open` startet keine fremde App
   (Capacitor iOS: `createWebViewWith`), kein Klartext in IndexedDB, Cache und `localStorage`.

**Abbruchkriterium:** Lassen sich Punkt 1 bis 3 auf echten Geräten nicht erfüllen, ist die
Empfehlung neu zu bewerten, bevor M1a beginnt.

## 8. Quellen

Versionen und Aktivität (abgerufen 2026-10-08):

- https://api.github.com/repos/ionic-team/capacitor/releases
- https://capacitorjs.com/docs/main/reference/support-policy
- https://api.github.com/repos/tauri-apps/tauri/releases, https://crates.io/crates/tauri
- https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md
- https://github.com/mozilla/uniffi-rs, https://crates.io/crates/uniffi
- https://github.com/bitwarden/sdk-internal
- https://storage.googleapis.com/flutter_infra_release/releases/releases_linux.json
- https://github.com/fzyzcjy/flutter_rust_bridge, https://cjycode.com/flutter_rust_bridge/
- https://github.com/JetBrains/compose-multiplatform/releases
- https://blog.jetbrains.com/kotlin/2025/05/compose-multiplatform-1-8-0-released-compose-multiplatform-for-ios-is-stable-and-production-ready/
- https://crates.io/crates/gobley-uniffi-bindgen
- https://github.com/jhugman/uniffi-bindgen-react-native
- https://github.com/redbadger/crux
- https://github.com/jedisct1/libsodium-sys-stable

Quelltext:

- https://github.com/ionic-team/capacitor/blob/main/android/capacitor/src/main/java/com/getcapacitor/Bridge.java
- https://github.com/ionic-team/capacitor/blob/main/ios/Capacitor/Capacitor/WebViewDelegationHandler.swift
- https://github.com/ionic-team/capacitor/blob/main/android-template/app/src/main/AndroidManifest.xml
- https://github.com/ionic-team/capacitor-plugins/tree/main/local-notifications
- https://github.com/tauri-apps/wry/blob/dev/src/android/kotlin/RustWebViewClient.kt
- https://github.com/tauri-apps/wry/blob/dev/src/android/binding.rs
- https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/manager/webview.rs
- https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/templates/mobile/android/app/src/main/AndroidManifest.xml
- https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-cli/templates/mobile/ios/project.yml
- https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/biometric
- https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/notification
- https://github.com/flutter/flutter/tree/master/packages/flutter_tools/templates/app/android.tmpl/app/src

Dokumentation:

- https://v2.tauri.app/start/prerequisites/, https://v2.tauri.app/develop/plugins/develop-mobile/,
  https://v2.tauri.app/security/csp/
- https://capacitorjs.com/docs/plugins/android, https://capacitorjs.com/docs/apis/http
- https://docs.rs/tauri-plugin-widgets/
- https://docs.flutter.dev/perf/app-size, https://docs.flutter.dev/ui/accessibility,
  https://docs.flutter.dev/platform-integration/platform-channels
- https://pub.dev/packages/flutter_local_notifications, https://pub.dev/packages/home_widget,
  https://pub.dev/packages/local_auth, https://pub.dev/packages/saf_util
- https://developer.android.com/training/testing/local-tests/robolectric
- https://developer.android.com/develop/ui/compose/accessibility
- https://developer.android.com/develop/ui/compose/glance
- https://developer.android.com/privacy-and-security/keystore
- https://developer.android.com/reference/android/Manifest.permission#USE_EXACT_ALARM
- https://developer.android.com/training/data-storage/shared/documents-files
- https://developer.apple.com/documentation/widgetkit
- https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave
- https://developer.apple.com/documentation/uikit/providing-access-to-directories
