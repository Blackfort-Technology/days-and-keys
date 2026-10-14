// SPDX-License-Identifier: MPL-2.0
//! Der Kern: Einrichtung, Entsperren, Schreiben, Rotation, Sicherung und Wiederherstellung.
//!
//! Geheime Schlüssel existieren nur hier (AE-12). Die Plattform hält `KK` und `KK_h` nur hinter
//! ihrem Hardware-Schlüssel und reicht sie beim Entsperren herein; zum Umhüllen bekommt sie sie
//! nach Einrichtung, Rotation und Wiederherstellung einmal heraus.

use std::collections::BTreeMap;

use rusqlite::Connection;
use zeroize::Zeroize;

use crate::fehler::{Ergebnis, Fehler};
use crate::kette::{
    self, Ersetzungsmarker, Glied, GliedArt, Gruendung, Huelle, Kopf, Rotationsgrund,
};
use crate::modell::{
    Antwort, Antwortwert, Ausloeser, BedingungArt, Bezugsart, Einstellungen, Ereignis,
    ErfasstUeber, Feldtyp, Formular, FormularArt, Id, Klasse, Messdef, Objekt, Praeparat, Status,
    Zustand, hex,
};
use crate::sicherung::{self, Bestand, Geprueft};
use crate::sodium::{self, BoxPaar, Geheim, Zufall};
use crate::speicher;
use crate::tresor::{self, Bund, BundH, Datensatz, NeuerDatensatz, Recovery, Weg, Ziel};
use crate::vorlagen;
use crate::woerter;
use crate::zeit::Zeitpunkt;

pub const ZWEI_STUNDEN_MS: i64 = 2 * 3600 * 1000;
pub const ZWOELF_STUNDEN_MS: i64 = 12 * 3600 * 1000;
pub const DREI_TAGE_MS: i64 = 3 * 24 * 3600 * 1000;

/// Zeitquelle; in Tests ersetzbar (Protokoll 6.3, #81).
pub trait Uhr: Send + Sync {
    fn jetzt(&self) -> Zeitpunkt;
}

pub struct Eintrag {
    pub datensatz_id: Id,
    pub revision_id: Id,
    pub objekt: Objekt,
}

pub struct Sitzung {
    pub kk: Geheim,
    pub kk_h: Geheim,
    pub konto_id: [u8; 32],
    pub geraet_id: Id,
    pub kopf: Kopf,
    pub bund: Bund,
    pub bund_h: BundH,
    pub eintraege: Vec<Eintrag>,
}

impl Sitzung {
    pub fn messdef(&self, id: &Id) -> Option<&Messdef> {
        self.bund.messdefs.iter().find(|m| m.id == *id)
    }
    pub fn messdef_schluessel(&self, s: &str) -> Option<&Messdef> {
        self.bund.messdefs.iter().find(|m| m.schluessel == s)
    }
    pub fn formular(&self, id: &Id) -> Option<&Formular> {
        self.bund.formulare.iter().find(|f| f.id == *id)
    }
    pub fn formular_schluessel(&self, s: &str) -> Option<&Formular> {
        self.bund.formulare.iter().find(|f| f.schluessel == s)
    }
    pub fn tageswechsel(&self) -> i32 {
        self.bund.einstellungen.tageswechsel_min
    }
}

/// Schlüssel zum Umhüllen durch die Plattform (Hardware-Schlüssel, Keychain).
pub struct NeueSchluessel {
    pub generation: u64,
    pub generation_h: u64,
    pub kk: Vec<u8>,
    pub kk_h: Vec<u8>,
}

impl Drop for NeueSchluessel {
    fn drop(&mut self) {
        self.kk.zeroize();
        self.kk_h.zeroize();
    }
}

struct EinrichtungOffen {
    rk: Geheim,
    kk: Geheim,
    kk_h: Geheim,
    tageswechsel_min: i32,
    probe: Vec<usize>,
    woerter: Vec<String>,
}

impl Drop for EinrichtungOffen {
    fn drop(&mut self) {
        for w in &mut self.woerter {
            w.zeroize();
        }
    }
}

/// Vorbereiteter Bestand, der erst nach dem Umhüllen durch die Plattform geschrieben wird.
struct Vorbereitet {
    meta: Vec<(&'static str, Vec<u8>)>,
    kette: Vec<Vec<u8>>,
    bund: Vec<u8>,
    bund_h: Vec<u8>,
    /// (datensatz_id, revision_id, aktuell, roh)
    datensaetze: Vec<(Id, Id, bool, Vec<u8>)>,
    tombstones: Vec<Vec<u8>>,
    alles_ersetzen: bool,
    kk: Geheim,
    kk_h: Geheim,
}

pub struct Kern {
    pub(crate) db: Connection,
    pub(crate) z: Box<dyn Zufall>,
    pub(crate) uhr: Box<dyn Uhr>,
    pub(crate) sitzung: Option<Sitzung>,
    einrichtung: Option<EinrichtungOffen>,
    vorbereitet: Option<Vorbereitet>,
    pub(crate) recovery: Option<Recovery>,
    pub(crate) bestaetigung: Option<(String, [u8; 32])>,
}

pub enum Lage {
    NichtEingerichtet,
    Gesperrt {
        generation: u64,
        generation_h: u64,
        kk_umhuellung: Option<Vec<u8>>,
        kk_h_umhuellung: Option<Vec<u8>>,
    },
    Entsperrt,
}

fn meta_pflicht(db: &Connection, k: &str) -> Ergebnis<Vec<u8>> {
    speicher::meta(db, k)?.ok_or(Fehler::NichtEingerichtet)
}

fn arr<const N: usize>(v: Vec<u8>) -> Ergebnis<[u8; N]> {
    v.try_into()
        .map_err(|_| Fehler::Format("Länge im Speicher"))
}

impl Kern {
    pub fn neu(db: Connection, z: Box<dyn Zufall>, uhr: Box<dyn Uhr>) -> Self {
        sodium::init();
        Kern {
            db,
            z,
            uhr,
            sitzung: None,
            einrichtung: None,
            vorbereitet: None,
            recovery: None,
            bestaetigung: None,
        }
    }

    pub fn oeffnen(pfad: &str, uhr: Box<dyn Uhr>) -> Ergebnis<Self> {
        Ok(Kern::neu(
            speicher::oeffnen(pfad)?,
            Box::new(sodium::SystemZufall),
            uhr,
        ))
    }

    pub fn jetzt(&self) -> Zeitpunkt {
        self.uhr.jetzt()
    }

    pub fn eingerichtet(&self) -> Ergebnis<bool> {
        Ok(speicher::meta(&self.db, "konto_id")?.is_some())
    }

    pub fn lage(&self) -> Ergebnis<Lage> {
        if !self.eingerichtet()? {
            return Ok(Lage::NichtEingerichtet);
        }
        if self.sitzung.is_some() {
            return Ok(Lage::Entsperrt);
        }
        Ok(Lage::Gesperrt {
            generation: speicher::meta_u64(&self.db, "generation")?.unwrap_or(1),
            generation_h: speicher::meta_u64(&self.db, "generation_h")?.unwrap_or(1),
            kk_umhuellung: speicher::meta(&self.db, "kk_umh")?,
            kk_h_umhuellung: speicher::meta(&self.db, "kk_h_umh")?,
        })
    }

    pub fn sitzung(&self) -> Ergebnis<&Sitzung> {
        self.sitzung.as_ref().ok_or(Fehler::Gesperrt)
    }

    pub fn sperren(&mut self) {
        self.sitzung = None;
        self.bestaetigung = None;
    }

    // -----------------------------------------------------------------------------------------
    // Einrichtung
    // -----------------------------------------------------------------------------------------

