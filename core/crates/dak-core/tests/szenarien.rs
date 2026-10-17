// SPDX-License-Identifier: MPL-2.0
//! Szenarien aus #100: Einrichten, Status, Sicherung, Wiederherstellung, Rotation und die
//! Negativfälle. Alle Daten sind erfunden (Invariante 5).

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use dak_core::fehler::Fehler;
use dak_core::kern::{Kern, Uhr};
use dak_core::kette::Rotationsgrund;
use dak_core::modell::{Antwort, Antwortwert, Ausloeser, Objekt, Praeparat, Zustand};
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

impl TestUhr {
    fn neu(tag: i64, minute: i64) -> Self {
        TestUhr(Arc::new(AtomicI64::new(
            (tag * 1440 + minute - 120) * 60_000,
        )))
    }
    fn weiter_min(&self, m: i64) {
        self.0.fetch_add(m * 60_000, Ordering::SeqCst);
    }
}

struct Geraet {
    kern: Kern,
    kk: Vec<u8>,
    kk_h: Vec<u8>,
    woerter: Vec<String>,
}

fn einrichten(seed: u8, uhr: &TestUhr) -> Geraet {
    let mut k = Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(seed)),
        Box::new(uhr.clone()),
    );
    let (woerter, probe, s) = k.einrichtung_beginnen(240).unwrap();
    let falsch: Vec<String> = probe.iter().map(|_| "zoo".to_string()).collect();
    assert_eq!(
        k.einrichtung_abschliessen(&falsch, None, None).unwrap_err(),
        Fehler::Woerter,
        "falsche Wortprobe muss scheitern"
    );
    let antworten: Vec<String> = probe.iter().map(|i| woerter[*i].clone()).collect();
    k.einrichtung_abschliessen(&antworten, None, None).unwrap();
    Geraet {
        kk: s.kk.clone(),
        kk_h: s.kk_h.clone(),
        kern: k,
        woerter,
    }
}

fn status(g: &mut Geraet, stimmung: i64) -> [u8; 16] {
    let f = vorlagen::formular_id("status_jetzt");
    g.kern
        .status_speichern(
            f,
            None,
            vec![Antwort {
                messdef: vorlagen::messdef_id("stimmung"),
                zustand: Zustand::Beantwortet,
                wert: Some(Antwortwert {
                    zahl: Some(stimmung),
                    ..Default::default()
                }),
                aktiv_gesetzt: true,
            }],
            false,
            Ausloeser::Selbst,
            None,
            None,
            None,
        )
        .unwrap()
}

fn stimmungen(k: &Kern) -> Vec<i64> {
    let s = k.sitzung().unwrap();
    s.eintraege
        .iter()
        .filter_map(|e| match &e.objekt {
            Objekt::Status(st) => st
                .antworten
                .iter()
                .find(|a| a.messdef == vorlagen::messdef_id("stimmung"))
                .and_then(|a| a.wert.as_ref()?.zahl),
            _ => None,
        })
        .collect()
}

fn neues_geraet(seed: u8, uhr: &TestUhr) -> Kern {
    Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(seed)),
        Box::new(uhr.clone()),
    )
}

fn sichern(g: &mut Geraet) -> (String, Vec<u8>) {
    let (name, b) = g.kern.sicherung_erstellen().unwrap();
    g.kern.sicherung_bestaetigen(&b).unwrap();
    (name, b)
}

#[test]
fn mehrere_status_am_tag_und_neu_entsperren() {
    let uhr = TestUhr::neu(tag_aus_datum(2026, 10, 14), 8 * 60);
    let mut g = einrichten(1, &uhr);
    for (i, w) in [-2, 0, 3, 1].iter().enumerate() {
        status(&mut g, *w);
        uhr.weiter_min(150 + i as i64);
    }
    assert_eq!(stimmungen(&g.kern), vec![-2, 0, 3, 1]);
    g.kern.sperren();
    assert_eq!(g.kern.sitzung().err(), Some(Fehler::Gesperrt));
    // Falscher Schlüssel öffnet nichts.
    assert!(g.kern.entsperren(vec![7; 32], g.kk_h.clone()).is_err());
    g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).unwrap();
    assert_eq!(stimmungen(&g.kern), vec![-2, 0, 3, 1]);
    // Alle Status liegen am selben Lebenstag.
    let tage: std::collections::BTreeSet<i64> = g
        .kern
        .sitzung()
        .unwrap()
        .eintraege
        .iter()
        .map(|e| e.objekt.lebenstag())
        .collect();
    assert_eq!(tage.len(), 1);
}

