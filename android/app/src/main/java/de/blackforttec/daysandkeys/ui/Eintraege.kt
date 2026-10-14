// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
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
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.EintragZeile

@Composable
fun EintraegeSeite(z: AppZustand) {
    var stand by remember { mutableIntStateOf(0) }
    val heute = remember { z.versuch { z.kern.heute() } ?: 0L }
    val liste = remember(stand) { z.versuch { z.kern.eintraege(heute - 60, heute) }.orEmpty() }
    var gewaehlt by remember { mutableStateOf<EintragZeile?>(null) }
    LazyColumn {
        item { Text("Einträge der letzten 60 Tage", style = MaterialTheme.typography.titleMedium, modifier = Modifier.padding(16.dp)) }
        if (liste.isEmpty()) item { Text("Noch keine Einträge.", modifier = Modifier.padding(16.dp)) }
        itemsIndexed(liste) { i, e ->
            Column(Modifier.fillMaxWidth().clickable { gewaehlt = e }.padding(horizontal = 16.dp, vertical = 8.dp)) {
                if (i == 0 || liste[i - 1].datum != e.datum) { Text(e.datum, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.secondary) }
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(e.uhrzeit, style = MaterialTheme.typography.bodyMedium)
                    Column {
                        Text(e.name, style = MaterialTheme.typography.bodyLarge)
                        if (e.zusammenfassung.isNotEmpty()) Text(e.zusammenfassung, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
            HorizontalDivider()
        }
    }
    gewaehlt?.let { e ->
        var loeschen by remember { mutableStateOf(false) }
        AlertDialog(
            onDismissRequest = { gewaehlt = null },
            title = { Text("${e.name}, ${e.datum} ${e.uhrzeit}") },
            text = { Text(if (loeschen) "Wirklich löschen? Aus dem Bestand ist der Eintrag sofort weg; in älteren Sicherungen steht er noch, bis sie nach höchstens 30 Tagen aus der Aufbewahrung fallen." else e.zusammenfassung.ifEmpty { "Ohne Angaben" }) },
            confirmButton = {
                if (loeschen) Button(onClick = { z.versuch { z.kern.eintragLoeschen(e.datensatzId) }; z.geaendert(); gewaehlt = null; stand++ }) { Text("Löschen") }
                else Button(onClick = {
                    val d = z.versuch { z.kern.eintrag(e.datensatzId) } ?: return@Button
                    val schl = if (e.istEreignis) z.versuch { z.kern.startseite() }?.ereignisse?.firstOrNull { it.formularId == e.formularId }?.schluessel
                        ?: if (e.name == "Tablette genommen") "einnahme" else null
                    else formularSchluessel(z, e.formularId)
                    if (schl != null) z.zu(Seite.Formular(schl, korrektur = d))
                    gewaehlt = null
                }) { Text("Korrigieren") }
            },
            dismissButton = {
                if (!loeschen) TextButton(onClick = { loeschen = true }) { Text("Löschen …") } else TextButton(onClick = { gewaehlt = null }) { Text("Abbrechen") }
            },
        )
    }
}
