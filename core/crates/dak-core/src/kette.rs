// SPDX-License-Identifier: MPL-2.0
//! Identität und Kontokette (Protokoll 4).
//!
//! Gründungsobjekt mit Geräte- und Recovery-Signatur, `konto_id` als Hash darüber; jede spätere
//! Änderung ist ein signiertes Glied mit laufender Nummer und Vorgänger-Hash. Signaturen gibt es
//! nur hier, nie über Werte.

use crate::cbor::{self, MapBau, MapLeser, Wert};
use crate::fehler::{Ergebnis, Fehler};
use crate::modell::{Id, SUITE_S1, kopf, kopf_pruefen, typ};
use crate::sodium::{self, SignPaar};

pub const SEP_GRUENDUNG: &[u8] = b"dak-gruendung-v1";
pub const SEP_KETTE: &[u8] = b"dak-kontokette-v1";
pub const SEP_MARKER: &[u8] = b"dak-ersetzung-v1";

/// Signiertes Objekt: Inhalt als Bytes plus eine oder zwei Signaturen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Huelle {
    pub inhalt: Vec<u8>,
    pub sig1: [u8; 64],
    pub sig2: Option<[u8; 64]>,
}

impl Huelle {
    pub fn kodieren(&self) -> Vec<u8> {
        cbor::kodieren(
            &MapBau::neu()
                .mit(0, cbor::bytes(&self.inhalt))
                .mit(1, cbor::bytes(&self.sig1))
                .opt(2, self.sig2.map(|s| cbor::bytes(&s)))
                .fertig(),
        )
    }

    pub fn dekodieren(b: &[u8]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
        let h = Huelle {
            inhalt: m.bytes(0)?,
            sig1: m.bytes_n(1)?,
            sig2: m
                .opt_bytes(2)?
                .map(|s| s.try_into().map_err(|_| Fehler::Format("Signaturlänge")))
                .transpose()?,
        };
        m.ende()?;
        Ok(h)
    }
}

fn signierbar(sep: &[u8], inhalt: &[u8]) -> Vec<u8> {
    let mut v = sep.to_vec();
    v.extend_from_slice(inhalt);
    v
}

// ---------------------------------------------------------------------------------------------
// Gründung
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gruendung {
    pub geraet_id: Id,
    pub dk_sig: [u8; 32],
    pub dk_enc: [u8; 32],
    pub rk_sig: [u8; 32],
    pub rk_enc: [u8; 32],
    pub erstellt_um: i64,
}

impl Gruendung {
    fn wert(&self) -> Wert {
        kopf(typ::GRUENDUNG)
            .mit(2, cbor::zahl(SUITE_S1))
            .mit(3, cbor::bytes(&self.geraet_id))
            .mit(4, cbor::bytes(&self.dk_sig))
            .mit(5, cbor::bytes(&self.dk_enc))
            .mit(6, cbor::bytes(&self.rk_sig))
            .mit(7, cbor::bytes(&self.rk_enc))
            .mit(8, cbor::zahl(self.erstellt_um))
            .fertig()
    }

    fn aus(b: &[u8]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
        kopf_pruefen(&mut m, typ::GRUENDUNG)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        let g = Gruendung {
            geraet_id: m.bytes_n(3)?,
            dk_sig: m.bytes_n(4)?,
            dk_enc: m.bytes_n(5)?,
            rk_sig: m.bytes_n(6)?,
            rk_enc: m.bytes_n(7)?,
            erstellt_um: m.i64(8)?,
        };
        m.ende()?;
        Ok(g)
    }

    /// Signiert mit Gerät und Recovery-Schlüssel.
    pub fn signieren(&self, geraet: &SignPaar, recovery: &SignPaar) -> Huelle {
        assert_eq!(geraet.oeffentlich, self.dk_sig);
        assert_eq!(recovery.oeffentlich, self.rk_sig);
        let inhalt = cbor::kodieren(&self.wert());
        let s = signierbar(SEP_GRUENDUNG, &inhalt);
        Huelle {
            sig1: geraet.signieren(&s),
            sig2: Some(recovery.signieren(&s)),
            inhalt,
        }
    }
}

