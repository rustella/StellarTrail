package com.rustella.stellartrail.core.network

import io.ktor.http.HttpStatusCode
import java.io.IOException
import java.net.UnknownHostException
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

@Serializable
data class ApiErrorBody(
    val code: String? = null,
    val message: String? = null,
    @SerialName("retry_after_seconds") val retryAfterSeconds: Long? = null,
)

class ApiException(
    val statusCode: Int,
    val errorCode: String?,
    val rawBody: String,
    val retryAfterSeconds: Long?,
    override val message: String,
) : RuntimeException(message) {
    val isUnauthorized: Boolean get() = statusCode == HttpStatusCode.Unauthorized.value
    val isCaptchaRequired: Boolean get() = statusCode == 428 || errorCode == "captcha_required"
    val isRateLimited: Boolean get() = statusCode == HttpStatusCode.TooManyRequests.value || errorCode == "rate_limited"

    companion object {
        fun from(status: HttpStatusCode, body: String, json: Json, retryAfterHeader: String? = null): ApiException {
            val parsed = runCatching { json.decodeFromString<ApiErrorBody>(body) }.getOrNull()
            val fallback = body.takeIf { it.isNotBlank() } ?: status.description
            val retryAfterSeconds = retryAfterHeader?.toLongOrNull()?.takeIf { it > 0 }
                ?: parsed?.retryAfterSeconds?.takeIf { it > 0 }
            return ApiException(status.value, parsed?.code, body, retryAfterSeconds, parsed?.message ?: fallback)
        }
    }
}

fun Throwable.userMessage(): String = when (this) {
    is ApiException -> message
    is UnknownHostException -> "无法连接到 API，请检查网络或 API Base URL。"
    is IOException -> "网络请求失败，请检查网络后重试。"
    else -> message ?: "网络请求失败，请稍后重试"
}
