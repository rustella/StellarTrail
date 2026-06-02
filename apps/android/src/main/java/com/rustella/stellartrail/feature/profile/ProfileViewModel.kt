package com.rustella.stellartrail.feature.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.BuildConfig
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class ProfileViewModel(
    private val authRepository: AuthRepositoryContract,
    private val themeRepository: ThemeRepository,
    private val appConfigStore: AppConfigStore,
    private val profileRepository: ProfileRepositoryContract? = null,
) : ViewModel() {
    val session = authRepository.session
    val theme: StateFlow<ThemeMode> = themeRepository.theme
    val config = appConfigStore.config
    val canEditBaseUrl: Boolean = BuildConfig.DEBUG && BuildConfig.APPLICATION_ID.endsWith(".debug")

    fun setTheme(theme: ThemeMode) = themeRepository.setTheme(theme)
    fun updateBaseUrl(value: String) {
        if (canEditBaseUrl) appConfigStore.updateBaseUrl(value)
    }
    fun resetBaseUrl() {
        if (canEditBaseUrl) appConfigStore.resetBaseUrl()
    }
    fun refreshCurrentProfile() {
        if (session.value == null) return
        val repository = profileRepository ?: return
        viewModelScope.launch {
            runCatching { repository.currentProfile() }.onSuccess { response ->
                authRepository.updateSessionUser(response.user)
            }
        }
    }
    fun logout() = authRepository.logout()
}