#[test]
fn wert_ausserhalb_der_skala_und_antwort_auf_fremde_frage() {
    let uhr = TestUhr::neu(20_000, 600);
    let mut g = einrichten(2, &uhr);
    let f = vorlagen::formular_id("status_jetzt");
    let zu_gross = Antwort {
        messdef: vorlagen::messdef_id("stimmung"),
        zustand: Zustand::Beantwortet,
        wert: Some(Antwortwert {
            zahl: Some(6),
            ..Default::default()
        }),
        aktiv_gesetzt: true,
    };
    assert!(
        g.kern
            .status_speichern(
                f,
                None,
                vec![zu_gross],
                false,
                Ausloeser::Selbst,
                None,
                None,
                None
            )
            .is_err()
    );
    let fremd = Antwort {
        messdef: vorlagen::messdef_id("woche_medikation"),
        zustand: Zustand::Beantwortet,
        wert: Some(Antwortwert {
            zahl: Some(5),
            ..Default::default()
        }),
        aktiv_gesetzt: true,
    };
    assert!(
        g.kern
            .status_speichern(
                f,
                None,
                vec![fremd],
                false,
                Ausloeser::Selbst,
                None,
                None,
                None
            )
            .is_err()
    );
    // Wirkungsfrage ohne bestätigte Einnahme: nicht gezeigt, also nicht beantwortbar.
    let wirkung = Antwort {
        messdef: vorlagen::messdef_id("wirkung_jetzt"),
        zustand: Zustand::Beantwortet,
        wert: Some(Antwortwert {
            zahl: Some(1),
            ..Default::default()
        }),
        aktiv_gesetzt: true,
    };
    assert!(
        g.kern
            .status_speichern(
                f,
                None,
                vec![wirkung.clone()],
                false,
                Ausloeser::Selbst,
                None,
                None,
                None
            )
            .is_err()
    );
    // Nach einer Einnahme eines Präparats mit spürbarer Wirkung geht es.
    let p = g
        .kern
        .praeparat_speichern(Praeparat {
            id: [0; 16],
            handelsname: "Präparat A".into(),
            wirkstoff: None,
            staerke: Some("Stärke 1".into()),
            form: None,
            plan: vec![],
            bedarf: false,
            wirkung_spuerbar: true,
            aktiv: true,
            erinnerung: false,
            status_nach_min: None,
        })
        .unwrap();
    g.kern
        .ereignis_erfassen(
            vorlagen::formular_id("einnahme"),
            None,
            vec![],
            Some(p),
            None,
        )
        .unwrap();
    uhr.weiter_min(1);
    g.kern
        .status_speichern(
            f,
            None,
            vec![wirkung],
            false,
            Ausloeser::Selbst,
            None,
            None,
            None,
        )
        .unwrap();
    // Einnahme ohne Präparat wird abgelehnt.
    assert!(
        g.kern
            .ereignis_erfassen(vorlagen::formular_id("einnahme"), None, vec![], None, None)
            .is_err()
    );
}

