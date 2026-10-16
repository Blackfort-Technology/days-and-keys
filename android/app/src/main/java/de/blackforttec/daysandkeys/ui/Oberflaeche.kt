// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.activity.compose.BackHandler
import androidx.activity.compose.LocalOnBackPressedDispatcherOwner
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.NavigationRailItemDefaults
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
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
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
        // Querformat auf dem Telefon: Reiter an den linken Rand, die Höhe gehört dem Inhalt (#155).
        val konf = LocalConfiguration.current
        val seitlich = konf.screenWidthDp > konf.screenHeightDp && konf.screenHeightDp < 480
        val innen = s !is Seite.Sperre && s !is Seite.Einrichtung && s !is Seite.Wiederherstellung
        fun zurueckZiel(): Seite = when (s) {
            is Seite.Medikament -> Seite.Medikamente
            is Seite.Eintrag -> Seite.Eintraege
            is Seite.Hilfe -> when {
                z.entsperrt() -> Seite.Start
                z.kern.lage() is KernLage.NichtEingerichtet -> Seite.Einrichtung
                else -> Seite.Sperre
            }
            else -> Seite.Start
        }
        BackHandler(enabled = innen && s != Seite.Start) { z.zu(zurueckZiel()) }
        val zurueck = LocalOnBackPressedDispatcherOwner.current?.onBackPressedDispatcher
        val formularName = remember(s) {
            when (s) {
                is Seite.Formular -> z.versuch { z.kern.formular(s.schluessel) }?.name
                is Seite.Eintrag -> z.versuch { z.kern.eintrag(s.id) }?.zeile?.name
                else -> null
            }
        }
        val zeigeLeiste = innen && z.entsperrt() && !istFormular(s)
        Row {
            if (seitlich && zeigeLeiste) Leiste(s, seitlich = true) { z.zu(it) }
            Scaffold(
                // Die Leiste hat den linken Rand schon eingerückt, der Inhalt nicht noch einmal.
                modifier = Modifier.weight(1f).then(
                    if (seitlich && zeigeLeiste) Modifier.consumeWindowInsets(WindowInsets.safeDrawing.only(WindowInsetsSides.Start)) else Modifier,
                ),
                snackbarHost = { SnackbarHost(snack) },
                containerColor = if (istFormular(s)) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.background,
                topBar = {
                    if (innen) {
                        val titel = when (s) {
                            Seite.Start -> stringResource(R.string.oberflaeche_heute)
                            Seite.Verlauf, Seite.Eintraege -> stringResource(R.string.oberflaeche_verlauf)
                            is Seite.Eintrag -> formularName ?: stringResource(R.string.oberflaeche_eintrag)
                            Seite.Medikamente -> stringResource(R.string.oberflaeche_medikamente)
                            is Seite.Medikament -> if (s.p == null) stringResource(R.string.oberflaeche_praeparat_neu) else stringResource(R.string.oberflaeche_praeparat_bearbeiten)
                            Seite.Einstellungen -> stringResource(R.string.oberflaeche_mehr)
                            Seite.Hilfe -> stringResource(R.string.allgemein_hilfe)
                            is Seite.Formular -> if (s.korrektur != null) stringResource(R.string.oberflaeche_korrektur, formularName ?: "") else (formularName ?: "")
                            else -> ""
                        }
                        TopAppBar(
                            title = {
                                Column {
                                    Text(titel, style = if (istFormular(s) || s is Seite.Eintrag) MaterialTheme.typography.titleLarge else MaterialTheme.typography.headlineSmall, maxLines = 1)
                                    if (s == Seite.Start) z.start?.let { Text(it.heuteLang, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                            },
                            navigationIcon = {
                                when {
                                    // Wie die Systemgeste, damit Rückfragen der Seite greifen (ungespeicherte Änderungen).
                                    istFormular(s) -> IconButton(onClick = { zurueck?.onBackPressed() }) { Symbol(R.drawable.sym_close, stringResource(R.string.oberflaeche_schliessen)) }
                                    s == Seite.Hilfe || s is Seite.Eintrag -> IconButton(onClick = { zurueck?.onBackPressed() }) { Symbol(R.drawable.sym_arrow_back, stringResource(R.string.allgemein_zurueck)) }
                                }
                            },
                            actions = {
                                if (s != Seite.Hilfe) TextButton(onClick = { z.zu(Seite.Hilfe) }) { Text(stringResource(R.string.allgemein_hilfe)) }
                                if (z.entsperrt() && s != Seite.Hilfe && !istFormular(s)) IconButton(onClick = { z.sperren() }) { Symbol(R.drawable.sym_lock, stringResource(R.string.oberflaeche_sperren)) }
                            },
                            colors = TopAppBarDefaults.topAppBarColors(
                                containerColor = if (istFormular(s)) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.background,
                            ),
                        )
                    }
                },
                bottomBar = { if (!seitlich && zeigeLeiste) Leiste(s) { z.zu(it) } },
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
                        is Seite.Eintrag -> EintragSeite(z, s.id)
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
}

/** Vier Reiter mit Symbol und Namen (Vorgabe 5); ab 150 % Schrift nur der gewählte mit Namen. */
@Composable
private fun Leiste(s: Seite, seitlich: Boolean = false, zu: (Seite) -> Unit) {
    val eintraege = listOf(
        Triple(stringResource(R.string.oberflaeche_heute), Seite.Start, R.drawable.sym_wb_twilight),
        Triple(stringResource(R.string.oberflaeche_verlauf), Seite.Verlauf, R.drawable.sym_timeline),
        Triple(stringResource(R.string.oberflaeche_medikamente), Seite.Medikamente, R.drawable.sym_medication),
        Triple(stringResource(R.string.oberflaeche_mehr), Seite.Einstellungen, R.drawable.sym_more_horiz),
    )
    val gross = LocalDensity.current.fontScale >= 1.5f
    fun gewaehlt(ziel: Seite) = s == ziel || (ziel == Seite.Medikamente && s is Seite.Medikament) || (ziel == Seite.Verlauf && (s == Seite.Eintraege || s is Seite.Eintrag))
    if (seitlich) {
        NavigationRail(
            Modifier.fillMaxHeight(),
            containerColor = MaterialTheme.colorScheme.surfaceContainer,
            // Kameraaussparung im Querformat liegt am Rand der Leiste.
            windowInsets = WindowInsets.safeDrawing.only(WindowInsetsSides.Vertical + WindowInsetsSides.Start),
        ) {
            for ((titel, ziel, symbol) in eintraege) NavigationRailItem(
                selected = gewaehlt(ziel),
                onClick = { zu(ziel) },
                icon = { Symbol(symbol, if (gross) titel else null) },
                label = { Text(titel, style = MaterialTheme.typography.labelMedium.copy(fontSize = 13.sp), maxLines = 1, softWrap = false) },
                alwaysShowLabel = !gross,
                colors = NavigationRailItemDefaults.colors(
                    indicatorColor = MaterialTheme.colorScheme.primaryContainer,
                    selectedIconColor = MaterialTheme.colorScheme.onPrimaryContainer,
                    selectedTextColor = MaterialTheme.colorScheme.onSurface,
                    unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                ),
            )
        }
        return
    }
    NavigationBar(containerColor = MaterialTheme.colorScheme.surfaceContainer) {
        for ((titel, ziel, symbol) in eintraege) {
            NavigationBarItem(
                selected = gewaehlt(ziel),
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
