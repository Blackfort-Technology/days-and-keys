// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys

import android.content.Intent
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.enableEdgeToEdge
import androidx.activity.compose.setContent
import de.blackforttec.daysandkeys.ui.AppZustand
import de.blackforttec.daysandkeys.ui.DaysAndKeysOberflaeche

class MainActivity : ComponentActivity() {
    private lateinit var zustand: AppZustand
    private var imHintergrundSeit: Long = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        // Symbole der Systemleisten dunkel auf hellem, hell auf dunklem Thema (#119).
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        // Kein Bildschirmfoto, kein Vorschaubild im App-Umschalter (Protokoll 14, #69).
        window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
        window.setHideOverlayWindows(true)
        zustand = AppZustand(application as DaysAndKeysApp, this)
        zustand.ziel = intent?.getStringExtra("ziel")
        setContent { DaysAndKeysOberflaeche(zustand) }
        window.decorView.filterTouchesWhenObscured = true
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        intent.getStringExtra("ziel")?.let { zustand.zielSetzen(it) }
    }

    override fun onStop() {
        super.onStop()
        imHintergrundSeit = System.currentTimeMillis()
        zustand.imHintergrund()
    }

    override fun onStart() {
        super.onStart()
        if (imHintergrundSeit > 0) zustand.zurueck(System.currentTimeMillis() - imHintergrundSeit)
    }
}