#[test]
fn sicherung_und_wiederherstellung_auf_zweitem_geraet() {
    let uhr = TestUhr::neu(20_100, 600);
    let mut a = einrichten(3, &uhr);
    status(&mut a, 2);
    uhr.weiter_min(10);
    let geloescht = status(&mut a, -4);
    uhr.weiter_min(10);
    status(&mut a, 1);
    a.kern.eintrag_loeschen(geloescht).unwrap();
    let datei = sichern(&mut a);

    // Falsche Wörter
    let mut b = neues_geraet(4, &uhr);
    let mut falsch = a.woerter.clone();
    falsch.swap(0, 1);
    assert!(
        b.wiederherstellung_beginnen(&falsch).is_err()
            || b.kandidaten(std::slice::from_ref(&datei), &[])
                .unwrap()
                .iter()
                .all(|k| !k.gueltig)
    );

    let mut b = neues_geraet(5, &uhr);
    b.wiederherstellung_beginnen(&a.woerter).unwrap();
    let k = b.kandidaten(std::slice::from_ref(&datei), &[]).unwrap();
    assert!(k[0].gueltig && k[0].empfohlen);
    let (neu, _marker_name, marker) = b
        .wiederherstellen(std::slice::from_ref(&datei), &[], 0)
        .unwrap();
    b.vorbereitetes_abschliessen(None, None).unwrap();
    assert_eq!(
        stimmungen(&b),
        vec![2, 1],
        "gelöschter Eintrag bleibt gelöscht"
    );
    assert_eq!(neu.generation, 2);
    // Das neue Gerät liest mit den neuen Schlüsseln, nicht mit den alten.
    b.sperren();
    assert!(
        b.entsperren(a.kk.clone(), a.kk_h.clone()).is_err(),
        "alter KK öffnet den neuen Bestand nicht"
    );
    b.entsperren(neu.kk.clone(), neu.kk_h.clone()).unwrap();
    // Die Sicherungsnummer läuft weiter.
    let (name_b, datei_b) = b.sicherung_erstellen().unwrap();
    b.sicherung_bestaetigen(&datei_b).unwrap();
    assert!(name_b.ends_with("000002.daks"), "{name_b}");

    // Das alte Gerät sieht den Marker und sichert nicht mehr.
    assert!(
        !a.kern
            .marker_pruefen(std::slice::from_ref(&marker))
            .unwrap()
    );
    assert!(a.kern.sicherung_erstellen().is_err());

    // Neue Sicherung des neuen Geräts ist mit alten Schlüsseln nicht lesbar, aber mit den Wörtern.
    assert!(
        dak_core::sicherung::pruefen_mit_kk(
            &datei_b,
            &dak_core::sodium::Geheim::aus(&a.kk),
            &b.sitzung().unwrap().konto_id
        )
        .is_err()
    );
    let mut c = neues_geraet(6, &uhr);
    c.wiederherstellung_beginnen(&a.woerter).unwrap();
    let ks = c
        .kandidaten(
            &[datei.clone(), (name_b.clone(), datei_b.clone())],
            &[marker],
        )
        .unwrap();
    assert!(
        !ks[0].empfohlen && ks[1].empfohlen,
        "die längere Kette gewinnt vor der Nummer"
    );
}

#[test]
fn plan_fuer_tiefs_kommt_mit_der_sicherung() {
    // #131: Der Plan lag nur unverschlüsselt in den Metadaten und fehlte nach der Wiederherstellung.
    let uhr = TestUhr::neu(20_200, 600);
    let mut a = einrichten(21, &uhr);
    let plan = "Erfundener Plan: spazieren gehen, Musik anmachen".to_string();
    a.kern.krisenplan_setzen(Some(plan.clone())).unwrap();
    // Einstellungen speichern lässt den Plan stehen, auch wenn sie ihn nicht mitbringen.
    let mut e = a.kern.sitzung().unwrap().bund.einstellungen.clone();
    e.krisenplan = None;
    a.kern.einstellungen_setzen(e).unwrap();
    assert_eq!(a.kern.krisenplan().unwrap().as_deref(), Some(plan.as_str()));
    status(&mut a, 1);
    let datei = sichern(&mut a);

    let mut b = neues_geraet(22, &uhr);
    b.wiederherstellung_beginnen(&a.woerter).unwrap();
    b.wiederherstellen(std::slice::from_ref(&datei), &[], 0)
        .unwrap();
    b.vorbereitetes_abschliessen(None, None).unwrap();
    assert_eq!(
        b.krisenplan().unwrap().as_deref(),
        Some(plan.as_str()),
        "Plan ist nach der Wiederherstellung da"
    );
    b.sperren();
    assert_eq!(
        b.krisenplan().unwrap().as_deref(),
        Some(plan.as_str()),
        "und gesperrt für die Hilfe lesbar"
    );

    // Löschen entfernt beide Kopien.
    a.kern.krisenplan_setzen(None).unwrap();
    assert_eq!(a.kern.krisenplan().unwrap(), None);
    assert_eq!(
        a.kern.sitzung().unwrap().bund.einstellungen.krisenplan,
        None
    );
    a.kern.sperren();
    a.kern.entsperren(a.kk.clone(), a.kk_h.clone()).unwrap();
    assert_eq!(
        a.kern.krisenplan().unwrap(),
        None,
        "kommt beim Entsperren nicht zurück"
    );
}

