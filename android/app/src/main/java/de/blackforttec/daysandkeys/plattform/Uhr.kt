// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.plattform

import de.blackforttec.daysandkeys.kern.DakUhr
import de.blackforttec.daysandkeys.kern.Zeitpunkt
import java.time.ZoneId
import java.time.ZonedDateTime

/** Zeit mit Versatz und IANA-Zone zum Zeitpunkt der Erfassung (Protokoll 6.3). */
class Systemuhr : DakUhr {
    override fun jetzt(): Zeitpunkt {
        val z = ZonedDateTime.now(ZoneId.systemDefault())
        return Zeitpunkt(z.toInstant().toEpochMilli(), z.offset.totalSeconds / 60, z.zone.id)
    }
}
