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
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
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
import java.time.Instant
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
    val auf = remember(s) { mutableStateMapOf<Int, Boolean>() }
    val istEreignis = f.art == Formularart.EREIGNIS
    val praeparate = remember(s) { if (f.brauchtPraeparat) z.versuch { z.kern.praeparate() }.orEmpty().filter { it.aktiv } else emptyList() }

    LaunchedEffect(s) {
        s.vorbelegt.forEach { antworten[it.messdefId] = it }
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
            z.meldung = if (s.korrektur != null) "Korrigiert" else "Festgehalten um ${uhrzeitAus(zeitpunkt?.utcMs ?: System.currentTimeMillis())}"
            z.zu(if (f.schluessel == "krise") Seite.Hilfe else if (s.korrektur != null) Seite.Eintraege else Seite.Start)
        }
    }

    // Ganze Seite: Inhalt direkt auf der Fläche, „Festhalten“ fest unten (#134, Vorgabe 4.3).
    Column(Modifier.fillMaxSize().imePadding()) {
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand)) {
            Row(Modifier.fillMaxWidth().padding(top = Abstand.s), verticalAlignment = Alignment.CenterVertically) {
                Zeitchip(zeitpunkt?.let { zeitchipText(it) } ?: "Jetzt") { zeitWahl = true }
                Spacer(Modifier.weight(1f))
                if (entwurfId != null && s.korrektur == null) TextButton(onClick = {
                    entwurfId?.let { id -> z.versuch { z.kern.entwurfVerwerfen(id) } }
                    z.zu(Seite.Start)
                }) { Text("Entwurf verwerfen") }
            }
            if (f.schluessel == "krise") {
                Text("Danach zeigt die App die Hilfe-Seite mit Nummern. Die App benachrichtigt niemanden.", style = MaterialTheme.typography.bodyLarge, modifier = Modifier.padding(top = Abstand.m))
            }
            if (f.brauchtPraeparat) {
                Kopf("Präparat")
                if (praeparate.isEmpty()) Text("Noch kein Präparat angelegt. Du kannst es unter Medikamente anlegen.", style = MaterialTheme.typography.bodyMedium)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    for (p in praeparate) AuswahlChip(p.handelsname + (p.staerke?.let { " $it" } ?: ""), praeparat == p.id) { praeparat = p.id }
                }
            }
            for ((i, g) in f.gruppen.withIndex()) {
                val beantwortet = g.fragen.count { antworten.containsKey(it.messdefId) }
                when {
                    g.zugeklappt -> {
                        if (!f.blockAnbieten && !blockOffen) continue
                        Aufklappgruppe(
                            f.bezugText?.let { "Seit dem letzten Status, $it" } ?: g.titel,
                            if (beantwortet > 0) "$beantwortet beantwortet" else "optional · ${g.fragen.size} Fragen",
                            blockOffen, { blockOffen = !blockOffen },
                        ) { for (q in g.fragen) Frage(q, antworten) }
                    }
                    g.aufklappbar -> {
                        val offen = auf[i] ?: (beantwortet > 0)
                        Aufklappgruppe(
                            g.titel + (g.hinweis?.let { " · $it" } ?: ""),
                            if (beantwortet > 0) "$beantwortet beantwortet" else "optional · ${g.fragen.size} Fragen",
                            offen, { auf[i] = !offen },
                        ) { for (q in g.fragen) Frage(q, antworten) }
                    }
                    else -> {
                        if (g.titel.isNotEmpty()) Kopf(g.titel, g.hinweis)
                        Column(verticalArrangement = Arrangement.spacedBy(Abstand.frage)) { for (q in g.fragen) Frage(q, antworten) }
                    }
                }
            }
            Text(
                "Was du auslässt, bleibt leer. Nichts wird aus früheren Einträgen übernommen.",
                style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(vertical = Abstand.xl),
            )
        }
        Aktionsleiste(if (s.korrektur != null) "Korrektur festhalten" else "Festhalten") { speichern() }
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
        Text(q.frage, style = MaterialTheme.typography.titleMedium)
        q.hinweis?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        when (q.feldart) {
            Feldart.SKALA -> {
                val stufen = (q.max ?: 10) - (q.min ?: 0) + 1
                if (stufen > 5) Stufenleiste(q, a?.zahl) { w -> if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w) }
                else Column {
                    Stufenwahl(((q.min ?: 0)..(q.max ?: 5)).map { it to (if ((q.min ?: 0) < 0 && it > 0) "+$it" else "$it") }, a?.zahl) { w -> if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w) }
                    Ankerzeile(q.anker.map { it.text })
                }
            }
            Feldart.STUFEN -> Stufenwahl(q.anker.map { it.wert to it.text }, a?.zahl) { w ->
                if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w)
            }
            Feldart.JA_NEIN -> Stufenwahl(listOf(1L to "ja", 0L to "nein"), a?.zahl) { w ->
                if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, zahl = w)
            }
            Feldart.EINFACHAUSWAHL -> if (q.optionen.size <= 5) Stufenwahl(q.optionen.map { it.id.toLong() to it.text }, a?.auswahl?.firstOrNull()?.option?.toLong()) { w ->
                if (w == null) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, auswahl = listOf(Auswahl(q.optionen.first { it.id.toLong() == w }.id, null)))
            } else FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                for (o in q.optionen) {
                    val gewaehlt = a?.auswahl?.any { it.option == o.id } == true
                    AuswahlChip(o.text, gewaehlt) {
                        if (gewaehlt) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, auswahl = listOf(Auswahl(o.id, null)))
                    }
                }
            }
            Feldart.MEHRFACHAUSWAHL -> Column {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    for (o in q.optionen) {
                        val gewaehlt = a?.auswahl?.any { it.option == o.id } == true
                        AuswahlChip(o.text, gewaehlt) {
                            val alt = a?.auswahl.orEmpty()
                            val neu = if (gewaehlt) alt.filter { it.option != o.id } else alt + Auswahl(o.id, null)
                            if (neu.isEmpty()) antworten.remove(q.messdefId) else antworten[q.messdefId] = antwort(q, auswahl = neu)
                        }
                    }
                }
                if (q.mitStaerke) for (x in a?.auswahl.orEmpty().filter { it.option.toInt() != 99 }) {
                    val name = q.optionen.firstOrNull { it.id == x.option }?.text ?: ""
                    Text(name, style = MaterialTheme.typography.labelMedium, modifier = Modifier.padding(top = Abstand.m, bottom = Abstand.xs))
                    Stufenwahl(listOf(2L to "leicht", 5L to "mittel", 8L to "stark"), x.staerke) { w ->
                        val neu = a!!.auswahl.map { if (it.option == x.option) Auswahl(it.option, w) else it }
                        antworten[q.messdefId] = antwort(q, auswahl = neu)
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
            AuswahlChip("kam nicht vor", kam) {
                if (kam) antworten.remove(q.messdefId) else antworten[q.messdefId] = AntwortEingabe(q.messdefId, AntwortZustand.KAM_NICHT_VOR, null, null, emptyList(), true)
            }
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

/** „Heute, 08:15“ oder „Gestern, 23:40“ für den Zeitchip (#134). */
fun zeitchipText(z: Zeitpunkt): String {
    val t = Instant.ofEpochMilli(z.utcMs).atZone(ZoneId.systemDefault())
    val tag = if (t.toLocalDate() == LocalDate.now()) "Heute" else if (t.toLocalDate() == LocalDate.now().minusDays(1)) "Gestern" else t.toLocalDate().toString()
    return "$tag, %02d:%02d".format(t.hour, t.minute)
}
