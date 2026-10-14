// SPDX-License-Identifier: MPL-2.0
//! Strenges, deterministisches CBOR (RFC 8949, Abschnitt 4.2.1) für alle Formate des Kerns.
//!
//! Erlaubt sind nur: ganze Zahlen (positiv und negativ, kürzeste Form), Bytefolgen, Text
//! (gültiges UTF-8), Felder, Maps mit ganzzahligen Schlüsseln 0 bis 23 in aufsteigender
//! Reihenfolge, `true`, `false`, `null`. Unbestimmte Längen, Fließkommazahlen, Tags, doppelte
//! oder unbekannte Schlüssel werden abgelehnt (Protokoll Abschnitt 3, Architektur AE-14).
//!
//! Abweichung von AE-14: statt einer Schicht auf `minicbor` ein eigener, kleiner Codec. Er
//! braucht keine Abhängigkeit, und jede Regel steht an einer Stelle; `ciborium` dient in den
//! Tests als unabhängige Gegenprobe.

use crate::fehler::{Ergebnis, Fehler};

pub const MAX_TIEFE: usize = 16;
pub const MAX_LAENGE: usize = 64 * 1024 * 1024;
pub const MAX_SCHLUESSEL: u64 = 23;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wert {
    Zahl(i128),
    Bytes(Vec<u8>),
    Text(String),
    Feld(Vec<Wert>),
    Map(Vec<(u64, Wert)>),
    Bool(bool),
    Null,
}

impl Drop for Wert {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        match self {
            Wert::Bytes(b) => b.zeroize(),
            Wert::Text(t) => t.zeroize(),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Schreiben
// ---------------------------------------------------------------------------------------------

fn kopf(out: &mut Vec<u8>, haupttyp: u8, n: u64) {
    let mt = haupttyp << 5;
    if n < 24 {
        out.push(mt | n as u8);
    } else if n <= u8::MAX as u64 {
        out.push(mt | 24);
        out.push(n as u8);
    } else if n <= u16::MAX as u64 {
        out.push(mt | 25);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    } else if n <= u32::MAX as u64 {
        out.push(mt | 26);
        out.extend_from_slice(&(n as u32).to_be_bytes());
    } else {
        out.push(mt | 27);
        out.extend_from_slice(&n.to_be_bytes());
    }
}

pub fn schreiben(w: &Wert, out: &mut Vec<u8>) {
    match w {
        Wert::Zahl(z) => {
            if *z >= 0 {
                kopf(out, 0, u64::try_from(*z).expect("Zahl zu groß"));
            } else {
                kopf(out, 1, u64::try_from(-1 - *z).expect("Zahl zu klein"));
            }
        }
        Wert::Bytes(b) => {
            kopf(out, 2, b.len() as u64);
            out.extend_from_slice(b);
        }
        Wert::Text(t) => {
            kopf(out, 3, t.len() as u64);
            out.extend_from_slice(t.as_bytes());
        }
        Wert::Feld(f) => {
            kopf(out, 4, f.len() as u64);
            for e in f {
                schreiben(e, out);
            }
        }
        Wert::Map(m) => {
            let mut sortiert: Vec<&(u64, Wert)> = m.iter().collect();
            sortiert.sort_by_key(|(k, _)| *k);
            for p in sortiert.windows(2) {
                assert!(p[0].0 != p[1].0, "doppelter Map-Schlüssel");
            }
            kopf(out, 5, m.len() as u64);
            for (k, v) in sortiert {
                assert!(*k <= MAX_SCHLUESSEL, "Map-Schlüssel über 23");
                kopf(out, 0, *k);
                schreiben(v, out);
            }
        }
        Wert::Bool(false) => out.push(0xf4),
        Wert::Bool(true) => out.push(0xf5),
        Wert::Null => out.push(0xf6),
    }
}

pub fn kodieren(w: &Wert) -> Vec<u8> {
    let mut out = Vec::new();
    schreiben(w, &mut out);
    out
}

// ---------------------------------------------------------------------------------------------
// Lesen
// ---------------------------------------------------------------------------------------------

struct Leser<'a> {
    d: &'a [u8],
    pos: usize,
}

impl<'a> Leser<'a> {
    fn byte(&mut self) -> Ergebnis<u8> {
        let b = *self.d.get(self.pos).ok_or(Fehler::Format("CBOR zu kurz"))?;
        self.pos += 1;
        Ok(b)
    }

