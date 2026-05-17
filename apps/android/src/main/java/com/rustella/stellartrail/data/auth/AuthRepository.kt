package com.rustella.stellartrail.data.auth

import com.rustella.stellartrail.core.session.SessionStore
import com.rustella.stellartrail.domain.auth.CaptchaChallengeResponse
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeResponse
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.UserSession
import kotlinx.coroutines.flow.StateFlow

interface AuthRepositoryContract {
    val session: StateFlow<UserSession?>
    suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse
    suspend fun createCaptcha(account: String): CaptchaChallengeResponse
    suspend fun login(account: String, password: String, captchaTicket: String? = null, captchaAnswer: String? = null): LoginResponse
    suspend fun register(request: RegisterRequest): LoginResponse
    fun logout()
}

class AuthRepository(
    private val api: AuthApi,
    private val sessionStore: SessionStore,
) : AuthRepositoryContract {
    override val session: StateFlow<UserSession?> = sessionStore.session

    override suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse =
        api.sendEmailVerificationCode(com.rustella.stellartrail.domain.auth.EmailVerificationCodeRequest(email.trim()))

    override suspend fun createCaptcha(account: String): CaptchaChallengeResponse =
        api.createCaptcha(com.rustella.stellartrail.domain.auth.CaptchaChallengeRequest(account.trim()))

    override suspend fun login(
        account: String,
        password: String,
        captchaTicket: String?,
        captchaAnswer: String?,
    ): LoginResponse {
        val response = api.loginWithPassword(
            com.rustella.stellartrail.domain.auth.PasswordLoginRequest(
                account = account.trim(),
                password = password,
                captchaTicket = captchaTicket?.trim()?.takeIf { it.isNotEmpty() },
                captchaAnswer = captchaAnswer?.trim()?.takeIf { it.isNotEmpty() },
            ),
        )
        sessionStore.save(response)
        return response
    }

    override suspend fun register(request: RegisterRequest): LoginResponse {
        val response = api.register(request)
        sessionStore.save(response)
        return response
    }

    override fun logout() {
        sessionStore.clear()
    }
}
