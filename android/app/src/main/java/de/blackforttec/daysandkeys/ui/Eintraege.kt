// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp

/** Einträge mit Tagesköpfen (#135, Vorgabe 4.8); Tipp öffnet das Detail als Seite. */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun EintraegeSeite(z: AppZustand) {
    var tage by remember { mutableIntStateOf(60) }
    val heute = remember { z.versuch { z.kern.heute() } ?: 0L }
    val liste = remember(tage) { z.versuch { z.kern.eintraege(heute - tage, heute) }.orEmpty() }
    LazyColumn(Modifier.fillMaxSize()) {
        if (liste.isEmpty()) item {
            Text("Noch keine Einträge. Was du festhältst, erscheint hier.", style = MaterialTheme.typography.bodyLarge, modifier = Modifier.padding(Abstand.rand))
        }
        for ((tag, zeilen) in liste.groupBy { it.tagText }) {
            stickyHeader(tag) {
                Text(
                    tag, style = MaterialTheme.typography.titleSmall,
                    modifier = Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.background).padding(horizontal = Abstand.rand, vertical = Abstand.s).semantics { heading() },
                )
            }
            items(zeilen, key = { it.datensatzId }) { e ->
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 72.dp).clickable { z.zu(Seite.Eintrag(e.datensatzId)) }.padding(horizontal = Abstand.rand, vertical = Abstand.s),
                    horizontalArrangement = Arrangement.spacedBy(Abstand.m),
                ) {
                    Text(e.uhrzeit, style = MaterialTheme.typography.labelLarge, modifier = Modifier.width(48.dp).padding(top = 2.dp))
                    Column(Modifier.weight(1f)) {
                        Text(e.name, style = MaterialTheme.typography.bodyLarge)
                        if (e.zusammenfassung.isNotEmpty()) Text(e.zusammenfassung, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2)
                    }
                }
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            }
        }
        item { TextButton(onClick = { tage += 60 }, modifier = Modifier.padding(Abstand.s)) { Text("Ältere Einträge laden") } }
    }
}

/** Eintrag als Seite: Frage über der Antwort, Freitexte im Wortlaut (#129, #135). */
@Composable
fun EintragSeite(z: AppZustand, id: String) {
    val d = remember(id) { z.versuch { z.kern.eintrag(id) } } ?: return
    var loeschen by remember { mutableStateOf(false) }
    Column(Modifier.fillMaxSize()) {
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
            Text("${d.zeile.tagText}, ${d.zeile.uhrzeit}", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = Abstand.s))
            if (d.alleAntworten.isEmpty()) Text("Ohne Angaben", style = MaterialTheme.typography.bodyLarge)
            for (a in d.alleAntworten) {
                val (frage, antwort) = a.indexOf(": ").let { if (it < 0) null to a else a.substring(0, it) to a.substring(it + 2) }
                Column {
                    frage?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    Text(antwort, style = MaterialTheme.typography.bodyLarge)
                }
            }
            TextButton(onClick = { loeschen = true }) { Text("Eintrag löschen", color = MaterialTheme.colorScheme.error) }
        }
        Row(Modifier.fillMaxWidth().padding(Abstand.l)) {
            FilledTonalButton(onClick = {
                val schl = d.zeile.schluessel.ifEmpty { null } ?: return@FilledTonalButton
                z.zu(Seite.Formular(schl, korrektur = d))
            }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.hauptknopf)) { Text("Korrigieren") }
        }
    }
    if (loeschen) AlertDialog(
        onDismissRequest = { loeschen = false },
        title = { Text("Eintrag löschen?") },
        text = { Text("Er verschwindet sofort aus der App. In älteren Sicherungen bleibt er höchstens 30 Tage erhalten.") },
        confirmButton = {
            Button(onClick = {
                z.versuch { z.kern.eintragLoeschen(d.zeile.datensatzId) }; z.geaendert(); loeschen = false; z.zu(Seite.Eintraege)
            }, colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error, contentColor = MaterialTheme.colorScheme.onError)) { Text("Löschen") }
        },
        dismissButton = { TextButton(onClick = { loeschen = false }) { Text("Behalten") } },
    )
}
