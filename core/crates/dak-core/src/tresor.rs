// SPDX-License-Identifier: MPL-2.0
//! Schlüsselbund, Datensätze und Kapseln (Protokoll 7, 8, 10.1).
//!
//! Teilschlüssel werden nur per `crypto_kdf` aus `KK` bzw. `KK_h` abgeleitet; die Wurzeln selbst
//! verschlüsseln nichts. Kontexte und Nummern stehen in `docs/format.md`.

use zeroize::Zeroizing;

use crate::cbor::{self, MapBau, MapLeser, Wert, als_feld};
use crate::fehler::{Ergebnis, Fehler};
use crate::modell::{
    self, Einstellungen, FORMATVERSION, Formular, Id, Klasse, Messdef, Objekt, Praeparat, SUITE_S1,
    kopf, kopf_pruefen, typ,
};
use crate::sodium::{self, BoxPaar, Geheim, Zufall, lp};

pub const KDF_BUND: &[u8; 8] = b"dakbund_";
pub const KDF_BUND_H: &[u8; 8] = b"dakbundh";
pub const KDF_SICHERUNG: &[u8; 8] = b"daksich_";
pub const KDF_MANIFEST: &[u8; 8] = b"dakmanif";
pub const KDF_RECOVERY: &[u8; 8] = b"dakrecov";
pub const KDF_ENTWURF: &[u8; 8] = b"dakentw_";

pub const SEP_BUND: &[u8] = b"dak-bund-v1";
pub const SEP_DATENSATZ: &[u8] = b"dak-datensatz-v1";
pub const SEP_UMHUELLUNG: &[u8] = b"dak-umhuellung-v1";
pub const SEP_KAPSEL: &str = "dak-kapsel-v1";
pub const SEP_KAPSEL_H: &str = "dak-kapsel-h-v1";

pub const BLOCK: usize = 256;

/// Recovery-Schlüssel aus den Wörtern: getrennte Paare für Signatur und Verschlüsselung.
pub struct Recovery {
    pub sig: sodium::SignPaar,
    pub enc: BoxPaar,
}

