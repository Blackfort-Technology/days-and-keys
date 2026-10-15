// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TimePicker
import androidx.compose.material3.rememberTimePickerState
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
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.PraeparatDaten
import de.blackforttec.daysandkeys.kern.Planzeit

/** Liste der Präparate (#136, Vorgabe 4.6). */
@Composable
fun MedikamenteSeite(z: AppZustand) {
    val liste = remember(z.seite) { z.versuch { z.kern.praeparate() }.orEmpty() }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.verticalScroll(rememberScrollState()).padding(bottom = 96.dp)) {
            if (liste.isEmpty()) Column(Modifier.padding(Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
                Symbol(R.drawable.sym_medication, null)
                Text("Noch keine Medikamente eingetragen. Du kannst die App auch ganz ohne nutzen.", style = MaterialTheme.typography.bodyLarge)
            }
            for (p in liste) {
                Klickzeile(p.handelsname + (p.staerke?.let { " $it" } ?: ""), when {
                    !p.aktiv -> "pausiert"
                    p.bedarf -> "bei Bedarf"
                    p.plan.isEmpty() -> "ohne festen Plan"
                    else -> p.plan.joinToString(" · ") { uhrzeit(it.uhrzeitMin) }
                }) { z.zu(Seite.Medikament(p)) }
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            }
        }
        ExtendedFloatingActionButton(
            onClick = { z.zu(Seite.Medikament(null)) },
            modifier = Modifier.align(Alignment.BottomEnd).padding(Abstand.rand),
            containerColor = MaterialTheme.colorScheme.primaryContainer,
        ) { Text("Präparat hinzufügen") }
    }
}

