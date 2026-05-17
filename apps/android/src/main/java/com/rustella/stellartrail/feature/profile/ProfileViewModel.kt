package com.rustella.stellartrail.feature.profile

import androidx.lifecycle.ViewModel
import com.rustella.stellartrail.BuildConfig
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import kotlinx.coroutines.flow.StateFlow

class ProfileViewModel(
    private val authRepository: AuthRepositoryContract,
    private val themeRepository: ThemeRepository,
    private val appConfigStore: AppConfigStore,
) : ViewModel() {
    val session = authRepository.session
    val theme: StateFlow<ThemeMode> = themeRepository.theme
    val config = appConfigStore.config
    val canEditBaseUrl: Boolean = BuildConfig.DEBUG

    fun setTheme(theme: ThemeMode) = themeRepository.setTheme(theme)
    fun updateBaseUrl(value: String) {
        if (BuildConfig.DEBUG) appConfigStore.updateBaseUrl(value)
    }
    fun resetBaseUrl() {
        if (BuildConfig.DEBUG) appConfigStore.resetBaseUrl()
    }
    fun logout() = authRepository.logout()
}
