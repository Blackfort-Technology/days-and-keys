# Verteilung über Google Play

Stand 2026-10-17, Issue #159. Die Release-Pipeline (`.github/workflows/release.yml`) baut bei
jedem Lauf neben der APK ein signiertes AAB und lädt es in die **interne Testspur** von Google
Play, sobald in der GitHub-Umgebung `release` das Secret `PLAY_SERVICE_ACCOUNT_JSON` gesetzt ist.
Bis dahin überspringt der Job `play` den Upload und schreibt den Grund als Hinweis ins Log.

Die Menüpunkte der Play Console ändern sich gelegentlich und heißen in der deutschen Oberfläche
teils anders als in der englischen. Wo unten kein Name steht, ist der Punkt allgemein beschrieben.

## Einmalig in der Play Console (Christian)

1. **Entwicklerkonto als Organisation** für die Blackfort Technology UG anlegen. Google verlangt
   dafür eine D-U-N-S-Nummer, eine Identitätsprüfung und eine einmalige Gebühr. Bei
   Organisationskonten entfällt nach unserem Kenntnisstand die Pflicht, vor der ersten
   Produktionsfreigabe einen geschlossenen Test mit einer Mindestzahl an Testern zu fahren; das
   betrifft private Konten. Vor dem Anlegen kurz gegenprüfen.
2. **App anlegen:** Standardsprache Deutsch, Typ App, kostenlos. Den Paketnamen fragt die Console
   erst beim ersten Upload ab; er lautet `de.blackforttec.daysandkeys` und lässt sich danach nicht
   mehr ändern.
3. **Erstes AAB von Hand hochladen.** Die Publisher API arbeitet erst mit einer App, für die schon
   ein Bundle existiert. Dazu das AAB des neuesten Releases
   (https://github.com/Blackfort-Technology/days-and-keys/releases, Datei
   `days-and-keys-0.1.N.aab`) herunterladen, Prüfsumme gegen die `.sha256`-Datei prüfen und im
   Bereich für interne Tests eine neue Version mit diesem AAB anlegen.
4. **Play App Signing mit einem von Google erzeugten Schlüssel.** Beim ersten Upload bietet die
   Console die App-Signatur durch Google an; die Voreinstellung mit einem von Google erzeugten
   Schlüssel übernehmen und keinen eigenen App-Signaturschlüssel hochladen. Der Schlüssel, mit dem
   unsere Pipeline heute signiert (in OpenBao unter `lab/apps/days-and-keys`), wird damit zum
   **Upload-Schlüssel**: Play prüft an ihm, dass ein Upload von uns kommt, und signiert die
   ausgelieferte Fassung selbst neu. Geht der Upload-Schlüssel verloren, kann der Play-Support ihn
   zurücksetzen; den App-Signaturschlüssel verwahrt Google.
5. **Tester festlegen:** für die interne Testspur eine Testerliste mit den Google-Konten der
   Versuchsteilnehmenden anlegen. Sie erhalten einen Teilnahme-Link und installieren danach über
   den Play Store.
6. **Dienstkonto für die Pipeline:**
   - In der Google Cloud ein Projekt für die Play-Anbindung anlegen (oder ein vorhandenes ohne
     andere Aufgaben nehmen), dort die *Google Play Android Developer API* aktivieren.
   - Ein Dienstkonto anlegen und dafür einen Schlüssel im JSON-Format erzeugen. Blockiert eine
     Organisationsrichtlinie das Erzeugen von Schlüsseln, muss sie für dieses Projekt aufgehoben
     werden.
   - In der Play Console unter der Benutzer- und Rechteverwaltung die E-Mail-Adresse des
     Dienstkontos einladen, mit der Rolle **Release-Manager** und **nur für diese App**, ohne
     kontoweite Rechte. Lässt die Console eine engere Auswahl zu, genügt das Recht, Versionen in
     Testspuren zu veröffentlichen; Produktionsrechte braucht die Pipeline nicht.
