// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import de.blackforttec.daysandkeys.R

/** Gesperrt sichtbar sind nur Entsperren und die Hilfe (AE-20); Knöpfe im Daumenbereich (Vorgabe 4.10). */
@Composable
fun SperrSeite(z: AppZustand) {
    LaunchedEffect(Unit) { if (z.ziel != null) z.entsperren() }
    Column(Modifier.fillMaxSize().padding(horizontal = Abstand.rand), horizontalAlignment = Alignment.CenterHorizontally) {
        Spacer(Modifier.weight(0.4f))
        Image(painterResource(R.drawable.ic_vordergrund), null, Modifier.size(96.dp).background(MaterialTheme.colorScheme.primary, CircleShape))
        Spacer(Modifier.height(Abstand.l))
        Text(stringResource(R.string.allgemein_app_name), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.sperre_gesperrt), style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.weight(0.6f))
        if (z.ersetzt) {
            Hinweisfeld(stringResource(R.string.sperre_ersetzt))
            Spacer(Modifier.height(Abstand.m))
        }
        Button(onClick = { z.entsperren() }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.hauptknopf)) { Text(stringResource(R.string.sperre_entsperren)) }
        Spacer(Modifier.height(Abstand.m))
        OutlinedButton(onClick = { z.zu(Seite.Hilfe) }, modifier = Modifier.fillMaxWidth().heightIn(min = Abstand.ziel)) { Text(stringResource(R.string.allgemein_hilfe_krise)) }
        Text(
            stringResource(R.string.sperre_hinweis),
            style = MaterialTheme.typography.bodySmall, textAlign = TextAlign.Center, color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(vertical = Abstand.l),
        )
    }
}
