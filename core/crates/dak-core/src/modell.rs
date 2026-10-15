// SPDX-License-Identifier: MPL-2.0
//! Fachliche Objekte (Protokoll 5 und 6) und ihre CBOR-Form. Feldtabellen in `docs/format.md`.

use crate::cbor::{self, MapBau, MapLeser, Wert, als_feld};
use crate::fehler::{Ergebnis, Fehler};
use crate::zeit::Zeitpunkt;

pub const FORMATVERSION: u64 = 1;
pub const SUITE_S1: u64 = 1;

pub type Id = [u8; 16];

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn id_aus_hex(s: &str) -> Ergebnis<Id> {
    if s.len() != 32 {
        return Err(Fehler::Eingabe("ID-Länge"));
    }
    let mut out = [0u8; 16];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
            .map_err(|_| Fehler::Eingabe("ID-Zeichen"))?;
    }
    Ok(out)
}

/// Typkennungen (Feld 0 jedes Objekts).
pub mod typ {
    pub const GRUENDUNG: u64 = 1;
    pub const GLIED: u64 = 2;
    pub const BUND: u64 = 3;
    pub const BUND_H: u64 = 4;
    pub const DATENSATZ: u64 = 5;
    pub const KAPSEL: u64 = 6;
    pub const KAPSEL_H: u64 = 7;
    pub const SICHERUNG_INNEN: u64 = 8;
    pub const MANIFEST: u64 = 9;
    pub const STATUS: u64 = 10;
    pub const EREIGNIS: u64 = 11;
    pub const ABFRAGE: u64 = 12;
    pub const MESSDEF: u64 = 13;
    pub const FORMULAR: u64 = 14;
    pub const PRAEPARAT: u64 = 15;
    pub const EINSTELLUNGEN: u64 = 16;
    pub const ENTWURF: u64 = 17;
    pub const UMHUELLUNG: u64 = 19;
    pub const SICHERUNG_KOPF: u64 = 20;
    pub const ERSETZUNGSMARKER: u64 = 21;
    pub const BUND_BLOB: u64 = 22;
}

/// Prüft Typ und Formatversion am Anfang eines Objekts.
pub fn kopf_pruefen(m: &mut MapLeser, erwartet: u64) -> Ergebnis<()> {
    if m.u64(0)? != erwartet {
        return Err(Fehler::Format("falscher Typ"));
    }
    let v = m.u64(1)?;
    if v != FORMATVERSION {
        return Err(Fehler::Version(v));
    }
    Ok(())
}

pub fn kopf(t: u64) -> MapBau {
    MapBau::neu()
        .mit(0, cbor::zahl(t))
        .mit(1, cbor::zahl(FORMATVERSION))
}

fn u8_feld(m: &mut MapLeser, k: u64) -> Ergebnis<u8> {
    u8::try_from(m.u64(k)?).map_err(|_| Fehler::Format("Wert außerhalb"))
}

fn ids(f: &[Id]) -> Wert {
    Wert::Feld(f.iter().map(|i| cbor::bytes(i)).collect())
}

