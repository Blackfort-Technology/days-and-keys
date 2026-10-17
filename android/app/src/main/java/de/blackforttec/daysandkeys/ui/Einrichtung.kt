// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.view.View
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.DakFehler
import de.blackforttec.daysandkeys.kern.Schluessel
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
            label = { Text(stringResource(R.string.einrichtung_wort, nr)) },
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

/** Die 24 Wörter in zwei Spalten: drei passen bei 360 dp Breite nicht, Nummer und Wort brachen um (#120). */
@Composable
fun WortListe(woerter: List<String>) {
    for (zeile in woerter.chunked(2).withIndex()) {
        Row(Modifier.fillMaxWidth()) {
            for ((i, w) in zeile.value.withIndex()) {
                Row(Modifier.weight(1f)) {
                    Text("${zeile.index * 2 + i + 1}".padStart(2) + ".", style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.width(32.dp))
                    Text(w, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.SemiBold, maxLines = 1, softWrap = false)
                }
            }
        }
    }
}

/**
 * Recovery-Wörter ausstellen und abfragen, erst zusammen mit der Sicherung (#165). `rahmen` setzt
 * Titel und Text passend zur Umgebung (Einrichtungsschritt oder Unterseite von Mehr).
 */
@Composable
fun WoerterEinrichten(
    z: AppZustand,
    rahmen: @Composable (titel: String, text: String, inhalt: @Composable () -> Unit) -> Unit,
    fertig: () -> Unit,
) {
    val neu = remember { z.versuch { z.kern.woerterEinrichtenBeginnen() } } ?: return
    var abfrage by remember { mutableStateOf(false) }
    var abgeschlossen by remember { mutableStateOf(false) }
    val probe = remember { mutableStateListOf("", "", "") }
    val probeFalsch = stringResource(R.string.einrichtung_probe_falsch)
    // Wer abbricht, lässt keine Wörter im Kern liegen.
    DisposableEffect(Unit) { onDispose { if (!abgeschlossen) z.kern.woerterEinrichtenVerwerfen() } }
    BackHandler(enabled = abfrage) { abfrage = false }
    SensibleAnsicht()
    if (!abfrage) {
        rahmen(stringResource(R.string.einrichtung_woerter_titel), stringResource(R.string.einrichtung_woerter_text)) {
            WortListe(neu.woerter)
            Button(onClick = { abfrage = true }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_aufgeschrieben)) }
        }
    } else {
        rahmen(stringResource(R.string.einrichtung_probe_titel), stringResource(R.string.einrichtung_probe_text)) {
            for ((i, pos) in neu.probe.withIndex()) WortFeld(z, pos.toInt() + 1, probe[i]) { probe[i] = it }
            Button(onClick = {
                if (z.versuch { z.kern.woerterEinrichtenProbePruefen(probe.toList()) } != true) { z.meldung = probeFalsch; return@Button }
                if (z.versuch { z.kern.woerterEinrichtenAbschliessen(probe.toList()) } != null) { abgeschlossen = true; fertig() }
            }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.allgemein_pruefen)) }
            TextButton(onClick = { abfrage = false }) { Text(stringResource(R.string.einrichtung_woerter_zeigen)) }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun EinrichtungSeite(z: AppZustand) {
    // Fünf Schritte ohne Recovery-Wörter; die kommen mit der Sicherung, jetzt oder später (#165).
    var schritt by remember { mutableIntStateOf(0) }
    var tageswechsel by remember { mutableIntStateOf(4) }
    var schluessel by remember { mutableStateOf<Schluessel?>(null) }
    var sicherungJetzt by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val ordnerWahl = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        if (uri != null) { z.app.ordner.uebernehmen(uri); z.sichernImHintergrund { z.meldung = it } }
        schritt = 4
    }
    val erlaubnis = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { schritt = 5 }

    BackHandler(enabled = schritt == 3 && sicherungJetzt) { sicherungJetzt = false }
    if (schritt == 0) { Begruessung(z) { schritt = 1 }; return }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(vertical = 16.dp)) {
        when (schritt) {
            1 -> Schritt(1, stringResource(R.string.einrichtung_tag_titel), stringResource(R.string.einrichtung_tag_text)) {
                Stufenwahl(listOf(0L to "0", 2L to "2", 3L to "3", 4L to "4", 5L to "5", 6L to "6"), tageswechsel.toLong()) { it?.let { w -> tageswechsel = w.toInt() } }
                Text(stringResource(R.string.einrichtung_uhr), style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Button(onClick = {
                    schluessel = z.versuch { z.kern.einrichtungBeginnenOhneWoerter(tageswechsel * 60) }
                    if (schluessel != null) schritt = 2
                }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.allgemein_weiter)) }
            }
            2 -> Schritt(2, stringResource(R.string.einrichtung_schutz_titel), stringResource(R.string.einrichtung_schutz_text)) {
                Button(onClick = {
                    val k = schluessel!!
                    scope.launch {
                        try {
                            z.arbeitet = true
                            val umh = z.app.schluessel.umhuellen(z.activity, k.generation, k.kk, k.kkH)
                            z.kern.einrichtungAbschliessen(emptyList(), umh, null)
                            schluessel = null
                            z.nachEntsperren(navigieren = false)
                            schritt = 3
                        } catch (x: Schluesselspeicher.Abgebrochen) {
                            z.meldung = x.message
                        } catch (x: DakFehler.Fehler) {
                            z.meldung = x.meldung
                        } finally {
                            z.arbeitet = false
                        }
                    }
                }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_schuetzen)) }
            }
            3 -> if (sicherungJetzt) {
                WoerterEinrichten(z, { titel, text, inhalt -> Schritt(3, titel, text, inhalt) }) { ordnerWahl.launch(null) }
            } else {
                Schritt(3, stringResource(R.string.einrichtung_sicherung_titel), stringResource(R.string.einrichtung_sicherung_text)) {
                    Button(onClick = { sicherungJetzt = true }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_sicherung_jetzt)) }
                    TextButton(onClick = { schritt = 4 }) { Text(stringResource(R.string.einrichtung_spaeter)) }
                }
            }
            4 -> Schritt(4, stringResource(R.string.allgemein_erinnerungen), stringResource(R.string.einrichtung_erinnerungen_text)) {
                Button(onClick = {
                    if (Build.VERSION.SDK_INT >= 33) erlaubnis.launch(Manifest.permission.POST_NOTIFICATIONS) else schritt = 5
                }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_benachrichtigungen_erlauben)) }
                TextButton(onClick = {
                    // „Ohne Erinnerungen“ schaltet die voreingestellten Erinnerungen ab (#149).
                    z.versuch { z.kern.einstellungen() }?.let { e -> z.versuch { z.kern.einstellungenSpeichern(e.copy(erinnerungen = e.erinnerungen.map { it.copy(aktiv = false) })) } }
                    schritt = 5
                }) { Text(stringResource(R.string.einrichtung_ohne_erinnerungen)) }
            }
            else -> Schritt(5, stringResource(R.string.einrichtung_fertig), if (Build.MANUFACTURER.equals("samsung", ignoreCase = true)) stringResource(R.string.einrichtung_samsung) else null) {
                if (!de.blackforttec.daysandkeys.plattform.Planer.exaktErlaubt(z.activity)) {
                    OutlinedButton(onClick = {
                        z.activity.startActivity(Intent(Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM, Uri.parse("package:" + z.activity.packageName)))
                    }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_puenktlich)) }
                }
                Button(onClick = { z.planen(); z.zu(Seite.Start) }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_zur_startseite)) }
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
    val wiederhergestellt = stringResource(R.string.einrichtung_wiederhergestellt)
    val ordnerWahl = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri: Uri? ->
        if (uri != null) {
            baum = uri
            z.app.ordner.uebernehmen(uri)
            kandidaten = z.versuch { z.kern.kandidaten(z.app.ordner.sicherungen(uri), z.app.ordner.marker(uri)) }
        }
    }
    Column(Modifier.verticalScroll(rememberScrollState()).padding(vertical = 16.dp)) {
        if (kandidaten == null) {
            Abschnitt(stringResource(R.string.einrichtung_wiederherstellen_titel), stringResource(R.string.einrichtung_wiederherstellen_text)) {
                for (i in 0 until 24) WortFeld(z, i + 1, woerter[i]) { woerter[i] = it }
                Button(onClick = {
                    if (z.versuch { z.kern.wiederherstellungBeginnen(woerter.toList()) } != null) ordnerWahl.launch(null)
                }, modifier = Modifier.fillMaxWidth()) { Text(stringResource(R.string.einrichtung_ordner_sicherungen)) }
                TextButton(onClick = { z.zu(Seite.Einrichtung) }) { Text(stringResource(R.string.allgemein_zurueck)) }
            }
        } else {
            val k = kandidaten!!
            val gueltig = k.filter { it.gueltig }
            Abschnitt(stringResource(R.string.einrichtung_gefunden), if (gueltig.size > 1 && gueltig.none { it.empfohlen }) stringResource(R.string.einrichtung_zwei_staende) else null) {
                if (k.isEmpty()) Text(stringResource(R.string.einrichtung_keine_sicherung))
                for (c in k.sortedByDescending { it.erstelltMs }) {
                    val text = if (c.gueltig) stringResource(if (c.empfohlen) R.string.einrichtung_sicherung_nr_neueste else R.string.einrichtung_sicherung_nr, c.sicherungNr.toLong(), zeitText(c.erstelltMs)) else "${c.name}: ${c.grund}"
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
                                z.meldung = wiederhergestellt
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
                TextButton(onClick = { kandidaten = null }) { Text(stringResource(R.string.einrichtung_anderer_ordner)) }
            }
        }
    }
}

