// SPDX-License-Identifier: MPL-2.0
//! Sicherungsdatei (Protokoll 10.1, Architektur AE-18).
//!
//! ```text
//! Datei   = "DAKSICH1" || u32be(len(kopf)) || kopf || { u32be(len(teil)) || teil }*
//! kopf    = cbor{0:20, 1:fv, 2:suite, 3:kapsel, 4:kapsel_h, 5:stromkopf}
//! teile   = crypto_secretstream(KDF(KK,"daksich_",1)) über cbor(innen) in 64-KiB-Stücken,
//!           letztes Stück mit TAG_FINAL; AD je Stück = "dak-sicherung-v1" || fv || konto_id
//! innen   = cbor{0:8, 1:fv, 3:gruendung, 4:kette, 5:bund, 6:bund_h, 7:datensaetze,
//!                8:tombstones, 9:manifest, 10:manifest_mac}
//! ```
//! Das Manifest ist mit einem eigenen Teilschlüssel aus `KK` authentisiert (keine Signatur, #64).

use zeroize::Zeroizing;

use crate::cbor::{self, MapLeser, Wert};
use crate::fehler::{Ergebnis, Fehler};
use crate::kette::{self, Huelle, Kopf};
use crate::modell::{FORMATVERSION, Id, SUITE_S1, kopf, kopf_pruefen, typ};
use crate::sodium::{self, Geheim, StromLeser, StromSchreiber, lp};
use crate::tresor::{
    self, KDF_MANIFEST, KDF_SICHERUNG, KapselInhalt, Recovery, bytesliste, bytesliste_aus,
};

pub const MAGIC: &[u8; 8] = b"DAKSICH1";
pub const SEP_SICHERUNG: &[u8] = b"dak-sicherung-v1";
pub const STUECK: usize = 64 * 1024;
pub const MAX_DATEI: usize = 512 * 1024 * 1024;

/// Alles, was in eine Sicherung gehört, als Bytes aus dem Speicher.
pub struct Bestand {
    pub gruendung: Vec<u8>,
    pub kette: Vec<Vec<u8>>,
    pub bund: Vec<u8>,
    pub bund_h: Vec<u8>,
    pub datensaetze: Vec<Vec<u8>>,
    pub tombstones: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub konto_id: [u8; 32],
    pub kettenkopf: [u8; 32],
    pub generation: u64,
    pub generation_h: u64,
    pub geraet_id: Id,
    pub sicherung_nr: u64,
    pub erstellt_um: i64,
    pub hashes: Vec<[u8; 32]>,
    pub tombstones: Vec<Vec<u8>>,
    pub bund_hash: [u8; 32],
    pub bund_h_hash: [u8; 32],
    pub kapsel_hash: [u8; 32],
    pub kapsel_h_hash: [u8; 32],
    pub kette_hash: [u8; 32],
}

impl Manifest {
    fn kodieren(&self) -> Vec<u8> {
        cbor::kodieren(
            &kopf(typ::MANIFEST)
                .mit(2, cbor::zahl(SUITE_S1))
                .mit(3, cbor::bytes(&self.konto_id))
                .mit(4, cbor::bytes(&self.kettenkopf))
                .mit(5, cbor::zahl(self.generation))
                .mit(6, cbor::zahl(self.generation_h))
                .mit(7, cbor::bytes(&self.geraet_id))
                .mit(8, cbor::zahl(self.sicherung_nr))
                .mit(9, cbor::zahl(self.erstellt_um))
                .mit(
                    10,
                    Wert::Feld(self.hashes.iter().map(|h| cbor::bytes(h)).collect()),
                )
                .mit(11, bytesliste(&self.tombstones))
                .mit(12, cbor::bytes(&self.bund_hash))
                .mit(13, cbor::bytes(&self.bund_h_hash))
                .mit(14, cbor::bytes(&self.kapsel_hash))
                .mit(15, cbor::bytes(&self.kapsel_h_hash))
                .mit(16, cbor::bytes(&self.kette_hash))
                .fertig(),
        )
    }

    fn dekodieren(b: &[u8]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
        kopf_pruefen(&mut m, typ::MANIFEST)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        let x = Manifest {
            konto_id: m.bytes_n(3)?,
            kettenkopf: m.bytes_n(4)?,
            generation: m.u64(5)?,
            generation_h: m.u64(6)?,
            geraet_id: m.bytes_n(7)?,
            sicherung_nr: m.u64(8)?,
            erstellt_um: m.i64(9)?,
            hashes: m
                .feld(10)?
                .into_iter()
                .map(|h| {
                    cbor::als_bytes(h)?
                        .try_into()
                        .map_err(|_| Fehler::Format("Hash"))
                })
                .collect::<Ergebnis<_>>()?,
            tombstones: bytesliste_aus(m.wert(11)?)?,
            bund_hash: m.bytes_n(12)?,
            bund_h_hash: m.bytes_n(13)?,
            kapsel_hash: m.bytes_n(14)?,
            kapsel_h_hash: m.bytes_n(15)?,
            kette_hash: m.bytes_n(16)?,
        };
        m.ende()?;
        Ok(x)
    }
}

