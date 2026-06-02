package com.rustella.stellartrail.data.skills

import android.content.Context
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

data class KnotCacheStatus(
    val cachedKnotCount: Int = 0,
    val lastUpdatedAtMillis: Long? = null,
)

interface KnotCacheStore {
    val status: StateFlow<KnotCacheStatus>
    suspend fun save(locale: SkillLocale, details: List<KnotDetail>): KnotCacheStatus
    suspend fun clear(): KnotCacheStatus
    suspend fun readDetails(locale: SkillLocale): List<KnotDetail>
    suspend fun findDetail(id: String, locale: SkillLocale): KnotDetail?
}

class AndroidKnotCacheStore(context: Context) : KnotCacheStore {
    private val preferences = context.getSharedPreferences("stellartrail_knot_cache", Context.MODE_PRIVATE)
    private val _status = MutableStateFlow(loadStatus())
    override val status: StateFlow<KnotCacheStatus> = _status.asStateFlow()

    override suspend fun save(locale: SkillLocale, details: List<KnotDetail>): KnotCacheStatus {
        val snapshot = KnotCacheSnapshot(
            locale = locale,
            cachedAtMillis = System.currentTimeMillis(),
            details = details,
        )
        preferences.edit().putString(KEY_CACHE, json.encodeToString(snapshot)).apply()
        return updateStatus(snapshot)
    }

    override suspend fun clear(): KnotCacheStatus {
        preferences.edit().remove(KEY_CACHE).apply()
        _status.value = KnotCacheStatus()
        return _status.value
    }

    override suspend fun readDetails(locale: SkillLocale): List<KnotDetail> =
        loadSnapshot()
            ?.takeIf { it.locale == locale }
            ?.details
            .orEmpty()

    override suspend fun findDetail(id: String, locale: SkillLocale): KnotDetail? =
        readDetails(locale).firstOrNull { it.id == id || it.slug == id }

    private fun loadStatus(): KnotCacheStatus =
        loadSnapshot()?.let { snapshot ->
            KnotCacheStatus(
                cachedKnotCount = snapshot.details.size,
                lastUpdatedAtMillis = snapshot.cachedAtMillis,
            )
        } ?: KnotCacheStatus()

    private fun loadSnapshot(): KnotCacheSnapshot? =
        preferences.getString(KEY_CACHE, null)
            ?.let { raw -> runCatching { json.decodeFromString<KnotCacheSnapshot>(raw) }.getOrNull() }

    private fun updateStatus(snapshot: KnotCacheSnapshot): KnotCacheStatus {
        _status.value = KnotCacheStatus(
            cachedKnotCount = snapshot.details.size,
            lastUpdatedAtMillis = snapshot.cachedAtMillis,
        )
        return _status.value
    }

    private companion object {
        const val KEY_CACHE = "knot_cache"
    }
}

class InMemoryKnotCacheStore : KnotCacheStore {
    private var snapshot: KnotCacheSnapshot? = null
    private val _status = MutableStateFlow(KnotCacheStatus())
    override val status: StateFlow<KnotCacheStatus> = _status.asStateFlow()

    override suspend fun save(locale: SkillLocale, details: List<KnotDetail>): KnotCacheStatus {
        snapshot = KnotCacheSnapshot(
            locale = locale,
            cachedAtMillis = System.currentTimeMillis(),
            details = details,
        )
        _status.value = KnotCacheStatus(details.size, snapshot?.cachedAtMillis)
        return _status.value
    }

    override suspend fun clear(): KnotCacheStatus {
        snapshot = null
        _status.value = KnotCacheStatus()
        return _status.value
    }

    override suspend fun readDetails(locale: SkillLocale): List<KnotDetail> =
        snapshot?.takeIf { it.locale == locale }?.details.orEmpty()

    override suspend fun findDetail(id: String, locale: SkillLocale): KnotDetail? =
        readDetails(locale).firstOrNull { it.id == id || it.slug == id }
}

@Serializable
private data class KnotCacheSnapshot(
    val locale: SkillLocale,
    val cachedAtMillis: Long,
    val details: List<KnotDetail>,
)

private val json = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}
