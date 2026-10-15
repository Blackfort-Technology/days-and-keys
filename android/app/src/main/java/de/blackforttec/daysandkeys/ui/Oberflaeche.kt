// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.sp
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.KernLage

/** Formulare und Präparate sind ganze Seiten ohne Leiste, mit Schließen links (Vorgabe 5). */
private fun istFormular(s: Seite) = s is Seite.Formular || s is Seite.Medikament

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
        fun zurueckZiel(): Seite = when (s) {
            is Seite.Medikament -> Seite.Medikamente
            is Seite.Hilfe -> when {
                z.entsperrt() -> Seite.Start
                z.kern.lage() is KernLage.NichtEingerichtet -> Seite.Einrichtung
                else -> Seite.Sperre
            }
            else -> Seite.Start
        }
        BackHandler(enabled = innen && s != Seite.Start) { z.zu(zurueckZiel()) }
        val formularName = remember(s) { (s as? Seite.Formular)?.let { f -> z.versuch { z.kern.formular(f.schluessel) }?.name } }
        Scaffold(
            snackbarHost = { SnackbarHost(snack) },
            containerColor = if (istFormular(s)) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.background,
            topBar = {
                if (innen) {
                    val titel = when (s) {
                        Seite.Start -> "Heute"
                        Seite.Verlauf, Seite.Eintraege -> "Verlauf"
                        Seite.Medikamente -> "Medikamente"
                        is Seite.Medikament -> if (s.p == null) "Neues Präparat" else "Präparat bearbeiten"
                        Seite.Einstellungen -> "Mehr"
                        Seite.Hilfe -> "Hilfe"
                        is Seite.Formular -> (if (s.korrektur != null) "Korrektur: " else "") + (formularName ?: "")
                        else -> ""
                    }
                    TopAppBar(
                        title = {
                            Column {
                                Text(titel, style = MaterialTheme.typography.headlineSmall)
                                if (s == Seite.Start) z.start?.let { Text(it.heuteLang, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                            }
                        },
                        navigationIcon = {
                            when {
                                istFormular(s) -> IconButton(onClick = { z.zu(zurueckZiel()) }) { Symbol(R.drawable.sym_close, "Schließen") }
                                s == Seite.Hilfe -> IconButton(onClick = { z.zu(zurueckZiel()) }) { Symbol(R.drawable.sym_arrow_back, "Zurück") }
                            }
                        },
                        actions = {
                            if (s != Seite.Hilfe) TextButton(onClick = { z.zu(Seite.Hilfe) }) { Text("Hilfe") }
                            if (z.entsperrt() && s != Seite.Hilfe && !istFormular(s)) IconButton(onClick = { z.sperren() }) { Symbol(R.drawable.sym_lock, "Sperren") }
                        },
                        colors = TopAppBarDefaults.topAppBarColors(
                            containerColor = if (istFormular(s)) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.background,
                        ),
                    )
                }
            },
            bottomBar = { if (innen && z.entsperrt() && !istFormular(s)) Leiste(s) { z.zu(it) } },
        ) { pad ->
            Box(Modifier.padding(pad)) {
                when (s) {
                    Seite.Sperre -> SperrSeite(z)
                    Seite.Einrichtung -> EinrichtungSeite(z)
                    Seite.Wiederherstellung -> WiederherstellungSeite(z)
                    Seite.Start -> StartSeite(z)
                    is Seite.Formular -> FormularSeite(z, s)
                    Seite.Verlauf -> VerlaufReiter(z, eintraege = false)
                    Seite.Eintraege -> VerlaufReiter(z, eintraege = true)
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

/** Vier Reiter mit Symbol und Namen (Vorgabe 5); ab 150 % Schrift nur der gewählte mit Namen. */
@Composable
private fun Leiste(s: Seite, zu: (Seite) -> Unit) {
    val eintraege = listOf(
        Triple("Heute", Seite.Start, R.drawable.sym_wb_twilight),
        Triple("Verlauf", Seite.Verlauf, R.drawable.sym_timeline),
        Triple("Medikamente", Seite.Medikamente, R.drawable.sym_medication),
        Triple("Mehr", Seite.Einstellungen, R.drawable.sym_more_horiz),
    )
    val gross = LocalDensity.current.fontScale >= 1.5f
    NavigationBar(containerColor = MaterialTheme.colorScheme.surfaceContainer) {
        for ((titel, ziel, symbol) in eintraege) {
            val gewaehlt = s == ziel || (ziel == Seite.Medikamente && s is Seite.Medikament) || (ziel == Seite.Verlauf && s == Seite.Eintraege)
            NavigationBarItem(
                selected = gewaehlt,
                onClick = { zu(ziel) },
                icon = { Symbol(symbol, if (gross) titel else null) },
                label = { Text(titel, style = MaterialTheme.typography.labelMedium.copy(fontSize = 13.sp), maxLines = 1, softWrap = false) },
                alwaysShowLabel = !gross,
                colors = NavigationBarItemDefaults.colors(
                    indicatorColor = MaterialTheme.colorScheme.primaryContainer,
                    selectedIconColor = MaterialTheme.colorScheme.onPrimaryContainer,
                    selectedTextColor = MaterialTheme.colorScheme.onSurface,
                    unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                ),
            )
        }
    }
}
