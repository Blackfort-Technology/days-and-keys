// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.view.View
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.kern.DakFehler
import de.blackforttec.daysandkeys.kern.Einrichtung
import de.blackforttec.daysandkeys.plattform.Schluesselspeicher
import kotlinx.coroutines.launch

/** Bedienungshilfen dürfen diese Ansicht nicht auslesen (#69, ab Android 14). */
@Composable
fun SensibleAnsicht() {
    val v = LocalView.current
    DisposableEffect(v) {
        if (Build.VERSION.SDK_INT >= 34) v.setAccessibilityDataSensitive(View.ACCESSIBILITY_DATA_SENSITIVE_YES)
        onDispose { if (Build.VERSION.SDK_INT >= 34) v.setAccessibilityDataSensitive(View.ACCESSIBILITY_DATA_SENSITIVE_AUTO) }
    }
}

/** Ein Wort eingeben: Tastatur ohne Lernen und Vorschläge, Auswahl aus der Wortliste (#69). */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun WortFeld(z: AppZustand, nr: Int, wert: String, aendern: (String) -> Unit) {
    Column {
        OutlinedTextField(
            value = wert,
            onValueChange = { aendern(it.trim().lowercase()) },
            label = { Text("Wort $nr") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, autoCorrectEnabled = false),
            modifier = Modifier.fillMaxWidth(),
        )
        val vorschlaege = if (wert.length >= 2) z.kern.wortVorschlaege(wert) else emptyList()
        if (vorschlaege.isNotEmpty() && vorschlaege != listOf(wert)) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (w in vorschlaege) AssistChip(onClick = { aendern(w) }, label = { Text(w) })
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun EinrichtungSeite(z: AppZustand) {
    var schritt by remember { mutableIntStateOf(0) }
    var tageswechsel by remember { mutableIntStateOf(4) }
    var einrichtung by remember { mutableStateOf<Einrichtung?>(null) }
    val probe = remember { mutableStateListOf("", "", "") }
    val scope = rememberCoroutineScope()
    val ordnerWahl = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        if (uri != null) { z.app.ordner.uebernehmen(uri); z.sichernImHintergrund { z.meldung = it } }
        schritt = 6
    }
    val erlaubnis = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { schritt = 7 }

    Column(Modifier.verticalScroll(rememberScrollState()).padding(vertical = 16.dp)) {
        when (schritt) {
            0 -> Abschnitt("Willkommen bei Days & Keys") {
                Text("Hier hältst du fest, wie es dir geht, so oft du willst: mehrmals am Tag, jederzeit. Dazu Medikamente, Ereignisse und auf Wunsch einen Tagesabschluss.")
                Text("Alles bleibt verschlüsselt auf diesem Gerät. Die App hat keinen Internetzugang und schickt nichts an einen Server.")
                Text("Wichtig: Wer den Code dieses Geräts kennt, kann die Einträge lesen.", style = MaterialTheme.typography.bodyMedium)
                Button(onClick = { schritt = 1 }, modifier = Modifier.fillMaxWidth()) { Text("Neu einrichten") }
                OutlinedButton(onClick = { z.zu(Seite.Wiederherstellung) }, modifier = Modifier.fillMaxWidth()) { Text("Aus einer Sicherung wiederherstellen") }
                TextButton(onClick = { z.zu(Seite.Hilfe) }) { Text("Hilfe in einer Krise") }
            }
            1 -> Abschnitt("Wann beginnt dein Tag?", "Einträge vor dieser Uhrzeit zählen zum Vortag. Später nicht mehr änderbar für schon erfasste Einträge.") {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    for (h in listOf(0, 2, 3, 4, 5, 6)) FilterChip(selected = tageswechsel == h, onClick = { tageswechsel = h }, label = { Text("$h:00 Uhr") })
                }
                Button(onClick = {
                    einrichtung = z.versuch { z.kern.einrichtungBeginnen(tageswechsel * 60) }
                    if (einrichtung != null) schritt = 2
                }, modifier = Modifier.fillMaxWidth()) { Text("Weiter") }
            }
            2 -> {
                SensibleAnsicht()
                val e = einrichtung!!
                Abschnitt("Deine 24 Recovery-Wörter", "Schreib sie auf Papier und bewahre sie sicher auf. Nur mit ihnen lassen sich deine Einträge auf einem neuen Gerät wiederherstellen. Kein Bildschirmfoto, keine Notiz-App.") {
                    // Zwei Spalten: drei passen bei 360 dp Breite nicht, Nummer und Wort brachen um (#120).
                    for (zeile in e.woerter.chunked(2).withIndex()) {
                        Row(Modifier.fillMaxWidth()) {
                            for ((i, w) in zeile.value.withIndex()) {
                                Text(
                                    "${(zeile.index * 2 + i + 1).toString().padStart(2)}. $w", fontFamily = FontFamily.Monospace,
                                    maxLines = 1, softWrap = false, modifier = Modifier.weight(1f),
                                )
                            }
                        }
                    }
                    Button(onClick = { schritt = 3 }, modifier = Modifier.fillMaxWidth()) { Text("Ich habe sie aufgeschrieben") }
                }
            }
            3 -> {
                SensibleAnsicht()
                val e = einrichtung!!
                Abschnitt("Kurze Probe", "Gib diese drei Wörter von deinem Zettel ein.") {
                    for ((i, pos) in e.probe.withIndex()) WortFeld(z, pos.toInt() + 1, probe[i]) { probe[i] = it }
                    Button(onClick = {
                        if (z.versuch { z.kern.einrichtungProbePruefen(probe.toList()) } == true) schritt = 4
                        else z.meldung = "Die Wörter passen nicht. Schau noch einmal auf deinen Zettel."
                    }, modifier = Modifier.fillMaxWidth()) { Text("Prüfen") }
                    TextButton(onClick = { schritt = 2 }) { Text("Wörter noch einmal zeigen") }
                }
            }
            4 -> Abschnitt("Mit dem Gerät schützen", "Days & Keys öffnet sich künftig mit Fingerabdruck, Gesicht oder Geräte-Code. Der Schlüssel liegt im Sicherheitschip des Geräts.") {
                Button(onClick = {
                    val e = einrichtung!!
                    scope.launch {
                        try {
                            z.arbeitet = true
                            val umh = z.app.schluessel.umhuellen(z.activity, e.schluessel.generation, e.schluessel.kk, e.schluessel.kkH)
                            z.kern.einrichtungAbschliessen(probe.toList(), umh, null)
                            einrichtung = null
                            z.nachEntsperren(navigieren = false)
                            schritt = 5
                        } catch (x: Schluesselspeicher.Abgebrochen) {
                            z.meldung = x.message
                        } catch (x: DakFehler.Fehler) {
                            z.meldung = x.meldung
                        } finally {
                            z.arbeitet = false
                        }
                    }
                }, modifier = Modifier.fillMaxWidth()) { Text("Schützen und einrichten") }
            }
            5 -> Abschnitt("Automatische Sicherung", "Wähle einen Ordner, in den die App selbst verschlüsselte Sicherungen schreibt, zum Beispiel einen Cloud-Ordner. Ohne Sicherung gehen bei Verlust des Geräts alle Einträge verloren.") {
                Button(onClick = { ordnerWahl.launch(null) }, modifier = Modifier.fillMaxWidth()) { Text("Ordner wählen") }
                TextButton(onClick = { schritt = 6 }) { Text("Später") }
            }
            6 -> Abschnitt("Erinnerungen", "Erinnerungen haben neutrale Texte ohne Medikamentennamen. Die App ersetzt keinen Medikamentenwecker.") {
                Button(onClick = {
                    if (Build.VERSION.SDK_INT >= 33) erlaubnis.launch(Manifest.permission.POST_NOTIFICATIONS) else schritt = 7
                }, modifier = Modifier.fillMaxWidth()) { Text("Benachrichtigungen erlauben") }
                TextButton(onClick = { schritt = 7 }) { Text("Ohne Erinnerungen") }
            }
            else -> Abschnitt("Fertig", "Auf Samsung-Geräten: in den Akku-Einstellungen die App auf „Nicht eingeschränkt\" setzen, sonst kommen Erinnerungen unzuverlässig.") {
                if (!de.blackforttec.daysandkeys.plattform.Planer.exaktErlaubt(z.activity)) {
                    OutlinedButton(onClick = {
                        z.activity.startActivity(Intent(Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM, Uri.parse("package:" + z.activity.packageName)))
                    }, modifier = Modifier.fillMaxWidth()) { Text("Pünktliche Erinnerungen erlauben") }
                }
                Button(onClick = { z.planen(); z.zu(Seite.Start) }, modifier = Modifier.fillMaxWidth()) { Text("Los geht's") }
            }
        }
    }
}

