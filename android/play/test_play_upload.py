# SPDX-License-Identifier: MPL-2.0
"""Offline-Test für play_upload.py (#159): Reihenfolge der API-Aufrufe, Aufräumen bei Fehlern,
keine Ausgabe des Schlüssels. Läuft in der CI, weil die Release-Pipeline im PR nicht läuft."""
import contextlib
import hashlib
import io
import json
import os
import tempfile
import unittest
from unittest import mock

import play_upload as pu

PAKET = "de.blackforttec.daysandkeys"


class Antwort:
    def __init__(self, status, daten=None):
        self.status_code = status
        self._daten = daten or {}
        self.text = json.dumps(self._daten)
        self.content = self.text.encode()

    def json(self):
        return self._daten


class Sitzung:
    """Zeichnet Aufrufe auf und antwortet wie die Publisher API."""

    def __init__(self, inhalt, version_code=42, fehler_bei=None, sha=None):
        self.aufrufe = []
        self.version_code = version_code
        self.fehler_bei = fehler_bei
        self.sha = sha or hashlib.sha256(inhalt).hexdigest()

    def _antwort(self, schritt, daten):
        return Antwort(500, {"error": {"message": "kaputt"}}) if schritt == self.fehler_bei else Antwort(200, daten)

    def post(self, url, **kw):
        if url.endswith("/edits"):
            self.aufrufe.append(("insert", None))
            return self._antwort("insert", {"id": "e1"})
        if "/upload/" in url and url.endswith("/edits/e1/bundles"):
            if kw.get("params") != {"uploadType": "media"}:
                raise AssertionError(kw.get("params"))
            self.aufrufe.append(("upload", len(kw["data"])))
            return self._antwort("upload", {"versionCode": self.version_code, "sha256": self.sha})
        if url.endswith("/edits/e1:commit"):
            self.aufrufe.append(("commit", None))
            return self._antwort("commit", {"id": "e1"})
        raise AssertionError(url)

    def put(self, url, json=None, **kw):
        if not url.endswith("/edits/e1/tracks/internal"):
            raise AssertionError(url)
        self.aufrufe.append(("track", json))
        return self._antwort("track", json)

    def delete(self, url, **kw):
        if not url.endswith("/edits/e1"):
            raise AssertionError(url)
        self.aufrufe.append(("delete", None))
        return Antwort(204)


class PlayUploadTest(unittest.TestCase):
    def setUp(self):
        self.inhalt = b"PK\x03\x04 kein echtes Bundle"
        d = tempfile.mkdtemp()
        self.aab = os.path.join(d, "app.aab")
        with open(self.aab, "wb") as f:
            f.write(self.inhalt)

    def lauf(self, s, **kw):
        with contextlib.redirect_stdout(io.StringIO()):
            pu.hochladen(s, PAKET, self.aab, 42, "0.1.42", **kw)

    def test_reihenfolge_und_entwurf(self):
        s = Sitzung(self.inhalt)
        self.lauf(s)
        self.assertEqual([a[0] for a in s.aufrufe], ["insert", "upload", "track", "commit"])
        self.assertEqual(s.aufrufe[1][1], len(self.inhalt))
        self.assertEqual(
            s.aufrufe[2][1],
            {"track": "internal", "releases": [{"name": "0.1.42", "versionCodes": ["42"], "status": "draft"}]},
        )

    def test_fehler_verwirft_bearbeitung(self):
        for schritt in ("upload", "track", "commit"):
            s = Sitzung(self.inhalt, fehler_bei=schritt)
            with self.assertRaises(pu.Fehler):
                self.lauf(s)
            schritte = [a[0] for a in s.aufrufe]
            self.assertEqual(schritte[-1], "delete", schritt)
            if schritt != "commit":
                self.assertNotIn("commit", schritte, schritt)

    def test_falscher_version_code(self):
        s = Sitzung(self.inhalt, version_code=41)
        with self.assertRaises(pu.Fehler):
            self.lauf(s)
        self.assertEqual([a[0] for a in s.aufrufe], ["insert", "upload", "delete"])

    def test_falsche_pruefsumme(self):
        s = Sitzung(self.inhalt, sha="00" * 32)
        with self.assertRaises(pu.Fehler):
            self.lauf(s)
        self.assertEqual(s.aufrufe[-1][0], "delete")

    def test_unbekannter_status(self):
        with self.assertRaises(pu.Fehler):
            self.lauf(Sitzung(self.inhalt), status="inProgress")

    def test_schluessel_erscheint_nie_in_der_meldung(self):
        geheim = '{"type": "authorized_user", "private_key": "GEHEIM-12345"'
        for wert in (geheim, geheim + "}"):
            err = io.StringIO()
            with mock.patch.dict(os.environ, {"PLAY_SERVICE_ACCOUNT_JSON": wert}), contextlib.redirect_stderr(err):
                rc = pu.main(["--paket", PAKET, "--aab", self.aab, "--version-code", "42", "--name", "0.1.42"])
            self.assertEqual(rc, 1)
            self.assertNotIn("GEHEIM", err.getvalue())
            self.assertIn("Fehler:", err.getvalue())

    def test_leerer_schluessel(self):
        with mock.patch.dict(os.environ, {"PLAY_SERVICE_ACCOUNT_JSON": ""}), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(pu.main(["--paket", PAKET, "--aab", self.aab, "--version-code", "1", "--name", "x"]), 1)

    @unittest.skipUnless(os.environ.get("DAK_PLAY_DEPS") == "1", "nur mit installierten Abhängigkeiten (CI)")
    def test_sitzung_aus_dienstkonto(self):
        # Erfundenes Dienstkonto mit frisch erzeugtem Schlüssel; es wird nichts gesendet.
        from cryptography.hazmat.primitives import serialization
        from cryptography.hazmat.primitives.asymmetric import rsa
        from google.auth.transport.requests import AuthorizedSession

        pem = (
            rsa.generate_private_key(public_exponent=65537, key_size=2048)
            .private_bytes(serialization.Encoding.PEM, serialization.PrivateFormat.PKCS8, serialization.NoEncryption())
            .decode()
        )
        info = {
            "type": "service_account",
            "project_id": "beispiel",
            "private_key_id": "0",
            "private_key": pem,
            "client_email": "upload@beispiel.iam.gserviceaccount.com",
            "client_id": "0",
            "token_uri": "https://oauth2.googleapis.com/token",
        }
        with mock.patch.dict(os.environ, {"PLAY_SERVICE_ACCOUNT_JSON": json.dumps(info)}):
            s = pu.sitzung_aus_umgebung()
        self.assertIsInstance(s, AuthorizedSession)
        self.assertEqual(s.credentials._scopes, [pu.SCOPE])


if __name__ == "__main__":
    unittest.main()
