// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.plattform

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import de.blackforttec.daysandkeys.kern.DakKern
import de.blackforttec.daysandkeys.kern.DateiInfo
import de.blackforttec.daysandkeys.kern.SicherungDatei

/**
 * Automatische Sicherung in einen einmal gewählten Ordner über das Storage Access Framework
 * (Protokoll 10.2, AE-17). Die Dateien sind verschlüsselt; der Ordner darf in einer Cloud liegen.
 */
class Sicherungsordner(private val ctx: Context) {
    private val prefs = ctx.getSharedPreferences("sicherung", Context.MODE_PRIVATE)

    var ordner: Uri?
        get() = prefs.getString("ordner", null)?.let(Uri::parse)
        set(v) { prefs.edit().putString("ordner", v?.toString()).apply() }

    fun uebernehmen(uri: Uri) {
        ctx.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
        ordner = uri
    }

    data class Datei(val name: String, val docUri: Uri, val geaendert: Long)

    fun dateien(baum: Uri = ordner ?: error("kein Ordner")): List<Datei> {
        val kinder = DocumentsContract.buildChildDocumentsUriUsingTree(baum, DocumentsContract.getTreeDocumentId(baum))
        val out = mutableListOf<Datei>()
        ctx.contentResolver.query(kinder, arrayOf(DocumentsContract.Document.COLUMN_DOCUMENT_ID, DocumentsContract.Document.COLUMN_DISPLAY_NAME, DocumentsContract.Document.COLUMN_LAST_MODIFIED), null, null, null)?.use { c ->
            while (c.moveToNext()) {
                val name = c.getString(1) ?: continue
                if (name.endsWith(".daks") || name.endsWith(".dakm")) {
                    out += Datei(name, DocumentsContract.buildDocumentUriUsingTree(baum, c.getString(0)), c.getLong(2))
                }
            }
        }
        return out
    }

    fun lesen(d: Datei): ByteArray = ctx.contentResolver.openInputStream(d.docUri)!!.use { it.readBytes() }

    fun schreiben(name: String, bytes: ByteArray, baum: Uri = ordner ?: error("kein Ordner")): Datei {
        val eltern = DocumentsContract.buildDocumentUriUsingTree(baum, DocumentsContract.getTreeDocumentId(baum))
        val uri = DocumentsContract.createDocument(ctx.contentResolver, eltern, "application/octet-stream", name) ?: error("Datei ließ sich nicht anlegen")
        ctx.contentResolver.openOutputStream(uri, "wt")!!.use { it.write(bytes) }
        return Datei(name, uri, System.currentTimeMillis())
    }

    /** Sichert, liest zurück, lässt den Kern prüfen, räumt nach der Aufbewahrungsregel auf. */
    fun sichern(kern: DakKern): Ergebnis {
        val baum = ordner ?: return Ergebnis.KeinOrdner
        val vorhanden = dateien(baum)
        val marker = vorhanden.filter { it.name.endsWith(".dakm") }.map { lesen(it) }
        if (!kern.markerPruefen(marker)) return Ergebnis.Ersetzt
        val s = kern.sicherungErstellen()
        val d = schreiben(s.name, s.bytes, baum)
        val zurueck = lesen(d)
        kern.sicherungBestaetigen(zurueck)
        val tag = { ms: Long -> Math.floorDiv(ms, 86_400_000L) }
        val alle = dateien(baum).filter { it.name.endsWith(".daks") }
        val weg = kern.aufbewahrung(alle.map { DateiInfo(it.name, tag(it.geaendert)) }, tag(System.currentTimeMillis()))
        for (n in weg) alle.firstOrNull { it.name == n }?.let { DocumentsContract.deleteDocument(ctx.contentResolver, it.docUri) }
        return Ergebnis.Gesichert(s.name)
    }

    fun sicherungen(baum: Uri): List<SicherungDatei> = dateien(baum).filter { it.name.endsWith(".daks") }.map { SicherungDatei(it.name, lesen(it)) }

    fun marker(baum: Uri): List<ByteArray> = dateien(baum).filter { it.name.endsWith(".dakm") }.map { lesen(it) }

    sealed class Ergebnis {
        data object KeinOrdner : Ergebnis()
        data object Ersetzt : Ergebnis()
        data class Gesichert(val name: String) : Ergebnis()
    }
}
