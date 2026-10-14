// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Ruhig, ohne Farbwertung von Werten (Gesamtkonzept Abschnitt 6, AE-27).
private val hell = lightColorScheme(
    primary = Color(0xFF3D4A5C), onPrimary = Color.White,
    secondary = Color(0xFF6B7A8F), secondaryContainer = Color(0xFFDDE3EA), onSecondaryContainer = Color(0xFF1D2733),
    background = Color(0xFFF7F6F3), surface = Color(0xFFFFFFFF), surfaceVariant = Color(0xFFECEAE5),
    tertiary = Color(0xFF8A6D3B),
)
private val dunkel = darkColorScheme(
    primary = Color(0xFFB9C6D8), onPrimary = Color(0xFF1D2733),
    secondary = Color(0xFF9AA8BA), secondaryContainer = Color(0xFF34404F), onSecondaryContainer = Color(0xFFE3E8EE),
    background = Color(0xFF15181C), surface = Color(0xFF1D2126), surfaceVariant = Color(0xFF2A2F35),
    tertiary = Color(0xFFD8B979),
)

@Composable
fun Thema(inhalt: @Composable () -> Unit) {
    MaterialTheme(colorScheme = if (isSystemInDarkTheme()) dunkel else hell, content = inhalt)
}