    /// Erzeugt Recovery-Wörter und Klassenschlüssel; geschrieben wird erst beim Abschluss.
    pub fn einrichtung_beginnen(
        &mut self,
        tageswechsel_min: i32,
    ) -> Ergebnis<(Vec<String>, Vec<usize>, NeueSchluessel)> {
        if self.eingerichtet()? {
            return Err(Fehler::SchonEingerichtet);
        }
        Einstellungen::standard(tageswechsel_min).pruefen()?;
        let rk = Geheim::zufall(32, &mut *self.z);
        let woerter = woerter::kodieren(&rk);
        let mut probe: Vec<usize> = Vec::new();
        while probe.len() < 3 {
            let i = (sodium::zufall_bytes::<1>(&mut *self.z)[0] as usize) % woerter::ANZAHL;
            if !probe.contains(&i) {
                probe.push(i);
            }
        }
        probe.sort_unstable();
        let kk = Geheim::zufall(32, &mut *self.z);
        let kk_h = Geheim::zufall(32, &mut *self.z);
        let s = NeueSchluessel {
            generation: 1,
            generation_h: 1,
            kk: kk.bytes().to_vec(),
            kk_h: kk_h.bytes().to_vec(),
        };
        self.einrichtung = Some(EinrichtungOffen {
            rk,
            kk,
            kk_h,
            tageswechsel_min,
            probe: probe.clone(),
            woerter: woerter.clone(),
        });
        Ok((woerter, probe, s))
    }

    /// Prüft die Wortprobe und schreibt den Bestand. `kk_umh`/`kk_h_umh` sind die von der
    /// Plattform erzeugten Umhüllungen (Android) oder `None` (iOS: Keychain hält den Schlüssel).
    pub fn einrichtung_abschliessen(
        &mut self,
        probe_antworten: &[String],
        kk_umh: Option<Vec<u8>>,
        kk_h_umh: Option<Vec<u8>>,
    ) -> Ergebnis<()> {
        let e = self
            .einrichtung
            .as_ref()
            .ok_or(Fehler::Zustand("keine Einrichtung offen"))?;
        if probe_antworten.len() != e.probe.len() {
            return Err(Fehler::Woerter);
        }
        for (i, a) in e.probe.iter().zip(probe_antworten) {
            if woerter::index(a) != woerter::index(&e.woerter[*i]) {
                return Err(Fehler::Woerter);
            }
        }
        let e = self.einrichtung.take().expect("geprüft");
        let jetzt = self.uhr.jetzt();
        let rec = Recovery::aus_rk(&e.rk);
        let geraet_id = tresor::neue_id(&mut *self.z);
        let dk_sig_saat = Geheim::zufall(32, &mut *self.z);
        let dk_enc_saat = Geheim::zufall(32, &mut *self.z);
        let dk_sig = sodium::SignPaar::aus_saat(&dk_sig_saat);
        let dk_enc = BoxPaar::aus_saat(&dk_enc_saat);
        let gruendung = Gruendung {
            geraet_id,
            dk_sig: dk_sig.oeffentlich,
            dk_enc: dk_enc.oeffentlich,
            rk_sig: rec.sig.oeffentlich,
            rk_enc: rec.enc.oeffentlich,
            erstellt_um: jetzt.utc_ms,
        }
        .signieren(&dk_sig, &rec.sig);
        let kopf = kette::pruefen(&gruendung, &[])?;
        let konto_id = kopf.konto_id;
        let v = vorlagen::standard()?;
        let bund = Bund {
            version: 1,
            generation: 1,
            ds: Geheim::zufall(32, &mut *self.z),
            ws_saat: Geheim::zufall(32, &mut *self.z),
            dk_sig_saat,
            dk_enc_saat,
            messdefs: v.messdefs,
            formulare: v.formulare,
            praeparate: Vec::new(),
            einstellungen: Einstellungen::standard(e.tageswechsel_min),
        };
        let bund_h = BundH {
            version: 1,
            generation_h: 1,
            ws_saat: Geheim::zufall(32, &mut *self.z),
            messdefs: Vec::new(),
            bezeichnungen: Vec::new(),
        };
        let bund_b =
            tresor::bund_verschluesseln(&bund, &e.kk, &konto_id, &kopf.kopf_hash, &mut *self.z);
        let bund_h_b = tresor::bund_h_verschluesseln(
            &bund_h,
            &e.kk_h,
            &konto_id,
            &kopf.kopf_hash,
            &mut *self.z,
        );
        let kapsel = tresor::kapsel(&konto_id, 1, &e.kk, &rec.enc.oeffentlich, false);
        let kapsel_h = tresor::kapsel(&konto_id, 1, &e.kk_h, &rec.enc.oeffentlich, true);
        let mut meta: Vec<(&'static str, Vec<u8>)> = vec![
            (
                "formatversion",
                crate::modell::FORMATVERSION.to_be_bytes().to_vec(),
            ),
            ("konto_id", konto_id.to_vec()),
            ("geraet_id", geraet_id.to_vec()),
            ("gruendung", gruendung.kodieren()),
            ("generation", 1u64.to_be_bytes().to_vec()),
            ("generation_h", 1u64.to_be_bytes().to_vec()),
            ("kapsel", kapsel),
            ("kapsel_h", kapsel_h),
            ("ws_g_pub", bund.ws().oeffentlich.to_vec()),
            ("sicherung_nr", 0u64.to_be_bytes().to_vec()),
        ];
        if let Some(u) = kk_umh {
            meta.push(("kk_umh", u));
        }
        if let Some(u) = kk_h_umh {
            meta.push(("kk_h_umh", u));
        }
        let tx = self.db.transaction()?;
        for (k, v) in &meta {
            speicher::meta_setzen(&tx, k, v)?;
        }
        speicher::bund_setzen(&tx, 1, &bund_b)?;
        speicher::bund_setzen(&tx, 2, &bund_h_b)?;
        tx.commit()?;
        self.sitzung = Some(Sitzung {
            kk: e.kk.clone(),
            kk_h: e.kk_h.clone(),
            konto_id,
            geraet_id,
            kopf,
            bund,
            bund_h,
            eintraege: Vec::new(),
        });
        Ok(())
    }

    // -----------------------------------------------------------------------------------------
    // Entsperren
    // -----------------------------------------------------------------------------------------

    fn kopf_laden(&self) -> Ergebnis<Kopf> {
        let g = Huelle::dekodieren(&meta_pflicht(&self.db, "gruendung")?)?;
        let glieder = speicher::kette(&self.db)?
            .iter()
            .map(|b| Huelle::dekodieren(b))
            .collect::<Ergebnis<Vec<_>>>()?;
        let kopf = kette::pruefen(&g, &glieder)?;
        if kopf.konto_id != arr::<32>(meta_pflicht(&self.db, "konto_id")?)? {
            return Err(Fehler::Integritaet("Konto im Speicher"));
        }
        Ok(kopf)
    }

    pub fn entsperren(&mut self, mut kk_b: Vec<u8>, mut kk_h_b: Vec<u8>) -> Ergebnis<()> {
        let kk = Geheim::aus(&kk_b);
        let kk_h = Geheim::aus(&kk_h_b);
        kk_b.zeroize();
        kk_h_b.zeroize();
        if kk.len() != 32 || kk_h.len() != 32 {
            return Err(Fehler::Eingabe("Schlüssellänge"));
        }
        let kopf = self.kopf_laden()?;
        let konto_id = kopf.konto_id;
        let geraet_id: Id = arr(meta_pflicht(&self.db, "geraet_id")?)?;
        if kopf.geraet_id != geraet_id {
            return Err(Fehler::Kette("dieses Gerät ist ersetzt"));
        }
        let bund = tresor::bund_entschluesseln(
            &speicher::bund(&self.db, 1)?.ok_or(Fehler::NichtEingerichtet)?,
            &kk,
            &konto_id,
            &kopf.kopf_hash,
        )
        .map_err(|_| Fehler::Integritaet("Schlüssel passt nicht"))?;
        let bund_h = tresor::bund_h_entschluesseln(
            &speicher::bund(&self.db, 2)?.ok_or(Fehler::NichtEingerichtet)?,
            &kk_h,
            &konto_id,
            &kopf.kopf_hash,
        )
        .map_err(|_| Fehler::Integritaet("Schlüssel passt nicht"))?;
        if bund.generation != kopf.generation || bund_h.generation_h != kopf.generation_h {
            return Err(Fehler::Integritaet("Generation des Bundes"));
        }
        let mut eintraege = Vec::new();
        let ws = bund.ws();
        for roh in speicher::datensaetze(&self.db, true)? {
            let d = Datensatz::dekodieren(&roh)?;
            let ds = (d.umhuellung.weg == Weg::Datenschluessel).then_some(&bund.ds);
            if d.umhuellung.weg == Weg::Datenschluessel
                && d.umhuellung.generation != bund.generation
            {
                return Err(Fehler::Integritaet("Umhüllung einer alten Generation"));
            }
            let dk = tresor::dk_holen(&d, &konto_id, ds, Some(&ws))?;
            let objekt = tresor::oeffnen(&d, &konto_id, &dk)?;
            eintraege.push(Eintrag {
                datensatz_id: d.datensatz_id,
                revision_id: d.revision_id,
                objekt,
            });
        }
        eintraege.sort_by_key(|e| e.objekt.zeitpunkt().utc_ms);
        self.sitzung = Some(Sitzung {
            kk,
            kk_h,
            konto_id,
            geraet_id,
            kopf,
            bund,
            bund_h,
            eintraege,
        });
        self.entwuerfe_aufraeumen()?;
        Ok(())
    }

