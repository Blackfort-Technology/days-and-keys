// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.Feldart
import de.blackforttec.daysandkeys.kern.Richtung
import de.blackforttec.daysandkeys.kern.Verlauf

/**
 * Verlauf (#43, #74, #105): jeder Eintrag als Punkt, auch mehrere am Tag. Grundniveau als
 * gleitender Mittelwert über Tageswerte, Lücken bleiben Lücken, keine Farbwertung.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun VerlaufSeite(z: AppZustand) {
    val fragen = remember {
        listOf("status_jetzt", "tagesabschluss").flatMap { s -> z.versuch { z.kern.formular(s) }?.gruppen.orEmpty().flatMap { it.fragen } }
            .filter { it.feldart in setOf(Feldart.SKALA, Feldart.STUFEN, Feldart.ZAHL, Feldart.DAUER) }
            .distinctBy { it.schluessel }
    }
    var frage by remember { mutableStateOf("stimmung") }
    var tage by remember { mutableIntStateOf(28) }
    var mittel by remember { mutableIntStateOf(7) }
    val heute = remember { z.versuch { z.kern.heute() } ?: 0L }
    val v: Verlauf? = remember(frage, tage) { z.versuch { z.kern.verlauf(frage, heute - tage + 1, heute) } }
    Column(Modifier.verticalScroll(rememberScrollState())) {
        Abschnitt("Verlauf") {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (q in fragen) FilterChip(selected = frage == q.schluessel, onClick = { frage = q.schluessel }, label = { Text(q.frage.removeSuffix("?").take(28)) })
            }
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for ((t, n) in listOf(28 to "4 Wochen", 91 to "3 Monate", 365 to "1 Jahr")) FilterChip(selected = tage == t, onClick = { tage = t }, label = { Text(n) })
                for (m in listOf(7, 28)) FilterChip(selected = mittel == m, onClick = { mittel = m }, label = { Text("Mittel $m Tage") })
            }
        }
        if (v != null) {
            Abschnitt(v.frage, v.hinweis) {
                Diagramm(v, heute - tage + 1, heute, mittel)
                if (v.marker.isNotEmpty()) Text("Marker: " + v.marker.groupBy { it.name }.map { "${it.key} (${it.value.size})" }.joinToString(", "), style = MaterialTheme.typography.bodySmall)
                if (v.einnahmenMs.isNotEmpty()) Text("Einnahmen im Zeitraum: ${v.einnahmenMs.size}", style = MaterialTheme.typography.bodySmall)
            }
            if (frage == "stimmung") {
                Abschnitt("Grundniveau mit und ohne", "Rein beschreibend, mit Fallzahl. Kein Beleg für eine Ursache.") {
                    for ((schl, name) in listOf("tag_bewegung" to "Bewegung", "tag_tageslicht" to "Tageslicht draußen")) {
                        val f = z.versuch { z.kern.faktorvergleich(schl, heute - tage + 1, heute) } ?: continue
                        fun t(x: Double?) = x?.let { "%+.1f".format(it) } ?: "-"
                        Text("$name: an ${f.tageMit} Tagen mit ${t(f.mittelMit)}, an ${f.tageOhne} Tagen ohne ${t(f.mittelOhne)}")
                    }
                }
            }
        }
    }
}

@Composable
fun Diagramm(v: Verlauf, von: Long, bis: Long, mittel: Int) {
    val farbePunkt = MaterialTheme.colorScheme.primary
    val farbeLinie = MaterialTheme.colorScheme.tertiary
    val farbeAchse = MaterialTheme.colorScheme.secondary
    val min = (v.min ?: v.punkte.minOfOrNull { it.wert } ?: 0).toFloat()
    val max = (v.max ?: v.punkte.maxOfOrNull { it.wert } ?: 10).toFloat().let { if (it == min) min + 1 else it }
    val reihe = if (mittel == 7) v.gleitend7 else v.gleitend28
    Canvas(Modifier.fillMaxWidth().height(220.dp).semantics { contentDescription = "Verlauf mit ${v.punkte.size} Einträgen an ${v.tageMitWert} von ${v.tageGesamt} Tagen" }) {
        val breite = size.width; val hoehe = size.height
        val tage = (bis - von + 1).coerceAtLeast(1)
        fun x(tag: Double) = ((tag - von) / tage * breite).toFloat()
        fun y(w: Double) = (hoehe - (w - min) / (max - min) * hoehe).toFloat()
        // Nulllinie bei bipolaren Skalen, sonst Grundlinie
        val null0 = if (v.richtung == Richtung.BIPOLAR) 0.0 else min.toDouble()
        drawLine(farbeAchse, Offset(0f, y(null0)), Offset(breite, y(null0)), strokeWidth = 2f)
        for (m in v.marker) drawLine(farbeAchse.copy(alpha = 0.4f), Offset(x(m.lebenstag + 0.5), 0f), Offset(x(m.lebenstag + 0.5), hoehe), pathEffect = PathEffect.dashPathEffect(floatArrayOf(8f, 8f)))
        // Jeder Eintrag ein Punkt, innerhalb des Tages nach Uhrzeit verteilt.
        for (p in v.punkte) drawCircle(farbePunkt, radius = 6f, center = Offset(x(p.lebenstag + p.uhrzeitMin / 1440.0), y(p.wert.toDouble())))
        // Grundniveau: nur über Tage mit genügend Werten, Lücken bleiben offen.
        var vorher: Offset? = null
        for (m in reihe) {
            val w = m.wert
            if (w == null) { vorher = null; continue }
            val o = Offset(x(m.lebenstag + 0.5), y(w))
            vorher?.let { drawLine(farbeLinie, it, o, strokeWidth = 4f) }
            vorher = o
        }
    }
    Text("${v.anker.firstOrNull()?.text ?: ""} … ${v.anker.lastOrNull()?.text ?: ""}", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.secondary)
}
