// SPDX-License-Identifier: MPL-2.0
//! Ansichtsmodelle (AE-06): was sichtbar ist, welcher Bezugszeitraum gilt, welche Punkte ein
//! Verlauf hat. Beschreibend, ohne Deutung, ohne Farbwertung (AE-27, #74).

use std::collections::BTreeMap;

use crate::api::*;
use crate::fehler::{Ergebnis, Fehler};
use crate::kern::{DREI_TAGE_MS, Kern, Sitzung, ZWEI_STUNDEN_MS};
use crate::modell::{
    self, Antwort, BedingungArt, Bezugsart, Einstellungen, Erinnerung, ErinnerungArt, Feldtyp,
    FormularArt, Id, Krisenregel, Objekt, Polung, Zustand, hex,
};
use crate::speicher;
use crate::vorlagen::{self, EIGENE_MARKER_AB, STIMMUNG, TAG_MARKER};
use crate::zeit::{self, Zeitpunkt};

pub(crate) fn aenderung(k: &mut Kern) -> Ergebnis<()> {
    let n = speicher::meta_u64(&k.db, "aenderungen")?.unwrap_or(0) + 1;
    speicher::meta_u64_setzen(&k.db, "aenderungen", n)
}

fn feldart(f: Feldtyp) -> Feldart {
    match f {
        Feldtyp::Skala => Feldart::Skala,
        Feldtyp::Stufen => Feldart::Stufen,
        Feldtyp::Einfachauswahl => Feldart::Einfachauswahl,
        Feldtyp::Mehrfachauswahl => Feldart::Mehrfachauswahl,
        Feldtyp::Text => Feldart::Text,
        Feldtyp::Dauer => Feldart::Dauer,
        Feldtyp::Uhrzeit => Feldart::Uhrzeit,
        Feldtyp::Zahl => Feldart::Zahl,
        Feldtyp::JaNein => Feldart::JaNein,
    }
}

fn richtung(p: Polung) -> Richtung {
    match p {
        Polung::Keine => Richtung::Keine,
        Polung::HoeherBesser => Richtung::HoeherBesser,
        Polung::HoeherMehr => Richtung::HoeherMehr,
        Polung::Bipolar => Richtung::Bipolar,
    }
}

fn formularart(a: FormularArt) -> Formularart {
    match a {
        FormularArt::StatusJetzt => Formularart::StatusJetzt,
        FormularArt::Tagesabschluss => Formularart::Tagesabschluss,
        FormularArt::Wochenrueckblick => Formularart::Wochenrueckblick,
        FormularArt::Ereignis => Formularart::Ereignis,
    }
}

/// Optionen mit den selbst gewählten Namen der eigenen Marker; unbenannte eigene Marker fehlen.
fn optionen(s: &Sitzung, md: &modell::Messdef) -> Vec<Wahl> {
    md.optionen
        .iter()
        .filter_map(|o| {
            if md.schluessel == TAG_MARKER && o.id >= EIGENE_MARKER_AB {
                let i = (o.id - EIGENE_MARKER_AB) as usize;
                s.bund.einstellungen.eigene_marker.get(i).map(|t| Wahl {
                    id: o.id,
                    text: t.clone(),
                })
            } else {
                Some(Wahl {
                    id: o.id,
                    text: o.text.clone(),
                })
            }
        })
        .collect()
}

fn frage(s: &Sitzung, md: &modell::Messdef) -> FrageAnsicht {
    FrageAnsicht {
        messdef_id: hex(&md.id),
        schluessel: md.schluessel.clone(),
        frage: md.frage.clone(),
        feldart: feldart(md.feldtyp),
        min: md.min,
        max: md.max,
        anker: md
            .anker
            .iter()
            .map(|(w, t)| Anker {
                wert: *w,
                text: t.clone(),
            })
            .collect(),
        optionen: optionen(s, md),
        mit_staerke: md.mit_staerke,
        kam_nicht_vor: md.kam_nicht_vor,
        richtung: richtung(md.polung),
        einheit: md.einheit.clone(),
        hinweis: md.hinweis.clone(),
    }
}

