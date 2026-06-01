package com.rustella.stellartrail.feature.auth

import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.domain.auth.CaptchaChallengeResponse
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeResponse
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.RegisterRequest
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
        assertEquals("请填写用户名或邮箱和密码", viewModel.state.value.error)

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
    fun registerAndResetPasswordUseExistingRepositoryFlows() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = AuthViewModel(repository, AuthMode.REGISTER)

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
        var lastEmailLoginEmail: String? = null
        var lastRegisterRequest: RegisterRequest? = null
        var lastResetEmail: String? = null

        override suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse = codeResponse(email)
        override suspend fun sendEmailLoginCode(email: String): EmailVerificationCodeResponse = codeResponse(email)

        override suspend fun loginWithEmailCode(email: String, emailCode: String): LoginResponse {
            emailLoginCalls += 1
            lastEmailLoginEmail = email
            return loginResponse(email)
        }

        override suspend fun sendPasswordResetCode(email: String): EmailVerificationCodeResponse = codeResponse(email)

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

        override suspend fun sendBindEmailCode(email: String): EmailVerificationCodeResponse = codeResponse(email)
        override suspend fun bindEmail(email: String, emailCode: String): LoginUser = user(email)

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

        private fun codeResponse(email: String): EmailVerificationCodeResponse = EmailVerificationCodeResponse(
            email = email,
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
            nickname = "星野徒步者",
        )
    }
}
