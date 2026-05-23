package com.rustella.stellartrail.core.network

import android.util.Log
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
import okhttp3.Call
import okhttp3.EventListener
import okhttp3.Handshake
import okhttp3.Protocol
import java.io.IOException
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Proxy

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
        val requestUrl = buildUrl(path, query)
        try {
            var response = httpClient.prepareRequest(requestUrl) {
                this.method = method
                attachAuthAndDefaults(locale)
                configure()
            }.execute()
            if (response.status == HttpStatusCode.Unauthorized && canRefreshAfterUnauthorized(path)) {
                val refreshed = refreshWithStoredToken()
                if (refreshed) {
                    response = httpClient.prepareRequest(requestUrl) {
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
        } catch (error: Throwable) {
            if (error is CancellationException) throw error
            Log.w(
                NETWORK_LOG_TAG,
                "${method.value} ${requestUrl.substringBefore('?')} failed: ${error::class.java.name}: ${error.message}",
            )
            throw error
        }
    }

    fun resolveAssetUrl(pathOrUrl: String): String {
        if (pathOrUrl.startsWith("http://") || pathOrUrl.startsWith("https://")) return pathOrUrl
        return configProvider().assetsBaseUrl.trimEnd('/') + "/" + pathOrUrl.trimStart('/')
    }

    @PublishedApi
    internal fun buildUrl(path: String, query: Map<String, String?> = emptyMap()): String {
        val builder = URLBuilder().takeFrom(baseUrl)
        val cleanPath = versionedApiPath(path).trimStart('/')
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
    internal fun canRefreshAfterUnauthorized(path: String): Boolean =
        !versionedApiPath(path).startsWith("$API_PREFIX/auth/")

    @PublishedApi
    internal suspend fun refreshWithStoredToken(): Boolean {
        val refreshToken = refreshTokenProvider()?.takeIf { it.isNotBlank() } ?: return false
        return try {
            val response = post<RefreshTokenRequest, LoginResponse>(
                "/auth/refresh",
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
            engine {
                config {
                    eventListenerFactory { NetworkDiagnosticsEventListener() }
                }
            }
            install(ContentNegotiation) {
                json(defaultJson)
            }
        }
    }
}

@PublishedApi
internal const val NETWORK_LOG_TAG = "StellarTrailApi"

@PublishedApi
internal const val API_PREFIX = "/api/v1"

@PublishedApi
internal fun versionedApiPath(path: String): String {
    if (path == "/healthz" || path.startsWith("$API_PREFIX/")) return path
    val normalized = if (path.startsWith('/')) path else "/$path"
    return API_PREFIX + normalized
}

private class NetworkDiagnosticsEventListener : EventListener() {
    override fun dnsStart(call: Call, domainName: String) {
        Log.i(NETWORK_LOG_TAG, "dnsStart ${call.label()} domain=$domainName")
    }

    override fun dnsEnd(call: Call, domainName: String, inetAddressList: List<InetAddress>) {
        val addresses = inetAddressList.joinToString(",") { it.hostAddress ?: it.hostName }
        Log.i(NETWORK_LOG_TAG, "dnsEnd ${call.label()} domain=$domainName addresses=$addresses")
    }

    override fun connectStart(call: Call, inetSocketAddress: InetSocketAddress, proxy: Proxy) {
        Log.i(NETWORK_LOG_TAG, "connectStart ${call.label()} target=${inetSocketAddress.label()} proxy=${proxy.type()}")
    }

    override fun secureConnectStart(call: Call) {
        Log.i(NETWORK_LOG_TAG, "tlsStart ${call.label()}")
    }

    override fun secureConnectEnd(call: Call, handshake: Handshake?) {
        Log.i(
            NETWORK_LOG_TAG,
            "tlsEnd ${call.label()} version=${handshake?.tlsVersion} cipher=${handshake?.cipherSuite}",
        )
    }

    override fun connectEnd(call: Call, inetSocketAddress: InetSocketAddress, proxy: Proxy, protocol: Protocol?) {
        Log.i(NETWORK_LOG_TAG, "connectEnd ${call.label()} target=${inetSocketAddress.label()} protocol=$protocol")
    }

    override fun connectFailed(
        call: Call,
        inetSocketAddress: InetSocketAddress,
        proxy: Proxy,
        protocol: Protocol?,
        ioe: IOException,
    ) {
        Log.w(
            NETWORK_LOG_TAG,
            "connectFailed ${call.label()} target=${inetSocketAddress.label()} protocol=$protocol error=${ioe::class.java.name}: ${ioe.message}",
        )
    }

    override fun responseHeadersEnd(call: Call, response: okhttp3.Response) {
        Log.i(NETWORK_LOG_TAG, "responseHeaders ${call.label()} code=${response.code}")
    }

    override fun callFailed(call: Call, ioe: IOException) {
        Log.w(NETWORK_LOG_TAG, "callFailed ${call.label()} error=${ioe::class.java.name}: ${ioe.message}")
    }

    private fun Call.label(): String {
        val url = request().url
        return "${request().method} ${url.host}${url.encodedPath}"
    }

    private fun InetSocketAddress.label(): String {
        val host = address?.hostAddress ?: hostString
        return "$host:$port"
    }
}
