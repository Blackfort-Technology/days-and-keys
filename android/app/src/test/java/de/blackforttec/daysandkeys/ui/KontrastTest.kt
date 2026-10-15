// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.Color
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.pow

/** Rechnet die Kontrastpaare aus docs/design/designvorgabe.md 3.1 nach (WCAG 2.2, #133). */
class KontrastTest {
    private fun kanal(c: Float) = if (c <= 0.04045f) c / 12.92 else ((c + 0.055) / 1.055).pow(2.4)
    private fun leuchte(c: Color) = 0.2126 * kanal(c.red) + 0.7152 * kanal(c.green) + 0.0722 * kanal(c.blue)
    private fun kontrast(a: Color, b: Color): Double {
        val (h, d) = listOf(leuchte(a), leuchte(b)).sortedDescending()
        return (h + 0.05) / (d + 0.05)
    }

    private fun paare(s: ColorScheme) = listOf(
        Triple("Text auf Fläche", s.onSurface to s.surface, 4.5),
        Triple("Text auf Grund", s.onBackground to s.background, 4.5),
        Triple("Hinweis auf Fläche", s.onSurfaceVariant to s.surface, 4.5),
        Triple("Hinweis auf Grund", s.onSurfaceVariant to s.background, 4.5),
        Triple("Hinweis auf Leiste", s.onSurfaceVariant to s.surfaceContainer, 4.5),
        Triple("Hinweis auf Karte", s.onSurfaceVariant to s.surfaceContainerLow, 4.5),
        Triple("Hauptknopf", s.onPrimary to s.primary, 4.5),
        Triple("Handlungsfarbe als Text", s.primary to s.surface, 4.5),
        Triple("Gewählt", s.onPrimaryContainer to s.primaryContainer, 4.5),
        Triple("Tonknopf", s.onSecondaryContainer to s.secondaryContainer, 4.5),
        Triple("Hinweiskarte", s.onTertiaryContainer to s.tertiaryContainer, 4.5),
        Triple("Rand von Bedienelementen", s.outline to s.surface, 3.0),
        Triple("Grundniveau-Linie", s.tertiary to s.surface, 3.0),
        Triple("Fehler", s.error to s.surface, 4.5),
        Triple("Text auf Spur", s.onSurface to s.surfaceContainerHighest, 4.5),
        Triple("Marke der Stufenleiste auf Spur", s.primary to s.surfaceContainerHighest, 3.0),
    )

    @Test
    fun hellUndDunkelErreichenDieSchwellen() {
        val befunde = listOf("hell" to hell, "dunkel" to dunkel).flatMap { (name, s) ->
            paare(s).mapNotNull { (was, p, soll) ->
                val k = kontrast(p.first, p.second)
                if (k < soll) "$name, $was: %.2f statt %.1f".format(k, soll) else null
            }
        }
        assertTrue(befunde.joinToString("\n"), befunde.isEmpty())
    }

    @Test
    fun gegenprobeErkenntZuSchwachenKontrast() {
        // Der alte Hinweiston auf dem alten Grund lag bei 4,04:1 und muss als zu schwach gelten.
        assertTrue(kontrast(Color(0xFF6B7A8F), Color(0xFFF7F6F3)) < 4.5)
    }
}