#[test]
fn ersetztes_geraet_schreibt_weiter_wird_erkannt() {
    let uhr = TestUhr::neu(20_200, 600);
    let mut a = einrichten(7, &uhr);
    status(&mut a, 0);
    let d1 = sichern(&mut a);
    let mut b = neues_geraet(8, &uhr);
    b.wiederherstellung_beginnen(&a.woerter).unwrap();
    let (_n, _mn, marker) = b
        .wiederherstellen(std::slice::from_ref(&d1), &[], 0)
        .unwrap();
    b.vorbereitetes_abschliessen(None, None).unwrap();
    let (nb, db) = b.sicherung_erstellen().unwrap();
    b.sicherung_bestaetigen(&db).unwrap();
    // Das alte Gerät hat den Marker nicht gesehen, rotiert und sichert weiter (verzweigt).
    let neu = a.kern.rotation_beginnen(Rotationsgrund::AufWunsch).unwrap();
    a.kern.vorbereitetes_abschliessen(None, None).unwrap();
    let _ = neu;
    status(&mut a, -5);
    let (na, da) = a.kern.sicherung_erstellen().unwrap();
    a.kern.sicherung_bestaetigen(&da).unwrap();
    let mut c = neues_geraet(9, &uhr);
    c.wiederherstellung_beginnen(&a.woerter).unwrap();
    // Schon ohne Marker: die Kette von B ersetzt Gerät A, A ist danach weitergelaufen.
    let ohne = c
        .kandidaten(&[(na.clone(), da.clone()), (nb.clone(), db.clone())], &[])
        .unwrap();
    assert!(!ohne[0].gueltig && ohne[1].empfohlen);
    // Mit Marker: die Datei des ersetzten Geräts ist ungültig.
    let mit = c.kandidaten(&[(na, da), (nb, db)], &[marker]).unwrap();
    assert!(
        !mit[0].gueltig,
        "Sicherung des ersetzten Geräts nach der Ersetzung"
    );
    assert!(mit[1].gueltig && mit[1].empfohlen);
}

#[test]
fn zwei_wiederherstellungen_ergeben_zwei_zweige() {
    let uhr = TestUhr::neu(20_250, 600);
    let mut a = einrichten(30, &uhr);
    status(&mut a, 0);
    let d1 = sichern(&mut a);
    let mut aus = Vec::new();
    for seed in [31u8, 32] {
        let mut x = neues_geraet(seed, &uhr);
        x.wiederherstellung_beginnen(&a.woerter).unwrap();
        x.wiederherstellen(std::slice::from_ref(&d1), &[], 0)
            .unwrap();
        x.vorbereitetes_abschliessen(None, None).unwrap();
        let (n, d) = x.sicherung_erstellen().unwrap();
        x.sicherung_bestaetigen(&d).unwrap();
        aus.push((n, d));
    }
    let mut c = neues_geraet(33, &uhr);
    c.wiederherstellung_beginnen(&a.woerter).unwrap();
    let k = c.kandidaten(&aus, &[]).unwrap();
    assert!(
        k.iter().all(|x| x.gueltig && !x.empfohlen),
        "verzweigt: die Person wählt"
    );
    assert_ne!(k[0].zweig, k[1].zweig);
}

