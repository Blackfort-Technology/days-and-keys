// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

@Composable
fun Abschnitt(titel: String, untertitel: String? = null, inhalt: @Composable () -> Unit) {
    Card(
        Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
    ) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            if (titel.isNotEmpty()) Text(titel, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
            if (untertitel != null) Text(untertitel, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.secondary)
            inhalt()
        }
    }
}

@Composable
fun Abstand(h: Int = 12) = Spacer(Modifier.height(h.dp))

fun uhrzeit(min: Int) = "%02d:%02d".format(min / 60, min % 60)

fun zeitText(ms: Long): String =
    DateTimeFormatter.ofPattern("dd.MM.yyyy, HH:mm").format(Instant.ofEpochMilli(ms).atZone(ZoneId.systemDefault()))

fun vorText(ms: Long): String {
    val min = (System.currentTimeMillis() - ms) / 60_000
    return when {
        min < 2 -> "gerade eben"
        min < 60 -> "vor $min Minuten"
        min < 48 * 60 -> "vor ${min / 60} Stunden"
        else -> "vor ${min / 1440} Tagen"
    }
}
