// SPDX-License-Identifier: MPL-2.0
//! Testvektoren der Formatversion 1 (`docs/format.md` Abschnitt 9). Die Dateien unter
//! `core/testvectors/v1/` sind einmal erzeugt und werden nie neu geschrieben: jede spätere
//! Version des Kerns muss sie lesen und genau diesen Inhalt ergeben.
//!
//! Erzeugen (nur beim Anlegen eines neuen Formats):
//! `DAK_TESTVEKTOREN_SCHREIBEN=1 cargo test -p dak-core --test testvektoren -- --ignored`

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use dak_core::kern::{Kern, Uhr};
use dak_core::modell::{Antwort, Antwortwert, Ausloeser, Objekt, Zustand};
use dak_core::sodium::TestZufall;
use dak_core::speicher;
use dak_core::vorlagen;
use dak_core::zeit::{Zeitpunkt, tag_aus_datum};

#[derive(Clone)]
struct TestUhr(Arc<AtomicI64>);
impl Uhr for TestUhr {
    fn jetzt(&self) -> Zeitpunkt {
        Zeitpunkt {
            utc_ms: self.0.load(Ordering::SeqCst),
            versatz_min: 120,
            zone: "Europe/Berlin".into(),
        }
    }
}

fn ordner() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testvectors/v1")
}

/// Erwarteter Inhalt: (Lebenstag, Uhrzeit in Minuten, Formular, Stimmung).
fn inhalt(k: &Kern) -> Vec<String> {
    let s = k.sitzung().unwrap();
    let st = vorlagen::messdef_id("stimmung");
    let mut v: Vec<String> = s
        .eintraege
        .iter()
        .map(|e| match &e.objekt {
            Objekt::Status(x) => format!(
                "status {} {} {} {:?}",
                x.lebenstag,
                x.zeitpunkt.uhrzeit_min(),
                s.formular(&x.formular).unwrap().schluessel,
                x.antworten
                    .iter()
                    .find(|a| a.messdef == st)
                    .and_then(|a| a.wert.as_ref()?.zahl)
            ),
            Objekt::Ereignis(x) => format!(
                "ereignis {} {} {}",
                x.lebenstag,
                x.zeitpunkt.uhrzeit_min(),
                x.art
            ),
        })
        .collect();
    v.sort();
    v
}

#[test]
#[ignore = "nur zum einmaligen Erzeugen"]
fn testvektoren_erzeugen() {
    if std::env::var("DAK_TESTVEKTOREN_SCHREIBEN").is_err() {
        return;
    }
    let tag = tag_aus_datum(2026, 10, 1);
    let uhr = TestUhr(Arc::new(AtomicI64::new(
        (tag * 1440 + 8 * 60 - 120) * 60_000,
    )));
    let mut k = Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(42)),
        Box::new(uhr.clone()),
    );
    let (woerter, probe, _s) = k.einrichtung_beginnen(240).unwrap();
    let p: Vec<String> = probe.iter().map(|i| woerter[*i].clone()).collect();
    k.einrichtung_abschliessen(&p, None, None).unwrap();
    let f = vorlagen::formular_id("status_jetzt");
    for (i, w) in [-3i64, -1, 2, 0, 4].iter().enumerate() {
        let a = vec![Antwort {
            messdef: vorlagen::messdef_id("stimmung"),
            zustand: Zustand::Beantwortet,
            wert: Some(Antwortwert {
                zahl: Some(*w),
                ..Default::default()
            }),
            aktiv_gesetzt: true,
        }];
        k.status_speichern(f, None, a, i == 2, Ausloeser::Selbst, None, None, None)
            .unwrap();
        uhr.0.fetch_add(170 * 60_000, Ordering::SeqCst);
    }
    k.ereignis_erfassen(
        vorlagen::formular_id("aufgewacht"),
        None,
        vec![],
        None,
        None,
    )
    .unwrap();
    let (_, b) = k.sicherung_erstellen().unwrap();
    std::fs::create_dir_all(ordner()).unwrap();
    std::fs::write(ordner().join("sicherung.daks"), &b).unwrap();
    std::fs::write(ordner().join("woerter.txt"), woerter.join(" ") + "\n").unwrap();
    std::fs::write(ordner().join("erwartet.txt"), inhalt(&k).join("\n") + "\n").unwrap();
}

#[test]
fn testvektoren_v1_lesen() {
    let b = std::fs::read(ordner().join("sicherung.daks")).expect("Testvektor fehlt");
    let woerter: Vec<String> = std::fs::read_to_string(ordner().join("woerter.txt"))
        .unwrap()
        .split_whitespace()
        .map(String::from)
        .collect();
    let erwartet: Vec<String> = std::fs::read_to_string(ordner().join("erwartet.txt"))
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    let uhr = TestUhr(Arc::new(AtomicI64::new(1_800_000_000_000)));
    let mut k = Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(7)),
        Box::new(uhr),
    );
    k.wiederherstellung_beginnen(&woerter).unwrap();
    let d = vec![("sicherung.daks".to_string(), b.clone())];
    let kand = k.kandidaten(&d, &[]).unwrap();
    assert!(kand[0].gueltig && kand[0].empfohlen, "{:?}", kand[0].grund);
    k.wiederherstellen(&d, &[], 0).unwrap();
    k.vorbereitetes_abschliessen(None, None).unwrap();
    assert_eq!(inhalt(&k), erwartet);
    // Negativprobe: ein verändertes Byte im Strom macht den Vektor ungültig.
    let mut kaputt = b;
    let i = kaputt.len() - 5;
    kaputt[i] ^= 0x40;
    let mut k2 = Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(8)),
        Box::new(TestUhr(Arc::new(AtomicI64::new(1_800_000_000_000)))),
    );
    k2.wiederherstellung_beginnen(&woerter).unwrap();
    assert!(!k2.kandidaten(&[("x".into(), kaputt)], &[]).unwrap()[0].gueltig);
}

#[test]
fn feste_vorlagen_ids() {
    // Diese IDs stehen in jedem Datensatz; ändern sie sich, passen alte Daten nicht mehr.
    // Gegenprobe mit einer unabhängigen Implementierung (Python hashlib.blake2b, digest_size=16).
    assert_eq!(
        dak_core::modell::hex(&vorlagen::messdef_id("stimmung")),
        "7fde3f4cbb5a0bfc76d1efd5fa680389"
    );
}