#[test]
fn rotation_auf_wunsch_und_abgebrochene_rotation() {
    let uhr = TestUhr::neu(20_300, 600);
    let mut g = einrichten(10, &uhr);
    status(&mut g, 3);
    // Abgebrochen: vorbereitet, aber nicht abgeschlossen -> Bestand unverändert, alter KK gilt.
    let _ = g.kern.rotation_beginnen(Rotationsgrund::AufWunsch).unwrap();
    g.kern.vorbereitetes_verwerfen();
    g.kern.sperren();
    g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).unwrap();
    assert_eq!(stimmungen(&g.kern), vec![3]);
    // Vollständig
    let neu = g.kern.rotation_beginnen(Rotationsgrund::AufWunsch).unwrap();
    g.kern.vorbereitetes_abschliessen(None, None).unwrap();
    g.kern.sperren();
    assert!(g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).is_err());
    g.kern.entsperren(neu.kk.clone(), neu.kk_h.clone()).unwrap();
    assert_eq!(stimmungen(&g.kern), vec![3]);
}

#[test]
fn manipulierte_sicherungen_werden_abgelehnt() {
    let uhr = TestUhr::neu(20_400, 600);
    let mut a = einrichten(11, &uhr);
    status(&mut a, 1);
    let (n, d) = sichern(&mut a);
    let mut b = neues_geraet(12, &uhr);
    b.wiederherstellung_beginnen(&a.woerter).unwrap();
    // Abgeschnitten
    let kurz = (n.clone(), d[..d.len() - 10].to_vec());
    // Ein Byte im Strom verändert
    let mut kaputt = d.clone();
    let i = kaputt.len() - 30;
    kaputt[i] ^= 1;
    // Fremdes Konto mit eigener Sicherung
    let mut fremd = einrichten(13, &uhr);
    status(&mut fremd, 0);
    let (nf, df) = sichern(&mut fremd);
    let k = b
        .kandidaten(&[kurz, (n.clone(), kaputt), (nf, df)], &[])
        .unwrap();
    assert!(
        k.iter().all(|x| !x.gueltig),
        "{:?}",
        k.iter().map(|x| x.grund.clone()).collect::<Vec<_>>()
    );
    // Ein Gerät mit fremdem KK, aber eigener Kette: Bund öffnet nicht.
    assert!(a.kern.sitzung().is_ok());
    a.kern.sperren();
    assert!(
        a.kern
            .entsperren(fremd.kk.clone(), fremd.kk_h.clone())
            .is_err()
    );
}

#[test]
fn wortprobe_spaeter() {
    let uhr = TestUhr::neu(20_500, 600);
    let g = einrichten(14, &uhr);
    assert!(g.kern.woerter_pruefen(&g.woerter).unwrap());
    let mut w = g.woerter.clone();
    w[3] = w[4].clone();
    assert!(!g.kern.woerter_pruefen(&w).unwrap());
}

#[test]
fn entwurf_ueberlebt_das_sperren() {
    let uhr = TestUhr::neu(20_600, 600);
    let mut g = einrichten(15, &uhr);
    let f = vorlagen::formular_id("status_jetzt");
    let a = vec![Antwort {
        messdef: vorlagen::messdef_id("antrieb"),
        zustand: Zustand::Beantwortet,
        wert: Some(Antwortwert {
            zahl: Some(4),
            ..Default::default()
        }),
        aktiv_gesetzt: true,
    }];
    g.kern.sperren();
    let id = g.kern.entwurf_sichern(None, f, a.clone(), false).unwrap();
    g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).unwrap();
    let e = g.kern.entwuerfe().unwrap();
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].antworten, a);
    g.kern
        .status_speichern(
            f,
            None,
            e[0].antworten.clone(),
            false,
            Ausloeser::Selbst,
            None,
            Some(id),
            None,
        )
        .unwrap();
    assert!(g.kern.entwuerfe().unwrap().is_empty());
    // Nach zwölf Stunden verworfen.
    g.kern.entwurf_sichern(None, f, a, false).unwrap();
    uhr.weiter_min(12 * 60 + 1);
    g.kern.sperren();
    g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).unwrap();
    assert!(g.kern.entwuerfe().unwrap().is_empty());
}

