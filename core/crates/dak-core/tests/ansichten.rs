// SPDX-License-Identifier: MPL-2.0
//! Ansichtsmodelle über die App-Schnittstelle (`dak_core::api`): Verlauf, Planer, Export,
//! Krisenregel, eigene Marker. Alle Daten sind erfunden (Invariante 5).

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use dak_core::api::*;
use dak_core::kern::{Kern, Uhr};
use dak_core::sodium::TestZufall;
use dak_core::speicher;
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

fn kern(seed: u8, tag: i64, minute: i64) -> (Arc<DakKern>, TestUhr) {
    let uhr = TestUhr(Arc::new(AtomicI64::new(
        (tag * 1440 + minute - 120) * 60_000,
    )));
    let k = DakKern::aus_kern(Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(seed)),
        Box::new(uhr.clone()),
    ));
    let e = k.einrichtung_beginnen(240).unwrap();
    let p: Vec<String> = e
        .probe
        .iter()
        .map(|i| e.woerter[*i as usize].clone())
        .collect();
    k.einrichtung_abschliessen(p, None, None).unwrap();
    (k, uhr)
}

fn frage_id(k: &DakKern, formular: &str, schluessel: &str) -> (String, String) {
    let f = k.formular(formular.into()).unwrap();
    let q = f
        .gruppen
        .iter()
        .flat_map(|g| g.fragen.iter())
        .find(|q| q.schluessel == schluessel)
        .unwrap()
        .messdef_id
        .clone();
    (f.id, q)
}

fn stimmung(k: &DakKern, w: i64) {
    let (f, q) = frage_id(k, "status_jetzt", "stimmung");
    k.status_speichern(StatusEingabe {
        formular_id: f,
        zeitpunkt: None,
        antworten: vec![AntwortEingabe {
            messdef_id: q,
            zustand: AntwortZustand::Beantwortet,
            zahl: Some(w),
            text: None,
            auswahl: vec![],
            aktiv_gesetzt: true,
        }],
        block_offen: false,
        aus_entwurf: None,
        korrektur_von: None,
    })
    .unwrap();
}

fn tagesabschluss(k: &DakKern, w: i64) {
    let (f, q) = frage_id(k, "tagesabschluss", "stimmung");
    k.status_speichern(StatusEingabe {
        formular_id: f,
        zeitpunkt: None,
        antworten: vec![AntwortEingabe {
            messdef_id: q,
            zustand: AntwortZustand::Beantwortet,
            zahl: Some(w),
            text: None,
            auswahl: vec![],
            aktiv_gesetzt: true,
        }],
        block_offen: false,
        aus_entwurf: None,
        korrektur_von: None,
    })
    .unwrap();
}

#[test]
fn startseite_zuletzt_meint_denselben_status_wie_der_zaehler() {
    // #122: Zähler und „zuletzt“ betrachten beide nur „Status jetzt“, nicht den Tagesabschluss.
    let t0 = tag_aus_datum(2026, 10, 1);
    let (k, uhr) = kern(6, t0, 8 * 60);
    stimmung(&k, 1);
    let status_ms = uhr.0.load(Ordering::SeqCst);
    uhr.0.fetch_add(13 * 3_600_000, Ordering::SeqCst);
    tagesabschluss(&k, 2);
    let s = k.startseite().unwrap();
    assert_eq!(s.status_heute, 1);
    assert_eq!(
        s.letzter_status_ms,
        Some(status_ms),
        "zuletzt ist der Status von 08:00"
    );
}

