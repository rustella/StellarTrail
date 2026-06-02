package com.rustella.stellartrail.data.auth

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.auth.BindPhoneRequest
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.SmsCodeRequest
import com.rustella.stellartrail.domain.auth.SmsLoginRequest
import com.rustella.stellartrail.domain.auth.SmsPasswordResetRequest
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
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Test

class AuthApiTest {
    @Test
    fun smsAuthPathsMatchBackendRoutes() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = AuthApi(testClient { request ->
            requests += request
            when (request.url.encodedPath) {
                "/api/v1/auth/sms-registration-code",
                "/api/v1/auth/sms-login-code",
                "/api/v1/auth/sms-password-reset-code",
                "/api/v1/me/phone-binding-code",
                "/api/v1/me/phone-rebinding-current-code" -> respondJson(smsCodeJson)
                "/api/v1/auth/sms-register",
                "/api/v1/auth/sms-login",
                "/api/v1/auth/sms-password-reset" -> respondJson(loginJson)
                "/api/v1/me/phone-binding" -> respondJson(bindPhoneJson)
                else -> error("unexpected request ${request.method.value} ${request.url}")
            }
        })

        api.sendSmsRegistrationCode(SmsCodeRequest("13800000000"))
        api.smsRegister(smsRegisterRequest)
        api.sendSmsLoginCode(SmsCodeRequest("13800000000"))
        api.smsLogin(SmsLoginRequest("13800000000", "ticket", "123456"))
        api.sendSmsPasswordResetCode(SmsCodeRequest("13800000000"))
        api.smsPasswordReset(SmsPasswordResetRequest("13800000000", "ticket", "123456", "Password1", "Password1"))
        api.sendBindPhoneCode(SmsCodeRequest("13800000000"))
        api.sendCurrentPhoneRebindingCode()
        api.bindPhone(bindPhoneRequest)

        assertEquals(List(9) { HttpMethod.Post }, requests.map { it.method })
        assertEquals(
            listOf(
                "/api/v1/auth/sms-registration-code",
                "/api/v1/auth/sms-register",
                "/api/v1/auth/sms-login-code",
                "/api/v1/auth/sms-login",
                "/api/v1/auth/sms-password-reset-code",
                "/api/v1/auth/sms-password-reset",
                "/api/v1/me/phone-binding-code",
                "/api/v1/me/phone-rebinding-current-code",
                "/api/v1/me/phone-binding",
            ),
            requests.map { it.url.encodedPath },
        )
    }

    @Test
    fun smsDtosUseBackendWireFieldNamesAndDecodePhoneUser() {
        val registerJson = ApiClient.defaultJson.encodeToString(smsRegisterRequest)
        val bindJson = ApiClient.defaultJson.encodeToString(bindPhoneRequest)
        val decodedRegister = ApiClient.defaultJson.decodeFromString<JsonObject>(registerJson)
        val decodedBind = ApiClient.defaultJson.decodeFromString<JsonObject>(bindJson)
        val login = ApiClient.defaultJson.decodeFromString<LoginResponse>(loginJson)

        assertEquals("ticket", decodedRegister.getValue("sms_ticket").jsonPrimitive.content)
        assertEquals("123456", decodedRegister.getValue("sms_verification_code").jsonPrimitive.content)
        assertEquals("Password1", decodedRegister.getValue("confirm_password").jsonPrimitive.content)
        assertEquals("current-ticket", decodedBind.getValue("current_sms_ticket").jsonPrimitive.content)
        assertEquals("654321", decodedBind.getValue("current_sms_verification_code").jsonPrimitive.content)
        assertEquals("13800000000", login.user.phone)
    }

    private fun testClient(handler: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData): ApiClient {
        val engine = MockEngine { request -> handler(request) }
        return ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )
    }

    private fun MockRequestHandleScope.respondJson(content: String) = respond(
        content = content,
        headers = headersOf(HttpHeaders.ContentType, "application/json"),
    )
}

private val smsRegisterRequest = SmsRegisterRequest(
    username = "trail_user",
    nickname = "星野徒步者",
    phone = "13800000000",
    password = "Password1",
    confirmPassword = "Password1",
    smsTicket = "ticket",
    smsVerificationCode = "123456",
)

private val bindPhoneRequest = BindPhoneRequest(
    phone = "13900000000",
    smsTicket = "new-ticket",
    smsVerificationCode = "123456",
    currentSmsTicket = "current-ticket",
    currentSmsVerificationCode = "654321",
)

private val smsCodeJson = """
{
  "phone": "13800000000",
  "sms_ticket": "ticket",
  "expires_at": "2099-01-01T00:00:00Z",
  "debug_code": "123456"
}
""".trimIndent()

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