    // -----------------------------------------------------------------------------------------
    // Bund schreiben
    // -----------------------------------------------------------------------------------------

    pub(crate) fn bund_speichern(&mut self) -> Ergebnis<()> {
        let s = self.sitzung.as_mut().ok_or(Fehler::Gesperrt)?;
        s.bund.version += 1;
        let b = tresor::bund_verschluesseln(
            &s.bund,
            &s.kk,
            &s.konto_id,
            &s.kopf.kopf_hash,
            &mut *self.z,
        );
        speicher::bund_setzen(&self.db, 1, &b)?;
        Ok(())
    }

    pub fn einstellungen_setzen(&mut self, e: Einstellungen) -> Ergebnis<()> {
        e.pruefen()?;
        self.sitzung
            .as_mut()
            .ok_or(Fehler::Gesperrt)?
            .bund
            .einstellungen = e;
        self.bund_speichern()
    }

    pub fn praeparat_speichern(&mut self, mut p: Praeparat) -> Ergebnis<Id> {
        p.pruefen()?;
        let neu_id = tresor::neue_id(&mut *self.z);
        let s = self.sitzung.as_mut().ok_or(Fehler::Gesperrt)?;
        if let Some(alt) = s.bund.praeparate.iter_mut().find(|x| x.id == p.id) {
            *alt = p.clone();
        } else {
            if s.bund.praeparate.len() >= 30 {
                return Err(Fehler::Eingabe("zu viele Präparate"));
            }
            p.id = neu_id;
            s.bund.praeparate.push(p.clone());
        }
        self.bund_speichern()?;
        Ok(p.id)
    }

    /// Krisenplan (#105): auf Wunsch der Person ohne Entsperren lesbar, deshalb unverschlüsselt.
    pub fn krisenplan_setzen(&mut self, text: Option<String>) -> Ergebnis<()> {
        self.sitzung()?;
        match text {
            Some(t) if !t.trim().is_empty() => {
                if t.len() > 8000 {
                    return Err(Fehler::Eingabe("Krisenplan zu lang"));
                }
                speicher::meta_setzen(&self.db, "krisenplan", t.as_bytes())
            }
            _ => speicher::meta_loeschen(&self.db, "krisenplan"),
        }
    }

    pub fn krisenplan(&self) -> Ergebnis<Option<String>> {
        Ok(speicher::meta(&self.db, "krisenplan")?.and_then(|b| String::from_utf8(b).ok()))
    }

    // -----------------------------------------------------------------------------------------
    // Status und Ereignisse schreiben
    // -----------------------------------------------------------------------------------------

    fn eintrag_schreiben(&mut self, objekt: Objekt, vorgaenger: Option<(Id, Id)>) -> Ergebnis<Id> {
        let s = self.sitzung.as_ref().ok_or(Fehler::Gesperrt)?;
        Datensatz::klasse_pruefen(&objekt, Klasse::Gesundheit, &s.bund.messdefs)?;
        let datensatz_id = match &vorgaenger {
            Some((d, _)) => *d,
            None => tresor::neue_id(&mut *self.z),
        };
        if speicher::ist_geloescht(&self.db, &datensatz_id)? {
            return Err(Fehler::NichtGefunden("Eintrag gelöscht"));
        }
        let d = tresor::versiegeln(
            NeuerDatensatz {
                konto_id: &s.konto_id,
                datensatz_id,
                vorgaenger: vorgaenger.map(|v| v.1),
                klasse: Klasse::Gesundheit,
                objekt: &objekt,
            },
            Ziel::Datenschluessel {
                ds: &s.bund.ds,
                generation: s.bund.generation,
            },
            &mut *self.z,
        );
        speicher::datensatz_schreiben(&self.db, &d.datensatz_id, &d.revision_id, &d.kodieren())?;
        let s = self.sitzung.as_mut().expect("geprüft");
        s.eintraege.retain(|e| e.datensatz_id != datensatz_id);
        s.eintraege.push(Eintrag {
            datensatz_id,
            revision_id: d.revision_id,
            objekt,
        });
        s.eintraege.sort_by_key(|e| e.objekt.zeitpunkt().utc_ms);
        Ok(datensatz_id)
    }

    /// Ende des letzten Status mit geöffnetem Block „seit dem letzten Status" (Protokoll 6.1).
    pub fn bezug_von(&self, bis_utc: i64) -> Ergebnis<(Option<i64>, bool)> {
        let s = self.sitzung()?;
        let seit_ids: Vec<Id> = s
            .bund
            .messdefs
            .iter()
            .filter(|m| m.bezugsart == Bezugsart::SeitLetztem)
            .map(|m| m.id)
            .collect();
        let mut letzter: Option<i64> = None;
        let mut aufgewacht: Option<i64> = None;
        for e in &s.eintraege {
            let t = e.objekt.zeitpunkt().utc_ms;
            if t >= bis_utc {
                continue;
            }
            match &e.objekt {
                Objekt::Status(st) if st.ausloeser != Ausloeser::Abfrage && !st.vorlaeufig => {
                    if st
                        .antworten
                        .iter()
                        .any(|a| seit_ids.contains(&a.messdef) && a.zustand != Zustand::BlockZu)
                    {
                        letzter = Some(letzter.map_or(t, |l: i64| l.max(t)));
                    }
                }
                Objekt::Ereignis(ev) if ev.art == "aufgewacht" => {
                    aufgewacht = Some(aufgewacht.map_or(t, |l: i64| l.max(t)))
                }
                _ => {}
            }
        }
        let grenze = bis_utc - ZWOELF_STUNDEN_MS;
        let mut von = letzter;
        if let Some(a) = aufgewacht
            && von.is_none_or(|v| v < a)
        {
            von = Some(a);
        }
        match von {
            Some(v) if v >= grenze => Ok((Some(v), false)),
            _ => Ok((Some(grenze), true)),
        }
    }

    /// Wurde heute (Lebenstag) eine Einnahme eines Präparats mit spürbarer Wirkung bestätigt?
    pub fn einnahme_mit_wirkung_heute(&self, lebenstag: i64) -> Ergebnis<bool> {
        let s = self.sitzung()?;
        Ok(s.eintraege.iter().any(|e| match &e.objekt {
            Objekt::Ereignis(ev) => {
                ev.art == "einnahme"
                    && !ev.vorlaeufig
                    && ev.lebenstag == lebenstag
                    && ev.praeparat.is_some_and(|p| {
                        s.bund
                            .praeparate
                            .iter()
                            .any(|x| x.id == p && x.wirkung_spuerbar)
                    })
            }
            _ => false,
        }))
    }

    /// Letzter nicht vorläufiger Status (für den Zwei-Stunden-Abstand).
    pub fn letzter_status_ms(&self) -> Ergebnis<Option<i64>> {
        let s = self.sitzung()?;
        Ok(s.eintraege.iter().rev().find_map(|e| match &e.objekt {
            Objekt::Status(st) if !st.vorlaeufig => Some(st.zeitpunkt.utc_ms),
            _ => None,
        }))
    }

