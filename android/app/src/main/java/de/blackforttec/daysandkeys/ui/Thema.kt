// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import de.blackforttec.daysandkeys.R

// Ruhig, warm, ohne Farbwertung von Werten (AE-27). Rot nur für Fehler und Löschen.
// Werte und Kontraste: docs/design/designvorgabe.md, Abschnitt 3.1 (#133).
internal val hell = lightColorScheme(
    primary = Color(0xFF1E5F5A), onPrimary = Color(0xFFFFFFFF),
    primaryContainer = Color(0xFFCFE6E1), onPrimaryContainer = Color(0xFF0A2F2B),
    secondary = Color(0xFF5E6B67), onSecondary = Color(0xFFFFFFFF),
    secondaryContainer = Color(0xFFE6DFD3), onSecondaryContainer = Color(0xFF2A251F),
    tertiary = Color(0xFF8A5714), onTertiary = Color(0xFFFFFFFF),
    tertiaryContainer = Color(0xFFF5E3C4), onTertiaryContainer = Color(0xFF3A2600),
    error = Color(0xFFB3261E), onError = Color(0xFFFFFFFF),
    errorContainer = Color(0xFFF9DEDC), onErrorContainer = Color(0xFF410E0B),
    background = Color(0xFFF7F4EF), onBackground = Color(0xFF1F1C18),
    surface = Color(0xFFFFFCF8), onSurface = Color(0xFF1F1C18),
    surfaceVariant = Color(0xFFE0D9D0), onSurfaceVariant = Color(0xFF5A544C),
    surfaceTint = Color(0xFF1E5F5A),
    surfaceContainerLowest = Color(0xFFFFFFFF), surfaceContainerLow = Color(0xFFF4F0EA),
    surfaceContainer = Color(0xFFEDE8E1), surfaceContainerHigh = Color(0xFFE7E1D9),
    surfaceContainerHighest = Color(0xFFE0D9D0),
    outline = Color(0xFF7F776C), outlineVariant = Color(0xFFD6CFC5),
    inverseSurface = Color(0xFF34302B), inverseOnSurface = Color(0xFFF4EFE8),
    inversePrimary = Color(0xFF8FD0C6), scrim = Color(0xFF000000),
)
internal val dunkel = darkColorScheme(
    primary = Color(0xFF8FD0C6), onPrimary = Color(0xFF00332E),
    primaryContainer = Color(0xFF1A4B46), onPrimaryContainer = Color(0xFFCDEDE7),
    secondary = Color(0xFFB4C3BE), onSecondary = Color(0xFF1F2B28),
    secondaryContainer = Color(0xFF3B352E), onSecondaryContainer = Color(0xFFEFE4D6),
    tertiary = Color(0xFFE3B56C), onTertiary = Color(0xFF422C00),
    tertiaryContainer = Color(0xFF4A3510), onTertiaryContainer = Color(0xFFF8DFB4),
    error = Color(0xFFF2B8B5), onError = Color(0xFF601410),
    errorContainer = Color(0xFF8C1D18), onErrorContainer = Color(0xFFF9DEDC),
    background = Color(0xFF131211), onBackground = Color(0xFFECE6DE),
    surface = Color(0xFF1C1A18), onSurface = Color(0xFFECE6DE),
    surfaceVariant = Color(0xFF34302B), onSurfaceVariant = Color(0xFFBDB5AA),
    surfaceTint = Color(0xFF8FD0C6),
    surfaceContainerLowest = Color(0xFF0F0E0D), surfaceContainerLow = Color(0xFF211F1C),
    surfaceContainer = Color(0xFF26231F), surfaceContainerHigh = Color(0xFF2D2A26),
    surfaceContainerHighest = Color(0xFF34302B),
    outline = Color(0xFF958C80), outlineVariant = Color(0xFF47413A),
    inverseSurface = Color(0xFFECE6DE), inverseOnSurface = Color(0xFF34302B),
    inversePrimary = Color(0xFF1E5F5A), scrim = Color(0xFF000000),
)

// Atkinson Hyperlegible Next (SIL OFL 1.1), variable Schrift, Gewicht über die Achse.
@OptIn(ExperimentalTextApi::class)
private fun schnitt(w: FontWeight) = Font(R.font.atkinson_next, w, variationSettings = FontVariation.Settings(FontVariation.weight(w.weight)))
private val Atkinson = FontFamily(schnitt(FontWeight.Normal), schnitt(FontWeight.Medium), schnitt(FontWeight.SemiBold), schnitt(FontWeight.Bold))

private fun stil(sp: Int, zeile: Int, gewicht: FontWeight) =
    TextStyle(fontFamily = Atkinson, fontSize = sp.sp, lineHeight = zeile.sp, fontWeight = gewicht, letterSpacing = 0.sp)

private val schrift = Typography(
    displaySmall = stil(34, 40, FontWeight.SemiBold),
    headlineSmall = stil(26, 32, FontWeight.SemiBold),
    titleLarge = stil(20, 26, FontWeight.SemiBold),
    titleMedium = stil(18, 24, FontWeight.Medium),
    titleSmall = stil(15, 20, FontWeight.SemiBold),
    bodyLarge = stil(17, 24, FontWeight.Normal),
    bodyMedium = stil(15, 22, FontWeight.Normal),
    bodySmall = stil(13, 18, FontWeight.Normal),
    labelLarge = stil(16, 20, FontWeight.SemiBold),
    labelMedium = stil(14, 18, FontWeight.Medium),
    labelSmall = stil(12, 16, FontWeight.Medium),
)
private val formen = Shapes(
    extraSmall = RoundedCornerShape(8.dp), small = RoundedCornerShape(12.dp),
    medium = RoundedCornerShape(16.dp), large = RoundedCornerShape(24.dp), extraLarge = RoundedCornerShape(28.dp),
)

object Abstand {
    val xs = 4.dp; val s = 8.dp; val m = 12.dp; val l = 16.dp; val xl = 24.dp; val xxl = 32.dp
    val rand = 16.dp; val abschnitt = 24.dp; val frage = 20.dp; val hauptknopf = 56.dp; val ziel = 48.dp
}

object Bewegung {
    const val KURZ = 120; const val MITTEL = 220; const val LANG = 320
    val betont = CubicBezierEasing(0.05f, 0.7f, 0.1f, 1.0f)
}

@Composable
fun Thema(inhalt: @Composable () -> Unit) {
    MaterialTheme(colorScheme = if (isSystemInDarkTheme()) dunkel else hell, typography = schrift, shapes = formen, content = inhalt)
}