fn kette_hash(gruendung: &[u8], kette: &[Vec<u8>]) -> [u8; 32] {
    let mut teile: Vec<Vec<u8>> = vec![lp(gruendung)];
    teile.extend(kette.iter().map(|g| lp(g)));
    let refs: Vec<&[u8]> = teile.iter().map(|v| &v[..]).collect();
    sodium::hash32(&refs)
}

fn datensatz_hashes(d: &[Vec<u8>]) -> Vec<[u8; 32]> {
    let mut h: Vec<[u8; 32]> = d.iter().map(|x| sodium::hash32(&[x])).collect();
    h.sort();
    h
}

fn strom_ad(konto_id: &[u8; 32]) -> Vec<u8> {
    let mut ad = lp(SEP_SICHERUNG);
    ad.extend(FORMATVERSION.to_be_bytes());
    ad.extend(lp(konto_id));
    ad
}

pub struct Schreibdaten<'a> {
    pub bestand: &'a Bestand,
    pub kk: &'a Geheim,
    pub kapsel: &'a [u8],
    pub kapsel_h: &'a [u8],
    pub kopf: &'a Kopf,
    pub geraet_id: Id,
    pub sicherung_nr: u64,
    pub erstellt_um: i64,
}

/// Erzeugt die Bytes einer Sicherungsdatei.
pub fn schreiben(s: Schreibdaten<'_>) -> Vec<u8> {
    let b = s.bestand;
    let manifest = Manifest {
        konto_id: s.kopf.konto_id,
        kettenkopf: s.kopf.kopf_hash,
        generation: s.kopf.generation,
        generation_h: s.kopf.generation_h,
        geraet_id: s.geraet_id,
        sicherung_nr: s.sicherung_nr,
        erstellt_um: s.erstellt_um,
        hashes: datensatz_hashes(&b.datensaetze),
        tombstones: b.tombstones.clone(),
        bund_hash: sodium::hash32(&[&b.bund]),
        bund_h_hash: sodium::hash32(&[&b.bund_h]),
        kapsel_hash: sodium::hash32(&[s.kapsel]),
        kapsel_h_hash: sodium::hash32(&[s.kapsel_h]),
        kette_hash: kette_hash(&b.gruendung, &b.kette),
    }
    .kodieren();
    let mac = sodium::mac32(&sodium::kdf(s.kk, KDF_MANIFEST, 1), &[&manifest]);
    let innen = Zeroizing::new(cbor::kodieren(
        &kopf(typ::SICHERUNG_INNEN)
            .mit(3, cbor::bytes(&b.gruendung))
            .mit(4, bytesliste(&b.kette))
            .mit(5, cbor::bytes(&b.bund))
            .mit(6, cbor::bytes(&b.bund_h))
            .mit(7, bytesliste(&b.datensaetze))
            .mit(8, bytesliste(&b.tombstones))
            .mit(9, cbor::bytes(&manifest))
            .mit(10, cbor::bytes(&mac))
            .fertig(),
    ));

    let (mut w, stromkopf) = StromSchreiber::neu(&sodium::kdf(s.kk, KDF_SICHERUNG, 1));
    let kopf_b = cbor::kodieren(
        &kopf(typ::SICHERUNG_KOPF)
            .mit(2, cbor::zahl(SUITE_S1))
            .mit(3, cbor::bytes(s.kapsel))
            .mit(4, cbor::bytes(s.kapsel_h))
            .mit(5, cbor::bytes(&stromkopf))
            .fertig(),
    );
    let mut out = MAGIC.to_vec();
    out.extend((kopf_b.len() as u32).to_be_bytes());
    out.extend(&kopf_b);
    let ad = strom_ad(&s.kopf.konto_id);
    let stuecke: Vec<&[u8]> = if innen.is_empty() {
        vec![&[][..]]
    } else {
        innen.chunks(STUECK).collect()
    };
    let n = stuecke.len();
    for (i, st) in stuecke.into_iter().enumerate() {
        let c = w.schreiben(st, &ad, i + 1 == n);
        out.extend((c.len() as u32).to_be_bytes());
        out.extend(c);
    }
    out
}

/// Öffentlicher Kopf einer Sicherungsdatei.
pub struct DateiKopf {
    pub kapsel: Vec<u8>,
    pub kapsel_h: Vec<u8>,
    pub stromkopf: [u8; 24],
    pub rest_ab: usize,
}