/// Lastprobe aus #100: zwei Jahre mit zehn Status je Tag. Läuft in der Pipeline im Release-Profil.
#[test]
#[ignore = "Lastprobe, in der CI mit --ignored im Release-Profil"]
fn lastprobe_zwei_jahre() {
    let uhr = TestUhr::neu(20_000, 7 * 60);
    let mut g = einrichten(16, &uhr);
    let t0 = std::time::Instant::now();
    for _tag in 0..730 {
        for i in 0..10 {
            status(&mut g, (i % 11) - 5);
            uhr.weiter_min(90);
        }
        uhr.weiter_min(1440 - 900);
    }
    let schreiben = t0.elapsed();
    g.kern.sperren();
    let t1 = std::time::Instant::now();
    g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).unwrap();
    let entsperren = t1.elapsed();
    let t2 = std::time::Instant::now();
    let (n, d) = g.kern.sicherung_erstellen().unwrap();
    g.kern.sicherung_bestaetigen(&d).unwrap();
    let sichern = t2.elapsed();
    let t3 = std::time::Instant::now();
    let mut b = neues_geraet(17, &uhr);
    b.wiederherstellung_beginnen(&g.woerter).unwrap();
    b.wiederherstellen(&[(n, d.clone())], &[], 0).unwrap();
    b.vorbereitetes_abschliessen(None, None).unwrap();
    let restore = t3.elapsed();
    assert_eq!(b.sitzung().unwrap().eintraege.len(), 7300);
    println!(
        "LASTPROBE 7300 Status: schreiben {:?}, entsperren {:?}, Sicherung {:?} ({} KiB), Wiederherstellung mit Rotation {:?}",
        schreiben,
        entsperren,
        sichern,
        d.len() / 1024,
        restore
    );
}

// ---------------------------------------------------------------------------------------------
// Recovery-Wörter erst mit der Sicherung (#165)
// ---------------------------------------------------------------------------------------------

fn einrichten_ohne_woerter(seed: u8, uhr: &TestUhr) -> Geraet {
    let mut k = neues_geraet(seed, uhr);
    let s = k.einrichtung_beginnen_ohne_woerter(240).unwrap();
    k.einrichtung_abschliessen(&[], None, None).unwrap();
    Geraet {
        kk: s.kk.clone(),
        kk_h: s.kk_h.clone(),
        kern: k,
        woerter: Vec::new(),
    }
}

fn woerter_einrichten(g: &mut Geraet) {
    let (woerter, probe) = g.kern.woerter_einrichten_beginnen().unwrap();
    let antworten: Vec<String> = probe.iter().map(|i| woerter[*i].clone()).collect();
    g.kern.woerter_einrichten_abschliessen(&antworten).unwrap();
    g.woerter = woerter;
}

