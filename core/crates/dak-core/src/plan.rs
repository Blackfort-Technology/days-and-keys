// SPDX-License-Identifier: MPL-2.0
//! Lokaler Planer für Erinnerungen (Protokoll 11, AE-24, #42). Er plant nur einige Tage voraus,
//! in Ortszeit; die Plattform rechnet mit ihrer Zeitzonendatenbank in Zeitpunkte um. Texte sind
//! neutral: keine Präparatnamen, keine Werte (#69).

use crate::api::Benachrichtigung;
use crate::fehler::{Ergebnis, Fehler};
use crate::kern::Kern;
use crate::modell::{ErinnerungArt, Objekt, hex};
use crate::zeit;

pub(crate) fn planen(k: &Kern, tage: u32, max_gesamt: u32) -> Ergebnis<Vec<Benachrichtigung>> {
    let s = k.sitzung()?;
    if tage == 0 || tage > 14 || max_gesamt == 0 {
        return Err(Fehler::Eingabe("Planungszeitraum"));
    }
    let jetzt = k.jetzt();
    let heute_lokal = jetzt.lokal_minuten().div_euclid(zeit::MINUTEN_TAG);
    let jetzt_min = jetzt.uhrzeit_min();
    let max_tag = s.bund.einstellungen.max_benachrichtigungen as usize;
    let mut alle = Vec::new();
    for d in 0..tage as i64 {
        let tag = heute_lokal + d;
        let bit = 1u8 << zeit::wochentag(tag);
        // (Vorrang, Minute, Eintrag): Medikamente zuerst.
        let mut tag_liste: Vec<(u8, i32, Benachrichtigung)> = Vec::new();
        for p in s.bund.praeparate.iter().filter(|p| p.aktiv && p.erinnerung) {
            for (i, z) in p.plan.iter().enumerate() {
                if z.wochentage & bit != 0 {
                    tag_liste.push((
                        0,
                        z.uhrzeit_min,
                        Benachrichtigung {
                            schluessel: format!(
                                "med-{}-{}-{}",
                                &hex(&p.id)[..8],
                                i,
                                zeit::iso_datum(tag)
                            ),
                            datum_iso: zeit::iso_datum(tag),
                            uhrzeit_min: z.uhrzeit_min,
                            kanal: "medikament".into(),
                            titel: "Erinnerung".into(),
                            text: "Zeit für die geplante Einnahme.".into(),
                            ziel: "einnahme".into(),
                        },
                    ));
                }
            }
        }
        // Status einige Zeit nach einer heute bestätigten Einnahme.
        if d == 0 {
            for e in &s.eintraege {
                if let Objekt::Ereignis(ev) = &e.objekt
                    && ev.art == "einnahme"
                    && ev.zeitpunkt.lokal_minuten().div_euclid(zeit::MINUTEN_TAG) == tag
                    && let Some(p) = ev
                        .praeparat
                        .and_then(|p| s.bund.praeparate.iter().find(|x| x.id == p))
                    && let Some(n) = p.status_nach_min
                {
                    let m = ev.zeitpunkt.uhrzeit_min() + n as i32;
                    if m < 1440 {
                        tag_liste.push((
                            1,
                            m,
                            Benachrichtigung {
                                schluessel: format!("nach-{}", &hex(&e.datensatz_id)[..12]),
                                datum_iso: zeit::iso_datum(tag),
                                uhrzeit_min: m,
                                kanal: "erinnerung".into(),
                                titel: "Erinnerung".into(),
                                text: "Wie geht es gerade? Ein kurzer Status genügt.".into(),
                                ziel: "status_jetzt".into(),
                            },
                        ));
                    }
                }
            }
        }
        for (i, e) in s.bund.einstellungen.erinnerungen.iter().enumerate() {
            if !e.aktiv || e.wochentage & bit == 0 {
                continue;
            }
            let (ziel, text) = match e.art {
                ErinnerungArt::Status => (
                    "status_jetzt",
                    "Wie geht es gerade? Ein kurzer Status genügt.",
                ),
                ErinnerungArt::Tagesabschluss => (
                    "tagesabschluss",
                    "Zeit für den Tagesabschluss, wenn du magst.",
                ),
                ErinnerungArt::Wochenrueckblick => (
                    "wochenrueckblick",
                    "Zeit für den Wochenrückblick, wenn du magst.",
                ),
            };
            tag_liste.push((
                2,
                e.uhrzeit_min,
                Benachrichtigung {
                    schluessel: format!("eri-{}-{}", i, zeit::iso_datum(tag)),
                    datum_iso: zeit::iso_datum(tag),
                    uhrzeit_min: e.uhrzeit_min,
                    kanal: "erinnerung".into(),
                    titel: "Erinnerung".into(),
                    text: text.into(),
                    ziel: ziel.into(),
                },
            ));
        }
        if d == 0 {
            tag_liste.retain(|(_, m, _)| *m > jetzt_min);
        }
        tag_liste.sort_by_key(|(v, m, _)| (*v, *m));
        tag_liste.truncate(max_tag);
        tag_liste.sort_by_key(|(_, m, _)| *m);
        alle.extend(tag_liste.into_iter().map(|(_, _, b)| b));
    }
    alle.truncate(max_gesamt as usize);
    Ok(alle)
}
