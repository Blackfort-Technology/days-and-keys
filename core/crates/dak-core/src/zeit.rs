// SPDX-License-Identifier: MPL-2.0
//! Zeitmodell (Protokoll 6.3): UTC plus Versatz plus IANA-Zone, Lebenstag mit eigenem
//! Tageswechsel. Die Zeitzonenregeln kennt die Plattform; der Kern bekommt den Versatz zum
//! Zeitpunkt der Erfassung mitgeliefert und rechnet damit. So verschiebt kein späterer
//! Wechsel der Zeitzone oder der Sommerzeit einen gespeicherten Eintrag.

use crate::cbor::{self, MapBau, MapLeser, Wert};
use crate::fehler::{Ergebnis, Fehler};

pub const MINUTEN_TAG: i64 = 24 * 60;
pub const TAGESWECHSEL_STANDARD: i32 = 4 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Zeitpunkt {
    pub utc_ms: i64,
    pub versatz_min: i32,
    pub zone: String,
}

impl Zeitpunkt {
    pub fn wert(&self) -> Wert {
        MapBau::neu()
            .mit(0, cbor::zahl(self.utc_ms))
            .mit(1, cbor::zahl(self.versatz_min))
            .mit(2, cbor::text(&self.zone))
            .fertig()
    }

    pub fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        let z = Zeitpunkt {
            utc_ms: m.i64(0)?,
            versatz_min: i32::try_from(m.i64(1)?).map_err(|_| Fehler::Format("Versatz"))?,
            zone: m.text(2)?,
        };
        m.ende()?;
        z.pruefen()?;
        Ok(z)
    }

    pub fn pruefen(&self) -> Ergebnis<()> {
        if !(-18 * 60..=18 * 60).contains(&self.versatz_min) || self.zone.len() > 64 {
            return Err(Fehler::Eingabe("Zeitzone"));
        }
        Ok(())
    }

    /// Lokale Minuten seit 1970-01-01 00:00 Ortszeit.
    pub fn lokal_minuten(&self) -> i64 {
        self.utc_ms.div_euclid(60_000) + self.versatz_min as i64
    }

    /// Lebenstag als Tagesnummer seit 1970-01-01 (Ortszeit, nach dem Tageswechsel).
    pub fn lebenstag(&self, tageswechsel_min: i32) -> i64 {
        (self.lokal_minuten() - tageswechsel_min as i64).div_euclid(MINUTEN_TAG)
    }

    /// Uhrzeit in Minuten nach Mitternacht (Ortszeit).
    pub fn uhrzeit_min(&self) -> i32 {
        self.lokal_minuten().rem_euclid(MINUTEN_TAG) as i32
    }
}