pub(crate) fn formular(k: &Kern, schluessel: &str) -> Ergebnis<FormularAnsicht> {
    let s = k.sitzung()?;
    let f = s
        .formular_schluessel(schluessel)
        .ok_or(Fehler::NichtGefunden("Formular"))?;
    let jetzt = k.jetzt();
    let heute = jetzt.lebenstag(s.tageswechsel());
    let wirkung = k.einnahme_mit_wirkung_heute(heute)?;
    let mut gruppen = Vec::new();
    for g in &f.gruppen {
        let fragen: Vec<FrageAnsicht> = g
            .messdefs
            .iter()
            .filter(|id| !s.bund.einstellungen.ausgeblendet.contains(id))
            .filter(|id| {
                !(f.bedingungen
                    .iter()
                    .any(|(b, a)| b == *id && *a == BedingungArt::EinnahmeMitWirkungHeute)
                    && !wirkung)
            })
            .filter_map(|id| s.messdef(id).map(|m| frage(s, m)))
            .collect();
        if !fragen.is_empty() || f.art == FormularArt::Ereignis {
            gruppen.push(GruppeAnsicht {
                titel: g.titel.clone(),
                hinweis: g.hinweis.clone(),
                zugeklappt: g.zugeklappt,
                fragen,
            });
        }
    }
    let hat_block = f.gruppen.iter().any(|g| g.zugeklappt);
    let (bezug_text, block_anbieten) = if hat_block {
        let (von, gekappt) = k.bezug_von(jetzt.utc_ms)?;
        let text = if gekappt {
            "in den letzten zwölf Stunden".to_owned()
        } else {
            let z = Zeitpunkt {
                utc_ms: von.unwrap_or(jetzt.utc_ms),
                versatz_min: jetzt.versatz_min,
                zone: jetzt.zone.clone(),
            };
            format!("seit {} Uhr", zeit::uhrzeit_text(z.uhrzeit_min()))
        };
        let abstand = k.letzter_status_ms()?.map(|t| jetzt.utc_ms - t);
        (Some(text), abstand.is_none_or(|a| a >= ZWEI_STUNDEN_MS))
    } else {
        (None, false)
    };
    Ok(FormularAnsicht {
        id: hex(&f.id),
        schluessel: f.schluessel.clone(),
        name: f.name.clone(),
        art: formularart(f.art),
        gruppen,
        bezug_text,
        block_anbieten,
        braucht_praeparat: matches!(
            f.schluessel.as_str(),
            "einnahme" | "einnahme_ausgelassen" | "dosiswechsel"
        ),
    })
}

fn wochentag_bit(tag: i64) -> u8 {
    1 << zeit::wochentag(tag)
}

