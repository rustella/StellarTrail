package com.rustella.stellartrail.feature.profile

import com.rustella.stellartrail.core.config.InMemoryAppConfigStore
import com.rustella.stellartrail.core.theme.InMemoryThemeRepository
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ProfileSettingsViewModelTest {
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
    fun bindPhoneUsesNewPhoneTicketAndUpdatesNotice() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = ProfileSettingsViewModel(repository, InMemoryThemeRepository(), InMemoryAppConfigStore())

        viewModel.sendBindPhoneCode("13900000000")
        advanceUntilIdle()
        viewModel.bindPhone("13900000000", "123456", null)
        advanceUntilIdle()

        assertEquals("13900000000", repository.lastBindPhone)
        assertEquals("new-phone-ticket", repository.lastBindSmsTicket)
        assertEquals("123456", repository.lastBindSmsCode)
        assertEquals(null, repository.lastCurrentSmsTicket)
        assertEquals("绑定完成", viewModel.actionState.value.phoneNotice)
        assertTrue(viewModel.actionState.value.phoneBindingCompleted)

        viewModel.consumePhoneBindingCompletion()

        assertFalse(viewModel.actionState.value.phoneBindingCompleted)
        assertEquals(null, viewModel.actionState.value.phoneNotice)
    }

    @Test
    fun rebindPhoneUsesCurrentAndNewPhoneTickets() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = ProfileSettingsViewModel(repository, InMemoryThemeRepository(), InMemoryAppConfigStore())

        viewModel.sendRebindCurrentPhoneCode()
        viewModel.sendBindPhoneCode("13900000000")
        advanceUntilIdle()
        viewModel.bindPhone("13900000000", "123456", "654321")
        advanceUntilIdle()

        assertEquals("current-phone-ticket", repository.lastCurrentSmsTicket)
        assertEquals("654321", repository.lastCurrentSmsCode)
        assertEquals("new-phone-ticket", repository.lastBindSmsTicket)
        assertEquals("修改完成", viewModel.actionState.value.phoneNotice)
        assertTrue(viewModel.actionState.value.phoneBindingCompleted)
    }

    @Test
    fun phonePasswordResetUsesSmsResetTicket() = runTest {
        val repository = FakeAuthRepository()
        val viewModel = ProfileSettingsViewModel(repository, InMemoryThemeRepository(), InMemoryAppConfigStore())

        viewModel.sendSmsPasswordResetCode("13800000000")
        advanceUntilIdle()
        viewModel.resetPasswordByPhone("13800000000", "123456", "Password2", "Password2")
        advanceUntilIdle()

        assertEquals("13800000000", repository.lastSmsResetPhone)
        assertEquals("password-reset-ticket", repository.lastSmsResetTicket)
        assertEquals("123456", repository.lastSmsResetCode)
        assertEquals("密码已更新", viewModel.actionState.value.passwordNotice)
    }

    private class FakeAuthRepository : AuthRepositoryContract {
        private val sessionState = MutableStateFlow<UserSession?>(null)
        override val session: StateFlow<UserSession?> = sessionState
        var lastBindPhone: String? = null
        var lastBindSmsTicket: String? = null
        var lastBindSmsCode: String? = null
        var lastCurrentSmsTicket: String? = null
        var lastCurrentSmsCode: String? = null
        var lastSmsResetPhone: String? = null
        var lastSmsResetTicket: String? = null
        var lastSmsResetCode: String? = null

        override suspend fun sendBindPhoneCode(phone: String): SmsCodeResponse = SmsCodeResponse(
            phone = phone,
            smsTicket = "new-phone-ticket",
            expiresAt = "2099-01-01T00:00:00Z",
            debugCode = "123456",
        )

        override suspend fun sendRebindCurrentPhoneCode(): SmsCodeResponse = SmsCodeResponse(
            phone = "13800000000",
            smsTicket = "current-phone-ticket",
            expiresAt = "2099-01-01T00:00:00Z",
            debugCode = "654321",
        )

        override suspend fun bindPhone(
            phone: String,
            smsTicket: String,
            smsCode: String,
            currentSmsTicket: String?,
            currentSmsCode: String?,
        ): LoginUser {
            lastBindPhone = phone
            lastBindSmsTicket = smsTicket
            lastBindSmsCode = smsCode
            lastCurrentSmsTicket = currentSmsTicket
            lastCurrentSmsCode = currentSmsCode
            return user(phone)
        }

        override suspend fun sendSmsPasswordResetCode(phone: String): SmsCodeResponse = SmsCodeResponse(
            phone = phone,
            smsTicket = "password-reset-ticket",
            expiresAt = "2099-01-01T00:00:00Z",
            debugCode = "123456",
        )

        override suspend fun smsResetPassword(
            phone: String,
            smsTicket: String,
            smsCode: String,
            password: String,
            confirmPassword: String,
        ): LoginResponse {
            lastSmsResetPhone = phone
            lastSmsResetTicket = smsTicket
            lastSmsResetCode = smsCode
            return loginResponse(phone)
        }

        override suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse = unused()
        override suspend fun sendEmailLoginCode(email: String): EmailVerificationCodeResponse = unused()
        override suspend fun loginWithEmailCode(email: String, emailCode: String): LoginResponse = unused()
        override suspend fun sendPasswordResetCode(email: String): EmailVerificationCodeResponse = unused()
        override suspend fun resetPassword(email: String, emailCode: String, password: String, confirmPassword: String): LoginResponse = unused()
        override suspend fun sendBindEmailCode(email: String): EmailVerificationCodeResponse = unused()
        override suspend fun bindEmail(email: String, emailCode: String): LoginUser = unused()
        override suspend fun sendSmsRegistrationCode(phone: String): SmsCodeResponse = unused()
        override suspend fun smsRegister(request: SmsRegisterRequest): LoginResponse = unused()
        override suspend fun sendSmsLoginCode(phone: String): SmsCodeResponse = unused()
        override suspend fun smsLogin(phone: String, smsTicket: String, smsCode: String): LoginResponse = unused()
        override suspend fun createCaptcha(account: String): CaptchaChallengeResponse = unused()
        override suspend fun login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?): LoginResponse = unused()
        override suspend fun register(request: RegisterRequest): LoginResponse = unused()
        override fun logout() = Unit

        private fun loginResponse(account: String): LoginResponse = LoginResponse(
            accessToken = "fixture-access-token",
            expiresAt = "2099-01-01T00:00:00Z",
            refreshToken = "fixture-refresh-token",
            refreshExpiresAt = "2099-01-02T00:00:00Z",
            user = user(account),
        )

        private fun user(phone: String): LoginUser = LoginUser(
            id = "fixture-user",
            username = "trail_user",
            email = "trail@example.test",
            phone = phone,
            nickname = "星野徒步者",
        )

        private fun <T> unused(): T = error("unused fake method")
    }
}
