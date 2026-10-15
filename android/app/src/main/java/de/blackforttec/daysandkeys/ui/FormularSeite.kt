// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TimePicker
import androidx.compose.material3.rememberTimePickerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.AntwortEingabe
import de.blackforttec.daysandkeys.kern.AntwortZustand
import de.blackforttec.daysandkeys.kern.Auswahl
import de.blackforttec.daysandkeys.kern.EreignisEingabe
import de.blackforttec.daysandkeys.kern.Feldart
import de.blackforttec.daysandkeys.kern.FormularAnsicht
import de.blackforttec.daysandkeys.kern.Formularart
import de.blackforttec.daysandkeys.kern.FrageAnsicht
import de.blackforttec.daysandkeys.kern.StatusEingabe
import de.blackforttec.daysandkeys.kern.Zeitpunkt
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneId
import kotlinx.coroutines.delay

private fun antwort(f: FrageAnsicht, zahl: Long? = null, text: String? = null, auswahl: List<Auswahl> = emptyList()) =
    AntwortEingabe(f.messdefId, AntwortZustand.BEANTWORTET, zahl, text, auswahl, true)

@OptIn(ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)
@Composable
fun FormularSeite(z: AppZustand, s: Seite.Formular) {
    val f: FormularAnsicht = remember(s) { z.versuch { z.kern.formular(s.schluessel) } } ?: return
    val antworten = remember(s) { mutableStateMapOf<String, AntwortEingabe>() }
    var blockOffen by remember(s) { mutableStateOf(false) }
    var entwurfId by remember(s) { mutableStateOf(s.entwurf) }
    var praeparat by remember(s) { mutableStateOf(s.praeparat ?: s.korrektur?.praeparatId) }
    var zeitpunkt by remember(s) { mutableStateOf(s.korrektur?.zeitpunkt) }
    var zeitWahl by remember { mutableStateOf(false) }
    var geladen by remember(s) { mutableStateOf(false) }
    val istEreignis = f.art == Formularart.EREIGNIS
    val praeparate = remember(s) { if (f.brauchtPraeparat) z.versuch { z.kern.praeparate() }.orEmpty().filter { it.aktiv } else emptyList() }

    LaunchedEffect(s) {
        s.korrektur?.let { k -> k.antworten.forEach { antworten[it.messdefId] = it }; blockOffen = k.blockOffen }
        s.entwurf?.let { id -> z.versuch { z.kern.entwurfLaden(id) }?.let { e -> e.antworten.forEach { antworten[it.messdefId] = it }; blockOffen = e.blockOffen } }
        geladen = true
    }
    // Laufend als Entwurf versiegeln (#94); auch nach dem Sperren verfügbar.
    if (!istEreignis && s.korrektur == null) {
        LaunchedEffect(antworten.toMap(), blockOffen) {
            if (!geladen || antworten.isEmpty()) return@LaunchedEffect
            delay(600)
            entwurfId = z.versuch { z.kern.entwurfSichern(entwurfId, f.id, antworten.values.toList(), blockOffen) } ?: entwurfId
        }
    }

    fun speichern() {
        val r = if (istEreignis) z.versuch {
            z.kern.ereignisErfassen(EreignisEingabe(f.id, zeitpunkt, antworten.values.toList(), praeparat, s.korrektur?.zeile?.datensatzId))
        } else z.versuch {
            z.kern.statusSpeichern(StatusEingabe(f.id, zeitpunkt, antworten.values.toList(), blockOffen, entwurfId, s.korrektur?.zeile?.datensatzId))
        }
        if (r != null) {
            z.geaendert()
            z.meldung = if (s.korrektur != null) "Korrigiert" else "Festgehalten"
            z.zu(if (f.schluessel == "krise") Seite.Hilfe else Seite.Start)
        }
    }

    Column(Modifier.verticalScroll(rememberScrollState()).padding(vertical = 8.dp)) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text(f.name, style = MaterialTheme.typography.titleLarge)
            TextButton(onClick = { zeitWahl = true }) {
                Text(zeitpunkt?.let { "um " + uhrzeit(((it.utcMs / 60000 + it.versatzMin).mod(1440L)).toInt()) } ?: "jetzt, Zeit ändern")
            }
        }
        if (f.schluessel == "krise") {
            Abschnitt("Krise festhalten?", "Danach zeigt die App die Hilfe-Seite mit Nummern. Die App benachrichtigt niemanden.") {}
        }
        if (f.brauchtPraeparat) {
            Abschnitt("Präparat") {
                if (praeparate.isEmpty()) Text("Noch kein Präparat angelegt (unter Medikamente).")
                FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    for (p in praeparate) FilterChip(selected = praeparat == p.id, onClick = { praeparat = p.id }, label = { Text(p.handelsname + (p.staerke?.let { " $it" } ?: "")) })
                }
            }
        }
        for (g in f.gruppen) {
            if (g.zugeklappt) {
                if (!f.blockAnbieten && !blockOffen) continue
                Abschnitt(g.titel + (f.bezugText?.let { " ($it)" } ?: ""), if (!blockOffen) "Zugeklappt. Öffnen, wenn du etwas dazu sagen willst." else null) {
                    if (!blockOffen) OutlinedButton(onClick = { blockOffen = true }) { Text("Öffnen") }
                    else for (q in g.fragen) Frage(q, antworten)
                }
            } else {
                Abschnitt(g.titel, g.hinweis) { for (q in g.fragen) Frage(q, antworten) }
            }
        }
        Row(Modifier.fillMaxWidth().padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Button(onClick = { speichern() }, modifier = Modifier.weight(1f).height(52.dp)) { Text("Speichern") }
            OutlinedButton(onClick = {
                entwurfId?.let { id -> z.versuch { z.kern.entwurfVerwerfen(id) } }
                z.zu(Seite.Start)
            }, modifier = Modifier.height(52.dp)) { Text("Verwerfen") }
        }
        Text(
            "Unbeantwortete Fragen gelten als übersprungen. Nichts wird aus früheren Einträgen übernommen.",
            style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.secondary, modifier = Modifier.padding(horizontal = 16.dp),
        )
        Abstand(24)
    }

    if (zeitWahl) {
        val jetzt = LocalTime.now()
        val st = rememberTimePickerState(jetzt.hour, jetzt.minute, true)
        var gestern by remember { mutableStateOf(false) }
        AlertDialog(
            onDismissRequest = { zeitWahl = false },
            title = { Text("Wann war das?") },
            text = {
                Column {
                    TimePicker(st)
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        FilterChip(selected = !gestern, onClick = { gestern = false }, label = { Text("heute") })
                        Box(Modifier.width(8.dp))
                        FilterChip(selected = gestern, onClick = { gestern = true }, label = { Text("gestern") })
                    }
                }
            },
            confirmButton = {
                Button(onClick = {
                    val zdt = LocalDate.now().minusDays(if (gestern) 1 else 0).atTime(st.hour, st.minute).atZone(ZoneId.systemDefault())
                    zeitpunkt = Zeitpunkt(zdt.toInstant().toEpochMilli(), zdt.offset.totalSeconds / 60, zdt.zone.id)
                    zeitWahl = false
                }) { Text("Übernehmen") }
            },
            dismissButton = { TextButton(onClick = { zeitpunkt = null; zeitWahl = false }) { Text("Jetzt") } },
        )
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Frage(q: FrageAnsicht, antworten: MutableMap<String, AntwortEingabe>) {
    val a = antworten[q.messdefId]
    Column(Modifier.fillMaxWidth().padding(vertical = 6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(q.frage, style = MaterialTheme.typography.bodyLarge)
        q.hinweis?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.secondary) }
        when (q.feldart) {
            Feldart.SKALA -> TippReihe(q, a?.zahl) { w -> if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w) }
            Feldart.STUFEN -> FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (an in q.anker) FilterChip(selected = a?.zahl == an.wert, onClick = {
                    if (a?.zahl == an.wert) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = an.wert)
                }, label = { Text(an.text) })
            }
            Feldart.JA_NEIN -> Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for ((w, t) in listOf(1L to "ja", 0L to "nein")) FilterChip(selected = a?.zahl == w, onClick = {
                    if (a?.zahl == w) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w)
                }, label = { Text(t) })
            }
            Feldart.EINFACHAUSWAHL -> FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (o in q.optionen) {
                    val gewaehlt = a?.auswahl?.any { it.option == o.id } == true
                    FilterChip(selected = gewaehlt, onClick = {
                        if (gewaehlt) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, auswahl = listOf(Auswahl(o.id, null)))
                    }, label = { Text(o.text) })
                }
            }
            Feldart.MEHRFACHAUSWAHL -> Column {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    for (o in q.optionen) {
                        val gewaehlt = a?.auswahl?.any { it.option == o.id } == true
                        FilterChip(selected = gewaehlt, onClick = {
                            val alt = a?.auswahl.orEmpty()
                            val neu = if (gewaehlt) alt.filter { it.option != o.id } else alt + Auswahl(o.id, null)
                            if (neu.isEmpty()) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, auswahl = neu)
                        }, label = { Text(o.text) })
                    }
                }
                if (q.mitStaerke) for (x in a?.auswahl.orEmpty().filter { it.option.toInt() != 99 }) {
                    val name = q.optionen.firstOrNull { it.id == x.option }?.text ?: ""
                    // Name über den Stufen; nebeneinander wurde „stark“ senkrecht gequetscht (#120).
                    Text(name, style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 4.dp))
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        for ((w, t) in listOf(2L to "leicht", 5L to "mittel", 8L to "stark")) FilterChip(selected = x.staerke == w, onClick = {
                            val neu = a!!.auswahl.map { if (it.option == x.option) Auswahl(it.option, if (it.staerke == w) null else w) else it }
                            antworten[q.messdefId] = antwort(q, auswahl = neu)
                        }, label = { Text(t) })
                    }
                }
            }
            Feldart.TEXT -> OutlinedTextField(
                value = a?.text ?: "",
                onValueChange = { t -> if (t.isEmpty()) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, text = t.take(20000)) },
                modifier = Modifier.fillMaxWidth(), minLines = 2,
            )
            Feldart.DAUER -> DauerFeld(a?.zahl) { w -> if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w) }
            Feldart.ZAHL, Feldart.UHRZEIT -> OutlinedTextField(
                value = a?.zahl?.toString() ?: "",
                onValueChange = { t -> val w = t.filter(Char::isDigit).take(4).toLongOrNull(); if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w.coerceIn(q.min ?: 0, q.max ?: 9999)) },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.width(140.dp),
            )
        }
        if (q.kamNichtVor) {
            val kam = a?.zustand == AntwortZustand.KAM_NICHT_VOR
            FilterChip(selected = kam, onClick = {
                if (kam) antworten.remove(q.messdefId) else antworten[q.messdefId] = AntwortEingabe(q.messdefId, AntwortZustand.KAM_NICHT_VOR, null, null, emptyList(), true)
            }, label = { Text("kam nicht vor") })
        }
    }
}