pub(crate) fn startseite(k: &Kern) -> Ergebnis<Startseite> {
    let s = k.sitzung()?;
    let jetzt = k.jetzt();
    let heute = jetzt.lebenstag(s.tageswechsel());
    let mut ereignisse: Vec<&modell::Formular> = s
        .bund
        .formulare
        .iter()
        .filter(|f| f.art == FormularArt::Ereignis && f.eingeschaltet && f.schluessel != "einnahme")
        .collect();
    ereignisse.sort_by_key(|f| f.reihenfolge);
    let knopf = |f: &modell::Formular| Knopf {
        formular_id: hex(&f.id),
        schluessel: f.schluessel.clone(),
        name: f.name.clone(),
    };
    let mut einnahmen_heute = Vec::new();
    let mut bedarf = Vec::new();
    for p in s.bund.praeparate.iter().filter(|p| p.aktiv) {
        if p.bedarf {
            bedarf.push(Knopf {
                formular_id: hex(&p.id),
                schluessel: "einnahme".into(),
                name: p.handelsname.clone(),
            });
        }
        let eintraege: Vec<&modell::Ereignis> = s
            .eintraege
            .iter()
            .filter_map(|e| match &e.objekt {
                Objekt::Ereignis(ev)
                    if ev.lebenstag == heute && ev.praeparat == Some(p.id) && !ev.vorlaeufig =>
                {
                    Some(ev)
                }
                _ => None,
            })
            .collect();
        let genommen = eintraege.iter().filter(|e| e.art == "einnahme").count();
        let ausgelassen = eintraege
            .iter()
            .filter(|e| e.art == "einnahme_ausgelassen")
            .count();
        let mut plan: Vec<&modell::Planzeit> = p
            .plan
            .iter()
            .filter(|z| z.wochentage & wochentag_bit(heute) != 0)
            .collect();
        plan.sort_by_key(|z| z.uhrzeit_min);
        for (i, z) in plan.iter().enumerate() {
            // Reihenfolge: zuerst genommene, dann ausgelassene Einnahmen den Planzeiten zuordnen.
            let stand = if i < genommen {
                EinnahmeStand::Genommen
            } else if i < genommen + ausgelassen {
                EinnahmeStand::Ausgelassen
            } else {
                EinnahmeStand::Offen
            };
            einnahmen_heute.push(EinnahmeHeute {
                praeparat_id: hex(&p.id),
                name: p.handelsname.clone(),
                staerke: p.staerke.clone(),
                uhrzeit_min: z.uhrzeit_min,
                dosis: z.dosis.clone(),
                stand,
            });
        }
    }
    einnahmen_heute.sort_by_key(|e| e.uhrzeit_min);
    let entwuerfe = k
        .entwuerfe()?
        .into_iter()
        .map(|e| EntwurfKurz {
            id: hex(&e.id),
            formular_id: hex(&e.formular),
            formular_name: s
                .formular(&e.formular)
                .map(|f| f.name.clone())
                .unwrap_or_default(),
            erstellt_ms: e.erstellt.utc_ms,
            antworten: e
                .antworten
                .iter()
                .filter(|a| a.zustand == Zustand::Beantwortet)
                .count() as u32,
        })
        .collect();
    let status_heute: Vec<&modell::Status> = s
        .eintraege
        .iter()
        .filter_map(|e| match &e.objekt {
            Objekt::Status(st) if st.lebenstag == heute => Some(st),
            _ => None,
        })
        .collect();
    let letzte = k.letzte_sicherung()?;
    let aenderungen = speicher::meta_u64(&k.db, "aenderungen")?.unwrap_or(0);
    let gesichert = speicher::meta_u64(&k.db, "aenderungen_gesichert")?.unwrap_or(0);
    let erinnert = |art: ErinnerungArt| {
        s.bund
            .einstellungen
            .erinnerungen
            .iter()
            .any(|e| e.art == art && e.aktiv && e.wochentage & wochentag_bit(heute) != 0)
    };
    Ok(Startseite {
        heute: zeit::datum_text(heute),
        ereignisse: ereignisse.into_iter().map(knopf).collect(),
        einnahmen_heute,
        bedarf,
        entwuerfe,
        status_heute: status_heute
            .iter()
            .filter(|st| st.art == FormularArt::StatusJetzt)
            .count() as u32,
        letzter_status_ms: k.letzter_status_jetzt_ms()?,
        tagesabschluss_heute: !status_heute
            .iter()
            .any(|st| st.art == FormularArt::Tagesabschluss),
        wochenrueckblick_heute: erinnert(ErinnerungArt::Wochenrueckblick)
            && !status_heute
                .iter()
                .any(|st| st.art == FormularArt::Wochenrueckblick),
        letzte_sicherung_ms: letzte,
        sicherung_warnung: letzte.is_none_or(|t| jetzt.utc_ms - t > DREI_TAGE_MS),
        sicherung_noetig: aenderungen > gesichert
            || letzte.is_none_or(|t| {
                zeit::Zeitpunkt {
                    utc_ms: t,
                    ..jetzt.clone()
                }
                .lebenstag(s.tageswechsel())
                    < heute
            }),
        krisenplan_hinweis: krisenregel_greift(s, heute),
    })
}

/// Tageswert je Lebenstag für eine Messdefinition: der zeitlich letzte Wert des Tages.
fn tageswerte(s: &Sitzung, md: &Id) -> BTreeMap<i64, i64> {
    let mut m: BTreeMap<i64, (i64, i64)> = BTreeMap::new();
    for e in &s.eintraege {
        if let Objekt::Status(st) = &e.objekt
            && !st.vorlaeufig
            && let Some(w) = wert(&st.antworten, md)
        {
            let t = st.zeitpunkt.utc_ms;
            let ein = m.entry(st.lebenstag).or_insert((t, w));
            if t >= ein.0 {
                *ein = (t, w);
            }
        }
    }
    m.into_iter().map(|(k, (_, w))| (k, w)).collect()
}

fn wert(a: &[Antwort], md: &Id) -> Option<i64> {
    a.iter()
        .find(|x| x.messdef == *md && x.zustand == Zustand::Beantwortet)
        .and_then(|x| x.wert.as_ref()?.zahl)
}