/// Tagesnummer aus Kalenderdatum (Algorithmus von Howard Hinnant).
pub fn tag_aus_datum(jahr: i64, monat: u32, tag: u32) -> i64 {
    let y = if monat <= 2 { jahr - 1 } else { jahr };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = monat as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + tag as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Kalenderdatum aus Tagesnummer.
pub fn datum_aus_tag(t: i64) -> (i64, u32, u32) {
    let z = t + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Wochentag: 0 = Montag … 6 = Sonntag.
pub fn wochentag(t: i64) -> u32 {
    // 1970-01-01 war ein Donnerstag (3).
    (t + 3).rem_euclid(7) as u32
}

pub fn datum_text(t: i64) -> String {
    let (j, m, d) = datum_aus_tag(t);
    format!("{d:02}.{m:02}.{j}")
}

/// Wochentag und Datum, z. B. „Donnerstag, 15. Oktober“ (Startseite, #134).
pub fn datum_lang(t: i64) -> String {
    const TAGE: [&str; 7] = [
        "Montag",
        "Dienstag",
        "Mittwoch",
        "Donnerstag",
        "Freitag",
        "Samstag",
        "Sonntag",
    ];
    const MONATE: [&str; 12] = [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ];
    let (_, m, d) = datum_aus_tag(t);
    // Tag 0 (1.1.1970) war ein Donnerstag.
    format!(
        "{}, {}. {}",
        TAGE[(t + 3).rem_euclid(7) as usize],
        d,
        MONATE[(m - 1) as usize]
    )
}

pub fn iso_datum(t: i64) -> String {
    let (j, m, d) = datum_aus_tag(t);
    format!("{j:04}-{m:02}-{d:02}")
}

pub fn uhrzeit_text(min: i32) -> String {
    format!("{:02}:{:02}", min.div_euclid(60), min.rem_euclid(60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datum_lang_mit_wochentag() {
        assert_eq!(
            datum_lang(tag_aus_datum(2026, 10, 15)),
            "Donnerstag, 15. Oktober"
        );
        assert_eq!(datum_lang(tag_aus_datum(2026, 3, 1)), "Sonntag, 1. März");
        assert_eq!(
            datum_lang(tag_aus_datum(1970, 1, 1)),
            "Donnerstag, 1. Januar"
        );
    }

    fn zp(utc_ms: i64, versatz: i32) -> Zeitpunkt {
        Zeitpunkt {
            utc_ms,
            versatz_min: versatz,
            zone: "Europe/Berlin".into(),
        }
    }

    #[test]
    fn datum_rundweg() {
        for t in [-1000i64, 0, 1, 365, 10_957, 20_000, 20_740, 30_000] {
            let (j, m, d) = datum_aus_tag(t);
            assert_eq!(tag_aus_datum(j, m, d), t);
        }
        assert_eq!(tag_aus_datum(1970, 1, 1), 0);
        assert_eq!(tag_aus_datum(2000, 3, 1), 11_017);
        assert_eq!(iso_datum(tag_aus_datum(2026, 10, 14)), "2026-10-14");
        assert_eq!(wochentag(tag_aus_datum(2026, 10, 14)), 2); // Mittwoch
    }

    #[test]
    fn lebenstag_mit_tageswechsel() {
        let tag = tag_aus_datum(2026, 10, 14);
        // 14.10.2026 01:30 Ortszeit (UTC+2) gehört bei Tageswechsel 4:00 zum 13.10.
        let utc = (tag * MINUTEN_TAG + 90 - 120) * 60_000;
        assert_eq!(zp(utc, 120).lebenstag(TAGESWECHSEL_STANDARD), tag - 1);
        assert_eq!(zp(utc, 120).lebenstag(0), tag);
        // 04:00 Ortszeit gehört zum neuen Tag
        let utc4 = (tag * MINUTEN_TAG + 240 - 120) * 60_000;
        assert_eq!(zp(utc4, 120).lebenstag(TAGESWECHSEL_STANDARD), tag);
        assert_eq!(zp(utc4, 120).uhrzeit_min(), 240);
    }

    #[test]
    fn sommerzeitwechsel_verschiebt_nichts() {
        // 25.10.2026: Ende der Sommerzeit. Zwei Status zur selben UTC-Zeit, einmal mit altem,
        // einmal mit neuem Versatz erfasst, behalten ihren gespeicherten Versatz.
        let a = zp(1_793_000_000_000, 120);
        let b = zp(1_793_000_000_000, 60);
        assert_eq!(a.utc_ms, b.utc_ms);
        assert_eq!(a.uhrzeit_min() - b.uhrzeit_min(), 60);
    }

    #[test]
    fn zeitpunkt_cbor() {
        let z = zp(-5, -90);
        assert_eq!(Zeitpunkt::aus(z.wert()).unwrap(), z);
        assert!(
            Zeitpunkt::aus(
                MapBau::neu()
                    .mit(0, cbor::zahl(1))
                    .mit(1, cbor::zahl(9999))
                    .mit(2, cbor::text("x"))
                    .fertig()
            )
            .is_err()
        );
    }
}
