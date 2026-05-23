import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.kotlin.compose)
}


val clientConfigProperties = Properties()
val clientConfigFile = project.file("config.properties")
val clientConfigExampleFile = project.file("config.example.properties")
val clientConfigSource = if (clientConfigFile.exists()) clientConfigFile else clientConfigExampleFile
if (clientConfigSource.exists()) {
    clientConfigSource.inputStream().use { clientConfigProperties.load(it) }
}

fun clientConfigValue(key: String, fallback: String): String =
    clientConfigProperties.getProperty(key)?.trim()?.takeIf { it.isNotEmpty() } ?: fallback

fun quotedBuildConfigString(value: String): String =
    "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\""


android {
    namespace = "com.rustella.stellartrail"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.rustella.stellartrail"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
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
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
        }
        release {
            isMinifyEnabled = false
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