fn krisenregel_greift(s: &Sitzung, heute: i64) -> bool {
    let Some(Krisenregel { tage, unter }) = s.bund.einstellungen.krisenregel.clone() else {
        return false;
    };
    let tw = tageswerte(s, &vorlagen::messdef_id(STIMMUNG));
    // Gilt für die letzten `tage` Tage bis heute oder bis gestern (heute noch ohne Eintrag).
    let ende = if tw.contains_key(&heute) {
        heute
    } else {
        heute - 1
    };
    (0..tage as i64).all(|i| tw.get(&(ende - i)).is_some_and(|w| *w < unter))
}

fn gleitend(
    tw: &BTreeMap<i64, i64>,
    von: i64,
    bis: i64,
    fenster: i64,
    mindestens: usize,
) -> Vec<Mittel> {
    (von..=bis)
        .map(|t| {
            let v: Vec<i64> = ((t - fenster + 1)..=t)
                .filter_map(|d| tw.get(&d).copied())
                .collect();
            Mittel {
                lebenstag: t,
                wert: (v.len() >= mindestens)
                    .then(|| v.iter().sum::<i64>() as f64 / v.len() as f64),
            }
        })
        .collect()
}

pub(crate) fn verlauf(k: &Kern, schluessel: &str, von: i64, bis: i64) -> Ergebnis<Verlauf> {
    let s = k.sitzung()?;
    if bis < von || bis - von > 3660 {
        return Err(Fehler::Eingabe("Zeitraum"));
    }
    let md = s
        .messdef_schluessel(schluessel)
        .ok_or(Fehler::NichtGefunden("Frage"))?;
    if !crate::kern::feld_ist_zahl(md.feldtyp) {
        return Err(Fehler::Eingabe("Verlauf nur für Zahlenwerte"));
    }
    let mut punkte = Vec::new();
    let mut tage: BTreeMap<i64, Vec<(i64, i64)>> = BTreeMap::new();
    let mut einnahmen_ms = Vec::new();
    let mut marker = Vec::new();
    let marker_md = s
        .messdef_schluessel(TAG_MARKER)
        .map(|m| (m.id, optionen(s, m)));
    for e in &s.eintraege {
        let lt = e.objekt.lebenstag();
        if lt < von || lt > bis {
            continue;
        }
        match &e.objekt {
            Objekt::Status(st) if !st.vorlaeufig => {
                if let Some(w) = wert(&st.antworten, &md.id) {
                    punkte.push(Punkt {
                        utc_ms: st.zeitpunkt.utc_ms,
                        lebenstag: lt,
                        uhrzeit_min: st.zeitpunkt.uhrzeit_min(),
                        wert: w,
                        aus_tagesabschluss: st.art == FormularArt::Tagesabschluss,
                    });
                    tage.entry(lt).or_default().push((st.zeitpunkt.utc_ms, w));
                }
                if let Some((mid, opts)) = &marker_md
                    && let Some(a) = st
                        .antworten
                        .iter()
                        .find(|a| a.messdef == *mid && a.zustand == Zustand::Beantwortet)
                {
                    for (o, _) in &a
                        .wert
                        .as_ref()
                        .map(|w| w.auswahl.clone())
                        .unwrap_or_default()
                    {
                        if let Some(name) = opts.iter().find(|x| x.id == *o) {
                            marker.push(Marker {
                                lebenstag: lt,
                                option: *o,
                                name: name.text.clone(),
                            });
                        }
                    }
                }
            }
            Objekt::Ereignis(ev) if ev.art == "einnahme" && !ev.vorlaeufig => {
                einnahmen_ms.push(ev.zeitpunkt.utc_ms)
            }
            _ => {}
        }
    }
    let tw: BTreeMap<i64, i64> = tage
        .iter()
        .map(|(t, v)| (*t, v.iter().max_by_key(|x| x.0).map(|x| x.1).unwrap_or(0)))
        .collect();
    let tage_v: Vec<Tageswert> = tage
        .iter()
        .map(|(t, v)| Tageswert {
            lebenstag: *t,
            datum: zeit::datum_text(*t),
            anzahl: v.len() as u32,
            min: v.iter().map(|x| x.1).min().unwrap_or(0),
            max: v.iter().map(|x| x.1).max().unwrap_or(0),
            letzter: tw[t],
        })
        .collect();
    let gesamt = (bis - von + 1) as u32;
    Ok(Verlauf {
        messdef_id: hex(&md.id),
        frage: md.frage.clone(),
        min: md.min,
        max: md.max,
        anker: md
            .anker
            .iter()
            .map(|(w, t)| Anker {
                wert: *w,
                text: t.clone(),
            })
            .collect(),
        richtung: richtung(md.polung),
        tage_mit_wert: tage_v.len() as u32,
        tage_gesamt: gesamt,
        gleitend7: gleitend(&tw, von, bis, 7, 4),
        gleitend28: gleitend(&tw, von, bis, 28, 14),
        punkte,
        tage: tage_v,
        marker,
        einnahmen_ms,
        hinweis: format!(
            "Jeder Punkt ist ein Eintrag. Der Tageswert ist der letzte Eintrag des Tages. Mittelwerte nur bei genügend Tagen (7 Tage: mindestens 4, 28 Tage: mindestens 14); Lücken bleiben Lücken. An {} von {} Tagen gibt es einen Wert.",
            tw.len(),
            gesamt
        ),
    })
}

