package com.rustella.stellartrail.data.auth

import com.rustella.stellartrail.core.session.SessionStore
import com.rustella.stellartrail.domain.auth.BindEmailCodeRequest
import com.rustella.stellartrail.domain.auth.BindEmailRequest
import com.rustella.stellartrail.domain.auth.CaptchaChallengeResponse
import com.rustella.stellartrail.domain.auth.EmailLoginCodeRequest
import com.rustella.stellartrail.domain.auth.EmailLoginRequest
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeRequest
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeResponse
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.PasswordLoginRequest
import com.rustella.stellartrail.domain.auth.PasswordResetCodeRequest
import com.rustella.stellartrail.domain.auth.PasswordResetRequest
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.UserSession
import kotlinx.coroutines.flow.StateFlow

interface AuthRepositoryContract {
    val session: StateFlow<UserSession?>
    suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse
    suspend fun sendEmailLoginCode(email: String): EmailVerificationCodeResponse
    suspend fun loginWithEmailCode(email: String, emailCode: String): LoginResponse
    suspend fun sendPasswordResetCode(email: String): EmailVerificationCodeResponse
    suspend fun resetPassword(email: String, emailCode: String, password: String, confirmPassword: String): LoginResponse
    suspend fun sendBindEmailCode(email: String): EmailVerificationCodeResponse
    suspend fun bindEmail(email: String, emailCode: String): LoginUser
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
        api.sendEmailVerificationCode(EmailVerificationCodeRequest(email.trim()))

    override suspend fun sendEmailLoginCode(email: String): EmailVerificationCodeResponse =
        api.sendEmailLoginCode(EmailLoginCodeRequest(email.trim()))

    override suspend fun loginWithEmailCode(email: String, emailCode: String): LoginResponse {
        val response = api.loginWithEmailCode(EmailLoginRequest(email.trim(), emailCode.trim()))
        sessionStore.save(response)
        return response
    }

    override suspend fun sendPasswordResetCode(email: String): EmailVerificationCodeResponse =
        api.sendPasswordResetCode(PasswordResetCodeRequest(email.trim()))

    override suspend fun resetPassword(
        email: String,
        emailCode: String,
        password: String,
        confirmPassword: String,
    ): LoginResponse {
        val response = api.resetPassword(
            PasswordResetRequest(
                email = email.trim(),
                emailVerificationCode = emailCode.trim(),
                password = password,
                confirmPassword = confirmPassword,
            ),
        )
        sessionStore.save(response)
        return response
    }

    override suspend fun sendBindEmailCode(email: String): EmailVerificationCodeResponse =
        api.sendBindEmailCode(BindEmailCodeRequest(email.trim()))

    override suspend fun bindEmail(email: String, emailCode: String): LoginUser {
        val response = api.bindEmail(BindEmailRequest(email.trim(), emailCode.trim()))
        sessionStore.session.value?.copy(user = response.user)?.let(sessionStore::save)
        return response.user
    }

    override suspend fun createCaptcha(account: String): CaptchaChallengeResponse =
        api.createCaptcha(com.rustella.stellartrail.domain.auth.CaptchaChallengeRequest(account.trim()))

    override suspend fun login(
        account: String,
        password: String,
        captchaTicket: String?,
        captchaAnswer: String?,
    ): LoginResponse {
        val response = api.loginWithPassword(
            PasswordLoginRequest(
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
