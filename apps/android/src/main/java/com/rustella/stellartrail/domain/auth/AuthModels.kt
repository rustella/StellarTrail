package com.rustella.stellartrail.domain.auth

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class EmailVerificationCodeRequest(
    val email: String,
)

@Serializable
data class EmailVerificationCodeResponse(
    val email: String,
    @SerialName("expires_at") val expiresAt: String,
    @SerialName("debug_code") val debugCode: String? = null,
)

@Serializable
data class RegisterRequest(
    val username: String,
    val email: String,
    val password: String,
    @SerialName("confirm_password") val confirmPassword: String,
    @SerialName("email_verification_code") val emailVerificationCode: String,
)

@Serializable
data class CaptchaChallengeRequest(
    val account: String,
)

@Serializable
data class CaptchaChallengeResponse(
    @SerialName("captcha_ticket") val captchaTicket: String,
    @SerialName("captcha_type") val captchaType: String,
    @SerialName("image_svg") val imageSvg: String,
    @SerialName("expires_at") val expiresAt: String,
    @SerialName("debug_answer") val debugAnswer: String? = null,
)

@Serializable
data class PasswordLoginRequest(
    val account: String,
    val password: String,
    @SerialName("captcha_ticket") val captchaTicket: String? = null,
    @SerialName("captcha_answer") val captchaAnswer: String? = null,
)

@Serializable
data class WechatLoginRequest(
    val code: String,
    val profile: WechatLoginProfile? = null,
)

@Serializable
data class WechatLoginProfile(
    val nickname: String? = null,
    @SerialName("avatar_url") val avatarUrl: String? = null,
)

@Serializable
data class LoginUser(
    val id: String,
    val username: String? = null,
    val email: String? = null,
    val nickname: String? = null,
    @SerialName("avatar_url") val avatarUrl: String? = null,
)

@Serializable
data class LoginResponse(
    @SerialName("access_token") val accessToken: String,
    @SerialName("expires_at") val expiresAt: String,
    @SerialName("refresh_token") val refreshToken: String,
    @SerialName("refresh_expires_at") val refreshExpiresAt: String,
    val user: LoginUser,
)

@Serializable
data class RefreshTokenRequest(
    @SerialName("refresh_token") val refreshToken: String,
)

@Serializable
data class UserSession(
    val accessToken: String,
    val expiresAt: String,
    val refreshToken: String,
    val refreshExpiresAt: String,
    val user: LoginUser,
)

fun LoginResponse.toSession(): UserSession = UserSession(
    accessToken = accessToken,
    expiresAt = expiresAt,
    refreshToken = refreshToken,
    refreshExpiresAt = refreshExpiresAt,
    user = user,
)
