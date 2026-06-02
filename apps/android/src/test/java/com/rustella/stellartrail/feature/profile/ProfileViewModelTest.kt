package com.rustella.stellartrail.feature.profile

import com.rustella.stellartrail.core.config.InMemoryAppConfigStore
import com.rustella.stellartrail.core.session.InMemorySessionStore
import com.rustella.stellartrail.core.theme.InMemoryThemeRepository
import com.rustella.stellartrail.data.auth.AuthApi
import com.rustella.stellartrail.data.auth.AuthRepository
import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.UserSession
import com.rustella.stellartrail.domain.profile.ListOutdoorExperiencesResponse
import com.rustella.stellartrail.domain.profile.ListRoadmapResponse
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.profile.OutdoorProfileResponse
import com.rustella.stellartrail.domain.profile.ProfileUserResponse
import com.rustella.stellartrail.domain.profile.RoadmapItem
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.serialization.json.JsonObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ProfileViewModelTest {
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
    fun refreshCurrentProfileStoresLatestAvatarUrlInSession() = runTest {
        val sessionStore = InMemorySessionStore(
            UserSession(
                accessToken = "access-token",
                expiresAt = "2099-01-01T00:00:00Z",
                refreshToken = "refresh-token",
                refreshExpiresAt = "2099-01-02T00:00:00Z",
                user = LoginUser(id = "user-1", username = "trail_user", nickname = "旧头像用户"),
            ),
        )
        val authRepository = AuthRepository(AuthApi(unusedApiClient()), sessionStore)
        val viewModel = ProfileViewModel(
            authRepository = authRepository,
            themeRepository = InMemoryThemeRepository(),
            appConfigStore = InMemoryAppConfigStore(),
            profileRepository = FakeProfileRepository(
                LoginUser(
                    id = "user-1",
                    username = "trail_user",
                    nickname = "星野徒步者",
                    avatarUrl = "https://assets.example.test/users/user-1/avatar/custom.png",
                ),
            ),
        )

        viewModel.refreshCurrentProfile()
        advanceUntilIdle()

        assertEquals(
            "https://assets.example.test/users/user-1/avatar/custom.png",
            sessionStore.session.value?.user?.avatarUrl,
        )
    }

    private fun unusedApiClient(): ApiClient {
        val engine = MockEngine { error("unused") }
        return ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )
    }

    private class FakeProfileRepository(private val user: LoginUser) : ProfileRepositoryContract {
        override suspend fun currentProfile(): ProfileUserResponse = ProfileUserResponse(user)
        override suspend fun outdoorProfile(): OutdoorProfileResponse = unused()
        override suspend fun updateOutdoorProfile(request: JsonObject): OutdoorProfileResponse = unused()
        override suspend fun listOutdoorExperiences(): ListOutdoorExperiencesResponse = unused()
        override suspend fun createOutdoorExperience(request: OutdoorExperienceRequest): OutdoorExperience = unused()
        override suspend fun updateOutdoorExperience(id: String, request: OutdoorExperienceRequest): OutdoorExperience = unused()
        override suspend fun deleteOutdoorExperience(id: String) = Unit
        override suspend fun listRoadmap(isLoggedIn: Boolean, status: RoadmapStatusFilter): ListRoadmapResponse = unused()
        override suspend fun voteRoadmapItem(id: String): RoadmapItem = unused()
        override suspend fun unvoteRoadmapItem(id: String): RoadmapItem = unused()
        override suspend fun subscribeRoadmapItem(id: String): RoadmapItem = unused()
        override suspend fun unsubscribeRoadmapItem(id: String): RoadmapItem = unused()

        private fun <T> unused(): T = error("unused")
    }
}