    /// Bildet die vollständige Antwortliste: was die Oberfläche nicht liefert, bekommt den
    /// passenden Zustand (übersprungen, nicht gezeigt, ausgeblendet, Block zu). Keine Vorbelegung.
    fn antworten_vervollstaendigen(
        &self,
        f: &Formular,
        eingabe: Vec<Antwort>,
        block_offen: bool,
        lebenstag: i64,
    ) -> Ergebnis<Vec<Antwort>> {
        let s = self.sitzung()?;
        let mut alle: Vec<Antwort> = Vec::new();
        let mut gegeben: BTreeMap<Id, Antwort> = BTreeMap::new();
        for a in eingabe {
            if gegeben.insert(a.messdef, a).is_some() {
                return Err(Fehler::Eingabe("Frage doppelt beantwortet"));
            }
        }
        let wirkung = self.einnahme_mit_wirkung_heute(lebenstag)?;
        for g in &f.gruppen {
            for id in &g.messdefs {
                let md = s
                    .messdef(id)
                    .ok_or(Fehler::Format("Formular verweist ins Leere"))?;
                let bedingt = f
                    .bedingungen
                    .iter()
                    .any(|(b, art)| b == id && *art == BedingungArt::EinnahmeMitWirkungHeute)
                    && !wirkung;
                let ausgeblendet = s.bund.einstellungen.ausgeblendet.contains(id);
                let block_zu = g.zugeklappt && !block_offen;
                match gegeben.remove(id) {
                    Some(a) => {
                        if bedingt || ausgeblendet || block_zu {
                            return Err(Fehler::Eingabe("Antwort auf eine nicht gezeigte Frage"));
                        }
                        match a.zustand {
                            Zustand::Beantwortet => md.wert_pruefen(
                                a.wert.as_ref().ok_or(Fehler::Eingabe("Wert fehlt"))?,
                            )?,
                            Zustand::Uebersprungen => {}
                            Zustand::KamNichtVor if md.kam_nicht_vor => {}
                            _ => return Err(Fehler::Eingabe("Zustand nicht zulässig")),
                        }
                        if a.zustand != Zustand::Beantwortet && a.wert.is_some() {
                            return Err(Fehler::Eingabe("Wert ohne Antwort"));
                        }
                        alle.push(a);
                    }
                    None => {
                        let zustand = if bedingt {
                            Zustand::NichtGezeigtBedingung
                        } else if ausgeblendet {
                            Zustand::Ausgeblendet
                        } else if block_zu {
                            Zustand::BlockZu
                        } else {
                            Zustand::Uebersprungen
                        };
                        alle.push(Antwort {
                            messdef: *id,
                            zustand,
                            wert: None,
                            aktiv_gesetzt: false,
                        });
                    }
                }
            }
        }
        if !gegeben.is_empty() {
            return Err(Fehler::Eingabe(
                "Antwort auf eine Frage, die nicht im Formular steht",
            ));
        }
        Ok(alle)
    }