/** Ein Schritt der Einrichtung als ganze Seite mit Fortschritt zur Orientierung (#136, Vorgabe 4.1). */
@Composable
fun Schritt(nr: Int, titel: String, text: String?, inhalt: @Composable () -> Unit) {
    Column(Modifier.fillMaxWidth().padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.m)) {
        LinearProgressIndicator(progress = { nr / 5f }, modifier = Modifier.fillMaxWidth().padding(top = Abstand.s))
        Text(stringResource(R.string.einrichtung_schritt, nr), style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(titel, style = MaterialTheme.typography.headlineSmall)
        text?.let { Text(it, style = MaterialTheme.typography.bodyLarge) }
        Abstand(4)
        inhalt()
    }
}

/** Begrüßung ohne Karte: Symbol, drei Zusagen, der Hinweis zum Geräte-Code gleich groß (Vorgabe 4.1). */
@Composable
fun Begruessung(z: AppZustand, weiter: () -> Unit) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = Abstand.rand), verticalArrangement = Arrangement.spacedBy(Abstand.l)) {
        Abstand(48)
        Image(painterResource(R.drawable.ic_vordergrund), null, Modifier.size(96.dp).background(MaterialTheme.colorScheme.primary, CircleShape))
        Text(stringResource(R.string.einrichtung_willkommen), style = MaterialTheme.typography.headlineSmall)
        for ((symbol, text) in listOf(
            R.drawable.sym_lock to stringResource(R.string.einrichtung_zusage_verschluesselt),
            R.drawable.sym_close to stringResource(R.string.einrichtung_zusage_offline),
            R.drawable.sym_schedule to stringResource(R.string.einrichtung_zusage_festhalten),
        )) Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Abstand.m)) {
            Icon(painterResource(symbol), null, tint = MaterialTheme.colorScheme.primary)
            Text(text, style = MaterialTheme.typography.bodyLarge)
        }
        Hinweisfeld(stringResource(R.string.einrichtung_code_hinweis))
        Abstand(16)
        Button(onClick = weiter, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.hauptknopf)) { Text(stringResource(R.string.einrichtung_einrichten)) }
        OutlinedButton(onClick = { z.zu(Seite.Wiederherstellung) }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.ziel)) { Text(stringResource(R.string.einrichtung_aus_sicherung)) }
        TextButton(onClick = { z.zu(Seite.Hilfe) }) { Text(stringResource(R.string.allgemein_hilfe_krise)) }
        Abstand(24)
    }
}
