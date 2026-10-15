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

    // Übersicht mit Unterseiten, Änderungen gelten sofort (#136, Vorgabe 4.9).
    var teil by remember { mutableStateOf<String?>(null) }
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
        if (z.versuch { z.kern.krisenplanSetzen(plan.ifBlank { null }) } != null) { planGespeichert = plan; z.meldung = "Gespeichert" }
    }
    val sicherungText = z.app.ordner.ordner?.let { z.start?.letzteSicherungMs?.let { "Letzte Sicherung ${vorText(it)}" } ?: "Ordner gewählt" } ?: "Noch kein Ordner gewählt"
    val regelText = e.krisenregelTage?.let { "$it Tage unter ${e.krisenregelUnter ?: -2}" } ?: "aus"

    if (teil == null) {
        Column(Modifier.verticalScroll(rememberScrollState())) {
            Text("Für schwere Momente", style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile("Mein Plan für Tiefs", if (plan.isBlank()) "noch nicht geschrieben" else plan.lineSequence().first().take(60)) { teil = "plan" }
            Klickzeile("Plan automatisch zeigen", regelText) { teil = "regel" }
            Klickzeile("Vertrauenspersonen", if (personen.isEmpty()) "keine" else personen.joinToString(", ") { it.name }) { teil = "personen" }
            Text("Alltag", style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile("Erinnerungen", e.erinnerungen.filter { it.aktiv }.joinToString(", ") { uhrzeit(it.uhrzeitMin) }.ifEmpty { "keine" }) { teil = "erinnerungen" }
            Klickzeile("Tag und Sperre", "Tag beginnt um ${e.tageswechselMin / 60}:00 · sperren nach ${if (e.sperrzeitS < 60u) "${e.sperrzeitS} s" else "${e.sperrzeitS / 60u} min"}") { teil = "tag" }
            Klickzeile("Eigene Tagesmarker", marker.filter { it.isNotBlank() }.joinToString(", ").ifEmpty { "keine" }) { teil = "marker" }
            Text("Daten", style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile("Sicherung", sicherungText) { teil = "sicherung" }
            Klickzeile("Recovery-Wörter prüfen", "ob dein Zettel noch stimmt") { teil = "woerter" }
            Klickzeile("Schlüssel erneuern", "bei Verdacht, dass jemand Zugriff hatte") { teil = "schluessel" }
            Klickzeile("Alles exportieren", "Tabelle im Klartext") { teil = "export" }
            Text("Über", style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(start = Abstand.rand, top = Abstand.l, bottom = Abstand.xs))
            Klickzeile("Days & Keys", "Version ${BuildConfigInfo.version}") { teil = "ueber" }
            Abstand(24)
        }
        return
    }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            IconButton(onClick = { teil = null }) { Symbol(R.drawable.sym_arrow_back, "Zurück zu Mehr") }
            Text(when (teil) {
                "plan" -> "Mein Plan für Tiefs"; "regel" -> "Plan automatisch zeigen"; "personen" -> "Vertrauenspersonen"
                "erinnerungen" -> "Erinnerungen"; "tag" -> "Tag und Sperre"; "marker" -> "Eigene Tagesmarker"
                "sicherung" -> "Sicherung"; "woerter" -> "Recovery-Wörter prüfen"; "schluessel" -> "Schlüssel erneuern"
                "export" -> "Alles exportieren"; else -> "Days & Keys"
            }, style = MaterialTheme.typography.titleLarge)
        }
        when (teil) {
            "plan" -> {
                Hinweisfeld("Dieser Text ist über die Hilfe auch gesperrt lesbar und liegt dafür unverschlüsselt auf dem Gerät. In die Sicherung kommt er verschlüsselt mit.")
                Text("Frühwarnzeichen, was hilft, wen ich anrufe.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                OutlinedTextField(plan, { plan = it.take(8000) }, minLines = 8, modifier = Modifier.fillMaxWidth())
            }
            "regel" -> {
                Text("Die App zeigt deinen Plan, wenn du mehrere Tage in Folge eine niedrige Stimmung einträgst. Selbst eingestellt, keine Diagnose.", style = MaterialTheme.typography.bodyMedium)
                Stufenwahl(listOf(0L to "aus", 3L to "3 Tage", 5L to "5 Tage", 7L to "7 Tage"), e.krisenregelTage?.toLong() ?: 0L) { w ->
                    sofort(if (w == null || w == 0L) e.copy(krisenregelTage = null, krisenregelUnter = null) else e.copy(krisenregelTage = w.toUInt(), krisenregelUnter = e.krisenregelUnter ?: -2))
                }
                if (e.krisenregelTage != null) {
                    Text("Unter", style = MaterialTheme.typography.titleSmall)
                    Stufenwahl(listOf(-3L to "-3", -2L to "-2", -1L to "-1", 0L to "0"), e.krisenregelUnter) { w -> w?.let { sofort(e.copy(krisenregelUnter = it)) } }
                }
            }
            "personen" -> {
                Text("Erscheinen in der Hilfe, aber nur entsperrt. Die App benachrichtigt niemanden.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                for ((i, p) in personen.withIndex()) Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    OutlinedTextField(p.name, { personen[i] = p.copy(name = it.take(100)) }, label = { Text("Name") }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(p.telefon, { personen[i] = p.copy(telefon = it.take(40)) }, label = { Text("Telefon") }, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone), modifier = Modifier.weight(1f))
                }
                if (personen.size < 10) TextButton(onClick = { personen.add(Vertrauensperson("", "")) }) { Text("Person hinzufügen") }
            }
            "erinnerungen" -> {
                Text("Neutrale Texte, höchstens ${e.maxBenachrichtigungen} am Tag.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                for ((art, name, std) in listOf(Triple(2.toUByte(), "Tagesabschluss", 21 * 60), Triple(3.toUByte(), "Wochenrückblick (Sonntag)", 18 * 60))) {
                    val r = erinnerung(art)
                    SchalterZeile(name, r?.takeIf { it.aktiv }?.let { "um ${uhrzeit(it.uhrzeitMin)}" }, r?.aktiv == true) { an ->
                        erinnerungSetzen(art, ErinnerungDaten(art, r?.uhrzeitMin ?: std, r?.wochentage ?: (if (art.toInt() == 3) 0x40u else 0x7fu), an)); speichern()
                    }
                    if (r?.aktiv == true) Stufenwahl(listOf(18, 19, 20, 21, 22).map { it * 60L to "$it" }, r.uhrzeitMin.toLong()) { w ->
                        w?.let { erinnerungSetzen(art, r.copy(uhrzeitMin = it.toInt())); speichern() }
                    }
                }
                Text("Status-Erinnerungen", style = MaterialTheme.typography.titleSmall)
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
                Text("Der Tag beginnt um", style = MaterialTheme.typography.titleSmall)
                Stufenwahl(listOf(0, 2, 3, 4, 5, 6).map { it * 60L to "$it:00" }, e.tageswechselMin.toLong()) { w -> w?.let { sofort(e.copy(tageswechselMin = it.toInt())) } }
                Text("Sperren nach", style = MaterialTheme.typography.titleSmall)
                Stufenwahl(listOf(30L to "30 s", 60L to "1 min", 300L to "5 min"), e.sperrzeitS.toLong()) { w -> w?.let { sofort(e.copy(sperrzeitS = it.toUInt())) } }
            }
            "marker" -> {
                Text("Bis zu drei Marker mit selbst gewählten Namen, zum Beispiel für besondere Tage. Die Namen liegen nur verschlüsselt auf dem Gerät.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                for (i in 0 until 3) OutlinedTextField(marker[i], { marker[i] = it.take(40) }, label = { Text("Marker ${i + 1}") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            }
            "sicherung" -> {
                Text(sicherungText, style = MaterialTheme.typography.bodyLarge)
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    Button(onClick = { z.sichernImHintergrund { z.meldung = it } }) { Text("Jetzt sichern") }
                    OutlinedButton(onClick = { ordnerWahl.launch(null) }) { Text("Ordner ändern") }
                }
            }
            "woerter" -> {
                Text("Gib deine 24 Wörter ein, um zu prüfen, ob dein Zettel noch stimmt. Die Wörter werden nicht gespeichert.", style = MaterialTheme.typography.bodyMedium)
                WortProbe(z) { teil = null }
            }
            "schluessel" -> {
                Text("Bei Verdacht, dass jemand Zugriff hatte. Neue Einträge und Sicherungen sind dann mit alten Schlüsseln nicht mehr lesbar.", style = MaterialTheme.typography.bodyMedium)
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
            "export" -> {
                Text("Alle Einträge als Tabelle (CSV) im Klartext, zum Beispiel als Ausstieg oder für einen Arzttermin. Speichere sie nicht im Sicherungsordner und nicht in einem Ordner, der in eine Cloud synchronisiert wird.", style = MaterialTheme.typography.bodyMedium)
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
            else -> {
                Text("Version ${BuildConfigInfo.version} · Open Source (MPL-2.0)", style = MaterialTheme.typography.bodyLarge)
                Text("Quelltext: https://github.com/Blackfort-Technology/days-and-keys", style = MaterialTheme.typography.bodyMedium)
                Text("Keine Diagnosen, keine Empfehlungen, keine automatischen Zusammenhänge. Die App ersetzt keine ärztliche Beratung.", style = MaterialTheme.typography.bodyMedium)
                Text("Schlüssel: " + if (z.app.schluessel.strongBox) "StrongBox" else "Keystore des Geräts", style = MaterialTheme.typography.bodyMedium)
                Text("Schrift: Atkinson Hyperlegible Next (SIL Open Font License 1.1). Symbole: Material Symbols (Apache License 2.0).", style = MaterialTheme.typography.bodyMedium)
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
    Column {
        for (i in 0 until 24) WortFeld(z, i + 1, w[i]) { w[i] = it }
        Button(onClick = {
            z.meldung = if (z.versuch { z.kern.woerterPruefen(w.toList()) } == true) "Die Wörter stimmen." else "Die Wörter passen nicht."
            w.indices.forEach { w[it] = "" }
            fertig()
        }) { Text("Prüfen") }
    }
}
