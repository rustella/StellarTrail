package com.rustella.stellartrail.data.auth

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.auth.CaptchaChallengeRequest
import com.rustella.stellartrail.domain.auth.CaptchaChallengeResponse
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeRequest
import com.rustella.stellartrail.domain.auth.EmailLoginCodeRequest
import com.rustella.stellartrail.domain.auth.EmailLoginRequest
import com.rustella.stellartrail.domain.auth.PasswordResetCodeRequest
import com.rustella.stellartrail.domain.auth.PasswordResetRequest
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeResponse
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.PasswordLoginRequest
import com.rustella.stellartrail.domain.auth.RefreshTokenRequest
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.WechatLoginRequest

class AuthApi(private val apiClient: ApiClient) {
    suspend fun sendEmailVerificationCode(request: EmailVerificationCodeRequest): EmailVerificationCodeResponse =
        apiClient.post("/api/auth/email-verification-code", request)

    suspend fun sendEmailLoginCode(request: EmailLoginCodeRequest): EmailVerificationCodeResponse =
        apiClient.post("/api/auth/email-login-code", request)

    suspend fun loginWithEmailCode(request: EmailLoginRequest): LoginResponse =
        apiClient.post("/api/auth/email-login", request)

    suspend fun sendPasswordResetCode(request: PasswordResetCodeRequest): EmailVerificationCodeResponse =
        apiClient.post("/api/auth/password-reset-code", request)

    suspend fun resetPassword(request: PasswordResetRequest): LoginResponse =
        apiClient.post("/api/auth/password-reset", request)

    suspend fun createCaptcha(request: CaptchaChallengeRequest): CaptchaChallengeResponse =
        apiClient.post("/api/auth/captcha", request)

    suspend fun register(request: RegisterRequest): LoginResponse =
        apiClient.post("/api/auth/register", request)

    suspend fun loginWithPassword(request: PasswordLoginRequest): LoginResponse =
        apiClient.post("/api/auth/login", request)

    suspend fun refresh(request: RefreshTokenRequest): LoginResponse =
        apiClient.post("/api/auth/refresh", request)

    suspend fun loginWithWechatCode(request: WechatLoginRequest): LoginResponse =
        apiClient.post("/api/auth/wechat-login", request)
}
