// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import de.blackforttec.daysandkeys.kern.Hilfe

/** Hilfe ohne Entsperren (#71, AE-20): Nummern öffnen nur das Wählfeld, die App ruft nicht selbst an. */
@Composable
fun HilfeSeite(z: AppZustand) {
    val h: Hilfe? = remember(z.seite) { z.versuch { z.kern.hilfe() } }
    fun waehlen(nummer: String) {
        z.activity.startActivity(Intent(Intent.ACTION_DIAL, Uri.parse("tel:" + nummer.replace(" ", ""))))
    }
    Column(Modifier.verticalScroll(rememberScrollState())) {
        Abschnitt("Hilfe", h?.hinweis) {
            for (n in h?.nummern.orEmpty()) {
                Row(Modifier.fillMaxWidth()) {
                    Button(onClick = { waehlen(n.nummer) }, modifier = Modifier.fillMaxWidth()) {
                        Text("${n.name}: ${n.nummer}")
                    }
                }
                Text(n.hinweis, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.secondary)
            }
        }
        h?.krisenplan?.let { plan ->
            Abschnitt("Mein Plan für Tiefs") { Text(plan) }
        }
        if (h != null && h.vertrauenspersonen.isNotEmpty()) {
            Abschnitt("Meine Vertrauenspersonen") {
                for (v in h.vertrauenspersonen) {
                    OutlinedButton(onClick = { waehlen(v.telefon) }, modifier = Modifier.fillMaxWidth()) { Text("${v.name}: ${v.telefon}") }
                }
            }
        }
        if (!z.entsperrt()) {
            Abschnitt("") { OutlinedButton(onClick = { z.zu(if (z.kern.lage() is de.blackforttec.daysandkeys.kern.KernLage.NichtEingerichtet) Seite.Einrichtung else Seite.Sperre) }) { Text("Zurück") } }
        }
    }
}
