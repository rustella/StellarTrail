package com.rustella.stellartrail.feature.home

import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.gear.CreateGearRequest
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.GearTemplate
import com.rustella.stellartrail.domain.gear.GearTemplateCategory
import com.rustella.stellartrail.domain.gear.ListGearTemplatesResponse
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.ListGearsResponse
import com.rustella.stellartrail.domain.gear.UpdateGearRequest
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.PageInfo
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.skills.SkillLocale
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
import java.net.UnknownHostException

@OptIn(ExperimentalCoroutinesApi::class)
class HomeViewModelTest {
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
    fun guestDashboardLoadsPublicTemplatesAndSkillsWithoutPrivateGearCalls() = runTest {
        val gearRepository = FakeGearRepository()
        val viewModel = HomeViewModel(gearRepository, FakeSkillRepository())

        viewModel.load(isLoggedIn = false)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.isLoggedIn)
        assertEquals(1, gearRepository.templateCalls)
        assertEquals(0, gearRepository.privateGearCalls)
        assertEquals("周末轻徒步清单", state.templates.single().title)
        assertEquals("绳结", state.skills.single().title)
    }

    @Test
    fun guestDashboardNetworkFailureUpdatesErrorWithoutCrashing() = runTest {
        val viewModel = HomeViewModel(
            gearRepository = FakeGearRepository(),
            skillRepository = FakeSkillRepository(failListSkills = true),
        )

        viewModel.load(isLoggedIn = false)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
    }

    private class FakeGearRepository : GearRepositoryContract {
        var templateCalls = 0
        var privateGearCalls = 0

        override suspend fun listTemplates(): ListGearTemplatesResponse {
            templateCalls += 1
            return ListGearTemplatesResponse(
                items = listOf(
                    GearTemplate(
                        id = "weekend-hike",
                        title = "周末轻徒步清单",
                        categories = listOf(
                            GearTemplateCategory("carry", "背负与收纳", listOf("20L 背包", "收纳袋")),
                        ),
                    ),
                ),
            )
        }

        override suspend fun listCategories(tab: GearTab): GearCategoriesResponse {
            privateGearCalls += 1
            return GearCategoriesResponse(emptyList())
        }

        override suspend fun stats(tab: GearTab): GearStatsResponse {
            privateGearCalls += 1
            return EMPTY_STATS
        }

        override suspend fun list(request: ListGearsRequest): ListGearsResponse {
            privateGearCalls += 1
            return ListGearsResponse(emptyList())
        }

        override suspend fun get(id: String): GearItem = error("unused")
        override suspend fun create(request: CreateGearRequest): GearItem = error("unused")
        override suspend fun update(id: String, request: UpdateGearRequest): GearItem = error("unused")
        override suspend fun archive(id: String) = Unit
        override suspend fun delete(id: String) = Unit
        override suspend fun undelete(id: String): GearItem = error("unused")
        override suspend fun restore(id: String): GearItem = error("unused")
    }

    private class FakeSkillRepository(
        private val failListSkills: Boolean = false,
    ) : SkillRepositoryContract {
        override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = SkillCategoriesResponse(
            listOf(SkillCategorySummary("knots", "knots", "绳结", "常用户外绳结", 8, "/api/v1/skills/knots")),
        ).also {
            if (failListSkills) throw UnknownHostException("api.stellartrail.example")
        }

        override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse =
            KnotListResponse(locale, emptyList(), PageInfo(limit = 20, offset = 0))

        override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail = error("unused")
        override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
    }
}
