// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys

import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Wächter für #144 (AE-22): Oberflächentexte stehen in res/values/strings.xml, nicht im Kotlin-Code.
 * Lint `HardcodedText` prüft nur XML-Layouts, in Compose findet er nichts. Dieser Test liest die
 * Quellen unter src/main/java (ohne die erzeugte Kernbindung `kern/`) und meldet jedes String-Literal,
 * das wie ein Nutzertext aussieht: mindestens ein Buchstabe und dazu ein Leerzeichen, ein Umlaut oder
 * ein Großbuchstabe am Anfang. Eingesetzte Werte (`$x`, `${x}`) und Formatangaben (`%02d`) zählen
 * nicht mit. Kleingeschriebene Einzelwörter erkennt die Regel nicht; die gehören trotzdem nach strings.xml.
 */
class OberflaechentexteTest {
    /** Eng halten: jeder Eintrag ist kein Oberflächentext, gilt nur in seiner Datei und braucht eine Begründung. */
    private val erlaubt = mapOf(
        // Name des Keystore-Anbieters von Android, kein Text.
        "AndroidKeyStore" to "plattform/Schluesselspeicher.kt",
        // Transformationsname für Cipher.getInstance, kein Text.
        "AES/GCM/NoPadding" to "plattform/Schluesselspeicher.kt",
        // Ausnahmetexte von error(); die Oberfläche zeigt davon nur den Klassennamen.
        "kein Ordner" to "plattform/Sicherungsordner.kt",
        "Datei ließ sich nicht anlegen" to "plattform/Sicherungsordner.kt",
        // Formatmuster für DateTimeFormatter, kein Text.
        "dd.MM.yyyy, HH:mm" to "ui/Bausteine.kt",
        "HH:mm" to "ui/Bausteine.kt",
    )

    private data class Literal(val zeile: Int, val roh: String, val text: String)

    /** Kleiner Lexer: überspringt Kommentare und Zeichenliterale, folgt Templates in die Tiefe. */
    private class Lexer(private val q: String) {
        val literale = mutableListOf<Literal>()
        private var zeile = 1

        init {
            var i = 0
            while (i < q.length) {
                val c = q[i]
                when {
                    q.startsWith("//", i) -> while (i < q.length && q[i] != '\n') i++
                    q.startsWith("/*", i) -> {
                        val e = q.indexOf("*/", i + 2)
                        zeile += q.substring(i, e).count { it == '\n' }
                        i = e + 2
                    }
                    q.startsWith("\"\"\"", i) -> error("Rohstrings unterstützt der Wächter nicht (Zeile $zeile)")
                    c == '\'' -> i = if (q[i + 1] == '\\') q.indexOf('\'', i + 3) + 1 else i + 3
                    c == '"' -> i = string(i + 1)
                    else -> { if (c == '\n') zeile++; i++ }
                }
            }
        }

        /** [start] zeigt hinter das öffnende Anführungszeichen; liefert die Stelle hinter dem schließenden. */
        private fun string(start: Int): Int {
            var j = start
            val text = StringBuilder()
            val anfang = zeile
            while (j < q.length) {
                val c = q[j]
                when {
                    c == '\\' -> { text.append(q[j + 1]); j += 2 }
                    c == '"' -> { literale += Literal(anfang, q.substring(start, j), text.toString()); return j + 1 }
                    c == '$' && j + 1 < q.length && q[j + 1] == '{' -> { j = ausdruck(j + 2); text.append(WERT) }
                    c == '$' && j + 1 < q.length && (q[j + 1].isLetter() || q[j + 1] == '_') -> {
                        j++
                        while (j < q.length && (q[j].isLetterOrDigit() || q[j] == '_')) j++
                        text.append(WERT)
                    }
                    else -> { if (c == '\n') zeile++; text.append(c); j++ }
                }
            }
            error("offenes Literal ab Zeile $anfang")
        }

        /** Ausdruck in `${…}`; Literale darin werden mitgeprüft. */
        private fun ausdruck(start: Int): Int {
            var j = start
            var tiefe = 1
            while (j < q.length) {
                when (q[j]) {
                    '"' -> { j = string(j + 1); continue }
                    '{' -> tiefe++
                    '}' -> if (--tiefe == 0) return j + 1
                    '\n' -> zeile++
                }
                j++
            }
            error("offenes Template")
        }
    }

    private fun nutzertext(text: String): Boolean {
        val t = FORMATANGABE.replace(text.replace(WERT.toString(), ""), "")
        if (t.none { it.isLetter() }) return false
        return ' ' in t || UMLAUT.containsMatchIn(t) || t[0].isUpperCase()
    }

    @Test
    fun keineOberflaechentexteImCode() {
        // Der Test läuft im Modulverzeichnis android/app; fehlt der Ordner, darf er nicht still bestehen.
        val wurzel = File("src/main/java/de/blackforttec/daysandkeys")
        assertTrue("Quellen nicht gefunden: ${wurzel.absolutePath}", wurzel.isDirectory)
        val befunde = mutableListOf<String>()
        val genutzt = mutableSetOf<String>()
        var dateien = 0
        wurzel.walkTopDown().filter { it.isFile && it.extension == "kt" }.sorted().forEach { f ->
            val pfad = f.relativeTo(wurzel).invariantSeparatorsPath
            if (pfad.startsWith("kern/")) return@forEach
            dateien++
            for (l in Lexer(f.readText()).literale) {
                if (!nutzertext(l.text)) continue
                if (erlaubt[l.roh] == pfad) genutzt += l.roh else befunde += "$pfad:${l.zeile}: \"${l.roh}\""
            }
        }
        assertTrue("zu wenige Quelldateien gelesen: $dateien", dateien >= 10)
        assertTrue("Texte gehören nach res/values/strings.xml (#144):\n" + befunde.joinToString("\n"), befunde.isEmpty())
        // Veraltete Ausnahmen fallen auf, damit die Liste nicht wächst.
        assertEquals("Positivliste enthält ungenutzte Einträge", erlaubt.keys, genutzt)
    }

    @Test
    fun gegenprobeErkenntTexte() {
        val probe = """
            // Text("Kommentar zählt nicht")
            Text("Probe Text")
            Text("Gesperrt")
            Text("Präparat")
            Text("${'$'}{n.name}, ${'$'}{n.nummer}, Wählfeld öffnen")
            Text("Zuletzt um ${'$'}{uhrzeitAus(it)}")
            Text("status_jetzt"); Text("ziel"); Text("%02d:%02d"); Text("${'$'}tag, %02d:%02d"); Text(" · ")
            Text("${'$'}{a ?: ""} … ${'$'}{b ?: ""}"); val c = '"'
        """.trimIndent()
        val treffer = Lexer(probe).literale.filter { nutzertext(it.text) }.map { it.roh }
        assertEquals(
            listOf("Probe Text", "Gesperrt", "Präparat", "\${n.name}, \${n.nummer}, Wählfeld öffnen", "Zuletzt um \${uhrzeitAus(it)}"),
            treffer,
        )
    }

    private companion object {
        /** Platzhalter für eingesetzte Werte. */
        const val WERT = '\u0000'
        val FORMATANGABE = Regex("""%(\d+\$)?[-#+ 0,(]*\d*(\.\d+)?[a-zA-Z]""")
        val UMLAUT = Regex("[äöüÄÖÜß]")
    }
}