    fn nimm(&mut self, n: usize) -> Ergebnis<&'a [u8]> {
        if n > MAX_LAENGE || self.pos.checked_add(n).is_none_or(|e| e > self.d.len()) {
            return Err(Fehler::Format("CBOR-Länge"));
        }
        let s = &self.d[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn argument(&mut self, info: u8) -> Ergebnis<u64> {
        let n = match info {
            0..=23 => return Ok(info as u64),
            24 => {
                let v = self.byte()? as u64;
                if v < 24 {
                    return Err(Fehler::Format("CBOR nicht kürzeste Form"));
                }
                v
            }
            25 => {
                let v = u16::from_be_bytes(self.nimm(2)?.try_into().expect("2")) as u64;
                if v <= u8::MAX as u64 {
                    return Err(Fehler::Format("CBOR nicht kürzeste Form"));
                }
                v
            }
            26 => {
                let v = u32::from_be_bytes(self.nimm(4)?.try_into().expect("4")) as u64;
                if v <= u16::MAX as u64 {
                    return Err(Fehler::Format("CBOR nicht kürzeste Form"));
                }
                v
            }
            27 => {
                let v = u64::from_be_bytes(self.nimm(8)?.try_into().expect("8"));
                if v <= u32::MAX as u64 {
                    return Err(Fehler::Format("CBOR nicht kürzeste Form"));
                }
                v
            }
            _ => return Err(Fehler::Format("CBOR unbestimmte Länge oder reserviert")),
        };
        Ok(n)
    }

    fn wert(&mut self, tiefe: usize) -> Ergebnis<Wert> {
        if tiefe > MAX_TIEFE {
            return Err(Fehler::Format("CBOR zu tief"));
        }
        let b = self.byte()?;
        let (mt, info) = (b >> 5, b & 0x1f);
        match mt {
            0 => Ok(Wert::Zahl(self.argument(info)? as i128)),
            1 => Ok(Wert::Zahl(-1 - self.argument(info)? as i128)),
            2 => {
                let n = self.argument(info)? as usize;
                Ok(Wert::Bytes(self.nimm(n)?.to_vec()))
            }
            3 => {
                let n = self.argument(info)? as usize;
                let s = std::str::from_utf8(self.nimm(n)?)
                    .map_err(|_| Fehler::Format("CBOR-Text kein UTF-8"))?;
                Ok(Wert::Text(s.to_owned()))
            }
            4 => {
                let n = self.argument(info)? as usize;
                if n > self.d.len() - self.pos {
                    return Err(Fehler::Format("CBOR-Feldlänge"));
                }
                let mut f = Vec::with_capacity(n);
                for _ in 0..n {
                    f.push(self.wert(tiefe + 1)?);
                }
                Ok(Wert::Feld(f))
            }
            5 => {
                let n = self.argument(info)? as usize;
                if n > self.d.len() - self.pos {
                    return Err(Fehler::Format("CBOR-Maplänge"));
                }
                let mut m: Vec<(u64, Wert)> = Vec::with_capacity(n);
                for _ in 0..n {
                    let kb = self.byte()?;
                    if kb >> 5 != 0 {
                        return Err(Fehler::Format("CBOR-Mapschlüssel keine Zahl"));
                    }
                    let k = self.argument(kb & 0x1f)?;
                    if k > MAX_SCHLUESSEL {
                        return Err(Fehler::Format("CBOR-Mapschlüssel über 23"));
                    }
                    if let Some((letzter, _)) = m.last()
                        && k <= *letzter
                    {
                        return Err(Fehler::Format("CBOR-Mapschlüssel nicht aufsteigend"));
                    }
                    let v = self.wert(tiefe + 1)?;
                    m.push((k, v));
                }
                Ok(Wert::Map(m))
            }
            7 => match info {
                20 => Ok(Wert::Bool(false)),
                21 => Ok(Wert::Bool(true)),
                22 => Ok(Wert::Null),
                _ => Err(Fehler::Format("CBOR einfacher Wert nicht erlaubt")),
            },
            _ => Err(Fehler::Format("CBOR-Tag nicht erlaubt")),
        }
    }
}

/// Dekodiert genau einen Wert; Bytes dahinter sind ein Fehler.
pub fn dekodieren(d: &[u8]) -> Ergebnis<Wert> {
    if d.len() > MAX_LAENGE {
        return Err(Fehler::Format("CBOR zu groß"));
    }
    let mut l = Leser { d, pos: 0 };
    let w = l.wert(0)?;
    if l.pos != d.len() {
        return Err(Fehler::Format("CBOR mit Restbytes"));
    }
    Ok(w)
}

// ---------------------------------------------------------------------------------------------
// Bequemer Zugriff auf Maps mit Prüfung auf unbekannte Schlüssel
// ---------------------------------------------------------------------------------------------

/// Baut eine Map. `None`-Werte werden ausgelassen.
#[derive(Default)]
pub struct MapBau(Vec<(u64, Wert)>);

impl MapBau {
    pub fn neu() -> Self {
        MapBau(Vec::new())
    }
    pub fn mit(mut self, k: u64, v: Wert) -> Self {
        self.0.push((k, v));
        self
    }
    pub fn opt(self, k: u64, v: Option<Wert>) -> Self {
        match v {
            Some(v) => self.mit(k, v),
            None => self,
        }
    }
    pub fn fertig(self) -> Wert {
        Wert::Map(self.0)
    }
}

pub fn zahl<T: Into<i128>>(v: T) -> Wert {
    Wert::Zahl(v.into())
}
pub fn bytes(v: &[u8]) -> Wert {
    Wert::Bytes(v.to_vec())
}
pub fn text(v: &str) -> Wert {
    Wert::Text(v.to_owned())
}

/// Liest Felder einer Map. `ende()` verlangt, dass jeder Schlüssel gelesen wurde.
pub struct MapLeser {
    eintraege: Vec<(u64, Option<Wert>)>,
}

impl MapLeser {
    pub fn aus(mut w: Wert) -> Ergebnis<Self> {
        match &mut w {
            Wert::Map(m) => Ok(MapLeser {
                eintraege: m.drain(..).map(|(k, v)| (k, Some(v))).collect(),
            }),
            _ => Err(Fehler::Format("Map erwartet")),
        }
    }

