// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.Hilfe

/** Hilfe ohne Entsperren (#71, AE-20): Nummern öffnen nur das Wählfeld, die App ruft nicht selbst an. */
/** Hilfe nach dem Muster eines persönlichen Krisenplans (#136, Vorgabe 4.11), auch gesperrt. */
@Composable
fun HilfeSeite(z: AppZustand) {
    val h: Hilfe? = remember(z.seite) { z.versuch { z.kern.hilfe() } }
    fun waehlen(nummer: String) {
        z.activity.startActivity(Intent(Intent.ACTION_DIAL, Uri.parse("tel:" + nummer.replace(" ", ""))))
    }
    val nummern = h?.nummern.orEmpty()
    val notruf = nummern.firstOrNull { it.nummer == "112" }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
        h?.hinweis?.let { Text(it, style = MaterialTheme.typography.bodyLarge) }
        notruf?.let { n ->
            Button(onClick = { waehlen(n.nummer) }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.hauptknopf).semantics { contentDescription = "${n.name}, ${n.nummer}, ${n.hinweis}, Wählfeld öffnen" }) {
                Symbol(R.drawable.sym_call, null); Text("  ${n.name} ${n.nummer} · ${n.hinweis}")
            }
        }
        Kopf("Mein Plan für Tiefs")
        val plan = h?.krisenplan
        if (plan != null) Text(
            plan, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onTertiaryContainer,
            modifier = Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.tertiaryContainer, MaterialTheme.shapes.medium).padding(Abstand.l),
        ) else Text(
            if (z.entsperrt()) "Unter Mehr › Mein Plan für Tiefs kannst du eigene Schritte aufschreiben. Sie stehen dann hier, auch ohne Entsperren." else "Noch kein eigener Plan aufgeschrieben.",
            style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Kopf("Menschen, denen ich vertraue")
        if (!z.entsperrt()) Text("Deine Vertrauenspersonen siehst du nach dem Entsperren.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        else if (h?.vertrauenspersonen.isNullOrEmpty()) Text("Noch keine eingetragen (unter Mehr › Vertrauenspersonen).", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        else for (v in h!!.vertrauenspersonen) Nummernzeile(v.name, v.telefon) { waehlen(v.telefon) }
        // Gleiche Hinweise bilden eine Gruppe; der Hinweis steht als Überschrift, nicht zwischen Knöpfen.
        for ((hinweis, gruppe) in nummern.filter { it != notruf }.groupBy { it.hinweis }) {
            Kopf(hinweis.replaceFirstChar { it.uppercase() })
            for (n in gruppe) Nummernzeile(n.name, n.nummer) { waehlen(n.nummer) }
        }
        Text("Die Nummern öffnen nur das Wählfeld, angerufen wird erst, wenn du es dort bestätigst.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(vertical = Abstand.xl))
    }
}

@Composable
private fun Nummernzeile(name: String, nummer: String, waehlen: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().heightIn(min = 56.dp).clickable(role = Role.Button) { waehlen() }
            .semantics(mergeDescendants = true) { contentDescription = "$name, $nummer, Wählfeld öffnen" },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(name, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Text(nummer, style = MaterialTheme.typography.titleMedium, maxLines = 1, softWrap = false)
        }
        Symbol(R.drawable.sym_call, null)
    }
}
