// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys

import de.blackforttec.daysandkeys.kern.AntwortEingabe
import de.blackforttec.daysandkeys.kern.AntwortZustand
import de.blackforttec.daysandkeys.kern.DakFehler
import de.blackforttec.daysandkeys.kern.DakKern
import de.blackforttec.daysandkeys.kern.DakUhr
import de.blackforttec.daysandkeys.kern.KernLage
import de.blackforttec.daysandkeys.kern.SicherungDatei
import de.blackforttec.daysandkeys.kern.StatusEingabe
import de.blackforttec.daysandkeys.kern.Zeitpunkt
import java.io.File
import java.nio.file.Files
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

/**
 * Fährt die erzeugte Kotlin-Bindung gegen den echten Kern (Linux-Bibliothek aus core/target).
 * Belegt die Kette Kotlin, UniFFI, JNA, Rust ohne Gerät (#101). Alle Daten sind erfunden.
 */
class KernBindungTest {
    private class Uhr(var ms: Long) : DakUhr {
        override fun jetzt() = Zeitpunkt(ms, 120, "Europe/Berlin")
    }

    private fun status(k: DakKern, wert: Long) {
        val f = k.formular("status_jetzt")
        val stimmung = f.gruppen.flatMap { it.fragen }.first { it.schluessel == "stimmung" }
        k.statusSpeichern(StatusEingabe(f.id, null, listOf(AntwortEingabe(stimmung.messdefId, AntwortZustand.BEANTWORTET, wert, null, emptyList(), true)), false, null, null))
    }

    @Test
    fun einrichtenMehrereStatusSicherungUndWiederherstellung() {
        val dir = Files.createTempDirectory("dak").toFile()
        val uhr = Uhr(1_792_000_000_000)
        val a = DakKern.neu(File(dir, "a.sqlite").path, uhr)
        assertTrue(a.lage() is KernLage.NichtEingerichtet)
        val e = a.einrichtungBeginnen(240)
        assertEquals(24, e.woerter.size)
        val probe = e.probe.map { e.woerter[it.toInt()] }
        assertTrue(a.einrichtungProbePruefen(probe))
        a.einrichtungAbschliessen(probe, byteArrayOf(1, 2, 3), null)
        // Mehrmals am selben Tag festhalten.
        for (w in listOf(-2L, 1L, 3L)) { status(a, w); uhr.ms += 3 * 3600_000 }
        val heute = a.heute()
        val v = a.verlauf("stimmung", heute - 6, heute)
        assertEquals(3, v.punkte.size)
        assertEquals(1, v.tage.size)
        assertEquals(3L, v.tage[0].letzter)
        // Wert außerhalb der Skala wird abgelehnt.
        try { status(a, 9); fail("muss scheitern") } catch (x: DakFehler.Fehler) { assertEquals(500u, x.code) }
        // Sperren: die umhüllten Schlüssel bleiben gespeichert.
        a.sperren()
        val l = a.lage() as KernLage.Gesperrt
        assertTrue(l.kkUmhuellung!!.contentEquals(byteArrayOf(1, 2, 3)))
        a.entsperren(e.schluessel.kk.copyOf(), e.schluessel.kkH.copyOf())
        val s = a.sicherungErstellen()
        a.sicherungBestaetigen(s.bytes)
        // Zweites Gerät: Wiederherstellung mit den Wörtern.
        val b = DakKern.neu(File(dir, "b.sqlite").path, uhr)
        b.wiederherstellungBeginnen(e.woerter)
        val k = b.kandidaten(listOf(SicherungDatei(s.name, s.bytes)), emptyList())
        assertTrue(k.single().gueltig && k.single().empfohlen)
        val w = b.wiederherstellen(listOf(SicherungDatei(s.name, s.bytes)), emptyList(), 0u)
        b.vorbereitetesAbschliessen(null, null)
        assertEquals(3, b.verlauf("stimmung", heute - 6, heute).punkte.size)
        assertEquals(2uL, w.schluessel.generation)
        // Das alte Gerät erkennt den Marker und sichert nicht mehr.
        assertTrue(!a.markerPruefen(listOf(w.marker)))
        dir.deleteRecursively()
    }
}