#[test]
fn eintragsdetail_zeigt_alle_antworten_mit_notiz() {
    // #129: Die Liste kürzt auf drei Antworten, das Detail zeigt alle, Freitext im Wortlaut.
    let t0 = tag_aus_datum(2026, 10, 2);
    let (k, _uhr) = kern(7, t0, 9 * 60);
    let f = k.formular("status_jetzt".into()).unwrap();
    let id = |schl: &str| {
        f.gruppen
            .iter()
            .flat_map(|g| g.fragen.iter())
            .find(|q| q.schluessel == schl)
            .unwrap()
            .messdef_id
            .clone()
    };
    let zahl = |schl: &str, w: i64| AntwortEingabe {
        messdef_id: id(schl),
        zustand: AntwortZustand::Beantwortet,
        zahl: Some(w),
        text: None,
        auswahl: vec![],
        aktiv_gesetzt: true,
    };
    let mut antworten = vec![
        zahl("stimmung", 2),
        zahl("antrieb", 6),
        zahl("konzentration", 5),
        zahl("muedigkeit", 3),
    ];
    antworten.push(AntwortEingabe {
        messdef_id: id("notiz_jetzt"),
        zustand: AntwortZustand::Beantwortet,
        zahl: None,
        text: Some("Erfundene Notiz".into()),
        auswahl: vec![],
        aktiv_gesetzt: true,
    });
    k.status_speichern(StatusEingabe {
        formular_id: f.id.clone(),
        zeitpunkt: None,
        antworten,
        block_offen: false,
        aus_entwurf: None,
        korrektur_von: None,
    })
    .unwrap();
    let z = k.eintraege(t0, t0).unwrap().remove(0);
    assert!(
        z.zusammenfassung.ends_with("2 weitere"),
        "{}",
        z.zusammenfassung
    );
    assert!(!z.zusammenfassung.contains("Erfundene Notiz"));
    let d = k.eintrag(z.datensatz_id).unwrap();
    assert_eq!(d.alle_antworten.len(), 5, "{:?}", d.alle_antworten);
    assert!(
        d.alle_antworten
            .iter()
            .any(|a| a.ends_with(": Erfundene Notiz"))
    );
}

#[test]
fn schlaf_morgens_anbieten_und_im_verlauf() {
    // #139: Schlaf wird morgens mit „Aufgewacht“ erfasst und erscheint im Verlauf.
    let t0 = tag_aus_datum(2026, 10, 3);
    let (k, uhr) = kern(8, t0, 7 * 60 + 30);
    assert!(
        k.startseite().unwrap().schlaf_anbieten,
        "morgens ohne Eintrag angeboten"
    );
    let f = k.formular("aufgewacht".into()).unwrap();
    let id = |schl: &str| {
        f.gruppen
            .iter()
            .flat_map(|g| g.fragen.iter())
            .find(|q| q.schluessel == schl)
            .unwrap()
            .messdef_id
            .clone()
    };
    let zahl = |schl: &str, w: i64| AntwortEingabe {
        messdef_id: id(schl),
        zustand: AntwortZustand::Beantwortet,
        zahl: Some(w),
        text: None,
        auswahl: vec![],
        aktiv_gesetzt: true,
    };
    k.ereignis_erfassen(EreignisEingabe {
        formular_id: f.id.clone(),
        zeitpunkt: None,
        antworten: vec![zahl("schlaf_dauer", 7 * 60 + 15), zahl("schlaf_guete", 4)],
        praeparat_id: None,
        korrektur_von: None,
    })
    .unwrap();
    assert!(
        !k.startseite().unwrap().schlaf_anbieten,
        "nach dem Eintrag nicht mehr"
    );
    let v = k.verlauf("schlaf_guete".into(), t0, t0).unwrap();
    assert_eq!(v.punkte.len(), 1);
    assert_eq!(v.punkte[0].wert, 4);
    assert_eq!((v.min, v.max), (Some(1), Some(5)));
    let d = k.verlauf("schlaf_dauer".into(), t0, t0).unwrap();
    assert_eq!(d.punkte[0].wert, 435);
    // Am nächsten Tag nachmittags nicht angeboten.
    uhr.0
        .fetch_add(24 * 3_600_000 + 8 * 3_600_000, Ordering::SeqCst);
    assert!(
        !k.startseite().unwrap().schlaf_anbieten,
        "nachmittags nicht"
    );
    // Der Tagesabschluss fragt den Schlaf nicht mehr.
    let ta = k.formular("tagesabschluss".into()).unwrap();
    assert!(
        !ta.gruppen
            .iter()
            .flat_map(|g| g.fragen.iter())
            .any(|q| q.schluessel.contains("schlaf"))
    );
}

