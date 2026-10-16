// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.plattform

import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import de.blackforttec.daysandkeys.MainActivity
import de.blackforttec.daysandkeys.R
import de.blackforttec.daysandkeys.kern.Benachrichtigung
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneId
import org.json.JSONArray
import org.json.JSONObject

/**
 * Setzt die vom Kern geplanten lokalen Benachrichtigungen (Protokoll 11, #42). Der Plan liegt
 * unverschlüsselt in den App-Einstellungen, damit er nach einem Neustart ohne Entsperren wieder
 * gesetzt werden kann; er enthält nur Uhrzeiten und neutrale Texte, keine Präparatnamen, keine Werte.
 */
object Planer {
    const val KANAL_MEDIKAMENT = "medikament"
    const val KANAL_ERINNERUNG = "erinnerung"

    fun kanaeleAnlegen(ctx: Context) {
        val nm = ctx.getSystemService(NotificationManager::class.java)
        nm.createNotificationChannel(NotificationChannel(KANAL_MEDIKAMENT, ctx.getString(R.string.benachrichtigung_kanal_einnahme), NotificationManager.IMPORTANCE_HIGH).apply {
            lockscreenVisibility = Notification.VISIBILITY_PRIVATE
        })
        nm.createNotificationChannel(NotificationChannel(KANAL_ERINNERUNG, ctx.getString(R.string.benachrichtigung_kanal_erinnerungen), NotificationManager.IMPORTANCE_DEFAULT).apply {
            lockscreenVisibility = Notification.VISIBILITY_PRIVATE
        })
    }

    private fun prefs(ctx: Context) = ctx.getSharedPreferences("planer", Context.MODE_PRIVATE)

    fun setzen(ctx: Context, plan: List<Benachrichtigung>) {
        val arr = JSONArray()
        for (b in plan) {
            val wann = LocalDate.parse(b.datumIso).atTime(LocalTime.ofSecondOfDay(b.uhrzeitMin.toLong() * 60)).atZone(ZoneId.systemDefault()).toInstant().toEpochMilli()
            arr.put(JSONObject().put("id", b.schluessel).put("wann", wann).put("kanal", b.kanal).put("titel", b.titel).put("text", b.text).put("ziel", b.ziel))
        }
        alleAbbrechen(ctx)
        prefs(ctx).edit().putString("plan", arr.toString()).apply()
        planen(ctx)
    }

    fun exaktErlaubt(ctx: Context): Boolean = ctx.getSystemService(AlarmManager::class.java).canScheduleExactAlarms()

    /** Setzt alle künftigen Einträge des gespeicherten Plans (auch nach einem Neustart). */
    fun planen(ctx: Context) {
        val am = ctx.getSystemService(AlarmManager::class.java)
        val arr = JSONArray(prefs(ctx).getString("plan", "[]"))
        val jetzt = System.currentTimeMillis()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            val wann = o.getLong("wann")
            if (wann <= jetzt) continue
            val pi = absicht(ctx, o)
            if (am.canScheduleExactAlarms()) am.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, wann, pi)
            else am.setWindow(AlarmManager.RTC_WAKEUP, wann, 10 * 60_000L, pi)
        }
    }

    private fun absicht(ctx: Context, o: JSONObject): PendingIntent {
        val i = Intent(ctx, ErinnerungEmpfaenger::class.java)
            .putExtra("id", o.getString("id")).putExtra("kanal", o.getString("kanal"))
            .putExtra("titel", o.getString("titel")).putExtra("text", o.getString("text")).putExtra("ziel", o.getString("ziel"))
        return PendingIntent.getBroadcast(ctx, o.getString("id").hashCode(), i, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    }

    private fun alleAbbrechen(ctx: Context) {
        val am = ctx.getSystemService(AlarmManager::class.java)
        val arr = JSONArray(prefs(ctx).getString("plan", "[]"))
        for (i in 0 until arr.length()) am.cancel(absicht(ctx, arr.getJSONObject(i)))
    }
}

class ErinnerungEmpfaenger : BroadcastReceiver() {
    override fun onReceive(ctx: Context, intent: Intent) {
        val oeffnen = Intent(ctx, MainActivity::class.java).putExtra("ziel", intent.getStringExtra("ziel")).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
        val pi = PendingIntent.getActivity(ctx, intent.getStringExtra("id").hashCode(), oeffnen, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val n = Notification.Builder(ctx, intent.getStringExtra("kanal") ?: Planer.KANAL_ERINNERUNG)
            .setSmallIcon(android.R.drawable.ic_popup_reminder)
            .setContentTitle(intent.getStringExtra("titel"))
            .setContentText(intent.getStringExtra("text"))
            .setAutoCancel(true)
            .setContentIntent(pi)
            .setVisibility(Notification.VISIBILITY_PRIVATE)
            .build()
        val nm = ctx.getSystemService(NotificationManager::class.java)
        if (nm.areNotificationsEnabled()) nm.notify(intent.getStringExtra("id").hashCode(), n)
    }
}

class StartEmpfaenger : BroadcastReceiver() {
    override fun onReceive(ctx: Context, intent: Intent) {
        Planer.kanaeleAnlegen(ctx)
        Planer.planen(ctx)
    }
}
