package com.rustella.stellartrail.feature.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.BuildConfig
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.profile.OutdoorProfile
import com.rustella.stellartrail.domain.profile.RoadmapItem
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonObjectBuilder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

data class RoadmapUiState(
    val loggedIn: Boolean = false,
    val selectedStatus: RoadmapStatusFilter = RoadmapStatusFilter.All,
    val loading: Boolean = false,
    val error: String? = null,
    val items: List<RoadmapItem> = emptyList(),
    val actionLoadingId: String? = null,
    val loginPromptMessage: String? = null,
)

class RoadmapViewModel(
    private val profileRepository: ProfileRepositoryContract,
    authRepository: AuthRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(RoadmapUiState())
    val state: StateFlow<RoadmapUiState> = _state

    init {
        viewModelScope.launch {
            authRepository.session.collect { session ->
                _state.update { it.copy(loggedIn = session != null) }
                load()
            }
        }
    }

    fun selectStatus(status: RoadmapStatusFilter) {
        if (_state.value.selectedStatus == status) return
        _state.update { it.copy(selectedStatus = status) }
        load()
    }

    fun load() {
        viewModelScope.launch {
            val snapshot = _state.value
            _state.update { it.copy(loading = true, error = null) }
            runCatching {
                profileRepository.listRoadmap(snapshot.loggedIn, snapshot.selectedStatus)
            }.onSuccess { response ->
                _state.update { it.copy(loading = false, items = response.items, error = null) }
            }.onFailure { error ->
                _state.update { it.copy(loading = false, items = emptyList(), error = error.userMessage()) }
            }
        }
    }

    fun toggleVote(item: RoadmapItem) = toggleRoadmapAction(
        item = item,
        loginMessage = "登录后可以给你关心的功能投票。",
        action = {
            if (item.isVoted) profileRepository.unvoteRoadmapItem(item.id) else profileRepository.voteRoadmapItem(item.id)
        },
    )

    fun toggleSubscription(item: RoadmapItem) = toggleRoadmapAction(
        item = item,
        loginMessage = "登录后可以订阅功能进度，在站内查看更新。",
        action = {
            if (item.isSubscribed) {
                profileRepository.unsubscribeRoadmapItem(item.id)
            } else {
                profileRepository.subscribeRoadmapItem(item.id)
            }
        },
    )

    fun dismissLoginPrompt() {
        _state.update { it.copy(loginPromptMessage = null) }
    }

    private fun toggleRoadmapAction(
        item: RoadmapItem,
        loginMessage: String,
        action: suspend () -> RoadmapItem,
    ) {
        if (!_state.value.loggedIn) {
            _state.update { it.copy(loginPromptMessage = loginMessage) }
            return
        }
        if (_state.value.actionLoadingId != null) return
        viewModelScope.launch {
            _state.update { it.copy(actionLoadingId = item.id, error = null) }
            runCatching { action() }.onSuccess { updated ->
                _state.update { state ->
                    state.copy(
                        actionLoadingId = null,
                        items = state.items.map { if (it.id == updated.id) updated else it },
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(actionLoadingId = null, error = error.userMessage()) }
            }
        }
    }
}

data class OutdoorProfileForm(
    val outdoorId: String = "",
    val realName: String = "",
    val gender: String = "",
    val birthDate: String = "",
    val heightCm: String = "",
    val bloodType: String = "",
    val phone: String = "",
    val emergencyContact: String = "",
    val emergencyContactRelationship: String = "",
    val emergencyPhone: String = "",
    val medicalHistory: String = "",
    val allergyHistory: String = "",
    val medicalResponseNote: String = "",
    val dietPreference: String = "",
    val insurancePolicyNo: String = "",
    val insuranceCompanyPhone: String = "",
)

data class OutdoorProfileUiState(
    val loggedIn: Boolean = false,
    val loading: Boolean = false,
    val saving: Boolean = false,
    val error: String? = null,
    val notice: String? = null,
    val form: OutdoorProfileForm = OutdoorProfileForm(),
)

class OutdoorProfileViewModel(
    private val profileRepository: ProfileRepositoryContract,
    authRepository: AuthRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(OutdoorProfileUiState())
    val state: StateFlow<OutdoorProfileUiState> = _state

    init {
        viewModelScope.launch {
            authRepository.session.collect { session ->
                val wasLoggedIn = _state.value.loggedIn
                if (session == null) {
                    _state.value = OutdoorProfileUiState()
                } else {
                    _state.update { it.copy(loggedIn = true) }
                    if (!wasLoggedIn) load()
                }
            }
        }
    }

    fun load() {
        if (!_state.value.loggedIn) return
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            runCatching { profileRepository.outdoorProfile() }.onSuccess { response ->
                _state.update {
                    it.copy(
                        loading = false,
                        error = null,
                        form = response.profile.toForm(),
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(loading = false, error = error.userMessage()) }
            }
        }
    }

    fun updateForm(form: OutdoorProfileForm) {
        _state.update { it.copy(form = form, notice = null) }
    }

    fun save() {
        if (!_state.value.loggedIn || _state.value.saving) return
        val form = _state.value.form
        val height = form.heightCm.trim().takeIf { it.isNotEmpty() }?.toIntOrNull()
        if (form.heightCm.isNotBlank() && height == null) {
            _state.update { it.copy(error = "身高需要填写数字") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(saving = true, error = null, notice = null) }
            runCatching { profileRepository.updateOutdoorProfile(form.toPayload()) }.onSuccess { response ->
                _state.update {
                    it.copy(
                        saving = false,
                        error = null,
                        notice = "户外资料已保存",
                        form = response.profile.toForm(),
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(saving = false, error = error.userMessage()) }
            }
        }
    }
}

data class OutdoorExperienceForm(
    val title: String = "",
    val startDate: String = "",
    val endDate: String = "",
    val dayCount: String = "",
    val companionCount: String = "",
    val routeSummary: String = "",
    val gearSummary: String = "",
    val foodSummary: String = "",
    val budgetSummary: String = "",
    val notes: String = "",
)

data class OutdoorExperiencesUiState(
    val loggedIn: Boolean = false,
    val loading: Boolean = false,
    val saving: Boolean = false,
    val error: String? = null,
    val items: List<OutdoorExperience> = emptyList(),
)

class OutdoorExperiencesViewModel(
    private val profileRepository: ProfileRepositoryContract,
    authRepository: AuthRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(OutdoorExperiencesUiState())
    val state: StateFlow<OutdoorExperiencesUiState> = _state

    init {
        viewModelScope.launch {
            authRepository.session.collect { session ->
                val wasLoggedIn = _state.value.loggedIn
                if (session == null) {
                    _state.value = OutdoorExperiencesUiState()
                } else {
                    _state.update { it.copy(loggedIn = true) }
                    if (!wasLoggedIn) load()
                }
            }
        }
    }

    fun load() {
        if (!_state.value.loggedIn) return
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            runCatching { profileRepository.listOutdoorExperiences() }.onSuccess { response ->
                _state.update { it.copy(loading = false, error = null, items = response.items) }
            }.onFailure { error ->
                _state.update { it.copy(loading = false, error = error.userMessage()) }
            }
        }
    }

    fun save(id: String?, form: OutdoorExperienceForm, onSaved: () -> Unit) {
        if (_state.value.saving) return
        val validationError = form.validationError()
        if (validationError != null) {
            _state.update { it.copy(error = validationError) }
            return
        }
        val request = form.toRequest()
        viewModelScope.launch {
            _state.update { it.copy(saving = true, error = null) }
            runCatching {
                if (id == null) {
                    profileRepository.createOutdoorExperience(request)
                } else {
                    profileRepository.updateOutdoorExperience(id, request)
                }
            }.onSuccess { saved ->
                _state.update { state ->
                    val nextItems = if (id == null) {
                        listOf(saved) + state.items
                    } else {
                        state.items.map { if (it.id == saved.id) saved else it }
                    }
                    state.copy(saving = false, items = nextItems, error = null)
                }
                onSaved()
            }.onFailure { error ->
                _state.update { it.copy(saving = false, error = error.userMessage()) }
            }
        }
    }

    fun delete(id: String) {
        if (_state.value.saving) return
        viewModelScope.launch {
            _state.update { it.copy(saving = true, error = null) }
            runCatching { profileRepository.deleteOutdoorExperience(id) }.onSuccess {
                _state.update { state ->
                    state.copy(saving = false, items = state.items.filterNot { it.id == id }, error = null)
                }
            }.onFailure { error ->
                _state.update { it.copy(saving = false, error = error.userMessage()) }
            }
        }
    }
}

data class ProfileSettingsActionState(
    val accountError: String? = null,
    val emailNotice: String? = null,
    val phoneNotice: String? = null,
    val passwordNotice: String? = null,
    val emailCodeLoading: Boolean = false,
    val emailBindingLoading: Boolean = false,
    val phoneCodeLoading: Boolean = false,
    val currentPhoneCodeLoading: Boolean = false,
    val phoneBindingLoading: Boolean = false,
    val bindPhoneSmsTicket: String = "",
    val currentPhoneSmsTicket: String = "",
    val passwordSmsTicket: String = "",
    val passwordCodeLoading: Boolean = false,
    val passwordLoading: Boolean = false,
)

class ProfileSettingsViewModel(
    private val authRepository: AuthRepositoryContract,
    private val themeRepository: ThemeRepository,
    private val appConfigStore: AppConfigStore,
) : ViewModel() {
    val session = authRepository.session
    val theme: StateFlow<ThemeMode> = themeRepository.theme
    val config = appConfigStore.config
    val canEditBaseUrl: Boolean = BuildConfig.DEBUG && BuildConfig.APPLICATION_ID.endsWith(".debug")

    private val _actionState = MutableStateFlow(ProfileSettingsActionState())
    val actionState: StateFlow<ProfileSettingsActionState> = _actionState

    fun setTheme(theme: ThemeMode) = themeRepository.setTheme(theme)

    fun updateBaseUrl(value: String) {
        if (canEditBaseUrl) appConfigStore.updateBaseUrl(value)
    }

    fun resetBaseUrl() {
        if (canEditBaseUrl) appConfigStore.resetBaseUrl()
    }

    fun logout() = authRepository.logout()

    fun clearMessages() {
        _actionState.update {
            it.copy(
                accountError = null,
                emailNotice = null,
                phoneNotice = null,
                passwordNotice = null,
                bindPhoneSmsTicket = "",
                currentPhoneSmsTicket = "",
                passwordSmsTicket = "",
            )
        }
    }

    fun sendBindEmailCode(email: String) {
        if (_actionState.value.emailCodeLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(emailCodeLoading = true, accountError = null, emailNotice = null) }
            runCatching { authRepository.sendBindEmailCode(email) }.onSuccess { response ->
                _actionState.update {
                    it.copy(emailCodeLoading = false, emailNotice = "验证码已发送到 ${response.email}。${debugSuffix(response.debugCode)}")
                }
            }.onFailure { error ->
                _actionState.update { it.copy(emailCodeLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun bindEmail(email: String, code: String) {
        if (_actionState.value.emailBindingLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(emailBindingLoading = true, accountError = null, emailNotice = null) }
            runCatching { authRepository.bindEmail(email, code) }.onSuccess {
                _actionState.update { it.copy(emailBindingLoading = false, emailNotice = "邮箱已绑定") }
            }.onFailure { error ->
                _actionState.update { it.copy(emailBindingLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun sendBindPhoneCode(phone: String) {
        if (_actionState.value.phoneCodeLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(phoneCodeLoading = true, accountError = null, phoneNotice = null, bindPhoneSmsTicket = "") }
            runCatching { authRepository.sendBindPhoneCode(phone) }.onSuccess { response ->
                _actionState.update {
                    it.copy(
                        phoneCodeLoading = false,
                        phoneNotice = "验证码已发送到 ${response.phone}。${debugSuffix(response.debugCode)}",
                        bindPhoneSmsTicket = response.smsTicket,
                    )
                }
            }.onFailure { error ->
                _actionState.update { it.copy(phoneCodeLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun sendRebindCurrentPhoneCode() {
        if (_actionState.value.currentPhoneCodeLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(currentPhoneCodeLoading = true, accountError = null, phoneNotice = null, currentPhoneSmsTicket = "") }
            runCatching { authRepository.sendRebindCurrentPhoneCode() }.onSuccess { response ->
                _actionState.update {
                    it.copy(
                        currentPhoneCodeLoading = false,
                        phoneNotice = "验证码已发送到 ${response.phone}。${debugSuffix(response.debugCode)}",
                        currentPhoneSmsTicket = response.smsTicket,
                    )
                }
            }.onFailure { error ->
                _actionState.update { it.copy(currentPhoneCodeLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun bindPhone(phone: String, code: String, currentCode: String?) {
        val snapshot = _actionState.value
        if (phone.isBlank() || code.isBlank()) {
            _actionState.update { it.copy(accountError = "请填写手机号和验证码") }
            return
        }
        if (snapshot.bindPhoneSmsTicket.isBlank()) {
            _actionState.update { it.copy(accountError = "请先获取新手机号验证码") }
            return
        }
        val normalizedCurrentCode = currentCode?.trim()?.takeIf { it.isNotEmpty() }
        if (currentCode != null && normalizedCurrentCode == null) {
            _actionState.update { it.copy(accountError = "请填写当前手机号验证码") }
            return
        }
        if (normalizedCurrentCode != null && snapshot.currentPhoneSmsTicket.isBlank()) {
            _actionState.update { it.copy(accountError = "请先获取当前手机号验证码") }
            return
        }
        if (_actionState.value.phoneBindingLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(phoneBindingLoading = true, accountError = null, phoneNotice = null) }
            runCatching {
                authRepository.bindPhone(
                    phone = phone,
                    smsTicket = snapshot.bindPhoneSmsTicket,
                    smsCode = code,
                    currentSmsTicket = snapshot.currentPhoneSmsTicket.takeIf { normalizedCurrentCode != null },
                    currentSmsCode = normalizedCurrentCode,
                )
            }.onSuccess {
                _actionState.update {
                    it.copy(
                        phoneBindingLoading = false,
                        phoneNotice = "手机号已更新",
                        bindPhoneSmsTicket = "",
                        currentPhoneSmsTicket = "",
                    )
                }
            }.onFailure { error ->
                _actionState.update { it.copy(phoneBindingLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun sendPasswordResetCode(email: String) {
        if (_actionState.value.passwordCodeLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(passwordCodeLoading = true, accountError = null, passwordNotice = null) }
            runCatching { authRepository.sendPasswordResetCode(email) }.onSuccess { response ->
                _actionState.update {
                    it.copy(passwordCodeLoading = false, passwordNotice = "验证码已发送到 ${response.email}。${debugSuffix(response.debugCode)}")
                }
            }.onFailure { error ->
                _actionState.update { it.copy(passwordCodeLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun sendSmsPasswordResetCode(phone: String) {
        if (_actionState.value.passwordCodeLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(passwordCodeLoading = true, accountError = null, passwordNotice = null, passwordSmsTicket = "") }
            runCatching { authRepository.sendSmsPasswordResetCode(phone) }.onSuccess { response ->
                _actionState.update {
                    it.copy(
                        passwordCodeLoading = false,
                        passwordNotice = "验证码已发送到 ${response.phone}。${debugSuffix(response.debugCode)}",
                        passwordSmsTicket = response.smsTicket,
                    )
                }
            }.onFailure { error ->
                _actionState.update { it.copy(passwordCodeLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun resetPassword(email: String, code: String, password: String, confirmPassword: String) {
        if (password != confirmPassword) {
            _actionState.update { it.copy(accountError = "两次输入的密码不一致") }
            return
        }
        if (_actionState.value.passwordLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(passwordLoading = true, accountError = null, passwordNotice = null) }
            runCatching { authRepository.resetPassword(email, code, password, confirmPassword) }.onSuccess {
                _actionState.update { it.copy(passwordLoading = false, passwordNotice = "密码已更新") }
            }.onFailure { error ->
                _actionState.update { it.copy(passwordLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    fun resetPasswordByPhone(phone: String, code: String, password: String, confirmPassword: String) {
        if (password != confirmPassword) {
            _actionState.update { it.copy(accountError = "两次输入的密码不一致") }
            return
        }
        val snapshot = _actionState.value
        if (phone.isBlank() || code.isBlank()) {
            _actionState.update { it.copy(accountError = "请填写手机号和验证码") }
            return
        }
        if (snapshot.passwordSmsTicket.isBlank()) {
            _actionState.update { it.copy(accountError = "请先获取短信验证码") }
            return
        }
        if (_actionState.value.passwordLoading) return
        viewModelScope.launch {
            _actionState.update { it.copy(passwordLoading = true, accountError = null, passwordNotice = null) }
            runCatching {
                authRepository.smsResetPassword(
                    phone = phone,
                    smsTicket = snapshot.passwordSmsTicket,
                    smsCode = code,
                    password = password,
                    confirmPassword = confirmPassword,
                )
            }.onSuccess {
                _actionState.update { it.copy(passwordLoading = false, passwordNotice = "密码已更新", passwordSmsTicket = "") }
            }.onFailure { error ->
                _actionState.update { it.copy(passwordLoading = false, accountError = error.userMessage()) }
            }
        }
    }

    private fun debugSuffix(debugCode: String?): String =
        if (BuildConfig.DEBUG && debugCode != null) " 验证码提示：$debugCode" else ""
}

fun OutdoorExperience.toForm(): OutdoorExperienceForm = OutdoorExperienceForm(
    title = title,
    startDate = startDate.orEmpty(),
    endDate = endDate.orEmpty(),
    dayCount = dayCount?.takeIf { it > 0 }?.toString().orEmpty(),
    companionCount = companionCount?.takeIf { it > 0 }?.toString().orEmpty(),
    routeSummary = routeSummary.orEmpty(),
    gearSummary = gearSummary.orEmpty(),
    foodSummary = foodSummary.orEmpty(),
    budgetSummary = budgetSummary.orEmpty(),
    notes = notes.orEmpty(),
)

private fun OutdoorProfile.toForm(): OutdoorProfileForm = OutdoorProfileForm(
    outdoorId = outdoorId.orEmpty(),
    realName = realName.orEmpty(),
    gender = gender.orEmpty(),
    birthDate = birthDate.orEmpty(),
    heightCm = heightCm?.toString().orEmpty(),
    bloodType = bloodType.orEmpty(),
    phone = phone.orEmpty(),
    emergencyContact = emergencyContact.orEmpty(),
    emergencyContactRelationship = emergencyContactRelationship.orEmpty(),
    emergencyPhone = emergencyPhone.orEmpty(),
    medicalHistory = medicalHistory.orEmpty(),
    allergyHistory = allergyHistory.orEmpty(),
    medicalResponseNote = medicalResponseNote.orEmpty(),
    dietPreference = dietPreference.orEmpty(),
    insurancePolicyNo = insurancePolicyNo.orEmpty(),
    insuranceCompanyPhone = insuranceCompanyPhone.orEmpty(),
)

private fun OutdoorProfileForm.toPayload(): JsonObject = buildJsonObject {
    putNullableString("outdoor_id", outdoorId)
    putNullableString("real_name", realName)
    putNullableString("gender", gender)
    putNullableString("birth_date", birthDate)
    val heightValue = heightCm.trim().takeIf { it.isNotEmpty() }?.toIntOrNull()
    put("height_cm", heightValue?.let { JsonPrimitive(it) } ?: JsonNull)
    putNullableString("phone", phone)
    putNullableString("emergency_contact", emergencyContact)
    putNullableString("emergency_contact_relationship", emergencyContactRelationship)
    putNullableString("emergency_phone", emergencyPhone)
    putNullableString("blood_type", bloodType)
    putNullableString("medical_history", medicalHistory)
    putNullableString("allergy_history", allergyHistory)
    putNullableString("medical_response_note", medicalResponseNote)
    putNullableString("diet_preference", dietPreference)
    putNullableString("insurance_policy_no", insurancePolicyNo)
    putNullableString("insurance_company_phone", insuranceCompanyPhone)
}

private fun JsonObjectBuilder.putNullableString(key: String, value: String) {
    val normalized = value.trim().takeIf { it.isNotEmpty() }
    put(key, normalized?.let { JsonPrimitive(it) } ?: JsonNull)
}

private fun OutdoorExperienceForm.toRequest(): OutdoorExperienceRequest = OutdoorExperienceRequest(
    title = title.trim(),
    startDate = startDate.trim().takeIf { it.isNotEmpty() },
    endDate = endDate.trim().takeIf { it.isNotEmpty() },
    dayCount = dayCount.trim().takeIf { it.isNotEmpty() }?.toLongOrNull(),
    companionCount = companionCount.trim().takeIf { it.isNotEmpty() }?.toLongOrNull(),
    routeSummary = routeSummary.trim().takeIf { it.isNotEmpty() },
    gearSummary = gearSummary.trim().takeIf { it.isNotEmpty() },
    foodSummary = foodSummary.trim().takeIf { it.isNotEmpty() },
    budgetSummary = budgetSummary.trim().takeIf { it.isNotEmpty() },
    notes = notes.trim().takeIf { it.isNotEmpty() },
)

private fun OutdoorExperienceForm.validationError(): String? {
    if (title.isBlank()) return "请填写经历标题"
    if (dayCount.isNotBlank() && dayCount.toLongOrNull() == null) return "天数需要填写数字"
    if (companionCount.isNotBlank() && companionCount.toLongOrNull() == null) return "同行人数需要填写数字"
    return null
}
