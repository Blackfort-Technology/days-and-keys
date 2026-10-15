// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.ProgressBarRangeInfo
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.progressBarRangeInfo
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.setProgress
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.FrageAnsicht
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlin.math.roundToLong

/** Karte auf dem Seitengrund (Startseite, Listen). Formulare stehen ohne Karten auf der Fläche. */
@Composable
fun Abschnitt(titel: String, untertitel: String? = null, inhalt: @Composable () -> Unit) {
    Card(
        Modifier.fillMaxWidth().padding(horizontal = Abstand.rand, vertical = 6.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow),
    ) {
        Column(Modifier.padding(Abstand.l), verticalArrangement = Arrangement.spacedBy(Abstand.s)) {
            if (titel.isNotEmpty()) Text(titel, style = MaterialTheme.typography.titleMedium)
            if (untertitel != null) Text(untertitel, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            inhalt()
        }
    }
}

@Composable
fun Abstand(h: Int = 12) = Spacer(Modifier.height(h.dp))

fun uhrzeit(min: Int) = "%02d:%02d".format(min / 60, min % 60)

fun zeitText(ms: Long): String =
    DateTimeFormatter.ofPattern("dd.MM.yyyy, HH:mm").format(Instant.ofEpochMilli(ms).atZone(ZoneId.systemDefault()))

fun uhrzeitAus(ms: Long): String = DateTimeFormatter.ofPattern("HH:mm").format(Instant.ofEpochMilli(ms).atZone(ZoneId.systemDefault()))

fun vorText(ms: Long): String {
    val min = (System.currentTimeMillis() - ms) / 60_000
    return when {
        min < 2 -> "gerade eben"
        min < 60 -> "vor $min Minuten"
        min < 120 -> "vor 1 Stunde"
        min < 48 * 60 -> "vor ${min / 60} Stunden"
        else -> "vor ${min / 1440} Tagen"
    }
}

@Composable
fun Symbol(id: Int, beschreibung: String? = null, modifier: Modifier = Modifier.size(24.dp)) =
    Icon(painterResource(id), beschreibung, modifier)

private fun wertText(w: Long, bipolar: Boolean) = if (bipolar && w > 0) "+$w" else "$w"
private fun sprechWert(w: Long) = if (w < 0) "minus ${-w}" else if (w > 0) "plus $w" else "null"

/**
 * Stufenleiste für Skalen mit mehr als fünf Stufen (#134, Designvorgabe 4.0). Ohne Antwort keine
 * Marke, also keine Startposition; Tippen setzt die nächste Stufe, Ziehen wandert stufenweise mit
 * Haptik, ein zweiter Tipp auf die gewählte Stufe nimmt die Antwort zurück (#73). Jeder Wert
 * sieht gleich aus (AE-27).
 */
@Composable
fun Stufenleiste(q: FrageAnsicht, wert: Long?, setzen: (Long?) -> Unit) {
    val min = q.min ?: 0
    val max = q.max ?: 10
    val n = (max - min).coerceAtLeast(1)
    val bipolar = min < 0
    val farbe = MaterialTheme.colorScheme
    val haptik = LocalHapticFeedback.current
    val messer = rememberTextMeasurer()
    val schrift = MaterialTheme.typography.labelLarge.copy(color = farbe.onPrimary)
    val blase = MaterialTheme.typography.displaySmall.copy(color = farbe.onSurface)
    val aktuell by rememberUpdatedState(wert)
    val setze by rememberUpdatedState(setzen)
    var zieh by remember { mutableStateOf<Long?>(null) }
    val rand = with(LocalDensity.current) { 22.dp.toPx() }
    fun stufe(x: Float, breite: Float): Long =
        (min + ((x - rand) / (breite - 2 * rand) * n).roundToLong()).coerceIn(min, max)
    val anker = q.anker.firstOrNull()?.text to q.anker.lastOrNull()?.text
    val mitte = if (bipolar) 0L else min + n / 2
    Column(Modifier.fillMaxWidth()) {
        Canvas(
            Modifier.fillMaxWidth().height(48.dp)
                .pointerInput(min, max) {
                    detectTapGestures { o ->
                        val s = stufe(o.x, size.width.toFloat())
                        haptik.performHapticFeedback(HapticFeedbackType.SegmentTick)
                        setze(if (s == aktuell) null else s)
                    }
                }
                .pointerInput(min, max) {
                    detectHorizontalDragGestures(
                        onDragStart = { o -> zieh = stufe(o.x, size.width.toFloat()); haptik.performHapticFeedback(HapticFeedbackType.SegmentTick) },
                        onHorizontalDrag = { c, _ ->
                            val s = stufe(c.position.x, size.width.toFloat())
                            if (s != zieh) { zieh = s; haptik.performHapticFeedback(HapticFeedbackType.SegmentTick) }
                        },
                        onDragEnd = { zieh?.let { setze(it) }; zieh = null },
                        onDragCancel = { zieh = null },
                    )
                }
                .semantics(mergeDescendants = true) {
                    contentDescription = "${q.frage} Von ${sprechWert(min)}${anker.first?.let { ", $it" } ?: ""}, bis ${sprechWert(max)}${anker.second?.let { ", $it" } ?: ""}"
                    stateDescription = wert?.let { sprechWert(it) } ?: "nicht beantwortet"
                    progressBarRangeInfo = ProgressBarRangeInfo((wert ?: mitte).toFloat(), min.toFloat()..max.toFloat(), (n - 1).toInt())
                    setProgress { v -> setze(v.roundToLong().coerceIn(min, max)); true }
                    customActions = listOf(CustomAccessibilityAction("Antwort entfernen") { setze(null); true })
                },
        ) {
            val b = size.width
            val h = size.height
            val ecke = CornerRadius(16.dp.toPx())
            drawRoundRect(farbe.surfaceContainerHighest, cornerRadius = ecke)
            drawRoundRect(farbe.outline, cornerRadius = ecke, style = Stroke(1.dp.toPx()))
            fun x(w: Long) = rand + (w - min).toFloat() / n * (b - 2 * rand)
            for (w in min..max) {
                if (bipolar && w == 0L) drawCircle(farbe.outline, 4.dp.toPx(), Offset(x(w), h / 2), style = Stroke(1.5.dp.toPx()))
                else drawCircle(farbe.outline, 2.dp.toPx(), Offset(x(w), h / 2))
            }
            val zeige = zieh ?: wert
            if (zeige != null) {
                val mx = x(zeige)
                drawCircle(farbe.primary, 20.dp.toPx(), Offset(mx, h / 2))
                val t = messer.measure(wertText(zeige, bipolar), schrift)
                drawText(t, topLeft = Offset(mx - t.size.width / 2f, h / 2 - t.size.height / 2f))
                if (zieh != null) {
                    // Wertblase über dem Finger, damit der Daumen den Wert nicht verdeckt.
                    val tb = messer.measure(wertText(zeige, bipolar), blase)
                    val bx = (mx - tb.size.width / 2f - 12f).coerceIn(0f, b - tb.size.width - 24f)
                    drawRoundRect(farbe.surfaceContainerHigh, Offset(bx, -tb.size.height - 28f), Size(tb.size.width + 24f, tb.size.height + 12f), CornerRadius(12.dp.toPx()))
                    drawText(tb, topLeft = Offset(bx + 12f, -tb.size.height - 22f))
                }
            }
        }
        if (q.anker.isNotEmpty()) Row(Modifier.fillMaxWidth().padding(top = 4.dp), horizontalArrangement = Arrangement.SpaceBetween) {
            for (an in q.anker) Text(an.text, style = MaterialTheme.typography.labelSmall, color = farbe.onSurfaceVariant)
        }
    }
}

/**
 * Stufenwahl für bis zu fünf Antworten (#134). Gewählt mit Füllung und kräftigem Rand, nicht nur
 * über Farbe. Ab 150 % Schrift oder bei mehr Antworten eine senkrechte Liste.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Stufenwahl(optionen: List<Pair<Long, String>>, gewaehlt: Long?, setzen: (Long?) -> Unit) {
    val farbe = MaterialTheme.colorScheme
    // Senkrecht bei großer Schrift, mehr als fünf Antworten oder langen Texten in engen Segmenten.
    val senkrecht = LocalDensity.current.fontScale >= 1.5f || optionen.size > 6 ||
        (optionen.size == 6 && optionen.any { it.second.length > 3 }) ||
        (optionen.size >= 4 && optionen.any { it.second.length > 11 })
    if (senkrecht) {
        Column(Modifier.fillMaxWidth()) {
            for ((w, t) in optionen) Row(
                Modifier.fillMaxWidth().heightIn(min = Abstand.ziel)
                    .selectable(selected = gewaehlt == w, role = Role.RadioButton) { setzen(if (gewaehlt == w) null else w) },
                verticalAlignment = Alignment.CenterVertically,
            ) {
                RadioButton(selected = gewaehlt == w, onClick = null)
                Text(t, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.padding(start = Abstand.s))
            }
        }
        return
    }
    SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
        optionen.forEachIndexed { i, (w, t) ->
            val an = gewaehlt == w
            SegmentedButton(
                selected = an,
                onClick = { setzen(if (an) null else w) },
                shape = SegmentedButtonDefaults.itemShape(i, optionen.size, MaterialTheme.shapes.small),
                border = BorderStroke(if (an) 2.dp else 1.dp, if (an) farbe.primary else farbe.outline),
                colors = SegmentedButtonDefaults.colors(
                    activeContainerColor = farbe.primaryContainer, activeContentColor = farbe.onPrimaryContainer,
                    inactiveContainerColor = farbe.surface, inactiveContentColor = farbe.onSurface,
                ),
                icon = {},
                modifier = Modifier.heightIn(min = Abstand.ziel),
            ) { Text(t, style = MaterialTheme.typography.labelMedium, maxLines = 2, textAlign = TextAlign.Center) }
        }
    }
}

/** Auswahlchip für Mehrfachauswahl: gewählt mit Häkchen, nicht nur über Füllung (#134). */
@Composable
fun AuswahlChip(text: String, gewaehlt: Boolean, umschalten: () -> Unit) {
    FilterChip(
        selected = gewaehlt, onClick = umschalten, label = { Text(text, style = MaterialTheme.typography.labelMedium) },
        leadingIcon = if (gewaehlt) ({ Symbol(R.drawable.sym_check, null, Modifier.size(18.dp)) }) else null,
    )
}

/** Gruppe mit Kopfzeile, auf Tipp auf- und zuklappbar (#134). */
@Composable
fun Aufklappgruppe(titel: String, zeile: String, offen: Boolean, umschalten: () -> Unit, inhalt: @Composable () -> Unit) {
    Column(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 56.dp)
                .clickable(role = Role.Button, onClickLabel = if (offen) "zuklappen" else "aufklappen") { umschalten() }
                .semantics { stateDescription = if (offen) "aufgeklappt" else "zugeklappt" }
                .padding(vertical = Abstand.s),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(titel, style = MaterialTheme.typography.titleSmall)
                Text(zeile, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Symbol(R.drawable.sym_expand_more, null, Modifier.size(24.dp).rotate(if (offen) 180f else 0f))
        }
        AnimatedVisibility(
            offen,
            enter = expandVertically(tween(Bewegung.MITTEL)) + fadeIn(tween(Bewegung.MITTEL)),
            exit = shrinkVertically(tween(Bewegung.MITTEL)) + fadeOut(tween(Bewegung.MITTEL)),
        ) { Column(verticalArrangement = Arrangement.spacedBy(Abstand.frage)) { inhalt() } }
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}

/** Fester Bereich am unteren Rand mit der Hauptaktion (#134). */
@Composable
fun Aktionsleiste(text: String, onClick: () -> Unit) {
    Surface(color = MaterialTheme.colorScheme.surface) {
        Column(Modifier.navigationBarsPadding()) {
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Button(onClick = onClick, modifier = Modifier.fillMaxWidth().padding(Abstand.l).heightIn(min = Abstand.hauptknopf)) { Text(text) }
        }
    }
}

/** Zeitchip: „Jetzt“ oder die gewählte Zeit, öffnet die Zeitwahl (#134). */
@Composable
fun Zeitchip(text: String, onClick: () -> Unit) {
    AssistChip(onClick = onClick, label = { Text(text) }, leadingIcon = { Symbol(R.drawable.sym_schedule, null, Modifier.size(18.dp)) })
}

@Composable
fun Kopf(titel: String, zusatz: String? = null) {
    Column(Modifier.fillMaxWidth().padding(top = Abstand.xl, bottom = Abstand.s)) {
        Text(titel, style = MaterialTheme.typography.titleSmall)
        zusatz?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
    }
}

@Composable
fun Klickzeile(titel: String, zeile: String? = null, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().heightIn(min = 56.dp).clickable(role = Role.Button) { onClick() }.padding(horizontal = Abstand.l, vertical = Abstand.s),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(titel, style = MaterialTheme.typography.bodyLarge)
            zeile?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        }
        Box { Symbol(R.drawable.sym_chevron_right) }
    }
}
