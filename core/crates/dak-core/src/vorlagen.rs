// SPDX-License-Identifier: MPL-2.0
//! Mitgelieferte Fragenvorlage (M1a: feste Vorlage, #22, #75, #105). Die Daten stehen in
//! `templates/standard-v1.json` (CC BY 4.0). IDs entstehen deterministisch aus dem Schlüssel,
//! damit feste Fragen wie die Stimmung (Auslöser für den Krisenplan, #71) überall dieselbe
//! Messreihe bilden.

use serde::Deserialize;

use crate::fehler::{Ergebnis, Fehler};
use crate::modell::{
    BedingungArt, Bezugsart, Feldtyp, Formular, FormularArt, Gruppe, Id, Klasse, Messdef, Option_,
    Polung,
};
use crate::sodium;

const QUELLE: &str = include_str!("../../../../templates/standard-v1.json");

pub const SEP_MESSDEF: &[u8] = b"dak-messdef-v1";
pub const SEP_FORMULAR: &[u8] = b"dak-formular-v1";

pub const STIMMUNG: &str = "stimmung";
pub const TAG_MARKER: &str = "tag_marker";
pub const EIGENE_MARKER_AB: u16 = 11;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Datei {
    #[allow(dead_code)]
    lizenz: String,
    version: u32,
    #[allow(dead_code)]
    hinweis: String,
    skalen: std::collections::BTreeMap<String, Vec<(i64, String)>>,
    messdefs: Vec<MessdefQ>,
    formulare: Vec<FormularQ>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MessdefQ {
    schluessel: String,
    frage: String,
    feldtyp: String,
    min: Option<i64>,
    max: Option<i64>,
    #[serde(default)]
    anker: Vec<(i64, String)>,
    skala: Option<String>,
    #[serde(default)]
    optionen: Vec<(u16, String)>,
    #[serde(default)]
    mit_staerke: bool,
    polung: String,
    bezug: String,
    #[serde(default)]
    kam_nicht_vor: bool,
    einheit: Option<String>,
    hinweis: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GruppeQ {
    titel: String,
    hinweis: Option<String>,
    fragen: Vec<String>,
    #[serde(default)]
    zugeklappt: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormularQ {
    schluessel: String,
    name: String,
    art: String,
    reihenfolge: u32,
    gruppen: Vec<GruppeQ>,
    bedingungen: Vec<(String, String)>,
}

pub fn messdef_id(schluessel: &str) -> Id {
    sodium::hash16(&[SEP_MESSDEF, schluessel.as_bytes()])
}

pub fn formular_id(schluessel: &str) -> Id {
    sodium::hash16(&[SEP_FORMULAR, schluessel.as_bytes()])
}

pub struct Vorlage {
    pub version: u32,
    pub messdefs: Vec<Messdef>,
    pub formulare: Vec<Formular>,
}

pub fn standard() -> Ergebnis<Vorlage> {
    let d: Datei = serde_json::from_str(QUELLE).map_err(|_| Fehler::Format("Vorlage"))?;
    let mut messdefs = Vec::new();
    for q in d.messdefs {
        let feldtyp = match q.feldtyp.as_str() {
            "skala" => Feldtyp::Skala,
            "stufen" => Feldtyp::Stufen,
            "einfach" => Feldtyp::Einfachauswahl,
            "mehrfach" => Feldtyp::Mehrfachauswahl,
            "text" => Feldtyp::Text,
            "dauer" => Feldtyp::Dauer,
            "uhrzeit" => Feldtyp::Uhrzeit,
            "zahl" => Feldtyp::Zahl,
            "janein" => Feldtyp::JaNein,
            _ => return Err(Fehler::Format("Vorlage: Feldtyp")),
        };
        let mut anker = q.anker;
        let (mut min, mut max) = (q.min, q.max);
        if let Some(s) = &q.skala {
            anker = d
                .skalen
                .get(s)
                .cloned()
                .ok_or(Fehler::Format("Vorlage: Skala"))?;
            min = anker.first().map(|a| a.0);
            max = anker.last().map(|a| a.0);
        }
        if feldtyp == Feldtyp::Stufen && (min.is_none() || max.is_none()) {
            return Err(Fehler::Format("Vorlage: Stufen ohne Grenzen"));
        }
        messdefs.push(Messdef {
            id: messdef_id(&q.schluessel),
            schluessel: q.schluessel,
            frage: q.frage,
            feldtyp,
            min,
            max,
            anker,
            optionen: q
                .optionen
                .into_iter()
                .map(|(id, text)| Option_ { id, text })
                .collect(),
            polung: match q.polung.as_str() {
                "keine" => Polung::Keine,
                "hoeher_besser" => Polung::HoeherBesser,
                "hoeher_mehr" => Polung::HoeherMehr,
                "bipolar" => Polung::Bipolar,
                _ => return Err(Fehler::Format("Vorlage: Polung")),
            },
            bezugsart: match q.bezug.as_str() {
                "jetzt" => Bezugsart::Jetzt,
                "seit_letztem" => Bezugsart::SeitLetztem,
                "tag" => Bezugsart::Tag,
                "woche" => Bezugsart::Woche,
                "ereignis" => Bezugsart::Ereignis,
                _ => return Err(Fehler::Format("Vorlage: Bezug")),
            },
            klasse: Klasse::Gesundheit,
            kam_nicht_vor: q.kam_nicht_vor,
            mit_staerke: q.mit_staerke,
            einheit: q.einheit,
            hinweis: q.hinweis,
        });
    }
    let mut formulare = Vec::new();
    for f in d.formulare {
        let id_von = |s: &str| -> Ergebnis<Id> {
            if messdefs.iter().any(|m| m.schluessel == s) {
                Ok(messdef_id(s))
            } else {
                Err(Fehler::Format("Vorlage: unbekannte Frage"))
            }
        };
        let mut gruppen = Vec::new();
        for g in f.gruppen {
            gruppen.push(Gruppe {
                titel: g.titel,
                hinweis: g.hinweis,
                messdefs: g
                    .fragen
                    .iter()
                    .map(|s| id_von(s))
                    .collect::<Ergebnis<_>>()?,
                zugeklappt: g.zugeklappt,
            });
        }
        let mut bedingungen = Vec::new();
        for (frage, art) in f.bedingungen {
            bedingungen.push((
                id_von(&frage)?,
                match art.as_str() {
                    "einnahme_mit_wirkung_heute" => BedingungArt::EinnahmeMitWirkungHeute,
                    "zwei_stunden_seit_letztem" => BedingungArt::ZweiStundenSeitLetztem,
                    _ => return Err(Fehler::Format("Vorlage: Bedingung")),
                },
            ));
        }
        formulare.push(Formular {
            id: formular_id(&f.schluessel),
            schluessel: f.schluessel,
            name: f.name,
            art: match f.art.as_str() {
                "status_jetzt" => FormularArt::StatusJetzt,
                "tagesabschluss" => FormularArt::Tagesabschluss,
                "wochenrueckblick" => FormularArt::Wochenrueckblick,
                "ereignis" => FormularArt::Ereignis,
                _ => return Err(Fehler::Format("Vorlage: Formularart")),
            },
            gruppen,
            eingeschaltet: true,
            bedingungen,
            reihenfolge: f.reihenfolge,
        });
    }
    Ok(Vorlage {
        version: d.version,
        messdefs,
        formulare,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vorlage_ist_gueltig_und_vollstaendig() {
        let v = standard().unwrap();
        assert_eq!(v.messdefs.len(), 51);
        let st = v
            .messdefs
            .iter()
            .find(|m| m.schluessel == STIMMUNG)
            .unwrap();
        assert_eq!(
            (st.min, st.max, st.polung),
            (Some(-5), Some(5), Polung::Bipolar)
        );
        // Eine Stimmungsfrage für Status und Tagesabschluss (#105: kein zweites Konstrukt).
        let mit_stimmung = v
            .formulare
            .iter()
            .filter(|f| f.gruppen.iter().any(|g| g.messdefs.contains(&st.id)))
            .count();
        assert_eq!(mit_stimmung, 2);
        // Jede Messdefinition lässt sich kodieren und wieder lesen.
        for m in &v.messdefs {
            assert_eq!(&Messdef::aus(m.wert()).unwrap(), m);
        }
        for f in &v.formulare {
            assert_eq!(&Formular::aus(f.wert()).unwrap(), f);
        }
        // Keine Substanzfelder in M1a.
        assert!(v.messdefs.iter().all(|m| m.klasse != Klasse::Hochsensibel));
    }
}
