// SPDX-License-Identifier: MPL-2.0
package de.blackforttec.daysandkeys.plattform

import android.hardware.biometrics.BiometricManager.Authenticators.BIOMETRIC_STRONG
import android.hardware.biometrics.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import android.hardware.biometrics.BiometricPrompt
import android.os.CancellationSignal
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.security.keystore.StrongBoxUnavailableException
import androidx.activity.ComponentActivity
import de.blackforttec.daysandkeys.R
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlinx.coroutines.suspendCancellableCoroutine

/**
 * Umhüllt KK und KK_h mit einem nicht exportierbaren AES-Schlüssel im Android Keystore
 * (StrongBox, sonst TEE). Jede Nutzung verlangt eine frische Geräteauthentifizierung:
 * Biometrie (stark) oder Geräte-Code (Protokoll 8.3, Entscheidung #99). Eine neu eingerichtete
 * Biometrie macht den Schlüssel nicht ungültig, weil der Geräte-Code ohnehin öffnet.
 *
 * Grenze (AE-12): Nach dem Enthüllen liegen die Bytes kurz im Speicher dieser Klasse; sie werden
 * sofort an den Kern übergeben und hier überschrieben.
 */
class Schluesselspeicher {
    private val ks: KeyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    private fun alias(generation: ULong) = "dak_kk_g$generation"

    var strongBox: Boolean = false
        private set

    private fun schluessel(generation: ULong, neu: Boolean): SecretKey {
        val a = alias(generation)
        if (!neu) return ks.getKey(a, null) as SecretKey
        if (ks.containsAlias(a)) ks.deleteEntry(a)
        fun spec(strong: Boolean) = KeyGenParameterSpec.Builder(a, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setUserAuthenticationRequired(true)
            .setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG or KeyProperties.AUTH_DEVICE_CREDENTIAL)
            .setInvalidatedByBiometricEnrollment(false)
            .setUnlockedDeviceRequired(true)
            .setIsStrongBoxBacked(strong)
            .build()
        val g = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        return try {
            g.init(spec(true)); strongBox = true; g.generateKey()
        } catch (e: StrongBoxUnavailableException) {
            g.init(spec(false)); strongBox = false; g.generateKey()
        }
    }

    private suspend fun authentifizieren(activity: ComponentActivity, cipher: Cipher, titel: String): Cipher =
        suspendCancellableCoroutine { k ->
            val abbruch = CancellationSignal()
            k.invokeOnCancellation { abbruch.cancel() }
            val prompt = BiometricPrompt.Builder(activity)
                .setTitle(titel)
                .setSubtitle(activity.getString(R.string.geraeteschutz_untertitel))
                .setAllowedAuthenticators(BIOMETRIC_STRONG or DEVICE_CREDENTIAL)
                .setConfirmationRequired(false)
                .build()
            prompt.authenticate(BiometricPrompt.CryptoObject(cipher), abbruch, activity.mainExecutor,
                object : BiometricPrompt.AuthenticationCallback() {
                    override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                        k.resume(result.cryptoObject?.cipher ?: cipher)
                    }
                    override fun onAuthenticationError(code: Int, msg: CharSequence) {
                        k.resumeWithException(Abgebrochen(msg.toString()))
                    }
                })
        }

    class Abgebrochen(meldung: String) : Exception(meldung)

    /** Native Bestätigung einer sensiblen Aktion mit frischer Geräteauthentifizierung (Protokoll 13). */
    suspend fun bestaetigen(activity: ComponentActivity, titel: String, text: String): Unit =
        suspendCancellableCoroutine { k ->
            val abbruch = CancellationSignal()
            k.invokeOnCancellation { abbruch.cancel() }
            BiometricPrompt.Builder(activity).setTitle(titel).setDescription(text)
                .setAllowedAuthenticators(BIOMETRIC_STRONG or DEVICE_CREDENTIAL).setConfirmationRequired(true).build()
                .authenticate(abbruch, activity.mainExecutor, object : BiometricPrompt.AuthenticationCallback() {
                    override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) = k.resume(Unit)
                    override fun onAuthenticationError(code: Int, msg: CharSequence) = k.resumeWithException(Abgebrochen(msg.toString()))
                })
        }

    /** Umhüllt KK || KK_h unter einem neuen Schlüssel dieser Generation; Ergebnis: iv || chiffrat. */
    suspend fun umhuellen(activity: ComponentActivity, generation: ULong, kk: ByteArray, kkH: ByteArray): ByteArray {
        val c = Cipher.getInstance("AES/GCM/NoPadding")
        c.init(Cipher.ENCRYPT_MODE, schluessel(generation, neu = true))
        val ok = authentifizieren(activity, c, activity.getString(R.string.geraeteschutz_schuetzen))
        val klar = kk + kkH
        try {
            return ok.iv + ok.doFinal(klar)
        } finally {
            klar.fill(0); kk.fill(0); kkH.fill(0)
        }
    }

    /** Enthüllt KK und KK_h nach frischer Geräteauthentifizierung. */
    suspend fun enthuellen(activity: ComponentActivity, generation: ULong, umhuellung: ByteArray): Pair<ByteArray, ByteArray> {
        val c = Cipher.getInstance("AES/GCM/NoPadding")
        c.init(Cipher.DECRYPT_MODE, schluessel(generation, neu = false), GCMParameterSpec(128, umhuellung.copyOfRange(0, 12)))
        val ok = authentifizieren(activity, c, activity.getString(R.string.geraeteschutz_entsperren))
        val klar = ok.doFinal(umhuellung.copyOfRange(12, umhuellung.size))
        val a = klar.copyOfRange(0, 32)
        val b = klar.copyOfRange(32, 64)
        klar.fill(0)
        return a to b
    }

    /** Alte Generationen nach einer Rotation entfernen. */
    fun aufraeumen(aktuell: ULong) {
        for (a in ks.aliases().toList()) {
            if (a.startsWith("dak_kk_g") && a != alias(aktuell)) ks.deleteEntry(a)
        }
    }
}
