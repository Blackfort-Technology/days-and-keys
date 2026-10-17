#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Lädt ein signiertes AAB in eine Testspur von Google Play (#159).

Ablauf über die Android Publisher API v3: edits.insert, edits.bundles.upload,
edits.tracks.update, edits.commit. Schlägt ein Schritt fehl, wird die Bearbeitung
verworfen (edits.delete), damit in der Play Console nichts Halbfertiges liegen bleibt.

Der Dienstkonto-Schlüssel kommt ausschließlich aus der Umgebungsvariablen
PLAY_SERVICE_ACCOUNT_JSON. Das Skript gibt ihn nie aus, auch nicht in Fehlermeldungen.

Aufruf:
  play_upload.py --paket de.blackforttec.daysandkeys --aab app.aab --version-code 42 \
      --name 0.1.42 [--spur internal] [--status draft]
"""
import argparse
import hashlib
import json
import os
import sys

API = "https://androidpublisher.googleapis.com/androidpublisher/v3/applications"
UPLOAD = "https://androidpublisher.googleapis.com/upload/androidpublisher/v3/applications"
SCOPE = "https://www.googleapis.com/auth/androidpublisher"
# draft: Pflicht, solange die App in Play noch nie veröffentlicht wurde.
# completed: sobald es eine veröffentlichte Fassung gibt. Gestaffelte Auslieferung ist hier nicht vorgesehen.
STATUS = ("draft", "completed")


class Fehler(Exception):
    pass


def pruefe(antwort, schritt):
    """Wirft bei HTTP-Fehlern; der Antworttext stammt von Google und enthält keine Zugangsdaten."""
    if antwort.status_code // 100 != 2:
        raise Fehler(f"{schritt}: HTTP {antwort.status_code}: {antwort.text[:2000]}")
    return antwort.json() if antwort.content else {}


def hochladen(sitzung, paket, aab, version_code, name, spur="internal", status="draft"):
    """Führt eine vollständige Bearbeitung aus. `sitzung` ist eine autorisierte requests-Sitzung."""
    if status not in STATUS:
        raise Fehler(f"Status {status!r} nicht vorgesehen, erlaubt: {', '.join(STATUS)}")
    basis = f"{API}/{paket}/edits"
    edit = pruefe(sitzung.post(basis, json={}, timeout=60), "edits.insert")["id"]
    try:
        # Ganz in den Speicher (einige zehn MB): eine Wiederholung nach Token-Erneuerung sendet so wieder alles.
        with open(aab, "rb") as f:
            inhalt = f.read()
        bundle = pruefe(
            sitzung.post(
                f"{UPLOAD}/{paket}/edits/{edit}/bundles",
                params={"uploadType": "media"},
                data=inhalt,
                headers={"Content-Type": "application/octet-stream"},
                timeout=900,
            ),
            "edits.bundles.upload",
        )
        if int(bundle.get("versionCode", -1)) != int(version_code):
            raise Fehler(f"Play meldet versionCode {bundle.get('versionCode')}, erwartet {version_code}")
        if bundle.get("sha256") and bundle["sha256"].lower() != hashlib.sha256(inhalt).hexdigest():
            raise Fehler("Prüfsumme des hochgeladenen AAB stimmt nicht mit der lokalen Datei überein")
        print(f"AAB hochgeladen: versionCode {bundle['versionCode']}, sha256 {bundle.get('sha256', '-')}")
        release = {"name": name, "versionCodes": [str(version_code)], "status": status}
        pruefe(
            sitzung.put(f"{basis}/{edit}/tracks/{spur}", json={"track": spur, "releases": [release]}, timeout=60),
            "edits.tracks.update",
        )
        pruefe(sitzung.post(f"{basis}/{edit}:commit", timeout=120), "edits.commit")
    except BaseException:
        try:
            sitzung.delete(f"{basis}/{edit}", timeout=60)
        except Exception:  # Aufräumen darf den eigentlichen Fehler nicht verdecken.
            print("Hinweis: Bearbeitung konnte nicht verworfen werden; sie verfällt bei Google von selbst.", file=sys.stderr)
        raise
    print(f"Spur {spur}: Release {name} mit Status {status} übernommen.")


def sitzung_aus_umgebung():
    roh = os.environ.get("PLAY_SERVICE_ACCOUNT_JSON", "")
    if not roh.strip():
        raise Fehler("PLAY_SERVICE_ACCOUNT_JSON ist leer")
    try:
        info = json.loads(roh)
    except ValueError:
        # Bewusst ohne die Originalmeldung, die Teile des Inhalts zitieren könnte.
        raise Fehler("PLAY_SERVICE_ACCOUNT_JSON ist kein gültiges JSON") from None
    if not isinstance(info, dict) or info.get("type") != "service_account":
        raise Fehler("PLAY_SERVICE_ACCOUNT_JSON ist kein Dienstkonto-Schlüssel (type service_account erwartet)")
    # Erst hier importieren, damit der Test ohne die Abhängigkeiten läuft.
    from google.auth.transport.requests import AuthorizedSession
    from google.oauth2 import service_account

    return AuthorizedSession(service_account.Credentials.from_service_account_info(info, scopes=[SCOPE]))


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--paket", required=True)
    p.add_argument("--aab", required=True)
    p.add_argument("--version-code", required=True, type=int)
    p.add_argument("--name", required=True)
    p.add_argument("--spur", default="internal")
    p.add_argument("--status", default="draft", choices=STATUS)
    a = p.parse_args(argv)
    try:
        hochladen(sitzung_aus_umgebung(), a.paket, a.aab, a.version_code, a.name, a.spur, a.status)
    except Fehler as e:
        print(f"Fehler: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