pub fn kopf_lesen(d: &[u8]) -> Ergebnis<DateiKopf> {
    if d.len() > MAX_DATEI || d.len() < 12 || &d[..8] != MAGIC {
        return Err(Fehler::Format("keine Sicherungsdatei"));
    }
    let n = u32::from_be_bytes(d[8..12].try_into().expect("4")) as usize;
    if n > 4096 || 12 + n > d.len() {
        return Err(Fehler::Format("Sicherungskopf"));
    }
    let mut m = MapLeser::aus(cbor::dekodieren(&d[12..12 + n])?)?;
    kopf_pruefen(&mut m, typ::SICHERUNG_KOPF)?;
    if m.u64(2)? != SUITE_S1 {
        return Err(Fehler::Format("Suite"));
    }
    let k = DateiKopf {
        kapsel: m.bytes(3)?,
        kapsel_h: m.bytes(4)?,
        stromkopf: m.bytes_n(5)?,
        rest_ab: 12 + n,
    };
    m.ende()?;
    Ok(k)
}

/// Eine vollständig geprüfte Sicherung.
pub struct Geprueft {
    pub bestand: Bestand,
    pub manifest: Manifest,
    pub kopf: Kopf,
    pub kk: Geheim,
    pub kk_h: Geheim,
    pub kapsel: Vec<u8>,
    pub kapsel_h: Vec<u8>,
}

/// Öffnet und prüft eine Sicherung mit den Recovery-Schlüsseln (Protokoll 10.1, Ablauf #87):
/// Kapsel, Innenteil, Ankerregel, Kontokette, Manifest-MAC, Hashes, Kapsel_h.
pub fn pruefen_mit_recovery(d: &[u8], rk: &Recovery) -> Ergebnis<Geprueft> {
    let k = kopf_lesen(d)?;
    let KapselInhalt {
        konto_id,
        generation,
        kk,
    } = tresor::kapsel_oeffnen(&k.kapsel, &rk.enc, false).map_err(|_| Fehler::Woerter)?;
    let g = innen_pruefen(d, &k, &kk, &konto_id)?;
    // Ankerregel: die Wörter müssen den am Kettenkopf gültigen Recovery-Schlüsseln entsprechen.
    if g.kopf.rk_sig != rk.sig.oeffentlich || g.kopf.rk_enc != rk.enc.oeffentlich {
        return Err(Fehler::Kette(
            "Recovery-Wörter gehören nicht zum Kettenkopf",
        ));
    }
    if g.kopf.generation != generation {
        return Err(Fehler::Integritaet("Generation der Kapsel"));
    }
    let kh = tresor::kapsel_oeffnen(&k.kapsel_h, &rk.enc, true)?;
    if kh.konto_id != konto_id
        || kh.generation != g.kopf.generation_h
        || g.manifest.kapsel_h_hash != sodium::hash32(&[&k.kapsel_h])
    {
        return Err(Fehler::Integritaet("Kapsel_h"));
    }
    Ok(Geprueft {
        bestand: g.bestand,
        manifest: g.manifest,
        kopf: g.kopf,
        kk,
        kk_h: kh.kk,
        kapsel: k.kapsel,
        kapsel_h: k.kapsel_h,
    })
}

/// Liest eine eigene Sicherung mit `KK` zurück (Prüfung nach dem Schreiben, AE-17).
pub fn pruefen_mit_kk(d: &[u8], kk: &Geheim, konto_id: &[u8; 32]) -> Ergebnis<Manifest> {
    let k = kopf_lesen(d)?;
    Ok(innen_pruefen(d, &k, kk, konto_id)?.manifest)
}

struct Innen {
    bestand: Bestand,
    manifest: Manifest,
    kopf: Kopf,
}

