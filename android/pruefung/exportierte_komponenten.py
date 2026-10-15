#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Prüft ein (zusammengeführtes) AndroidManifest gegen die Positivliste exportierter Komponenten (#116).

Semgrep meldet pro Manifest höchstens die erste exportierte Activity, deshalb diese Prüfung.
Aufruf: exportierte_komponenten.py <AndroidManifest.xml>   Exit 0 = in Ordnung, 1 = Befund.
"""
import sys
import xml.etree.ElementTree as ET

A = "{http://schemas.android.com/apk/res/android}"
PAKET = "de.blackforttec.daysandkeys"

# Name -> erlaubte Intent-Aktionen
POSITIVLISTE = {
    PAKET + ".MainActivity": {"android.intent.action.MAIN"},
    PAKET + ".plattform.StartEmpfaenger": {
        "android.intent.action.BOOT_COMPLETED",
        "android.intent.action.MY_PACKAGE_REPLACED",
        "android.intent.action.TIMEZONE_CHANGED",
    },
}
# Nur Apps mit dieser Berechtigung dürfen die Komponente aufrufen (Signatur- oder Systemebene).
SCHUTZBERECHTIGUNGEN = {"android.permission.DUMP"}
ARTEN = ("activity", "activity-alias", "receiver", "service", "provider")


def voller_name(name):
    return PAKET + name if name.startswith(".") else name


def befunde(pfad):
    # Eingabe ist das eigene Build-Ergebnis, keine fremde Datei; expat lädt keine externen Entitäten.
    # nosemgrep: python.lang.security.use-defused-xml-parse.use-defused-xml-parse
    wurzel = ET.parse(pfad).getroot()  # nosec B314
    app = wurzel.find("application")
    if app is None:
        return ["kein <application>-Element"]
    liste = []
    for art in ARTEN:
        for k in app.findall(art):
            name = voller_name(k.get(A + "name", "?"))
            filter_ = k.findall("intent-filter")
            exportiert = k.get(A + "exported")
            # Ohne Angabe gilt eine Komponente mit Intent-Filter als exportiert (vor Android 12).
            if exportiert != "true" and not (exportiert is None and filter_):
                continue
            if k.get(A + "permission") in SCHUTZBERECHTIGUNGEN:
                continue
            if name not in POSITIVLISTE:
                liste.append(f"{art} {name} ist exportiert und steht nicht auf der Positivliste")
                continue
            for f in filter_:
                if f.find("data") is not None:
                    liste.append(f"{art} {name} nimmt Daten (<data>) an")
                for a in f.findall("action"):
                    aktion = a.get(A + "name")
                    if aktion not in POSITIVLISTE[name]:
                        liste.append(f"{art} {name} nimmt die Aktion {aktion} an")
    return liste


if __name__ == "__main__":
    b = befunde(sys.argv[1])
    for zeile in b:
        print("Befund:", zeile)
    print("Exportierte Komponenten:", "Befund" if b else "in Ordnung")
    sys.exit(1 if b else 0)
