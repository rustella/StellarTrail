package com.rustella.stellartrail.feature.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.BuildConfig
import com.rustella.stellartrail.core.network.ApiException
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.SmsCodeResponse
import com.rustella.stellartrail.domain.auth.SmsRegisterRequest
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

enum class AuthMode { LOGIN, VERIFICATION_CODE, REGISTER, RESET_PASSWORD }

enum class AuthRegisterMethod { PHONE, EMAIL }

enum class AuthResetMethod { PHONE, EMAIL }

data class AuthUiState(
    val mode: AuthMode = AuthMode.LOGIN,
    val registerMethod: AuthRegisterMethod = AuthRegisterMethod.PHONE,
    val resetMethod: AuthResetMethod = AuthResetMethod.PHONE,
    val account: String = "",
    val password: String = "",
    val username: String = "",
    val nickname: String = "",
    val email: String = "",
    val phone: String = "",
    val verificationAccount: String = "",
    val verificationCode: String = "",
    val confirmPassword: String = "",
    val emailCode: String = "",
    val smsCode: String = "",
    val smsLoginTicket: String = "",
    val smsRegisterTicket: String = "",
    val smsResetTicket: String = "",
    val resetPassword: String = "",
    val resetConfirmPassword: String = "",
    val captchaTicket: String = "",
    val captchaAnswer: String = "",
    val captchaSvg: String? = null,
    val debugCaptchaAnswer: String? = null,
    val notice: String? = null,
    val error: String? = null,
    val loading: Boolean = false,
    val smsCooldownSecondsByPhone: Map<String, Int> = emptyMap(),
)

fun AuthUiState.smsCooldownRemaining(target: String): Int =
    smsCooldownSecondsByPhone[normalizeSmsCooldownKey(target)] ?: 0

fun smsCodeActionLabel(remainingSeconds: Int): String =
    if (remainingSeconds > 0) "重新获取 ${remainingSeconds}s" else "获取验证码"

