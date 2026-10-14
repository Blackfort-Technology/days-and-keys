// SPDX-License-Identifier: MPL-2.0
//! Recovery-Wörter: 256 Bit Entropie als 24 Wörter nach BIP39 (Architektur AE-35).
//!
//! Eigene kleine Kodierung (Entropie plus 8 Bit Prüfsumme aus SHA-256, 11 Bit je Wort) auf der
//! englischen BIP39-Liste. Die Liste ist mit ihrer SHA-256 festgeschrieben.

use std::sync::OnceLock;

use crate::fehler::{Ergebnis, Fehler};
use crate::sodium::{self, Geheim};

const LISTE: &str = include_str!("../daten/bip39-english.txt");
pub const LISTE_SHA256: &str = "2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda";
pub const ANZAHL: usize = 24;

fn liste() -> &'static Vec<&'static str> {
    static L: OnceLock<Vec<&'static str>> = OnceLock::new();
    L.get_or_init(|| {
        let h = sodium::sha256(LISTE.as_bytes());
        let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, LISTE_SHA256, "Wortliste verändert");
        let v: Vec<&str> = LISTE.lines().collect();
        assert_eq!(v.len(), 2048);
        v
    })
}

/// 32 Byte Entropie in 24 Wörter.
pub fn kodieren(entropie: &Geheim) -> Vec<String> {
    assert_eq!(entropie.len(), 32);
    let pruef = sodium::sha256(entropie.bytes())[0];
    let mut bits = Geheim::neu(33);
    bits.bytes_mut()[..32].copy_from_slice(entropie.bytes());
    bits.bytes_mut()[32] = pruef;
    let b = bits.bytes();
    let l = liste();
    (0..ANZAHL)
        .map(|i| {
            let mut idx = 0usize;
            for j in 0..11 {
                let bit = i * 11 + j;
                let v = (b[bit / 8] >> (7 - bit % 8)) & 1;
                idx = (idx << 1) | v as usize;
            }
            l[idx].to_owned()
        })
        .collect()
}

/// Sucht ein Wort; die ersten vier Buchstaben genügen (in BIP39 eindeutig).
pub fn index(wort: &str) -> Option<usize> {
    let w = wort.trim().to_lowercase();
    let l = liste();
    if let Some(i) = l.iter().position(|x| *x == w) {
        return Some(i);
    }
    if w.chars().count() >= 4 {
        let treffer: Vec<usize> = l
            .iter()
            .enumerate()
            .filter(|(_, x)| x.starts_with(&w))
            .map(|(i, _)| i)
            .collect();
        if treffer.len() == 1 {
            return Some(treffer[0]);
        }
    }
    None
}

/// 24 Wörter zurück in 32 Byte Entropie, mit Prüfsumme.
pub fn dekodieren(woerter: &[String]) -> Ergebnis<Geheim> {
    if woerter.len() != ANZAHL {
        return Err(Fehler::Woerter);
    }
    let mut bits = Geheim::neu(33);
    for (i, w) in woerter.iter().enumerate() {
        let idx = index(w).ok_or(Fehler::Woerter)?;
        for j in 0..11 {
            let bit = i * 11 + j;
            if (idx >> (10 - j)) & 1 == 1 {
                bits.bytes_mut()[bit / 8] |= 1 << (7 - bit % 8);
            }
        }
    }
    let e = Geheim::aus(&bits.bytes()[..32]);
    if sodium::sha256(e.bytes())[0] != bits.bytes()[32] {
        return Err(Fehler::Woerter);
    }
    Ok(e)
}

/// Vorschläge für die Wortauswahl beim Eingeben (Client-Härtung #69: Auswahl statt Tastatur).
pub fn vorschlaege(anfang: &str, max: usize) -> Vec<String> {
    let a = anfang.trim().to_lowercase();
    if a.is_empty() {
        return Vec::new();
    }
    liste()
        .iter()
        .filter(|w| w.starts_with(&a))
        .take(max)
        .map(|w| (*w).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bip39_testvektor_trezor() {
        // Trezor-Vektor: 32 Byte 0x00 -> "abandon" x23 + "art"
        let e = Geheim::neu(32);
        let w = kodieren(&e);
        assert_eq!(w[0], "abandon");
        assert_eq!(w[23], "art");
        // 32 Byte 0x7f -> "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title"
        let e = Geheim::aus(&[0x7f; 32]);
        let w = kodieren(&e);
        assert_eq!(
            w.join(" "),
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title"
        );
        assert_eq!(dekodieren(&w).unwrap().bytes(), &[0x7f; 32]);
    }

    #[test]
    fn pruefsumme_und_kurzform() {
        let e = Geheim::aus(&[0x80; 32]);
        let mut w = kodieren(&e);
        let kurz: Vec<String> = w.iter().map(|x| x.chars().take(4).collect()).collect();
        assert_eq!(dekodieren(&kurz).unwrap().bytes(), e.bytes());
        // Ein Wort tauschen: Prüfsumme muss fast immer anschlagen; hier gezielt das letzte.
        w[23] = if w[23] == "zoo" {
            "zone".into()
        } else {
            "zoo".into()
        };
        assert_eq!(dekodieren(&w).unwrap_err(), Fehler::Woerter);
        assert_eq!(dekodieren(&w[..23]).unwrap_err(), Fehler::Woerter);
    }
}
