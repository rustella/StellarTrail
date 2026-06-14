package com.rustella.stellartrail.feature.skills

import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.data.skills.KnotCacheStatus
import com.rustella.stellartrail.domain.skills.FavoriteKnotItem
import com.rustella.stellartrail.domain.skills.FavoriteKnotStatusResponse
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsRequest
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.PageInfo
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SkillsViewModelTest {
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
    fun loadNetworkFailureUpdatesErrorWithoutCrashing() = runTest {
        val viewModel = SkillsViewModel(FakeSkillRepository(failKnots = true))

        viewModel.openKnots()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
    }

    @Test
    fun openingFavoritesLoadsFavoriteSkillList() = runTest {
        val viewModel = SkillsViewModel(FakeSkillRepository())

        viewModel.openFavorites()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(SkillsMode.Favorites, state.mode)
        assertEquals(listOf("单套结"), state.favoriteKnots.map { it.knot.title })
    }

    @Test
    fun loadIfNeededKeepsCatalogDuringBackgroundFailure() = runTest {
        val repository = FakeSkillRepository()
        val viewModel = SkillsViewModel(repository)

        viewModel.loadIfNeeded()
        advanceUntilIdle()
        repository.failSkills = true
        viewModel.loadIfNeeded()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertTrue(state.hasLoaded)
        assertFalse(state.loading)
        assertFalse(state.refreshing)
        assertEquals(listOf("绳结"), state.categories.map { it.title })
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
    }

    private class FakeSkillRepository(
        private val failKnots: Boolean = false,
    ) : SkillRepositoryContract {
        override val knotCacheStatus: StateFlow<KnotCacheStatus> = MutableStateFlow(KnotCacheStatus())
        var failSkills = false

        override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse {
            if (failSkills) throw UnknownHostException("api.stellartrail.example")
            return SkillCategoriesResponse(
                listOf(SkillCategorySummary("knots", "knots", "绳结", "常用户外绳结", 8, "/api/v1/skills/knots")),
            )
        }

        override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse {
            if (failKnots) throw UnknownHostException("api.stellartrail.example")
            return KnotListResponse(
                locale = locale,
                items = listOf(
                    KnotSummary(
                        id = "bowline",
                        slug = "bowline",
                        title = "单套结",
                        summary = "固定绳圈",
                        href = "/api/v1/skills/knots/bowline",
                    ),
                ),
                page = PageInfo(limit = 20, offset = 0),
            )
        }

        override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail = error("unused")
        override suspend fun listFavoriteSkills(locale: SkillLocale, request: ListFavoriteSkillsRequest): ListFavoriteSkillsResponse =
            ListFavoriteSkillsResponse(
                locale = locale,
                items = listOf(
                    FavoriteKnotItem(
                        skillCategory = "knots",
                        favoritedAt = "2026-05-01T00:00:00Z",
                        knot = KnotSummary(
                            id = "bowline",
                            slug = "bowline",
                            title = "单套结",
                            summary = "固定绳圈",
                            href = "/api/v1/skills/knots/bowline",
                        ),
                    ),
                ),
                page = PageInfo(limit = request.limit, offset = request.offset),
            )
        override suspend fun getFavoriteKnotStatus(id: String): FavoriteKnotStatusResponse = error("unused")
        override suspend fun favoriteKnot(id: String): FavoriteKnotStatusResponse = error("unused")
        override suspend fun unfavoriteKnot(id: String): FavoriteKnotStatusResponse = error("unused")
        override suspend fun cacheAllKnots(locale: SkillLocale): KnotCacheStatus = error("unused")
        override suspend fun clearKnotCache(): KnotCacheStatus = error("unused")
        override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
    }
}
