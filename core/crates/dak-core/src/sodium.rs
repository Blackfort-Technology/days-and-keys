// SPDX-License-Identifier: MPL-2.0
//! Dünne, sichere Hülle um libsodium. Nur dieses Modul ruft libsodium auf (Architektur AE-05).
//!
//! Regeln: Längen werden vor jedem Aufruf geprüft, jeder Rückgabewert wird ausgewertet,
//! geheime Schlüssel liegen in [`Geheim`] (Speicher aus `sodium_malloc`, beim Freigeben
//! überschrieben), entschlüsselte Klartexte kommen als [`Zeroizing`] zurück.

use std::ptr::NonNull;
use std::sync::Once;

use libsodium_sys as ffi;
use zeroize::Zeroizing;

use crate::fehler::{Ergebnis, Fehler};

pub const SCHLUESSEL: usize = 32;
pub const NONCE: usize = 24;
pub const ABYTES: usize = 16;
pub const SIGNATUR: usize = 64;
pub const SEAL_BYTES: usize = 48;
pub const STREAM_HEADER: usize = 24;
pub const STREAM_ABYTES: usize = 17;

static INIT: Once = Once::new();

/// Initialisiert libsodium genau einmal. Scheitert die Initialisierung, ist der Kern unbenutzbar.
pub fn init() {
    INIT.call_once(|| {
        // SAFETY: sodium_init ist threadsicher und darf mehrfach aufgerufen werden.
        let rc = unsafe { ffi::sodium_init() };
        assert!(rc >= 0, "libsodium konnte nicht initialisiert werden");
    });
}

/// Geheimer Speicher fester Länge aus `sodium_malloc` (Schutzseiten, beim Freigeben genullt).
pub struct Geheim {
    ptr: NonNull<u8>,
    len: usize,
}

// SAFETY: Der Speicher gehört exklusiv dieser Struktur; Zugriffe laufen über &self/&mut self.
unsafe impl Send for Geheim {}
unsafe impl Sync for Geheim {}

impl Geheim {
    /// Neuer, mit Nullen gefüllter geheimer Speicher.
    pub fn neu(len: usize) -> Self {
        init();
        assert!(len > 0);
        // SAFETY: sodium_malloc liefert len Bytes oder NULL.
        let p = unsafe { ffi::sodium_malloc(len) } as *mut u8;
        let ptr = NonNull::new(p).expect("sodium_malloc: kein Speicher");
        // SAFETY: ptr zeigt auf len gültige Bytes.
        unsafe { ffi::sodium_memzero(ptr.as_ptr().cast(), len) };
        Geheim { ptr, len }
    }

    /// Kopiert `bytes` in geheimen Speicher. Der Aufrufer überschreibt seine Kopie selbst.
    pub fn aus(bytes: &[u8]) -> Self {
        let mut g = Geheim::neu(bytes.len());
        g.bytes_mut().copy_from_slice(bytes);
        g
    }

    pub fn zufall(len: usize, z: &mut dyn Zufall) -> Self {
        let mut g = Geheim::neu(len);
        z.fuellen(g.bytes_mut());
        g
    }

    pub fn bytes(&self) -> &[u8] {
        // SAFETY: ptr ist gültig für len Bytes, solange self lebt.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY: wie oben, exklusiver Zugriff über &mut self.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Vergleich in konstanter Zeit.
    pub fn gleich(&self, andere: &[u8]) -> bool {
        andere.len() == self.len && gleich_konstant(self.bytes(), andere)
    }
}

impl Clone for Geheim {
    fn clone(&self) -> Self {
        Geheim::aus(self.bytes())
    }
}

impl Drop for Geheim {
    fn drop(&mut self) {
        // SAFETY: ptr stammt aus sodium_malloc; sodium_free überschreibt vor dem Freigeben.
        unsafe { ffi::sodium_free(self.ptr.as_ptr().cast()) };
    }
}

impl std::fmt::Debug for Geheim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Geheim({} Byte)", self.len)
    }
}

/// Vergleich in konstanter Zeit über gleich lange Puffer.
pub fn gleich_konstant(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    // SAFETY: beide Zeiger gültig für a.len() Bytes.
    unsafe { ffi::sodium_memcmp(a.as_ptr().cast(), b.as_ptr().cast(), a.len()) == 0 }
}