@Composable
fun WiederherstellungSeite(z: AppZustand) {
    SensibleAnsicht()
    val woerter = remember { mutableStateListOf(*Array(24) { "" }) }
    var kandidaten by remember { mutableStateOf<List<de.blackforttec.daysandkeys.kern.Kandidat>?>(null) }
    var baum by remember { mutableStateOf<Uri?>(null) }
    val scope = rememberCoroutineScope()
    val ordnerWahl = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        if (uri != null) {
            baum = uri
            z.app.ordner.uebernehmen(uri)
            kandidaten = z.versuch { z.kern.kandidaten(z.app.ordner.sicherungen(uri), z.app.ordner.marker(uri)) }
        }
    }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(vertical = 16.dp)) {
        if (kandidaten == null) {
            Abschnitt("Wiederherstellen", "Gib deine 24 Wörter ein und wähle dann den Ordner mit den Sicherungen.") {
                for (i in 0 until 24) WortFeld(z, i + 1, woerter[i]) { woerter[i] = it }
                Button(onClick = {
                    if (z.versuch { z.kern.wiederherstellungBeginnen(woerter.toList()) } != null) ordnerWahl.launch(null)
                }, modifier = Modifier.fillMaxWidth()) { Text("Ordner mit Sicherungen wählen") }
                TextButton(onClick = { z.zu(Seite.Einrichtung) }) { Text("Zurück") }
            }
        } else {
            val k = kandidaten!!
            val gueltig = k.filter { it.gueltig }
            Abschnitt("Gefundene Sicherungen", if (gueltig.size > 1 && gueltig.none { it.empfohlen }) "Es gibt zwei verschiedene Stände. Wähle den richtigen." else null) {
                if (k.isEmpty()) Text("In diesem Ordner liegt keine Sicherung.")
                for (c in k.sortedByDescending { it.erstelltMs }) {
                    val text = if (c.gueltig) "Nr. ${c.sicherungNr} vom ${zeitText(c.erstelltMs)}" + (if (c.empfohlen) " (neueste)" else "") else "${c.name}: ${c.grund}"
                    if (c.gueltig) Button(onClick = {
                        val u = baum ?: return@Button
                        scope.launch {
                            try {
                                z.arbeitet = true
                                val w = z.kern.wiederherstellen(z.app.ordner.sicherungen(u), z.app.ordner.marker(u), c.index)
                                val umh = z.app.schluessel.umhuellen(z.activity, w.schluessel.generation, w.schluessel.kk, w.schluessel.kkH)
                                z.kern.vorbereitetesAbschliessen(umh, null)
                                z.app.ordner.schreiben(w.markerName, w.marker, u)
                                z.app.schluessel.aufraeumen(w.schluessel.generation)
                                z.nachEntsperren()
                                z.meldung = "Wiederhergestellt. Entziehe dem alten Gerät den Zugang zum Sicherungsordner, falls es noch existiert."
                            } catch (x: Schluesselspeicher.Abgebrochen) {
                                z.meldung = x.message
                            } catch (x: DakFehler.Fehler) {
                                z.meldung = x.meldung
                            } finally {
                                z.arbeitet = false
                            }
                        }
                    }, modifier = Modifier.fillMaxWidth()) { Text(text) } else Text(text, color = MaterialTheme.colorScheme.secondary)
                }
                TextButton(onClick = { kandidaten = null }) { Text("Anderen Ordner wählen") }
            }
        }
    }
}
