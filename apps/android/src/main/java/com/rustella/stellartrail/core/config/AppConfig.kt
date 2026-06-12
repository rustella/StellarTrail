package com.rustella.stellartrail.core.config

import android.content.Context
import com.rustella.stellartrail.BuildConfig
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/** API endpoint configuration shared by repositories. */
data class AppConfig(
    val baseUrl: String = sanitizeBaseUrl(BuildConfig.DEFAULT_API_BASE_URL),
    val assetsBaseUrl: String = sanitizeBaseUrl(BuildConfig.DEFAULT_ASSETS_BASE_URL),
    val domainCandidates: List<AppDomainCandidate> = parseDomainCandidates(BuildConfig.DEFAULT_DOMAIN_CANDIDATES),
    val clientIdentity: String = buildClientIdentity(
        client = BuildConfig.DEFAULT_CLIENT,
        version = BuildConfig.DEFAULT_CLIENT_VERSION,
    ),
    val requestSignature: RequestSignatureCredentials? = requestSignatureCredentials(
        appId = BuildConfig.DEFAULT_REQUEST_SIGNATURE_APP_ID,
        appSecret = BuildConfig.DEFAULT_REQUEST_SIGNATURE_APP_SECRET,
    ),
    val certificatePins: List<AppCertificatePin> = parseCertificatePins(BuildConfig.DEFAULT_CERTIFICATE_PINS),
)

data class AppDomainCandidate(
    val id: String,
    val apiBaseUrl: String,
    val assetsBaseUrl: String,
)

data class RequestSignatureCredentials(
    val appId: String,
    val appSecret: String,
)

data class AppCertificatePin(
    val hostname: String,
    val pin: String,
)

interface AppConfigStore {
    val config: StateFlow<AppConfig>
    fun updateBaseUrl(baseUrl: String)
    fun resetBaseUrl()
}

class AndroidAppConfigStore(context: Context) : AppConfigStore {
    private val preferences = context.getSharedPreferences("stellartrail_config", Context.MODE_PRIVATE)
    private val defaultBaseUrl = sanitizeBaseUrl(BuildConfig.DEFAULT_API_BASE_URL)
    private val defaultAssetsBaseUrl = sanitizeBaseUrl(BuildConfig.DEFAULT_ASSETS_BASE_URL)
    private val _config = MutableStateFlow(
        AppConfig(
            baseUrl = preferences.getString(KEY_BASE_URL, defaultBaseUrl) ?: defaultBaseUrl,
            assetsBaseUrl = preferences.getString(KEY_ASSETS_BASE_URL, defaultAssetsBaseUrl) ?: defaultAssetsBaseUrl,
        ),
    )

    override val config: StateFlow<AppConfig> = _config.asStateFlow()

    override fun updateBaseUrl(baseUrl: String) {
        val sanitized = sanitizeBaseUrl(baseUrl)
        preferences.edit().putString(KEY_BASE_URL, sanitized).apply()
        _config.value = _config.value.copy(baseUrl = sanitized)
    }

    override fun resetBaseUrl() {
        preferences.edit().remove(KEY_BASE_URL).remove(KEY_ASSETS_BASE_URL).apply()
        _config.value = _config.value.copy(baseUrl = defaultBaseUrl, assetsBaseUrl = defaultAssetsBaseUrl)
    }

    private companion object {
        const val KEY_BASE_URL = "base_url"
        const val KEY_ASSETS_BASE_URL = "assets_base_url"
    }
}

class InMemoryAppConfigStore(initial: AppConfig = AppConfig()) : AppConfigStore {
    private val _config = MutableStateFlow(initial)
    override val config: StateFlow<AppConfig> = _config.asStateFlow()
    override fun updateBaseUrl(baseUrl: String) {
        _config.value = _config.value.copy(baseUrl = sanitizeBaseUrl(baseUrl))
    }
    override fun resetBaseUrl() {
        _config.value = AppConfig()
    }
}

fun sanitizeBaseUrl(baseUrl: String): String = baseUrl.trim().trimEnd('/')

fun buildClientIdentity(client: String, version: String): String {
    val sanitizedClient = client.trim().ifEmpty { "android" }
    val sanitizedVersion = version.trim().ifEmpty { "0.1.0" }
    return "$sanitizedClient/$sanitizedVersion"
}

fun parseDomainCandidates(value: String): List<AppDomainCandidate> =
    value.split(';')
        .mapNotNull { rawCandidate ->
            val parts = rawCandidate.split('|')
            if (parts.size != 3) return@mapNotNull null
            val id = parts[0].trim()
            val apiBaseUrl = sanitizeBaseUrl(parts[1])
            val assetsBaseUrl = sanitizeBaseUrl(parts[2])
            if (id.isEmpty() || apiBaseUrl.isEmpty() || assetsBaseUrl.isEmpty()) return@mapNotNull null
            AppDomainCandidate(id = id, apiBaseUrl = apiBaseUrl, assetsBaseUrl = assetsBaseUrl)
        }

fun parseCertificatePins(value: String): List<AppCertificatePin> =
    value.split(';')
        .mapNotNull { rawPin ->
            val parts = rawPin.split('|')
            if (parts.size != 2) return@mapNotNull null
            val hostname = parts[0].trim().lowercase()
            val pin = parts[1].trim()
            if (hostname.isEmpty() || pin.isEmpty()) return@mapNotNull null
            if (!pin.startsWith("sha256/")) return@mapNotNull null
            AppCertificatePin(hostname = hostname, pin = pin)
        }

fun requestSignatureCredentials(appId: String, appSecret: String): RequestSignatureCredentials? {
    val sanitizedAppId = appId.trim()
    val sanitizedAppSecret = appSecret.trim()
    if (sanitizedAppId.isEmpty() || sanitizedAppSecret.isEmpty()) return null
    if (sanitizedAppId.startsWith("example-") || sanitizedAppSecret.startsWith("example-")) return null
    return RequestSignatureCredentials(appId = sanitizedAppId, appSecret = sanitizedAppSecret)
}