    fn zeitpunkt_pruefen(&self, z: &Zeitpunkt) -> Ergebnis<()> {
        z.pruefen()?;
        let jetzt = self.uhr.jetzt().utc_ms;
        if z.utc_ms > jetzt + 5 * 60 * 1000 {
            return Err(Fehler::Eingabe("Zeitpunkt in der Zukunft"));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn status_speichern(
        &mut self,
        formular_id: Id,
        zeitpunkt: Option<Zeitpunkt>,
        antworten: Vec<Antwort>,
        block_offen: bool,
        ausloeser: Ausloeser,
        abfrage: Option<Id>,
        aus_entwurf: Option<Id>,
        korrektur_von: Option<Id>,
    ) -> Ergebnis<Id> {
        let s = self.sitzung()?;
        let f = s
            .formular(&formular_id)
            .ok_or(Fehler::NichtGefunden("Formular"))?
            .clone();
        if f.art == FormularArt::Ereignis {
            return Err(Fehler::Eingabe("Ereignisformular"));
        }
        let jetzt = self.uhr.jetzt();
        let zp = zeitpunkt.unwrap_or_else(|| jetzt.clone());
        self.zeitpunkt_pruefen(&zp)?;
        let lebenstag = zp.lebenstag(s.tageswechsel());
        let vorgaenger = match korrektur_von {
            Some(d) => Some((
                d,
                s.eintraege
                    .iter()
                    .find(|e| e.datensatz_id == d)
                    .ok_or(Fehler::NichtGefunden("Eintrag"))?
                    .revision_id,
            )),
            None => None,
        };
        let block_offen = block_offen && f.gruppen.iter().any(|g| g.zugeklappt);
        let antworten = self.antworten_vervollstaendigen(&f, antworten, block_offen, lebenstag)?;
        let (bezug_von, bezug_bis) = if block_offen {
            let (v, _) = self.bezug_von(zp.utc_ms)?;
            (v, Some(zp.utc_ms))
        } else {
            (None, None)
        };
        let status_id = tresor::neue_id(&mut *self.z);
        let objekt = Objekt::Status(Status {
            id: status_id,
            zeitpunkt: zp,
            erfasst_um: jetzt.utc_ms,
            ausloeser,
            abfrage,
            erfasst_ueber: if aus_entwurf.is_some() {
                ErfasstUeber::Entwurf
            } else {
                ErfasstUeber::App
            },
            vorlaeufig: false,
            bezug_von,
            bezug_bis,
            antworten,
            lebenstag,
            formular: formular_id,
            art: f.art,
        });
        let id = self.eintrag_schreiben(objekt, vorgaenger)?;
        if let Some(e) = aus_entwurf {
            speicher::entwurf_loeschen(&self.db, &e)?;
        }
        Ok(id)
    }

    pub fn ereignis_erfassen(
        &mut self,
        formular_id: Id,
        zeitpunkt: Option<Zeitpunkt>,
        antworten: Vec<Antwort>,
        praeparat: Option<Id>,
        korrektur_von: Option<Id>,
    ) -> Ergebnis<Id> {
        let s = self.sitzung()?;
        let f = s
            .formular(&formular_id)
            .ok_or(Fehler::NichtGefunden("Formular"))?
            .clone();
        if f.art != FormularArt::Ereignis {
            return Err(Fehler::Eingabe("kein Ereignisformular"));
        }
        if let Some(p) = praeparat
            && !s.bund.praeparate.iter().any(|x| x.id == p)
        {
            return Err(Fehler::NichtGefunden("Präparat"));
        }
        if matches!(f.schluessel.as_str(), "einnahme" | "einnahme_ausgelassen")
            && praeparat.is_none()
        {
            return Err(Fehler::Eingabe("Präparat fehlt"));
        }
        let jetzt = self.uhr.jetzt();
        let zp = zeitpunkt.unwrap_or_else(|| jetzt.clone());
        self.zeitpunkt_pruefen(&zp)?;
        let lebenstag = zp.lebenstag(s.tageswechsel());
        let vorgaenger = match korrektur_von {
            Some(d) => Some((
                d,
                s.eintraege
                    .iter()
                    .find(|e| e.datensatz_id == d)
                    .ok_or(Fehler::NichtGefunden("Eintrag"))?
                    .revision_id,
            )),
            None => None,
        };
        let antworten = self.antworten_vervollstaendigen(&f, antworten, true, lebenstag)?;
        let objekt = Objekt::Ereignis(Ereignis {
            id: tresor::neue_id(&mut *self.z),
            art: f.schluessel.clone(),
            formular: formular_id,
            zeitpunkt: zp,
            erfasst_um: jetzt.utc_ms,
            lebenstag,
            antworten,
            praeparat,
            vorlaeufig: false,
        });
        self.eintrag_schreiben(objekt, vorgaenger)
    }

    pub fn eintrag_loeschen(&mut self, datensatz_id: Id) -> Ergebnis<()> {
        let s = self.sitzung.as_mut().ok_or(Fehler::Gesperrt)?;
        if !s.eintraege.iter().any(|e| e.datensatz_id == datensatz_id) {
            return Err(Fehler::NichtGefunden("Eintrag"));
        }
        speicher::loeschen(&self.db, &datensatz_id)?;
        s.eintraege.retain(|e| e.datensatz_id != datensatz_id);
        Ok(())
    }

    // -----------------------------------------------------------------------------------------
    // Entwürfe (Protokoll 6.2 und 12, #94): an den Schreibschlüssel versiegelt, auch gesperrt
    // -----------------------------------------------------------------------------------------

    pub fn entwurf_sichern(
        &mut self,
        entwurf_id: Option<Id>,
        formular_id: Id,
        antworten: Vec<Antwort>,
        block_offen: bool,
    ) -> Ergebnis<Id> {
        let ws_pub: [u8; 32] = arr(meta_pflicht(&self.db, "ws_g_pub")?)?;
        let id = entwurf_id.unwrap_or_else(|| tresor::neue_id(&mut *self.z));
        let jetzt = self.uhr.jetzt();
        if antworten.len() > 200 {
            return Err(Fehler::Eingabe("Entwurf zu groß"));
        }
        let w = crate::modell::kopf(crate::modell::typ::ENTWURF)
            .mit(3, crate::cbor::bytes(&id))
            .mit(4, crate::cbor::bytes(&formular_id))
            .mit(5, jetzt.wert())
            .mit(6, crate::cbor::Wert::Bool(block_offen))
            .mit(
                7,
                crate::cbor::Wert::Feld(antworten.iter().map(antwort_wert).collect()),
            )
            .fertig();
        let klar = zeroize::Zeroizing::new(crate::cbor::kodieren(&w));
        let gepolstert = sodium::auffuellen(&klar, tresor::BLOCK);
        let c = sodium::versiegeln(&gepolstert, &ws_pub);
        speicher::entwurf_setzen(&self.db, &id, &c)?;
        Ok(id)
    }

    /// Alle Entwürfe öffnen; ältere als zwölf Stunden werden verworfen (#94).
    pub fn entwuerfe(&self) -> Ergebnis<Vec<EntwurfInhalt>> {
        let s = self.sitzung()?;
        let ws = s.bund.ws();
        let jetzt = self.uhr.jetzt().utc_ms;
        let mut v = Vec::new();
        for (id, roh) in speicher::entwuerfe(&self.db)? {
            let Ok(klar) = ws.oeffnen(&roh) else { continue };
            let Ok(klar) = sodium::auffuellung_entfernen(klar, tresor::BLOCK) else {
                continue;
            };
            let Ok(e) = entwurf_lesen(&klar) else {
                continue;
            };
            if e.id[..] != id[..] || jetzt - e.erstellt.utc_ms > ZWOELF_STUNDEN_MS {
                continue;
            }
            v.push(e);
        }
        v.sort_by_key(|e| e.erstellt.utc_ms);
        Ok(v)
    }

    fn entwuerfe_aufraeumen(&mut self) -> Ergebnis<()> {
        let gueltig: Vec<Id> = self.entwuerfe()?.into_iter().map(|e| e.id).collect();
        for (id, _) in speicher::entwuerfe(&self.db)? {
            if !gueltig.iter().any(|g| g[..] == id[..]) {
                speicher::entwurf_loeschen(&self.db, &id)?;
            }
        }
        Ok(())
    }

    pub fn entwurf_verwerfen(&mut self, id: Id) -> Ergebnis<()> {
        speicher::entwurf_loeschen(&self.db, &id)
    }

    // -----------------------------------------------------------------------------------------
    // Rotation (Protokoll 9)
    // -----------------------------------------------------------------------------------------

    /// Bereitet eine Rotation vor; geschrieben wird nach dem Umhüllen durch die Plattform.
    pub fn rotation_beginnen(&mut self, grund: Rotationsgrund) -> Ergebnis<NeueSchluessel> {
        let s = self.sitzung.as_ref().ok_or(Fehler::Gesperrt)?;
        let gruendung = Huelle::dekodieren(&meta_pflicht(&self.db, "gruendung")?)?;
        let mut kette_b = speicher::kette(&self.db)?;
        let v = rotieren(
            &mut *self.z,
            RotationsEingang {
                gruendung: &gruendung,
                kette_b: &mut kette_b,
                kopf: &s.kopf,
                bund: &s.bund,
                bund_h: &s.bund_h,
                kk_h: &s.kk_h,
                datensaetze: speicher::datensaetze(&self.db, false)?,
                aktuell: speicher::datensaetze(&self.db, true)?,
                dk_sig: &s.bund.dk_sig(),
                konto_id: &s.konto_id,
                grund,
                jetzt: self.uhr.jetzt().utc_ms,
            },
        )?;
        let neu = NeueSchluessel {
            generation: s.kopf.generation + 1,
            generation_h: s.kopf.generation_h,
            kk: v.kk.bytes().to_vec(),
            kk_h: v.kk_h.bytes().to_vec(),
        };
        self.vorbereitet = Some(v);
        Ok(neu)
    }

    /// Schreibt eine vorbereitete Rotation oder Wiederherstellung atomar und entsperrt mit den
    /// neuen Schlüsseln.
    pub fn vorbereitetes_abschliessen(
        &mut self,
        kk_umh: Option<Vec<u8>>,
        kk_h_umh: Option<Vec<u8>>,
    ) -> Ergebnis<()> {
        let v = self
            .vorbereitet
            .take()
            .ok_or(Fehler::Zustand("nichts vorbereitet"))?;
        let tx = self.db.transaction()?;
        if v.alles_ersetzen {
            speicher::leeren(&tx)?;
        }
        for (k, val) in &v.meta {
            speicher::meta_setzen(&tx, k, val)?;
        }
        match kk_umh {
            Some(u) => speicher::meta_setzen(&tx, "kk_umh", &u)?,
            None => speicher::meta_loeschen(&tx, "kk_umh")?,
        }
        match kk_h_umh {
            Some(u) => speicher::meta_setzen(&tx, "kk_h_umh", &u)?,
            None => speicher::meta_loeschen(&tx, "kk_h_umh")?,
        }
        speicher::kette_ersetzen(&tx, &v.kette)?;
        speicher::bund_setzen(&tx, 1, &v.bund)?;
        speicher::bund_setzen(&tx, 2, &v.bund_h)?;
        for (d, r, aktuell, roh) in &v.datensaetze {
            if v.alles_ersetzen {
                tx.execute(
                    "INSERT INTO datensatz (datensatz_id, revision_id, aktuell, roh) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![&d[..], &r[..], *aktuell as i64, roh],
                )?;
            } else {
                speicher::datensatz_ersetzen(&tx, d, r, roh)?;
            }
        }
        for t in &v.tombstones {
            tx.execute(
                "INSERT OR IGNORE INTO tombstone (datensatz_id) VALUES (?1)",
                [t],
            )?;
        }
        tx.commit()?;
        self.sitzung = None;
        let (kk, kk_h) = (v.kk.bytes().to_vec(), v.kk_h.bytes().to_vec());
        self.entsperren(kk, kk_h)
    }

    pub fn vorbereitetes_verwerfen(&mut self) {
        self.vorbereitet = None;
    }

    // -----------------------------------------------------------------------------------------
    // Sicherung (Protokoll 10.2)
    // -----------------------------------------------------------------------------------------

    pub fn bestand(&self) -> Ergebnis<Bestand> {
        Ok(Bestand {
            gruendung: meta_pflicht(&self.db, "gruendung")?,
            kette: speicher::kette(&self.db)?,
            bund: speicher::bund(&self.db, 1)?.ok_or(Fehler::NichtEingerichtet)?,
            bund_h: speicher::bund(&self.db, 2)?.ok_or(Fehler::NichtEingerichtet)?,
            datensaetze: speicher::datensaetze(&self.db, false)?,
            tombstones: speicher::tombstones(&self.db)?,
        })
    }

    /// Erzeugt die nächste Sicherung (Name und Bytes). Gültig wird sie erst mit
    /// [`Kern::sicherung_bestaetigen`], nachdem die Plattform sie geschrieben und zurückgelesen hat.
    pub fn sicherung_erstellen(&mut self) -> Ergebnis<(String, Vec<u8>)> {
        if speicher::meta(&self.db, "ersetzt")?.is_some() {
            return Err(Fehler::Kette(
                "dieses Gerät ist ersetzt und sichert nicht mehr",
            ));
        }
        let s = self.sitzung()?;
        let nr = speicher::meta_u64(&self.db, "sicherung_nr")?.unwrap_or(0) + 1;
        let kapsel = meta_pflicht(&self.db, "kapsel")?;
        let kapsel_h = meta_pflicht(&self.db, "kapsel_h")?;
        let bestand = self.bestand()?;
        let b = sicherung::schreiben(sicherung::Schreibdaten {
            bestand: &bestand,
            kk: &s.kk,
            kapsel: &kapsel,
            kapsel_h: &kapsel_h,
            kopf: &s.kopf,
            geraet_id: s.geraet_id,
            sicherung_nr: nr,
            erstellt_um: self.uhr.jetzt().utc_ms,
        });
        Ok((
            sicherung::dateiname(&s.konto_id, &s.geraet_id, s.kopf.generation, nr),
            b,
        ))
    }

    /// Prüft die zurückgelesenen Bytes und merkt sich die Sicherung als gelungen.
    pub fn sicherung_bestaetigen(&mut self, gelesen: &[u8]) -> Ergebnis<u64> {
        let s = self.sitzung()?;
        let m = sicherung::pruefen_mit_kk(gelesen, &s.kk, &s.konto_id)?;
        let nr = speicher::meta_u64(&self.db, "sicherung_nr")?.unwrap_or(0);
        if m.sicherung_nr != nr + 1
            || m.geraet_id != s.geraet_id
            || m.kettenkopf != s.kopf.kopf_hash
        {
            return Err(Fehler::Integritaet("zurückgelesene Sicherung passt nicht"));
        }
        speicher::meta_u64_setzen(&self.db, "sicherung_nr", m.sicherung_nr)?;
        speicher::meta_setzen(&self.db, "letzte_sicherung", &m.erstellt_um.to_be_bytes())?;
        Ok(m.sicherung_nr)
    }

    pub fn letzte_sicherung(&self) -> Ergebnis<Option<i64>> {
        Ok(speicher::meta(&self.db, "letzte_sicherung")?
            .and_then(|v| v.try_into().ok())
            .map(i64::from_be_bytes))
    }

    /// Prüft Ersetzungsmarker im Ordner (Protokoll 10.2): Ist dieses Gerät ersetzt, sichert es
    /// nicht mehr. Liefert `true`, wenn Sichern erlaubt ist.
    pub fn marker_pruefen(&mut self, marker: &[Vec<u8>]) -> Ergebnis<bool> {
        let kopf = self.kopf_laden()?;
        let geraet_id: Id = arr(meta_pflicht(&self.db, "geraet_id")?)?;
        for m in marker {
            if let Ok(e) = Ersetzungsmarker::pruefen(m, &kopf)
                && e.ersetztes_geraet == geraet_id
            {
                speicher::meta_setzen(&self.db, "ersetzt", &[1])?;
                self.sperren();
                return Ok(false);
            }
        }
        Ok(speicher::meta(&self.db, "ersetzt")?.is_none())
    }

    // -----------------------------------------------------------------------------------------
    // Wiederherstellung (Protokoll 10.3)
    // -----------------------------------------------------------------------------------------

    pub fn wiederherstellung_beginnen(&mut self, w: &[String]) -> Ergebnis<()> {
        if self.eingerichtet()? {
            return Err(Fehler::SchonEingerichtet);
        }
        let rk = woerter::dekodieren(w)?;
        self.recovery = Some(Recovery::aus_rk(&rk));
        Ok(())
    }

    /// Prüft eingegebene Wörter gegen den Kettenkopf, ohne sie zu speichern (#79, Wortprobe).
    pub fn woerter_pruefen(&self, w: &[String]) -> Ergebnis<bool> {
        let kopf = self.kopf_laden()?;
        let Ok(rk) = woerter::dekodieren(w) else {
            return Ok(false);
        };
        let r = Recovery::aus_rk(&rk);
        Ok(r.sig.oeffentlich == kopf.rk_sig && r.enc.oeffentlich == kopf.rk_enc)
    }

    pub fn kandidaten(
        &self,
        dateien: &[(String, Vec<u8>)],
        marker: &[Vec<u8>],
    ) -> Ergebnis<Vec<Kandidat>> {
        let rk = self
            .recovery
            .as_ref()
            .ok_or(Fehler::Zustand("keine Wiederherstellung offen"))?;
        Ok(kandidaten_waehlen(dateien, marker, rk))
    }

    /// Bereitet die Wiederherstellung aus Datei `index` vor: neues Gerät, `geraet_ersetzt`,
    /// Rotation. Liefert die neuen Schlüssel und den Ersetzungsmarker für den Ordner.
    pub fn wiederherstellen(
        &mut self,
        dateien: &[(String, Vec<u8>)],
        marker: &[Vec<u8>],
        index: usize,
    ) -> Ergebnis<(NeueSchluessel, String, Vec<u8>)> {
        let rk = self
            .recovery
            .as_ref()
            .ok_or(Fehler::Zustand("keine Wiederherstellung offen"))?;
        let k = kandidaten_waehlen(dateien, marker, rk);
        let kand = k
            .iter()
            .find(|x| x.index == index)
            .ok_or(Fehler::NichtGefunden("Sicherung"))?;
        if !kand.gueltig {
            return Err(Fehler::Kette("diese Sicherung ist nicht mehr gültig"));
        }
        let g: Geprueft = sicherung::pruefen_mit_recovery(&dateien[index].1, rk)?;
        let konto_id = g.kopf.konto_id;
        let bund_alt =
            tresor::bund_entschluesseln(&g.bestand.bund, &g.kk, &konto_id, &g.kopf.kopf_hash)?;
        let bund_h_alt = tresor::bund_h_entschluesseln(
            &g.bestand.bund_h,
            &g.kk_h,
            &konto_id,
            &g.kopf.kopf_hash,
        )?;
        // Neues Gerät und Glied `geraet_ersetzt`, signiert mit RK_sig und dem neuen Gerät.
        let geraet_id = tresor::neue_id(&mut *self.z);
        let dk_sig_saat = Geheim::zufall(32, &mut *self.z);
        let dk_enc_saat = Geheim::zufall(32, &mut *self.z);
        let dk_sig = sodium::SignPaar::aus_saat(&dk_sig_saat);
        let dk_enc = BoxPaar::aus_saat(&dk_enc_saat);
        let jetzt = self.uhr.jetzt().utc_ms;
        let gl = Glied {
            nr: g.kopf.laenge + 1,
            vorgaenger: g.kopf.kopf_hash,
            geraet_id,
            erstellt_um: jetzt,
            art: GliedArt::GeraetErsetzt {
                neues_geraet: geraet_id,
                dk_sig: dk_sig.oeffentlich,
                dk_enc: dk_enc.oeffentlich,
                letztes_glied_alt: g.kopf.laenge,
            },
        };
        let mut kette_b = g.bestand.kette.clone();
        kette_b.push(gl.signieren(&rk.sig, Some(&dk_sig)).kodieren());
        let gruendung = Huelle::dekodieren(&g.bestand.gruendung)?;
        let glieder = kette_b
            .iter()
            .map(|b| Huelle::dekodieren(b))
            .collect::<Ergebnis<Vec<_>>>()?;
        let kopf = kette::pruefen(&gruendung, &glieder)?;
        let marker_b = Ersetzungsmarker {
            konto_id,
            ersetztes_geraet: g.manifest.geraet_id,
            neues_geraet: geraet_id,
            glied_nr: gl.nr,
        }
        .signieren(&rk.sig);
        let marker_name = format!(
            "dak-{}-ersetzt-{}.dakm",
            &hex(&konto_id)[..8],
            &hex(&g.manifest.geraet_id)[..8]
        );
        // Tombstones anwenden.
        let tomb: Vec<Vec<u8>> = g.bestand.tombstones.clone();
        let datensaetze: Vec<Vec<u8>> = g
            .bestand
            .datensaetze
            .iter()
            .filter(|b| {
                Datensatz::dekodieren(b)
                    .map(|d| !tomb.iter().any(|t| t[..] == d.datensatz_id[..]))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        // Aktuelle Revision je Datensatz: die, auf die keine andere als Vorgänger zeigt.
        let mut aktuell: Vec<Vec<u8>> = Vec::new();
        {
            let alle: Vec<Datensatz> = datensaetze
                .iter()
                .map(|b| Datensatz::dekodieren(b))
                .collect::<Ergebnis<_>>()?;
            for d in &alle {
                if !alle.iter().any(|x| {
                    x.datensatz_id == d.datensatz_id && x.vorgaenger == Some(d.revision_id)
                }) {
                    aktuell.push(d.kodieren());
                }
            }
        }
        let mut bund = bund_alt;
        bund.dk_sig_saat = dk_sig_saat;
        bund.dk_enc_saat = dk_enc_saat;
        let mut v = rotieren(
            &mut *self.z,
            RotationsEingang {
                gruendung: &gruendung,
                kette_b: &mut kette_b,
                kopf: &kopf,
                bund: &bund,
                bund_h: &bund_h_alt,
                kk_h: &g.kk_h,
                datensaetze,
                aktuell,
                dk_sig: &dk_sig,
                konto_id: &konto_id,
                grund: Rotationsgrund::Wiederherstellung,
                jetzt,
            },
        )?;
        v.alles_ersetzen = true;
        v.tombstones = tomb;
        v.meta.extend([
            (
                "formatversion",
                crate::modell::FORMATVERSION.to_be_bytes().to_vec(),
            ),
            ("konto_id", konto_id.to_vec()),
            ("geraet_id", geraet_id.to_vec()),
            ("gruendung", g.bestand.gruendung.clone()),
            ("kapsel_h", g.kapsel_h.clone()),
            (
                "sicherung_nr",
                g.manifest.sicherung_nr.to_be_bytes().to_vec(),
            ),
        ]);
        let neu = NeueSchluessel {
            generation: kopf.generation + 1,
            generation_h: kopf.generation_h,
            kk: v.kk.bytes().to_vec(),
            kk_h: v.kk_h.bytes().to_vec(),
        };
        self.vorbereitet = Some(v);
        self.recovery = None;
        Ok((neu, marker_name, marker_b))
    }

    // -----------------------------------------------------------------------------------------
    // Native Bestätigung (Protokoll 13): an einen Digest der konkreten Aktion gebunden
    // -----------------------------------------------------------------------------------------

    pub fn bestaetigung_anfordern(&mut self, aktion: &str, text: &str) -> Ergebnis<[u8; 32]> {
        self.sitzung()?;
        let n = sodium::zufall_bytes::<16>(&mut *self.z);
        let d = sodium::hash32(&[
            b"dak-bestaetigung-v1",
            &sodium::lp(aktion.as_bytes()),
            &sodium::lp(text.as_bytes()),
            &n,
        ]);
        self.bestaetigung = Some((aktion.to_owned(), d));
        Ok(d)
    }

    pub fn bestaetigung_einloesen(&mut self, aktion: &str, digest: &[u8]) -> Ergebnis<()> {
        match self.bestaetigung.take() {
            Some((a, d)) if a == aktion && sodium::gleich_konstant(&d, digest) => Ok(()),
            _ => Err(Fehler::Bestaetigung),
        }
    }
}

pub(crate) fn antwort_wert(a: &Antwort) -> crate::cbor::Wert {
    use crate::cbor::{self, MapBau, Wert};
    MapBau::neu()
        .mit(0, cbor::bytes(&a.messdef))
        .mit(1, cbor::zahl(a.zustand as u8))
        .opt(
            2,
            a.wert.as_ref().map(|w| {
                MapBau::neu()
                    .opt(0, w.zahl.map(cbor::zahl))
                    .opt(1, w.text.as_deref().map(cbor::text))
                    .opt(
                        2,
                        (!w.auswahl.is_empty()).then(|| {
                            Wert::Feld(
                                w.auswahl
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
            }),
        )
        .mit(3, Wert::Bool(a.aktiv_gesetzt))
        .fertig()
}

pub struct EntwurfInhalt {
    pub id: Id,
    pub formular: Id,
    pub erstellt: Zeitpunkt,
    pub block_offen: bool,
    pub antworten: Vec<Antwort>,
}

fn entwurf_lesen(b: &[u8]) -> Ergebnis<EntwurfInhalt> {
    use crate::cbor::{self, MapLeser};
    let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
    crate::modell::kopf_pruefen(&mut m, crate::modell::typ::ENTWURF)?;
    let id = m.bytes_n(3)?;
    let formular = m.bytes_n(4)?;
    let erstellt = Zeitpunkt::aus(m.wert(5)?)?;
    let block_offen = m.bool(6)?;
    let mut antworten = Vec::new();
    for a in m.feld(7)? {
        let mut am = MapLeser::aus(a)?;
        let messdef = am.bytes_n(0)?;
        let zustand =
            Zustand::aus(u8::try_from(am.u64(1)?).map_err(|_| Fehler::Format("Zustand"))?)?;
        let wert = match am.opt_wert(2) {
            None => None,
            Some(w) => {
                let mut wm = MapLeser::aus(w)?;
                let zahl = wm.opt_i64(0)?;
                let text = wm.opt_text(1)?;
                let mut auswahl = Vec::new();
                if let Some(f) = wm.opt_wert(2) {
                    for x in cbor::als_feld(f)? {
                        let mut xm = MapLeser::aus(x)?;
                        auswahl.push((
                            u16::try_from(xm.u64(0)?).map_err(|_| Fehler::Format("Option"))?,
                            xm.opt_i64(1)?,
                        ));
                        xm.ende()?;
                    }
                }
                wm.ende()?;
                Some(Antwortwert {
                    zahl,
                    text,
                    auswahl,
                })
            }
        };
        let aktiv_gesetzt = am.bool(3)?;
        am.ende()?;
        antworten.push(Antwort {
            messdef,
            zustand,
            wert,
            aktiv_gesetzt,
        });
    }
    m.ende()?;
    Ok(EntwurfInhalt {
        id,
        formular,
        erstellt,
        block_offen,
        antworten,
    })
}

struct RotationsEingang<'a> {
    gruendung: &'a Huelle,
    kette_b: &'a mut Vec<Vec<u8>>,
    kopf: &'a Kopf,
    bund: &'a Bund,
    bund_h: &'a BundH,
    kk_h: &'a Geheim,
    datensaetze: Vec<Vec<u8>>,
    aktuell: Vec<Vec<u8>>,
    dk_sig: &'a sodium::SignPaar,
    konto_id: &'a [u8; 32],
    grund: Rotationsgrund,
    jetzt: i64,
}

/// Rotation der Klasse Gesundheit (der hochsensible Bereich ist in M1a nicht aktiv und rotiert
/// deshalb nicht mit, Protokoll 9 Schritt 2): neuer `KK`, neue `DS_g`, neuer `WS_g`, alle
/// Umhüllungen neu, neues Glied, neue Kapsel. Vorläufige Einträge an `WS_g` werden dabei unter
/// den neuen Datenschlüssel übernommen.
fn rotieren(z: &mut dyn Zufall, e: RotationsEingang<'_>) -> Ergebnis<Vorbereitet> {
    let neu_gen = e.kopf.generation + 1;
    let gl = Glied {
        nr: e.kopf.laenge + 1,
        vorgaenger: e.kopf.kopf_hash,
        geraet_id: e.kopf.geraet_id,
        erstellt_um: e.jetzt,
        art: GliedArt::Rotation {
            generation: neu_gen,
            generation_h: e.kopf.generation_h,
            grund: e.grund,
        },
    };
    e.kette_b.push(gl.signieren(e.dk_sig, None).kodieren());
    let glieder = e
        .kette_b
        .iter()
        .map(|b| Huelle::dekodieren(b))
        .collect::<Ergebnis<Vec<_>>>()?;
    let kopf_neu = kette::pruefen(e.gruendung, &glieder)?;
    let kk = Geheim::zufall(32, z);
    let ds_neu = Geheim::zufall(32, z);
    let ws_alt = e.bund.ws();
    let mut bund = Bund {
        version: e.bund.version + 1,
        generation: neu_gen,
        ds: ds_neu.clone(),
        ws_saat: Geheim::zufall(32, z),
        dk_sig_saat: e.bund.dk_sig_saat.clone(),
        dk_enc_saat: e.bund.dk_enc_saat.clone(),
        messdefs: e.bund.messdefs.clone(),
        formulare: e.bund.formulare.clone(),
        praeparate: e.bund.praeparate.clone(),
        einstellungen: e.bund.einstellungen.clone(),
    };
    let mut aus = Vec::with_capacity(e.datensaetze.len());
    for roh in &e.datensaetze {
        let mut d = Datensatz::dekodieren(roh)?;
        let ds_alt = (d.umhuellung.weg == Weg::Datenschluessel).then_some(&e.bund.ds);
        let dk = tresor::dk_holen(&d, e.konto_id, ds_alt, Some(&ws_alt))?;
        tresor::umhuellen_neu(
            &mut d,
            e.konto_id,
            &dk,
            Ziel::Datenschluessel {
                ds: &ds_neu,
                generation: neu_gen,
            },
            z,
        );
        let ist_aktuell = e.aktuell.iter().any(|a| a[..] == roh[..]);
        aus.push((d.datensatz_id, d.revision_id, ist_aktuell, d.kodieren()));
    }
    bund.version += 0;
    let bund_b = tresor::bund_verschluesseln(&bund, &kk, e.konto_id, &kopf_neu.kopf_hash, z);
    let mut bh = BundH {
        version: e.bund_h.version + 1,
        generation_h: e.bund_h.generation_h,
        ws_saat: e.bund_h.ws_saat.clone(),
        messdefs: e.bund_h.messdefs.clone(),
        bezeichnungen: e.bund_h.bezeichnungen.clone(),
    };
    bh.version += 0;
    let bund_h_b = tresor::bund_h_verschluesseln(&bh, e.kk_h, e.konto_id, &kopf_neu.kopf_hash, z);
    let kapsel = tresor::kapsel(e.konto_id, neu_gen, &kk, &kopf_neu.rk_enc, false);
    Ok(Vorbereitet {
        meta: vec![
            ("generation", neu_gen.to_be_bytes().to_vec()),
            ("generation_h", kopf_neu.generation_h.to_be_bytes().to_vec()),
            ("kapsel", kapsel),
            ("ws_g_pub", bund.ws().oeffentlich.to_vec()),
        ],
        kette: e.kette_b.clone(),
        bund: bund_b,
        bund_h: bund_h_b,
        datensaetze: aus,
        tombstones: Vec::new(),
        alles_ersetzen: false,
        kk,
        kk_h: e.kk_h.clone(),
    })
}

// ---------------------------------------------------------------------------------------------
// Auswahl der Sicherung (Protokoll 10.3): Kettenverwandtschaft vor der Nummer
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Kandidat {
    pub index: usize,
    pub name: String,
    pub sicherung_nr: u64,
    pub erstellt_um: i64,
    pub kette_laenge: u64,
    pub gueltig: bool,
    pub grund: Option<String>,
    pub empfohlen: bool,
    /// Zweig-Nummer: Kandidaten mit verschiedenen Zweigen stehen zur Wahl.
    pub zweig: u32,
}

fn kandidaten_waehlen(
    dateien: &[(String, Vec<u8>)],
    marker: &[Vec<u8>],
    rk: &Recovery,
) -> Vec<Kandidat> {
    struct G {
        index: usize,
        name: String,
        m: sicherung::Manifest,
        kopf: Kopf,
    }
    let mut gute: Vec<G> = Vec::new();
    let mut aus: Vec<Kandidat> = Vec::new();
    for (i, (name, b)) in dateien.iter().enumerate() {
        match sicherung::pruefen_mit_recovery(b, rk) {
            Ok(g) => gute.push(G {
                index: i,
                name: name.clone(),
                m: g.manifest,
                kopf: g.kopf,
            }),
            Err(f) => aus.push(Kandidat {
                index: i,
                name: name.clone(),
                sicherung_nr: 0,
                erstellt_um: 0,
                kette_laenge: 0,
                gueltig: false,
                grund: Some(match f {
                    Fehler::Woerter => {
                        "gehört zu anderen Recovery-Wörtern oder einem anderen Konto".into()
                    }
                    _ => format!("nicht lesbar ({})", f.code()),
                }),
                empfohlen: false,
                zweig: 0,
            }),
        }
    }
    // Ersetzte Geräte aus allen Ketten und aus gültigen Markern.
    let mut ersetzt: Vec<(Id, u64, [u8; 32])> = Vec::new();
    for g in &gute {
        for (dev, nr) in &g.kopf.ersetzt {
            ersetzt.push((*dev, *nr, g.kopf.hashes[(*nr - 1) as usize]));
        }
        for mk in marker {
            if let Ok(e) = Ersetzungsmarker::pruefen(mk, &g.kopf)
                && e.konto_id == g.kopf.konto_id
            {
                ersetzt.push((e.ersetztes_geraet, e.glied_nr, [0; 32]));
            }
        }
    }
    let ist_praefix = |a: &Kopf, b: &Kopf| {
        a.hashes.len() <= b.hashes.len() && b.hashes[..a.hashes.len()] == a.hashes[..]
    };
    let mut gueltig: Vec<&G> = Vec::new();
    for g in &gute {
        // Ungültig: das schreibende Gerät ist ersetzt, und diese Datei enthält die Ersetzung
        // nicht, ist aber nach der Ersetzung weitergelaufen bzw. von ihr überholt.
        let ungueltig = ersetzt.iter().any(|(dev, nr, h)| {
            *dev == g.m.geraet_id
                && !(g.kopf.hashes.len() as u64 >= *nr
                    && (*h == [0; 32] || g.kopf.hashes[(*nr - 1) as usize] == *h))
        }) && gute
            .iter()
            .any(|o| o.m.geraet_id != g.m.geraet_id && !ist_praefix(&o.kopf, &g.kopf));
        if ungueltig {
            aus.push(Kandidat {
                index: g.index,
                name: g.name.clone(),
                sicherung_nr: g.m.sicherung_nr,
                erstellt_um: g.m.erstellt_um,
                kette_laenge: g.kopf.laenge,
                gueltig: false,
                grund: Some("von einem Gerät, das inzwischen ersetzt ist".into()),
                empfohlen: false,
                zweig: 0,
            });
        } else {
            gueltig.push(g);
        }
    }
    // Zweige: Kandidaten, deren Ketten nicht Präfix voneinander sind.
    let mut koepfe: Vec<&Kopf> = Vec::new();
    for g in &gueltig {
        if !gueltig
            .iter()
            .any(|o| o.kopf.hashes.len() > g.kopf.hashes.len() && ist_praefix(&g.kopf, &o.kopf))
            && !koepfe.iter().any(|k| k.hashes == g.kopf.hashes)
        {
            koepfe.push(&g.kopf);
        }
    }
    let verzweigt = koepfe.len() > 1;
    for g in &gueltig {
        let zweig = koepfe
            .iter()
            .position(|k| ist_praefix(&g.kopf, k))
            .unwrap_or(0) as u32;
        aus.push(Kandidat {
            index: g.index,
            name: g.name.clone(),
            sicherung_nr: g.m.sicherung_nr,
            erstellt_um: g.m.erstellt_um,
            kette_laenge: g.kopf.laenge,
            gueltig: true,
            grund: None,
            empfohlen: false,
            zweig,
        });
    }
    if !verzweigt {
        // Höchste Kette, dann höchste Nummer.
        if let Some(best) = aus
            .iter_mut()
            .filter(|k| k.gueltig)
            .max_by_key(|k| (k.kette_laenge, k.sicherung_nr))
        {
            best.empfohlen = true;
        }
    }
    aus.sort_by_key(|k| k.index);
    aus
}

/// Fertige Klartextzeilen für Ansichten und Export.
pub fn messdef_klasse_ok(md: &Messdef) -> bool {
    md.klasse != Klasse::Hochsensibel
}

pub fn feld_ist_zahl(f: Feldtyp) -> bool {
    matches!(
        f,
        Feldtyp::Skala
            | Feldtyp::Stufen
            | Feldtyp::Zahl
            | Feldtyp::Dauer
            | Feldtyp::Uhrzeit
            | Feldtyp::JaNein
    )
}
