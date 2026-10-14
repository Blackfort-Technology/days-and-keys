// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

/** Gesperrt sichtbar sind nur Entsperren und die Hilfe (AE-20). */
@Composable
fun SperrSeite(z: AppZustand) {
    LaunchedEffect(Unit) { if (z.ziel != null) z.entsperren() }
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("Days & Keys", style = MaterialTheme.typography.headlineMedium)
        Text("Gesperrt", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.secondary)
        Button(onClick = { z.entsperren() }, modifier = Modifier.fillMaxWidth()) { Text("Entsperren") }
        OutlinedButton(onClick = { z.zu(Seite.Hilfe) }, modifier = Modifier.fillMaxWidth()) { Text("Hilfe in einer Krise") }
        Text(
            "Zum Entsperren genügt Fingerabdruck, Gesicht oder der Geräte-Code.",
            style = MaterialTheme.typography.bodySmall, textAlign = TextAlign.Center, color = MaterialTheme.colorScheme.secondary,
        )
    }
}
