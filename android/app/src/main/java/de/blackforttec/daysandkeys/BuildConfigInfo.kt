// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys

object BuildConfigInfo {
    val version: String get() = try {
        val app = DaysAndKeysAppHolder.app
        app.packageManager.getPackageInfo(app.packageName, 0).versionName ?: "?"
    } catch (e: Exception) { "?" }
}

object DaysAndKeysAppHolder {
    lateinit var app: DaysAndKeysApp
}
