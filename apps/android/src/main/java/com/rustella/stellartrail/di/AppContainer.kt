package com.rustella.stellartrail.di

import android.content.Context
import com.rustella.stellartrail.core.config.AndroidAppConfigStore
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.map.AndroidMapStylePreferenceRepository
import com.rustella.stellartrail.core.map.MapStylePreferenceRepository
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.core.network.AndroidOfflineHttpCacheStore
import com.rustella.stellartrail.core.network.OfflineHttpCacheStore
import com.rustella.stellartrail.core.session.AndroidSessionStore
import com.rustella.stellartrail.core.session.SessionStore
import com.rustella.stellartrail.core.theme.AndroidThemeRepository
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.core.trail.AndroidPendingTrailImportStore
import com.rustella.stellartrail.core.trail.PendingTrailImportStore
import com.rustella.stellartrail.data.atlas.GearAtlasApi
import com.rustella.stellartrail.data.atlas.GearAtlasRepository
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.data.auth.AuthApi
import com.rustella.stellartrail.data.auth.AuthRepository
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.data.gear.GearApi
import com.rustella.stellartrail.data.gear.GearRepository
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.packing.PackingApi
import com.rustella.stellartrail.data.packing.PackingRepository
import com.rustella.stellartrail.data.packing.PackingRepositoryContract
import com.rustella.stellartrail.data.profile.ProfileApi
import com.rustella.stellartrail.data.profile.ProfileRepository
import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import com.rustella.stellartrail.data.skills.AndroidKnotCacheStore
import com.rustella.stellartrail.data.skills.SkillApi
import com.rustella.stellartrail.data.skills.SkillRepository
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.data.trail.TrailApi
import com.rustella.stellartrail.data.trail.TrailRepository
import com.rustella.stellartrail.data.trail.TrailRepositoryContract
import com.rustella.stellartrail.data.trip.TripApi
import com.rustella.stellartrail.data.trip.TripRepository
import com.rustella.stellartrail.data.trip.TripRepositoryContract

interface AppContainer {
    val configStore: AppConfigStore
    val sessionStore: SessionStore
    val themeRepository: ThemeRepository
    val mapStylePreferenceRepository: MapStylePreferenceRepository
    val pendingTrailImportStore: PendingTrailImportStore
    val offlineHttpCacheStore: OfflineHttpCacheStore
    val apiClient: ApiClient
    val authRepository: AuthRepositoryContract
    val gearRepository: GearRepositoryContract
    val gearAtlasRepository: GearAtlasRepositoryContract
    val packingRepository: PackingRepositoryContract
    val skillRepository: SkillRepositoryContract
    val trailRepository: TrailRepositoryContract
    val tripRepository: TripRepositoryContract
    val profileRepository: ProfileRepositoryContract
}

class DefaultAppContainer(context: Context) : AppContainer {
    override val configStore: AppConfigStore = AndroidAppConfigStore(context.applicationContext)
    override val sessionStore: SessionStore = AndroidSessionStore(context.applicationContext)
    override val themeRepository: ThemeRepository = AndroidThemeRepository(context.applicationContext)
    override val mapStylePreferenceRepository: MapStylePreferenceRepository =
        AndroidMapStylePreferenceRepository(context.applicationContext)
    override val pendingTrailImportStore: PendingTrailImportStore =
        AndroidPendingTrailImportStore(context.applicationContext)
    override val offlineHttpCacheStore: OfflineHttpCacheStore = AndroidOfflineHttpCacheStore(context.applicationContext)
    override val apiClient: ApiClient = ApiClient(
        configProvider = { configStore.config.value },
        tokenProvider = { sessionStore.currentToken() },
        refreshTokenProvider = { sessionStore.currentRefreshToken() },
        sessionRefreshHandler = { sessionStore.save(it) },
        sessionExpiredHandler = { sessionStore.clear() },
        offlineCacheStore = offlineHttpCacheStore,
        cacheScopeProvider = { sessionStore.session.value?.user?.id ?: "guest" },
    )
    override val authRepository: AuthRepositoryContract = AuthRepository(AuthApi(apiClient), sessionStore)
    override val gearRepository: GearRepositoryContract = GearRepository(GearApi(apiClient))
    override val gearAtlasRepository: GearAtlasRepositoryContract = GearAtlasRepository(GearAtlasApi(apiClient))
    override val packingRepository: PackingRepositoryContract = PackingRepository(PackingApi(apiClient))
    override val skillRepository: SkillRepositoryContract = SkillRepository(
        api = SkillApi(apiClient),
        cacheStore = AndroidKnotCacheStore(context.applicationContext),
    )
    override val trailRepository: TrailRepositoryContract = TrailRepository(TrailApi(apiClient))
    override val tripRepository: TripRepositoryContract = TripRepository(TripApi(apiClient))
    override val profileRepository: ProfileRepositoryContract = ProfileRepository(ProfileApi(apiClient))
}
