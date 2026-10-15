// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.lifecycleScope
import de.blackforttec.daysandkeys.DaysAndKeysApp
import de.blackforttec.daysandkeys.MainActivity
import de.blackforttec.daysandkeys.kern.AntwortEingabe
import de.blackforttec.daysandkeys.kern.DakFehler
import de.blackforttec.daysandkeys.kern.EintragDetail
import de.blackforttec.daysandkeys.kern.KernLage
import de.blackforttec.daysandkeys.kern.PraeparatDaten
import de.blackforttec.daysandkeys.kern.Startseite
import de.blackforttec.daysandkeys.plattform.Planer
import de.blackforttec.daysandkeys.plattform.Schluesselspeicher
import de.blackforttec.daysandkeys.plattform.Sicherungsordner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

sealed class Seite {
    data object Sperre : Seite()
    data object Einrichtung : Seite()
    data object Wiederherstellung : Seite()
    data object Start : Seite()
    /** `vorbelegt`: Antworten, die die Person schon gegeben hat (Schnellstatus, #134); keine Vorbelegung aus Altwerten. */
    data class Formular(
        val schluessel: String, val entwurf: String? = null, val korrektur: EintragDetail? = null, val praeparat: String? = null,
        val vorbelegt: List<AntwortEingabe> = emptyList(),
    ) : Seite()
    data object Verlauf : Seite()
    data object Eintraege : Seite()
    /** Eintrag im Detail als eigene Seite (#135). */
    data class Eintrag(val id: String) : Seite()
    data object Medikamente : Seite()
    data class Medikament(val p: PraeparatDaten?) : Seite()
    data object Einstellungen : Seite()
    data object Hilfe : Seite()
}

/** Zustand der Oberfläche. Alle Inhalte kommen fertig aus dem Kern (AE-06). */
class AppZustand(val app: DaysAndKeysApp, val activity: MainActivity) {
    val kern get() = app.kern
    var seite by mutableStateOf<Seite>(Seite.Sperre)
    var meldung by mutableStateOf<String?>(null)
    var start by mutableStateOf<Startseite?>(null)
    var arbeitet by mutableStateOf(false)
    var krisenhinweisGesehen by mutableStateOf(false)
    /** Gerät wurde durch ein anderes ersetzt; die Sperrseite erklärt das dauerhaft (#150). */
    var ersetzt by mutableStateOf(false)
    /** Gewählte, noch nicht festgehaltene Stimmung auf Heute, übersteht den Reiterwechsel (#151). */
    var schnellwert by mutableStateOf<Long?>(null)
    var ziel: String? = null
    private var sperrzeitMs: Long = 60_000
    val scope get() = activity.lifecycleScope

    init {
        // Ein Neuaufbau der Activity darf einen entsperrten Kern nicht hinter eine Sperrseite
        // stellen, deren Knopf dann nichts tut (#148).
        seite = when (kern.lage()) {
            is KernLage.NichtEingerichtet -> Seite.Einrichtung
            is KernLage.Entsperrt -> Seite.Start
            else -> Seite.Sperre
        }
        if (seite == Seite.Start) nachEntsperren(navigieren = false)
    }

    fun entsperrt() = kern.lage() is KernLage.Entsperrt

    /** Fehler des Kerns als verständlicher Text; der Code hilft bei Rückmeldungen. */
    fun <T> versuch(block: () -> T): T? = try {
        block()
    } catch (e: DakFehler.Fehler) {
        meldung = "${e.meldung}"; null
    } catch (e: Schluesselspeicher.Abgebrochen) {
        meldung = e.message; null
    } catch (e: Exception) {
        meldung = "Unerwarteter Fehler: ${e.javaClass.simpleName}"; null
    }

    fun entsperren() {
        scope.launch {
            if (kern.lage() is KernLage.Entsperrt) { nachEntsperren(); return@launch }
            val l = kern.lage() as? KernLage.Gesperrt ?: return@launch
            val umh = l.kkUmhuellung ?: run { meldung = "Kein geschützter Schlüssel gefunden."; return@launch }
            try {
                arbeitet = true
                val (a, b) = app.schluessel.enthuellen(activity, l.generation, umh)
                withContext(Dispatchers.Default) { kern.entsperren(a, b) }
                a.fill(0); b.fill(0)
                nachEntsperren()
            } catch (e: Schluesselspeicher.Abgebrochen) {
                meldung = e.message
            } catch (e: DakFehler.Fehler) {
                meldung = e.meldung
            } finally {
                arbeitet = false
            }
        }
    }

    fun nachEntsperren(navigieren: Boolean = true) {
        versuch { sperrzeitMs = kern.einstellungen().sperrzeitS.toLong() * 1000 }
        neuLaden()
        planen()
        if (start?.sicherungNoetig == true) sichernImHintergrund()
        if (!navigieren) return
        val z = ziel
        ziel = null
        seite = if (z != null && z != "einnahme") Seite.Formular(z) else Seite.Start
    }

    fun zielSetzen(z: String) {
        if (entsperrt()) seite = if (z != "einnahme") Seite.Formular(z) else Seite.Start else ziel = z
    }

    fun sperren() {
        kern.sperren()
        start = null
        seite = Seite.Sperre
    }

    fun imHintergrund() {
        if (entsperrt() && start?.sicherungNoetig == true) sichernImHintergrund()
    }

    fun zurueck(abwesendMs: Long) {
        if (entsperrt() && abwesendMs > sperrzeitMs) sperren()
    }

    fun neuLaden() {
        start = versuch { kern.startseite() }
    }

    fun planen() {
        versuch { Planer.setzen(activity, kern.benachrichtigungen(7u, 60u)) }
    }

    /** Nach jeder Änderung: Startseite und Erinnerungen neu. */
    fun geaendert() {
        neuLaden()
        planen()
    }

    fun sichernImHintergrund(danach: (String) -> Unit = {}) {
        scope.launch {
            val ergebnis = withContext(Dispatchers.IO) {
                try {
                    when (val r = app.ordner.sichern(kern)) {
                        is Sicherungsordner.Ergebnis.Gesichert -> "Gesichert"
                        Sicherungsordner.Ergebnis.KeinOrdner -> "Noch kein Sicherungsordner gewählt"
                        Sicherungsordner.Ergebnis.Ersetzt -> "Dieses Gerät wurde durch ein anderes ersetzt und sichert nicht mehr."
                    }
                } catch (e: DakFehler.Fehler) {
                    "Sicherung fehlgeschlagen: ${e.meldung}"
                } catch (e: Exception) {
                    "Sicherung fehlgeschlagen: ${e.javaClass.simpleName}"
                }
            }
            if (ergebnis.startsWith("Dieses Gerät")) { ersetzt = true; sperren() }
            else { if (entsperrt()) neuLaden(); danach(ergebnis) }
        }
    }

    fun zu(s: Seite) {
        seite = s
    }
}
