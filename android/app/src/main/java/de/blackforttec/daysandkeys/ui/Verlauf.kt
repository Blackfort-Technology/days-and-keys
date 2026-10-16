// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
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
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.Feldart
import de.blackforttec.daysandkeys.kern.Richtung
import de.blackforttec.daysandkeys.kern.Verlauf
import java.time.LocalDate

/**
 * Verlauf (#43, #74, #105): jeder Eintrag als Punkt, auch mehrere am Tag. Grundniveau als
 * gleitender Mittelwert über Tageswerte, Lücken bleiben Lücken, keine Farbwertung.
 */
@OptIn(ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)
@Composable
fun VerlaufSeite(z: AppZustand) {
    // Fragen nach Bezug gruppiert, Stimmung oben (#135, Vorgabe 4.7).
    val gruppen = remember {
        listOf("status_jetzt" to "Jetzt", "tagesabschluss" to "Tagesabschluss", "aufgewacht" to "Schlaf am Morgen").map { (schl, titel) ->
            titel to z.versuch { z.kern.formular(schl) }?.gruppen.orEmpty().flatMap { it.fragen }
                .filter { it.feldart in setOf(Feldart.SKALA, Feldart.STUFEN, Feldart.ZAHL, Feldart.DAUER) }
        }
    }
    val fragen = remember { gruppen.flatMap { it.second }.distinctBy { it.schluessel } }
    var frage by remember { mutableStateOf("stimmung") }
    var tage by remember { mutableIntStateOf(28) }
    var mittel by remember { mutableIntStateOf(7) }
    var wahl by remember { mutableStateOf(false) }
    var lesen by remember { mutableStateOf(false) }
    var liste by remember { mutableStateOf(false) }
    val heute = remember { z.versuch { z.kern.heute() } ?: 0L }
    val v: Verlauf? = remember(frage, tage) { z.versuch { z.kern.verlauf(frage, heute - tage + 1, heute) } }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
        val gewaehlt = fragen.firstOrNull { it.schluessel == frage }
        Klickzeile(gewaehlt?.frage ?: frage, "Andere Frage wählen") { wahl = true }
        Stufenwahl(listOf(28L to "4 Wochen", 91L to "3 Monate", 365L to "1 Jahr"), tage.toLong()) { it?.let { w -> tage = w.toInt() } }
        if (v != null) {
            if (v.punkte.isEmpty()) {
                Text("In diesem Zeitraum gibt es noch keine Werte für diese Frage.", style = MaterialTheme.typography.bodyLarge, modifier = Modifier.padding(vertical = Abstand.xl))
            } else {
                Diagramm(v, heute - tage + 1, heute, mittel, dauer = gewaehlt?.feldart == Feldart.DAUER)
                Stufenwahl(listOf(7L to "Grundniveau 7 Tage", 28L to "28 Tage"), mittel.toLong()) { it?.let { w -> mittel = w.toInt() } }
            }
            Text(v.hinweis + (if (v.einnahmenMs.isNotEmpty()) " · ${v.einnahmenMs.size} Einnahmen" else ""), style = MaterialTheme.typography.bodyMedium)
            if (v.marker.isNotEmpty()) Text("Marker: " + v.marker.groupBy { it.name }.map { "${it.key} (${it.value.size})" }.joinToString(", "), style = MaterialTheme.typography.bodyMedium)
            Aufklappgruppe("So liest du das Diagramm", "Tageswert, Mittelwerte, Lücken", lesen, { lesen = !lesen }) {
                Text(v.erklaerung, style = MaterialTheme.typography.bodyMedium)
            }
            Aufklappgruppe("Tageswerte als Liste", "${v.tage.size} Tage mit Wert", liste, { liste = !liste }) {
                for (t in v.tage.reversed()) Row(Modifier.fillMaxWidth()) {
                    Text(t.datum, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
                    Text("${t.letzter}" + if (t.anzahl > 1u) "  (${t.anzahl} Einträge, ${t.min} bis ${t.max})" else "", style = MaterialTheme.typography.bodyMedium)
                }
            }
            if (v.vergleiche.isNotEmpty()) {
                Kopf("Grundniveau mit und ohne", v.vergleiche.first().hinweis)
                fun t(x: Double?) = x?.let { "%+.1f".format(it) } ?: "-"
                for (f in v.vergleiche) Text("${f.faktor}: an ${f.tageMit} Tagen mit ${t(f.mittelMit)}, an ${f.tageOhne} Tagen ohne ${t(f.mittelOhne)}", style = MaterialTheme.typography.bodyMedium)
            }
        }
        Abstand(24)
    }
    if (wahl) ModalBottomSheet(onDismissRequest = { wahl = false }) {
        Column(Modifier.verticalScroll(rememberScrollState()).padding(bottom = Abstand.xl)) {
            for ((titel, qs) in gruppen) if (qs.isNotEmpty()) {
                Text(titel, style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(horizontal = Abstand.rand, vertical = Abstand.s))
                // Kurzname zuerst, damit Folgefragen wie „Wie lange ungefähr?“ ihren Bezug behalten (#153).
                for (q in qs) Column(Modifier.fillMaxWidth().heightIn(min = Abstand.ziel).clickable { frage = q.schluessel; wahl = false }.padding(horizontal = Abstand.rand, vertical = Abstand.s), verticalArrangement = Arrangement.Center) {
                    Text(q.kurz ?: q.frage, style = MaterialTheme.typography.bodyLarge)
                    if (q.kurz != null) Text(q.frage, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
fun Diagramm(v: Verlauf, von: Long, bis: Long, mittel: Int, dauer: Boolean = false) {
    val farbePunkt = MaterialTheme.colorScheme.primary
    val farbeLinie = MaterialTheme.colorScheme.tertiary
    val farbeAchse = MaterialTheme.colorScheme.secondary
    val schrift = MaterialTheme.typography.labelSmall.copy(color = farbeAchse)
    val messer = rememberTextMeasurer()
    // Dauern in Stunden und nur so hoch wie nötig, mindestens 10 Std. (#153).
    val min = if (dauer) 0f else (v.min ?: v.punkte.minOfOrNull { it.wert } ?: 0).toFloat()
    val max = if (dauer) (((v.punkte.maxOfOrNull { it.wert } ?: 0) / 60 + 1) * 60).coerceAtLeast(600).toFloat()
    else (v.max ?: v.punkte.maxOfOrNull { it.wert } ?: 10).toFloat().let { if (it == min) min + 1 else it }
    val reihe = if (mittel == 7) v.gleitend7 else v.gleitend28
    Canvas(Modifier.fillMaxWidth().height(240.dp).semantics { contentDescription = "Verlauf mit ${v.punkte.size} Einträgen an ${v.tageMitWert} von ${v.tageGesamt} Tagen" }) {
        // Ränder für die Beschriftung: Werte links, Datum unten (#127).
        val links = (if (dauer) 48.dp else 30.dp).toPx(); val unten = 18.dp.toPx(); val oben = 6.dp.toPx()
        val breite = size.width - links; val hoehe = size.height - unten - oben
        val tage = (bis - von + 1).coerceAtLeast(1)
        fun x(tag: Double) = (links + (tag - von) / tage * breite).toFloat()
        fun y(w: Double) = (oben + hoehe - (w - min) / (max - min) * hoehe).toFloat()
        fun wert(w: Double) = if (dauer) "${(w / 60).toInt()} Std." else if (v.richtung == Richtung.BIPOLAR && w > 0) "+${w.toInt()}" else "${w.toInt()}"
        // Hilfslinien und Werte: unten, Mitte (bei bipolaren Skalen die Null), oben
        val mitte = if (v.richtung == Richtung.BIPOLAR) 0.0 else ((min + max) / 2.0)
        for (w in listOf(min.toDouble(), mitte, max.toDouble())) {
            val stark = v.richtung == Richtung.BIPOLAR && w == 0.0
            drawLine(farbeAchse.copy(alpha = if (stark) 0.9f else 0.25f), Offset(links, y(w)), Offset(size.width, y(w)), strokeWidth = if (stark) 2f else 1f)
            val t = messer.measure(wert(w), schrift)
            drawText(t, topLeft = Offset(links - t.size.width - 6f, y(w) - t.size.height / 2f))
        }
        // Datum etwa wöchentlich, vom letzten Tag aus gezählt
        val schritt = if (tage <= 35) 7 else if (tage <= 100) 28 else 91
        var d = bis
        while (d >= von) {
            val datum = LocalDate.ofEpochDay(d)
            val t = messer.measure("%02d.%02d.".format(datum.dayOfMonth, datum.monthValue), schrift)
            val mx = x(d + 0.5)
            drawLine(farbeAchse.copy(alpha = 0.25f), Offset(mx, oben), Offset(mx, oben + hoehe), strokeWidth = 1f)
            drawText(t, topLeft = Offset((mx - t.size.width / 2f).coerceIn(links, size.width - t.size.width), oben + hoehe + 3f))
            d -= schritt
        }
        for (m in v.marker) drawLine(farbeAchse.copy(alpha = 0.4f), Offset(x(m.lebenstag + 0.5), oben), Offset(x(m.lebenstag + 0.5), oben + hoehe), pathEffect = PathEffect.dashPathEffect(floatArrayOf(8f, 8f)))
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
    FlowRow(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("● Eintrag", style = MaterialTheme.typography.labelSmall, color = farbePunkt)
        Text("━ Grundniveau ($mittel Tage)", style = MaterialTheme.typography.labelSmall, color = farbeLinie)
        Text("${v.anker.firstOrNull()?.text ?: ""} … ${v.anker.lastOrNull()?.text ?: ""}", style = MaterialTheme.typography.labelSmall, color = farbeAchse)
    }
}

/** Reiter „Verlauf“: Diagramm und Einträge mit einem Umschalter oben (#134, Vorgabe 4.7). */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VerlaufReiter(z: AppZustand, eintraege: Boolean) {
    Column(Modifier.fillMaxSize()) {
        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth().padding(horizontal = Abstand.rand, vertical = Abstand.s)) {
            listOf("Diagramm" to Seite.Verlauf, "Einträge" to Seite.Eintraege).forEachIndexed { i, (t, ziel) ->
                SegmentedButton(
                    selected = (ziel == Seite.Eintraege) == eintraege,
                    onClick = { z.zu(ziel) },
                    shape = SegmentedButtonDefaults.itemShape(i, 2, MaterialTheme.shapes.small),
                    icon = {},
                    colors = SegmentedButtonDefaults.colors(activeContainerColor = MaterialTheme.colorScheme.primaryContainer, activeContentColor = MaterialTheme.colorScheme.onPrimaryContainer),
                ) { Text(t, style = MaterialTheme.typography.labelMedium) }
            }
        }
        Box(Modifier.weight(1f)) { if (eintraege) EintraegeSeite(z) else VerlaufSeite(z) }
    }
}
