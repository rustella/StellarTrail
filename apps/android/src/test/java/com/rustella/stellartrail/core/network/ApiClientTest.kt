package com.rustella.stellartrail.core.network

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.domain.common.HealthResponse
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

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
    fun authenticatedRequestRefreshesOnceOnUnauthorizedAndRetries() = runTest {
        val requests = mutableListOf<io.ktor.client.request.HttpRequestData>()
        var accessToken = "expired-access-token"
        var refreshToken = "old-refresh-token"
        val engine = MockEngine { request ->
            requests += request
            when (request.url.encodedPath) {
                "/api/me/gears/categories" -> {
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
                "/api/auth/refresh" -> respond(
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

        val response = client.get<HealthResponse>("/api/me/gears/categories")

        assertEquals("ok", response.status)
        assertEquals(
            listOf("/api/me/gears/categories", "/api/auth/refresh", "/api/me/gears/categories"),
            requests.map { it.url.encodedPath },
        )
        assertEquals("Bearer expired-access-token", requests[0].headers[HttpHeaders.Authorization])
        assertEquals("Bearer fresh-access-token", requests[2].headers[HttpHeaders.Authorization])
        assertEquals("new-refresh-token", refreshToken)
    }

}