class AuthViewModel(
    private val repository: AuthRepositoryContract,
    initialMode: AuthMode = AuthMode.LOGIN,
    private val smsCodeCooldownSeconds: Int = BuildConfig.SMS_CODE_COOLDOWN_SECONDS,
) : ViewModel() {
    private val _state = MutableStateFlow(AuthUiState(mode = initialMode))
    val state: StateFlow<AuthUiState> = _state.asStateFlow()
    private val smsCooldownJobs = mutableMapOf<String, Job>()

    fun switchMode(mode: AuthMode) {
        _state.update {
            it.copy(
                mode = mode,
                error = null,
                notice = null,
                captchaSvg = null,
                captchaTicket = "",
                captchaAnswer = "",
                emailCode = "",
                smsCode = "",
                verificationCode = "",
                smsLoginTicket = "",
                smsRegisterTicket = "",
                smsResetTicket = "",
            )
        }
    }

    fun updateAccount(value: String) = _state.update { it.copy(account = value) }
    fun updatePassword(value: String) = _state.update { it.copy(password = value) }
    fun updateUsername(value: String) = _state.update { it.copy(username = value) }
    fun updateNickname(value: String) = _state.update { it.copy(nickname = value) }
    fun updateEmail(value: String) = _state.update { it.copy(email = value) }
    fun updatePhone(value: String) = _state.update { it.copy(phone = value) }
    fun updateVerificationAccount(value: String) = _state.update { it.copy(verificationAccount = value, smsLoginTicket = "") }
    fun updateVerificationCode(value: String) = _state.update { it.copy(verificationCode = value) }
    fun updateConfirmPassword(value: String) = _state.update { it.copy(confirmPassword = value) }
    fun updateEmailCode(value: String) = _state.update { it.copy(emailCode = value) }
    fun updateSmsCode(value: String) = _state.update { it.copy(smsCode = value) }
    fun updateResetPassword(value: String) = _state.update { it.copy(resetPassword = value) }
    fun updateResetConfirmPassword(value: String) = _state.update { it.copy(resetConfirmPassword = value) }
    fun updateCaptchaAnswer(value: String) = _state.update { it.copy(captchaAnswer = value) }

    override fun onCleared() {
        smsCooldownJobs.values.forEach { it.cancel() }
        smsCooldownJobs.clear()
        super.onCleared()
    }

    fun setRegisterMethod(method: AuthRegisterMethod) {
        _state.update {
            it.copy(
                registerMethod = method,
                error = null,
                notice = null,
                emailCode = "",
                smsCode = "",
                smsRegisterTicket = "",
            )
        }
    }

    fun setResetMethod(method: AuthResetMethod) {
        _state.update {
            it.copy(
                resetMethod = method,
                error = null,
                notice = null,
                emailCode = "",
                smsCode = "",
                smsResetTicket = "",
            )
        }
    }

    fun login() {
        val current = _state.value
        if (current.account.isBlank() || current.password.isBlank()) {
            _state.update { it.copy(error = "请填写用户名、邮箱或手机号和密码") }
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
        viewModelScope.launch { sendEmailCodeRequest { repository.sendEmailCode(email).let { it.email to it.debugCode } } }
    }

    fun sendEmailLoginCode() {
        val email = _state.value.email.trim()
        if (email.isBlank()) {
            _state.update { it.copy(error = "请先填写邮箱") }
            return
        }
        viewModelScope.launch { sendEmailCodeRequest { repository.sendEmailLoginCode(email).let { it.email to it.debugCode } } }
    }

    fun loginWithEmailCode() {
        val current = _state.value
        if (current.email.isBlank() || current.emailCode.isBlank()) {
            _state.update { it.copy(error = "请填写邮箱和验证码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.loginWithEmailCode(current.email, current.emailCode)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun sendSmsLoginCode() {
        val phone = _state.value.phone.trim()
        if (phone.isBlank()) {
            _state.update { it.copy(error = "请先填写手机号") }
            return
        }
        viewModelScope.launch {
            sendSmsCodeRequest { repository.sendSmsLoginCode(phone) }
                ?.let { response -> _state.update { it.copy(smsLoginTicket = response.smsTicket) } }
        }
    }

    fun loginWithSmsCode() {
        val current = _state.value
        if (current.phone.isBlank() || current.smsCode.isBlank()) {
            _state.update { it.copy(error = "请填写手机号和短信验证码") }
            return
        }
        if (current.smsLoginTicket.isBlank()) {
            _state.update { it.copy(error = "请先获取短信验证码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.smsLogin(current.phone, current.smsLoginTicket, current.smsCode)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun sendVerificationLoginCode() {
        val target = _state.value.verificationAccount.trim()
        if (target.isBlank()) {
            _state.update { it.copy(error = "请先填写手机号或邮箱") }
            return
        }
        viewModelScope.launch {
            if (target.isEmailLoginTarget()) {
                _state.update { it.copy(smsLoginTicket = "") }
                sendEmailCodeRequest { repository.sendEmailLoginCode(target).let { it.email to it.debugCode } }
            } else {
                sendSmsCodeRequest(target) { repository.sendSmsLoginCode(target) }
                    ?.let { response -> _state.update { it.copy(smsLoginTicket = response.smsTicket) } }
            }
        }
    }

    fun loginWithVerificationCode() {
        val current = _state.value
        val target = current.verificationAccount.trim()
        val code = current.verificationCode.trim()
        if (target.isBlank() || code.isBlank()) {
            _state.update { it.copy(error = "请填写手机号或邮箱和验证码") }
            return
        }
        if (!target.isEmailLoginTarget() && current.smsLoginTicket.isBlank()) {
            _state.update { it.copy(error = "请先获取验证码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                if (target.isEmailLoginTarget()) {
                    repository.loginWithEmailCode(target, code)
                } else {
                    repository.smsLogin(target, current.smsLoginTicket, code)
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun sendPasswordResetCode() {
        val email = _state.value.email.trim()
        if (email.isBlank()) {
            _state.update { it.copy(error = "请先填写邮箱") }
            return
        }
        viewModelScope.launch { sendEmailCodeRequest { repository.sendPasswordResetCode(email).let { it.email to it.debugCode } } }
    }

    fun sendSmsRegistrationCode() {
        val phone = _state.value.phone.trim()
        if (phone.isBlank()) {
            _state.update { it.copy(error = "请先填写手机号") }
            return
        }
        viewModelScope.launch {
            sendSmsCodeRequest { repository.sendSmsRegistrationCode(phone) }
                ?.let { response -> _state.update { it.copy(smsRegisterTicket = response.smsTicket) } }
        }
    }

    fun sendSmsPasswordResetCode() {
        val phone = _state.value.phone.trim()
        if (phone.isBlank()) {
            _state.update { it.copy(error = "请先填写手机号") }
            return
        }
        viewModelScope.launch {
            sendSmsCodeRequest { repository.sendSmsPasswordResetCode(phone) }
                ?.let { response -> _state.update { it.copy(smsResetTicket = response.smsTicket) } }
        }
    }

    fun resetPassword() {
        val current = _state.value
        if (current.resetPassword != current.resetConfirmPassword) {
            _state.update { it.copy(error = "两次输入的密码不一致") }
            return
        }
        when (current.resetMethod) {
            AuthResetMethod.PHONE -> resetPasswordByPhone(current)
            AuthResetMethod.EMAIL -> resetPasswordByEmail(current)
        }
    }

    fun register() {
        val current = _state.value
        if (current.password != current.confirmPassword) {
            _state.update { it.copy(error = "两次输入的密码不一致") }
            return
        }
        when (current.registerMethod) {
            AuthRegisterMethod.PHONE -> registerByPhone(current)
            AuthRegisterMethod.EMAIL -> registerByEmail(current)
        }
    }

    fun refreshCaptcha() {
        val account = _state.value.account.trim()
        if (account.isBlank()) {
            _state.update { it.copy(error = "请先填写用户名、邮箱或手机号") }
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

    private fun registerByPhone(current: AuthUiState) {
        if (listOf(current.username, current.nickname, current.phone, current.smsCode, current.password).any { it.isBlank() }) {
            _state.update { it.copy(error = "请填写用户名、昵称、手机号、验证码和密码") }
            return
        }
        if (current.smsRegisterTicket.isBlank()) {
            _state.update { it.copy(error = "请先获取短信验证码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.smsRegister(
                    SmsRegisterRequest(
                        username = current.username,
                        nickname = current.nickname,
                        phone = current.phone,
                        password = current.password,
                        confirmPassword = current.confirmPassword,
                        smsTicket = current.smsRegisterTicket,
                        smsVerificationCode = current.smsCode,
                    ),
                )
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    private fun registerByEmail(current: AuthUiState) {
        if (listOf(current.username, current.email, current.emailCode, current.password).any { it.isBlank() }) {
            _state.update { it.copy(error = "请填写用户名、邮箱、验证码和密码") }
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

    private fun resetPasswordByPhone(current: AuthUiState) {
        if (current.phone.isBlank() || current.smsCode.isBlank() || current.resetPassword.isBlank()) {
            _state.update { it.copy(error = "请填写手机号、短信验证码和新密码") }
            return
        }
        if (current.smsResetTicket.isBlank()) {
            _state.update { it.copy(error = "请先获取短信验证码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.smsResetPassword(
                    phone = current.phone,
                    smsTicket = current.smsResetTicket,
                    smsCode = current.smsCode,
                    password = current.resetPassword,
                    confirmPassword = current.resetConfirmPassword,
                )
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    private fun resetPasswordByEmail(current: AuthUiState) {
        if (current.email.isBlank() || current.emailCode.isBlank() || current.resetPassword.isBlank()) {
            _state.update { it.copy(error = "请填写邮箱、验证码和新密码") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            try {
                repository.resetPassword(
                    email = current.email,
                    emailCode = current.emailCode,
                    password = current.resetPassword,
                    confirmPassword = current.resetConfirmPassword,
                )
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
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

    private suspend fun sendEmailCodeRequest(request: suspend () -> Pair<String, String?>) {
        _state.update { it.copy(loading = true, error = null, notice = null) }
        try {
            val (target, debugCode) = request()
            _state.update { it.copy(notice = codeNotice(target, debugCode)) }
        } catch (throwable: Throwable) {
            _state.update { it.copy(error = throwable.userMessage()) }
        } finally {
            _state.update { it.copy(loading = false) }
        }
    }

    private suspend fun sendSmsCodeRequest(request: suspend () -> SmsCodeResponse): SmsCodeResponse? {
        return sendSmsCodeRequest(_state.value.phone, request)
    }

    private suspend fun sendSmsCodeRequest(target: String, request: suspend () -> SmsCodeResponse): SmsCodeResponse? {
        if (!ensureSmsCooldownAllowsSend(target)) return null
        _state.update { it.copy(loading = true, error = null, notice = null) }
        return try {
            val response = request()
            startSmsCooldown(response.phone)
            _state.update { it.copy(notice = codeNotice(response.phone, response.debugCode)) }
            response
        } catch (throwable: Throwable) {
            val errorMessage = smsRateLimitMessage(target, throwable) ?: throwable.userMessage()
            _state.update { it.copy(error = errorMessage) }
            null
        } finally {
            _state.update { it.copy(loading = false) }
        }
    }

    private fun ensureSmsCooldownAllowsSend(target: String): Boolean {
        val remaining = _state.value.smsCooldownRemaining(target)
        if (remaining <= 0) return true
        _state.update { it.copy(error = "请 ${remaining} 秒后再获取验证码", notice = null) }
        return false
    }

    private fun smsRateLimitMessage(target: String, throwable: Throwable): String? {
        val exception = throwable as? ApiException
        if (exception?.isRateLimited != true) return null
        val retryAfterSeconds = exception.retryAfterSeconds?.toInt()?.takeIf { it > 0 }
            ?: smsCodeCooldownSeconds.coerceAtLeast(1)
        startSmsCooldown(target, retryAfterSeconds)
        return "验证码发送太频繁，请 ${retryAfterSeconds} 秒后重试"
    }

    private fun startSmsCooldown(target: String, seconds: Int = smsCodeCooldownSeconds) {
        val key = normalizeSmsCooldownKey(target)
        val normalizedSeconds = seconds.coerceAtLeast(0)
        if (key.isBlank() || normalizedSeconds <= 0) return
        smsCooldownJobs.remove(key)?.cancel()
        smsCooldownJobs[key] = viewModelScope.launch {
            for (remaining in normalizedSeconds downTo 1) {
                _state.update {
                    it.copy(smsCooldownSecondsByPhone = it.smsCooldownSecondsByPhone + (key to remaining))
                }
                delay(1_000)
            }
            _state.update {
                it.copy(smsCooldownSecondsByPhone = it.smsCooldownSecondsByPhone - key)
            }
            smsCooldownJobs.remove(key)
        }
    }

    private fun codeNotice(target: String, debugCode: String?): String =
        if (BuildConfig.DEBUG && debugCode != null) {
            "本地验证码：$debugCode"
        } else {
            "验证码已发送至 $target"
        }

    private fun String.isEmailLoginTarget(): Boolean = contains("@")
}

fun normalizeSmsCooldownKey(target: String): String {
    val digits = target.filter { it.isDigit() }
    return if (digits.length == 13 && digits.startsWith("86")) digits.drop(2) else digits
}
