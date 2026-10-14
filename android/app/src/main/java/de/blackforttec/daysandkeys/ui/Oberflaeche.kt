// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource

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
                is Seite.Hilfe -> if (z.entsperrt()) Seite.Start else Seite.Sperre
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
                if (innen && z.entsperrt()) NavigationBar {
                    val eintraege = listOf(
                        "Heute" to Seite.Start, "Verlauf" to Seite.Verlauf, "Einträge" to Seite.Eintraege,
                        "Medikamente" to Seite.Medikamente, "Mehr" to Seite.Einstellungen,
                    )
                    for ((titel, ziel) in eintraege) {
                        NavigationBarItem(
                            selected = s == ziel || (ziel == Seite.Medikamente && s is Seite.Medikament),
                            onClick = { z.zu(ziel) },
                            icon = {},
                            label = { Text(titel) },
                        )
                    }
                }
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