/// Quelle für Zufall. Im Betrieb [`SystemZufall`]; Tests und Testvektoren setzen einen
/// deterministischen Zufall ein, damit Ausgaben bytegenau prüfbar sind.
pub trait Zufall: Send {
    fn fuellen(&mut self, buf: &mut [u8]);

    fn bytes<const N: usize>(&mut self) -> [u8; N]
    where
        Self: Sized,
    {
        let mut b = [0u8; N];
        self.fuellen(&mut b);
        b
    }
}

pub fn zufall_bytes<const N: usize>(z: &mut dyn Zufall) -> [u8; N] {
    let mut b = [0u8; N];
    z.fuellen(&mut b);
    b
}

/// Zufall des Betriebssystems über `randombytes_buf`.
pub struct SystemZufall;

impl Zufall for SystemZufall {
    fn fuellen(&mut self, buf: &mut [u8]) {
        init();
        // SAFETY: buf ist gültig für buf.len() Bytes.
        unsafe { ffi::randombytes_buf(buf.as_mut_ptr().cast(), buf.len()) };
    }
}

/// Deterministischer Zufall aus Startwert und Zähler (`randombytes_buf_deterministic`).
/// Nur für Tests und Testvektoren.
#[cfg(any(test, feature = "testzufall"))]
pub struct TestZufall {
    startwert: [u8; 32],
    zaehler: u64,
}

#[cfg(any(test, feature = "testzufall"))]
impl TestZufall {
    pub fn neu(startwert: u8) -> Self {
        TestZufall {
            startwert: [startwert; 32],
            zaehler: 0,
        }
    }
}

#[cfg(any(test, feature = "testzufall"))]
impl Zufall for TestZufall {
    fn fuellen(&mut self, buf: &mut [u8]) {
        init();
        self.zaehler += 1;
        let mut seed = self.startwert;
        for (i, b) in self.zaehler.to_le_bytes().iter().enumerate() {
            seed[i] ^= b;
        }
        // SAFETY: buf gültig, seed hat randombytes_SEEDBYTES (32) Byte.
        unsafe {
            ffi::randombytes_buf_deterministic(buf.as_mut_ptr().cast(), buf.len(), seed.as_ptr())
        };
    }
}

// ---------------------------------------------------------------------------------------------
// AEAD: XChaCha20-Poly1305 IETF
// ---------------------------------------------------------------------------------------------

pub fn aead_verschluesseln(
    schluessel: &Geheim,
    nonce: &[u8; NONCE],
    klartext: &[u8],
    ad: &[u8],
) -> Vec<u8> {
    init();
    assert_eq!(schluessel.len(), SCHLUESSEL);
    let mut out = vec![0u8; klartext.len() + ABYTES];
    let mut out_len: u64 = 0;
    // SAFETY: Puffer haben die geforderten Längen.
    let rc = unsafe {
        ffi::crypto_aead_xchacha20poly1305_ietf_encrypt(
            out.as_mut_ptr(),
            &mut out_len,
            klartext.as_ptr(),
            klartext.len() as u64,
            ad.as_ptr(),
            ad.len() as u64,
            std::ptr::null(),
            nonce.as_ptr(),
            schluessel.bytes().as_ptr(),
        )
    };
    assert_eq!(rc, 0);
    out.truncate(out_len as usize);
    out
}

