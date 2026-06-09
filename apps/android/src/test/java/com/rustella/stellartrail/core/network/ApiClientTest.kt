package com.rustella.stellartrail.core.network

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.config.AppDomainCandidate
import com.rustella.stellartrail.core.config.RequestSignatureCredentials
import com.rustella.stellartrail.domain.common.HealthResponse
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.request.HttpRequestData
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.IOException
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

class ApiClientTest {
    @Test
    fun getAddsBaseUrlQueryAndBearerToken() = runTest {
        val requests = mutableListOf<io.ktor.client.request.HttpRequestData>()
        val engine = MockEngine { request ->
            requests += request
            respond(
                content = """{"status":"ok"}""",
                headers = headersOf(HttpHeaders.ContentType, "application/json"),
            )
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test/base") },
            tokenProvider = { "access-token" },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val response = client.get<HealthResponse>("/healthz", query = mapOf("q" to "trail", "empty" to ""))

        assertEquals("ok", response.status)
        val request = requests.single()
        assertEquals("/base/healthz", request.url.encodedPath)
        assertEquals("q=trail", request.url.encodedQuery)
        assertEquals("Bearer access-token", request.headers[HttpHeaders.Authorization])
        assertEquals("android/0.1.0", request.headers["X-StellarTrail-Client"])
    }

    @Test
    fun getSignsProtectedRequestQuery() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val engine = MockEngine { request ->
            requests += request
            respondJson("""{"status":"ok"}""")
        }
        val client = ApiClient(
            configProvider = { signedConfig },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
            nonceProvider = { "nonce-1" },
        )

        val response = client.get<HealthResponse>(
            "/me/gears/categories",
            query = mapOf("z" to "3", "empty" to "", "a" to "2"),
        )

        assertEquals("ok", response.status)
        val query = requests.single().url.parameters
        assertEquals("android-test-client", query["app_id"])
        assertEquals("nonce-1", query["nonce"])
        assertEquals(
            testHmacSha256Hex(
                "android-test-secret",
                listOf(
                    "STELLARTRAIL-HMAC-SHA256",
                    "GET",
                    "/api/v1/me/gears/categories",
                    "a=2&app_id=android-test-client&nonce=nonce-1&z=3",
                    EMPTY_BODY_SHA256_HEX,
                    "android-test-client",
                    "nonce-1",
                ).joinToString("\n"),
            ),
            query["signature"],
        )
    }

    @Test
    fun jsonPostAddsSignatureFieldsWithoutMutatingDto() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val engine = MockEngine { request ->
            requests += request
            respondJson("""{"status":"ok"}""")
        }
        val client = ApiClient(
            configProvider = { signedConfig },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
            nonceProvider = { "nonce-2" },
        )
        val body = buildJsonObject {
            put("b", 2)
            put("a", buildJsonObject { put("z", true) })
        }
        assertEquals("""{"a":{"z":true},"b":2}""", canonicalJsonBodyForSigning(body))

        val response = client.post<JsonObject, HealthResponse>("/me/gears", body)