/** Präparat als ganze Seite; Uhrzeiten über den Zeitdialog, Rückfrage bei Änderungen (#136). */
@OptIn(ExperimentalMaterial3Api::class)
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
    var zeitFuer by remember { mutableStateOf<Int?>(null) }
    var neueZeile by remember { mutableStateOf(false) }
    var nameFehlt by remember { mutableStateOf(false) }
    var verlassen by remember { mutableStateOf(false) }
    fun daten() = PraeparatDaten(alt?.id ?: "", name.trim(), wirkstoff.ifBlank { null }, staerke.ifBlank { null }, form.ifBlank { null }, plan.toList(), bedarf, wirkung, aktiv, erinnerung, statusNach.toLongOrNull())
    val geaendert = alt?.let { daten() != it.copy(wirkstoff = it.wirkstoff?.ifBlank { null }, staerke = it.staerke?.ifBlank { null }, form = it.form?.ifBlank { null }) } ?: (name.isNotBlank() || plan.isNotEmpty())
    BackHandler(enabled = geaendert) { verlassen = true }
    fun speichern() {
        if (name.isBlank()) { nameFehlt = true; return }
        if (z.versuch { z.kern.praeparatSpeichern(daten()) } != null) { z.geaendert(); z.meldung = "Gespeichert"; z.zu(Seite.Medikamente) }
    }

    Column(Modifier.fillMaxSize().imePadding()) {
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
            Kopf("Präparat")
            OutlinedTextField(name, { name = it; nameFehlt = false }, label = { Text("Name") }, singleLine = true, isError = nameFehlt,
                supportingText = if (nameFehlt) ({ Text("Bitte einen Namen eingeben.") }) else null, modifier = Modifier.fillMaxWidth())
            OutlinedTextField(wirkstoff, { wirkstoff = it }, label = { Text("Wirkstoff (optional)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            Row(horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                OutlinedTextField(staerke, { staerke = it }, label = { Text("Stärke") }, singleLine = true, modifier = Modifier.weight(1f))
                OutlinedTextField(form, { form = it }, label = { Text("Form") }, singleLine = true, modifier = Modifier.weight(1f))
            }
            Kopf("Einnahme")
            Stufenwahl(listOf(0L to "Nach Plan", 1L to "Bei Bedarf"), if (bedarf) 1L else 0L) { it?.let { w -> bedarf = w == 1L } }
            if (!bedarf) {
                for ((i, p) in plan.withIndex()) Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Abstand.s)) {
                    OutlinedButton(onClick = { zeitFuer = i }, modifier = Modifier.width(104.dp).heightIn(min = Abstand.ziel)) { Text(uhrzeit(p.uhrzeitMin), style = MaterialTheme.typography.labelLarge) }
                    OutlinedTextField(p.dosis, { plan[i] = p.copy(dosis = it.take(100)) }, label = { Text("Dosis") }, singleLine = true, modifier = Modifier.weight(1f))
                    IconButton(onClick = { plan.removeAt(i) }) { Symbol(R.drawable.sym_delete, "Uhrzeit ${uhrzeit(p.uhrzeitMin)} entfernen") }
                }
                TextButton(onClick = {
                    // Neue Uhrzeit vier Stunden nach der letzten, damit nicht zweimal 08:00 entsteht.
                    val naechste = plan.maxOfOrNull { it.uhrzeitMin }?.let { (it + 4 * 60).coerceAtMost(23 * 60) } ?: (8 * 60)
                    plan.add(Planzeit(naechste, "1", 0x7fu)); zeitFuer = plan.lastIndex; neueZeile = true
                }) { Text("Uhrzeit hinzufügen") }
            }
            Kopf("Erinnerung")
            SchalterZeile("An die Einnahme erinnern", "Die Benachrichtigung nennt kein Medikament.", erinnerung) { erinnerung = it }
            OutlinedTextField(statusNach, { statusNach = it.filter(Char::isDigit).take(3) }, label = { Text("Status-Erinnerung danach") }, suffix = { Text("Min.") },
                supportingText = { Text("Leer lassen für keine.") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.fillMaxWidth())
            Kopf("Im Status")
            SchalterZeile("Nach der Wirkung fragen", "Der Status fragt dann, ob du die Wirkung gerade spürst.", wirkung) { wirkung = it }
            SchalterZeile("Nehme ich derzeit", null, aktiv) { aktiv = it }
            if (alt != null) {
                Kopf("Wechsel", "Ein Dosis- oder Präparatwechsel ist zusätzlich ein Ereignis, damit er im Verlauf sichtbar ist.")
                OutlinedButton(onClick = { z.zu(Seite.Formular("dosiswechsel", praeparat = alt.id)) }) { Text("Dosis- oder Präparatwechsel festhalten") }
            }
            Abstand(24)
        }
        Aktionsleiste("Speichern") { speichern() }
    }

    zeitFuer?.let { i ->
        val p = plan.getOrNull(i) ?: run { zeitFuer = null; return }
        val st = rememberTimePickerState(p.uhrzeitMin / 60, p.uhrzeitMin % 60, true)
        AlertDialog(
            // Abbrechen beim Hinzufügen nimmt die neue Zeile wieder heraus (#151).
            onDismissRequest = { if (neueZeile) plan.removeAt(i); neueZeile = false; zeitFuer = null },
            title = { Text("Uhrzeit") },
            text = { TimePicker(st) },
            confirmButton = { Button(onClick = { plan[i] = p.copy(uhrzeitMin = st.hour * 60 + st.minute); neueZeile = false; zeitFuer = null }) { Text("Übernehmen") } },
            dismissButton = { TextButton(onClick = { if (neueZeile) plan.removeAt(i); neueZeile = false; zeitFuer = null }) { Text("Abbrechen") } },
        )
    }
    if (verlassen) AlertDialog(
        onDismissRequest = { verlassen = false },
        title = { Text("Änderungen verwerfen?") },
        confirmButton = { TextButton(onClick = { verlassen = false; z.zu(Seite.Medikamente) }) { Text("Verwerfen") } },
        dismissButton = { Button(onClick = { verlassen = false }) { Text("Weiter bearbeiten") } },
    )
}

/** Schalter mit Beschriftung und optionaler Erklärzeile statt Klammern (#136). */
@Composable
fun SchalterZeile(text: String, zeile: String?, wert: Boolean, setzen: (Boolean) -> Unit) {
    Row(Modifier.fillMaxWidth().heightIn(min = 56.dp), verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(1f)) {
            Text(text, style = MaterialTheme.typography.bodyLarge)
            zeile?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        }
        Switch(checked = wert, onCheckedChange = setzen)
    }
}

@Composable
fun Schalter(text: String, wert: Boolean, setzen: (Boolean) -> Unit) = SchalterZeile(text, null, wert, setzen)
