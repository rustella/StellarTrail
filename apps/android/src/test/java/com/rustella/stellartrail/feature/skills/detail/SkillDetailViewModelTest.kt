package com.rustella.stellartrail.feature.skills.detail

import com.rustella.stellartrail.data.skills.KnotCacheStatus
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.FavoriteKnotStatusResponse
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.KnotTaxonomyItem
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsRequest
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillLocale
import java.net.UnknownHostException
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
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SkillDetailViewModelTest {
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
    fun loadLoggedInLoadsDetailAndFavoriteStatus() = runTest {
        val repository = FakeSkillRepository(
            favoriteStatus = FavoriteKnotStatusResponse("knots", "taut-line", true, "2026-05-01T00:00:00Z"),
        )
        val viewModel = SkillDetailViewModel(repository, "taut-line")

        viewModel.load(isLoggedIn = true)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals("可调节绳结", state.detail?.title)
        assertTrue(state.isFavorited)
        assertEquals("2026-05-01T00:00:00Z", state.favoritedAt)
        assertEquals(1, repository.favoriteStatusCalls)
        assertNull(state.error)
    }

    @Test
    fun favoriteStatusFailureKeepsDetailVisible() = runTest {
        val repository = FakeSkillRepository(failFavoriteStatus = true)
        val viewModel = SkillDetailViewModel(repository, "taut-line")

        viewModel.load(isLoggedIn = true)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertNotNull(state.detail)
        assertFalse(state.isFavorited)
        assertNull(state.error)
        assertEquals(1, repository.favoriteStatusCalls)
    }

    @Test
    fun toggleFavoriteAndUnfavoriteUpdatesState() = runTest {
        val repository = FakeSkillRepository()
        val viewModel = SkillDetailViewModel(repository, "taut-line")
        viewModel.load(isLoggedIn = true)
        advanceUntilIdle()

        viewModel.toggleFavorite()
        advanceUntilIdle()

        assertTrue(viewModel.state.value.isFavorited)
        assertEquals(1, repository.favoriteCalls)

        viewModel.toggleFavorite()
        advanceUntilIdle()

        assertFalse(viewModel.state.value.isFavorited)
        assertEquals(1, repository.unfavoriteCalls)
    }

    @Test
    fun toggleWriteFailureRestoresOriginalFavoriteState() = runTest {
        val repository = FakeSkillRepository(
            favoriteStatus = FavoriteKnotStatusResponse("knots", "taut-line", true, "2026-05-01T00:00:00Z"),
            failWrites = true,
        )
        val viewModel = SkillDetailViewModel(repository, "taut-line")
        viewModel.load(isLoggedIn = true)
        advanceUntilIdle()

        viewModel.toggleFavorite()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertTrue(state.isFavorited)
        assertEquals("2026-05-01T00:00:00Z", state.favoritedAt)
        assertFalse(state.favoriteLoading)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.actionError)
        assertEquals(1, repository.unfavoriteCalls)
    }

    @Test
    fun loadLoggedOutDoesNotRequestFavoriteStatus() = runTest {
        val repository = FakeSkillRepository(
            favoriteStatus = FavoriteKnotStatusResponse("knots", "taut-line", true, "2026-05-01T00:00:00Z"),
        )
        val viewModel = SkillDetailViewModel(repository, "taut-line")

        viewModel.load(isLoggedIn = false)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertNotNull(state.detail)
        assertFalse(state.isFavorited)
        assertEquals(0, repository.favoriteStatusCalls)
    }

    private class FakeSkillRepository(
        private val favoriteStatus: FavoriteKnotStatusResponse = FavoriteKnotStatusResponse("knots", "taut-line", false, null),
        private val failFavoriteStatus: Boolean = false,
        private val failWrites: Boolean = false,
    ) : SkillRepositoryContract {
        override val knotCacheStatus: StateFlow<KnotCacheStatus> = MutableStateFlow(KnotCacheStatus())
        var favoriteStatusCalls = 0
        var favoriteCalls = 0
        var unfavoriteCalls = 0
        private var currentStatus = favoriteStatus

        override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail =
            KnotDetail(
                id = id,
                slug = id,
                title = "可调节绳结",
                summary = "调节绳索上的张力。",
                categories = listOf(KnotTaxonomyItem("camp", "camp", "露营")),
                types = listOf(KnotTaxonomyItem("tension", "tension", "张力调节")),
                locale = locale,
            )

        override suspend fun getFavoriteKnotStatus(id: String): FavoriteKnotStatusResponse {
            favoriteStatusCalls += 1
            if (failFavoriteStatus) throw UnknownHostException("api.stellartrail.example")
            return currentStatus.copy(knotId = id)
        }

        override suspend fun favoriteKnot(id: String): FavoriteKnotStatusResponse {
            favoriteCalls += 1
            if (failWrites) throw UnknownHostException("api.stellartrail.example")
            currentStatus = FavoriteKnotStatusResponse("knots", id, true, "2026-05-01T00:00:00Z")
            return currentStatus
        }

        override suspend fun unfavoriteKnot(id: String): FavoriteKnotStatusResponse {
            unfavoriteCalls += 1
            if (failWrites) throw UnknownHostException("api.stellartrail.example")
            currentStatus = FavoriteKnotStatusResponse("knots", id, false, null)
            return currentStatus
        }

        override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = error("unused")
        override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse = error("unused")
        override suspend fun listFavoriteSkills(locale: SkillLocale, request: ListFavoriteSkillsRequest): ListFavoriteSkillsResponse = error("unused")
        override suspend fun cacheAllKnots(locale: SkillLocale): KnotCacheStatus = error("unused")
        override suspend fun clearKnotCache(): KnotCacheStatus = error("unused")
        override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
    }
}