pub(crate) fn faktorvergleich(
    k: &Kern,
    faktor: &str,
    von: i64,
    bis: i64,
) -> Ergebnis<Faktorvergleich> {
    let s = k.sitzung()?;
    let f = s
        .messdef_schluessel(faktor)
        .ok_or(Fehler::NichtGefunden("Faktor"))?;
    if f.feldtyp != Feldtyp::JaNein {
        return Err(Fehler::Eingabe("Faktor muss ja/nein sein"));
    }
    let stimmung = tageswerte(s, &vorlagen::messdef_id(STIMMUNG));
    let faktor_tage = {
        let mut m = BTreeMap::new();
        for e in &s.eintraege {
            if let Objekt::Status(st) = &e.objekt
                && let Some(w) = wert(&st.antworten, &f.id)
            {
                m.insert(st.lebenstag, w);
            }
        }
        m
    };
    let (mut mit, mut ohne) = (Vec::new(), Vec::new());
    for t in von..=bis {
        if let (Some(w), Some(fv)) = (stimmung.get(&t), faktor_tage.get(&t)) {
            if *fv == 1 {
                mit.push(*w)
            } else {
                ohne.push(*w)
            }
        }
    }
    let m = |v: &Vec<i64>| (!v.is_empty()).then(|| v.iter().sum::<i64>() as f64 / v.len() as f64);
    Ok(Faktorvergleich {
        faktor: f.frage.clone(),
        tage_mit: mit.len() as u32,
        mittel_mit: m(&mit),
        tage_ohne: ohne.len() as u32,
        mittel_ohne: m(&ohne),
        hinweis: "Rein beschreibend: Tage mit und ohne diese Angabe, jeweils mit Fallzahl. Kein Beleg für eine Ursache.".into(),
    })
}

