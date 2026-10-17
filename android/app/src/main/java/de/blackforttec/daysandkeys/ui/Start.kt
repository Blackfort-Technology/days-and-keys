// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.Manifest
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import android.app.NotificationManager
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.AntwortEingabe
import de.blackforttec.daysandkeys.kern.AntwortZustand
import de.blackforttec.daysandkeys.kern.EinnahmeStand
import de.blackforttec.daysandkeys.kern.EreignisEingabe
import de.blackforttec.daysandkeys.kern.Knopf
import de.blackforttec.daysandkeys.kern.StatusEingabe

/** Heute (#134, Vorgabe 4.2): die Stimmung ist mit einem Tipp festgehalten, alles andere freiwillig. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun StartSeite(z: AppZustand) {
    LaunchedEffect(Unit) { z.neuLaden() }
    val s = z.start ?: return
    var bedarf by remember { mutableStateOf<Knopf?>(null) }
    var krise by remember { mutableStateOf<Knopf?>(null) }
    var alleEreignisse by remember { mutableStateOf(false) }
    val status = remember { z.versuch { z.kern.formular("status_jetzt") } }
    val stimmung = status?.gruppen?.firstOrNull()?.fragen?.firstOrNull()
    var benachrichtigungenErlaubt by remember { mutableStateOf(z.activity.getSystemService(NotificationManager::class.java).areNotificationsEnabled()) }
    val erlaubnis = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { ok ->
        benachrichtigungenErlaubt = ok || z.activity.getSystemService(NotificationManager::class.java).areNotificationsEnabled()
        if (benachrichtigungenErlaubt) z.planen()
    }
    var wert by z::schnellwert
    val prefs = remember { z.activity.getSharedPreferences("oberflaeche", android.content.Context.MODE_PRIVATE) }
    var sicherungSpaeterBis by remember { mutableLongStateOf(prefs.getLong("sicherung_spaeter_bis", 0L)) }
    val einnahmeFestgehalten = stringResource(R.string.start_einnahme_festgehalten)
    val vorPraefix = stringResource(R.string.baustein_vor_praefix)

    fun ereignis(k: Knopf) {
        // Ereignisse ohne Fragen werden mit dem Tipp gespeichert; „Krise“ fragt nach (Vorgabe 4.2).
        if (k.schluessel == "krise") { krise = k; return }
        val f = z.versuch { z.kern.formular(k.schluessel) } ?: return
        if (f.gruppen.all { it.fragen.isEmpty() } && !f.brauchtPraeparat) {
            z.versuch { z.kern.ereignisErfassen(EreignisEingabe(f.id, null, emptyList(), null, null)) } ?: return
            z.geaendert(); z.meldung = z.activity.getString(R.string.start_ereignis_festgehalten, k.name, uhrzeitAus(System.currentTimeMillis()))
        } else z.zu(Seite.Formular(k.schluessel))
    }

    Column(Modifier.verticalScroll(rememberScrollState()).padding(bottom = Abstand.xl)) {
        // Schnellstatus: die erste Frage von „Status“ aus der Vorlage, keine eigene Zeichenkette.
        if (status != null && stimmung != null) {
            Abschnitt(stimmung.frage) {
                Stufenleiste(stimmung, wert) { wert = it }
                if (wert != null) {
                    val antwort = AntwortEingabe(stimmung.messdefId, AntwortZustand.BEANTWORTET, wert, null, emptyList(), true)
                    Button(onClick = {
                        z.versuch { z.kern.statusSpeichern(StatusEingabe(status.id, null, listOf(antwort), false, null, null)) } ?: return@Button
                        wert = null; z.geaendert(); z.meldung = z.activity.getString(R.string.allgemein_festgehalten_um, uhrzeitAus(System.currentTimeMillis()))
                    }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.hauptknopf)) { Text(stringResource(R.string.allgemein_festhalten)) }
                    TextButton(onClick = { z.zu(Seite.Formular("status_jetzt", vorbelegt = listOf(antwort))); wert = null }) { Text(stringResource(R.string.start_mehr_angeben)) }
                } else {
                    if (s.statusHeute > 0u) s.letzterStatusMs?.let { Text(stringResource(R.string.start_zuletzt_um, uhrzeitAus(it)), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                        TextButton(onClick = { z.zu(Seite.Formular("status_jetzt")) }) { Text(stringResource(R.string.start_ausfuehrlich)) }
                        if (s.tagesabschlussHeute) TextButton(onClick = { z.zu(Seite.Formular("tagesabschluss")) }) { Text(stringResource(R.string.allgemein_tagesabschluss)) }
                        if (s.wochenrueckblickHeute) TextButton(onClick = { z.zu(Seite.Formular("wochenrueckblick")) }) { Text(stringResource(R.string.start_wochenrueckblick)) }
                    }
                }
            }
        }
        // Ohne Erlaubnis kommen Erinnerungen nicht an, etwa nach einer Wiederherstellung (#149).
        if (!benachrichtigungenErlaubt && z.versuch { z.kern.einstellungen() }?.erinnerungen?.any { it.aktiv } != false) {
            Abschnitt(stringResource(R.string.start_erinnerungen_aus_titel), stringResource(R.string.start_erinnerungen_aus_text)) {
                FilledTonalButton(onClick = { erlaubnis.launch(Manifest.permission.POST_NOTIFICATIONS) }) { Text(stringResource(R.string.start_erlauben)) }
            }
        }
        // Morgens bis 14 Uhr, solange heute noch nichts zum Schlaf erfasst ist; der Kern entscheidet (#139).
        if (s.schlafAnbieten) {
            Abschnitt(stringResource(R.string.start_schlaf_titel), stringResource(R.string.start_schlaf_text)) {
                FilledTonalButton(onClick = { z.zu(Seite.Formular("aufgewacht")) }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.ziel)) { Text(stringResource(R.string.start_schlaf_knopf)) }
            }
        }
        if (s.krisenplanHinweis && !z.krisenhinweisGesehen) {
            Abschnitt(stringResource(R.string.start_plan_titel), stringResource(R.string.start_plan_text)) {
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    FilledTonalButton(onClick = { z.zu(Seite.Hilfe) }) { Text(stringResource(R.string.start_plan_ansehen)) }
                    TextButton(onClick = { z.krisenhinweisGesehen = true }) { Text(stringResource(R.string.start_ausblenden)) }
                }
            }
        }
        for (e in s.entwuerfe) {
            Abschnitt(stringResource(R.string.start_entwurf_titel, e.formularName), pluralStringResource(R.plurals.start_entwurf_zeile, e.antworten.toInt(), vorText(e.erstelltMs), e.antworten.toInt())) {
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    FilledTonalButton(onClick = {
                        val f = z.versuch { z.kern.entwurfLaden(e.id) } ?: return@FilledTonalButton
                        val schl = formularSchluessel(z, f.formularId) ?: return@FilledTonalButton
                        z.zu(Seite.Formular(schl, entwurf = e.id))
                    }) { Text(stringResource(R.string.allgemein_weiter)) }
                    TextButton(onClick = { z.versuch { z.kern.entwurfVerwerfen(e.id) }; z.neuLaden() }) { Text(stringResource(R.string.allgemein_verwerfen)) }
                }
            }
        }
        if (s.einnahmenHeute.isNotEmpty() || s.bedarf.isNotEmpty()) {
            val nochNichts = s.einnahmenHeute.none { it.stand != EinnahmeStand.OFFEN }
            Abschnitt(stringResource(R.string.start_medikamente_titel), if (nochNichts) stringResource(R.string.start_medikamente_hinweis) else null) {
                for (e in s.einnahmenHeute) {
                    Row(Modifier.fillMaxWidth().padding(vertical = Abstand.xs), verticalAlignment = Alignment.Top) {
                        Text(uhrzeit(e.uhrzeitMin), style = MaterialTheme.typography.labelLarge, modifier = Modifier.width(56.dp).padding(top = 2.dp))
                        Column(Modifier.weight(1f)) {
                            Text(e.name + (e.staerke?.let { " $it" } ?: ""), style = MaterialTheme.typography.bodyLarge)
                            when (e.stand) {
                                EinnahmeStand.OFFEN -> {
                                    Text(stringResource(R.string.start_dosis, e.dosis), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s), modifier = Modifier.padding(top = Abstand.xs)) {
                                        FilledTonalButton(onClick = {
                                            z.versuch { z.kern.einnahmeBestaetigen(e.praeparatId, null) }; z.geaendert(); z.meldung = einnahmeFestgehalten
                                        }) { Text(stringResource(R.string.start_genommen_knopf)) }
                                        TextButton(onClick = { z.zu(Seite.Formular("einnahme_ausgelassen", praeparat = e.praeparatId)) }) { Text(stringResource(R.string.start_ausgelassen_knopf), maxLines = 1) }
                                    }
                                }
                                // Erledigt ohne Farbe für den Stand (AE-27).
                                EinnahmeStand.GENOMMEN -> Row(verticalAlignment = Alignment.CenterVertically) {
                                    Symbol(R.drawable.sym_check, null, Modifier.size(18.dp)); Text(" " + stringResource(R.string.start_genommen_stand), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                EinnahmeStand.AUSGELASSEN -> Text(stringResource(R.string.start_ausgelassen_stand), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                }
                if (s.bedarf.isNotEmpty()) {
                    Text(stringResource(R.string.allgemein_bei_bedarf), style = MaterialTheme.typography.titleSmall)
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                        for (b in s.bedarf) OutlinedButton(onClick = { bedarf = b }) { Text(b.name) }
                    }
                }
            }
        }
        Abschnitt(stringResource(R.string.start_ereignis_titel), stringResource(R.string.start_ereignis_text)) {
            val zeige = if (alleEreignisse) s.ereignisse else s.ereignisse.take(6)
            for (paar in zeige.chunked(2)) Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                for (k in paar) OutlinedButton(onClick = { ereignis(k) }, modifier = Modifier.weight(1f).heightIn(min = Abstand.ziel)) {
                    Text(k.name, style = MaterialTheme.typography.labelMedium, maxLines = 2)
                }
                if (paar.size == 1) Column(Modifier.weight(1f)) {}
            }
            if (s.ereignisse.size > 6) TextButton(onClick = { alleEreignisse = !alleEreignisse }) { Text(if (alleEreignisse) stringResource(R.string.start_weniger_zeigen) else stringResource(R.string.start_alle_ereignisse)) }
        }
        // Fehlt die Sicherung ganz, lässt sich der Hinweis eine Woche aufschieben (#165).
        if (s.sicherungWarnung && (s.letzteSicherungMs != null || System.currentTimeMillis() >= sicherungSpaeterBis)) {
            Abschnitt(
                stringResource(R.string.allgemein_sicherung),
                s.letzteSicherungMs?.let { stringResource(R.string.start_sicherung_alt, vorText(it).removePrefix(vorPraefix)) }
                    ?: stringResource(R.string.start_sicherung_fehlt),
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    FilledTonalButton(onClick = { z.mehrTeil = "sicherung"; z.zu(Seite.Einstellungen) }) { Text(stringResource(R.string.start_sicherung_einrichten)) }
                    if (s.letzteSicherungMs == null) TextButton(onClick = {
                        sicherungSpaeterBis = System.currentTimeMillis() + 7L * 24 * 3600 * 1000
                        prefs.edit().putLong("sicherung_spaeter_bis", sicherungSpaeterBis).apply()
                    }) { Text(stringResource(R.string.einrichtung_spaeter)) }
                }
            }
        } else s.letzteSicherungMs?.let {
            Text(stringResource(R.string.start_gesichert, vorText(it)), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(horizontal = Abstand.rand, vertical = Abstand.s))
        }
    }
    bedarf?.let { b ->
        AlertDialog(
            onDismissRequest = { bedarf = null },
            title = { Text(b.name) },
            text = { Text(stringResource(R.string.start_bedarf_frage)) },
            confirmButton = { Button(onClick = { z.versuch { z.kern.einnahmeBestaetigen(b.formularId, null) }; bedarf = null; z.geaendert() }) { Text(stringResource(R.string.start_bedarf_ja)) } },
            dismissButton = { TextButton(onClick = { bedarf = null }) { Text(stringResource(R.string.allgemein_abbrechen)) } },
        )
    }
    krise?.let { k ->
        AlertDialog(
            onDismissRequest = { krise = null },
            title = { Text(stringResource(R.string.start_krise_titel)) },
            text = { Text(stringResource(R.string.allgemein_krise_hinweis)) },
            confirmButton = {
                Button(onClick = {
                    val f = z.versuch { z.kern.formular(k.schluessel) }
                    if (f != null && z.versuch { z.kern.ereignisErfassen(EreignisEingabe(f.id, null, emptyList(), null, null)) } != null) z.geaendert()
                    krise = null; z.zu(Seite.Hilfe)
                }) { Text(stringResource(R.string.start_krise_knopf)) }
            },
            dismissButton = { TextButton(onClick = { krise = null }) { Text(stringResource(R.string.allgemein_abbrechen)) } },
        )
    }
}

/** Schlüssel eines Formulars aus seiner ID (für Entwürfe). */
fun formularSchluessel(z: AppZustand, formularId: String): String? =
    listOf("status_jetzt", "tagesabschluss", "wochenrueckblick").firstOrNull { z.versuch { z.kern.formular(it) }?.id == formularId }