/** Tipp-Reihe ohne Startposition; ein zweiter Tipp nimmt die Antwort zurück (#73). */
@Composable
fun TippReihe(q: FrageAnsicht, wert: Long?, setzen: (Long?) -> Unit) {
    val min = q.min ?: 0
    val max = q.max ?: 10
    Column {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(3.dp)) {
            for (w in min..max) {
                val gewaehlt = wert == w
                val farbe = if (gewaehlt) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
                Box(
                    Modifier.weight(1f).height(44.dp)
                        .background(farbe, RoundedCornerShape(8.dp))
                        .border(if (w == 0L && min < 0) 1.dp else 0.dp, MaterialTheme.colorScheme.secondary, RoundedCornerShape(8.dp))
                        .clickable { setzen(if (gewaehlt) null else w) }
                        .semantics { contentDescription = "$w"; selected = gewaehlt },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(if (w > 0 && min < 0) "+$w" else "$w", color = if (gewaehlt) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, style = MaterialTheme.typography.labelMedium, textAlign = TextAlign.Center)
                }
            }
        }
        if (q.anker.isNotEmpty()) Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            for (an in q.anker) Text(an.text, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.secondary)
        }
    }
}

@Composable
fun DauerFeld(wert: Long?, setzen: (Long?) -> Unit) {
    var h by remember { mutableStateOf(wert?.div(60)?.toString() ?: "") }
    var m by remember { mutableStateOf(wert?.rem(60)?.toString() ?: "") }
    fun neu() {
        val hh = h.toLongOrNull(); val mm = m.toLongOrNull()
        setzen(if (hh == null && mm == null) null else ((hh ?: 0) * 60 + (mm ?: 0)).coerceIn(0, 1440))
    }
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        OutlinedTextField(h, { h = it.filter(Char::isDigit).take(2); neu() }, label = { Text("Std.") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.width(100.dp))
        OutlinedTextField(m, { m = it.filter(Char::isDigit).take(2); neu() }, label = { Text("Min.") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.width(100.dp))
    }
}