/// Antworten als „Frage: Wert“. `voll` zeigt Freitexte im Wortlaut (Detail, #129), sonst nur „Text“.
fn antwort_texte(s: &Sitzung, a: &[Antwort], voll: bool) -> Vec<String> {
    a.iter()
        .filter(|x| x.zustand == Zustand::Beantwortet)
        .filter_map(|x| {
            let md = s.messdef(&x.messdef)?;
            let w = x.wert.as_ref();
            let wert = match md.feldtyp {
                Feldtyp::Text if voll => w.and_then(|w| w.text.clone()).unwrap_or_default(),
                Feldtyp::Text => "Text".to_owned(),
                Feldtyp::Einfachauswahl | Feldtyp::Mehrfachauswahl => optionen(s, md)
                    .iter()
                    .filter(|o| w.is_some_and(|w| w.auswahl.iter().any(|(id, _)| *id == o.id)))
                    .map(|o| o.text.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                Feldtyp::JaNein => if w.and_then(|w| w.zahl) == Some(1) {
                    "ja"
                } else {
                    "nein"
                }
                .to_owned(),
                _ => match (md.polung, w.and_then(|w| w.zahl)) {
                    (Polung::Bipolar, Some(z)) if z > 0 => format!("+{z}"),
                    (_, Some(z)) => z.to_string(),
                    _ => String::new(),
                },
            };
            Some(format!("{}: {}", md.frage.trim_end_matches('?'), wert))
        })
        .collect()
}

fn praeparat_name(s: &Sitzung, praeparat: Option<&Id>) -> Option<String> {
    praeparat
        .and_then(|p| s.bund.praeparate.iter().find(|x| x.id == *p))
        .map(|p| p.handelsname.clone())
}

fn zusammenfassung(s: &Sitzung, a: &[Antwort], praeparat: Option<&Id>) -> String {
    let mut teile: Vec<String> = praeparat_name(s, praeparat).into_iter().collect();
    let alle = antwort_texte(s, a, false);
    teile.extend(alle.iter().take(3).cloned());
    if alle.len() > 3 {
        teile.push(format!("{} weitere", alle.len() - 3));
    }
    teile.join(" · ")
}

fn zeile(s: &Sitzung, e: &crate::kern::Eintrag) -> EintragZeile {
    let (ist_ereignis, formular, antworten, praeparat) = match &e.objekt {
        Objekt::Status(st) => (false, st.formular, &st.antworten, None),
        Objekt::Ereignis(ev) => (true, ev.formular, &ev.antworten, ev.praeparat.as_ref()),
    };
    let z = e.objekt.zeitpunkt();
    EintragZeile {
        datensatz_id: hex(&e.datensatz_id),
        ist_ereignis,
        formular_id: hex(&formular),
        name: s
            .formular(&formular)
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "Eintrag".into()),
        utc_ms: z.utc_ms,
        lebenstag: e.objekt.lebenstag(),
        datum: zeit::datum_text(e.objekt.lebenstag()),
        uhrzeit: zeit::uhrzeit_text(z.uhrzeit_min()),
        zusammenfassung: zusammenfassung(s, antworten, praeparat),
    }
}

pub(crate) fn eintraege(k: &Kern, von: i64, bis: i64) -> Ergebnis<Vec<EintragZeile>> {
    let s = k.sitzung()?;
    let mut v: Vec<EintragZeile> = s
        .eintraege
        .iter()
        .filter(|e| (von..=bis).contains(&e.objekt.lebenstag()))
        .map(|e| zeile(s, e))
        .collect();
    v.sort_by_key(|z| std::cmp::Reverse(z.utc_ms));
    Ok(v)
}

pub(crate) fn eintrag(k: &Kern, id: &Id) -> Ergebnis<EintragDetail> {
    let s = k.sitzung()?;
    let e = s
        .eintraege
        .iter()
        .find(|e| e.datensatz_id == *id)
        .ok_or(Fehler::NichtGefunden("Eintrag"))?;
    let (antworten, praeparat, block_offen) = match &e.objekt {
        Objekt::Status(st) => (&st.antworten, None, st.bezug_von.is_some()),
        Objekt::Ereignis(ev) => (&ev.antworten, ev.praeparat.map(|p| hex(&p)), true),
    };
    let praep_id = match &e.objekt {
        Objekt::Ereignis(ev) => ev.praeparat.as_ref(),
        Objekt::Status(_) => None,
    };
    let mut alle: Vec<String> = praeparat_name(s, praep_id).into_iter().collect();
    alle.extend(antwort_texte(s, antworten, true));
    Ok(EintragDetail {
        zeile: zeile(s, e),
        alle_antworten: alle,
        zeitpunkt: e.objekt.zeitpunkt().clone(),
        antworten: antworten.iter().filter_map(antwort_zu).collect(),
        praeparat_id: praeparat,
        block_offen,
    })
}

pub(crate) fn praeparat_zu(p: &modell::Praeparat) -> PraeparatDaten {
    PraeparatDaten {
        id: hex(&p.id),
        handelsname: p.handelsname.clone(),
        wirkstoff: p.wirkstoff.clone(),
        staerke: p.staerke.clone(),
        form: p.form.clone(),
        plan: p
            .plan
            .iter()
            .map(|z| Planzeit {
                uhrzeit_min: z.uhrzeit_min,
                dosis: z.dosis.clone(),
                wochentage: z.wochentage,
            })
            .collect(),
        bedarf: p.bedarf,
        wirkung_spuerbar: p.wirkung_spuerbar,
        aktiv: p.aktiv,
        erinnerung: p.erinnerung,
        status_nach_min: p.status_nach_min,
    }
}

