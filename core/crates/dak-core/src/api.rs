// SPDX-License-Identifier: MPL-2.0
//! Einzige Schnittstelle zu den Apps (Architektur AE-06, AE-07). Ansichtsmodelle entstehen hier;
//! Kotlin und Swift zeichnen sie nur. IDs gehen als Hex-Text über die Grenze, Geheimnisse nur an
//! den drei Stellen Einrichtung, Rotation und Wiederherstellung (zum Umhüllen) und beim Entsperren.

use std::sync::{Arc, Mutex};

use crate::fehler::Fehler;
use crate::kern::{self, Kern, Lage, NeueSchluessel};
use crate::kette::Rotationsgrund;
use crate::modell::{self, Antwort, Antwortwert, Ausloeser, Zustand, hex, id_aus_hex};
use crate::zeit::Zeitpunkt;
use crate::{ansicht, plan, speicher, woerter};

// ---------------------------------------------------------------------------------------------
// Fehler und Uhr
// ---------------------------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "ffi", derive(uniffi::Error))]
pub enum DakFehler {
    #[error("{meldung} (Code {code})")]
    Fehler { code: u32, meldung: String },
}

impl From<Fehler> for DakFehler {
    fn from(f: Fehler) -> Self {
        DakFehler::Fehler {
            code: f.code(),
            meldung: f.to_string(),
        }
    }
}

pub type E<T> = Result<T, DakFehler>;

#[cfg_attr(feature = "ffi", uniffi::export(with_foreign))]
pub trait DakUhr: Send + Sync {
    fn jetzt(&self) -> Zeitpunkt;
}

struct UhrBruecke(Arc<dyn DakUhr>);

impl kern::Uhr for UhrBruecke {
    fn jetzt(&self) -> Zeitpunkt {
        self.0.jetzt()
    }
}

// ---------------------------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum KernLage {
    NichtEingerichtet,
    Gesperrt {
        generation: u64,
        generation_h: u64,
        kk_umhuellung: Option<Vec<u8>>,
        kk_h_umhuellung: Option<Vec<u8>>,
    },
    Entsperrt,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Schluessel {
    pub generation: u64,
    pub generation_h: u64,
    pub kk: Vec<u8>,
    pub kk_h: Vec<u8>,
}

