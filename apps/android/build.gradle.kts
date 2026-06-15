import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.kotlin.compose)
}

val androidAppVersionName = "0.0.1"
val androidAppVersionCode = 4

val clientConfigProperties = Properties()
val clientConfigFile = project.file("config.properties")
val clientConfigExampleFile = project.file("config.example.properties")
val clientConfigSource = if (clientConfigFile.exists()) clientConfigFile else clientConfigExampleFile
if (clientConfigSource.exists()) {
    clientConfigSource.inputStream().use { clientConfigProperties.load(it) }
}

fun clientConfigValue(key: String, fallback: String): String =
    clientConfigProperties.getProperty(key)?.trim()?.takeIf { it.isNotEmpty() } ?: fallback

fun clientConfigIntValue(key: String, fallback: Int): Int =
    clientConfigValue(key, fallback.toString()).toIntOrNull()?.takeIf { it > 0 } ?: fallback

fun quotedBuildConfigString(value: String): String =
    "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

val releaseSigningEnvironmentKeys = listOf(
    "STELLARTRAIL_ANDROID_KEYSTORE_PATH",
    "STELLARTRAIL_ANDROID_KEYSTORE_PASSWORD",
    "STELLARTRAIL_ANDROID_KEY_ALIAS",
    "STELLARTRAIL_ANDROID_KEY_PASSWORD",
)

fun releaseSigningEnvValue(key: String): String? =
    providers.environmentVariable(key).orNull?.trim()?.takeIf { it.isNotEmpty() }

fun isAndroidReleaseTaskRequested(): Boolean =
    gradle.startParameter.taskNames.any { requestedTask ->
        val taskName = requestedTask.substringAfterLast(':')
        taskName.equals("assemble", ignoreCase = true) ||
            taskName.equals("bundle", ignoreCase = true) ||
            taskName.contains("Release", ignoreCase = true)
    }

val releaseSigningRequested = isAndroidReleaseTaskRequested()

fun requiredReleaseSigningEnvValue(key: String): String =
    releaseSigningEnvValue(key) ?: error(
        "$key is required for Android release signing. " +
            "Set all release signing environment variables before running release builds: " +
            releaseSigningEnvironmentKeys.joinToString(", "),
    )


android {
    namespace = "com.rustella.stellartrail"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.rustella.stellartrail"
        minSdk = 26
        targetSdk = 36
        versionCode = androidAppVersionCode
        versionName = androidAppVersionName
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables.useSupportLibrary = true
        buildConfigField(
            "String",
            "DEFAULT_API_BASE_URL",
            quotedBuildConfigString(clientConfigValue("stellartrail.apiBaseUrl", "https://api.example.invalid")),
        )
        buildConfigField(
            "String",
            "DEFAULT_ASSETS_BASE_URL",
            quotedBuildConfigString(clientConfigValue("stellartrail.assetsBaseUrl", "https://assets.example.invalid")),
        )
        buildConfigField(
            "String",
            "DEFAULT_DOMAIN_CANDIDATES",
            quotedBuildConfigString(clientConfigValue("stellartrail.domainCandidates", "")),
        )
        buildConfigField(
            "String",
            "DEFAULT_CLIENT",
            quotedBuildConfigString(clientConfigValue("stellartrail.client", "android")),
        )
        buildConfigField(
            "String",
            "DEFAULT_CLIENT_VERSION",
            quotedBuildConfigString(androidAppVersionName),
        )
        buildConfigField(
            "String",
            "DEFAULT_REQUEST_SIGNATURE_APP_ID",
            quotedBuildConfigString(clientConfigValue("app_id", "example-android-client-id")),
        )
        buildConfigField(
            "String",
            "DEFAULT_REQUEST_SIGNATURE_APP_SECRET",
            quotedBuildConfigString(clientConfigValue("app_secret", "example-android-client-secret")),
        )
        buildConfigField(
            "String",
            "DEFAULT_CERTIFICATE_PINS",
            quotedBuildConfigString(clientConfigValue("stellartrail.certificatePins", "")),
        )
        buildConfigField(
            "int",
            "SMS_CODE_COOLDOWN_SECONDS",
            clientConfigIntValue("stellartrail.smsCodeCooldownSeconds", 60).toString(),
        )
    }

    signingConfigs {
        if (releaseSigningRequested) {
            create("release") {
                storeFile = file(requiredReleaseSigningEnvValue("STELLARTRAIL_ANDROID_KEYSTORE_PATH"))
                storePassword = requiredReleaseSigningEnvValue("STELLARTRAIL_ANDROID_KEYSTORE_PASSWORD")
                keyAlias = requiredReleaseSigningEnvValue("STELLARTRAIL_ANDROID_KEY_ALIAS")
                keyPassword = requiredReleaseSigningEnvValue("STELLARTRAIL_ANDROID_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
        }
        release {
            if (releaseSigningRequested) {
                signingConfig = signingConfigs.getByName("release")
            }
            isMinifyEnabled = true
            isShrinkResources = true
            ndk {
                debugSymbolLevel = "SYMBOL_TABLE"
            }
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    buildFeatures {
        buildConfig = true
        compose = true
    }

    packaging {
        resources.excludes += "/META-INF/{AL2.0,LGPL2.1}"
    }
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.material3)
    implementation(libs.androidx.compose.material.icons.extended)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.lifecycle.viewmodel.ktx)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.security.crypto)
    implementation(libs.ktor.client.core)
    implementation(libs.ktor.client.okhttp)
    implementation(libs.ktor.client.content.negotiation)
    implementation(libs.ktor.serialization.json)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.coil.compose)
    implementation(libs.coil.gif)
    implementation(libs.maptiler.sdk.kotlin)
    implementation(libs.play.services.location)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.ktor.client.mock)
    testImplementation(libs.okhttp.mockwebserver)

    androidTestImplementation(libs.androidx.test.ext.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.tooling)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}