pub(crate) fn praeparat_aus(p: PraeparatDaten) -> Ergebnis<modell::Praeparat> {
    let leer = |o: Option<String>| o.filter(|s| !s.trim().is_empty());
    Ok(modell::Praeparat {
        id: if p.id.is_empty() { [0; 16] } else { id(&p.id)? },
        handelsname: p.handelsname.trim().to_owned(),
        wirkstoff: leer(p.wirkstoff),
        staerke: leer(p.staerke),
        form: leer(p.form),
        plan: p
            .plan
            .into_iter()
            .map(|z| modell::Planzeit {
                uhrzeit_min: z.uhrzeit_min,
                dosis: z.dosis,
                wochentage: z.wochentage,
            })
            .collect(),
        bedarf: p.bedarf,
        wirkung_spuerbar: p.wirkung_spuerbar,
        aktiv: p.aktiv,
        erinnerung: p.erinnerung,
        status_nach_min: p.status_nach_min,
    })
}

pub(crate) fn einstellungen_zu(e: &Einstellungen) -> EinstellungenDaten {
    EinstellungenDaten {
        tageswechsel_min: e.tageswechsel_min,
        sperrzeit_s: e.sperrzeit_s,
        erinnerungen: e
            .erinnerungen
            .iter()
            .map(|x| ErinnerungDaten {
                art: x.art as u8,
                uhrzeit_min: x.uhrzeit_min,
                wochentage: x.wochentage,
                aktiv: x.aktiv,
            })
            .collect(),
        max_benachrichtigungen: e.max_benachrichtigungen,
        vertrauenspersonen: e
            .vertrauenspersonen
            .iter()
            .map(|v| Vertrauensperson {
                name: v.name.clone(),
                telefon: v.telefon.clone(),
            })
            .collect(),
        krisenregel_tage: e.krisenregel.as_ref().map(|k| k.tage),
        krisenregel_unter: e.krisenregel.as_ref().map(|k| k.unter),
        ausgeblendet: e.ausgeblendet.iter().map(|i| hex(i)).collect(),
        eigene_marker: e.eigene_marker.clone(),
    }
}

pub(crate) fn einstellungen_aus(e: EinstellungenDaten) -> Ergebnis<Einstellungen> {
    let mut erinnerungen = Vec::new();
    for x in e.erinnerungen {
        erinnerungen.push(Erinnerung {
            art: match x.art {
                1 => ErinnerungArt::Status,
                2 => ErinnerungArt::Tagesabschluss,
                3 => ErinnerungArt::Wochenrueckblick,
                _ => return Err(Fehler::Eingabe("Erinnerungsart")),
            },
            uhrzeit_min: x.uhrzeit_min,
            wochentage: x.wochentage,
            aktiv: x.aktiv,
        });
    }
    let krisenregel = match (e.krisenregel_tage, e.krisenregel_unter) {
        (Some(tage), Some(unter)) => Some(Krisenregel { tage, unter }),
        (None, None) => None,
        _ => return Err(Fehler::Eingabe("Regel für den Krisenplan unvollständig")),
    };
    let stimmung = vorlagen::messdef_id(STIMMUNG);
    let ausgeblendet = e
        .ausgeblendet
        .iter()
        .map(|s| id(s))
        .collect::<Ergebnis<Vec<_>>>()?;
    if ausgeblendet.contains(&stimmung) && krisenregel.is_some() {
        return Err(Fehler::Eingabe(
            "Die Stimmungsfrage ist Auslöser des Krisenplans und lässt sich dann nicht ausblenden",
        ));
    }
    Ok(Einstellungen {
        tageswechsel_min: e.tageswechsel_min,
        sperrzeit_s: e.sperrzeit_s,
        erinnerungen,
        max_benachrichtigungen: e.max_benachrichtigungen,
        vertrauenspersonen: e
            .vertrauenspersonen
            .into_iter()
            .map(|v| modell::Vertrauensperson {
                name: v.name,
                telefon: v.telefon,
            })
            .collect(),
        krisenregel,
        ausgeblendet,
        eigene_marker: e
            .eigene_marker
            .into_iter()
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .collect(),
        // Der Plan hat einen eigenen Weg (krisenplan_setzen); einstellungen_setzen behält ihn.
        krisenplan: None,
    })
}

