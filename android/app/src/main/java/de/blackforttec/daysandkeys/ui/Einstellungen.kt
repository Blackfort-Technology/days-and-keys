// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.BuildConfigInfo
import de.blackforttec.daysandkeys.kern.Aktion
import de.blackforttec.daysandkeys.kern.DakFehler
import de.blackforttec.daysandkeys.kern.EinstellungenDaten
import de.blackforttec.daysandkeys.kern.ErinnerungDaten
import de.blackforttec.daysandkeys.kern.Vertrauensperson
import de.blackforttec.daysandkeys.plattform.Schluesselspeicher
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
    val ordnerWahl = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        if (uri != null) { z.app.ordner.uebernehmen(uri); z.sichernImHintergrund { z.meldung = it } }
    }
    val exportZiel = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("text/csv")) { uri: Uri? ->
        val text = csv; csv = null
        if (uri != null && text != null) {
            z.activity.contentResolver.openOutputStream(uri, "wt")?.use { it.write(text.toByteArray()) }
            z.meldung = "Export gespeichert"
        }
    }
    fun speichern() {
        val neu = e.copy(eigeneMarker = marker.filter { it.isNotBlank() }, vertrauenspersonen = personen.filter { it.name.isNotBlank() && it.telefon.isNotBlank() })
        if (z.versuch { z.kern.einstellungenSpeichern(neu) } != null) { e = neu; z.geaendert(); z.meldung = "Gespeichert" }
    }
    fun erinnerung(art: UByte) = e.erinnerungen.firstOrNull { it.art == art }
    fun erinnerungSetzen(art: UByte, r: ErinnerungDaten?) {
        e = e.copy(erinnerungen = e.erinnerungen.filter { it.art != art } + listOfNotNull(r))
    }

    Column(Modifier.verticalScroll(rememberScrollState())) {
        Abschnitt("Tag und Sperre") {
            Text("Der Tag beginnt um", style = MaterialTheme.typography.bodyMedium)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (h in listOf(0, 2, 3, 4, 5, 6)) FilterChip(selected = e.tageswechselMin == h * 60, onClick = { e = e.copy(tageswechselMin = h * 60) }, label = { Text("$h:00") })
            }
            Text("Sperren nach", style = MaterialTheme.typography.bodyMedium)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for ((s, t) in listOf(30u to "30 s", 60u to "1 min", 300u to "5 min")) FilterChip(selected = e.sperrzeitS == s, onClick = { e = e.copy(sperrzeitS = s) }, label = { Text(t) })
            }
        }
        Abschnitt("Erinnerungen", "Neutrale Texte, höchstens ${e.maxBenachrichtigungen} am Tag.") {
            for ((art, name, std) in listOf(Triple(2.toUByte(), "Tagesabschluss", 21 * 60), Triple(3.toUByte(), "Wochenrückblick (Sonntag)", 18 * 60))) {
                val r = erinnerung(art)
                Schalter(name + (r?.let { " um ${uhrzeit(it.uhrzeitMin)}" } ?: ""), r?.aktiv == true) { an ->
                    erinnerungSetzen(art, ErinnerungDaten(art, r?.uhrzeitMin ?: std, r?.wochentage ?: (if (art.toInt() == 3) 0x40u else 0x7fu), an))
                }
                if (r?.aktiv == true) FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    for (h in listOf(18, 19, 20, 21, 22)) FilterChip(selected = r.uhrzeitMin == h * 60, onClick = { erinnerungSetzen(art, r.copy(uhrzeitMin = h * 60)) }, label = { Text("$h:00") })
                }
            }
            val status = e.erinnerungen.filter { it.art.toInt() == 1 }
            Text("Status-Erinnerungen: " + status.joinToString(", ") { uhrzeit(it.uhrzeitMin) }.ifEmpty { "keine" })
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (h in listOf(9, 12, 15, 18)) {
                    val an = status.any { it.uhrzeitMin == h * 60 }
                    FilterChip(selected = an, onClick = {
                        e = e.copy(erinnerungen = if (an) e.erinnerungen.filterNot { it.art.toInt() == 1 && it.uhrzeitMin == h * 60 } else e.erinnerungen + ErinnerungDaten(1u, h * 60, 0x7fu, true))
                    }, label = { Text("$h:00") })
                }
            }
        }
        Abschnitt("Eigene Tagesmarker", "Bis zu drei Marker mit selbst gewählten Namen, z. B. für besondere Tage. Die Namen liegen nur verschlüsselt auf dem Gerät.") {
            for (i in 0 until 3) OutlinedTextField(marker[i], { marker[i] = it.take(40) }, label = { Text("Marker ${i + 1}") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        }
        Abschnitt("Mein Plan für Tiefs", "Frühwarnzeichen, was hilft, wen ich anrufe. Dieser Text ist über die Hilfe auch ohne Entsperren lesbar und liegt dafür unverschlüsselt auf dem Gerät. In die Sicherung kommt er verschlüsselt mit.") {
            OutlinedTextField(plan, { plan = it.take(8000) }, minLines = 4, modifier = Modifier.fillMaxWidth())
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick = { if (z.versuch { z.kern.krisenplanSetzen(plan.ifBlank { null }) } != null) z.meldung = "Plan gespeichert" }) { Text("Plan speichern") }
            }
            Text("Plan zeigen, wenn ich mehrere Tage in Folge eine niedrige Stimmung eintrage (selbst eingestellt, keine Diagnose):", style = MaterialTheme.typography.bodySmall)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                FilterChip(selected = e.krisenregelTage == null, onClick = { e = e.copy(krisenregelTage = null, krisenregelUnter = null) }, label = { Text("aus") })
                for (t in listOf(3u, 5u, 7u)) FilterChip(selected = e.krisenregelTage == t, onClick = { e = e.copy(krisenregelTage = t, krisenregelUnter = e.krisenregelUnter ?: -2) }, label = { Text("$t Tage") })
            }
            if (e.krisenregelTage != null) FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (u in listOf(-3L, -2L, -1L, 0L)) FilterChip(selected = e.krisenregelUnter == u, onClick = { e = e.copy(krisenregelUnter = u) }, label = { Text("unter ${if (u > 0) "+" else ""}$u") })
            }
        }
        Abschnitt("Vertrauenspersonen", "Erscheinen in der Hilfe, aber nur entsperrt. Die App benachrichtigt niemanden.") {
            for ((i, p) in personen.withIndex()) Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                OutlinedTextField(p.name, { personen[i] = p.copy(name = it.take(100)) }, label = { Text("Name") }, singleLine = true, modifier = Modifier.weight(1f))
                OutlinedTextField(p.telefon, { personen[i] = p.copy(telefon = it.take(40)) }, label = { Text("Telefon") }, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone), modifier = Modifier.width(140.dp))
            }
            if (personen.size < 10) TextButton(onClick = { personen.add(Vertrauensperson("", "")) }) { Text("Person hinzufügen") }
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) { Button(onClick = { speichern() }) { Text("Einstellungen speichern") } }
        Abschnitt("Sicherung", z.app.ordner.ordner?.let { "Ordner gewählt. " + (z.start?.letzteSicherungMs?.let { "Letzte Sicherung ${vorText(it)}." } ?: "") } ?: "Noch kein Ordner gewählt.") {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick = { ordnerWahl.launch(null) }) { Text("Ordner wählen") }
                Button(onClick = { z.sichernImHintergrund { z.meldung = it } }) { Text("Jetzt sichern") }
            }
        }
        Abschnitt("Alles exportieren", "Alle Einträge als Tabelle (CSV) im Klartext, zum Beispiel als Ausstieg oder für einen Arzttermin.") {
            OutlinedButton(onClick = {
                scope.launch {
                    try {
                        val b = z.kern.bestaetigungAnfordern(Aktion.EXPORT_CSV)
                        z.app.schluessel.bestaetigen(z.activity, b.titel, b.text)
                        csv = z.kern.exportCsv(b.digest)
                        exportZiel.launch("days-and-keys-export.csv")
                    } catch (x: Schluesselspeicher.Abgebrochen) { z.meldung = x.message } catch (x: DakFehler.Fehler) { z.meldung = x.meldung }
                }
            }) { Text("Export erstellen") }
        }
        Abschnitt("Recovery-Wörter prüfen", "Gib deine 24 Wörter ein, um zu prüfen, ob dein Zettel noch stimmt. Die Wörter werden nicht gespeichert.") {
            if (!probe) OutlinedButton(onClick = { probe = true }) { Text("Prüfen") } else WortProbe(z) { probe = false }
        }
        Abschnitt("Schlüssel erneuern", "Bei Verdacht, dass jemand Zugriff hatte. Neue Einträge und Sicherungen sind dann mit alten Schlüsseln nicht mehr lesbar.") {
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
                        z.sichernImHintergrund { z.meldung = "Schlüssel erneuert. $it" }
                    } catch (x: Schluesselspeicher.Abgebrochen) { z.kern.vorbereitetesVerwerfen(); z.meldung = x.message } catch (x: DakFehler.Fehler) { z.kern.vorbereitetesVerwerfen(); z.meldung = x.meldung } finally { z.arbeitet = false }
                }
            }) { Text("Schlüssel erneuern") }
        }
        Abschnitt("Über", "Days & Keys ${BuildConfigInfo.version} · Open Source (MPL-2.0) · https://github.com/Blackfort-Technology/days-and-keys") {
            Text("Keine Diagnosen, keine Empfehlungen, keine automatischen Zusammenhänge. Die App ersetzt keine ärztliche Beratung.", style = MaterialTheme.typography.bodySmall)
            Text("Schlüssel: " + if (z.app.schluessel.strongBox) "StrongBox" else "Keystore des Geräts", style = MaterialTheme.typography.bodySmall)
        }
        Abstand(24)
    }
}

@Composable
fun WortProbe(z: AppZustand, fertig: () -> Unit) {
    SensibleAnsicht()
    val w = remember { mutableStateListOf(*Array(24) { "" }) }
    Column {
        for (i in 0 until 24) WortFeld(z, i + 1, w[i]) { w[i] = it }
        Button(onClick = {
            z.meldung = if (z.versuch { z.kern.woerterPruefen(w.toList()) } == true) "Die Wörter stimmen." else "Die Wörter passen nicht."
            w.indices.forEach { w[it] = "" }
            fertig()
        }) { Text("Prüfen") }
    }
}