7. **Schlüssel nach OpenBao** und die lokale Datei danach löschen:

   ```
   bao kv put -mount=lab apps/days-and-keys/google-play service_account_json=@dienstkonto.json
   ```

   Anschließend auf claude-control `python3 ~/dak/skripte/secrets_sync.py` ausführen. Das Skript
   spiegelt den Eintrag in das GitHub-Secret `PLAY_SERVICE_ACCOUNT_JSON` der Umgebung `release`
   und meldet am Ende `Play: True`. Fehlt der Pfad, überspringt es nur diesen Eintrag.

## Was danach automatisch passiert

Bei jedem Push auf `main`, der Kern, App, Vorlagen oder die Release-Pipeline ändert, und bei
manuellem Start des Workflows:

1. Job `android` baut APK und AAB in einem Gradle-Lauf mit demselben `versionCode`
   (Laufnummer des Workflows) und prüft:
   - APK-Signatur (`apksigner`) und AAB-Signatur (`jarsigner -verify`), beide mit demselben
     Zertifikat, also dem Upload-Schlüssel;
   - keine INTERNET-Berechtigung in APK (`aapt2`) und AAB (`bundletool dump manifest`, Version
     und SHA-256 festgelegt);
   - Paketname und `versionCode` des AAB.
   Danach legt er das GitHub-Release `android-0.1.N` mit APK, AAB und beiden `.sha256`-Dateien an.
2. Job `play` lädt das AAB aus genau diesem Release, prüft die Prüfsumme und lädt es mit
   `android/play/play_upload.py` in die Spur `internal`. Das Skript nutzt die Android Publisher API
   v3 (Bearbeitung öffnen, Bundle hochladen, Spur setzen, Bearbeitung abschließen), vergleicht
   `versionCode` und SHA-256 mit der Antwort von Play und verwirft die Bearbeitung bei jedem
   Fehler.
3. Der Status des Releases ist `draft`, solange die Repo-Variable `PLAY_RELEASE_STATUS` nicht
   gesetzt ist. Bei einer App, die in Play noch nie veröffentlicht wurde, lässt Google nur Entwürfe
   zu; Christian gibt den Entwurf dann in der Console für die Tester frei. Ist die erste Version
   freigegeben, `PLAY_RELEASE_STATUS` in den Repo-Variablen auf `completed` setzen; dann erreichen
   neue Versionen die internen Tester ohne Handgriff. Lehnt Play mit dem Hinweis ab, die App lasse
   nur Entwürfe zu, war das zu früh: Variable wieder entfernen.

Schlägt nur der Upload fehl, genügt es, den Job `play` des Laufs erneut zu starten; das AAB liegt
im Release. Jeder `versionCode` lässt sich in Play nur einmal hochladen.

## Wechsel von der GitHub-APK auf die Play-Fassung

Die APK aus den GitHub-Releases ist mit unserem Schlüssel signiert, die Fassung aus Play mit dem
App-Signaturschlüssel von Google. Android installiert eine App nicht über eine Fassung mit anderer
Signatur. Wer die APK schon installiert hat, wechselt deshalb so:

1. In der alten App eine **aktuelle Sicherung** in den Sicherungsordner schreiben lassen und
   prüfen, dass die App keine fehlgeschlagene oder veraltete Sicherung anzeigt.
2. Die **Recovery-Wörter** bereitlegen. Ohne sie lässt sich die Sicherung nicht öffnen.
3. Die alte App deinstallieren. Dabei löscht Android die Daten der App und ihre Schlüssel im
   Keystore; der Sicherungsordner bleibt, weil er außerhalb der App liegt.
4. Die App über den Teilnahme-Link aus Play installieren und beim Einrichten aus der Sicherung
   wiederherstellen: Sicherungsordner und Recovery-Wörter angeben. Berechtigungen wie
   Benachrichtigungen fragt die App neu ab.

Die Wiederherstellung auf diesem Weg ist getestet. Danach auf dem Gerät keine APK aus den
GitHub-Releases mehr installieren: beide Fassungen haben denselben Paketnamen, aber verschiedene
Signaturen, und schließen sich gegenseitig aus. Die Releases auf GitHub bleiben für Geräte ohne
Play Store bestehen.
