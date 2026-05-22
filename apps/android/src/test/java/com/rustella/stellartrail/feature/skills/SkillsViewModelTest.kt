package com.rustella.stellartrail.feature.skills

import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.PageInfo
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.skills.SkillLocale
import java.net.UnknownHostException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
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

        viewModel.load()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
    }

    private class FakeSkillRepository(
        private val failKnots: Boolean = false,
    ) : SkillRepositoryContract {
        override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = SkillCategoriesResponse(
            listOf(SkillCategorySummary("knots", "knots", "绳结", "常用户外绳结", 8, "/api/skills/knots")),
        )

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
                        href = "/api/skills/knots/bowline",
                    ),
                ),
                page = PageInfo(limit = 20, offset = 0),
            )
        }

        override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail = error("unused")
        override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
    }
}
