package com.rustella.stellartrail.core.config

import android.content.Context
import com.rustella.stellartrail.BuildConfig
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/** API endpoint configuration shared by repositories. */
data class AppConfig(
    val baseUrl: String = sanitizeBaseUrl(BuildConfig.DEFAULT_API_BASE_URL),
)

interface AppConfigStore {
    val config: StateFlow<AppConfig>
    fun updateBaseUrl(baseUrl: String)
    fun resetBaseUrl()
}

class AndroidAppConfigStore(context: Context) : AppConfigStore {
    private val preferences = context.getSharedPreferences("stellartrail_config", Context.MODE_PRIVATE)
    private val defaultBaseUrl = sanitizeBaseUrl(BuildConfig.DEFAULT_API_BASE_URL)
    private val _config = MutableStateFlow(AppConfig(preferences.getString(KEY_BASE_URL, defaultBaseUrl) ?: defaultBaseUrl))

    override val config: StateFlow<AppConfig> = _config.asStateFlow()

    override fun updateBaseUrl(baseUrl: String) {
        val sanitized = sanitizeBaseUrl(baseUrl)
        preferences.edit().putString(KEY_BASE_URL, sanitized).apply()
        _config.value = AppConfig(sanitized)
    }

    override fun resetBaseUrl() {
        preferences.edit().remove(KEY_BASE_URL).apply()
        _config.value = AppConfig(defaultBaseUrl)
    }

    private companion object {
        const val KEY_BASE_URL = "base_url"
    }
}

class InMemoryAppConfigStore(initial: AppConfig = AppConfig()) : AppConfigStore {
    private val _config = MutableStateFlow(initial)
    override val config: StateFlow<AppConfig> = _config.asStateFlow()
    override fun updateBaseUrl(baseUrl: String) {
        _config.value = AppConfig(sanitizeBaseUrl(baseUrl))
    }
    override fun resetBaseUrl() {
        _config.value = AppConfig()
    }
}

fun sanitizeBaseUrl(baseUrl: String): String = baseUrl.trim().trimEnd('/')
