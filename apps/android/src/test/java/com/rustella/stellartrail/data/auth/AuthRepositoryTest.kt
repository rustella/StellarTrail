package com.rustella.stellartrail.data.auth

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.core.session.InMemorySessionStore
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.SmsRegisterRequest
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.content.OutgoingContent
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import io.ktor.utils.io.core.readText
import io.ktor.utils.io.readRemaining
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Test

class AuthRepositoryTest {
    @Test
    fun smsLoginTrimsRequestFieldsAndSavesSession() = runTest {
        val bodies = mutableListOf<String>()
        val repository = repository { request ->
            bodies += request.bodyText()
            assertEquals(HttpMethod.Post, request.method)
            assertEquals("/api/v1/auth/sms-login", request.url.encodedPath)
            respondJson(loginJson)
        }

        repository.smsLogin(" 13800000000 ", " ticket ", " 123456 ")

        val body = ApiClient.defaultJson.decodeFromString<JsonObject>(bodies.single())
        assertEquals("13800000000", body.getValue("phone").jsonPrimitive.content)
        assertEquals("ticket", body.getValue("sms_ticket").jsonPrimitive.content)
        assertEquals("123456", body.getValue("sms_verification_code").jsonPrimitive.content)
        assertEquals("13800000000", repository.session.value?.user?.phone)
    }

    @Test
    fun smsRegisterTrimsPayloadAndSavesSession() = runTest {
        val bodies = mutableListOf<String>()
        val repository = repository { request ->
            bodies += request.bodyText()
            assertEquals("/api/v1/auth/sms-register", request.url.encodedPath)
            respondJson(loginJson)
        }

        repository.smsRegister(
            SmsRegisterRequest(
                username = " trail_user ",
                nickname = " 星野徒步者 ",
                phone = " 13800000000 ",
                password = "Password1",
                confirmPassword = "Password1",
                smsTicket = " ticket ",
                smsVerificationCode = " 123456 ",
            ),
        )

        val body = ApiClient.defaultJson.decodeFromString<JsonObject>(bodies.single())
        assertEquals("trail_user", body.getValue("username").jsonPrimitive.content)
        assertEquals("星野徒步者", body.getValue("nickname").jsonPrimitive.content)
        assertEquals("13800000000", body.getValue("phone").jsonPrimitive.content)
        assertEquals("ticket", body.getValue("sms_ticket").jsonPrimitive.content)
        assertEquals("123456", body.getValue("sms_verification_code").jsonPrimitive.content)
        assertEquals("13800000000", repository.session.value?.user?.phone)
    }

    @Test
    fun bindPhoneUpdatesCurrentSessionUserPhone() = runTest {
        val sessionStore = InMemorySessionStore()
        sessionStore.save(ApiClient.defaultJson.decodeFromString<LoginResponse>(loginJson))
        val repository = repository(sessionStore) { request ->
            assertEquals("/api/v1/me/phone-binding", request.url.encodedPath)
            respondJson(bindPhoneJson)
        }

        val user = repository.bindPhone(
            phone = " 13900000000 ",
            smsTicket = " new-ticket ",
            smsCode = " 123456 ",
            currentSmsTicket = " current-ticket ",
            currentSmsCode = " 654321 ",
        )

        assertEquals("13900000000", user.phone)
        assertEquals("13900000000", repository.session.value?.user?.phone)
    }

    @Test
    fun updateSessionUserStoresLatestAvatarUrl() {
        val sessionStore = InMemorySessionStore()
        sessionStore.save(ApiClient.defaultJson.decodeFromString<LoginResponse>(loginJson))
        val repository = repository(sessionStore) { respondJson(loginJson) }

        repository.updateSessionUser(
            LoginUser(
                id = "user-1",
                username = "trail_user",
                email = "trail@example.test",
                phone = "13800000000",
                nickname = "星野徒步者",
                avatarUrl = "https://assets.example.test/users/user-1/avatar.png",
            ),
        )

        assertEquals("https://assets.example.test/users/user-1/avatar.png", repository.session.value?.user?.avatarUrl)
    }

    private fun repository(
        sessionStore: InMemorySessionStore = InMemorySessionStore(),
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): AuthRepository {
        val engine = MockEngine { request -> handler(request) }
        val apiClient = ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )
        return AuthRepository(AuthApi(apiClient), sessionStore)
    }

    private fun MockRequestHandleScope.respondJson(content: String) = respond(
        content = content,
        headers = headersOf(HttpHeaders.ContentType, "application/json"),
    )

    private suspend fun HttpRequestData.bodyText(): String = when (val content = body) {
        is OutgoingContent.ByteArrayContent -> content.bytes().decodeToString()
        is OutgoingContent.ReadChannelContent -> content.readFrom().readRemaining().readText()
        else -> content.toString()
    }
}

private val loginJson = """
{
  "access_token": "fixture-access-token",
  "expires_at": "2099-01-01T00:00:00Z",
  "refresh_token": "fixture-refresh-token",
  "refresh_expires_at": "2099-01-02T00:00:00Z",
  "user": {
    "id": "user-1",
    "username": "trail_user",
    "email": "trail@example.test",
    "phone": "13800000000",
    "nickname": "星野徒步者",
    "avatar_url": null
  }
}
""".trimIndent()

private val bindPhoneJson = """
{
  "user": {
    "id": "user-1",
    "username": "trail_user",
    "email": "trail@example.test",
    "phone": "13900000000",
    "nickname": "星野徒步者",
    "avatar_url": null
  }
}
""".trimIndent()