fn innen_pruefen(d: &[u8], k: &DateiKopf, kk: &Geheim, konto_id: &[u8; 32]) -> Ergebnis<Innen> {
    let ad = strom_ad(konto_id);
    let mut r = StromLeser::neu(&sodium::kdf(kk, KDF_SICHERUNG, 1), &k.stromkopf)?;
    let mut pos = k.rest_ab;
    let mut innen = Zeroizing::new(Vec::new());
    let mut ende = false;
    while pos < d.len() {
        if ende {
            return Err(Fehler::Integritaet("Daten nach dem letzten Stück"));
        }
        if pos + 4 > d.len() {
            return Err(Fehler::Integritaet("abgeschnitten"));
        }
        let n = u32::from_be_bytes(d[pos..pos + 4].try_into().expect("4")) as usize;
        pos += 4;
        if n > STUECK + sodium::STREAM_ABYTES || pos + n > d.len() {
            return Err(Fehler::Integritaet("abgeschnitten"));
        }
        let (teil, letztes) = r.lesen(&d[pos..pos + n], &ad)?;
        innen.extend_from_slice(&teil);
        pos += n;
        ende = letztes;
    }
    if !ende {
        return Err(Fehler::Integritaet("abgeschnitten"));
    }
    let mut m = MapLeser::aus(cbor::dekodieren(&innen)?)?;
    kopf_pruefen(&mut m, typ::SICHERUNG_INNEN)?;
    let bestand = Bestand {
        gruendung: m.bytes(3)?,
        kette: bytesliste_aus(m.wert(4)?)?,
        bund: m.bytes(5)?,
        bund_h: m.bytes(6)?,
        datensaetze: bytesliste_aus(m.wert(7)?)?,
        tombstones: bytesliste_aus(m.wert(8)?)?,
    };
    let manifest_b = m.bytes(9)?;
    let mac: [u8; 32] = m.bytes_n(10)?;
    m.ende()?;
    let soll = sodium::mac32(&sodium::kdf(kk, KDF_MANIFEST, 1), &[&manifest_b]);
    if !sodium::gleich_konstant(&soll, &mac) {
        return Err(Fehler::Integritaet("Manifest-MAC"));
    }
    let manifest = Manifest::dekodieren(&manifest_b)?;
    let gruendung = Huelle::dekodieren(&bestand.gruendung)?;
    let glieder = bestand
        .kette
        .iter()
        .map(|g| Huelle::dekodieren(g))
        .collect::<Ergebnis<Vec<_>>>()?;
    let kopf = kette::pruefen(&gruendung, &glieder)?;
    if kopf.konto_id != *konto_id || manifest.konto_id != *konto_id {
        return Err(Fehler::Integritaet("Konto der Sicherung"));
    }
    if manifest.kettenkopf != kopf.kopf_hash
        || manifest.generation != kopf.generation
        || manifest.generation_h != kopf.generation_h
    {
        return Err(Fehler::Integritaet("Manifest passt nicht zur Kette"));
    }
    if manifest.kette_hash != kette_hash(&bestand.gruendung, &bestand.kette)
        || manifest.bund_hash != sodium::hash32(&[&bestand.bund])
        || manifest.bund_h_hash != sodium::hash32(&[&bestand.bund_h])
        || manifest.kapsel_hash != sodium::hash32(&[&k.kapsel])
        || manifest.hashes != datensatz_hashes(&bestand.datensaetze)
        || manifest.tombstones != bestand.tombstones
    {
        return Err(Fehler::Integritaet("Hash im Manifest"));
    }
    Ok(Innen {
        bestand,
        manifest,
        kopf,
    })
}

/// Dateiname: Kürzel der konto_id, Gerätekennung, Generation, laufende Nummer (Protokoll 10.2).
pub fn dateiname(konto_id: &[u8; 32], geraet_id: &Id, generation: u64, nr: u64) -> String {
    format!(
        "dak-{}-{}-g{}-{:06}.daks",
        &crate::modell::hex(konto_id)[..8],
        &crate::modell::hex(geraet_id)[..8],
        generation,
        nr
    )
}

/// Kürzel aus einem Dateinamen: (Konto, Gerät, Nummer).
pub fn dateiname_lesen(name: &str) -> Option<(String, String, u64)> {
    let s = name.strip_prefix("dak-")?.strip_suffix(".daks")?;
    let t: Vec<&str> = s.split('-').collect();
    if t.len() != 4 || t[0].len() != 8 || t[1].len() != 8 {
        return None;
    }
    Some((t[0].to_owned(), t[1].to_owned(), t[3].parse().ok()?))
}

/// Aufbewahrung (Protokoll 10.2): die letzten zehn Sicherungen und je eine pro Tag der letzten
/// 30 Tage behalten; nur Dateien der eigenen Kette (gleiches Kontokürzel) werden gelöscht.
/// `dateien`: (Name, Tagesnummer der Änderung). Liefert die zu löschenden Namen.
pub fn aufbewahrung(konto_kuerzel: &str, dateien: &[(String, i64)], heute: i64) -> Vec<String> {
    let mut eigene: Vec<(u64, &String, i64)> = dateien
        .iter()
        .filter_map(|(n, t)| {
            dateiname_lesen(n)
                .filter(|(k, _, _)| k == konto_kuerzel)
                .map(|(_, _, nr)| (nr, n, *t))
        })
        .collect();
    eigene.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(a.1)));
    let mut behalten: Vec<&String> = eigene.iter().take(10).map(|(_, n, _)| *n).collect();
    let mut tage_gesehen = std::collections::BTreeSet::new();
    for (_, n, t) in &eigene {
        if heute - *t < 30 && tage_gesehen.insert(*t) {
            behalten.push(n);
        }
    }
    eigene
        .iter()
        .filter(|(_, n, _)| !behalten.contains(n))
        .map(|(_, n, _)| (*n).clone())
        .collect()
}
