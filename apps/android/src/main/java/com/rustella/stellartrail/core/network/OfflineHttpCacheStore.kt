package com.rustella.stellartrail.core.network

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.security.MessageDigest

data class OfflineHttpCacheStatus(
    val cachedResponseCount: Int = 0,
    val lastUpdatedAtMillis: Long? = null,
)

data class OfflineHttpCacheEntry(
    val body: String,
    val cachedAtMillis: Long,
)

interface OfflineHttpCacheStore {
    val status: StateFlow<OfflineHttpCacheStatus>
    suspend fun read(key: String): OfflineHttpCacheEntry?
    suspend fun write(key: String, body: String): OfflineHttpCacheStatus
    suspend fun clear(): OfflineHttpCacheStatus
}

class AndroidOfflineHttpCacheStore(context: Context) : OfflineHttpCacheStore {
    private val preferences = context.getSharedPreferences("stellartrail_offline_http_cache", Context.MODE_PRIVATE)
    private val _status = MutableStateFlow(loadStatus())
    override val status: StateFlow<OfflineHttpCacheStatus> = _status.asStateFlow()

    override suspend fun read(key: String): OfflineHttpCacheEntry? =
        preferences.getString(cachePreferenceKey(key), null)
            ?.let { raw -> runCatching { json.decodeFromString<OfflineHttpCacheSnapshot>(raw) }.getOrNull() }
            ?.let { snapshot -> OfflineHttpCacheEntry(snapshot.body, snapshot.cachedAtMillis) }

    override suspend fun write(key: String, body: String): OfflineHttpCacheStatus {
        val snapshot = OfflineHttpCacheSnapshot(
            body = body,
            cachedAtMillis = System.currentTimeMillis(),
        )
        preferences.edit()
            .putString(cachePreferenceKey(key), json.encodeToString(snapshot))
            .apply()
        return refreshStatus()
    }

    override suspend fun clear(): OfflineHttpCacheStatus {
        preferences.edit().clear().apply()
        _status.value = OfflineHttpCacheStatus()
        return _status.value
    }

    private fun loadStatus(): OfflineHttpCacheStatus =
        preferences.all.values
            .asSequence()
            .filterIsInstance<String>()
            .mapNotNull { raw -> runCatching { json.decodeFromString<OfflineHttpCacheSnapshot>(raw) }.getOrNull() }
            .toList()
            .let { snapshots ->
                OfflineHttpCacheStatus(
                    cachedResponseCount = snapshots.size,
                    lastUpdatedAtMillis = snapshots.maxOfOrNull { it.cachedAtMillis },
                )
            }

    private fun refreshStatus(): OfflineHttpCacheStatus {
        _status.value = loadStatus()
        return _status.value
    }

    private fun cachePreferenceKey(key: String): String = "http_cache_${sha256(key)}"
}

class InMemoryOfflineHttpCacheStore : OfflineHttpCacheStore {
    private val entries = linkedMapOf<String, OfflineHttpCacheEntry>()
    private val _status = MutableStateFlow(OfflineHttpCacheStatus())
    override val status: StateFlow<OfflineHttpCacheStatus> = _status.asStateFlow()

    override suspend fun read(key: String): OfflineHttpCacheEntry? = entries[key]

    override suspend fun write(key: String, body: String): OfflineHttpCacheStatus {
        entries[key] = OfflineHttpCacheEntry(body = body, cachedAtMillis = System.currentTimeMillis())
        return refreshStatus()
    }

    override suspend fun clear(): OfflineHttpCacheStatus {
        entries.clear()
        _status.value = OfflineHttpCacheStatus()
        return _status.value
    }

    private fun refreshStatus(): OfflineHttpCacheStatus {
        _status.value = OfflineHttpCacheStatus(
            cachedResponseCount = entries.size,
            lastUpdatedAtMillis = entries.values.maxOfOrNull { it.cachedAtMillis },
        )
        return _status.value
    }
}

@Serializable
private data class OfflineHttpCacheSnapshot(
    val body: String,
    val cachedAtMillis: Long,
)

private val json = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

private fun sha256(value: String): String {
    val digest = MessageDigest.getInstance("SHA-256").digest(value.toByteArray(Charsets.UTF_8))
    return digest.joinToString("") { byte -> "%02x".format(byte) }
}