#[test]
fn gruppen_aufklappbar_aus_der_vorlage() {
    // #134: Die Vorlage bestimmt, welche Gruppen anfangs zugeklappt sind; die Oberfläche nicht.
    let (k, _u) = kern(9, tag_aus_datum(2026, 10, 4), 9 * 60);
    let titel = |schl: &str| -> Vec<(String, bool)> {
        k.formular(schl.into())
            .unwrap()
            .gruppen
            .iter()
            .map(|g| (g.titel.clone(), g.aufklappbar))
            .collect()
    };
    let st = titel("status_jetzt");
    assert!(st.contains(&("Wie geht es mir".into(), false)));
    assert!(st.contains(&("Was ist gerade stark".into(), true)));
    assert!(st.contains(&("Körper".into(), true)));
    let ta = titel("tagesabschluss");
    assert!(ta.contains(&("Stimmung heute Abend".into(), false)));
    assert!(ta.contains(&("Faktoren".into(), true)));
    assert_eq!(k.startseite().unwrap().heute_lang, "Sonntag, 4. Oktober");
}

#[test]
fn eintragsliste_kurz_lesbar_und_mit_tageskopf() {
    // #135: Kurznamen statt ausgeschriebener Fragen, Dauer lesbar, Tagesköpfe, Schlüssel statt Name.
    let t0 = tag_aus_datum(2026, 10, 6);
    let (k, uhr) = kern(10, t0, 7 * 60 + 30);
    let f = k.formular("aufgewacht".into()).unwrap();
    let q = |schl: &str| {
        f.gruppen
            .iter()
            .flat_map(|g| g.fragen.iter())
            .find(|q| q.schluessel == schl)
            .unwrap()
            .messdef_id
            .clone()
    };
    k.ereignis_erfassen(EreignisEingabe {
        formular_id: f.id.clone(),
        zeitpunkt: None,
        antworten: vec![AntwortEingabe {
            messdef_id: q("schlaf_dauer"),
            zustand: AntwortZustand::Beantwortet,
            zahl: Some(7 * 60 + 20),
            text: None,
            auswahl: vec![],
            aktiv_gesetzt: true,
        }],
        praeparat_id: None,
        korrektur_von: None,
    })
    .unwrap();
    stimmung(&k, 2);
    let heute = k.eintraege(t0, t0).unwrap();
    let st = heute
        .iter()
        .find(|z| z.schluessel == "status_jetzt")
        .unwrap();
    assert_eq!(st.name, "Status");
    assert_eq!(st.zusammenfassung, "Stimmung +2");
    assert_eq!(st.tag_text, "Heute");
    let sl = heute.iter().find(|z| z.schluessel == "aufgewacht").unwrap();
    assert_eq!(sl.zusammenfassung, "Schlaf 7 Std. 20 Min.");
    uhr.0.fetch_add(24 * 3_600_000, Ordering::SeqCst);
    assert_eq!(k.eintraege(t0, t0).unwrap()[0].tag_text, "Gestern");
    uhr.0.fetch_add(24 * 3_600_000, Ordering::SeqCst);
    assert_eq!(
        k.eintraege(t0, t0).unwrap()[0].tag_text,
        "Dienstag, 6. Oktober"
    );
    assert_eq!(k.formular("einnahme".into()).unwrap().name, "Einnahme");
}

#[test]
fn verlauf_zeigt_jeden_eintrag_und_laesst_luecken() {
    let t0 = tag_aus_datum(2026, 9, 1);
    let (k, uhr) = kern(1, t0, 8 * 60);
    // Tag 0: drei Status; Tage 1-4: je einer; Tag 5-9: nichts; Tag 10: einer.
    for w in [-3, 0, 2] {
        stimmung(&k, w);
        uhr.0.fetch_add(4 * 3_600_000, Ordering::SeqCst);
    }
    for _ in 1..=4 {
        uhr.0.fetch_add(24 * 3_600_000, Ordering::SeqCst);
        stimmung(&k, 1);
    }
    uhr.0.fetch_add(6 * 86_400_000, Ordering::SeqCst);
    stimmung(&k, -1);
    let heute = k.heute().unwrap();
    let v = k.verlauf("stimmung".into(), t0, heute).unwrap();
    assert_eq!(v.punkte.len(), 8, "jeder Eintrag ein Punkt");
    let tag0 = v.tage.iter().find(|t| t.lebenstag == t0).unwrap();
    assert_eq!(
        (tag0.anzahl, tag0.min, tag0.max, tag0.letzter),
        (3, -3, 2, 2),
        "Tageswert ist der letzte Eintrag"
    );
    assert!(v.tage_mit_wert < v.tage_gesamt);
    // 7-Tage-Mittel nur mit mindestens vier Tageswerten: am Ende (nach Lücke) leer.
    let ende = v.gleitend7.last().unwrap();
    assert!(ende.wert.is_none(), "keine Interpolation über die Lücke");
    let tag4 = v.gleitend7.iter().find(|m| m.lebenstag == t0 + 4).unwrap();
    assert!(
        (tag4.wert.unwrap() - 1.2).abs() < 1e-9,
        "Mittel über (2, 1, 1, 1, 1): {:?}",
        tag4.wert
    );
}