#[test]
fn einrichtung_ohne_woerter_sichert_erst_mit_woertern() {
    let uhr = TestUhr::neu(20_700, 600);
    let mut a = einrichten_ohne_woerter(21, &uhr);
    assert!(a.kern.woerter_offen().unwrap());
    assert_eq!(
        a.kern.woerter_offen_seit().unwrap(),
        Some(uhr.jetzt().utc_ms)
    );
    status(&mut a, 3);
    assert_eq!(
        a.kern.sicherung_erstellen().unwrap_err(),
        Fehler::Zustand("erst Recovery-Wörter einrichten"),
        "ohne Wörter keine Sicherung, die niemand öffnen kann"
    );

    // Falsche Probe schreibt nichts.
    let (woerter, probe) = a.kern.woerter_einrichten_beginnen().unwrap();
    assert_eq!(woerter.len(), 24);
    let falsch: Vec<String> = probe.iter().map(|_| "zoo".to_string()).collect();
    assert!(!a.kern.woerter_einrichten_probe_pruefen(&falsch).unwrap());
    assert_eq!(
        a.kern.woerter_einrichten_abschliessen(&falsch).unwrap_err(),
        Fehler::Woerter
    );
    assert!(a.kern.woerter_offen().unwrap());
    let antworten: Vec<String> = probe.iter().map(|i| woerter[*i].clone()).collect();
    a.kern.woerter_einrichten_abschliessen(&antworten).unwrap();
    a.woerter = woerter;
    assert!(!a.kern.woerter_offen().unwrap());
    assert_eq!(a.kern.woerter_offen_seit().unwrap(), None);
    assert!(a.kern.woerter_pruefen(&a.woerter).unwrap());
    assert!(
        a.kern.woerter_einrichten_beginnen().is_err(),
        "bekannte Wörter lassen sich hier nicht ersetzen"
    );

    // Bund und Bund_h sind an den neuen Kettenkopf gebunden: Entsperren klappt weiter.
    a.kern.sperren();
    a.kern.entsperren(a.kk.clone(), a.kk_h.clone()).unwrap();
    assert_eq!(stimmungen(&a.kern), vec![3]);
    let datei = sichern(&mut a);

    // Fremde Wörter öffnen die Sicherung nicht.
    let fremd = einrichten(22, &uhr);
    let mut c = neues_geraet(23, &uhr);
    c.wiederherstellung_beginnen(&fremd.woerter).unwrap();
    assert!(
        c.kandidaten(std::slice::from_ref(&datei), &[])
            .unwrap()
            .iter()
            .all(|k| !k.gueltig)
    );

    // Mit den neuen Wörtern vollständig auf einem zweiten Gerät, beide Kapseln.
    let mut b = neues_geraet(24, &uhr);
    b.wiederherstellung_beginnen(&a.woerter).unwrap();
    let k = b.kandidaten(std::slice::from_ref(&datei), &[]).unwrap();
    assert!(k[0].gueltig && k[0].empfohlen);
    let (neu, _, _) = b
        .wiederherstellen(std::slice::from_ref(&datei), &[], 0)
        .unwrap();
    b.vorbereitetes_abschliessen(None, None).unwrap();
    assert_eq!(stimmungen(&b), vec![3]);
    assert!(!b.woerter_offen().unwrap());
    b.sperren();
    b.entsperren(neu.kk.clone(), neu.kk_h.clone()).unwrap();
    assert!(b.sicherung_erstellen().is_ok());
}

#[test]
fn offene_woerter_verfallen_beim_sperren() {
    let uhr = TestUhr::neu(20_710, 600);
    let mut g = einrichten_ohne_woerter(25, &uhr);
    let (woerter, probe) = g.kern.woerter_einrichten_beginnen().unwrap();
    let antworten: Vec<String> = probe.iter().map(|i| woerter[*i].clone()).collect();
    g.kern.sperren();
    g.kern.entsperren(g.kk.clone(), g.kk_h.clone()).unwrap();
    assert_eq!(
        g.kern
            .woerter_einrichten_abschliessen(&antworten)
            .unwrap_err(),
        Fehler::Zustand("keine Wörter offen")
    );
    assert!(g.kern.woerter_offen().unwrap());
    // Ein neuer Anlauf gibt neue Wörter.
    woerter_einrichten(&mut g);
    assert_ne!(g.woerter, woerter);
    assert!(!g.kern.woerter_offen().unwrap());
}

#[test]
fn einrichtung_mit_woertern_hat_keine_offenen() {
    let uhr = TestUhr::neu(20_720, 600);
    let mut g = einrichten(26, &uhr);
    assert!(!g.kern.woerter_offen().unwrap());
    assert_eq!(g.kern.woerter_offen_seit().unwrap(), None);
    assert!(g.kern.woerter_einrichten_beginnen().is_err());
    assert!(g.kern.sicherung_erstellen().is_ok());
    // Ohne Wörter eingerichtet und gesperrt: neue Wörter nur entsperrt.
    let mut h = einrichten_ohne_woerter(27, &uhr);
    h.kern.sperren();
    assert_eq!(
        h.kern.woerter_einrichten_beginnen().unwrap_err(),
        Fehler::Gesperrt
    );
}
