// SPDX-License-Identifier: MPL-2.0
//! Typisierte Fehler mit stabilen Codes (Architektur Abschnitt 7). Kein Fehlertext enthält
//! Inhalte, Schlüssel oder Feldnamen; die Oberfläche zeigt einen verständlichen Text und den Code.

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Fehler {
    #[error("Formatfehler ({0})")]
    Format(&'static str),
    #[error("Integritätsprüfung fehlgeschlagen ({0})")]
    Integritaet(&'static str),
    #[error("Kontokette ungültig ({0})")]
    Kette(&'static str),
    #[error("Die Recovery-Wörter passen nicht")]
    Woerter,
    #[error("Die App ist gesperrt")]
    Gesperrt,
    #[error("Die App ist noch nicht eingerichtet")]
    NichtEingerichtet,
    #[error("Die App ist schon eingerichtet")]
    SchonEingerichtet,
    #[error("Unzulässige Eingabe ({0})")]
    Eingabe(&'static str),
    #[error("Bestätigung fehlt oder passt nicht zur Aktion")]
    Bestaetigung,
    #[error("Nicht gefunden ({0})")]
    NichtGefunden(&'static str),
    #[error("Ablauf passt nicht ({0})")]
    Zustand(&'static str),
    #[error("Speicherfehler ({0})")]
    Speicher(String),
    #[error("Unbekannte Formatversion {0}: bitte die App aktualisieren")]
    Version(u64),
}

impl Fehler {
    /// Stabiler Code für Rückmeldungen ohne Inhalte.
    pub fn code(&self) -> u32 {
        match self {
            Fehler::Format(_) => 100,
            Fehler::Integritaet(_) => 200,
            Fehler::Kette(_) => 300,
            Fehler::Woerter => 310,
            Fehler::Gesperrt => 400,
            Fehler::NichtEingerichtet => 410,
            Fehler::SchonEingerichtet => 411,
            Fehler::Eingabe(_) => 500,
            Fehler::Bestaetigung => 510,
            Fehler::NichtGefunden(_) => 520,
            Fehler::Zustand(_) => 530,
            Fehler::Speicher(_) => 600,
            Fehler::Version(_) => 700,
        }
    }
}

impl From<rusqlite::Error> for Fehler {
    fn from(e: rusqlite::Error) -> Self {
        // Nur die Fehlerart, keine Werte aus der Anfrage.
        Fehler::Speicher(match e {
            rusqlite::Error::SqliteFailure(f, _) => format!("{:?}", f.code),
            rusqlite::Error::QueryReturnedNoRows => "keine Zeile".into(),
            _ => "sqlite".into(),
        })
    }
}

pub type Ergebnis<T> = Result<T, Fehler>;
