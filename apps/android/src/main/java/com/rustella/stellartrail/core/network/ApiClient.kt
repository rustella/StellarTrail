package com.rustella.stellartrail.core.network

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.RefreshTokenRequest
import com.rustella.stellartrail.domain.skills.SkillLocale
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.accept
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.header
import io.ktor.client.request.prepareRequest
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.URLBuilder
import io.ktor.http.appendPathSegments
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.http.takeFrom
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

/** Thin HTTP boundary around the existing StellarTrail Rust JSON API. */
class ApiClient(
    private val configProvider: () -> AppConfig,
    @PublishedApi internal val tokenProvider: () -> String? = { null },
    @PublishedApi internal val refreshTokenProvider: () -> String? = { null },
    @PublishedApi internal val sessionRefreshHandler: suspend (LoginResponse) -> Unit = {},
    @PublishedApi internal val sessionExpiredHandler: () -> Unit = {},
    @PublishedApi internal val httpClient: HttpClient = defaultHttpClient(),
    @PublishedApi internal val json: Json = defaultJson,
) {
    val baseUrl: String get() = configProvider().baseUrl

    suspend inline fun <reified Response> get(
        path: String,
        query: Map<String, String?> = emptyMap(),
        locale: SkillLocale? = null,
    ): Response = send(HttpMethod.Get, path, query, locale) {
        accept(ContentType.Application.Json)
    }

    suspend inline fun <reified Request : Any, reified Response> post(
        path: String,
        request: Request,
    ): Response = send(HttpMethod.Post, path) {
        contentType(ContentType.Application.Json)
        setBody(request)
    }

    suspend inline fun <reified Request : Any, reified Response> patch(
        path: String,
        request: Request,
    ): Response = send(HttpMethod.Patch, path) {
        contentType(ContentType.Application.Json)
        setBody(request)
    }

    suspend fun delete(path: String) {
        send<Unit>(HttpMethod.Delete, path)
    }

    suspend inline fun <reified Response> send(
        method: HttpMethod,
        path: String,
        query: Map<String, String?> = emptyMap(),
        locale: SkillLocale? = null,
        crossinline configure: HttpRequestBuilder.() -> Unit = {},
    ): Response {
        var response = httpClient.prepareRequest(buildUrl(path, query)) {
            this.method = method
            attachAuthAndDefaults(locale)
            configure()
        }.execute()
        if (response.status == HttpStatusCode.Unauthorized && canRefreshAfterUnauthorized(path)) {
            val refreshed = refreshWithStoredToken()
            if (refreshed) {
                response = httpClient.prepareRequest(buildUrl(path, query)) {
                    this.method = method
                    attachAuthAndDefaults(locale)
                    configure()
                }.execute()
            }
        }
        val text = response.bodyAsText()
        if (!response.status.isSuccess()) {
            throw ApiException.from(response.status, text, json)
        }
        if (Response::class == Unit::class) {
            @Suppress("UNCHECKED_CAST")
            return Unit as Response
        }
        return json.decodeFromString(text)
    }

    fun resolveAssetUrl(pathOrUrl: String): String {
        if (pathOrUrl.startsWith("http://") || pathOrUrl.startsWith("https://")) return pathOrUrl
        return configProvider().assetsBaseUrl.trimEnd('/') + "/" + pathOrUrl.trimStart('/')
    }

    @PublishedApi
    internal fun buildUrl(path: String, query: Map<String, String?> = emptyMap()): String {
        val builder = URLBuilder().takeFrom(baseUrl)
        val cleanPath = path.trimStart('/')
        if (cleanPath.isNotEmpty()) {
            builder.appendPathSegments(cleanPath.split('/'))
        }
        query.forEach { (key, value) ->
            if (!value.isNullOrBlank()) builder.parameters.append(key, value)
        }
        return builder.buildString()
    }

    @PublishedApi
    internal fun HttpRequestBuilder.attachAuthAndDefaults(locale: SkillLocale?) {
        accept(ContentType.Application.Json)
        tokenProvider()?.takeIf { it.isNotBlank() }?.let { bearerAuth(it) }
        locale?.let { header("X-StellarTrail-Locale", it.headerValue) }
    }

    @PublishedApi
    internal fun canRefreshAfterUnauthorized(path: String): Boolean = !path.startsWith("/api/auth/")

    @PublishedApi
    internal suspend fun refreshWithStoredToken(): Boolean {
        val refreshToken = refreshTokenProvider()?.takeIf { it.isNotBlank() } ?: return false
        return try {
            val response = post<RefreshTokenRequest, LoginResponse>(
                "/api/auth/refresh",
                RefreshTokenRequest(refreshToken),
            )
            sessionRefreshHandler(response)
            true
        } catch (error: Throwable) {
            if (error is CancellationException) throw error
            sessionExpiredHandler()
            false
        }
    }

    companion object {
        val defaultJson: Json = Json {
            ignoreUnknownKeys = true
            explicitNulls = false
            encodeDefaults = false
        }

        fun defaultHttpClient(): HttpClient = HttpClient(OkHttp) {
            install(ContentNegotiation) {
                json(defaultJson)
            }
        }
    }
}