        assertEquals("ok", response.status)
        assertFalse(body.containsKey("app_id"))
        val sentBody = ApiClient.defaultJson.parseToJsonElement(requests.single().bodyText()).jsonObject
        assertEquals(JsonPrimitive(2), sentBody["b"])
        assertEquals("android-test-client", sentBody["app_id"]?.jsonPrimitive?.content)
        assertEquals("nonce-2", sentBody["nonce"]?.jsonPrimitive?.content)
        assertEquals(
            testHmacSha256Hex(
                "android-test-secret",
                listOf(
                    "STELLARTRAIL-HMAC-SHA256",
                    "POST",
                    "/api/v1/me/gears",
                    "",
                    sha256Hex("""{"a":{"z":true},"b":2}""".encodeToByteArray()),
                    "android-test-client",
                    "nonce-2",
                ).joinToString("\n"),
            ),
            sentBody["signature"]?.jsonPrimitive?.content,
        )
    }

    @Test
    fun productionDomainProbeKeepsFirstHealthyDomainFamily() = runTest {
        val requests = mutableListOf<io.ktor.client.request.HttpRequestData>()
        val engine = MockEngine { request ->
            requests += request
            when (request.url.host to request.url.encodedPath) {
                "api.example.invalid" to "/healthz" -> respondJson("""{"status":"ok"}""")
                "api.example.invalid" to "/api/v1/me/gears/categories" -> respondJson("""{"status":"ok"}""")
                else -> error("unexpected request ${request.url}")
            }
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.invalid", domainCandidates = domainCandidates) },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
            domainProbeTimeoutMillis = null,
        )

        val response = client.get<HealthResponse>("/me/gears/categories")

        assertEquals("ok", response.status)
        assertEquals(
            listOf(
                "api.example.invalid/healthz",
                "api.example.invalid/api/v1/me/gears/categories",
            ),
            requests.map { "${it.url.host}${it.url.encodedPath}" },
        )
        assertEquals(
            "https://assets.example.invalid/stellartrail-knots-media/knot.webp",
            client.resolveAssetUrl("https://assets-alt2.example.invalid/stellartrail-knots-media/knot.webp"),
        )
    }

    @Test
    fun productionDomainProbeFallsThroughToSecondHealthyDomainFamily() = runTest {
        val requests = mutableListOf<io.ktor.client.request.HttpRequestData>()
        val engine = MockEngine { request ->
            requests += request
            when (request.url.host to request.url.encodedPath) {
                "api.example.invalid" to "/healthz" -> respondJson(
                    """{"status":"down"}""",
                    HttpStatusCode.ServiceUnavailable,
                )
                "api-alt1.example.invalid" to "/healthz" -> respondJson("""{"status":"ok"}""")
                "api-alt1.example.invalid" to "/api/v1/me/gears/categories" -> respondJson("""{"status":"ok"}""")
                else -> error("unexpected request ${request.url}")
            }
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.invalid", domainCandidates = domainCandidates) },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
            domainProbeTimeoutMillis = null,
        )

        val response = client.get<HealthResponse>("/me/gears/categories")

        assertEquals("ok", response.status)
        assertEquals(
            listOf(
                "api.example.invalid/healthz",
                "api-alt1.example.invalid/healthz",
                "api-alt1.example.invalid/api/v1/me/gears/categories",
            ),
            requests.map { "${it.url.host}${it.url.encodedPath}" },
        )
        assertEquals(
            "https://assets-alt1.example.invalid/stellartrail-knots-media/knot.webp",
            client.resolveAssetUrl("https://assets.example.invalid/stellartrail-knots-media/knot.webp"),
        )
    }

    @Test
    fun customNonProductionBaseUrlSkipsProductionDomainProbe() = runTest {
        val requests = mutableListOf<io.ktor.client.request.HttpRequestData>()
        val engine = MockEngine { request ->
            requests += request
            respondJson("""{"status":"ok"}""")
        }
        val client = ApiClient(
            configProvider = { AppConfig("http://10.0.2.2:8080") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val response = client.get<HealthResponse>("/me/gears/categories")

        assertEquals("ok", response.status)
        assertEquals(
            listOf("10.0.2.2/api/v1/me/gears/categories"),
            requests.map { "${it.url.host}${it.url.encodedPath}" },
        )
    }

    @Test
    fun nonSuccessResponseThrowsApiExceptionWithParsedMessage() = runTest {
        val engine = MockEngine {
            respond(
                content = """{"code":"captcha_required","message":"请输入验证码"}""",
                status = HttpStatusCode(428, "Precondition Required"),
                headers = headersOf(HttpHeaders.ContentType, "application/json"),
            )
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val exception = runCatching { client.get<HealthResponse>("/healthz") }.exceptionOrNull()

        assertTrue(exception is ApiException)
        exception as ApiException
        assertEquals(428, exception.statusCode)
        assertEquals("captcha_required", exception.errorCode)
        assertEquals("请输入验证码", exception.message)
        assertTrue(exception.isCaptchaRequired)
    }

    @Test
    fun rateLimitedResponseParsesRetryAfterSeconds() = runTest {
        val engine = MockEngine {
            respond(
                content = """{"code":"rate_limited","message":"Too many requests.","retry_after_seconds":42}""",
                status = HttpStatusCode.TooManyRequests,
                headers = headersOf(
                    HttpHeaders.ContentType to listOf("application/json"),
                    "Retry-After" to listOf("75"),
                ),
            )
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val exception = runCatching { client.get<HealthResponse>("/healthz") }.exceptionOrNull()

        assertTrue(exception is ApiException)
        exception as ApiException
        assertTrue(exception.isRateLimited)
        assertEquals(75L, exception.retryAfterSeconds)
    }

    @Test
    fun authenticatedRequestRefreshesOnceOnUnauthorizedAndRetries() = runTest {
        val requests = mutableListOf<io.ktor.client.request.HttpRequestData>()
        var accessToken = "expired-access-token"
        var refreshToken = "old-refresh-token"
        val engine = MockEngine { request ->
            requests += request
            when (request.url.encodedPath) {
                "/api/v1/me/gears/categories" -> {
                    if (request.headers[HttpHeaders.Authorization] == "Bearer fresh-access-token") {
                        respond(
                            content = """{"status":"ok"}""",
                            headers = headersOf(HttpHeaders.ContentType, "application/json"),
                        )
                    } else {
                        respond(
                            content = """{"code":"unauthorized","message":"Unauthorized"}""",
                            status = HttpStatusCode.Unauthorized,
                            headers = headersOf(HttpHeaders.ContentType, "application/json"),
                        )
                    }
                }
                "/api/v1/auth/refresh" -> respond(
                    content = """{
                        "access_token":"fresh-access-token",
                        "expires_at":"2026-05-17T12:00:00Z",
                        "refresh_token":"new-refresh-token",
                        "refresh_expires_at":"2026-06-17T12:00:00Z",
                        "user":{"id":"u1","username":"trail_alice"}
                    }""".trimIndent(),
                    headers = headersOf(HttpHeaders.ContentType, "application/json"),
                )
                else -> error("unexpected path ${request.url.encodedPath}")
            }
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            tokenProvider = { accessToken },
            refreshTokenProvider = { refreshToken },
            sessionRefreshHandler = { response ->
                accessToken = response.accessToken
                refreshToken = response.refreshToken
            },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val response = client.get<HealthResponse>("/api/v1/me/gears/categories")

        assertEquals("ok", response.status)
        assertEquals(
            listOf("/api/v1/me/gears/categories", "/api/v1/auth/refresh", "/api/v1/me/gears/categories"),
            requests.map { it.url.encodedPath },
        )
        assertEquals("Bearer expired-access-token", requests[0].headers[HttpHeaders.Authorization])
        assertEquals("Bearer fresh-access-token", requests[2].headers[HttpHeaders.Authorization])
        assertEquals("new-refresh-token", refreshToken)
    }

    @Test
    fun refreshSkipsNetworkCallWhenTokenWasAlreadyUpdated() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        var accessToken = "expired-access-token"
        var refreshToken = "old-refresh-token"
        val engine = MockEngine { request ->
            requests += request
            when (request.url.encodedPath) {
                "/api/v1/auth/refresh" -> respond(
                    content = """{
                        "access_token":"fresh-access-token",
                        "expires_at":"2026-05-17T12:00:00Z",
                        "refresh_token":"new-refresh-token",
                        "refresh_expires_at":"2026-06-17T12:00:00Z",
                        "user":{"id":"u1","username":"trail_alice"}
                    }""".trimIndent(),
                    headers = headersOf(HttpHeaders.ContentType, "application/json"),
                )
                else -> error("unexpected path ${request.url.encodedPath}")
            }
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            tokenProvider = { accessToken },
            refreshTokenProvider = { refreshToken },
            sessionRefreshHandler = { response ->
                accessToken = response.accessToken
                refreshToken = response.refreshToken
            },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        assertTrue(client.refreshWithStoredToken(accessTokenUsed = "expired-access-token"))
        assertTrue(client.refreshWithStoredToken(accessTokenUsed = "expired-access-token"))

        assertEquals(listOf("/api/v1/auth/refresh"), requests.map { it.url.encodedPath })
        assertEquals("fresh-access-token", accessToken)
        assertEquals("new-refresh-token", refreshToken)
    }

    @Test
    fun refreshNetworkFailureDoesNotExpireSession() = runTest {
        var expired = false
        val engine = MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/v1/me/gears/categories" -> respondJson(
                    """{"code":"unauthorized","message":"Unauthorized"}""",
                    HttpStatusCode.Unauthorized,
                )
                "/api/v1/auth/refresh" -> throw IOException("offline")
                else -> error("unexpected path ${request.url.encodedPath}")
            }
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            tokenProvider = { "expired-access-token" },
            refreshTokenProvider = { "refresh-token" },
            sessionExpiredHandler = { expired = true },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val exception = runCatching { client.get<HealthResponse>("/me/gears/categories") }.exceptionOrNull()

        assertTrue(exception is ApiException)
        assertFalse(expired)
    }

    @Test
    fun refreshUnauthorizedExpiresSession() = runTest {
        var expired = false
        val engine = MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/v1/me/gears/categories" -> respondJson(
                    """{"code":"unauthorized","message":"Unauthorized"}""",
                    HttpStatusCode.Unauthorized,
                )
                "/api/v1/auth/refresh" -> respondJson(
                    """{"code":"unauthorized","message":"Unauthorized"}""",
                    HttpStatusCode.Unauthorized,
                )
                else -> error("unexpected path ${request.url.encodedPath}")
            }
        }
        val client = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            tokenProvider = { "expired-access-token" },
            refreshTokenProvider = { "refresh-token" },
            sessionExpiredHandler = { expired = true },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )

        val exception = runCatching { client.get<HealthResponse>("/me/gears/categories") }.exceptionOrNull()

        assertTrue(exception is ApiException)
        assertTrue(expired)
    }

    @Test
    fun getStoresAndReplaysCachedResponseOnNetworkFailure() = runTest {
        val cacheStore = InMemoryOfflineHttpCacheStore()
        val onlineClient = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            cacheScopeProvider = { "user-a" },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { respondJson("""{"status":"fresh"}""") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        )

        val fresh = onlineClient.get<HealthResponse>("/me/gears/categories", query = mapOf("tab" to "available"))

        assertEquals("fresh", fresh.status)
        assertEquals(1, cacheStore.status.value.cachedResponseCount)

        val offlineClient = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            cacheScopeProvider = { "user-a" },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { throw IOException("offline") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        )

        val cached = offlineClient.get<HealthResponse>("/me/gears/categories", query = mapOf("tab" to "available"))

        assertEquals("fresh", cached.status)
    }

    @Test
    fun cachedResponsesAreSeparatedByUserScope() = runTest {
        val cacheStore = InMemoryOfflineHttpCacheStore()
        ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            cacheScopeProvider = { "user-a" },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { respondJson("""{"status":"user-a"}""") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        ).get<HealthResponse>("/me/gears/categories")

        val userBClient = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            cacheScopeProvider = { "user-b" },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { throw IOException("offline") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        )

        val userBException = runCatching { userBClient.get<HealthResponse>("/me/gears/categories") }.exceptionOrNull()
        assertTrue(userBException is IOException)

        val userAClient = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            cacheScopeProvider = { "user-a" },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { throw IOException("offline") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        )
        assertEquals("user-a", userAClient.get<HealthResponse>("/me/gears/categories").status)
    }

    @Test
    fun mutationFailuresDoNotReplayCachedGetResponses() = runTest {
        val cacheStore = InMemoryOfflineHttpCacheStore()
        ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { respondJson("""{"status":"cached"}""") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        ).get<HealthResponse>("/me/gears/categories")

        val offlineClient = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            offlineCacheStore = cacheStore,
            httpClient = HttpClient(MockEngine { throw IOException("offline") }) {
                install(ContentNegotiation) { json(ApiClient.defaultJson) }
            },
        )

        val exception = runCatching {
            offlineClient.post<kotlinx.serialization.json.JsonObject, HealthResponse>(
                "/me/gears/categories",
                kotlinx.serialization.json.buildJsonObject { },
            )
        }.exceptionOrNull()

        assertTrue(exception is IOException)
    }

    private fun MockRequestHandleScope.respondJson(
        content: String,
        status: HttpStatusCode = HttpStatusCode.OK,
    ) = respond(
        content = content,
        status = status,
        headers = headersOf(HttpHeaders.ContentType, "application/json"),
    )

    private fun HttpRequestData.bodyText(): String =
        (body as TextContent).text

    private fun testHmacSha256Hex(secret: String, message: String): String {
        val mac = Mac.getInstance("HmacSHA256")
        mac.init(SecretKeySpec(secret.encodeToByteArray(), "HmacSHA256"))
        return mac.doFinal(message.encodeToByteArray()).joinToString(separator = "") { "%02x".format(it.toInt() and 0xff) }
    }

    private val signedConfig = AppConfig(
        baseUrl = "https://api.example.test",
        requestSignature = RequestSignatureCredentials(
            appId = "android-test-client",
            appSecret = "android-test-secret",
        ),
    )

    private val domainCandidates = listOf(
        AppDomainCandidate(
            id = "primary",
            apiBaseUrl = "https://api.example.invalid",
            assetsBaseUrl = "https://assets.example.invalid",
        ),
        AppDomainCandidate(
            id = "backup",
            apiBaseUrl = "https://api-alt1.example.invalid",
            assetsBaseUrl = "https://assets-alt1.example.invalid",
        ),
        AppDomainCandidate(
            id = "backup-2",
            apiBaseUrl = "https://api-alt2.example.invalid",
            assetsBaseUrl = "https://assets-alt2.example.invalid",
        ),
    )
}
