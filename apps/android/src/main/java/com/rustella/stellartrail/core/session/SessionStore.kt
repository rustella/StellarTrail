package com.rustella.stellartrail.core.session

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.UserSession
import com.rustella.stellartrail.domain.auth.toSession
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

interface SessionStore {
    val session: StateFlow<UserSession?>
    fun currentToken(): String?
    fun currentRefreshToken(): String?
    fun save(response: LoginResponse)
    fun save(session: UserSession)
    fun clear()
}

class AndroidSessionStore(
    context: Context,
    private val json: Json = Json { ignoreUnknownKeys = true; explicitNulls = false },
) : SessionStore {
    private val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()
    private val preferences = EncryptedSharedPreferences.create(
        context,
        "stellartrail_session",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )
    private val _session = MutableStateFlow(load())

    override val session: StateFlow<UserSession?> = _session.asStateFlow()

    override fun currentToken(): String? = _session.value?.accessToken
    override fun currentRefreshToken(): String? = _session.value?.refreshToken

    override fun save(response: LoginResponse) = save(response.toSession())

    override fun save(session: UserSession) {
        preferences.edit().putString(KEY_SESSION, json.encodeToString(session)).apply()
        _session.value = session
    }

    override fun clear() {
        preferences.edit().remove(KEY_SESSION).apply()
        _session.value = null
    }

    private fun load(): UserSession? = preferences.getString(KEY_SESSION, null)?.let { raw ->
        runCatching { json.decodeFromString<UserSession>(raw) }.getOrNull()
    }

    private companion object {
        const val KEY_SESSION = "session"
    }
}

class InMemorySessionStore(initial: UserSession? = null) : SessionStore {
    private val _session = MutableStateFlow(initial)
    override val session: StateFlow<UserSession?> = _session.asStateFlow()
    override fun currentToken(): String? = _session.value?.accessToken
    override fun currentRefreshToken(): String? = _session.value?.refreshToken
    override fun save(response: LoginResponse) = save(response.toSession())
    override fun save(session: UserSession) {
        _session.value = session
    }
    override fun clear() {
        _session.value = null
    }
}
