// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.TextAutoSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import de.blackforttec.daysandkeys.kern.KernLage

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DaysAndKeysOberflaeche(z: AppZustand) {
    Thema {
        val snack = remember { SnackbarHostState() }
        LaunchedEffect(z.meldung) {
            z.meldung?.let { snack.showSnackbar(it); z.meldung = null }
        }
        val s = z.seite
        val innen = s !is Seite.Sperre && s !is Seite.Einrichtung && s !is Seite.Wiederherstellung
        BackHandler(enabled = innen && s != Seite.Start) {
            z.zu(when (s) {
                is Seite.Medikament -> Seite.Medikamente
                is Seite.Hilfe -> when {
                    z.entsperrt() -> Seite.Start
                    z.kern.lage() is KernLage.NichtEingerichtet -> Seite.Einrichtung
                    else -> Seite.Sperre
                }
                else -> Seite.Start
            })
        }
        Scaffold(
            snackbarHost = { SnackbarHost(snack) },
            topBar = {
                if (innen) TopAppBar(
                    title = { Text("Days & Keys") },
                    actions = {
                        TextButton(onClick = { z.zu(Seite.Hilfe) }) { Text("Hilfe") }
                        if (z.entsperrt()) TextButton(onClick = { z.sperren() }) { Text("Sperren") }
                    },
                )
            },
            bottomBar = {
                if (innen && z.entsperrt()) Leiste(s) { z.zu(it) }
            },
        ) { pad ->
            Box(Modifier.padding(pad)) {
                when (s) {
                    Seite.Sperre -> SperrSeite(z)
                    Seite.Einrichtung -> EinrichtungSeite(z)
                    Seite.Wiederherstellung -> WiederherstellungSeite(z)
                    Seite.Start -> StartSeite(z)
                    is Seite.Formular -> FormularSeite(z, s)
                    Seite.Verlauf -> VerlaufSeite(z)
                    Seite.Eintraege -> EintraegeSeite(z)
                    Seite.Medikamente -> MedikamenteSeite(z)
                    is Seite.Medikament -> MedikamentSeite(z, s.p)
                    Seite.Einstellungen -> EinstellungenSeite(z)
                    Seite.Hilfe -> HilfeSeite(z)
                }
                if (z.arbeitet) CircularProgressIndicator(Modifier.align(Alignment.Center))
            }
        }
    }
}

/**
 * Untere Leiste in den Farben der App: nur Text, ohne Umbruch, die gewählte Seite hinterlegt (#120).
 * Die Material-Leiste reserviert Platz für ein Symbol und bricht lange Beschriftungen um.
 */
@Composable
private fun Leiste(s: Seite, zu: (Seite) -> Unit) {
    val eintraege = listOf(
        "Heute" to Seite.Start, "Verlauf" to Seite.Verlauf, "Einträge" to Seite.Eintraege,
        "Medikamente" to Seite.Medikamente, "Mehr" to Seite.Einstellungen,
    )
    Surface(color = MaterialTheme.colorScheme.surface, tonalElevation = 2.dp) {
        Row(
            Modifier.fillMaxWidth().navigationBarsPadding().padding(horizontal = 4.dp, vertical = 6.dp),
            horizontalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            for ((titel, ziel) in eintraege) {
                val gewaehlt = s == ziel || (ziel == Seite.Medikamente && s is Seite.Medikament)
                Box(
                    Modifier.weight(1f).heightIn(min = 48.dp).clip(RoundedCornerShape(16.dp))
                        .background(if (gewaehlt) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent)
                        .selectable(selected = gewaehlt, role = Role.Tab, onClick = { zu(ziel) }),
                    contentAlignment = Alignment.Center,
                ) {
                    // Schrumpft bei schmalen Geräten oder großer Systemschrift, statt abzuschneiden.
                    BasicText(
                        titel, maxLines = 1, softWrap = false,
                        style = MaterialTheme.typography.labelMedium.copy(
                            fontWeight = if (gewaehlt) FontWeight.SemiBold else FontWeight.Normal,
                            color = if (gewaehlt) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.secondary,
                        ),
                        autoSize = TextAutoSize.StepBased(minFontSize = 8.sp, maxFontSize = 12.sp, stepSize = 0.5.sp),
                        modifier = Modifier.padding(horizontal = 2.dp),
                    )
                }
            }
        }
    }
}
