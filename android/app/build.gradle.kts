// SPDX-License-Identifier: MPL-2.0
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "de.blackforttec.daysandkeys"
    compileSdk = 37

    defaultConfig {
        applicationId = "de.blackforttec.daysandkeys"
        minSdk = 33
        targetSdk = 36
        versionCode = (System.getenv("DAK_VERSION_CODE") ?: "1").toInt()
        versionName = System.getenv("DAK_VERSION_NAME") ?: "0.1.0"
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }

    signingConfigs {
        create("release") {
            // Nur im Release-Lauf der CI gesetzt (#9, #68); PR-Läufe bauen unsigniert.
            val pfad = System.getenv("DAK_KEYSTORE")
            if (pfad != null) {
                storeFile = file(pfad)
                storePassword = System.getenv("DAK_KEYSTORE_PASS")
                keyAlias = System.getenv("DAK_KEY_ALIAS")
                keyPassword = System.getenv("DAK_KEYSTORE_PASS")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            if (System.getenv("DAK_KEYSTORE") != null) signingConfig = signingConfigs.getByName("release")
        }
        // Vorschau-Builds laufen als eigene App neben der Release-App, mit eigenen Daten (#133).
        debug {
            applicationIdSuffix = ".vorschau"
            versionNameSuffix = "-vorschau"
        }
    }

    buildFeatures { compose = true }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    packaging { jniLibs { useLegacyPackaging = false } }
    lint { abortOnError = true; warningsAsErrors = false }
    testOptions {
        unitTests.all {
            // Linux-Bibliothek des Kerns für den Bindungstest (core/target/release).
            it.systemProperty("jna.library.path", rootProject.file("../core/target/release").absolutePath)
        }
    }
}


dependencies {
    val bom = platform("androidx.compose:compose-bom:2026.08.00")
    implementation(bom)
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.activity:activity-compose:1.12.4")
    implementation("net.java.dev.jna:jna:5.17.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
    testImplementation("junit:junit:4.13.2")
    testImplementation("net.java.dev.jna:jna:5.17.0")
}
