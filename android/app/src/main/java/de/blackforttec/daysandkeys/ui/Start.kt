// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.EinnahmeStand
import de.blackforttec.daysandkeys.kern.Knopf

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun StartSeite(z: AppZustand) {
    LaunchedEffect(Unit) { z.neuLaden() }
    val s = z.start ?: return
    var bedarf by remember { mutableStateOf<Knopf?>(null) }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(vertical = 8.dp)) {
        Text(s.heute, style = MaterialTheme.typography.titleLarge, modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp))
        // Morgens bis 14 Uhr, solange heute noch nichts zum Schlaf erfasst ist; der Kern entscheidet (#139).
        if (s.schlafAnbieten) {
            Abschnitt("Wie hast du geschlafen?", "Wie lange und wie gut, in ein paar Sekunden.") {
                FilledTonalButton(onClick = { z.zu(Seite.Formular("aufgewacht")) }, modifier = Modifier.fillMaxWidth().height(56.dp)) { Text("Schlaf festhalten") }
            }
        }
        Abschnitt("Wie geht es dir gerade?", when (s.statusHeute.toInt()) {
            0 -> "Heute noch kein Status. Halte ihn fest, so oft du willst."
            1 -> "Heute 1 Status" + (s.letzterStatusMs?.let { ", zuletzt ${vorText(it)}" } ?: "")
            else -> "Heute ${s.statusHeute} Status" + (s.letzterStatusMs?.let { ", zuletzt ${vorText(it)}" } ?: "")
        }) {
            Button(onClick = { z.zu(Seite.Formular("status_jetzt")) }, modifier = Modifier.fillMaxWidth().height(56.dp)) { Text("Status jetzt festhalten") }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (s.tagesabschlussHeute) OutlinedButton(onClick = { z.zu(Seite.Formular("tagesabschluss")) }) { Text("Tagesabschluss") }
                if (s.wochenrueckblickHeute) OutlinedButton(onClick = { z.zu(Seite.Formular("wochenrueckblick")) }) { Text("Wochenrückblick") }
            }
        }
        if (s.krisenplanHinweis && !z.krisenhinweisGesehen) {
            Abschnitt("Dein Plan für Tiefs", "Du hast eingestellt, dass die App ihn dir zeigt, wenn deine Tageswerte eine Weile niedrig sind.") {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Button(onClick = { z.zu(Seite.Hilfe) }) { Text("Plan und Hilfe ansehen") }
                    TextButton(onClick = { z.krisenhinweisGesehen = true }) { Text("Ausblenden") }
                }
            }
        }
        for (e in s.entwuerfe) {
            Abschnitt("Angefangen: ${e.formularName}", "${vorText(e.erstelltMs)}, ${e.antworten} Antworten") {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Button(onClick = {
                        val f = z.versuch { z.kern.entwurfLaden(e.id) } ?: return@Button
                        val schl = formularSchluessel(z, f.formularId) ?: return@Button
                        z.zu(Seite.Formular(schl, entwurf = e.id))
                    }) { Text("Weiter ausfüllen") }
                    TextButton(onClick = { z.versuch { z.kern.entwurfVerwerfen(e.id) }; z.neuLaden() }) { Text("Verwerfen") }
                }
            }
        }
        if (s.einnahmenHeute.isNotEmpty() || s.bedarf.isNotEmpty()) {
            Abschnitt("Medikamente heute", "Einnahmen werden nie vorausgefüllt. Bestätige nur, was du genommen hast.") {
                for (e in s.einnahmenHeute) {
                    // Text über die volle Breite, Knöpfe darunter: daneben brach der Name mitten im Wort (#120).
                    Column(Modifier.fillMaxWidth()) {
                        Column {
                            Text("${uhrzeit(e.uhrzeitMin)}  ${e.name}" + (e.staerke?.let { " $it" } ?: ""))
                            Text(e.dosis + " · " + when (e.stand) {
                                EinnahmeStand.GENOMMEN -> "genommen"
                                EinnahmeStand.AUSGELASSEN -> "ausgelassen"
                                EinnahmeStand.OFFEN -> "offen"
                            }, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.secondary)
                        }
                        if (e.stand == EinnahmeStand.OFFEN) Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            FilledTonalButton(onClick = {
                                z.versuch { z.kern.einnahmeBestaetigen(e.praeparatId, null) }; z.geaendert(); z.meldung = "Einnahme festgehalten"
                            }) { Text("Genommen") }
                            TextButton(onClick = { z.zu(Seite.Formular("einnahme_ausgelassen", praeparat = e.praeparatId)) }) { Text("Ausgelassen") }
                        }
                    }
                }
                if (s.bedarf.isNotEmpty()) {
                    Text("Bei Bedarf", style = MaterialTheme.typography.labelLarge)
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        for (b in s.bedarf) OutlinedButton(onClick = { bedarf = b }) { Text(b.name) }
                    }
                }
            }
        }
        Abschnitt("Ereignis festhalten", "Ein Tipp, die Uhrzeit ist jetzt und lässt sich ändern.") {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                for (k in s.ereignisse) OutlinedButton(onClick = { z.zu(Seite.Formular(k.schluessel)) }) { Text(k.name) }
            }
        }
        Abschnitt("Sicherung", when {
            s.letzteSicherungMs == null -> "Noch keine Sicherung. Ohne Sicherung gehen bei Verlust des Geräts alle Einträge verloren."
            s.sicherungWarnung -> "Letzte Sicherung ${vorText(s.letzteSicherungMs!!)}. Bitte prüfen."
            else -> "Letzte Sicherung ${vorText(s.letzteSicherungMs!!)}"
        }) {
            if (s.sicherungWarnung) OutlinedButton(onClick = { z.zu(Seite.Einstellungen) }) { Text("Sicherung einrichten") }
        }
    }
    bedarf?.let { b ->
        AlertDialog(
            onDismissRequest = { bedarf = null },
            title = { Text(b.name) },
            text = { Text("Jetzt eingenommen?") },
            confirmButton = { Button(onClick = { z.versuch { z.kern.einnahmeBestaetigen(b.formularId, null) }; bedarf = null; z.geaendert() }) { Text("Ja, genommen") } },
            dismissButton = { TextButton(onClick = { bedarf = null }) { Text("Abbrechen") } },
        )
    }
}

/** Schlüssel eines Formulars aus seiner ID (für Entwürfe). */
fun formularSchluessel(z: AppZustand, formularId: String): String? =
    listOf("status_jetzt", "tagesabschluss", "wochenrueckblick").firstOrNull { z.versuch { z.kern.formular(it) }?.id == formularId }
