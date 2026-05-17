package com.rustella.stellartrail.di

import android.content.Context
import com.rustella.stellartrail.core.config.AndroidAppConfigStore
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.core.session.AndroidSessionStore
import com.rustella.stellartrail.core.session.SessionStore
import com.rustella.stellartrail.core.theme.AndroidThemeRepository
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.data.auth.AuthApi
import com.rustella.stellartrail.data.auth.AuthRepository
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.data.gear.GearApi
import com.rustella.stellartrail.data.gear.GearRepository
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.skills.SkillApi
import com.rustella.stellartrail.data.skills.SkillRepository
import com.rustella.stellartrail.data.skills.SkillRepositoryContract

interface AppContainer {
    val configStore: AppConfigStore
    val sessionStore: SessionStore
    val themeRepository: ThemeRepository
    val apiClient: ApiClient
    val authRepository: AuthRepositoryContract
    val gearRepository: GearRepositoryContract
    val skillRepository: SkillRepositoryContract
}

class DefaultAppContainer(context: Context) : AppContainer {
    override val configStore: AppConfigStore = AndroidAppConfigStore(context.applicationContext)
    override val sessionStore: SessionStore = AndroidSessionStore(context.applicationContext)
    override val themeRepository: ThemeRepository = AndroidThemeRepository(context.applicationContext)
    override val apiClient: ApiClient = ApiClient(
        configProvider = { configStore.config.value },
        tokenProvider = { sessionStore.currentToken() },
        refreshTokenProvider = { sessionStore.currentRefreshToken() },
        sessionRefreshHandler = { sessionStore.save(it) },
        sessionExpiredHandler = { sessionStore.clear() },
    )
    override val authRepository: AuthRepositoryContract = AuthRepository(AuthApi(apiClient), sessionStore)
    override val gearRepository: GearRepositoryContract = GearRepository(GearApi(apiClient))
    override val skillRepository: SkillRepositoryContract = SkillRepository(SkillApi(apiClient))
}