pub(crate) fn hilfe(k: &Kern) -> Ergebnis<Hilfe> {
    let n = |name: &str, nummer: &str, hinweis: &str| Nummer {
        name: name.into(),
        nummer: nummer.into(),
        hinweis: hinweis.into(),
    };
    Ok(Hilfe {
        nummern: vec![
            n("Notruf", "112", "bei akuter Gefahr"),
            n("TelefonSeelsorge", "0800 111 0 111", "rund um die Uhr, kostenfrei, anonym"),
            n("TelefonSeelsorge", "0800 111 0 222", "rund um die Uhr, kostenfrei, anonym"),
            n("TelefonSeelsorge", "116 123", "rund um die Uhr, kostenfrei, anonym"),
            n("Nummer gegen Kummer", "116 111", "für Kinder und Jugendliche"),
            n("Ärztlicher Bereitschaftsdienst", "116 117", "wenn die Praxis geschlossen ist"),
        ],
        krisenplan: k.krisenplan()?,
        vertrauenspersonen: k
            .sitzung()
            .map(|s| s.bund.einstellungen.vertrauenspersonen.iter().map(|v| Vertrauensperson { name: v.name.clone(), telefon: v.telefon.clone() }).collect())
            .unwrap_or_default(),
        hinweis: "Die App benachrichtigt niemanden. Wenn es dir schlecht geht, ruf eine der Nummern an oder sprich mit einer Person, der du vertraust.".into(),
    })
}

fn csv(t: &str) -> String {
    let t = t.replace(['\r', '\n'], " ");
    if t.contains(';') || t.contains('"') {
        format!("\"{}\"", t.replace('"', "\"\""))
    } else {
        t
    }
}

pub(crate) fn export_csv(k: &Kern) -> Ergebnis<String> {
    let s = k.sitzung()?;
    let mut out =
        String::from("datum;uhrzeit;zeitzone;art;formular;praeparat;frage;zustand;wert\n");
    for e in &s.eintraege {
        let z = e.objekt.zeitpunkt();
        let (art, formular, antworten, praeparat) = match &e.objekt {
            Objekt::Status(st) => ("Status", st.formular, &st.antworten, None),
            Objekt::Ereignis(ev) => ("Ereignis", ev.formular, &ev.antworten, ev.praeparat),
        };
        let fname = s
            .formular(&formular)
            .map(|f| f.name.clone())
            .unwrap_or_default();
        let pname = praeparat
            .and_then(|p| s.bund.praeparate.iter().find(|x| x.id == p))
            .map(|p| p.handelsname.clone())
            .unwrap_or_default();
        let kopf = format!(
            "{};{};{};{};{};{}",
            zeit::iso_datum(e.objekt.lebenstag()),
            zeit::uhrzeit_text(z.uhrzeit_min()),
            csv(&z.zone),
            art,
            csv(&fname),
            csv(&pname)
        );
        if antworten.is_empty() {
            out.push_str(&format!("{kopf};;;\n"));
        }
        for a in antworten {
            let Some(md) = s.messdef(&a.messdef) else {
                continue;
            };
            let zustand = match a.zustand {
                Zustand::Beantwortet => "beantwortet",
                Zustand::Uebersprungen => "übersprungen",
                Zustand::KamNichtVor => "kam nicht vor",
                Zustand::NichtGezeigtBedingung => "nicht gezeigt",
                Zustand::Ausgeblendet => "ausgeblendet",
                Zustand::BlockZu => "Block zu",
            };
            let wert = match &a.wert {
                None => String::new(),
                Some(w) => {
                    let mut t = Vec::new();
                    if let Some(z) = w.zahl {
                        t.push(z.to_string());
                    }
                    if let Some(x) = &w.text {
                        t.push(x.clone());
                    }
                    for (o, st) in &w.auswahl {
                        let name = optionen(s, md)
                            .into_iter()
                            .find(|x| x.id == *o)
                            .map(|x| x.text)
                            .unwrap_or_else(|| o.to_string());
                        t.push(match st {
                            Some(st) => format!("{name} ({st})"),
                            None => name,
                        });
                    }
                    t.join(", ")
                }
            };
            out.push_str(&format!(
                "{kopf};{};{};{}\n",
                csv(&md.frage),
                zustand,
                csv(&wert)
            ));
        }
    }
    let _ = Bezugsart::Jetzt;
    Ok(out)
}
