package com.rustella.stellartrail.feature.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.BuildConfig
import com.rustella.stellartrail.core.network.ApiException
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.domain.auth.RegisterRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

enum class AuthMode { LOGIN, REGISTER }

data class AuthUiState(
    val mode: AuthMode = AuthMode.LOGIN,
    val account: String = "",
    val password: String = "",
    val username: String = "",
    val email: String = "",
    val confirmPassword: String = "",
    val emailCode: String = "",
    val captchaTicket: String = "",
    val captchaAnswer: String = "",
    val captchaSvg: String? = null,
    val debugCaptchaAnswer: String? = null,
    val notice: String? = null,
    val error: String? = null,
    val loading: Boolean = false,
)

class AuthViewModel(private val repository: AuthRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(AuthUiState())
    val state: StateFlow<AuthUiState> = _state.asStateFlow()

    fun switchMode(mode: AuthMode) {
        _state.update { it.copy(mode = mode, error = null, notice = null, captchaSvg = null, captchaTicket = "", captchaAnswer = "") }
    }

    fun updateAccount(value: String) = _state.update { it.copy(account = value) }
    fun updatePassword(value: String) = _state.update { it.copy(password = value) }
    fun updateUsername(value: String) = _state.update { it.copy(username = value) }
    fun updateEmail(value: String) = _state.update { it.copy(email = value) }
    fun updateConfirmPassword(value: String) = _state.update { it.copy(confirmPassword = value) }
    fun updateEmailCode(value: String) = _state.update { it.copy(emailCode = value) }
    fun updateCaptchaAnswer(value: String) = _state.update { it.copy(captchaAnswer = value) }

    fun login() {
        val current = _state.value
        if (current.account.isBlank() || current.password.isBlank()) {
            _state.update { it.copy(error = "请填写用户名或邮箱和密码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.login(
                    account = current.account,
                    password = current.password,
                    captchaTicket = current.captchaTicket.takeIf { it.isNotBlank() },
                    captchaAnswer = current.captchaAnswer.takeIf { it.isNotBlank() },
                )
            } catch (throwable: Throwable) {
                if (throwable is ApiException && throwable.isCaptchaRequired) {
                    loadCaptcha(current.account, "多次登录失败，请输入验证码后重试")
                } else {
                    _state.update { it.copy(error = throwable.userMessage()) }
                }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun sendEmailCode() {
        val email = _state.value.email.trim()
        if (email.isBlank()) {
            _state.update { it.copy(error = "请先填写邮箱") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                val response = repository.sendEmailCode(email)
                val notice = if (BuildConfig.DEBUG && response.debugCode != null) {
                    "本地验证码：${response.debugCode}"
                } else {
                    "验证码已发送至 ${response.email}"
                }
                _state.update { it.copy(notice = notice) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun register() {
        val current = _state.value
        if (current.password != current.confirmPassword) {
            _state.update { it.copy(error = "两次输入的密码不一致") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.register(
                    RegisterRequest(
                        username = current.username.trim(),
                        email = current.email.trim(),
                        password = current.password,
                        confirmPassword = current.confirmPassword,
                        emailVerificationCode = current.emailCode.trim(),
                    ),
                )
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun refreshCaptcha() {
        val account = _state.value.account.trim()
        if (account.isBlank()) {
            _state.update { it.copy(error = "请先填写用户名或邮箱") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                loadCaptcha(account, "验证码已刷新")
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    private suspend fun loadCaptcha(account: String, notice: String) {
        try {
            val response = repository.createCaptcha(account)
            _state.update {
                it.copy(
                    captchaTicket = response.captchaTicket,
                    captchaSvg = response.imageSvg,
                    debugCaptchaAnswer = response.debugAnswer.takeIf { BuildConfig.DEBUG },
                    captchaAnswer = "",
                    notice = notice,
                    error = null,
                )
            }
        } catch (throwable: Throwable) {
            _state.update { it.copy(error = throwable.userMessage()) }
        }
    }
}