impl Recovery {
    pub fn aus_rk(rk: &Geheim) -> Self {
        let s = sodium::kdf(rk, KDF_RECOVERY, 1);
        let e = sodium::kdf(rk, KDF_RECOVERY, 2);
        Recovery {
            sig: sodium::SignPaar::aus_saat(&s),
            enc: BoxPaar::aus_saat(&e),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Bund (Klasse Gesundheit) und hochsensibler Bund
// ---------------------------------------------------------------------------------------------

pub struct Bund {
    pub version: u64,
    pub generation: u64,
    pub ds: Geheim,
    pub ws_saat: Geheim,
    pub dk_sig_saat: Geheim,
    pub dk_enc_saat: Geheim,
    pub messdefs: Vec<Messdef>,
    pub formulare: Vec<Formular>,
    pub praeparate: Vec<Praeparat>,
    pub einstellungen: Einstellungen,
}

impl Bund {
    pub fn ws(&self) -> BoxPaar {
        BoxPaar::aus_saat(&self.ws_saat)
    }
    pub fn dk_sig(&self) -> sodium::SignPaar {
        sodium::SignPaar::aus_saat(&self.dk_sig_saat)
    }
    pub fn dk_enc(&self) -> BoxPaar {
        BoxPaar::aus_saat(&self.dk_enc_saat)
    }

    fn wert(&self, konto_id: &[u8; 32]) -> Wert {
        kopf(typ::BUND)
            .mit(2, cbor::zahl(SUITE_S1))
            .mit(3, cbor::bytes(konto_id))
            .mit(4, cbor::zahl(self.version))
            .mit(5, cbor::zahl(self.generation))
            .mit(6, cbor::bytes(self.ds.bytes()))
            .mit(7, cbor::bytes(self.ws_saat.bytes()))
            .mit(8, cbor::bytes(self.dk_sig_saat.bytes()))
            .mit(9, cbor::bytes(self.dk_enc_saat.bytes()))
            .mit(
                10,
                Wert::Feld(self.messdefs.iter().map(|m| m.wert()).collect()),
            )
            .mit(
                11,
                Wert::Feld(self.formulare.iter().map(|f| f.wert()).collect()),
            )
            .mit(
                12,
                Wert::Feld(self.praeparate.iter().map(|p| p.wert()).collect()),
            )
            .mit(13, self.einstellungen.wert())
            .fertig()
    }

    fn aus(w: Wert, konto_id: &[u8; 32]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        kopf_pruefen(&mut m, typ::BUND)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        if m.bytes_n::<32>(3)? != *konto_id {
            return Err(Fehler::Integritaet("Bund eines anderen Kontos"));
        }
        let version = m.u64(4)?;
        let generation = m.u64(5)?;
        let geheim = |b: Vec<u8>| -> Ergebnis<Geheim> {
            let b = Zeroizing::new(b);
            if b.len() != 32 {
                return Err(Fehler::Format("Schlüssellänge"));
            }
            Ok(Geheim::aus(&b))
        };
        let ds = geheim(m.bytes(6)?)?;
        let ws_saat = geheim(m.bytes(7)?)?;
        let dk_sig_saat = geheim(m.bytes(8)?)?;
        let dk_enc_saat = geheim(m.bytes(9)?)?;
        let messdefs = m
            .feld(10)?
            .into_iter()
            .map(Messdef::aus)
            .collect::<Ergebnis<Vec<_>>>()?;
        let formulare = m
            .feld(11)?
            .into_iter()
            .map(Formular::aus)
            .collect::<Ergebnis<Vec<_>>>()?;
        let praeparate = m
            .feld(12)?
            .into_iter()
            .map(Praeparat::aus)
            .collect::<Ergebnis<Vec<_>>>()?;
        let einstellungen = Einstellungen::aus(m.wert(13)?)?;
        m.ende()?;
        Ok(Bund {
            version,
            generation,
            ds,
            ws_saat,
            dk_sig_saat,
            dk_enc_saat,
            messdefs,
            formulare,
            praeparate,
            einstellungen,
        })
    }
}

pub struct BundH {
    pub version: u64,
    pub generation_h: u64,
    pub ws_saat: Geheim,
    pub messdefs: Vec<Messdef>,
    /// Neutrale Bezeichnungen je Platznummer (ab M1b).
    pub bezeichnungen: Vec<(u16, String)>,
}

impl BundH {
    pub fn ws(&self) -> BoxPaar {
        BoxPaar::aus_saat(&self.ws_saat)
    }

    fn wert(&self, konto_id: &[u8; 32]) -> Wert {
        kopf(typ::BUND_H)
            .mit(2, cbor::zahl(SUITE_S1))
            .mit(3, cbor::bytes(konto_id))
            .mit(4, cbor::zahl(self.version))
            .mit(5, cbor::zahl(self.generation_h))
            .mit(6, cbor::bytes(self.ws_saat.bytes()))
            .mit(
                7,
                Wert::Feld(self.messdefs.iter().map(|m| m.wert()).collect()),
            )
            .mit(
                8,
                Wert::Feld(
                    self.bezeichnungen
                        .iter()
                        .map(|(n, t)| {
                            MapBau::neu()
                                .mit(0, cbor::zahl(*n))
                                .mit(1, cbor::text(t))
                                .fertig()
                        })
                        .collect(),
                ),
            )
            .fertig()
    }

    fn aus(w: Wert, konto_id: &[u8; 32]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(w)?;
        kopf_pruefen(&mut m, typ::BUND_H)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        if m.bytes_n::<32>(3)? != *konto_id {
            return Err(Fehler::Integritaet("Bund eines anderen Kontos"));
        }
        let version = m.u64(4)?;
        let generation_h = m.u64(5)?;
        let s = Zeroizing::new(m.bytes(6)?);
        if s.len() != 32 {
            return Err(Fehler::Format("Schlüssellänge"));
        }
        let ws_saat = Geheim::aus(&s);
        let messdefs = m
            .feld(7)?
            .into_iter()
            .map(Messdef::aus)
            .collect::<Ergebnis<Vec<_>>>()?;
        let mut bezeichnungen = Vec::new();
        for b in m.feld(8)? {
            let mut bm = MapLeser::aus(b)?;
            bezeichnungen.push((
                u16::try_from(bm.u64(0)?).map_err(|_| Fehler::Format("Platz"))?,
                bm.text(1)?,
            ));
            bm.ende()?;
        }
        m.ende()?;
        Ok(BundH {
            version,
            generation_h,
            ws_saat,
            messdefs,
            bezeichnungen,
        })
    }
}

fn bund_ad(klasse: u8, konto_id: &[u8; 32], kettenkopf: &[u8; 32], version: u64) -> Vec<u8> {
    let mut ad = lp(SEP_BUND);
    ad.extend(lp(konto_id));
    ad.extend(lp(kettenkopf));
    ad.extend(version.to_be_bytes());
    ad.push(klasse);
    ad
}

fn blob(
    klasse: u8,
    version: u64,
    kettenkopf: &[u8; 32],
    nonce: &[u8; 24],
    chiffrat: &[u8],
) -> Vec<u8> {
    cbor::kodieren(
        &kopf(typ::BUND_BLOB)
            .mit(2, cbor::zahl(SUITE_S1))
            .mit(3, cbor::zahl(klasse))
            .mit(4, cbor::zahl(version))
            .mit(5, cbor::bytes(kettenkopf))
            .mit(6, cbor::bytes(nonce))
            .mit(7, cbor::bytes(chiffrat))
            .fertig(),
    )
}

struct BlobTeile {
    klasse: u8,
    version: u64,
    kettenkopf: [u8; 32],
    nonce: [u8; 24],
    chiffrat: Vec<u8>,
}

fn blob_lesen(b: &[u8]) -> Ergebnis<BlobTeile> {
    let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
    kopf_pruefen(&mut m, typ::BUND_BLOB)?;
    if m.u64(2)? != SUITE_S1 {
        return Err(Fehler::Format("Suite"));
    }
    let t = BlobTeile {
        klasse: u8::try_from(m.u64(3)?).map_err(|_| Fehler::Format("Klasse"))?,
        version: m.u64(4)?,
        kettenkopf: m.bytes_n(5)?,
        nonce: m.bytes_n(6)?,
        chiffrat: m.bytes(7)?,
    };
    m.ende()?;
    Ok(t)
}

pub fn bund_verschluesseln(
    b: &Bund,
    kk: &Geheim,
    konto_id: &[u8; 32],
    kettenkopf: &[u8; 32],
    z: &mut dyn Zufall,
) -> Vec<u8> {
    let k = sodium::kdf(kk, KDF_BUND, 1);
    let nonce = sodium::zufall_bytes::<24>(z);
    let klar = Zeroizing::new(cbor::kodieren(&b.wert(konto_id)));
    let c = sodium::aead_verschluesseln(
        &k,
        &nonce,
        &klar,
        &bund_ad(1, konto_id, kettenkopf, b.version),
    );
    blob(1, b.version, kettenkopf, &nonce, &c)
}

/// Entschlüsselt den Bund und prüft die Bindung an Konto und Kettenkopf.
pub fn bund_entschluesseln(
    blob_b: &[u8],
    kk: &Geheim,
    konto_id: &[u8; 32],
    kettenkopf: &[u8; 32],
) -> Ergebnis<Bund> {
    let t = blob_lesen(blob_b)?;
    if t.klasse != 1 || t.kettenkopf != *kettenkopf {
        return Err(Fehler::Integritaet("Bund passt nicht zum Kettenkopf"));
    }
    let k = sodium::kdf(kk, KDF_BUND, 1);
    let klar = sodium::aead_entschluesseln(
        &k,
        &t.nonce,
        &t.chiffrat,
        &bund_ad(1, konto_id, kettenkopf, t.version),
    )?;
    let b = Bund::aus(cbor::dekodieren(&klar)?, konto_id)?;
    if b.version != t.version {
        return Err(Fehler::Integritaet("Bundversion"));
    }
    Ok(b)
}

pub fn bund_h_verschluesseln(
    b: &BundH,
    kk_h: &Geheim,
    konto_id: &[u8; 32],
    kettenkopf: &[u8; 32],
    z: &mut dyn Zufall,
) -> Vec<u8> {
    let k = sodium::kdf(kk_h, KDF_BUND_H, 1);
    let nonce = sodium::zufall_bytes::<24>(z);
    let klar = Zeroizing::new(cbor::kodieren(&b.wert(konto_id)));
    let c = sodium::aead_verschluesseln(
        &k,
        &nonce,
        &klar,
        &bund_ad(2, konto_id, kettenkopf, b.version),
    );
    blob(2, b.version, kettenkopf, &nonce, &c)
}

pub fn bund_h_entschluesseln(
    blob_b: &[u8],
    kk_h: &Geheim,
    konto_id: &[u8; 32],
    kettenkopf: &[u8; 32],
) -> Ergebnis<BundH> {
    let t = blob_lesen(blob_b)?;
    if t.klasse != 2 || t.kettenkopf != *kettenkopf {
        return Err(Fehler::Integritaet("Bund passt nicht zum Kettenkopf"));
    }
    let k = sodium::kdf(kk_h, KDF_BUND_H, 1);
    let klar = sodium::aead_entschluesseln(
        &k,
        &t.nonce,
        &t.chiffrat,
        &bund_ad(2, konto_id, kettenkopf, t.version),
    )?;
    let b = BundH::aus(cbor::dekodieren(&klar)?, konto_id)?;
    if b.version != t.version {
        return Err(Fehler::Integritaet("Bundversion"));
    }
    Ok(b)
}

// ---------------------------------------------------------------------------------------------
// Datensätze
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Art {
    Inhalt = 1,
    Korrektur = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weg {
    /// `dk` mit dem Datenschlüssel der Klasse und Generation verschlüsselt.
    Datenschluessel = 1,
    /// `dk` an den Schreibschlüssel der Klasse Gesundheit versiegelt (vorläufig, Entwürfe).
    SchreibGesundheit = 2,
    /// `dk` an den hochsensiblen Schreibschlüssel versiegelt.
    SchreibHochsensibel = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Umhuellung {
    pub weg: Weg,
    pub generation: u64,
    pub nonce: Option<[u8; 24]>,
    pub chiffrat: Vec<u8>,
}

impl Umhuellung {
    fn kodieren(&self) -> Vec<u8> {
        cbor::kodieren(
            &kopf(typ::UMHUELLUNG)
                .mit(2, cbor::zahl(SUITE_S1))
                .mit(3, cbor::zahl(self.weg as u8))
                .mit(4, cbor::zahl(self.generation))
                .opt(5, self.nonce.map(|n| cbor::bytes(&n)))
                .mit(6, cbor::bytes(&self.chiffrat))
                .fertig(),
        )
    }

    fn dekodieren(b: &[u8]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
        kopf_pruefen(&mut m, typ::UMHUELLUNG)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        let weg = match m.u64(3)? {
            1 => Weg::Datenschluessel,
            2 => Weg::SchreibGesundheit,
            3 => Weg::SchreibHochsensibel,
            _ => return Err(Fehler::Format("Umhüllungsweg")),
        };
        let u = Umhuellung {
            weg,
            generation: m.u64(4)?,
            nonce: m
                .opt_bytes(5)?
                .map(|n| n.try_into().map_err(|_| Fehler::Format("Nonce")))
                .transpose()?,
            chiffrat: m.bytes(6)?,
        };
        m.ende()?;
        if (u.weg == Weg::Datenschluessel) != u.nonce.is_some() {
            return Err(Fehler::Format("Umhüllung"));
        }
        Ok(u)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Datensatz {
    pub datensatz_id: Id,
    pub revision_id: Id,
    pub vorgaenger: Option<Id>,
    pub art: Art,
    pub klasse: Klasse,
    pub umhuellung: Umhuellung,
    pub nonce: [u8; 24],
    pub chiffrat: Vec<u8>,
}

impl Datensatz {
    pub fn kodieren(&self) -> Vec<u8> {
        cbor::kodieren(
            &kopf(typ::DATENSATZ)
                .mit(2, cbor::zahl(SUITE_S1))
                .mit(3, cbor::bytes(&self.datensatz_id))
                .mit(4, cbor::bytes(&self.revision_id))
                .opt(5, self.vorgaenger.as_ref().map(|v| cbor::bytes(v)))
                .mit(6, cbor::zahl(self.art as u8))
                .mit(7, cbor::zahl(self.klasse as u8))
                .mit(8, cbor::bytes(&self.umhuellung.kodieren()))
                .mit(9, cbor::bytes(&self.nonce))
                .mit(10, cbor::bytes(&self.chiffrat))
                .fertig(),
        )
    }

    pub fn dekodieren(b: &[u8]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
        kopf_pruefen(&mut m, typ::DATENSATZ)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        let d = Datensatz {
            datensatz_id: m.bytes_n(3)?,
            revision_id: m.bytes_n(4)?,
            vorgaenger: m
                .opt_bytes(5)?
                .map(|v| v.try_into().map_err(|_| Fehler::Format("ID-Länge")))
                .transpose()?,
            art: match m.u64(6)? {
                1 => Art::Inhalt,
                2 => Art::Korrektur,
                _ => return Err(Fehler::Format("Datensatzart")),
            },
            klasse: Klasse::aus(u8::try_from(m.u64(7)?).map_err(|_| Fehler::Format("Klasse"))?)?,
            umhuellung: Umhuellung::dekodieren(&m.bytes(8)?)?,
            nonce: m.bytes_n(9)?,
            chiffrat: m.bytes(10)?,
        };
        m.ende()?;
        Ok(d)
    }

    fn ad(&self, konto_id: &[u8; 32]) -> Vec<u8> {
        let mut ad = lp(SEP_DATENSATZ);
        ad.extend(lp(konto_id));
        ad.extend(lp(&self.datensatz_id));
        ad.extend(lp(&self.revision_id));
        ad.extend(lp(self.vorgaenger.as_ref().map(|v| &v[..]).unwrap_or(&[])));
        ad.push(self.art as u8);
        ad.push(self.klasse as u8);
        ad.extend(FORMATVERSION.to_be_bytes());
        ad
    }

    fn umhuellung_ad(&self, konto_id: &[u8; 32], generation: u64) -> Vec<u8> {
        let mut ad = lp(SEP_UMHUELLUNG);
        ad.extend(lp(konto_id));
        ad.extend(lp(&self.datensatz_id));
        ad.extend(lp(&self.revision_id));
        ad.push(self.klasse as u8);
        ad.extend(generation.to_be_bytes());
        ad
    }

    fn versiegelt_klartext(&self, dk: &Geheim) -> Zeroizing<Vec<u8>> {
        Zeroizing::new(cbor::kodieren(
            &MapBau::neu()
                .mit(0, cbor::bytes(&self.datensatz_id))
                .mit(1, cbor::bytes(&self.revision_id))
                .mit(2, cbor::bytes(dk.bytes()))
                .fertig(),
        ))
    }

    /// Prüft, dass die Schutzklasse zu den Messdefinitionen des Objekts passt (Protokoll 6.2).
    pub fn klasse_pruefen(objekt: &Objekt, klasse: Klasse, messdefs: &[Messdef]) -> Ergebnis<()> {
        for a in objekt.antworten() {
            let md = messdefs
                .iter()
                .find(|m| m.id == a.messdef)
                .ok_or(Fehler::Eingabe("unbekannte Messdefinition"))?;
            if md.klasse != klasse && !(md.klasse == Klasse::Normal && klasse == Klasse::Gesundheit)
            {
                return Err(Fehler::Eingabe("Schutzklasse passt nicht"));
            }
        }
        Ok(())
    }
}

/// Neuer Datensatz bzw. neue Revision mit frischem `dk`.
pub struct NeuerDatensatz<'a> {
    pub konto_id: &'a [u8; 32],
    pub datensatz_id: Id,
    pub vorgaenger: Option<Id>,
    pub klasse: Klasse,
    pub objekt: &'a Objekt,
}

pub enum Ziel<'a> {
    Datenschluessel {
        ds: &'a Geheim,
        generation: u64,
    },
    Schreibschluessel {
        oeffentlich: &'a [u8; 32],
        weg: Weg,
        generation: u64,
    },
}

pub fn versiegeln(n: NeuerDatensatz<'_>, ziel: Ziel<'_>, z: &mut dyn Zufall) -> Datensatz {
    let dk = Geheim::zufall(32, z);
    let revision_id: Id = sodium::zufall_bytes::<16>(z);
    let nonce = sodium::zufall_bytes::<24>(z);
    let mut d = Datensatz {
        datensatz_id: n.datensatz_id,
        revision_id,
        vorgaenger: n.vorgaenger,
        art: if n.vorgaenger.is_some() {
            Art::Korrektur
        } else {
            Art::Inhalt
        },
        klasse: n.klasse,
        umhuellung: Umhuellung {
            weg: Weg::Datenschluessel,
            generation: 0,
            nonce: None,
            chiffrat: Vec::new(),
        },
        nonce,
        chiffrat: Vec::new(),
    };
    let klar = sodium::auffuellen(&Zeroizing::new(cbor::kodieren(&n.objekt.wert())), BLOCK);
    d.chiffrat = sodium::aead_verschluesseln(&dk, &nonce, &klar, &d.ad(n.konto_id));
    d.umhuellung = umhuellen(&d, &dk, n.konto_id, ziel, z);
    d
}

fn umhuellen(
    d: &Datensatz,
    dk: &Geheim,
    konto_id: &[u8; 32],
    ziel: Ziel<'_>,
    z: &mut dyn Zufall,
) -> Umhuellung {
    match ziel {
        Ziel::Datenschluessel { ds, generation } => {
            let un = sodium::zufall_bytes::<24>(z);
            let c = sodium::aead_verschluesseln(
                ds,
                &un,
                dk.bytes(),
                &d.umhuellung_ad(konto_id, generation),
            );
            Umhuellung {
                weg: Weg::Datenschluessel,
                generation,
                nonce: Some(un),
                chiffrat: c,
            }
        }
        Ziel::Schreibschluessel {
            oeffentlich,
            weg,
            generation,
        } => {
            let c = sodium::versiegeln(&d.versiegelt_klartext(dk), oeffentlich);
            Umhuellung {
                weg,
                generation,
                nonce: None,
                chiffrat: c,
            }
        }
    }
}

/// Holt `dk` aus der Umhüllung.
pub fn dk_holen(
    d: &Datensatz,
    konto_id: &[u8; 32],
    ds: Option<&Geheim>,
    ws: Option<&BoxPaar>,
) -> Ergebnis<Geheim> {
    match d.umhuellung.weg {
        Weg::Datenschluessel => {
            let ds = ds.ok_or(Fehler::Gesperrt)?;
            let n = d.umhuellung.nonce.ok_or(Fehler::Format("Nonce"))?;
            let k = sodium::aead_entschluesseln(
                ds,
                &n,
                &d.umhuellung.chiffrat,
                &d.umhuellung_ad(konto_id, d.umhuellung.generation),
            )?;
            if k.len() != 32 {
                return Err(Fehler::Integritaet("dk-Länge"));
            }
            Ok(Geheim::aus(&k))
        }
        Weg::SchreibGesundheit | Weg::SchreibHochsensibel => {
            let ws = ws.ok_or(Fehler::Gesperrt)?;
            let klar = ws.oeffnen(&d.umhuellung.chiffrat)?;
            let mut m = MapLeser::aus(cbor::dekodieren(&klar)?)?;
            if m.bytes_n::<16>(0)? != d.datensatz_id || m.bytes_n::<16>(1)? != d.revision_id {
                return Err(Fehler::Integritaet(
                    "Umhüllung gehört zu einem anderen Datensatz",
                ));
            }
            let k = Zeroizing::new(m.bytes(2)?);
            m.ende()?;
            if k.len() != 32 {
                return Err(Fehler::Integritaet("dk-Länge"));
            }
            Ok(Geheim::aus(&k))
        }
    }
}

pub fn oeffnen(d: &Datensatz, konto_id: &[u8; 32], dk: &Geheim) -> Ergebnis<Objekt> {
    let klar = sodium::aead_entschluesseln(dk, &d.nonce, &d.chiffrat, &d.ad(konto_id))?;
    let klar = sodium::auffuellung_entfernen(klar, BLOCK)?;
    Objekt::aus(cbor::dekodieren(&klar)?)
}

/// Rotation und Übernahme: `dk` neu umhüllen, Datensatz bleibt.
pub fn umhuellen_neu(
    d: &mut Datensatz,
    konto_id: &[u8; 32],
    dk: &Geheim,
    ziel: Ziel<'_>,
    z: &mut dyn Zufall,
) {
    d.umhuellung = umhuellen(d, dk, konto_id, ziel, z);
}

// ---------------------------------------------------------------------------------------------
// Kapseln (Protokoll 10.1)
// ---------------------------------------------------------------------------------------------

pub fn kapsel(
    konto_id: &[u8; 32],
    generation: u64,
    kk: &Geheim,
    rk_enc: &[u8; 32],
    hochsensibel: bool,
) -> Vec<u8> {
    let t = if hochsensibel {
        typ::KAPSEL_H
    } else {
        typ::KAPSEL
    };
    let sep = if hochsensibel {
        SEP_KAPSEL_H
    } else {
        SEP_KAPSEL
    };
    let klar = Zeroizing::new(cbor::kodieren(
        &kopf(t)
            .mit(2, cbor::text(sep))
            .mit(3, cbor::bytes(konto_id))
            .mit(4, cbor::zahl(generation))
            .mit(5, cbor::bytes(kk.bytes()))
            .fertig(),
    ));
    sodium::versiegeln(&klar, rk_enc)
}

pub struct KapselInhalt {
    pub konto_id: [u8; 32],
    pub generation: u64,
    pub kk: Geheim,
}

pub fn kapsel_oeffnen(c: &[u8], rk_enc: &BoxPaar, hochsensibel: bool) -> Ergebnis<KapselInhalt> {
    let klar = rk_enc.oeffnen(c)?;
    let mut m = MapLeser::aus(cbor::dekodieren(&klar)?)?;
    kopf_pruefen(
        &mut m,
        if hochsensibel {
            typ::KAPSEL_H
        } else {
            typ::KAPSEL
        },
    )?;
    if m.text(2)?
        != if hochsensibel {
            SEP_KAPSEL_H
        } else {
            SEP_KAPSEL
        }
    {
        return Err(Fehler::Format("Kapsel-Trenner"));
    }
    let konto_id = m.bytes_n(3)?;
    let generation = m.u64(4)?;
    let kk = Zeroizing::new(m.bytes(5)?);
    m.ende()?;
    if kk.len() != 32 {
        return Err(Fehler::Format("Schlüssellänge"));
    }
    Ok(KapselInhalt {
        konto_id,
        generation,
        kk: Geheim::aus(&kk),
    })
}

/// Liste von Bytefolgen in CBOR (für Sicherung und Manifest).
pub fn bytesliste(v: &[Vec<u8>]) -> Wert {
    Wert::Feld(v.iter().map(|b| cbor::bytes(b)).collect())
}

pub fn bytesliste_aus(w: Wert) -> Ergebnis<Vec<Vec<u8>>> {
    als_feld(w)?.into_iter().map(cbor::als_bytes).collect()
}

pub fn neue_id(z: &mut dyn Zufall) -> Id {
    sodium::zufall_bytes::<16>(z)
}

pub use modell::hex;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modell::{
        Antwort, Antwortwert, Ausloeser, ErfasstUeber, FormularArt, Status, Zustand,
    };
    use crate::sodium::TestZufall;
    use crate::zeit::Zeitpunkt;

    fn objekt() -> Objekt {
        Objekt::Status(Status {
            id: [5; 16],
            zeitpunkt: Zeitpunkt {
                utc_ms: 1,
                versatz_min: 0,
                zone: "UTC".into(),
            },
            erfasst_um: 1,
            ausloeser: Ausloeser::Selbst,
            abfrage: None,
            erfasst_ueber: ErfasstUeber::App,
            vorlaeufig: false,
            bezug_von: None,
            bezug_bis: None,
            antworten: vec![Antwort {
                messdef: [6; 16],
                zustand: Zustand::Beantwortet,
                wert: Some(Antwortwert {
                    zahl: Some(2),
                    ..Default::default()
                }),
                aktiv_gesetzt: true,
            }],
            lebenstag: 0,
            formular: [7; 16],
            art: FormularArt::StatusJetzt,
        })
    }

    #[test]
    fn datensatz_rundweg_und_bindung() {
        let mut z = TestZufall::neu(21);
        let konto = [1u8; 32];
        let ds = Geheim::zufall(32, &mut z);
        let o = objekt();
        let d = versiegeln(
            NeuerDatensatz {
                konto_id: &konto,
                datensatz_id: [9; 16],
                vorgaenger: None,
                klasse: Klasse::Gesundheit,
                objekt: &o,
            },
            Ziel::Datenschluessel {
                ds: &ds,
                generation: 1,
            },
            &mut z,
        );
        let b = d.kodieren();
        let d2 = Datensatz::dekodieren(&b).unwrap();
        let dk = dk_holen(&d2, &konto, Some(&ds), None).unwrap();
        assert_eq!(oeffnen(&d2, &konto, &dk).unwrap(), o);
        // anderes Konto: Umhüllung passt nicht
        assert!(dk_holen(&d2, &[2; 32], Some(&ds), None).is_err());
        // Klasse im Klartextkopf hochgesetzt: AEAD schlägt an
        let mut d3 = d2.clone();
        d3.klasse = Klasse::Normal;
        assert!(oeffnen(&d3, &konto, &dk).is_err());
        // Umhüllung eines anderen Datensatzes eingesetzt
        let d4 = versiegeln(
            NeuerDatensatz {
                konto_id: &konto,
                datensatz_id: [8; 16],
                vorgaenger: None,
                klasse: Klasse::Gesundheit,
                objekt: &o,
            },
            Ziel::Datenschluessel {
                ds: &ds,
                generation: 1,
            },
            &mut z,
        );
        let mut d5 = d2.clone();
        d5.umhuellung = d4.umhuellung.clone();
        assert!(dk_holen(&d5, &konto, Some(&ds), None).is_err());
        // Auffüllung: Chiffrat ist ein Vielfaches des Blocks plus Tag
        assert_eq!((d2.chiffrat.len() - 16) % BLOCK, 0);
    }

    #[test]
    fn versiegelt_an_schreibschluessel() {
        let mut z = TestZufall::neu(22);
        let konto = [1u8; 32];
        let ws = BoxPaar::aus_saat(&Geheim::zufall(32, &mut z));
        let o = objekt();
        let d = versiegeln(
            NeuerDatensatz {
                konto_id: &konto,
                datensatz_id: [9; 16],
                vorgaenger: None,
                klasse: Klasse::Gesundheit,
                objekt: &o,
            },
            Ziel::Schreibschluessel {
                oeffentlich: &ws.oeffentlich,
                weg: Weg::SchreibGesundheit,
                generation: 1,
            },
            &mut z,
        );
        assert!(dk_holen(&d, &konto, None, None).is_err());
        let dk = dk_holen(&d, &konto, None, Some(&ws)).unwrap();
        assert_eq!(oeffnen(&d, &konto, &dk).unwrap(), o);
    }

    #[test]
    fn kapsel_rundweg() {
        let mut z = TestZufall::neu(23);
        let rk = Recovery::aus_rk(&Geheim::zufall(32, &mut z));
        let kk = Geheim::zufall(32, &mut z);
        let c = kapsel(&[3; 32], 4, &kk, &rk.enc.oeffentlich, false);
        let i = kapsel_oeffnen(&c, &rk.enc, false).unwrap();
        assert_eq!((i.konto_id, i.generation), ([3; 32], 4));
        assert!(i.kk.gleich(kk.bytes()));
        assert!(
            kapsel_oeffnen(&c, &rk.enc, true).is_err(),
            "normale Kapsel ist keine Kapsel_h"
        );
    }
}
