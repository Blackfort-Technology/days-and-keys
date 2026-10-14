// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys

import android.app.Application
import de.blackforttec.daysandkeys.kern.DakKern
import de.blackforttec.daysandkeys.plattform.Planer
import de.blackforttec.daysandkeys.plattform.Schluesselspeicher
import de.blackforttec.daysandkeys.plattform.Sicherungsordner
import de.blackforttec.daysandkeys.plattform.Systemuhr
import java.io.File

class DaysAndKeysApp : Application() {
    lateinit var kern: DakKern
        private set
    lateinit var schluessel: Schluesselspeicher
        private set
    lateinit var ordner: Sicherungsordner
        private set

    override fun onCreate() {
        super.onCreate()
        DaysAndKeysAppHolder.app = this
        // Speicher ohne Inhalte im Klartext; vom System-Backup ausgenommen (datenauszug.xml).
        val pfad = File(noBackupFilesDir, "dak.sqlite").absolutePath
        kern = DakKern.neu(pfad, Systemuhr())
        schluessel = Schluesselspeicher()
        ordner = Sicherungsordner(this)
        Planer.kanaeleAnlegen(this)
    }
}