    fn nimm(&mut self, k: u64) -> Option<Wert> {
        self.eintraege
            .iter_mut()
            .find(|(kk, _)| *kk == k)
            .and_then(|(_, v)| v.take())
    }

    pub fn wert(&mut self, k: u64) -> Ergebnis<Wert> {
        self.nimm(k).ok_or(Fehler::Format("Pflichtfeld fehlt"))
    }
    pub fn opt_wert(&mut self, k: u64) -> Option<Wert> {
        self.nimm(k)
    }
    pub fn zahl(&mut self, k: u64) -> Ergebnis<i128> {
        als_zahl(self.wert(k)?)
    }
    pub fn u64(&mut self, k: u64) -> Ergebnis<u64> {
        u64::try_from(self.zahl(k)?).map_err(|_| Fehler::Format("Zahl außerhalb"))
    }
    pub fn i64(&mut self, k: u64) -> Ergebnis<i64> {
        i64::try_from(self.zahl(k)?).map_err(|_| Fehler::Format("Zahl außerhalb"))
    }
    pub fn opt_i64(&mut self, k: u64) -> Ergebnis<Option<i64>> {
        match self.nimm(k) {
            None => Ok(None),
            Some(w) => Ok(Some(
                i64::try_from(als_zahl(w)?).map_err(|_| Fehler::Format("Zahl außerhalb"))?,
            )),
        }
    }
    pub fn bytes(&mut self, k: u64) -> Ergebnis<Vec<u8>> {
        als_bytes(self.wert(k)?)
    }
    pub fn bytes_n<const N: usize>(&mut self, k: u64) -> Ergebnis<[u8; N]> {
        self.bytes(k)?
            .try_into()
            .map_err(|_| Fehler::Format("Bytelänge"))
    }
    pub fn opt_bytes(&mut self, k: u64) -> Ergebnis<Option<Vec<u8>>> {
        self.nimm(k).map(als_bytes).transpose()
    }
    pub fn text(&mut self, k: u64) -> Ergebnis<String> {
        als_text(self.wert(k)?)
    }
    pub fn opt_text(&mut self, k: u64) -> Ergebnis<Option<String>> {
        self.nimm(k).map(als_text).transpose()
    }
    pub fn bool(&mut self, k: u64) -> Ergebnis<bool> {
        match self.wert(k)? {
            Wert::Bool(b) => Ok(b),
            _ => Err(Fehler::Format("Wahrheitswert erwartet")),
        }
    }
    pub fn feld(&mut self, k: u64) -> Ergebnis<Vec<Wert>> {
        als_feld(self.wert(k)?)
    }
    /// Alle Schlüssel gelesen? Sonst unbekanntes Feld, also Fehler.
    pub fn ende(self) -> Ergebnis<()> {
        if self.eintraege.iter().any(|(_, v)| v.is_some()) {
            return Err(Fehler::Format("unbekanntes Feld"));
        }
        Ok(())
    }
}

pub fn als_zahl(w: Wert) -> Ergebnis<i128> {
    match w {
        Wert::Zahl(z) => Ok(z),
        _ => Err(Fehler::Format("Zahl erwartet")),
    }
}
pub fn als_bytes(mut w: Wert) -> Ergebnis<Vec<u8>> {
    match &mut w {
        Wert::Bytes(b) => Ok(std::mem::take(b)),
        _ => Err(Fehler::Format("Bytes erwartet")),
    }
}
pub fn als_text(mut w: Wert) -> Ergebnis<String> {
    match &mut w {
        Wert::Text(t) => Ok(std::mem::take(t)),
        _ => Err(Fehler::Format("Text erwartet")),
    }
}
pub fn als_feld(mut w: Wert) -> Ergebnis<Vec<Wert>> {
    match &mut w {
        Wert::Feld(f) => Ok(std::mem::take(f)),
        _ => Err(Fehler::Format("Feld erwartet")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rund(w: Wert) {
        let k = kodieren(&w);
        assert_eq!(dekodieren(&k).unwrap(), w);
        // Gegenprobe mit einem unabhängigen Decoder.
        let fremd: ciborium::Value =
            ciborium::from_reader(&k[..]).expect("ciborium liest unsere Ausgabe");
        let mut nochmal = Vec::new();
        ciborium::into_writer(&fremd, &mut nochmal).unwrap();
        assert_eq!(nochmal, k, "ciborium schreibt dieselben Bytes");
    }

    #[test]
    fn rundwege() {
        for z in [
            0i128,
            1,
            23,
            24,
            255,
            256,
            65535,
            65536,
            u32::MAX as i128,
            u32::MAX as i128 + 1,
            -1,
            -24,
            -25,
            -256,
            -257,
            i64::MIN as i128,
        ] {
            rund(Wert::Zahl(z));
        }
        rund(Wert::Bytes(vec![1, 2, 3]));
        rund(Wert::Text("Größe".into()));
        rund(
            MapBau::neu()
                .mit(0, text("a"))
                .mit(3, zahl(1))
                .mit(23, Wert::Feld(vec![Wert::Null, Wert::Bool(true)]))
                .fertig(),
        );
    }

    #[test]
    fn bekannte_bytes_rfc8949() {
        // Anhang A von RFC 8949
        assert_eq!(kodieren(&zahl(1000)), [0x19, 0x03, 0xe8]);
        assert_eq!(kodieren(&zahl(-1000)), [0x39, 0x03, 0xe7]);
        assert_eq!(kodieren(&text("ü")), [0x62, 0xc3, 0xbc]);
        assert_eq!(
            kodieren(&MapBau::neu().mit(2, zahl(3)).mit(1, zahl(2)).fertig()),
            [0xa2, 0x01, 0x02, 0x02, 0x03]
        );
    }

    #[test]
    fn lehnt_ab() {
        let faelle: &[(&[u8], &str)] = &[
            (&[0x18, 0x05], "nicht kürzeste Form"),
            (&[0x19, 0x00, 0x10], "nicht kürzeste Form"),
            (&[0x5f, 0x41, 0x00, 0xff], "unbestimmte Länge"),
            (&[0xa2, 0x02, 0x00, 0x01, 0x00], "absteigende Schlüssel"),
            (&[0xa2, 0x01, 0x00, 0x01, 0x00], "doppelter Schlüssel"),
            (&[0xa1, 0x18, 0x18, 0x00], "Schlüssel über 23"),
            (&[0xa1, 0x61, 0x61, 0x00], "Textschlüssel"),
            (&[0xf9, 0x3c, 0x00], "Fließkomma"),
            (&[0xc1, 0x00], "Tag"),
            (&[0xf7], "undefined"),
            (&[0x62, 0xff, 0xfe], "kein UTF-8"),
            (&[0x01, 0x02], "Restbytes"),
            (&[0x45, 0x01], "zu kurz"),
            (
                &[0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
                "riesige Länge",
            ),
        ];
        for (b, warum) in faelle {
            assert!(dekodieren(b).is_err(), "muss abgelehnt werden: {warum}");
        }
        // Zu tief verschachtelt
        let mut tief = vec![0x81u8; MAX_TIEFE + 2];
        tief.push(0x00);
        assert!(dekodieren(&tief).is_err());
    }

    #[test]
    fn unbekanntes_feld_wird_abgelehnt() {
        let w = MapBau::neu().mit(0, zahl(1)).mit(5, zahl(2)).fertig();
        let mut m = MapLeser::aus(w).unwrap();
        assert_eq!(m.zahl(0).unwrap(), 1);
        assert!(m.ende().is_err());
    }
}