#[test]
fn planer_neutral_und_begrenzt() {
    let t0 = tag_aus_datum(2026, 10, 12);
    let (k, _uhr) = kern(2, t0, 6 * 60);
    let mut plan = vec![];
    for m in [
        7 * 60,
        9 * 60,
        11 * 60,
        13 * 60,
        15 * 60,
        17 * 60,
        19 * 60,
        20 * 60,
        21 * 60,
    ] {
        plan.push(Planzeit {
            uhrzeit_min: m,
            dosis: "1".into(),
            wochentage: 0x7f,
        });
    }
    k.praeparat_speichern(PraeparatDaten {
        id: String::new(),
        handelsname: "Präparat A".into(),
        wirkstoff: None,
        staerke: None,
        form: None,
        plan,
        bedarf: false,
        wirkung_spuerbar: true,
        aktiv: true,
        erinnerung: true,
        status_nach_min: None,
    })
    .unwrap();
    let b = k.benachrichtigungen(3, 60).unwrap();
    for tag in b
        .iter()
        .map(|x| x.datum_iso.clone())
        .collect::<std::collections::BTreeSet<_>>()
    {
        assert!(
            b.iter().filter(|x| x.datum_iso == tag).count() <= 8,
            "höchstens acht je Tag"
        );
    }
    assert!(
        b.iter()
            .all(|x| !x.text.contains("Präparat A") && !x.titel.contains("Präparat A")),
        "keine Namen in Benachrichtigungen"
    );
    // Medikamente haben Vorrang vor der Abend-Erinnerung (21:00 Tagesabschluss fällt weg, wenn voll).
    assert!(b.iter().filter(|x| x.kanal == "medikament").count() >= 8);
    assert!(
        k.benachrichtigungen(3, 5).unwrap().len() == 5,
        "Obergrenze gesamt (iOS 64)"
    );
}

#[test]
fn export_nur_nach_bestaetigung() {
    let (k, _u) = kern(3, 20_700, 600);
    stimmung(&k, 4);
    assert!(
        k.export_csv(vec![0; 32]).is_err(),
        "ohne Bestätigung kein Export"
    );
    let b = k.bestaetigung_anfordern(Aktion::ExportCsv).unwrap();
    assert!(k.bestaetigung_anfordern(Aktion::RotationAufWunsch).is_ok());
    assert!(
        k.export_csv(b.digest.clone()).is_err(),
        "Digest einer anderen, verdrängten Aktion gilt nicht"
    );
    let b = k.bestaetigung_anfordern(Aktion::ExportCsv).unwrap();
    let csv = k.export_csv(b.digest.clone()).unwrap();
    assert!(csv.starts_with("datum;uhrzeit;zeitzone;art;formular;praeparat;frage;zustand;wert\n"));
    assert!(csv.contains(";4\n"));
    assert!(
        k.export_csv(b.digest).is_err(),
        "Bestätigung nur einmal einlösbar"
    );
}

