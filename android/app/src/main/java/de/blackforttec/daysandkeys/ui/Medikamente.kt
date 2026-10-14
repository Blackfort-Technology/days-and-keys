// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

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
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.PraeparatDaten
import de.blackforttec.daysandkeys.kern.Planzeit

@Composable
fun MedikamenteSeite(z: AppZustand) {
    val liste = remember(z.seite) { z.versuch { z.kern.praeparate() }.orEmpty() }
    Column(Modifier.verticalScroll(rememberScrollState())) {
        Abschnitt("Medikamente", "Präparat, Stärke und Einnahmeplan. Ein Dosis- oder Präparatwechsel ist zusätzlich ein Ereignis, damit er im Verlauf sichtbar ist.") {
            Button(onClick = { z.zu(Seite.Medikament(null)) }, modifier = Modifier.fillMaxWidth()) { Text("Präparat hinzufügen") }
        }
        for (p in liste) {
            Abschnitt(p.handelsname + (p.staerke?.let { " $it" } ?: ""), buildString {
                if (!p.aktiv) append("pausiert · ")
                if (p.bedarf) append("bei Bedarf · ")
                append(p.plan.joinToString(", ") { uhrzeit(it.uhrzeitMin) + " " + it.dosis }.ifEmpty { "ohne festen Plan" })
            }) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedButton(onClick = { z.zu(Seite.Medikament(p)) }) { Text("Bearbeiten") }
                    TextButton(onClick = { z.zu(Seite.Formular("dosiswechsel", praeparat = p.id)) }) { Text("Wechsel festhalten") }
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun MedikamentSeite(z: AppZustand, alt: PraeparatDaten?) {
    var name by remember { mutableStateOf(alt?.handelsname ?: "") }
    var wirkstoff by remember { mutableStateOf(alt?.wirkstoff ?: "") }
    var staerke by remember { mutableStateOf(alt?.staerke ?: "") }
    var form by remember { mutableStateOf(alt?.form ?: "") }
    var bedarf by remember { mutableStateOf(alt?.bedarf ?: false) }
    var wirkung by remember { mutableStateOf(alt?.wirkungSpuerbar ?: false) }
    var aktiv by remember { mutableStateOf(alt?.aktiv ?: true) }
    var erinnerung by remember { mutableStateOf(alt?.erinnerung ?: true) }
    var statusNach by remember { mutableStateOf(alt?.statusNachMin?.toString() ?: "") }
    val plan = remember { mutableStateListOf(*(alt?.plan ?: emptyList()).toTypedArray()) }
    Column(Modifier.verticalScroll(rememberScrollState())) {
        Abschnitt(if (alt == null) "Neues Präparat" else "Präparat bearbeiten") {
            OutlinedTextField(name, { name = it }, label = { Text("Name (Handelsname)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            OutlinedTextField(wirkstoff, { wirkstoff = it }, label = { Text("Wirkstoff (optional)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(staerke, { staerke = it }, label = { Text("Stärke") }, singleLine = true, modifier = Modifier.weight(1f))
                OutlinedTextField(form, { form = it }, label = { Text("Form") }, singleLine = true, modifier = Modifier.weight(1f))
            }
            Schalter("Bei Bedarf statt nach Plan", bedarf) { bedarf = it }
            Schalter("Wirkung im Tagesverlauf spürbar (dann fragt der Status danach)", wirkung) { wirkung = it }
            Schalter("Aktiv", aktiv) { aktiv = it }
            Schalter("Erinnern (neutraler Text ohne Namen)", erinnerung) { erinnerung = it }
        }
        Abschnitt("Einnahmeplan") {
            for ((i, p) in plan.withIndex()) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    OutlinedTextField(uhrzeit(p.uhrzeitMin), { t ->
                        val teile = t.split(":"); val h = teile.getOrNull(0)?.toIntOrNull(); val m = teile.getOrNull(1)?.toIntOrNull() ?: 0
                        if (h != null && h in 0..23 && m in 0..59) plan[i] = p.copy(uhrzeitMin = h * 60 + m)
                    }, label = { Text("Uhrzeit") }, singleLine = true, modifier = Modifier.width(110.dp))
                    OutlinedTextField(p.dosis, { plan[i] = p.copy(dosis = it.take(100)) }, label = { Text("Dosis") }, singleLine = true, modifier = Modifier.weight(1f))
                    TextButton(onClick = { plan.removeAt(i) }) { Text("Entfernen") }
                }
            }
            OutlinedButton(onClick = { plan.add(Planzeit(8 * 60, "1", 0x7fu)) }) { Text("Uhrzeit hinzufügen") }
            OutlinedTextField(statusNach, { statusNach = it.filter(Char::isDigit).take(3) }, label = { Text("Status-Erinnerung nach Einnahme (Minuten, optional)") },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.fillMaxWidth())
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
            Button(onClick = {
                val p = PraeparatDaten(alt?.id ?: "", name, wirkstoff, staerke, form, plan.toList(), bedarf, wirkung, aktiv, erinnerung, statusNach.toLongOrNull())
                if (z.versuch { z.kern.praeparatSpeichern(p) } != null) { z.geaendert(); z.zu(Seite.Medikamente) }
            }) { Text("Speichern") }
        }
        Abstand(24)
    }
}

@Composable
fun Schalter(text: String, wert: Boolean, setzen: (Boolean) -> Unit) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
        Text(text, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        Switch(checked = wert, onCheckedChange = setzen)
    }
}