/// Prüft eine Gründungshülle und liefert Inhalt und `konto_id`.
pub fn gruendung_pruefen(h: &Huelle) -> Ergebnis<(Gruendung, [u8; 32])> {
    let g = Gruendung::aus(&h.inhalt)?;
    let s = signierbar(SEP_GRUENDUNG, &h.inhalt);
    let sig2 = h
        .sig2
        .ok_or(Fehler::Kette("Gründung ohne Recovery-Signatur"))?;
    if !sodium::signatur_pruefen(&g.dk_sig, &s, &h.sig1)
        || !sodium::signatur_pruefen(&g.rk_sig, &s, &sig2)
    {
        return Err(Fehler::Kette("Signatur der Gründung"));
    }
    Ok((g, konto_id(h)))
}

pub fn konto_id(h: &Huelle) -> [u8; 32] {
    sodium::hash32(&[SEP_GRUENDUNG, &h.kodieren()])
}

// ---------------------------------------------------------------------------------------------
// Glieder
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotationsgrund {
    Wiederherstellung = 1,
    RecoveryNeu = 2,
    AufWunsch = 3,
    PinAktiviert = 4,
    SuiteWechsel = 5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GliedArt {
    Rotation {
        generation: u64,
        generation_h: u64,
        grund: Rotationsgrund,
    },
    GeraetErsetzt {
        neues_geraet: Id,
        dk_sig: [u8; 32],
        dk_enc: [u8; 32],
        letztes_glied_alt: u64,
    },
    RecoveryNeu {
        rk_sig: [u8; 32],
        rk_enc: [u8; 32],
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glied {
    pub nr: u64,
    pub vorgaenger: [u8; 32],
    pub geraet_id: Id,
    pub erstellt_um: i64,
    pub art: GliedArt,
}

impl Glied {
    fn wert(&self) -> Wert {
        let b = kopf(typ::GLIED)
            .mit(2, cbor::zahl(SUITE_S1))
            .mit(3, cbor::zahl(self.nr))
            .mit(4, cbor::bytes(&self.vorgaenger))
            .mit(6, cbor::bytes(&self.geraet_id))
            .mit(7, cbor::zahl(self.erstellt_um));
        match &self.art {
            GliedArt::Rotation {
                generation,
                generation_h,
                grund,
            } => b
                .mit(5, cbor::zahl(1))
                .mit(8, cbor::zahl(*generation))
                .mit(9, cbor::zahl(*generation_h))
                .mit(10, cbor::zahl(*grund as u8)),
            GliedArt::GeraetErsetzt {
                neues_geraet,
                dk_sig,
                dk_enc,
                letztes_glied_alt,
            } => b
                .mit(5, cbor::zahl(2))
                .mit(11, cbor::bytes(neues_geraet))
                .mit(12, cbor::bytes(dk_sig))
                .mit(13, cbor::bytes(dk_enc))
                .mit(14, cbor::zahl(*letztes_glied_alt)),
            GliedArt::RecoveryNeu { rk_sig, rk_enc } => b
                .mit(5, cbor::zahl(3))
                .mit(15, cbor::bytes(rk_sig))
                .mit(16, cbor::bytes(rk_enc)),
        }
        .fertig()
    }

    fn aus(b: &[u8]) -> Ergebnis<Self> {
        let mut m = MapLeser::aus(cbor::dekodieren(b)?)?;
        kopf_pruefen(&mut m, typ::GLIED)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        let nr = m.u64(3)?;
        let vorgaenger = m.bytes_n(4)?;
        let art_code = m.u64(5)?;
        let geraet_id = m.bytes_n(6)?;
        let erstellt_um = m.i64(7)?;
        let art = match art_code {
            1 => GliedArt::Rotation {
                generation: m.u64(8)?,
                generation_h: m.u64(9)?,
                grund: match m.u64(10)? {
                    1 => Rotationsgrund::Wiederherstellung,
                    2 => Rotationsgrund::RecoveryNeu,
                    3 => Rotationsgrund::AufWunsch,
                    4 => Rotationsgrund::PinAktiviert,
                    5 => Rotationsgrund::SuiteWechsel,
                    _ => return Err(Fehler::Format("Rotationsgrund")),
                },
            },
            2 => GliedArt::GeraetErsetzt {
                neues_geraet: m.bytes_n(11)?,
                dk_sig: m.bytes_n(12)?,
                dk_enc: m.bytes_n(13)?,
                letztes_glied_alt: m.u64(14)?,
            },
            3 => GliedArt::RecoveryNeu {
                rk_sig: m.bytes_n(15)?,
                rk_enc: m.bytes_n(16)?,
            },
            _ => return Err(Fehler::Format("Gliedart")),
        };
        m.ende()?;
        Ok(Glied {
            nr,
            vorgaenger,
            geraet_id,
            erstellt_um,
            art,
        })
    }

    /// Signiert ein Glied. `sig2` ist bei `GeraetErsetzt` das neue Gerät, `sig1` dort `RK_sig`.
    pub fn signieren(&self, sig1: &SignPaar, sig2: Option<&SignPaar>) -> Huelle {
        let inhalt = cbor::kodieren(&self.wert());
        let s = signierbar(SEP_KETTE, &inhalt);
        Huelle {
            sig1: sig1.signieren(&s),
            sig2: sig2.map(|p| p.signieren(&s)),
            inhalt,
        }
    }
}

pub fn glied_hash(h: &Huelle) -> [u8; 32] {
    sodium::hash32(&[SEP_KETTE, &h.kodieren()])
}

/// Zustand am Kopf einer geprüften Kette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kopf {
    pub konto_id: [u8; 32],
    pub kopf_hash: [u8; 32],
    pub laenge: u64,
    pub geraet_id: Id,
    pub dk_sig: [u8; 32],
    pub dk_enc: [u8; 32],
    pub rk_sig: [u8; 32],
    pub rk_enc: [u8; 32],
    pub generation: u64,
    pub generation_h: u64,
    /// Ersetzte Geräte: (Gerät, Gliednummer der Ersetzung).
    pub ersetzt: Vec<(Id, u64)>,
    /// Hashes aller Glieder in Reihenfolge (für Verwandtschaftsprüfung).
    pub hashes: Vec<[u8; 32]>,
}

/// Prüft Gründung und alle Glieder in Reihenfolge.
pub fn pruefen(gruendung: &Huelle, glieder: &[Huelle]) -> Ergebnis<Kopf> {
    let (g, konto_id) = gruendung_pruefen(gruendung)?;
    let mut k = Kopf {
        konto_id,
        kopf_hash: konto_id,
        laenge: 0,
        geraet_id: g.geraet_id,
        dk_sig: g.dk_sig,
        dk_enc: g.dk_enc,
        rk_sig: g.rk_sig,
        rk_enc: g.rk_enc,
        generation: 1,
        generation_h: 1,
        ersetzt: Vec::new(),
        hashes: Vec::new(),
    };
    for h in glieder {
        let gl = Glied::aus(&h.inhalt)?;
        if gl.nr != k.laenge + 1 {
            return Err(Fehler::Kette("Gliednummer"));
        }
        if gl.vorgaenger != k.kopf_hash {
            return Err(Fehler::Kette("Vorgänger-Hash"));
        }
        let s = signierbar(SEP_KETTE, &h.inhalt);
        match &gl.art {
            GliedArt::Rotation {
                generation,
                generation_h,
                ..
            } => {
                if gl.geraet_id != k.geraet_id
                    || !sodium::signatur_pruefen(&k.dk_sig, &s, &h.sig1)
                    || h.sig2.is_some()
                {
                    return Err(Fehler::Kette("Rotation nicht vom aktuellen Gerät signiert"));
                }
                let neu_g = *generation == k.generation + 1;
                let neu_h = *generation_h == k.generation_h + 1;
                if !(neu_g || neu_h)
                    || (*generation != k.generation && !neu_g)
                    || (*generation_h != k.generation_h && !neu_h)
                {
                    return Err(Fehler::Kette("Generationsfolge"));
                }
                k.generation = *generation;
                k.generation_h = *generation_h;
            }
            GliedArt::GeraetErsetzt {
                neues_geraet,
                dk_sig,
                dk_enc,
                letztes_glied_alt,
            } => {
                let sig2 = h
                    .sig2
                    .ok_or(Fehler::Kette("Ersetzung ohne Gerätesignatur"))?;
                if !sodium::signatur_pruefen(&k.rk_sig, &s, &h.sig1)
                    || !sodium::signatur_pruefen(dk_sig, &s, &sig2)
                {
                    return Err(Fehler::Kette("Signatur der Ersetzung"));
                }
                if gl.geraet_id != *neues_geraet
                    || *letztes_glied_alt > k.laenge
                    || *neues_geraet == k.geraet_id
                {
                    return Err(Fehler::Kette("Ersetzung"));
                }
                k.ersetzt.push((k.geraet_id, gl.nr));
                k.geraet_id = *neues_geraet;
                k.dk_sig = *dk_sig;
                k.dk_enc = *dk_enc;
            }
            GliedArt::RecoveryNeu { rk_sig, rk_enc } => {
                if gl.geraet_id != k.geraet_id
                    || !sodium::signatur_pruefen(&k.dk_sig, &s, &h.sig1)
                    || h.sig2.is_some()
                {
                    return Err(Fehler::Kette(
                        "Recovery-Wechsel nicht vom aktuellen Gerät signiert",
                    ));
                }
                k.rk_sig = *rk_sig;
                k.rk_enc = *rk_enc;
            }
        }
        let hh = glied_hash(h);
        k.hashes.push(hh);
        k.kopf_hash = hh;
        k.laenge = gl.nr;
    }
    Ok(k)
}

// ---------------------------------------------------------------------------------------------
// Ersetzungsmarker (Protokoll 10.2): liegt nach einer Wiederherstellung im Sicherungsordner
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ersetzungsmarker {
    pub konto_id: [u8; 32],
    pub ersetztes_geraet: Id,
    pub neues_geraet: Id,
    pub glied_nr: u64,
}

impl Ersetzungsmarker {
    pub fn signieren(&self, rk_sig: &SignPaar) -> Vec<u8> {
        let inhalt = cbor::kodieren(
            &kopf(typ::ERSETZUNGSMARKER)
                .mit(2, cbor::zahl(SUITE_S1))
                .mit(3, cbor::bytes(&self.konto_id))
                .mit(4, cbor::bytes(&self.ersetztes_geraet))
                .mit(5, cbor::bytes(&self.neues_geraet))
                .mit(6, cbor::zahl(self.glied_nr))
                .fertig(),
        );
        let s = signierbar(SEP_MARKER, &inhalt);
        Huelle {
            sig1: rk_sig.signieren(&s),
            sig2: None,
            inhalt,
        }
        .kodieren()
    }

    /// Prüft gegen die am Kettenkopf gültigen Recovery-Schlüssel der eigenen Kette.
    pub fn pruefen(b: &[u8], kopf: &Kopf) -> Ergebnis<Self> {
        let h = Huelle::dekodieren(b)?;
        let s = signierbar(SEP_MARKER, &h.inhalt);
        if !sodium::signatur_pruefen(&kopf.rk_sig, &s, &h.sig1) {
            return Err(Fehler::Kette("Signatur des Markers"));
        }
        let mut m = MapLeser::aus(cbor::dekodieren(&h.inhalt)?)?;
        kopf_pruefen(&mut m, typ::ERSETZUNGSMARKER)?;
        if m.u64(2)? != SUITE_S1 {
            return Err(Fehler::Format("Suite"));
        }
        let e = Ersetzungsmarker {
            konto_id: m.bytes_n(3)?,
            ersetztes_geraet: m.bytes_n(4)?,
            neues_geraet: m.bytes_n(5)?,
            glied_nr: m.u64(6)?,
        };
        m.ende()?;
        if e.konto_id != kopf.konto_id {
            return Err(Fehler::Kette("Marker eines anderen Kontos"));
        }
        Ok(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sodium::{Geheim, TestZufall};

    fn paar(z: &mut TestZufall) -> SignPaar {
        SignPaar::aus_saat(&Geheim::zufall(32, z))
    }

    struct Aufbau {
        geraet: SignPaar,
        rk: SignPaar,
        gruendung: Huelle,
        glieder: Vec<Huelle>,
    }

    fn aufbau() -> Aufbau {
        let mut z = TestZufall::neu(7);
        let geraet = paar(&mut z);
        let rk = paar(&mut z);
        let g = Gruendung {
            geraet_id: [1; 16],
            dk_sig: geraet.oeffentlich,
            dk_enc: [2; 32],
            rk_sig: rk.oeffentlich,
            rk_enc: [3; 32],
            erstellt_um: 0,
        };
        let gruendung = g.signieren(&geraet, &rk);
        Aufbau {
            geraet,
            rk,
            gruendung,
            glieder: Vec::new(),
        }
    }

    fn rotation(a: &mut Aufbau, k: &Kopf) {
        let gl = Glied {
            nr: k.laenge + 1,
            vorgaenger: k.kopf_hash,
            geraet_id: k.geraet_id,
            erstellt_um: 1,
            art: GliedArt::Rotation {
                generation: k.generation + 1,
                generation_h: k.generation_h,
                grund: Rotationsgrund::AufWunsch,
            },
        };
        a.glieder.push(gl.signieren(&a.geraet, None));
    }

    #[test]
    fn kette_mit_rotation_und_ersetzung() {
        let mut a = aufbau();
        let k0 = pruefen(&a.gruendung, &a.glieder).unwrap();
        rotation(&mut a, &k0);
        let k1 = pruefen(&a.gruendung, &a.glieder).unwrap();
        assert_eq!(k1.generation, 2);

        let mut z = TestZufall::neu(9);
        let neu = paar(&mut z);
        let gl = Glied {
            nr: 2,
            vorgaenger: k1.kopf_hash,
            geraet_id: [9; 16],
            erstellt_um: 2,
            art: GliedArt::GeraetErsetzt {
                neues_geraet: [9; 16],
                dk_sig: neu.oeffentlich,
                dk_enc: [4; 32],
                letztes_glied_alt: 1,
            },
        };
        a.glieder.push(gl.signieren(&a.rk, Some(&neu)));
        let k2 = pruefen(&a.gruendung, &a.glieder).unwrap();
        assert_eq!(k2.geraet_id, [9; 16]);
        assert_eq!(k2.ersetzt, vec![([1; 16], 2)]);

        // Das alte Gerät darf danach nichts mehr signieren.
        let alt = Glied {
            nr: 3,
            vorgaenger: k2.kopf_hash,
            geraet_id: [1; 16],
            erstellt_um: 3,
            art: GliedArt::Rotation {
                generation: 3,
                generation_h: 1,
                grund: Rotationsgrund::AufWunsch,
            },
        };
        let mut b = a.glieder.clone();
        b.push(alt.signieren(&a.geraet, None));
        assert!(pruefen(&a.gruendung, &b).is_err());
    }

    #[test]
    fn manipulationen_werden_erkannt() {
        let mut a = aufbau();
        let k0 = pruefen(&a.gruendung, &a.glieder).unwrap();
        rotation(&mut a, &k0);
        let k1 = pruefen(&a.gruendung, &a.glieder).unwrap();
        rotation(&mut a, &k1);
        assert!(pruefen(&a.gruendung, &a.glieder).is_ok());
        // Abgeschnitten in der Mitte: Glied 2 ohne Glied 1
        assert!(pruefen(&a.gruendung, &a.glieder[1..]).is_err());
        // Vertauscht
        let v = vec![a.glieder[1].clone(), a.glieder[0].clone()];
        assert!(pruefen(&a.gruendung, &v).is_err());
        // Inhalt verändert
        let mut x = a.glieder.clone();
        x[0].inhalt[5] ^= 1;
        assert!(pruefen(&a.gruendung, &x).is_err());
        // Fremde Gründung mit eigenen Gliedern
        let mut z = TestZufall::neu(11);
        let g2 = Gruendung {
            geraet_id: [1; 16],
            dk_sig: a.geraet.oeffentlich,
            dk_enc: [2; 32],
            rk_sig: a.rk.oeffentlich,
            rk_enc: [3; 32],
            erstellt_um: 99,
        };
        let fremd = g2.signieren(&a.geraet, &a.rk);
        assert!(pruefen(&fremd, &a.glieder).is_err());
        let _ = paar(&mut z);
        // Generation springt
        let k = pruefen(&a.gruendung, &a.glieder).unwrap();
        let sprung = Glied {
            nr: k.laenge + 1,
            vorgaenger: k.kopf_hash,
            geraet_id: k.geraet_id,
            erstellt_um: 5,
            art: GliedArt::Rotation {
                generation: k.generation + 2,
                generation_h: 1,
                grund: Rotationsgrund::AufWunsch,
            },
        };
        let mut y = a.glieder.clone();
        y.push(sprung.signieren(&a.geraet, None));
        assert!(pruefen(&a.gruendung, &y).is_err());
    }
}
