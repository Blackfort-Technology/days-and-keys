// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.BuildConfigInfo
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.Aktion
import de.blackforttec.daysandkeys.kern.DakFehler
import de.blackforttec.daysandkeys.kern.EinstellungenDaten
import de.blackforttec.daysandkeys.kern.ErinnerungDaten
import de.blackforttec.daysandkeys.kern.Vertrauensperson
import de.blackforttec.daysandkeys.plattform.Schluesselspeicher
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun EinstellungenSeite(z: AppZustand) {
    val e0: EinstellungenDaten = remember { z.versuch { z.kern.einstellungen() } } ?: return
    var e by remember { mutableStateOf(e0) }
    val marker = remember { mutableStateListOf(*(e0.eigeneMarker + List(3 - e0.eigeneMarker.size.coerceAtMost(3)) { "" }).take(3).toTypedArray()) }
    val personen = remember { mutableStateListOf(*e0.vertrauenspersonen.toTypedArray()) }
    var plan by remember { mutableStateOf(z.versuch { z.kern.hilfe() }?.krisenplan ?: "") }
    var csv by remember { mutableStateOf<String?>(null) }
    var probe by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val gespeichert = stringResource(R.string.allgemein_gespeichert)
    val exportGespeichert = stringResource(R.string.mehr_export_gespeichert)
    // Der Ordner ist kein Compose-Zustand; ohne diese Spiegelung stand nach der Wahl weiter „kein Ordner“.
    var ordnerDa by remember { mutableStateOf(z.app.ordner.ordner != null) }
    val ordnerWahl = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        if (uri != null) { z.app.ordner.uebernehmen(uri); ordnerDa = true; z.sichernImHintergrund { z.meldung = it } }
    }
    val exportZiel = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("text/csv")) { uri: Uri? ->
        val text = csv; csv = null
        if (uri != null && text != null) {
            z.activity.contentResolver.openOutputStream(uri, "wt")?.use { it.write(text.toByteArray()) }
            z.meldung = exportGespeichert
        }
    }
    fun speichern() {
        val neu = e.copy(eigeneMarker = marker.filter { it.isNotBlank() }, vertrauenspersonen = personen.filter { it.name.isNotBlank() && it.telefon.isNotBlank() })
        if (z.versuch { z.kern.einstellungenSpeichern(neu) } != null) { e = neu; z.geaendert(); z.meldung = gespeichert }
    }
    fun erinnerung(art: UByte) = e.erinnerungen.firstOrNull { it.art == art }
    fun erinnerungSetzen(art: UByte, r: ErinnerungDaten?) {
        e = e.copy(erinnerungen = e.erinnerungen.filter { it.art != art } + listOfNotNull(r))
    }

    // Übersicht mit Unterseiten, Änderungen gelten sofort (#136, Vorgabe 4.9).
    var teil by remember { mutableStateOf<String?>(z.mehrTeil.also { z.mehrTeil = null }) }
    // Ohne Recovery-Wörter eingerichtet: Sicherung beginnt mit den Wörtern (#165).
    var woerterOffen by remember { mutableStateOf(z.versuch { z.kern.woerterOffenSeit() != null } == true) }
    var woerterJetzt by remember { mutableStateOf(false) }
    LaunchedEffect(teil) { if (teil != "sicherung") woerterJetzt = false }
    BackHandler(enabled = teil != null) { teil = null }
    fun sofort(neu: EinstellungenDaten) { e = neu; speichern() }
    LaunchedEffect(marker.toList(), personen.toList()) {
        if (marker.toList() == (e.eigeneMarker + List(3) { "" }).take(3) && personen.toList() == e.vertrauenspersonen) return@LaunchedEffect
        delay(600); speichern()
    }
    var planGespeichert by remember { mutableStateOf(plan) }
    LaunchedEffect(plan) {
        if (plan == planGespeichert) return@LaunchedEffect
        delay(600)
        if (z.versuch { z.kern.krisenplanSetzen(plan.ifBlank { null }) } != null) { planGespeichert = plan; z.meldung = gespeichert }
    }
    val sicherungText = if (woerterOffen) stringResource(R.string.mehr_sicherung_offen) else if (ordnerDa) z.start?.letzteSicherungMs?.let { stringResource(R.string.mehr_letzte_sicherung, vorText(it)) } ?: stringResource(R.string.mehr_ordner_gewaehlt) else stringResource(R.string.mehr_kein_ordner)
    val regelText = e.krisenregelTage?.let { stringResource(R.string.mehr_regel, it.toInt(), e.krisenregelUnter ?: -2L) } ?: stringResource(R.string.mehr_aus)

    if (teil == null) {
        Column(Modifier.verticalScroll(rememberScrollState())) {
            Text(stringResource(R.string.mehr_schwere_momente), style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile(stringResource(R.string.allgemein_plan_tiefs), if (plan.isBlank()) stringResource(R.string.mehr_plan_leer) else plan.lineSequence().first().take(60)) { teil = "plan" }
            Klickzeile(stringResource(R.string.mehr_plan_zeigen), regelText) { teil = "regel" }
            Klickzeile(stringResource(R.string.mehr_vertrauenspersonen), if (personen.isEmpty()) stringResource(R.string.mehr_keine) else personen.joinToString(", ") { it.name }) { teil = "personen" }
            Text(stringResource(R.string.mehr_alltag), style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            val keine = stringResource(R.string.mehr_keine)
            Klickzeile(stringResource(R.string.allgemein_erinnerungen), e.erinnerungen.filter { it.aktiv }.joinToString(", ") { uhrzeit(it.uhrzeitMin) }.ifEmpty { keine }) { teil = "erinnerungen" }
            val sperrzeit = if (e.sperrzeitS < 60u) stringResource(R.string.mehr_sekunden, e.sperrzeitS.toInt()) else stringResource(R.string.mehr_minuten, (e.sperrzeitS / 60u).toInt())
            Klickzeile(stringResource(R.string.mehr_tag_sperre), stringResource(R.string.mehr_tag_sperre_zeile, e.tageswechselMin / 60, sperrzeit)) { teil = "tag" }
            Klickzeile(stringResource(R.string.mehr_tagesmarker), marker.filter { it.isNotBlank() }.joinToString(", ").ifEmpty { keine }) { teil = "marker" }
            Text(stringResource(R.string.mehr_daten), style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile(stringResource(R.string.allgemein_sicherung), sicherungText) { teil = "sicherung" }
            if (!woerterOffen) Klickzeile(stringResource(R.string.mehr_woerter_pruefen), stringResource(R.string.mehr_woerter_pruefen_zeile)) { teil = "woerter" }
            Klickzeile(stringResource(R.string.mehr_schluessel_erneuern), stringResource(R.string.mehr_schluessel_erneuern_zeile)) { teil = "schluessel" }
            Klickzeile(stringResource(R.string.mehr_export), stringResource(R.string.mehr_export_zeile)) { teil = "export" }
            Text(stringResource(R.string.mehr_ueber), style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile(stringResource(R.string.allgemein_app_name), stringResource(R.string.mehr_version, BuildConfigInfo.version)) { teil = "ueber" }
            Abstand(24)
        }
        return
    }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            IconButton(onClick = { teil = null }) { Symbol(R.drawable.sym_arrow_back, stringResource(R.string.mehr_zurueck)) }
            Text(when (teil) {
                "plan" -> stringResource(R.string.allgemein_plan_tiefs); "regel" -> stringResource(R.string.mehr_plan_zeigen); "personen" -> stringResource(R.string.mehr_vertrauenspersonen)
                "erinnerungen" -> stringResource(R.string.allgemein_erinnerungen); "tag" -> stringResource(R.string.mehr_tag_sperre); "marker" -> stringResource(R.string.mehr_tagesmarker)
                "sicherung" -> stringResource(R.string.allgemein_sicherung); "woerter" -> stringResource(R.string.mehr_woerter_pruefen); "schluessel" -> stringResource(R.string.mehr_schluessel_erneuern)
                "export" -> stringResource(R.string.mehr_export); else -> stringResource(R.string.allgemein_app_name)
            }, style = MaterialTheme.typography.titleLarge)
        }
        when (teil) {
            "plan" -> {
                Hinweisfeld(stringResource(R.string.mehr_plan_hinweis))
                Text(stringResource(R.string.mehr_plan_zusatz), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                OutlinedTextField(plan, { plan = it.take(8000) }, minLines = 8, modifier = Modifier.fillMaxWidth())
            }
            "regel" -> {
                Text(stringResource(R.string.mehr_regel_text), style = MaterialTheme.typography.bodyMedium)
                Stufenwahl(listOf(0L to stringResource(R.string.mehr_aus), 3L to stringResource(R.string.allgemein_tage, 3), 5L to stringResource(R.string.allgemein_tage, 5), 7L to stringResource(R.string.allgemein_tage, 7)), e.krisenregelTage?.toLong() ?: 0L) { w ->
                    sofort(if (w == null || w == 0L) e.copy(krisenregelTage = null, krisenregelUnter = null) else e.copy(krisenregelTage = w.toUInt(), krisenregelUnter = e.krisenregelUnter ?: -2))
                }
                if (e.krisenregelTage != null) {
                    Text(stringResource(R.string.mehr_unter), style = MaterialTheme.typography.titleSmall)
                    Stufenwahl(listOf(-3L to "-3", -2L to "-2", -1L to "-1", 0L to "0"), e.krisenregelUnter) { w -> w?.let { sofort(e.copy(krisenregelUnter = it)) } }
                }
            }
            "personen" -> {
                Text(stringResource(R.string.mehr_personen_text), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                for ((i, p) in personen.withIndex()) Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    OutlinedTextField(p.name, { personen[i] = p.copy(name = it.take(100)) }, label = { Text(stringResource(R.string.allgemein_name)) }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(p.telefon, { personen[i] = p.copy(telefon = it.take(40)) }, label = { Text(stringResource(R.string.mehr_telefon)) }, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone), modifier = Modifier.weight(1f))
                }
                if (personen.size < 10) TextButton(onClick = { personen.add(Vertrauensperson("", "")) }) { Text(stringResource(R.string.mehr_person_hinzufuegen)) }
            }
            "erinnerungen" -> {
                Text(stringResource(R.string.mehr_erinnerungen_text, e.maxBenachrichtigungen.toInt()), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                for ((art, name, std) in listOf(Triple(2.toUByte(), stringResource(R.string.allgemein_tagesabschluss), 21 * 60), Triple(3.toUByte(), stringResource(R.string.mehr_wochenrueckblick_sonntag), 18 * 60))) {
                    val r = erinnerung(art)
                    SchalterZeile(name, r?.takeIf { it.aktiv }?.let { stringResource(R.string.mehr_um, uhrzeit(it.uhrzeitMin)) }, r?.aktiv == true) { an ->
                        erinnerungSetzen(art, ErinnerungDaten(art, r?.uhrzeitMin ?: std, r?.wochentage ?: (if (art.toInt() == 3) 0x40u else 0x7fu), an)); speichern()
                    }
                    if (r?.aktiv == true) Stufenwahl(listOf(18, 19, 20, 21, 22).map { it * 60L to "$it" }, r.uhrzeitMin.toLong()) { w ->
                        w?.let { erinnerungSetzen(art, r.copy(uhrzeitMin = it.toInt())); speichern() }
                    }
                }
                Text(stringResource(R.string.mehr_status_erinnerungen), style = MaterialTheme.typography.titleSmall)
                val status = e.erinnerungen.filter { it.art.toInt() == 1 }
                FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    for (h in listOf(9, 12, 15, 18)) {
                        val an = status.any { it.uhrzeitMin == h * 60 }
                        AuswahlChip("$h:00", an) {
                            sofort(e.copy(erinnerungen = if (an) e.erinnerungen.filterNot { it.art.toInt() == 1 && it.uhrzeitMin == h * 60 } else e.erinnerungen + ErinnerungDaten(1u, h * 60, 0x7fu, true)))
                        }
                    }
                }
            }
            "tag" -> {
                Text(stringResource(R.string.mehr_tag_beginnt), style = MaterialTheme.typography.titleSmall)
                Stufenwahl(listOf(0, 2, 3, 4, 5, 6).map { it * 60L to "$it:00" }, e.tageswechselMin.toLong()) { w -> w?.let { sofort(e.copy(tageswechselMin = it.toInt())) } }
                Text(stringResource(R.string.mehr_sperren_nach), style = MaterialTheme.typography.titleSmall)
                Stufenwahl(listOf(30L to stringResource(R.string.mehr_sekunden, 30), 60L to stringResource(R.string.mehr_minuten, 1), 300L to stringResource(R.string.mehr_minuten, 5)), e.sperrzeitS.toLong()) { w -> w?.let { sofort(e.copy(sperrzeitS = it.toUInt())) } }
            }
            "marker" -> {
                Text(stringResource(R.string.mehr_marker_text), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                for (i in 0 until 3) OutlinedTextField(marker[i], { marker[i] = it.take(40) }, label = { Text(stringResource(R.string.mehr_marker_feld, i + 1)) }, singleLine = true, modifier = Modifier.fillMaxWidth())
            }
            "sicherung" -> if (woerterOffen) {
                if (woerterJetzt) {
                    WoerterEinrichten(z, { titel, text, inhalt -> Kopf(titel, text); inhalt() }) {
                        woerterOffen = false; woerterJetzt = false; ordnerWahl.launch(null)
                    }
                } else {
                    Text(stringResource(R.string.mehr_sicherung_offen_text), style = MaterialTheme.typography.bodyLarge)
                    Button(onClick = { woerterJetzt = true }) { Text(stringResource(R.string.start_sicherung_einrichten)) }
                }
            } else {
                Text(sicherungText, style = MaterialTheme.typography.bodyLarge)
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    Button(onClick = { z.sichernImHintergrund { z.meldung = it } }) { Text(stringResource(R.string.mehr_jetzt_sichern)) }
                    OutlinedButton(onClick = { ordnerWahl.launch(null) }) { Text(stringResource(R.string.mehr_ordner_aendern)) }
                }
            }
            "woerter" -> {
                Text(stringResource(R.string.mehr_woerter_text), style = MaterialTheme.typography.bodyMedium)
                WortProbe(z) { teil = null }
            }
            "schluessel" -> {
                Text(stringResource(R.string.mehr_schluessel_text), style = MaterialTheme.typography.bodyMedium)
                OutlinedButton(onClick = {
                    scope.launch {
                        try {
                            z.arbeitet = true
                            val b = z.kern.bestaetigungAnfordern(Aktion.ROTATION_AUF_WUNSCH)
                            z.app.schluessel.bestaetigen(z.activity, b.titel, b.text)
                            val s = z.kern.rotationBeginnen(b.digest)
                            val umh = z.app.schluessel.umhuellen(z.activity, s.generation, s.kk, s.kkH)
                            z.kern.vorbereitetesAbschliessen(umh, null)
                            z.app.schluessel.aufraeumen(s.generation)
                            z.geaendert()
                            z.sichernImHintergrund { z.meldung = z.activity.getString(R.string.mehr_schluessel_erneuert, it) }
                        } catch (x: Schluesselspeicher.Abgebrochen) { z.kern.vorbereitetesVerwerfen(); z.meldung = x.message } catch (x: DakFehler.Fehler) { z.kern.vorbereitetesVerwerfen(); z.meldung = x.meldung } finally { z.arbeitet = false }
                    }
                }) { Text(stringResource(R.string.mehr_schluessel_erneuern)) }
            }
            "export" -> {
                Text(stringResource(R.string.mehr_export_text), style = MaterialTheme.typography.bodyMedium)
                OutlinedButton(onClick = {
                    scope.launch {
                        try {
                            val b = z.kern.bestaetigungAnfordern(Aktion.EXPORT_CSV)
                            z.app.schluessel.bestaetigen(z.activity, b.titel, b.text)
                            csv = z.kern.exportCsv(b.digest)
                            exportZiel.launch("days-and-keys-export.csv")
                        } catch (x: Schluesselspeicher.Abgebrochen) { z.meldung = x.message } catch (x: DakFehler.Fehler) { z.meldung = x.meldung }
                    }
                }) { Text(stringResource(R.string.mehr_export_erstellen)) }
            }
            else -> {
                Text(stringResource(R.string.mehr_version_lizenz, BuildConfigInfo.version), style = MaterialTheme.typography.bodyLarge)
                Text(stringResource(R.string.mehr_quelltext), style = MaterialTheme.typography.bodyMedium)
                Text(stringResource(R.string.mehr_keine_diagnosen), style = MaterialTheme.typography.bodyMedium)
                Text(if (z.app.schluessel.strongBox) stringResource(R.string.mehr_schluessel_strongbox) else stringResource(R.string.mehr_schluessel_keystore), style = MaterialTheme.typography.bodyMedium)
                Text(stringResource(R.string.mehr_schrift), style = MaterialTheme.typography.bodyMedium)
            }
        }
        Abstand(24)
    }
}

/** Hinweisfeld in der ruhigen Hinweisfarbe (Vorgabe 3.1, tertiaryContainer). */
@Composable
fun Hinweisfeld(text: String) {
    Text(
        text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onTertiaryContainer,
        modifier = Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.tertiaryContainer, MaterialTheme.shapes.medium).padding(Abstand.l),
    )
}

@Composable
fun WortProbe(z: AppZustand, fertig: () -> Unit) {
    SensibleAnsicht()
    val w = remember { mutableStateListOf(*Array(24) { "" }) }
    val stimmen = stringResource(R.string.mehr_woerter_stimmen)
    val passenNicht = stringResource(R.string.mehr_woerter_passen_nicht)
    Column {
        for (i in 0 until 24) WortFeld(z, i + 1, w[i]) { w[i] = it }
        Button(onClick = {
            z.meldung = if (z.versuch { z.kern.woerterPruefen(w.toList()) } == true) stimmen else passenNicht
            w.indices.forEach { w[it] = "" }
            fertig()
        }) { Text(stringResource(R.string.allgemein_pruefen)) }
    }
}