impl From<&NeueSchluessel> for Schluessel {
    fn from(n: &NeueSchluessel) -> Self {
        Schluessel {
            generation: n.generation,
            generation_h: n.generation_h,
            kk: n.kk.clone(),
            kk_h: n.kk_h.clone(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Einrichtung {
    pub woerter: Vec<String>,
    /// Positionen (0-basiert) der drei Wörter, die zur Probe abgefragt werden.
    pub probe: Vec<u32>,
    pub schluessel: Schluessel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum AntwortZustand {
    Beantwortet,
    Uebersprungen,
    KamNichtVor,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Auswahl {
    pub option: u16,
    pub staerke: Option<i64>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct AntwortEingabe {
    pub messdef_id: String,
    pub zustand: AntwortZustand,
    pub zahl: Option<i64>,
    pub text: Option<String>,
    pub auswahl: Vec<Auswahl>,
    pub aktiv_gesetzt: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct StatusEingabe {
    pub formular_id: String,
    /// Nur bei Nachträgen; sonst gilt die Uhr.
    pub zeitpunkt: Option<Zeitpunkt>,
    pub antworten: Vec<AntwortEingabe>,
    /// Wurde der Block „seit dem letzten Status" geöffnet?
    pub block_offen: bool,
    pub aus_entwurf: Option<String>,
    pub korrektur_von: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EreignisEingabe {
    pub formular_id: String,
    pub zeitpunkt: Option<Zeitpunkt>,
    pub antworten: Vec<AntwortEingabe>,
    pub praeparat_id: Option<String>,
    pub korrektur_von: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum Feldart {
    Skala,
    Stufen,
    Einfachauswahl,
    Mehrfachauswahl,
    Text,
    Dauer,
    Uhrzeit,
    Zahl,
    JaNein,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum Richtung {
    Keine,
    HoeherBesser,
    HoeherMehr,
    Bipolar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum Formularart {
    StatusJetzt,
    Tagesabschluss,
    Wochenrueckblick,
    Ereignis,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Anker {
    pub wert: i64,
    pub text: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Wahl {
    pub id: u16,
    pub text: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct FrageAnsicht {
    pub messdef_id: String,
    pub schluessel: String,
    pub frage: String,
    pub feldart: Feldart,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub anker: Vec<Anker>,
    pub optionen: Vec<Wahl>,
    pub mit_staerke: bool,
    pub kam_nicht_vor: bool,
    pub richtung: Richtung,
    pub einheit: Option<String>,
    pub hinweis: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct GruppeAnsicht {
    pub titel: String,
    pub hinweis: Option<String>,
    /// Gruppe „seit dem letzten Status": zugeklappt, öffnen per Tipp.
    pub zugeklappt: bool,
    pub fragen: Vec<FrageAnsicht>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct FormularAnsicht {
    pub id: String,
    pub schluessel: String,
    pub name: String,
    pub art: Formularart,
    pub gruppen: Vec<GruppeAnsicht>,
    /// Überschrift des Blocks „seit dem letzten Status", z. B. „seit 10:15 Uhr".
    pub bezug_text: Option<String>,
    /// Ab zwei Stunden Abstand bietet die App den Block an (nicht aufgeklappt).
    pub block_anbieten: bool,
    /// Ereignisse: braucht das Formular ein Präparat?
    pub braucht_praeparat: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Knopf {
    pub formular_id: String,
    pub schluessel: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum EinnahmeStand {
    Offen,
    Genommen,
    Ausgelassen,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EinnahmeHeute {
    pub praeparat_id: String,
    pub name: String,
    pub staerke: Option<String>,
    pub uhrzeit_min: i32,
    pub dosis: String,
    pub stand: EinnahmeStand,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EntwurfKurz {
    pub id: String,
    pub formular_id: String,
    pub formular_name: String,
    pub erstellt_ms: i64,
    pub antworten: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Startseite {
    pub heute: String,
    pub ereignisse: Vec<Knopf>,
    pub einnahmen_heute: Vec<EinnahmeHeute>,
    pub bedarf: Vec<Knopf>,
    pub entwuerfe: Vec<EntwurfKurz>,
    pub status_heute: u32,
    pub letzter_status_ms: Option<i64>,
    pub tagesabschluss_heute: bool,
    pub wochenrueckblick_heute: bool,
    pub letzte_sicherung_ms: Option<i64>,
    pub sicherung_warnung: bool,
    pub sicherung_noetig: bool,
    /// Eigene Regel für den Krisenplan greift (höchstens einmal am Tag, nicht modal).
    pub krisenplan_hinweis: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EintragZeile {
    pub datensatz_id: String,
    pub ist_ereignis: bool,
    pub formular_id: String,
    pub name: String,
    pub utc_ms: i64,
    pub lebenstag: i64,
    pub datum: String,
    pub uhrzeit: String,
    pub zusammenfassung: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EintragDetail {
    pub zeile: EintragZeile,
    /// Alle Antworten als „Frage: Wert“, Freitexte im Wortlaut (#129).
    pub alle_antworten: Vec<String>,
    pub zeitpunkt: Zeitpunkt,
    pub antworten: Vec<AntwortEingabe>,
    pub praeparat_id: Option<String>,
    pub block_offen: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Planzeit {
    pub uhrzeit_min: i32,
    pub dosis: String,
    pub wochentage: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct PraeparatDaten {
    /// Leer für ein neues Präparat.
    pub id: String,
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct ErinnerungDaten {
    /// 1 Status, 2 Tagesabschluss, 3 Wochenrückblick
    pub art: u8,
    pub uhrzeit_min: i32,
    pub wochentage: u8,
    pub aktiv: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Vertrauensperson {
    pub name: String,
    pub telefon: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EinstellungenDaten {
    pub tageswechsel_min: i32,
    pub sperrzeit_s: u32,
    pub erinnerungen: Vec<ErinnerungDaten>,
    pub max_benachrichtigungen: u32,
    pub vertrauenspersonen: Vec<Vertrauensperson>,
    pub krisenregel_tage: Option<u32>,
    pub krisenregel_unter: Option<i64>,
    pub ausgeblendet: Vec<String>,
    pub eigene_marker: Vec<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Punkt {
    pub utc_ms: i64,
    pub lebenstag: i64,
    pub uhrzeit_min: i32,
    pub wert: i64,
    pub aus_tagesabschluss: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Tageswert {
    pub lebenstag: i64,
    pub datum: String,
    pub anzahl: u32,
    pub min: i64,
    pub max: i64,
    /// Letzter Wert des Tages (beim Tagesabschluss der Abendwert, #105).
    pub letzter: i64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Mittel {
    pub lebenstag: i64,
    pub wert: Option<f64>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Marker {
    pub lebenstag: i64,
    pub option: u16,
    pub name: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Verlauf {
    pub messdef_id: String,
    pub frage: String,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub anker: Vec<Anker>,
    pub richtung: Richtung,
    /// Jeder einzelne Eintrag als Punkt, auch mehrere am Tag.
    pub punkte: Vec<Punkt>,
    pub tage: Vec<Tageswert>,
    pub tage_gesamt: u32,
    pub tage_mit_wert: u32,
    pub gleitend7: Vec<Mittel>,
    pub gleitend28: Vec<Mittel>,
    pub marker: Vec<Marker>,
    pub einnahmen_ms: Vec<i64>,
    pub hinweis: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Faktorvergleich {
    pub faktor: String,
    pub tage_mit: u32,
    pub mittel_mit: Option<f64>,
    pub tage_ohne: u32,
    pub mittel_ohne: Option<f64>,
    pub hinweis: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct SicherungDatei {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct DateiInfo {
    pub name: String,
    /// Tag der letzten Änderung als Tagesnummer seit 1970-01-01.
    pub tag: i64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Kandidat {
    pub index: u32,
    pub name: String,
    pub sicherung_nr: u64,
    pub erstellt_ms: i64,
    pub gueltig: bool,
    pub grund: Option<String>,
    pub empfohlen: bool,
    pub zweig: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Wiederherstellung {
    pub schluessel: Schluessel,
    pub marker_name: String,
    pub marker: Vec<u8>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Benachrichtigung {
    pub schluessel: String,
    pub datum_iso: String,
    pub uhrzeit_min: i32,
    /// `medikament` oder `erinnerung`
    pub kanal: String,
    pub titel: String,
    pub text: String,
    /// Formular, das sich beim Tippen öffnet (nach dem Entsperren).
    pub ziel: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Nummer {
    pub name: String,
    pub nummer: String,
    pub hinweis: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Hilfe {
    pub nummern: Vec<Nummer>,
    pub krisenplan: Option<String>,
    /// Nur entsperrt, sonst leer.
    pub vertrauenspersonen: Vec<Vertrauensperson>,
    pub hinweis: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum Aktion {
    ExportCsv,
    RotationAufWunsch,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Bestaetigung {
    pub aktion: Aktion,
    pub titel: String,
    pub text: String,
    pub digest: Vec<u8>,
}

// ---------------------------------------------------------------------------------------------
// Umwandlungen
// ---------------------------------------------------------------------------------------------

pub(crate) fn id(s: &str) -> Result<modell::Id, Fehler> {
    id_aus_hex(s)
}

pub(crate) fn antwort_aus(a: AntwortEingabe) -> Result<Antwort, Fehler> {
    let zustand = match a.zustand {
        AntwortZustand::Beantwortet => Zustand::Beantwortet,
        AntwortZustand::Uebersprungen => Zustand::Uebersprungen,
        AntwortZustand::KamNichtVor => Zustand::KamNichtVor,
    };
    let wert = (zustand == Zustand::Beantwortet).then(|| Antwortwert {
        zahl: a.zahl,
        text: a.text,
        auswahl: a.auswahl.iter().map(|x| (x.option, x.staerke)).collect(),
    });
    Ok(Antwort {
        messdef: id(&a.messdef_id)?,
        zustand,
        wert,
        aktiv_gesetzt: a.aktiv_gesetzt,
    })
}

pub(crate) fn antwort_zu(a: &Antwort) -> Option<AntwortEingabe> {
    let zustand = match a.zustand {
        Zustand::Beantwortet => AntwortZustand::Beantwortet,
        Zustand::Uebersprungen => AntwortZustand::Uebersprungen,
        Zustand::KamNichtVor => AntwortZustand::KamNichtVor,
        _ => return None,
    };
    let w = a.wert.clone().unwrap_or_default();
    Some(AntwortEingabe {
        messdef_id: hex(&a.messdef),
        zustand,
        zahl: w.zahl,
        text: w.text,
        auswahl: w
            .auswahl
            .iter()
            .map(|(o, s)| Auswahl {
                option: *o,
                staerke: *s,
            })
            .collect(),
        aktiv_gesetzt: a.aktiv_gesetzt,
    })
}

fn opt_id(s: &Option<String>) -> Result<Option<modell::Id>, Fehler> {
    s.as_deref()
        .filter(|x| !x.is_empty())
        .map(id_aus_hex)
        .transpose()
}

// ---------------------------------------------------------------------------------------------
// Objekt
// ---------------------------------------------------------------------------------------------

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct DakKern {
    k: Mutex<Kern>,
}

impl DakKern {
    fn mit<T>(&self, f: impl FnOnce(&mut Kern) -> Result<T, Fehler>) -> E<T> {
        let mut g = self.k.lock().map_err(|_| DakFehler::Fehler {
            code: 900,
            meldung: "Kern blockiert".into(),
        })?;
        Ok(f(&mut g)?)
    }

    /// Für Tests: Kern mit eigenem Speicher, Zufall und Uhr.
    pub fn aus_kern(k: Kern) -> Arc<Self> {
        Arc::new(DakKern { k: Mutex::new(k) })
    }
}

#[cfg_attr(feature = "ffi", uniffi::export)]
impl DakKern {
    #[cfg_attr(feature = "ffi", uniffi::constructor)]
    pub fn neu(pfad: String, uhr: Arc<dyn DakUhr>) -> E<Arc<Self>> {
        let k = Kern::oeffnen(&pfad, Box::new(UhrBruecke(uhr)))?;
        Ok(Arc::new(DakKern { k: Mutex::new(k) }))
    }

    pub fn lage(&self) -> E<KernLage> {
        self.mit(|k| {
            Ok(match k.lage()? {
                Lage::NichtEingerichtet => KernLage::NichtEingerichtet,
                Lage::Entsperrt => KernLage::Entsperrt,
                Lage::Gesperrt {
                    generation,
                    generation_h,
                    kk_umhuellung,
                    kk_h_umhuellung,
                } => KernLage::Gesperrt {
                    generation,
                    generation_h,
                    kk_umhuellung,
                    kk_h_umhuellung,
                },
            })
        })
    }

    pub fn einrichtung_beginnen(&self, tageswechsel_min: i32) -> E<Einrichtung> {
        self.mit(|k| {
            let (w, p, s) = k.einrichtung_beginnen(tageswechsel_min)?;
            Ok(Einrichtung {
                woerter: w,
                probe: p.iter().map(|x| *x as u32).collect(),
                schluessel: Schluessel::from(&s),
            })
        })
    }

    pub fn einrichtung_probe_pruefen(&self, probe: Vec<String>) -> E<bool> {
        self.mit(|k| k.einrichtung_probe_pruefen(&probe))
    }

    pub fn einrichtung_abschliessen(
        &self,
        probe: Vec<String>,
        kk_umhuellung: Option<Vec<u8>>,
        kk_h_umhuellung: Option<Vec<u8>>,
    ) -> E<()> {
        self.mit(|k| k.einrichtung_abschliessen(&probe, kk_umhuellung, kk_h_umhuellung))
    }

    pub fn entsperren(&self, kk: Vec<u8>, kk_h: Vec<u8>) -> E<()> {
        self.mit(|k| k.entsperren(kk, kk_h))
    }

    pub fn sperren(&self) {
        let _ = self.mit(|k| {
            k.sperren();
            Ok(())
        });
    }

    pub fn heute(&self) -> E<i64> {
        self.mit(|k| {
            let s = k.sitzung()?;
            Ok(k.jetzt().lebenstag(s.tageswechsel()))
        })
    }

    pub fn startseite(&self) -> E<Startseite> {
        self.mit(|k| ansicht::startseite(k))
    }

    pub fn formular(&self, schluessel: String) -> E<FormularAnsicht> {
        self.mit(|k| ansicht::formular(k, &schluessel))
    }

    pub fn status_speichern(&self, e: StatusEingabe) -> E<String> {
        self.mit(|k| {
            let antworten = e
                .antworten
                .into_iter()
                .map(antwort_aus)
                .collect::<Result<Vec<_>, _>>()?;
            let r = k.status_speichern(
                id(&e.formular_id)?,
                e.zeitpunkt,
                antworten,
                e.block_offen,
                Ausloeser::Selbst,
                None,
                opt_id(&e.aus_entwurf)?,
                opt_id(&e.korrektur_von)?,
            )?;
            ansicht::aenderung(k)?;
            Ok(hex(&r))
        })
    }

    pub fn ereignis_erfassen(&self, e: EreignisEingabe) -> E<String> {
        self.mit(|k| {
            let antworten = e
                .antworten
                .into_iter()
                .map(antwort_aus)
                .collect::<Result<Vec<_>, _>>()?;
            let r = k.ereignis_erfassen(
                id(&e.formular_id)?,
                e.zeitpunkt,
                antworten,
                opt_id(&e.praeparat_id)?,
                opt_id(&e.korrektur_von)?,
            )?;
            ansicht::aenderung(k)?;
            Ok(hex(&r))
        })
    }

    pub fn eintrag_loeschen(&self, datensatz_id: String) -> E<()> {
        self.mit(|k| {
            k.eintrag_loeschen(id(&datensatz_id)?)?;
            ansicht::aenderung(k)
        })
    }

    pub fn eintraege(&self, von_tag: i64, bis_tag: i64) -> E<Vec<EintragZeile>> {
        self.mit(|k| ansicht::eintraege(k, von_tag, bis_tag))
    }

    pub fn eintrag(&self, datensatz_id: String) -> E<EintragDetail> {
        self.mit(|k| ansicht::eintrag(k, &id(&datensatz_id)?))
    }

    pub fn entwurf_sichern(
        &self,
        entwurf_id: Option<String>,
        formular_id: String,
        antworten: Vec<AntwortEingabe>,
        block_offen: bool,
    ) -> E<String> {
        self.mit(|k| {
            let a = antworten
                .into_iter()
                .map(antwort_aus)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(hex(&k.entwurf_sichern(
                opt_id(&entwurf_id)?,
                id(&formular_id)?,
                a,
                block_offen,
            )?))
        })
    }

    pub fn entwurf_laden(&self, entwurf_id: String) -> E<StatusEingabe> {
        self.mit(|k| {
            let i = id(&entwurf_id)?;
            let e = k
                .entwuerfe()?
                .into_iter()
                .find(|e| e.id == i)
                .ok_or(Fehler::NichtGefunden("Entwurf"))?;
            Ok(StatusEingabe {
                formular_id: hex(&e.formular),
                zeitpunkt: None,
                antworten: e.antworten.iter().filter_map(antwort_zu).collect(),
                block_offen: e.block_offen,
                aus_entwurf: Some(entwurf_id.clone()),
                korrektur_von: None,
            })
        })
    }

    pub fn entwurf_verwerfen(&self, entwurf_id: String) -> E<()> {
        self.mit(|k| k.entwurf_verwerfen(id(&entwurf_id)?))
    }

    pub fn praeparate(&self) -> E<Vec<PraeparatDaten>> {
        self.mit(|k| {
            Ok(k.sitzung()?
                .bund
                .praeparate
                .iter()
                .map(ansicht::praeparat_zu)
                .collect())
        })
    }

    pub fn praeparat_speichern(&self, p: PraeparatDaten) -> E<String> {
        self.mit(|k| {
            let r = k.praeparat_speichern(ansicht::praeparat_aus(p)?)?;
            ansicht::aenderung(k)?;
            Ok(hex(&r))
        })
    }

    pub fn einnahme_bestaetigen(
        &self,
        praeparat_id: String,
        zeitpunkt: Option<Zeitpunkt>,
    ) -> E<String> {
        self.mit(|k| {
            let f = k
                .sitzung()?
                .formular_schluessel("einnahme")
                .ok_or(Fehler::NichtGefunden("Formular"))?
                .id;
            let r = k.ereignis_erfassen(f, zeitpunkt, vec![], Some(id(&praeparat_id)?), None)?;
            ansicht::aenderung(k)?;
            Ok(hex(&r))
        })
    }

    pub fn einstellungen(&self) -> E<EinstellungenDaten> {
        self.mit(|k| Ok(ansicht::einstellungen_zu(&k.sitzung()?.bund.einstellungen)))
    }

    pub fn einstellungen_speichern(&self, e: EinstellungenDaten) -> E<()> {
        self.mit(|k| {
            k.einstellungen_setzen(ansicht::einstellungen_aus(e)?)?;
            ansicht::aenderung(k)
        })
    }

    pub fn krisenplan_setzen(&self, text: Option<String>) -> E<()> {
        self.mit(|k| k.krisenplan_setzen(text))
    }

    /// Hilfe ohne Entsperren (#71): Nummern und eigener Plan immer, Kontakte nur entsperrt.
    pub fn hilfe(&self) -> E<Hilfe> {
        self.mit(|k| ansicht::hilfe(k))
    }

    pub fn verlauf(&self, schluessel: String, von_tag: i64, bis_tag: i64) -> E<Verlauf> {
        self.mit(|k| ansicht::verlauf(k, &schluessel, von_tag, bis_tag))
    }

    pub fn faktorvergleich(
        &self,
        faktor_schluessel: String,
        von_tag: i64,
        bis_tag: i64,
    ) -> E<Faktorvergleich> {
        self.mit(|k| ansicht::faktorvergleich(k, &faktor_schluessel, von_tag, bis_tag))
    }

    pub fn sicherung_erstellen(&self) -> E<SicherungDatei> {
        self.mit(|k| {
            let (name, bytes) = k.sicherung_erstellen()?;
            Ok(SicherungDatei { name, bytes })
        })
    }

    pub fn sicherung_bestaetigen(&self, gelesen: Vec<u8>) -> E<u64> {
        self.mit(|k| {
            let n = k.sicherung_bestaetigen(&gelesen)?;
            let a = speicher::meta_u64(&k.db, "aenderungen")?.unwrap_or(0);
            speicher::meta_u64_setzen(&k.db, "aenderungen_gesichert", a)?;
            Ok(n)
        })
    }

    pub fn marker_pruefen(&self, marker: Vec<Vec<u8>>) -> E<bool> {
        self.mit(|k| k.marker_pruefen(&marker))
    }

    /// Zu löschende Sicherungen nach der Aufbewahrungsregel (Protokoll 10.2).
    pub fn aufbewahrung(&self, dateien: Vec<DateiInfo>, heute_tag: i64) -> E<Vec<String>> {
        self.mit(|k| {
            let s = k.sitzung()?;
            let kuerzel = hex(&s.konto_id)[..8].to_owned();
            Ok(crate::sicherung::aufbewahrung(
                &kuerzel,
                &dateien
                    .into_iter()
                    .map(|d| (d.name, d.tag))
                    .collect::<Vec<_>>(),
                heute_tag,
            ))
        })
    }

    pub fn wiederherstellung_beginnen(&self, woerter: Vec<String>) -> E<()> {
        self.mit(|k| k.wiederherstellung_beginnen(&woerter))
    }

    pub fn kandidaten(
        &self,
        dateien: Vec<SicherungDatei>,
        marker: Vec<Vec<u8>>,
    ) -> E<Vec<Kandidat>> {
        self.mit(|k| {
            let d: Vec<(String, Vec<u8>)> =
                dateien.into_iter().map(|x| (x.name, x.bytes)).collect();
            Ok(k.kandidaten(&d, &marker)?
                .into_iter()
                .map(|c| Kandidat {
                    index: c.index as u32,
                    name: c.name,
                    sicherung_nr: c.sicherung_nr,
                    erstellt_ms: c.erstellt_um,
                    gueltig: c.gueltig,
                    grund: c.grund,
                    empfohlen: c.empfohlen,
                    zweig: c.zweig,
                })
                .collect())
        })
    }

    pub fn wiederherstellen(
        &self,
        dateien: Vec<SicherungDatei>,
        marker: Vec<Vec<u8>>,
        index: u32,
    ) -> E<Wiederherstellung> {
        self.mit(|k| {
            let d: Vec<(String, Vec<u8>)> =
                dateien.into_iter().map(|x| (x.name, x.bytes)).collect();
            let (s, marker_name, marker) = k.wiederherstellen(&d, &marker, index as usize)?;
            Ok(Wiederherstellung {
                schluessel: Schluessel::from(&s),
                marker_name,
                marker,
            })
        })
    }

    /// Schließt Rotation oder Wiederherstellung ab, nachdem die Plattform die neuen Schlüssel umhüllt hat.
    pub fn vorbereitetes_abschliessen(
        &self,
        kk_umhuellung: Option<Vec<u8>>,
        kk_h_umhuellung: Option<Vec<u8>>,
    ) -> E<()> {
        self.mit(|k| {
            k.vorbereitetes_abschliessen(kk_umhuellung, kk_h_umhuellung)?;
            ansicht::aenderung(k)
        })
    }

    pub fn vorbereitetes_verwerfen(&self) {
        let _ = self.mit(|k| {
            k.vorbereitetes_verwerfen();
            Ok(())
        });
    }

    /// Rotation auf Wunsch; verlangt eine eingelöste native Bestätigung.
    pub fn rotation_beginnen(&self, digest: Vec<u8>) -> E<Schluessel> {
        self.mit(|k| {
            k.bestaetigung_einloesen("rotation", &digest)?;
            Ok(Schluessel::from(
                &k.rotation_beginnen(Rotationsgrund::AufWunsch)?,
            ))
        })
    }

    pub fn woerter_pruefen(&self, woerter: Vec<String>) -> E<bool> {
        self.mit(|k| k.woerter_pruefen(&woerter))
    }

    pub fn wort_vorschlaege(&self, anfang: String) -> Vec<String> {
        woerter::vorschlaege(&anfang, 6)
    }

    /// Plan der lokalen Benachrichtigungen für die nächsten Tage (#42, Protokoll 11).
    pub fn benachrichtigungen(&self, tage: u32, max_gesamt: u32) -> E<Vec<Benachrichtigung>> {
        self.mit(|k| plan::planen(k, tage, max_gesamt))
    }

    pub fn bestaetigung_anfordern(&self, aktion: Aktion) -> E<Bestaetigung> {
        self.mit(|k| {
            let (schl, titel, text) = match aktion {
                Aktion::ExportCsv => (
                    "export_csv",
                    "Alles als Tabelle exportieren?",
                    "Die Datei enthält alle Einträge im Klartext. Du wählst danach, wo sie gespeichert wird. Nimm nicht den Sicherungsordner und keinen Ordner, der in eine Cloud synchronisiert wird.",
                ),
                Aktion::RotationAufWunsch => ("rotation", "Schlüssel erneuern?", "Neue Schlüssel für alle Einträge. Alte Sicherungen bleiben mit den alten Schlüsseln lesbar, bis sie aus der Aufbewahrung fallen."),
            };
            let d = k.bestaetigung_anfordern(schl, text)?;
            Ok(Bestaetigung { aktion, titel: titel.into(), text: text.into(), digest: d.to_vec() })
        })
    }

    /// Klartext-Ausstieg (#78): alle Einträge als CSV, nur nach eingelöster Bestätigung.
    pub fn export_csv(&self, digest: Vec<u8>) -> E<String> {
        self.mit(|k| {
            k.bestaetigung_einloesen("export_csv", &digest)?;
            ansicht::export_csv(k)
        })
    }
}
