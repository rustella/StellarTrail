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
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.PasswordLoginRequest
import com.rustella.stellartrail.domain.auth.PasswordResetCodeRequest
import com.rustella.stellartrail.domain.auth.PasswordResetRequest
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.SmsCodeRequest
import com.rustella.stellartrail.domain.auth.SmsCodeResponse
import com.rustella.stellartrail.domain.auth.SmsLoginRequest
import com.rustella.stellartrail.domain.auth.SmsPasswordResetRequest
import com.rustella.stellartrail.domain.auth.SmsRegisterRequest
import com.rustella.stellartrail.domain.auth.UserSession
import com.rustella.stellartrail.domain.auth.BindPhoneRequest
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
    suspend fun sendSmsRegistrationCode(phone: String): SmsCodeResponse
    suspend fun smsRegister(request: SmsRegisterRequest): LoginResponse
    suspend fun sendSmsLoginCode(phone: String): SmsCodeResponse
    suspend fun smsLogin(phone: String, smsTicket: String, smsCode: String): LoginResponse
    suspend fun sendSmsPasswordResetCode(phone: String): SmsCodeResponse
    suspend fun smsResetPassword(
        phone: String,
        smsTicket: String,
        smsCode: String,
        password: String,
        confirmPassword: String,
    ): LoginResponse
    suspend fun sendBindPhoneCode(phone: String): SmsCodeResponse
    suspend fun sendRebindCurrentPhoneCode(): SmsCodeResponse
    suspend fun bindPhone(
        phone: String,
        smsTicket: String,
        smsCode: String,
        currentSmsTicket: String? = null,
        currentSmsCode: String? = null,
    ): LoginUser
    suspend fun createCaptcha(account: String): CaptchaChallengeResponse
    suspend fun login(account: String, password: String, captchaTicket: String? = null, captchaAnswer: String? = null): LoginResponse
    suspend fun register(request: RegisterRequest): LoginResponse
    fun updateSessionUser(user: LoginUser) {}
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

    override suspend fun sendSmsRegistrationCode(phone: String): SmsCodeResponse =
        api.sendSmsRegistrationCode(SmsCodeRequest(phone.trim()))

    override suspend fun smsRegister(request: SmsRegisterRequest): LoginResponse {
        val response = api.smsRegister(
            request.copy(
                username = request.username.trim(),
                nickname = request.nickname.trim(),
                phone = request.phone.trim(),
                smsTicket = request.smsTicket.trim(),
                smsVerificationCode = request.smsVerificationCode.trim(),
            ),
        )
        sessionStore.save(response)
        return response
    }

    override suspend fun sendSmsLoginCode(phone: String): SmsCodeResponse =
        api.sendSmsLoginCode(SmsCodeRequest(phone.trim()))

    override suspend fun smsLogin(phone: String, smsTicket: String, smsCode: String): LoginResponse {
        val response = api.smsLogin(
            SmsLoginRequest(
                phone = phone.trim(),
                smsTicket = smsTicket.trim(),
                smsVerificationCode = smsCode.trim(),
            ),
        )
        sessionStore.save(response)
        return response
    }

    override suspend fun sendSmsPasswordResetCode(phone: String): SmsCodeResponse =
        api.sendSmsPasswordResetCode(SmsCodeRequest(phone.trim()))

    override suspend fun smsResetPassword(
        phone: String,
        smsTicket: String,
        smsCode: String,
        password: String,
        confirmPassword: String,
    ): LoginResponse {
        val response = api.smsPasswordReset(
            SmsPasswordResetRequest(
                phone = phone.trim(),
                smsTicket = smsTicket.trim(),
                smsVerificationCode = smsCode.trim(),
                password = password,
                confirmPassword = confirmPassword,
            ),
        )
        sessionStore.save(response)
        return response
    }

    override suspend fun sendBindPhoneCode(phone: String): SmsCodeResponse =
        api.sendBindPhoneCode(SmsCodeRequest(phone.trim()))

    override suspend fun sendRebindCurrentPhoneCode(): SmsCodeResponse =
        api.sendCurrentPhoneRebindingCode()

    override suspend fun bindPhone(
        phone: String,
        smsTicket: String,
        smsCode: String,
        currentSmsTicket: String?,
        currentSmsCode: String?,
    ): LoginUser {
        val response = api.bindPhone(
            BindPhoneRequest(
                phone = phone.trim(),
                smsTicket = smsTicket.trim(),
                smsVerificationCode = smsCode.trim(),
                currentSmsTicket = currentSmsTicket?.trim()?.takeIf { it.isNotEmpty() },
                currentSmsVerificationCode = currentSmsCode?.trim()?.takeIf { it.isNotEmpty() },
            ),
        )
        sessionStore.session.value?.copy(user = response.user)?.let(sessionStore::save)
        return response.user
    }

    override fun updateSessionUser(user: LoginUser) {
        sessionStore.session.value?.copy(user = user)?.let(sessionStore::save)
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