fn ids_aus(w: Wert) -> Ergebnis<Vec<Id>> {
    als_feld(w)?
        .into_iter()
        .map(|x| {
            cbor::als_bytes(x)?
                .try_into()
                .map_err(|_| Fehler::Format("ID-Länge"))
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Schutzklassen, Feldtypen
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Klasse {
    Normal = 0,
    Gesundheit = 1,
    Hochsensibel = 2,
}

impl Klasse {
    pub fn aus(v: u8) -> Ergebnis<Self> {
        Ok(match v {
            0 => Klasse::Normal,
            1 => Klasse::Gesundheit,
            2 => Klasse::Hochsensibel,
            _ => return Err(Fehler::Format("Klasse")),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feldtyp {
    /// Ganze Zahl zwischen min und max, als Tipp-Reihe ohne Vorauswahl.
    Skala = 0,
    /// Fünferskala mit eigenen Ankern (z. B. nie … sehr oft).
    Stufen = 1,
    Einfachauswahl = 2,
    Mehrfachauswahl = 3,
    Text = 4,
    /// Dauer in Minuten.
    Dauer = 5,
    /// Uhrzeit in Minuten nach Mitternacht.
    Uhrzeit = 6,
    Zahl = 7,
    JaNein = 8,
}

impl Feldtyp {
    pub fn aus(v: u8) -> Ergebnis<Self> {
        Ok(match v {
            0 => Feldtyp::Skala,
            1 => Feldtyp::Stufen,
            2 => Feldtyp::Einfachauswahl,
            3 => Feldtyp::Mehrfachauswahl,
            4 => Feldtyp::Text,
            5 => Feldtyp::Dauer,
            6 => Feldtyp::Uhrzeit,
            7 => Feldtyp::Zahl,
            8 => Feldtyp::JaNein,
            _ => return Err(Fehler::Format("Feldtyp")),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polung {
    Keine = 0,
    HoeherBesser = 1,
    HoeherMehr = 2,
    /// Bipolar um 0 (z. B. Stimmung -5 bis +5).
    Bipolar = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bezugsart {
    Jetzt = 0,
    SeitLetztem = 1,
    Tag = 2,
    Woche = 3,
    Ereignis = 4,
}

// ---------------------------------------------------------------------------------------------
// Messdefinition (Protokoll 5)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Option_ {
    pub id: u16,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Messdef {
    pub id: Id,
    pub schluessel: String,
    pub frage: String,
    pub feldtyp: Feldtyp,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub anker: Vec<(i64, String)>,
    pub optionen: Vec<Option_>,
    pub polung: Polung,
    pub bezugsart: Bezugsart,
    pub klasse: Klasse,
    pub kam_nicht_vor: bool,
    pub mit_staerke: bool,
    pub einheit: Option<String>,
    pub hinweis: Option<String>,
    /// Kurzname für Listen („Stimmung“), nur Anzeige (#135). Feld 18, fehlt ohne Kurznamen.
    pub kurz: Option<String>,
}

impl Messdef {
    pub fn wert(&self) -> Wert {
        kopf(typ::MESSDEF)
            .mit(3, cbor::bytes(&self.id))
            .mit(4, cbor::text(&self.schluessel))
            .mit(5, cbor::text(&self.frage))
            .mit(6, cbor::zahl(self.feldtyp as u8))
            .opt(7, self.min.map(cbor::zahl))
            .opt(8, self.max.map(cbor::zahl))
            .mit(
                9,
                Wert::Feld(
                    self.anker
                        .iter()
                        .map(|(w, t)| {
                            MapBau::neu()
                                .mit(0, cbor::zahl(*w))
                                .mit(1, cbor::text(t))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .mit(
                10,
                Wert::Feld(
                    self.optionen
                        .iter()
                        .map(|o| {
                            MapBau::neu()
                                .mit(0, cbor::zahl(o.id))
                                .mit(1, cbor::text(&o.text))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .mit(11, cbor::zahl(self.polung as u8))
            .mit(12, cbor::zahl(self.bezugsart as u8))
            .mit(13, cbor::zahl(self.klasse as u8))
            .mit(14, Wert::Bool(self.kam_nicht_vor))
            .mit(15, Wert::Bool(self.mit_staerke))
            .opt(16, self.einheit.as_deref().map(cbor::text))
            .opt(17, self.hinweis.as_deref().map(cbor::text))
            .opt(18, self.kurz.as_deref().map(cbor::text))
            .fertig()
    }

    pub fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        kopf_pruefen(&mut m, typ::MESSDEF)?;
        let id = m.bytes_n(3)?;
        let schluessel = m.text(4)?;
        let frage = m.text(5)?;
        let feldtyp = Feldtyp::aus(u8_feld(&mut m, 6)?)?;
        let min = m.opt_i64(7)?;
        let max = m.opt_i64(8)?;
        let mut anker = Vec::new();
        for a in m.feld(9)? {
            let mut am = MapLeser::aus(a)?;
            anker.push((am.i64(0)?, am.text(1)?));
            am.ende()?;
        }
        let mut optionen = Vec::new();
        for o in m.feld(10)? {
            let mut om = MapLeser::aus(o)?;
            optionen.push(Option_ {
                id: u16::try_from(om.u64(0)?).map_err(|_| Fehler::Format("Option"))?,
                text: om.text(1)?,
            });
            om.ende()?;
        }
        let polung = match u8_feld(&mut m, 11)? {
            0 => Polung::Keine,
            1 => Polung::HoeherBesser,
            2 => Polung::HoeherMehr,
            3 => Polung::Bipolar,
            _ => return Err(Fehler::Format("Polung")),
        };
        let bezugsart = match u8_feld(&mut m, 12)? {
            0 => Bezugsart::Jetzt,
            1 => Bezugsart::SeitLetztem,
            2 => Bezugsart::Tag,
            3 => Bezugsart::Woche,
            4 => Bezugsart::Ereignis,
            _ => return Err(Fehler::Format("Bezugsart")),
        };
        let klasse = Klasse::aus(u8_feld(&mut m, 13)?)?;
        let kam_nicht_vor = m.bool(14)?;
        let mit_staerke = m.bool(15)?;
        let einheit = m.opt_text(16)?;
        let hinweis = m.opt_text(17)?;
        let kurz = m.opt_text(18)?;
        m.ende()?;
        Ok(Messdef {
            id,
            schluessel,
            frage,
            feldtyp,
            min,
            max,
            anker,
            optionen,
            polung,
            bezugsart,
            klasse,
            kam_nicht_vor,
            mit_staerke,
            einheit,
            hinweis,
            kurz,
        })
    }

    /// Prüft einen Antwortwert gegen die Definition.
    pub fn wert_pruefen(&self, w: &Antwortwert) -> Ergebnis<()> {
        match self.feldtyp {
            Feldtyp::Skala
            | Feldtyp::Stufen
            | Feldtyp::Zahl
            | Feldtyp::Dauer
            | Feldtyp::Uhrzeit => {
                let z = w.zahl.ok_or(Fehler::Eingabe("Zahl fehlt"))?;
                if self.min.is_some_and(|m| z < m) || self.max.is_some_and(|m| z > m) {
                    return Err(Fehler::Eingabe("Wert außerhalb der Skala"));
                }
                if self.feldtyp == Feldtyp::Uhrzeit && !(0..1440).contains(&z) {
                    return Err(Fehler::Eingabe("Uhrzeit"));
                }
            }
            Feldtyp::JaNein => {
                if !matches!(w.zahl, Some(0) | Some(1)) {
                    return Err(Fehler::Eingabe("Ja/Nein"));
                }
            }
            Feldtyp::Text => {
                let t = w.text.as_ref().ok_or(Fehler::Eingabe("Text fehlt"))?;
                if t.len() > 20_000 {
                    return Err(Fehler::Eingabe("Text zu lang"));
                }
            }
            Feldtyp::Einfachauswahl | Feldtyp::Mehrfachauswahl => {
                if w.auswahl.is_empty()
                    || (self.feldtyp == Feldtyp::Einfachauswahl && w.auswahl.len() != 1)
                {
                    return Err(Fehler::Eingabe("Auswahl"));
                }
                for (o, s) in &w.auswahl {
                    if !self.optionen.iter().any(|x| x.id == *o) {
                        return Err(Fehler::Eingabe("unbekannte Option"));
                    }
                    if s.is_some() && !self.mit_staerke {
                        return Err(Fehler::Eingabe("Stärke nicht vorgesehen"));
                    }
                    if s.is_some_and(|s| !(0..=10).contains(&s)) {
                        return Err(Fehler::Eingabe("Stärke"));
                    }
                }
                let mut gesehen: Vec<u16> = w.auswahl.iter().map(|(o, _)| *o).collect();
                gesehen.sort_unstable();
                gesehen.dedup();
                if gesehen.len() != w.auswahl.len() {
                    return Err(Fehler::Eingabe("doppelte Option"));
                }
            }
        }
        if self.feldtyp != Feldtyp::Text && w.text.as_ref().is_some_and(|t| t.len() > 2000) {
            return Err(Fehler::Eingabe("Notiz zu lang"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// Formular
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormularArt {
    StatusJetzt = 0,
    Tagesabschluss = 1,
    Wochenrueckblick = 2,
    Ereignis = 3,
}

impl FormularArt {
    pub fn aus(v: u8) -> Ergebnis<Self> {
        Ok(match v {
            0 => FormularArt::StatusJetzt,
            1 => FormularArt::Tagesabschluss,
            2 => FormularArt::Wochenrueckblick,
            3 => FormularArt::Ereignis,
            _ => return Err(Fehler::Format("Formularart")),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gruppe {
    pub titel: String,
    pub hinweis: Option<String>,
    pub messdefs: Vec<Id>,
    pub zugeklappt: bool,
    /// Nur Anzeige (#134): anfangs zugeklappt, auf Tipp aufklappbar. Feld 4, fehlt wenn falsch.
    pub aufklappbar: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedingungArt {
    /// Nur zeigen, wenn heute eine Einnahme eines Präparats mit spürbarer Wirkung bestätigt ist.
    EinnahmeMitWirkungHeute = 1,
    /// Gruppe erst ab zwei Stunden Abstand zum letzten Status aufgeklappt anbieten.
    ZweiStundenSeitLetztem = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formular {
    pub id: Id,
    pub schluessel: String,
    pub name: String,
    pub art: FormularArt,
    pub gruppen: Vec<Gruppe>,
    pub eingeschaltet: bool,
    pub bedingungen: Vec<(Id, BedingungArt)>,
    /// Ereignisformulare: Symbol- bzw. Knopfreihenfolge auf der Startseite.
    pub reihenfolge: u32,
}

impl Formular {
    pub fn wert(&self) -> Wert {
        kopf(typ::FORMULAR)
            .mit(3, cbor::bytes(&self.id))
            .mit(4, cbor::text(&self.schluessel))
            .mit(5, cbor::text(&self.name))
            .mit(6, cbor::zahl(self.art as u8))
            .mit(
                7,
                Wert::Feld(
                    self.gruppen
                        .iter()
                        .map(|g| {
                            MapBau::neu()
                                .mit(0, cbor::text(&g.titel))
                                .opt(1, g.hinweis.as_deref().map(cbor::text))
                                .mit(2, ids(&g.messdefs))
                                .mit(3, Wert::Bool(g.zugeklappt))
                                .opt(4, g.aufklappbar.then_some(Wert::Bool(true)))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .mit(8, Wert::Bool(self.eingeschaltet))
            .mit(
                9,
                Wert::Feld(
                    self.bedingungen
                        .iter()
                        .map(|(id, a)| {
                            MapBau::neu()
                                .mit(0, cbor::bytes(id))
                                .mit(1, cbor::zahl(*a as u8))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .mit(10, cbor::zahl(self.reihenfolge))
            .fertig()
    }

    pub fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        kopf_pruefen(&mut m, typ::FORMULAR)?;
        let id = m.bytes_n(3)?;
        let schluessel = m.text(4)?;
        let name = m.text(5)?;
        let art = FormularArt::aus(u8_feld(&mut m, 6)?)?;
        let mut gruppen = Vec::new();
        for g in m.feld(7)? {
            let mut gm = MapLeser::aus(g)?;
            gruppen.push(Gruppe {
                titel: gm.text(0)?,
                hinweis: gm.opt_text(1)?,
                messdefs: ids_aus(gm.wert(2)?)?,
                zugeklappt: gm.bool(3)?,
                aufklappbar: match gm.opt_wert(4) {
                    None => false,
                    Some(Wert::Bool(b)) => b,
                    Some(_) => return Err(Fehler::Format("aufklappbar")),
                },
            });
            gm.ende()?;
        }
        let eingeschaltet = m.bool(8)?;
        let mut bedingungen = Vec::new();
        for b in m.feld(9)? {
            let mut bm = MapLeser::aus(b)?;
            let id = bm.bytes_n(0)?;
            let a = match u8_feld(&mut bm, 1)? {
                1 => BedingungArt::EinnahmeMitWirkungHeute,
                2 => BedingungArt::ZweiStundenSeitLetztem,
                _ => return Err(Fehler::Format("Bedingung")),
            };
            bm.ende()?;
            bedingungen.push((id, a));
        }
        let reihenfolge = u32::try_from(m.u64(10)?).map_err(|_| Fehler::Format("Reihenfolge"))?;
        m.ende()?;
        Ok(Formular {
            id,
            schluessel,
            name,
            art,
            gruppen,
            eingeschaltet,
            bedingungen,
            reihenfolge,
        })
    }
}

// ---------------------------------------------------------------------------------------------
// Antworten, Status, Ereignis
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zustand {
    Beantwortet = 0,
    Uebersprungen = 1,
    KamNichtVor = 2,
    NichtGezeigtBedingung = 3,
    Ausgeblendet = 4,
    BlockZu = 5,
}

impl Zustand {
    pub fn aus(v: u8) -> Ergebnis<Self> {
        Ok(match v {
            0 => Zustand::Beantwortet,
            1 => Zustand::Uebersprungen,
            2 => Zustand::KamNichtVor,
            3 => Zustand::NichtGezeigtBedingung,
            4 => Zustand::Ausgeblendet,
            5 => Zustand::BlockZu,
            _ => return Err(Fehler::Format("Zustand")),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Antwortwert {
    pub zahl: Option<i64>,
    pub text: Option<String>,
    /// Option und optional Stärke 0-10.
    pub auswahl: Vec<(u16, Option<i64>)>,
}

impl Antwortwert {
    fn wert(&self) -> Wert {
        MapBau::neu()
            .opt(0, self.zahl.map(cbor::zahl))
            .opt(1, self.text.as_deref().map(cbor::text))
            .opt(
                2,
                (!self.auswahl.is_empty()).then(|| {
                    Wert::Feld(
                        self.auswahl
                            .iter()
                            .map(|(o, s)| {
                                MapBau::neu()
                                    .mit(0, cbor::zahl(*o))
                                    .opt(1, s.map(cbor::zahl))
                                    .fertig()
                            })
                            .collect(),
                    )
                }),
            )
            .fertig()
    }

    fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        let zahl = m.opt_i64(0)?;
        let text = m.opt_text(1)?;
        let mut auswahl = Vec::new();
        if let Some(f) = m.opt_wert(2) {
            for a in als_feld(f)? {
                let mut am = MapLeser::aus(a)?;
                auswahl.push((
                    u16::try_from(am.u64(0)?).map_err(|_| Fehler::Format("Option"))?,
                    am.opt_i64(1)?,
                ));
                am.ende()?;
            }
        }
        m.ende()?;
        Ok(Antwortwert {
            zahl,
            text,
            auswahl,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Antwort {
    pub messdef: Id,
    pub zustand: Zustand,
    pub wert: Option<Antwortwert>,
    pub aktiv_gesetzt: bool,
}

impl Antwort {
    fn wert_cbor(&self) -> Wert {
        MapBau::neu()
            .mit(0, cbor::bytes(&self.messdef))
            .mit(1, cbor::zahl(self.zustand as u8))
            .opt(2, self.wert.as_ref().map(|w| w.wert()))
            .mit(3, Wert::Bool(self.aktiv_gesetzt))
            .fertig()
    }

    fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        let a = Antwort {
            messdef: m.bytes_n(0)?,
            zustand: Zustand::aus(u8_feld(&mut m, 1)?)?,
            wert: m.opt_wert(2).map(Antwortwert::aus).transpose()?,
            aktiv_gesetzt: m.bool(3)?,
        };
        m.ende()?;
        if (a.zustand == Zustand::Beantwortet) != a.wert.is_some() {
            return Err(Fehler::Format("Antwortwert passt nicht zum Zustand"));
        }
        Ok(a)
    }
}

fn antworten_wert(a: &[Antwort]) -> Wert {
    Wert::Feld(a.iter().map(|x| x.wert_cbor()).collect())
}

fn antworten_aus(w: Wert) -> Ergebnis<Vec<Antwort>> {
    als_feld(w)?.into_iter().map(Antwort::aus).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ausloeser {
    Selbst = 0,
    Abfrage = 1,
    NachEreignis = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErfasstUeber {
    App = 0,
    Entwurf = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub id: Id,
    pub zeitpunkt: Zeitpunkt,
    pub erfasst_um: i64,
    pub ausloeser: Ausloeser,
    pub abfrage: Option<Id>,
    pub erfasst_ueber: ErfasstUeber,
    pub vorlaeufig: bool,
    pub bezug_von: Option<i64>,
    pub bezug_bis: Option<i64>,
    pub antworten: Vec<Antwort>,
    pub lebenstag: i64,
    pub formular: Id,
    pub art: FormularArt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ereignis {
    pub id: Id,
    /// Schlüssel des Ereignisformulars, z. B. `einnahme`.
    pub art: String,
    pub formular: Id,
    pub zeitpunkt: Zeitpunkt,
    pub erfasst_um: i64,
    pub lebenstag: i64,
    pub antworten: Vec<Antwort>,
    pub praeparat: Option<Id>,
    pub vorlaeufig: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Objekt {
    Status(Status),
    Ereignis(Ereignis),
}

impl Objekt {
    pub fn wert(&self) -> Wert {
        match self {
            Objekt::Status(s) => kopf(typ::STATUS)
                .mit(3, cbor::bytes(&s.id))
                .mit(4, s.zeitpunkt.wert())
                .mit(5, cbor::zahl(s.erfasst_um))
                .mit(6, cbor::zahl(s.ausloeser as u8))
                .opt(7, s.abfrage.as_ref().map(|a| cbor::bytes(a)))
                .mit(8, cbor::zahl(s.erfasst_ueber as u8))
                .mit(9, Wert::Bool(s.vorlaeufig))
                .opt(10, s.bezug_von.map(cbor::zahl))
                .opt(11, s.bezug_bis.map(cbor::zahl))
                .mit(12, antworten_wert(&s.antworten))
                .mit(13, cbor::zahl(s.lebenstag))
                .mit(14, cbor::bytes(&s.formular))
                .mit(15, cbor::zahl(s.art as u8))
                .fertig(),
            Objekt::Ereignis(e) => kopf(typ::EREIGNIS)
                .mit(3, cbor::bytes(&e.id))
                .mit(4, cbor::text(&e.art))
                .mit(5, cbor::bytes(&e.formular))
                .mit(6, e.zeitpunkt.wert())
                .mit(7, cbor::zahl(e.erfasst_um))
                .mit(8, cbor::zahl(e.lebenstag))
                .mit(9, antworten_wert(&e.antworten))
                .opt(10, e.praeparat.as_ref().map(|p| cbor::bytes(p)))
                .mit(11, Wert::Bool(e.vorlaeufig))
                .fertig(),
        }
    }

    pub fn aus(w: Wert) -> Ergebnis<Self> {
        let t = match &w {
            Wert::Map(m) => m.iter().find(|(k, _)| *k == 0).and_then(|(_, v)| {
                if let Wert::Zahl(z) = v {
                    Some(*z)
                } else {
                    None
                }
            }),
            _ => None,
        }
        .ok_or(Fehler::Format("Objekt ohne Typ"))?;
        let mut m = MapLeser::aus(w)?;
        if t == typ::STATUS as i128 {
            kopf_pruefen(&mut m, typ::STATUS)?;
            let s = Status {
                id: m.bytes_n(3)?,
                zeitpunkt: Zeitpunkt::aus(m.wert(4)?)?,
                erfasst_um: m.i64(5)?,
                ausloeser: match u8_feld(&mut m, 6)? {
                    0 => Ausloeser::Selbst,
                    1 => Ausloeser::Abfrage,
                    2 => Ausloeser::NachEreignis,
                    _ => return Err(Fehler::Format("Auslöser")),
                },
                abfrage: m
                    .opt_bytes(7)?
                    .map(|b| b.try_into().map_err(|_| Fehler::Format("ID-Länge")))
                    .transpose()?,
                erfasst_ueber: match u8_feld(&mut m, 8)? {
                    0 => ErfasstUeber::App,
                    1 => ErfasstUeber::Entwurf,
                    _ => return Err(Fehler::Format("erfasst über")),
                },
                vorlaeufig: m.bool(9)?,
                bezug_von: m.opt_i64(10)?,
                bezug_bis: m.opt_i64(11)?,
                antworten: antworten_aus(m.wert(12)?)?,
                lebenstag: m.i64(13)?,
                formular: m.bytes_n(14)?,
                art: FormularArt::aus(u8_feld(&mut m, 15)?)?,
            };
            m.ende()?;
            Ok(Objekt::Status(s))
        } else if t == typ::EREIGNIS as i128 {
            kopf_pruefen(&mut m, typ::EREIGNIS)?;
            let e = Ereignis {
                id: m.bytes_n(3)?,
                art: m.text(4)?,
                formular: m.bytes_n(5)?,
                zeitpunkt: Zeitpunkt::aus(m.wert(6)?)?,
                erfasst_um: m.i64(7)?,
                lebenstag: m.i64(8)?,
                antworten: antworten_aus(m.wert(9)?)?,
                praeparat: m
                    .opt_bytes(10)?
                    .map(|b| b.try_into().map_err(|_| Fehler::Format("ID-Länge")))
                    .transpose()?,
                vorlaeufig: m.bool(11)?,
            };
            m.ende()?;
            Ok(Objekt::Ereignis(e))
        } else {
            Err(Fehler::Format("unbekannter Objekttyp"))
        }
    }

    pub fn zeitpunkt(&self) -> &Zeitpunkt {
        match self {
            Objekt::Status(s) => &s.zeitpunkt,
            Objekt::Ereignis(e) => &e.zeitpunkt,
        }
    }

    pub fn lebenstag(&self) -> i64 {
        match self {
            Objekt::Status(s) => s.lebenstag,
            Objekt::Ereignis(e) => e.lebenstag,
        }
    }

    pub fn antworten(&self) -> &[Antwort] {
        match self {
            Objekt::Status(s) => &s.antworten,
            Objekt::Ereignis(e) => &e.antworten,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Präparat, Einstellungen
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planzeit {
    pub uhrzeit_min: i32,
    pub dosis: String,
    /// Bit 0 = Montag … Bit 6 = Sonntag; 0x7f = täglich.
    pub wochentage: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Praeparat {
    pub id: Id,
    pub handelsname: String,
    pub wirkstoff: Option<String>,
    pub staerke: Option<String>,
    pub form: Option<String>,
    pub plan: Vec<Planzeit>,
    pub bedarf: bool,
    pub wirkung_spuerbar: bool,
    pub aktiv: bool,
    pub erinnerung: bool,
    pub status_nach_min: Option<i64>,
}

impl Praeparat {
    pub fn wert(&self) -> Wert {
        kopf(typ::PRAEPARAT)
            .mit(3, cbor::bytes(&self.id))
            .mit(4, cbor::text(&self.handelsname))
            .opt(5, self.wirkstoff.as_deref().map(cbor::text))
            .opt(6, self.staerke.as_deref().map(cbor::text))
            .opt(7, self.form.as_deref().map(cbor::text))
            .mit(
                8,
                Wert::Feld(
                    self.plan
                        .iter()
                        .map(|p| {
                            MapBau::neu()
                                .mit(0, cbor::zahl(p.uhrzeit_min))
                                .mit(1, cbor::text(&p.dosis))
                                .mit(2, cbor::zahl(p.wochentage))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .mit(9, Wert::Bool(self.bedarf))
            .mit(10, Wert::Bool(self.wirkung_spuerbar))
            .mit(11, Wert::Bool(self.aktiv))
            .mit(12, Wert::Bool(self.erinnerung))
            .opt(13, self.status_nach_min.map(cbor::zahl))
            .fertig()
    }

    pub fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        kopf_pruefen(&mut m, typ::PRAEPARAT)?;
        let id = m.bytes_n(3)?;
        let handelsname = m.text(4)?;
        let wirkstoff = m.opt_text(5)?;
        let staerke = m.opt_text(6)?;
        let form = m.opt_text(7)?;
        let mut plan = Vec::new();
        for p in m.feld(8)? {
            let mut pm = MapLeser::aus(p)?;
            plan.push(Planzeit {
                uhrzeit_min: i32::try_from(pm.i64(0)?).map_err(|_| Fehler::Format("Uhrzeit"))?,
                dosis: pm.text(1)?,
                wochentage: u8_feld(&mut pm, 2)?,
            });
            pm.ende()?;
        }
        let p = Praeparat {
            id,
            handelsname,
            wirkstoff,
            staerke,
            form,
            plan,
            bedarf: m.bool(9)?,
            wirkung_spuerbar: m.bool(10)?,
            aktiv: m.bool(11)?,
            erinnerung: m.bool(12)?,
            status_nach_min: m.opt_i64(13)?,
        };
        m.ende()?;
        Ok(p)
    }

    pub fn pruefen(&self) -> Ergebnis<()> {
        if self.handelsname.trim().is_empty() || self.handelsname.len() > 200 {
            return Err(Fehler::Eingabe("Name des Präparats"));
        }
        for p in &self.plan {
            if !(0..1440).contains(&p.uhrzeit_min)
                || p.wochentage == 0
                || p.wochentage > 0x7f
                || p.dosis.len() > 100
            {
                return Err(Fehler::Eingabe("Einnahmeplan"));
            }
        }
        if self.plan.len() > 12
            || self
                .status_nach_min
                .is_some_and(|m| !(15..=720).contains(&m))
        {
            return Err(Fehler::Eingabe("Einnahmeplan"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErinnerungArt {
    Status = 1,
    Tagesabschluss = 2,
    Wochenrueckblick = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erinnerung {
    pub art: ErinnerungArt,
    pub uhrzeit_min: i32,
    pub wochentage: u8,
    pub aktiv: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vertrauensperson {
    pub name: String,
    pub telefon: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Krisenregel {
    /// Zeig meinen Plan, wenn ich `tage` Tage in Folge einen Tageswert unter `unter` eintrage.
    pub tage: u32,
    pub unter: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Einstellungen {
    pub tageswechsel_min: i32,
    pub sperrzeit_s: u32,
    pub erinnerungen: Vec<Erinnerung>,
    pub max_benachrichtigungen: u32,
    pub vertrauenspersonen: Vec<Vertrauensperson>,
    pub krisenregel: Option<Krisenregel>,
    pub ausgeblendet: Vec<Id>,
    /// Selbst gewählte Bezeichnungen der eigenen Tagesmarker (Optionen 11 bis 13), nur im Bund.
    pub eigene_marker: Vec<String>,
    /// Plan für Tiefs (#131). Die Klartextkopie für die Hilfe ohne Entsperren liegt in den
    /// lokalen Metadaten; diese Kopie im Bund kommt mit in die Sicherung.
    pub krisenplan: Option<String>,
}

impl Einstellungen {
    pub fn standard(tageswechsel_min: i32) -> Self {
        Einstellungen {
            tageswechsel_min,
            sperrzeit_s: 60,
            erinnerungen: vec![
                Erinnerung {
                    art: ErinnerungArt::Tagesabschluss,
                    uhrzeit_min: 21 * 60,
                    wochentage: 0x7f,
                    aktiv: true,
                },
                Erinnerung {
                    art: ErinnerungArt::Wochenrueckblick,
                    uhrzeit_min: 18 * 60,
                    wochentage: 1 << 6,
                    aktiv: true,
                },
            ],
            max_benachrichtigungen: 8,
            vertrauenspersonen: Vec::new(),
            krisenregel: None,
            ausgeblendet: Vec::new(),
            eigene_marker: Vec::new(),
            krisenplan: None,
        }
    }

    pub fn wert(&self) -> Wert {
        kopf(typ::EINSTELLUNGEN)
            .mit(3, cbor::zahl(self.tageswechsel_min))
            .mit(4, cbor::zahl(self.sperrzeit_s))
            .mit(
                5,
                Wert::Feld(
                    self.erinnerungen
                        .iter()
                        .map(|e| {
                            MapBau::neu()
                                .mit(0, cbor::zahl(e.art as u8))
                                .mit(1, cbor::zahl(e.uhrzeit_min))
                                .mit(2, cbor::zahl(e.wochentage))
                                .mit(3, Wert::Bool(e.aktiv))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .mit(6, cbor::zahl(self.max_benachrichtigungen))
            .mit(
                7,
                Wert::Feld(
                    self.vertrauenspersonen
                        .iter()
                        .map(|v| {
                            MapBau::neu()
                                .mit(0, cbor::text(&v.name))
                                .mit(1, cbor::text(&v.telefon))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .opt(
                8,
                self.krisenregel.as_ref().map(|k| {
                    MapBau::neu()
                        .mit(0, cbor::zahl(k.tage))
                        .mit(1, cbor::zahl(k.unter))
                        .fertig()
                }),
            )
            .mit(9, ids(&self.ausgeblendet))
            .mit(
                10,
                Wert::Feld(self.eigene_marker.iter().map(|t| cbor::text(t)).collect()),
            )
            .opt(11, self.krisenplan.as_deref().map(cbor::text))
            .fertig()
    }

    pub fn aus(w: Wert) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        kopf_pruefen(&mut m, typ::EINSTELLUNGEN)?;
        let tageswechsel_min =
            i32::try_from(m.i64(3)?).map_err(|_| Fehler::Format("Tageswechsel"))?;
        let sperrzeit_s = u32::try_from(m.u64(4)?).map_err(|_| Fehler::Format("Sperrzeit"))?;
        let mut erinnerungen = Vec::new();
        for e in m.feld(5)? {
            let mut em = MapLeser::aus(e)?;
            erinnerungen.push(Erinnerung {
                art: match u8_feld(&mut em, 0)? {
                    1 => ErinnerungArt::Status,
                    2 => ErinnerungArt::Tagesabschluss,
                    3 => ErinnerungArt::Wochenrueckblick,
                    _ => return Err(Fehler::Format("Erinnerung")),
                },
                uhrzeit_min: i32::try_from(em.i64(1)?).map_err(|_| Fehler::Format("Uhrzeit"))?,
                wochentage: u8_feld(&mut em, 2)?,
                aktiv: em.bool(3)?,
            });
            em.ende()?;
        }
        let max_benachrichtigungen =
            u32::try_from(m.u64(6)?).map_err(|_| Fehler::Format("Obergrenze"))?;
        let mut vertrauenspersonen = Vec::new();
        for v in m.feld(7)? {
            let mut vm = MapLeser::aus(v)?;
            vertrauenspersonen.push(Vertrauensperson {
                name: vm.text(0)?,
                telefon: vm.text(1)?,
            });
            vm.ende()?;
        }
        let krisenregel = match m.opt_wert(8) {
            None => None,
            Some(k) => {
                let mut km = MapLeser::aus(k)?;
                let r = Krisenregel {
                    tage: u32::try_from(km.u64(0)?).map_err(|_| Fehler::Format("Regel"))?,
                    unter: km.i64(1)?,
                };
                km.ende()?;
                Some(r)
            }
        };
        let ausgeblendet = ids_aus(m.wert(9)?)?;
        let eigene_marker = m
            .feld(10)?
            .into_iter()
            .map(cbor::als_text)
            .collect::<Ergebnis<Vec<_>>>()?;
        let krisenplan = m.opt_text(11)?;
        m.ende()?;
        Ok(Einstellungen {
            tageswechsel_min,
            sperrzeit_s,
            erinnerungen,
            max_benachrichtigungen,
            vertrauenspersonen,
            krisenregel,
            ausgeblendet,
            eigene_marker,
            krisenplan,
        })
    }

    pub fn pruefen(&self) -> Ergebnis<()> {
        if self.krisenplan.as_ref().is_some_and(|t| t.len() > 8000) {
            return Err(Fehler::Eingabe("Krisenplan zu lang"));
        }
        if !(0..=12 * 60).contains(&self.tageswechsel_min) {
            return Err(Fehler::Eingabe("Tageswechsel zwischen 0 und 12 Uhr"));
        }
        if !(15..=3600).contains(&self.sperrzeit_s) {
            return Err(Fehler::Eingabe("Sperrzeit"));
        }
        if !(1..=12).contains(&self.max_benachrichtigungen) {
            return Err(Fehler::Eingabe("Obergrenze Benachrichtigungen"));
        }
        for e in &self.erinnerungen {
            if !(0..1440).contains(&e.uhrzeit_min) || e.wochentage == 0 || e.wochentage > 0x7f {
                return Err(Fehler::Eingabe("Erinnerung"));
            }
        }
        if let Some(k) = &self.krisenregel
            && (!(1..=60).contains(&k.tage) || !(-5..=5).contains(&k.unter))
        {
            return Err(Fehler::Eingabe("Regel für den Krisenplan"));
        }
        if self.vertrauenspersonen.len() > 10
            || self
                .vertrauenspersonen
                .iter()
                .any(|v| v.name.len() > 100 || v.telefon.len() > 40)
        {
            return Err(Fehler::Eingabe("Vertrauenspersonen"));
        }
        if self.eigene_marker.len() > 3
            || self
                .eigene_marker
                .iter()
                .any(|t| t.trim().is_empty() || t.len() > 40)
        {
            return Err(Fehler::Eingabe("eigene Marker"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_rundweg_und_strenge() {
        let s = Objekt::Status(Status {
            id: [1; 16],
            zeitpunkt: Zeitpunkt {
                utc_ms: 1_790_000_000_000,
                versatz_min: 120,
                zone: "Europe/Berlin".into(),
            },
            erfasst_um: 1_790_000_000_500,
            ausloeser: Ausloeser::Selbst,
            abfrage: None,
            erfasst_ueber: ErfasstUeber::App,
            vorlaeufig: false,
            bezug_von: Some(1_789_990_000_000),
            bezug_bis: Some(1_790_000_000_000),
            antworten: vec![
                Antwort {
                    messdef: [2; 16],
                    zustand: Zustand::Beantwortet,
                    wert: Some(Antwortwert {
                        zahl: Some(-3),
                        ..Default::default()
                    }),
                    aktiv_gesetzt: true,
                },
                Antwort {
                    messdef: [3; 16],
                    zustand: Zustand::Uebersprungen,
                    wert: None,
                    aktiv_gesetzt: false,
                },
            ],
            lebenstag: 20_740,
            formular: [4; 16],
            art: FormularArt::StatusJetzt,
        });
        let b = cbor::kodieren(&s.wert());
        assert_eq!(Objekt::aus(cbor::dekodieren(&b).unwrap()).unwrap(), s);
        // Unbekanntes Feld 23 wird abgelehnt.
        let mut w = cbor::dekodieren(&b).unwrap();
        if let Wert::Map(m) = &mut w {
            m.push((23, cbor::zahl(1)));
        }
        assert!(Objekt::aus(w).is_err());
    }

    #[test]
    fn beantwortet_ohne_wert_ist_ungueltig() {
        let a = MapBau::neu()
            .mit(0, cbor::bytes(&[0; 16]))
            .mit(1, cbor::zahl(0))
            .mit(3, Wert::Bool(false))
            .fertig();
        assert!(Antwort::aus(a).is_err());
    }
}
