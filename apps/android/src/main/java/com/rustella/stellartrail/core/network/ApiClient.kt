package com.rustella.stellartrail.core.network

import android.util.Log
import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.config.AppCertificatePin
import com.rustella.stellartrail.core.config.AppDomainCandidate
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
import io.ktor.client.statement.HttpStatement
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.URLBuilder
import io.ktor.http.appendPathSegments
import io.ktor.http.content.ByteArrayContent
import io.ktor.http.content.TextContent
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.http.takeFrom
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.encodeToJsonElement
import okhttp3.Call
import okhttp3.CertificatePinner
import okhttp3.EventListener
import okhttp3.Handshake
import okhttp3.Protocol
import java.io.ByteArrayOutputStream
import java.io.IOException
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Proxy
import java.net.URI
import java.security.MessageDigest
import java.util.UUID
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

/** Thin HTTP boundary around the existing StellarTrail Rust JSON API. */
class ApiClient(
    @PublishedApi internal val configProvider: () -> AppConfig,
    @PublishedApi internal val tokenProvider: () -> String? = { null },
    @PublishedApi internal val refreshTokenProvider: () -> String? = { null },
    @PublishedApi internal val sessionRefreshHandler: suspend (LoginResponse) -> Unit = {},
    @PublishedApi internal val sessionExpiredHandler: () -> Unit = {},
    @PublishedApi internal val offlineCacheStore: OfflineHttpCacheStore? = null,
    @PublishedApi internal val cacheScopeProvider: () -> String = { CACHE_SCOPE_GUEST },
    @PublishedApi internal val httpClient: HttpClient = defaultHttpClient(configProvider().certificatePins),
    @PublishedApi internal val json: Json = defaultJson,
    private val domainProbeTimeoutMillis: Long? = API_DOMAIN_HEALTH_TIMEOUT_MS,
    @PublishedApi internal val nonceProvider: () -> String = { UUID.randomUUID().toString() },
) {
    private val domainProbeMutex = Mutex()
    private val tokenRefreshMutex = Mutex()
    @Volatile
    private var domainProbeCompletedForBaseUrl: String? = null
    @Volatile
    private var selectedDomainConfig: AppConfig? = null

    val baseUrl: String get() = activeConfig().baseUrl

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
    ): Response = sendJson(HttpMethod.Post, path, request)

    suspend inline fun <reified Request : Any, reified Response> patch(
        path: String,
        request: Request,
    ): Response = sendJson(HttpMethod.Patch, path, request)

    suspend inline fun <reified Request : Any, reified Response> put(
        path: String,
        request: Request,
    ): Response = sendJson(HttpMethod.Put, path, request)

    suspend fun delete(path: String) {
        send<Unit>(HttpMethod.Delete, path)
    }

    suspend inline fun <reified Response> deleteReturning(path: String): Response =
        send(HttpMethod.Delete, path)

    suspend inline fun <reified Response> send(
        method: HttpMethod,
        path: String,
        query: Map<String, String?> = emptyMap(),
        locale: SkillLocale? = null,
        crossinline configure: HttpRequestBuilder.() -> Unit = {},
    ): Response {
        val cacheKey = cacheKeyForRequest(method, configProvider(), path, query, locale)
        var requestUrl = buildUrl(path, query)
        try {
            var prepared = prepareQueryRequest(method, path, query, locale, configure).also { requestUrl = it.url }
            var response = prepared.request.execute()
            if (response.status == HttpStatusCode.Unauthorized && canRefreshAfterUnauthorized(path)) {
                val refreshed = refreshWithStoredToken(prepared.accessToken)
                if (refreshed) {
                    prepared = prepareQueryRequest(method, path, query, locale, configure).also { requestUrl = it.url }
                    response = prepared.request.execute()
                }
            }
            val text = response.bodyAsText()
            if (!response.status.isSuccess()) {
                throw ApiException.from(response.status, text, json, response.headers["Retry-After"])
            }
            if (Response::class == Unit::class) {
                @Suppress("UNCHECKED_CAST")
                return Unit as Response
            }
            cacheKey?.let { offlineCacheStore?.write(it, text) }
            return json.decodeFromString(text)
        } catch (error: ApiException) {
            throw error
        } catch (error: Throwable) {
            if (error is CancellationException) throw error
            val cachedBody = cacheKey?.takeIf { error.isOfflineReplayableFailure() }?.let {
                offlineCacheStore?.read(it)?.body
            }
            if (cachedBody != null) {
                logNetworkWarning(
                    "${method.value} ${buildUrl(path, query).substringBefore('?')} failed offline; using cached response.",
                )
                if (Response::class == Unit::class) {
                    @Suppress("UNCHECKED_CAST")
                    return Unit as Response
                }
                return json.decodeFromString(cachedBody)
            }
            logNetworkWarning(
                "${method.value} ${requestUrl.substringBefore('?')} failed: ${error::class.java.name}: ${error.message}",
            )
            throw error
        }
    }

    suspend inline fun <reified Response> uploadFile(
        path: String,
        bytes: ByteArray,
        filename: String,
        contentType: String? = null,
    ): Response {
        val boundary = "StellarTrail-${nonceProvider()}"
        val bodyBytes = multipartFileBody("file", filename, contentType, bytes, boundary)
        val requestContentType = ContentType.parse("multipart/form-data; boundary=$boundary")
        var requestUrl = buildUrl(path)
        try {
            var prepared = prepareBinaryRequest(HttpMethod.Post, path, bodyBytes, requestContentType).also { requestUrl = it.url }
            var response = prepared.request.execute()
            if (response.status == HttpStatusCode.Unauthorized && canRefreshAfterUnauthorized(path)) {
                val refreshed = refreshWithStoredToken(prepared.accessToken)
                if (refreshed) {
                    prepared = prepareBinaryRequest(HttpMethod.Post, path, bodyBytes, requestContentType).also { requestUrl = it.url }
                    response = prepared.request.execute()
                }
            }
            val text = response.bodyAsText()
            if (!response.status.isSuccess()) {
                throw ApiException.from(response.status, text, json, response.headers["Retry-After"])
            }
            return json.decodeFromString(text)
        } catch (error: ApiException) {
            throw error
        } catch (error: Throwable) {
            if (error is CancellationException) throw error
            logNetworkWarning(
                "POST ${requestUrl.substringBefore('?')} failed: ${error::class.java.name}: ${error.message}",
            )
            throw error
        }
    }


    @PublishedApi
    internal suspend inline fun <reified Request : Any, reified Response> sendJson(
        method: HttpMethod,
        path: String,
        request: Request,
    ): Response {
        val unsignedBody = json.encodeToJsonElement(request)
        var requestUrl = buildUrl(path)
        try {
            var prepared = prepareJsonRequest(method, path, unsignedBody).also { requestUrl = it.url }
            var response = prepared.request.execute()
            if (response.status == HttpStatusCode.Unauthorized && canRefreshAfterUnauthorized(path)) {
                val refreshed = refreshWithStoredToken(prepared.accessToken)
                if (refreshed) {
                    prepared = prepareJsonRequest(method, path, unsignedBody).also { requestUrl = it.url }
                    response = prepared.request.execute()
                }
            }
            val text = response.bodyAsText()
            if (!response.status.isSuccess()) {
                throw ApiException.from(response.status, text, json, response.headers["Retry-After"])
            }
            if (Response::class == Unit::class) {
                @Suppress("UNCHECKED_CAST")
                return Unit as Response
            }
            return json.decodeFromString(text)
        } catch (error: ApiException) {
            throw error
        } catch (error: Throwable) {
            if (error is CancellationException) throw error
            logNetworkWarning(
                "${method.value} ${requestUrl.substringBefore('?')} failed: ${error::class.java.name}: ${error.message}",
            )
            throw error
        }
    }

    @PublishedApi
    internal suspend inline fun prepareQueryRequest(
        method: HttpMethod,
        path: String,
        query: Map<String, String?>,
        locale: SkillLocale?,
        crossinline configure: HttpRequestBuilder.() -> Unit,
    ): PreparedApiRequest {
        val requestConfig = configForRequest(path)
        val requestUrl = buildSignedUrl(requestConfig, method, path, query, EMPTY_BODY_SHA256_HEX)
        val accessToken = tokenProvider()?.takeIf { it.isNotBlank() }
        val request = httpClient.prepareRequest(requestUrl) {
            this.method = method
            attachAuthAndDefaults(locale, accessToken)
            configure()
        }
        return PreparedApiRequest(requestUrl, request, accessToken)
    }

    @PublishedApi
    internal suspend fun prepareJsonRequest(
        method: HttpMethod,
        path: String,
        unsignedBody: JsonElement,
    ): PreparedApiRequest {
        val requestConfig = configForRequest(path)
        val requestUrl = buildUrl(requestConfig, path)
        val signedBody = signedJsonBody(requestConfig, method, path, requestUrl, unsignedBody)
        val bodyText = json.encodeToString(JsonElement.serializer(), signedBody)
        val accessToken = tokenProvider()?.takeIf { it.isNotBlank() }
        val request = httpClient.prepareRequest(requestUrl) {
            this.method = method
            attachAuthAndDefaults(locale = null, accessToken = accessToken)
            contentType(ContentType.Application.Json)
            setBody(TextContent(bodyText, ContentType.Application.Json))
        }
        return PreparedApiRequest(requestUrl, request, accessToken)
    }

    @PublishedApi
    internal suspend fun prepareBinaryRequest(
        method: HttpMethod,
        path: String,
        bodyBytes: ByteArray,
        requestContentType: ContentType,
    ): PreparedApiRequest {
        val requestConfig = configForRequest(path)
        val requestUrl = buildSignedUrl(requestConfig, method, path, bodyHashHex = sha256Hex(bodyBytes))
        val accessToken = tokenProvider()?.takeIf { it.isNotBlank() }
        val request = httpClient.prepareRequest(requestUrl) {
            this.method = method
            attachAuthAndDefaults(locale = null, accessToken = accessToken)
            setBody(ByteArrayContent(bodyBytes, requestContentType))
        }
        return PreparedApiRequest(requestUrl, request, accessToken)
    }


    @PublishedApi
    internal fun buildSignedUrl(
        config: AppConfig,
        method: HttpMethod,
        path: String,
        query: Map<String, String?> = emptyMap(),
        bodyHashHex: String = EMPTY_BODY_SHA256_HEX,
    ): String {
        val credentials = config.requestSignature
        if (credentials == null || !shouldSignRequest(method, path)) {
            return buildUrl(config, path, query)
        }
        val nonce = nonceProvider().trim()
        val signingQuery = query.withoutSigningFields() + mapOf(
            SIGNING_FIELD_APP_ID to credentials.appId,
            SIGNING_FIELD_NONCE to nonce,
        )
        val unsignedUrl = buildUrl(config, path, signingQuery)
        val canonical = canonicalRequest(
            method = method.value,
            path = URI(unsignedUrl).rawPath,
            canonicalQuery = canonicalQuery(URI(unsignedUrl).rawQuery.orEmpty()),
            bodyHashHex = bodyHashHex,
            appId = credentials.appId,
            nonce = nonce,
        )
        val signature = hmacSha256Hex(credentials.appSecret, canonical)
        return buildUrl(config, path, signingQuery + mapOf(SIGNING_FIELD_SIGNATURE to signature))
    }

    @PublishedApi
    internal fun signedJsonBody(
        config: AppConfig,
        method: HttpMethod,
        path: String,
        requestUrl: String,
        unsignedBody: JsonElement,
    ): JsonElement {
        val credentials = config.requestSignature
        if (credentials == null || !shouldSignRequest(method, path)) return unsignedBody
        val bodyObject = unsignedBody as? JsonObject
            ?: error("Signed JSON requests must encode to a top-level JSON object.")
        val nonce = nonceProvider().trim()
        val bodyHash = sha256Hex(canonicalJsonBodyForSigning(bodyObject).encodeToByteArray())
        val requestUri = URI(requestUrl)
        val canonical = canonicalRequest(
            method = method.value,
            path = requestUri.rawPath,
            canonicalQuery = canonicalQuery(requestUri.rawQuery.orEmpty()),
            bodyHashHex = bodyHash,
            appId = credentials.appId,
            nonce = nonce,
        )
        val signature = hmacSha256Hex(credentials.appSecret, canonical)
        return JsonObject(
            bodyObject + mapOf(
                SIGNING_FIELD_APP_ID to JsonPrimitive(credentials.appId),
                SIGNING_FIELD_NONCE to JsonPrimitive(nonce),
                SIGNING_FIELD_SIGNATURE to JsonPrimitive(signature),
            ),
        )
    }

    fun resolveAssetUrl(pathOrUrl: String): String {
        if (pathOrUrl.startsWith("http://") || pathOrUrl.startsWith("https://")) {
            return normalizeKnownAssetUrl(pathOrUrl)
        }
        if (
            pathOrUrl.startsWith("android.resource://") ||
            pathOrUrl.startsWith("content://") ||
            pathOrUrl.startsWith("file://")
        ) {
            return pathOrUrl
        }
        return activeConfig().assetsBaseUrl.trimEnd('/') + "/" + pathOrUrl.trimStart('/')
    }

    @PublishedApi
    internal fun buildUrl(path: String, query: Map<String, String?> = emptyMap()): String {
        return buildUrl(activeConfig(), path, query)
    }

    @PublishedApi
    internal fun buildUrl(
        config: AppConfig,
        path: String,
        query: Map<String, String?> = emptyMap(),
    ): String {
        val builder = URLBuilder().takeFrom(config.baseUrl)
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
    internal suspend fun configForRequest(path: String): AppConfig {
        if (path != HEALTH_PATH) {
            ensureProductionDomainSelected()
        }
        return activeConfig()
    }

    private fun activeConfig(): AppConfig {
        val current = configProvider()
        return if (domainProbeCompletedForBaseUrl == current.baseUrl) {
            selectedDomainConfig ?: current
        } else {
            current
        }
    }

    private suspend fun ensureProductionDomainSelected() {
        val current = configProvider()
        if (!shouldProbeProductionDomains(current.baseUrl, current.domainCandidates)) {
            selectedDomainConfig = null
            domainProbeCompletedForBaseUrl = current.baseUrl
            return
        }
        if (domainProbeCompletedForBaseUrl == current.baseUrl) return
        domainProbeMutex.withLock {
            val latest = configProvider()
            if (!shouldProbeProductionDomains(latest.baseUrl, latest.domainCandidates)) {
                selectedDomainConfig = null
                domainProbeCompletedForBaseUrl = latest.baseUrl
                return
            }
            if (domainProbeCompletedForBaseUrl == latest.baseUrl) return
            for (candidate in latest.domainCandidates) {
                if (probeHealthz(candidate.apiBaseUrl)) {
                    selectedDomainConfig = AppConfig(
                        baseUrl = candidate.apiBaseUrl,
                        assetsBaseUrl = candidate.assetsBaseUrl,
                        domainCandidates = latest.domainCandidates,
                        clientIdentity = latest.clientIdentity,
                        requestSignature = latest.requestSignature,
                        certificatePins = latest.certificatePins,
                    )
                    domainProbeCompletedForBaseUrl = latest.baseUrl
                    return
                }
            }
            val fallback = latest.domainCandidates.first()
            selectedDomainConfig = AppConfig(
                baseUrl = fallback.apiBaseUrl,
                assetsBaseUrl = fallback.assetsBaseUrl,
                domainCandidates = latest.domainCandidates,
                clientIdentity = latest.clientIdentity,
                requestSignature = latest.requestSignature,
                certificatePins = latest.certificatePins,
            )
            domainProbeCompletedForBaseUrl = latest.baseUrl
        }
    }

    private suspend fun probeHealthz(apiBaseUrl: String): Boolean {
        val response = if (domainProbeTimeoutMillis == null) {
            executeHealthzProbe(apiBaseUrl)
        } else {
            withTimeoutOrNull(domainProbeTimeoutMillis) { executeHealthzProbe(apiBaseUrl) }
        } ?: return false
        return response.status.isSuccess()
    }

    private suspend fun executeHealthzProbe(apiBaseUrl: String) = runCatching {
        httpClient.prepareRequest(apiBaseUrl.trimEnd('/') + HEALTH_PATH) {
            method = HttpMethod.Get
            accept(ContentType.Application.Json)
            header("X-StellarTrail-Client", configProvider().clientIdentity)
        }.execute()
    }.getOrNull()

    private fun shouldProbeProductionDomains(apiBaseUrl: String, candidates: List<AppDomainCandidate>): Boolean =
        candidates.any { it.apiBaseUrl == sanitizeComparableBaseUrl(apiBaseUrl) }

    private fun normalizeKnownAssetUrl(url: String): String {
        val parsed = runCatching { URI(url) }.getOrNull() ?: return url
        val host = parsed.host?.lowercase() ?: return url
        val activeConfig = activeConfig()
        if (!activeConfig.isKnownAssetsHost(host)) return url
        val rawPath = parsed.rawPath.orEmpty()
        val rawQuery = parsed.rawQuery?.let { "?$it" }.orEmpty()
        val rawFragment = parsed.rawFragment?.let { "#$it" }.orEmpty()
        return activeConfig.assetsBaseUrl.trimEnd('/') + rawPath + rawQuery + rawFragment
    }

    @PublishedApi
    internal fun HttpRequestBuilder.attachAuthAndDefaults(locale: SkillLocale?, accessToken: String? = tokenProvider()?.takeIf { it.isNotBlank() }) {
        accept(ContentType.Application.Json)
        header("X-StellarTrail-Client", activeConfig().clientIdentity)
        accessToken?.let { bearerAuth(it) }
        locale?.let { header("X-StellarTrail-Locale", it.headerValue) }
    }

    @PublishedApi
    internal fun canRefreshAfterUnauthorized(path: String): Boolean =
        !versionedApiPath(path).startsWith("$API_PREFIX/auth/")

    @PublishedApi
    internal fun cacheKeyForRequest(
        method: HttpMethod,
        config: AppConfig,
        path: String,
        query: Map<String, String?>,
        locale: SkillLocale?,
    ): String? {
        if (method != HttpMethod.Get || path == HEALTH_PATH) return null
        val scope = cacheScopeProvider().trim().takeIf { it.isNotEmpty() } ?: CACHE_SCOPE_GUEST
        val queryKey = query.entries
            .asSequence()
            .filter { (_, value) -> !value.isNullOrBlank() }
            .sortedWith(compareBy({ it.key }, { it.value.orEmpty() }))
            .joinToString("&") { (key, value) -> "$key=$value" }
        return listOf(
            CACHE_KEY_VERSION,
            scope,
            sanitizeComparableBaseUrl(config.baseUrl),
            method.value,
            versionedApiPath(path),
            queryKey,
            locale?.headerValue.orEmpty(),
        ).joinToString("|")
    }

    @PublishedApi
    internal suspend fun refreshWithStoredToken(accessTokenUsed: String? = null): Boolean = tokenRefreshMutex.withLock {
        val currentAccessToken = tokenProvider()?.takeIf { it.isNotBlank() }
        if (accessTokenUsed != null && currentAccessToken != null && currentAccessToken != accessTokenUsed) {
            return@withLock true
        }
        val refreshToken = refreshTokenProvider()?.takeIf { it.isNotBlank() } ?: return@withLock false
        try {
            val response = post<RefreshTokenRequest, LoginResponse>(
                "/auth/refresh",
                RefreshTokenRequest(refreshToken),
            )
            sessionRefreshHandler(response)
            true
        } catch (error: ApiException) {
            if (error.isUnauthorized || error.errorCode == "unauthorized") {
                sessionExpiredHandler()
            }
            false
        } catch (error: Throwable) {
            if (error is CancellationException) throw error
            false
        }
    }

    companion object {
        val defaultJson: Json = Json {
            ignoreUnknownKeys = true
            explicitNulls = false
            encodeDefaults = false
        }

        fun defaultHttpClient(certificatePins: List<AppCertificatePin> = emptyList()): HttpClient = HttpClient(OkHttp) {
            engine {
                config {
                    eventListenerFactory { NetworkDiagnosticsEventListener() }
                    if (certificatePins.isNotEmpty()) {
                        val pinner = CertificatePinner.Builder().apply {
                            certificatePins.forEach { add(it.hostname, it.pin) }
                        }.build()
                        certificatePinner(pinner)
                    }
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
internal const val HEALTH_PATH = "/healthz"

private const val API_DOMAIN_HEALTH_TIMEOUT_MS = 3_000L
private const val CACHE_KEY_VERSION = "v1"
private const val CACHE_SCOPE_GUEST = "guest"
private const val SIGNATURE_ALGORITHM = "STELLARTRAIL-HMAC-SHA256"
private const val SIGNING_FIELD_APP_ID = "app_id"
private const val SIGNING_FIELD_NONCE = "nonce"
private const val SIGNING_FIELD_SIGNATURE = "signature"
private val SIGNING_FIELD_NAMES = setOf(SIGNING_FIELD_APP_ID, SIGNING_FIELD_NONCE, SIGNING_FIELD_SIGNATURE)

@PublishedApi
internal val EMPTY_BODY_SHA256_HEX: String = sha256Hex(ByteArray(0))

@PublishedApi
internal data class PreparedApiRequest(
    val url: String,
    val request: HttpStatement,
    val accessToken: String?,
)

@PublishedApi
internal fun logNetworkWarning(message: String) {
    runCatching { Log.w(NETWORK_LOG_TAG, message) }
}

@PublishedApi
internal fun Throwable.isOfflineReplayableFailure(): Boolean {
    var current: Throwable? = this
    while (current != null) {
        if (current is IOException) return true
        current = current.cause
    }
    return false
}

private fun sanitizeComparableBaseUrl(baseUrl: String): String = baseUrl.trim().trimEnd('/')

private fun AppConfig.isKnownAssetsHost(host: String): Boolean {
    val currentAssetsHost = runCatching { URI(assetsBaseUrl).host?.lowercase() }.getOrNull()
    if (currentAssetsHost == host) return true
    return domainCandidates.any { candidate ->
        runCatching { URI(candidate.assetsBaseUrl).host?.lowercase() }.getOrNull() == host
    }
}

@PublishedApi
internal fun versionedApiPath(path: String): String {
    if (path == HEALTH_PATH || path.startsWith("$API_PREFIX/")) return path
    val normalized = if (path.startsWith('/')) path else "/$path"
    return API_PREFIX + normalized
}

private fun shouldSignRequest(method: HttpMethod, path: String): Boolean {
    if (method == HttpMethod.Options) return false
    val normalizedPath = versionedApiPath(path)
    return normalizedPath.startsWith("$API_PREFIX/") && !isSignatureExemptPath(normalizedPath)
}

private fun isSignatureExemptPath(path: String): Boolean =
    path == "/healthz" ||
        path == "/ping" ||
        path == "/echo" ||
        path == "/api/v1/ping" ||
        path == "/api/v1/echo"

private fun Map<String, String?>.withoutSigningFields(): Map<String, String?> =
    filterKeys { it !in SIGNING_FIELD_NAMES }

private fun canonicalRequest(
    method: String,
    path: String,
    canonicalQuery: String,
    bodyHashHex: String,
    appId: String,
    nonce: String,
): String = listOf(
    SIGNATURE_ALGORITHM,
    method,
    path,
    canonicalQuery,
    bodyHashHex,
    appId,
    nonce,
).joinToString("\n")

private fun canonicalQuery(query: String): String =
    query
        .split('&')
        .asSequence()
        .filter { it.isNotEmpty() }
        .map { pair ->
            val separatorIndex = pair.indexOf('=')
            if (separatorIndex == -1) {
                pair to ""
            } else {
                pair.substring(0, separatorIndex) to pair.substring(separatorIndex + 1)
            }
        }
        .filter { (key, _) -> key != SIGNING_FIELD_SIGNATURE }
        .sortedWith(compareBy({ it.first }, { it.second }))
        .joinToString("&") { (key, value) -> "$key=$value" }

@PublishedApi
internal fun canonicalJsonBodyForSigning(body: JsonObject): String =
    canonicalJson(
        JsonObject(body.filterKeys { it !in SIGNING_FIELD_NAMES }),
    )

private fun canonicalJson(value: JsonElement): String = when (value) {
    JsonNull -> "null"
    is JsonPrimitive -> value.toString()
    is JsonArray -> value.joinToString(separator = ",", prefix = "[", postfix = "]") { canonicalJson(it) }
    is JsonObject -> value.entries
        .sortedBy { it.key }
        .joinToString(separator = ",", prefix = "{", postfix = "}") { (key, item) ->
            JsonPrimitive(key).toString() + ":" + canonicalJson(item)
        }
}

@PublishedApi
internal fun sha256Hex(bytes: ByteArray): String =
    MessageDigest.getInstance("SHA-256").digest(bytes).toHexString()

private fun hmacSha256Hex(secret: String, message: String): String {
    val mac = Mac.getInstance("HmacSHA256")
    mac.init(SecretKeySpec(secret.encodeToByteArray(), "HmacSHA256"))
    return mac.doFinal(message.encodeToByteArray()).toHexString()
}

@PublishedApi
internal fun multipartFileBody(
    fieldName: String,
    filename: String,
    contentType: String?,
    bytes: ByteArray,
    boundary: String,
): ByteArray {
    val safeFieldName = fieldName.filter { it.isLetterOrDigit() || it == '_' || it == '-' }.ifBlank { "file" }
    val safeFilename = filename
        .substringAfterLast('/')
        .substringAfterLast('\\')
        .replace('"', '_')
        .ifBlank { "trail" }
    val safeContentType = contentType?.trim()?.takeIf { it.isNotEmpty() } ?: "application/octet-stream"
    return ByteArrayOutputStream().apply {
        writeUtf8("--$boundary\r\n")
        writeUtf8("Content-Disposition: form-data; name=\"$safeFieldName\"; filename=\"$safeFilename\"\r\n")
        writeUtf8("Content-Type: $safeContentType\r\n\r\n")
        write(bytes)
        writeUtf8("\r\n--$boundary--\r\n")
    }.toByteArray()
}

private fun ByteArrayOutputStream.writeUtf8(value: String) {
    write(value.toByteArray(Charsets.UTF_8))
}


private fun ByteArray.toHexString(): String = joinToString(separator = "") { byte ->
    "%02x".format(byte.toInt() and 0xff)
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