pub fn aead_entschluesseln(
    schluessel: &Geheim,
    nonce: &[u8; NONCE],
    chiffrat: &[u8],
    ad: &[u8],
) -> Ergebnis<Zeroizing<Vec<u8>>> {
    init();
    if schluessel.len() != SCHLUESSEL || chiffrat.len() < ABYTES {
        return Err(Fehler::Integritaet("AEAD-Länge"));
    }
    let mut out = Zeroizing::new(vec![0u8; chiffrat.len() - ABYTES]);
    let mut out_len: u64 = 0;
    // SAFETY: Puffer haben die geforderten Längen.
    let rc = unsafe {
        ffi::crypto_aead_xchacha20poly1305_ietf_decrypt(
            out.as_mut_ptr(),
            &mut out_len,
            std::ptr::null_mut(),
            chiffrat.as_ptr(),
            chiffrat.len() as u64,
            ad.as_ptr(),
            ad.len() as u64,
            nonce.as_ptr(),
            schluessel.bytes().as_ptr(),
        )
    };
    if rc != 0 {
        return Err(Fehler::Integritaet("AEAD-Prüfung"));
    }
    out.truncate(out_len as usize);
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// KDF und Hash (BLAKE2b)
// ---------------------------------------------------------------------------------------------

/// Teilschlüssel per `crypto_kdf_derive_from_key`. Kontext genau acht Byte.
pub fn kdf(wurzel: &Geheim, kontext: &[u8; 8], id: u64) -> Geheim {
    init();
    assert_eq!(wurzel.len(), 32);
    let mut out = Geheim::neu(32);
    // SAFETY: out 32 Byte, Kontext 8 Byte, Wurzel 32 Byte.
    let rc = unsafe {
        ffi::crypto_kdf_derive_from_key(
            out.bytes_mut().as_mut_ptr(),
            32,
            id,
            kontext.as_ptr().cast(),
            wurzel.bytes().as_ptr(),
        )
    };
    assert_eq!(rc, 0);
    out
}

fn blake2b(schluessel: Option<&Geheim>, teile: &[&[u8]], laenge: usize) -> Vec<u8> {
    init();
    let mut state = std::mem::MaybeUninit::<ffi::crypto_generichash_state>::uninit();
    let (kp, kl) = match schluessel {
        Some(k) => (k.bytes().as_ptr(), k.len()),
        None => (std::ptr::null(), 0),
    };
    let mut out = vec![0u8; laenge];
    // SAFETY: state wird von init vollständig geschrieben, Längen innerhalb der Grenzen.
    unsafe {
        assert_eq!(
            ffi::crypto_generichash_init(state.as_mut_ptr(), kp, kl, laenge),
            0
        );
        for t in teile {
            assert_eq!(
                ffi::crypto_generichash_update(state.as_mut_ptr(), t.as_ptr(), t.len() as u64),
                0
            );
        }
        assert_eq!(
            ffi::crypto_generichash_final(state.as_mut_ptr(), out.as_mut_ptr(), laenge),
            0
        );
        ffi::sodium_memzero(
            state.as_mut_ptr().cast(),
            std::mem::size_of::<ffi::crypto_generichash_state>(),
        );
    }
    out
}

/// BLAKE2b-256 über die Verkettung der Teile.
pub fn hash32(teile: &[&[u8]]) -> [u8; 32] {
    blake2b(None, teile, 32).try_into().expect("32 Byte")
}

/// BLAKE2b-128, z. B. für abgeleitete feste IDs.
pub fn hash16(teile: &[&[u8]]) -> [u8; 16] {
    blake2b(None, teile, 16).try_into().expect("16 Byte")
}

/// Schlüsselgebundener BLAKE2b-256 als MAC.
pub fn mac32(schluessel: &Geheim, teile: &[&[u8]]) -> [u8; 32] {
    blake2b(Some(schluessel), teile, 32)
        .try_into()
        .expect("32 Byte")
}

/// Längenpräfix für eindeutige Verkettung von Feldern in AD und Hashes.
pub fn lp(teil: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(teil.len() + 4);
    v.extend_from_slice(&(teil.len() as u32).to_be_bytes());
    v.extend_from_slice(teil);
    v
}

// ---------------------------------------------------------------------------------------------
// Signaturen (Ed25519) und Box (X25519)
// ---------------------------------------------------------------------------------------------

pub struct SignPaar {
    pub oeffentlich: [u8; 32],
    geheim: Geheim,
}

impl SignPaar {
    pub fn aus_saat(saat: &Geheim) -> Self {
        init();
        assert_eq!(saat.len(), 32);
        let mut pk = [0u8; 32];
        let mut sk = Geheim::neu(64);
        // SAFETY: pk 32, sk 64, saat 32 Byte.
        let rc = unsafe {
            ffi::crypto_sign_seed_keypair(
                pk.as_mut_ptr(),
                sk.bytes_mut().as_mut_ptr(),
                saat.bytes().as_ptr(),
            )
        };
        assert_eq!(rc, 0);
        SignPaar {
            oeffentlich: pk,
            geheim: sk,
        }
    }

    pub fn signieren(&self, nachricht: &[u8]) -> [u8; SIGNATUR] {
        let mut sig = [0u8; SIGNATUR];
        // SAFETY: Längen passend.
        let rc = unsafe {
            ffi::crypto_sign_detached(
                sig.as_mut_ptr(),
                std::ptr::null_mut(),
                nachricht.as_ptr(),
                nachricht.len() as u64,
                self.geheim.bytes().as_ptr(),
            )
        };
        assert_eq!(rc, 0);
        sig
    }
}

pub fn signatur_pruefen(oeffentlich: &[u8], nachricht: &[u8], sig: &[u8]) -> bool {
    init();
    if oeffentlich.len() != 32 || sig.len() != SIGNATUR {
        return false;
    }
    // SAFETY: Längen geprüft.
    unsafe {
        ffi::crypto_sign_verify_detached(
            sig.as_ptr(),
            nachricht.as_ptr(),
            nachricht.len() as u64,
            oeffentlich.as_ptr(),
        ) == 0
    }
}

pub struct BoxPaar {
    pub oeffentlich: [u8; 32],
    geheim: Geheim,
}

impl BoxPaar {
    pub fn aus_saat(saat: &Geheim) -> Self {
        init();
        assert_eq!(saat.len(), 32);
        let mut pk = [0u8; 32];
        let mut sk = Geheim::neu(32);
        // SAFETY: Längen passend.
        let rc = unsafe {
            ffi::crypto_box_seed_keypair(
                pk.as_mut_ptr(),
                sk.bytes_mut().as_mut_ptr(),
                saat.bytes().as_ptr(),
            )
        };
        assert_eq!(rc, 0);
        BoxPaar {
            oeffentlich: pk,
            geheim: sk,
        }
    }

    /// Öffnet ein an diesen Schlüssel versiegeltes Chiffrat (`crypto_box_seal_open`).
    pub fn oeffnen(&self, chiffrat: &[u8]) -> Ergebnis<Zeroizing<Vec<u8>>> {
        if chiffrat.len() < SEAL_BYTES {
            return Err(Fehler::Integritaet("Siegel-Länge"));
        }
        let mut out = Zeroizing::new(vec![0u8; chiffrat.len() - SEAL_BYTES]);
        // SAFETY: Längen passend.
        let rc = unsafe {
            ffi::crypto_box_seal_open(
                out.as_mut_ptr(),
                chiffrat.as_ptr(),
                chiffrat.len() as u64,
                self.oeffentlich.as_ptr(),
                self.geheim.bytes().as_ptr(),
            )
        };
        if rc != 0 {
            return Err(Fehler::Integritaet("Siegel-Prüfung"));
        }
        Ok(out)
    }
}

/// Versiegelt `klartext` an einen öffentlichen X25519-Schlüssel (`crypto_box_seal`).
pub fn versiegeln(klartext: &[u8], oeffentlich: &[u8; 32]) -> Vec<u8> {
    init();
    let mut out = vec![0u8; klartext.len() + SEAL_BYTES];
    // SAFETY: Längen passend.
    let rc = unsafe {
        ffi::crypto_box_seal(
            out.as_mut_ptr(),
            klartext.as_ptr(),
            klartext.len() as u64,
            oeffentlich.as_ptr(),
        )
    };
    assert_eq!(rc, 0);
    out
}

// ---------------------------------------------------------------------------------------------
// Auffüllen
// ---------------------------------------------------------------------------------------------

pub fn auffuellen(daten: &[u8], block: usize) -> Zeroizing<Vec<u8>> {
    init();
    let mut buf = Zeroizing::new(Vec::with_capacity(daten.len() + block));
    buf.extend_from_slice(daten);
    buf.resize(daten.len() + block, 0);
    let mut neu_len: usize = 0;
    // SAFETY: buf hat Kapazität daten.len() + block.
    let rc = unsafe {
        ffi::sodium_pad(
            &mut neu_len,
            buf.as_mut_ptr(),
            daten.len(),
            block,
            buf.len(),
        )
    };
    assert_eq!(rc, 0);
    buf.truncate(neu_len);
    buf
}

pub fn auffuellung_entfernen(
    mut daten: Zeroizing<Vec<u8>>,
    block: usize,
) -> Ergebnis<Zeroizing<Vec<u8>>> {
    let mut len: usize = 0;
    // SAFETY: daten gültig.
    let rc = unsafe { ffi::sodium_unpad(&mut len, daten.as_ptr(), daten.len(), block) };
    if rc != 0 {
        return Err(Fehler::Format("Auffüllung"));
    }
    daten.truncate(len);
    Ok(daten)
}

// ---------------------------------------------------------------------------------------------
// secretstream (Sicherungsdatei, AE-18)
// ---------------------------------------------------------------------------------------------

pub const TAG_NACHRICHT: u8 = 0;
pub const TAG_ENDE: u8 = 3;

pub struct StromSchreiber {
    zustand: Box<ffi::crypto_secretstream_xchacha20poly1305_state>,
}

impl StromSchreiber {
    pub fn neu(schluessel: &Geheim) -> (Self, [u8; STREAM_HEADER]) {
        init();
        assert_eq!(schluessel.len(), 32);
        let mut header = [0u8; STREAM_HEADER];
        // SAFETY: Zustand wird von init_push vollständig geschrieben.
        let mut zustand: Box<ffi::crypto_secretstream_xchacha20poly1305_state> =
            Box::new(unsafe { std::mem::zeroed() });
        let rc = unsafe {
            ffi::crypto_secretstream_xchacha20poly1305_init_push(
                &mut *zustand,
                header.as_mut_ptr(),
                schluessel.bytes().as_ptr(),
            )
        };
        assert_eq!(rc, 0);
        (StromSchreiber { zustand }, header)
    }

    pub fn schreiben(&mut self, teil: &[u8], ad: &[u8], ende: bool) -> Vec<u8> {
        let mut out = vec![0u8; teil.len() + STREAM_ABYTES];
        let mut out_len: u64 = 0;
        let tag = if ende { TAG_ENDE } else { TAG_NACHRICHT };
        // SAFETY: Längen passend.
        let rc = unsafe {
            ffi::crypto_secretstream_xchacha20poly1305_push(
                &mut *self.zustand,
                out.as_mut_ptr(),
                &mut out_len,
                teil.as_ptr(),
                teil.len() as u64,
                ad.as_ptr(),
                ad.len() as u64,
                tag,
            )
        };
        assert_eq!(rc, 0);
        out.truncate(out_len as usize);
        out
    }
}

impl Drop for StromSchreiber {
    fn drop(&mut self) {
        // SAFETY: Zustand gehört uns.
        unsafe {
            ffi::sodium_memzero(
                (&mut *self.zustand as *mut ffi::crypto_secretstream_xchacha20poly1305_state)
                    .cast(),
                std::mem::size_of::<ffi::crypto_secretstream_xchacha20poly1305_state>(),
            )
        };
    }
}

pub struct StromLeser {
    zustand: Box<ffi::crypto_secretstream_xchacha20poly1305_state>,
}

impl StromLeser {
    pub fn neu(schluessel: &Geheim, header: &[u8; STREAM_HEADER]) -> Ergebnis<Self> {
        init();
        assert_eq!(schluessel.len(), 32);
        // SAFETY: Zustand wird von init_pull geschrieben.
        let mut zustand: Box<ffi::crypto_secretstream_xchacha20poly1305_state> =
            Box::new(unsafe { std::mem::zeroed() });
        let rc = unsafe {
            ffi::crypto_secretstream_xchacha20poly1305_init_pull(
                &mut *zustand,
                header.as_ptr(),
                schluessel.bytes().as_ptr(),
            )
        };
        if rc != 0 {
            return Err(Fehler::Integritaet("Strom-Kopf"));
        }
        Ok(StromLeser { zustand })
    }

    /// Liefert Klartext und ob dies der letzte Teil war.
    pub fn lesen(&mut self, chiffrat: &[u8], ad: &[u8]) -> Ergebnis<(Zeroizing<Vec<u8>>, bool)> {
        if chiffrat.len() < STREAM_ABYTES {
            return Err(Fehler::Integritaet("Strom-Länge"));
        }
        let mut out = Zeroizing::new(vec![0u8; chiffrat.len() - STREAM_ABYTES]);
        let mut out_len: u64 = 0;
        let mut tag: u8 = 0;
        // SAFETY: Längen passend.
        let rc = unsafe {
            ffi::crypto_secretstream_xchacha20poly1305_pull(
                &mut *self.zustand,
                out.as_mut_ptr(),
                &mut out_len,
                &mut tag,
                chiffrat.as_ptr(),
                chiffrat.len() as u64,
                ad.as_ptr(),
                ad.len() as u64,
            )
        };
        if rc != 0 {
            return Err(Fehler::Integritaet("Strom-Prüfung"));
        }
        out.truncate(out_len as usize);
        match tag {
            TAG_NACHRICHT => Ok((out, false)),
            TAG_ENDE => Ok((out, true)),
            _ => Err(Fehler::Integritaet("Strom-Kennzeichen")),
        }
    }
}

impl Drop for StromLeser {
    fn drop(&mut self) {
        // SAFETY: Zustand gehört uns.
        unsafe {
            ffi::sodium_memzero(
                (&mut *self.zustand as *mut ffi::crypto_secretstream_xchacha20poly1305_state)
                    .cast(),
                std::mem::size_of::<ffi::crypto_secretstream_xchacha20poly1305_state>(),
            )
        };
    }
}

/// SHA-256 (für Prüfsummen der Wortlisten, nicht für Schlüssel).
pub fn sha256(daten: &[u8]) -> [u8; 32] {
    init();
    let mut out = [0u8; 32];
    // SAFETY: out 32 Byte.
    let rc =
        unsafe { ffi::crypto_hash_sha256(out.as_mut_ptr(), daten.as_ptr(), daten.len() as u64) };
    assert_eq!(rc, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aead_rundweg_und_manipulation() {
        let mut z = TestZufall::neu(1);
        let k = Geheim::zufall(32, &mut z);
        let n = zufall_bytes::<24>(&mut z);
        let c = aead_verschluesseln(&k, &n, b"hallo", b"ad");
        assert_eq!(&*aead_entschluesseln(&k, &n, &c, b"ad").unwrap(), b"hallo");
        assert!(aead_entschluesseln(&k, &n, &c, b"andere ad").is_err());
        let mut c2 = c.clone();
        c2[0] ^= 1;
        assert!(aead_entschluesseln(&k, &n, &c2, b"ad").is_err());
        let k2 = Geheim::zufall(32, &mut z);
        assert!(aead_entschluesseln(&k2, &n, &c, b"ad").is_err());
    }

    #[test]
    fn siegel_und_signatur() {
        let mut z = TestZufall::neu(2);
        let paar = BoxPaar::aus_saat(&Geheim::zufall(32, &mut z));
        let c = versiegeln(b"geheim", &paar.oeffentlich);
        assert_eq!(&*paar.oeffnen(&c).unwrap(), b"geheim");
        let fremd = BoxPaar::aus_saat(&Geheim::zufall(32, &mut z));
        assert!(fremd.oeffnen(&c).is_err());

        let s = SignPaar::aus_saat(&Geheim::zufall(32, &mut z));
        let sig = s.signieren(b"nachricht");
        assert!(signatur_pruefen(&s.oeffentlich, b"nachricht", &sig));
        assert!(!signatur_pruefen(&s.oeffentlich, b"Nachricht", &sig));
    }

    #[test]
    fn strom_erkennt_abschneiden() {
        let mut z = TestZufall::neu(3);
        let k = Geheim::zufall(32, &mut z);
        let (mut w, kopf) = StromSchreiber::neu(&k);
        let a = w.schreiben(b"eins", b"x", false);
        let b = w.schreiben(b"zwei", b"", true);
        let mut r = StromLeser::neu(&k, &kopf).unwrap();
        let (t, ende) = r.lesen(&a, b"x").unwrap();
        assert_eq!((&t[..], ende), (&b"eins"[..], false));
        let (t, ende) = r.lesen(&b, b"").unwrap();
        assert_eq!((&t[..], ende), (&b"zwei"[..], true));
        // Reihenfolge vertauscht: muss scheitern.
        let mut r = StromLeser::neu(&k, &kopf).unwrap();
        assert!(r.lesen(&b, b"").is_err());
    }

    #[test]
    fn auffuellen_rundweg() {
        let p = auffuellen(b"abc", 256);
        assert_eq!(p.len(), 256);
        let u = auffuellung_entfernen(p, 256).unwrap();
        assert_eq!(&u[..], b"abc");
    }

    #[test]
    fn libsodium_bekannter_vektor_sha256() {
        // FIPS 180-2, "abc"
        let h = sha256(b"abc");
        assert_eq!(h[..4], [0xba, 0x78, 0x16, 0xbf]);
    }
}