#[test]
fn krisenregel_und_eigene_marker() {
    let t0 = tag_aus_datum(2026, 10, 1);
    let (k, uhr) = kern(4, t0, 20 * 60);
    let mut e = k.einstellungen().unwrap();
    e.krisenregel_tage = Some(3);
    e.krisenregel_unter = Some(-2);
    e.eigene_marker = vec!["Gedenktag".into()];
    k.einstellungen_speichern(e.clone()).unwrap();
    for w in [-3, -4, -3] {
        stimmung(&k, w);
        uhr.0.fetch_add(86_400_000, Ordering::SeqCst);
    }
    assert!(k.startseite().unwrap().krisenplan_hinweis);
    stimmung(&k, 0);
    assert!(
        !k.startseite().unwrap().krisenplan_hinweis,
        "ein Tag über der Grenze beendet die Folge"
    );
    // Eigener Marker erscheint mit eigenem Namen, unbenannte nicht.
    let f = k.formular("tagesabschluss".into()).unwrap();
    let m = f
        .gruppen
        .iter()
        .flat_map(|g| g.fragen.iter())
        .find(|q| q.schluessel == "tag_marker")
        .unwrap();
    assert!(
        m.optionen
            .iter()
            .any(|o| o.id == 11 && o.text == "Gedenktag")
    );
    assert!(!m.optionen.iter().any(|o| o.id == 12));
    // Stimmung ausblenden bei aktiver Krisenregel wird abgelehnt.
    e.ausgeblendet = vec![frage_id(&k, "status_jetzt", "stimmung").1];
    assert!(k.einstellungen_speichern(e).is_err());
}

#[test]
fn hilfe_ohne_entsperren() {
    let (k, _u) = kern(5, 20_800, 600);
    k.krisenplan_setzen(Some("Spazieren gehen, Person X anrufen".into()))
        .unwrap();
    k.sperren();
    let h = k.hilfe().unwrap();
    assert!(h.nummern.iter().any(|n| n.nummer == "112"));
    assert!(h.nummern.iter().any(|n| n.nummer == "0800 111 0 111"));
    assert_eq!(
        h.krisenplan.as_deref(),
        Some("Spazieren gehen, Person X anrufen")
    );
    assert!(h.vertrauenspersonen.is_empty(), "Kontakte nur entsperrt");
    assert!(k.startseite().is_err(), "sonst nichts ohne Entsperren");
}

#[test]
fn sicherungshinweis_erst_ab_dem_zweiten_tag_und_ohne_woerter_eingerichtet() {
    // #165: Einrichtung ohne Recovery-Wörter, der Hinweis auf die Sicherung wartet einen Tag.
    let t0 = tag_aus_datum(2026, 10, 2);
    let uhr = TestUhr(Arc::new(AtomicI64::new(
        (t0 * 1440 + 9 * 60 - 120) * 60_000,
    )));
    let k = DakKern::aus_kern(Kern::neu(
        speicher::im_speicher().unwrap(),
        Box::new(TestZufall::neu(31)),
        Box::new(uhr.clone()),
    ));
    k.einrichtung_beginnen_ohne_woerter(240).unwrap();
    k.einrichtung_abschliessen(Vec::new(), None, None).unwrap();
    let seit = k.woerter_offen_seit().unwrap();
    assert_eq!(seit, Some(uhr.0.load(Ordering::SeqCst)));
    assert!(
        !k.startseite().unwrap().sicherung_warnung,
        "am ersten Tag still"
    );
    uhr.0.fetch_add(23 * 3_600_000, Ordering::SeqCst);
    assert!(!k.startseite().unwrap().sicherung_warnung);
    uhr.0.fetch_add(2 * 3_600_000, Ordering::SeqCst);
    assert!(
        k.startseite().unwrap().sicherung_warnung,
        "ab dem zweiten Tag"
    );
    assert!(
        k.sicherung_erstellen().is_err(),
        "ohne Wörter keine Sicherung"
    );
    let w = k.woerter_einrichten_beginnen().unwrap();
    let p: Vec<String> = w
        .probe
        .iter()
        .map(|i| w.woerter[*i as usize].clone())
        .collect();
    assert!(k.woerter_einrichten_probe_pruefen(p.clone()).unwrap());
    k.woerter_einrichten_abschliessen(p).unwrap();
    assert_eq!(k.woerter_offen_seit().unwrap(), None);
    let d = k.sicherung_erstellen().unwrap();
    k.sicherung_bestaetigen(d.bytes).unwrap();
    assert!(
        !k.startseite().unwrap().sicherung_warnung,
        "nach der Sicherung still"
    );
}
