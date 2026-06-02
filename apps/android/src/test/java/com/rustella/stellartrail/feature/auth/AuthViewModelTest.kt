package com.rustella.stellartrail.feature.auth

import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.domain.auth.CaptchaChallengeResponse
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeResponse
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.SmsCodeResponse
import com.rustella.stellartrail.domain.auth.SmsRegisterRequest
import com.rustella.stellartrail.domain.auth.UserSession
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AuthViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun switchingModesClearsTransientErrorsAndKeepsSecondaryFormsReachable() {
        val viewModel = AuthViewModel(FakeAuthRepository())

        viewModel.login()
        assertEquals("请填写用户名、邮箱或手机号和密码", viewModel.state.value.error)

        viewModel.switchMode(AuthMode.RESET_PASSWORD)

        assertEquals(AuthMode.RESET_PASSWORD, viewModel.state.value.mode)
        assertNull(viewModel.state.value.error)
        assertNull(viewModel.state.value.notice)
        assertEquals("", viewModel.state.value.captchaTicket)
    }

    @Test
    fun emailCodeLoginUsesExistingRepositoryFlow() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = AuthViewModel(repository)

        viewModel.switchMode(AuthMode.EMAIL_CODE)
        viewModel.updateEmail("trail@example.test")
        viewModel.updateEmailCode("123456")
        viewModel.loginWithEmailCode()
        advanceUntilIdle()

        assertEquals(1, repository.emailLoginCalls)
        assertEquals("trail@example.test", repository.lastEmailLoginEmail)
        assertFalse(viewModel.state.value.loading)
        assertNull(viewModel.state.value.error)
    }

    @Test
    fun phoneCodeLoginStoresTicketForLoginOnly() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = AuthViewModel(repository)

        viewModel.switchMode(AuthMode.PHONE_CODE)
        viewModel.updatePhone("13800000000")
        viewModel.sendSmsLoginCode()
        advanceUntilIdle()
        viewModel.updateSmsCode("123456")
        viewModel.loginWithSmsCode()
        advanceUntilIdle()

        assertEquals(1, repository.smsLoginCodeCalls)
        assertEquals(1, repository.smsLoginCalls)
        assertEquals("13800000000", repository.lastSmsLoginPhone)
        assertEquals("sms-login-ticket", repository.lastSmsLoginTicket)
        assertEquals("123456", repository.lastSmsLoginCode)
        assertNull(viewModel.state.value.error)
    }

    @Test
    fun phoneRegisterRequiresItsOwnTicketAndUsesSmsRegisterPayload() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = AuthViewModel(repository, AuthMode.REGISTER)

        viewModel.updatePhone("13800000000")
        viewModel.sendSmsLoginCode()
        advanceUntilIdle()
        viewModel.updateUsername("trail_user")
        viewModel.updateNickname("星野徒步者")
        viewModel.updateSmsCode("123456")
        viewModel.updatePassword("Password1")
        viewModel.updateConfirmPassword("Password1")
        viewModel.register()
        advanceUntilIdle()

        assertEquals(0, repository.smsRegisterCalls)
        assertEquals("请先获取短信验证码", viewModel.state.value.error)

        viewModel.sendSmsRegistrationCode()
        advanceUntilIdle()
        viewModel.register()
        advanceUntilIdle()

        assertEquals(1, repository.smsRegisterCalls)
        assertEquals("trail_user", repository.lastSmsRegisterRequest?.username)
        assertEquals("星野徒步者", repository.lastSmsRegisterRequest?.nickname)
        assertEquals("sms-register-ticket", repository.lastSmsRegisterRequest?.smsTicket)
        assertNull(viewModel.state.value.error)
    }

    @Test
    fun phonePasswordResetUsesSmsResetTicket() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = AuthViewModel(repository, AuthMode.RESET_PASSWORD)

        viewModel.updatePhone("13800000000")
        viewModel.sendSmsPasswordResetCode()
        advanceUntilIdle()
        viewModel.updateSmsCode("654321")
        viewModel.updateResetPassword("Password2")
        viewModel.updateResetConfirmPassword("Password2")
        viewModel.resetPassword()
        advanceUntilIdle()

        assertEquals(1, repository.smsResetPasswordCalls)
        assertEquals("13800000000", repository.lastSmsResetPhone)
        assertEquals("sms-reset-ticket", repository.lastSmsResetTicket)
        assertEquals("654321", repository.lastSmsResetCode)
        assertFalse(viewModel.state.value.loading)
        assertNull(viewModel.state.value.error)
    }

    @Test
    fun registerAndResetPasswordCanStillUseEmailFlows() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = AuthViewModel(repository, AuthMode.REGISTER)

        viewModel.setRegisterMethod(AuthRegisterMethod.EMAIL)
        viewModel.updateUsername("trail_user")
        viewModel.updateEmail("trail@example.test")
        viewModel.updateEmailCode("123456")
        viewModel.updatePassword("Password1")
        viewModel.updateConfirmPassword("Password1")
        viewModel.register()
        advanceUntilIdle()

        assertEquals(1, repository.registerCalls)
        assertEquals("trail_user", repository.lastRegisterRequest?.username)

        viewModel.switchMode(AuthMode.RESET_PASSWORD)
        viewModel.setResetMethod(AuthResetMethod.EMAIL)
        viewModel.updateEmail("trail@example.test")
        viewModel.updateEmailCode("654321")
        viewModel.updateResetPassword("Password2")
        viewModel.updateResetConfirmPassword("Password2")
        viewModel.resetPassword()
        advanceUntilIdle()

        assertEquals(1, repository.resetPasswordCalls)
        assertEquals("trail@example.test", repository.lastResetEmail)
        assertFalse(viewModel.state.value.loading)
        assertNull(viewModel.state.value.error)
    }

    private class FakeAuthRepository : AuthRepositoryContract {
        private val sessionState = MutableStateFlow<UserSession?>(null)
        override val session: StateFlow<UserSession?> = sessionState
        var emailLoginCalls = 0
        var registerCalls = 0
        var resetPasswordCalls = 0
        var smsLoginCodeCalls = 0
        var smsLoginCalls = 0
        var smsRegisterCalls = 0
        var smsResetPasswordCalls = 0
        var lastEmailLoginEmail: String? = null
        var lastRegisterRequest: RegisterRequest? = null
        var lastResetEmail: String? = null
        var lastSmsLoginPhone: String? = null
        var lastSmsLoginTicket: String? = null
        var lastSmsLoginCode: String? = null
        var lastSmsRegisterRequest: SmsRegisterRequest? = null
        var lastSmsResetPhone: String? = null
        var lastSmsResetTicket: String? = null
        var lastSmsResetCode: String? = null

        override suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse = emailCodeResponse(email)
        override suspend fun sendEmailLoginCode(email: String): EmailVerificationCodeResponse = emailCodeResponse(email)

        override suspend fun loginWithEmailCode(email: String, emailCode: String): LoginResponse {
            emailLoginCalls += 1
            lastEmailLoginEmail = email
            return loginResponse(email)
        }

        override suspend fun sendPasswordResetCode(email: String): EmailVerificationCodeResponse = emailCodeResponse(email)

        override suspend fun resetPassword(
            email: String,
            emailCode: String,
            password: String,
            confirmPassword: String,
        ): LoginResponse {
            resetPasswordCalls += 1
            lastResetEmail = email
            return loginResponse(email)
        }

        override suspend fun sendBindEmailCode(email: String): EmailVerificationCodeResponse = emailCodeResponse(email)
        override suspend fun bindEmail(email: String, emailCode: String): LoginUser = user(email)

        override suspend fun sendSmsRegistrationCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-register-ticket")

        override suspend fun smsRegister(request: SmsRegisterRequest): LoginResponse {
            smsRegisterCalls += 1
            lastSmsRegisterRequest = request
            return loginResponse(request.phone)
        }

        override suspend fun sendSmsLoginCode(phone: String): SmsCodeResponse {
            smsLoginCodeCalls += 1
            return smsCodeResponse(phone, "sms-login-ticket")
        }

        override suspend fun smsLogin(phone: String, smsTicket: String, smsCode: String): LoginResponse {
            smsLoginCalls += 1
            lastSmsLoginPhone = phone
            lastSmsLoginTicket = smsTicket
            lastSmsLoginCode = smsCode
            return loginResponse(phone)
        }

        override suspend fun sendSmsPasswordResetCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-reset-ticket")

        override suspend fun smsResetPassword(
            phone: String,
            smsTicket: String,
            smsCode: String,
            password: String,
            confirmPassword: String,
        ): LoginResponse {
            smsResetPasswordCalls += 1
            lastSmsResetPhone = phone
            lastSmsResetTicket = smsTicket
            lastSmsResetCode = smsCode
            return loginResponse(phone)
        }

        override suspend fun sendBindPhoneCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-bind-ticket")
        override suspend fun sendRebindCurrentPhoneCode(): SmsCodeResponse = smsCodeResponse("13800000000", "sms-current-ticket")
        override suspend fun bindPhone(
            phone: String,
            smsTicket: String,
            smsCode: String,
            currentSmsTicket: String?,
            currentSmsCode: String?,
        ): LoginUser = user(phone).copy(phone = phone)

        override suspend fun createCaptcha(account: String): CaptchaChallengeResponse = CaptchaChallengeResponse(
            captchaTicket = "fixture-ticket",
            captchaType = "image",
            imageSvg = "<svg />",
            expiresAt = "2099-01-01T00:00:00Z",
            debugAnswer = "1234",
        )

        override suspend fun login(
            account: String,
            password: String,
            captchaTicket: String?,
            captchaAnswer: String?,
        ): LoginResponse = loginResponse(account)

        override suspend fun register(request: RegisterRequest): LoginResponse {
            registerCalls += 1
            lastRegisterRequest = request
            return loginResponse(request.email)
        }

        override fun logout() {
            sessionState.value = null
        }

        private fun emailCodeResponse(email: String): EmailVerificationCodeResponse = EmailVerificationCodeResponse(
            email = email,
            expiresAt = "2099-01-01T00:00:00Z",
            debugCode = "123456",
        )

        private fun smsCodeResponse(phone: String, ticket: String): SmsCodeResponse = SmsCodeResponse(
            phone = phone,
            smsTicket = ticket,
            expiresAt = "2099-01-01T00:00:00Z",
            debugCode = "123456",
        )

        private fun loginResponse(account: String): LoginResponse = LoginResponse(
            accessToken = "fixture-access-token",
            expiresAt = "2099-01-01T00:00:00Z",
            refreshToken = "fixture-refresh-token",
            refreshExpiresAt = "2099-01-02T00:00:00Z",
            user = user(account),
        )

        private fun user(account: String): LoginUser = LoginUser(
            id = "fixture-user",
            username = account,
            email = "trail@example.test",
            phone = "13800000000",
            nickname = "星野徒步者",
        )
    }
}
