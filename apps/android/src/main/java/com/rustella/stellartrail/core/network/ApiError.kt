package com.rustella.stellartrail.core.network

import io.ktor.http.HttpStatusCode
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

@Serializable
data class ApiErrorBody(
    val code: String? = null,
    val message: String? = null,
)

class ApiException(
    val statusCode: Int,
    val errorCode: String?,
    override val message: String,
) : RuntimeException(message) {
    val isUnauthorized: Boolean get() = statusCode == HttpStatusCode.Unauthorized.value
    val isCaptchaRequired: Boolean get() = statusCode == 428 || errorCode == "captcha_required"

    companion object {
        fun from(status: HttpStatusCode, body: String, json: Json): ApiException {
            val parsed = runCatching { json.decodeFromString<ApiErrorBody>(body) }.getOrNull()
            val fallback = body.takeIf { it.isNotBlank() } ?: status.description
            return ApiException(status.value, parsed?.code, parsed?.message ?: fallback)
        }
    }
}

fun Throwable.userMessage(): String = when (this) {
    is ApiException -> message
    else -> message ?: "网络请求失败，请稍后重试"
}
