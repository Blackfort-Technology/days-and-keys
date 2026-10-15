// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
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
    var wert by remember { mutableStateOf<Long?>(null) }

    fun ereignis(k: Knopf) {
        // Ereignisse ohne Fragen werden mit dem Tipp gespeichert; „Krise“ fragt nach (Vorgabe 4.2).
        if (k.schluessel == "krise") { krise = k; return }
        val f = z.versuch { z.kern.formular(k.schluessel) } ?: return
        if (f.gruppen.all { it.fragen.isEmpty() } && !f.brauchtPraeparat) {
            z.versuch { z.kern.ereignisErfassen(EreignisEingabe(f.id, null, emptyList(), null, null)) } ?: return
            z.geaendert(); z.meldung = "${k.name} um ${uhrzeitAus(System.currentTimeMillis())} festgehalten"
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
                        wert = null; z.geaendert(); z.meldung = "Festgehalten um ${uhrzeitAus(System.currentTimeMillis())}"
                    }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.hauptknopf)) { Text("Festhalten") }
                    TextButton(onClick = { z.zu(Seite.Formular("status_jetzt", vorbelegt = listOf(antwort))); wert = null }) { Text("Mehr angeben") }
                } else {
                    if (s.statusHeute > 0u) s.letzterStatusMs?.let { Text("Zuletzt um ${uhrzeitAus(it)}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                        TextButton(onClick = { z.zu(Seite.Formular("status_jetzt")) }) { Text("Ausführlich") }
                        if (s.tagesabschlussHeute) TextButton(onClick = { z.zu(Seite.Formular("tagesabschluss")) }) { Text("Tagesabschluss") }
                        if (s.wochenrueckblickHeute) TextButton(onClick = { z.zu(Seite.Formular("wochenrueckblick")) }) { Text("Wochenrückblick") }
                    }
                }
            }
        }
        // Morgens bis 14 Uhr, solange heute noch nichts zum Schlaf erfasst ist; der Kern entscheidet (#139).
        if (s.schlafAnbieten) {
            Abschnitt("Wie hast du geschlafen?", "Wie lange und wie gut, in ein paar Sekunden.") {
                FilledTonalButton(onClick = { z.zu(Seite.Formular("aufgewacht")) }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.ziel)) { Text("Schlaf festhalten") }
            }
        }
        if (s.krisenplanHinweis && !z.krisenhinweisGesehen) {
            Abschnitt("Dein Plan für Tiefs", "Du hast eingestellt, dass die App ihn dir zeigt, wenn deine Tageswerte eine Weile niedrig sind.") {
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    FilledTonalButton(onClick = { z.zu(Seite.Hilfe) }) { Text("Plan ansehen") }
                    TextButton(onClick = { z.krisenhinweisGesehen = true }) { Text("Ausblenden") }
                }
            }
        }
        for (e in s.entwuerfe) {
            Abschnitt("${e.formularName}, angefangen", "${vorText(e.erstelltMs)} · ${e.antworten} " + if (e.antworten == 1u) "Antwort" else "Antworten") {
                Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    FilledTonalButton(onClick = {
                        val f = z.versuch { z.kern.entwurfLaden(e.id) } ?: return@FilledTonalButton
                        val schl = formularSchluessel(z, f.formularId) ?: return@FilledTonalButton
                        z.zu(Seite.Formular(schl, entwurf = e.id))
                    }) { Text("Weiter") }
                    TextButton(onClick = { z.versuch { z.kern.entwurfVerwerfen(e.id) }; z.neuLaden() }) { Text("Verwerfen") }
                }
            }
        }
        if (s.einnahmenHeute.isNotEmpty() || s.bedarf.isNotEmpty()) {
            val nochNichts = s.einnahmenHeute.none { it.stand != EinnahmeStand.OFFEN }
            Abschnitt("Medikamente heute", if (nochNichts) "Bestätige nur, was du genommen hast." else null) {
                for (e in s.einnahmenHeute) {
                    Row(Modifier.fillMaxWidth().padding(vertical = Abstand.xs), verticalAlignment = Alignment.Top) {
                        Text(uhrzeit(e.uhrzeitMin), style = MaterialTheme.typography.labelLarge, modifier = Modifier.width(56.dp).padding(top = 2.dp))
                        Column(Modifier.weight(1f)) {
                            Text(e.name + (e.staerke?.let { " $it" } ?: ""), style = MaterialTheme.typography.bodyLarge)
                            when (e.stand) {
                                EinnahmeStand.OFFEN -> {
                                    Text("Dosis ${e.dosis}", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s), modifier = Modifier.padding(top = Abstand.xs)) {
                                        FilledTonalButton(onClick = {
                                            z.versuch { z.kern.einnahmeBestaetigen(e.praeparatId, null) }; z.geaendert(); z.meldung = "Einnahme festgehalten"
                                        }) { Text("Genommen") }
                                        TextButton(onClick = { z.zu(Seite.Formular("einnahme_ausgelassen", praeparat = e.praeparatId)) }) { Text("Ausgelassen", maxLines = 1) }
                                    }
                                }
                                // Erledigt ohne Farbe für den Stand (AE-27).
                                EinnahmeStand.GENOMMEN -> Row(verticalAlignment = Alignment.CenterVertically) {
                                    Symbol(R.drawable.sym_check, null, Modifier.size(18.dp)); Text(" genommen", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                EinnahmeStand.AUSGELASSEN -> Text("ausgelassen", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                }
                if (s.bedarf.isNotEmpty()) {
                    Text("Bei Bedarf", style = MaterialTheme.typography.titleSmall)
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                        for (b in s.bedarf) OutlinedButton(onClick = { bedarf = b }) { Text(b.name) }
                    }
                }
            }
        }
        Abschnitt("Ereignis festhalten", "Ein Tipp hält es mit der Uhrzeit von jetzt fest. Ändern kannst du sie danach.") {
            val zeige = if (alleEreignisse) s.ereignisse else s.ereignisse.take(6)
            for (paar in zeige.chunked(2)) Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                for (k in paar) OutlinedButton(onClick = { ereignis(k) }, modifier = Modifier.weight(1f).heightIn(min = Abstand.ziel)) {
                    Text(k.name, style = MaterialTheme.typography.labelMedium, maxLines = 2)
                }
                if (paar.size == 1) Column(Modifier.weight(1f)) {}
            }
            if (s.ereignisse.size > 6) TextButton(onClick = { alleEreignisse = !alleEreignisse }) { Text(if (alleEreignisse) "Weniger zeigen" else "Alle Ereignisse") }
        }
        if (s.sicherungWarnung) {
            Abschnitt(
                "Sicherung",
                s.letzteSicherungMs?.let { "Die letzte Sicherung ist ${vorText(it).removePrefix("vor ")} alt." }
                    ?: "Noch keine Sicherung eingerichtet. Geht das Gerät verloren, sind die Einträge sonst weg.",
            ) { FilledTonalButton(onClick = { z.zu(Seite.Einstellungen) }) { Text("Sicherung einrichten") } }
        } else s.letzteSicherungMs?.let {
            Text("Gesichert ${vorText(it)}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(horizontal = Abstand.rand, vertical = Abstand.s))
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
    krise?.let { k ->
        AlertDialog(
            onDismissRequest = { krise = null },
            title = { Text("Krise festhalten?") },
            text = { Text("Danach zeigt die App die Hilfe-Seite mit Nummern. Die App benachrichtigt niemanden.") },
            confirmButton = {
                Button(onClick = {
                    val f = z.versuch { z.kern.formular(k.schluessel) }
                    if (f != null && z.versuch { z.kern.ereignisErfassen(EreignisEingabe(f.id, null, emptyList(), null, null)) } != null) z.geaendert()
                    krise = null; z.zu(Seite.Hilfe)
                }) { Text("Festhalten und Hilfe zeigen") }
            },
            dismissButton = { TextButton(onClick = { krise = null }) { Text("Abbrechen") } },
        )
    }
}

/** Schlüssel eines Formulars aus seiner ID (für Entwürfe). */
fun formularSchluessel(z: AppZustand, formularId: String): String? =
    listOf("status_jetzt", "tagesabschluss", "wochenrueckblick").firstOrNull { z.versuch { z.kern.formular(it) }?.id == formularId }
