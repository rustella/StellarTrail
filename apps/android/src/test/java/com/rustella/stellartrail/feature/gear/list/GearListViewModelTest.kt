package com.rustella.stellartrail.feature.gear.list

import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.domain.gear.CreateGearRequest
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.GearTemplate
import com.rustella.stellartrail.domain.gear.GearTemplateCategory
import com.rustella.stellartrail.domain.gear.ListGearTemplatesResponse
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.ListGearsResponse
import com.rustella.stellartrail.domain.gear.UpdateGearRequest
import com.rustella.stellartrail.feature.home.EMPTY_STATS
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.net.UnknownHostException

@OptIn(ExperimentalCoroutinesApi::class)
class GearListViewModelTest {
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
    fun guestGearListUsesPublicTemplatesAndBlocksPrivateActions() = runTest {
        val repository = FakeGearRepository()
        val viewModel = GearListViewModel(repository)

        viewModel.refresh(isLoggedIn = false)
        advanceUntilIdle()
        viewModel.loadMore()
        viewModel.archive("gear-1")
        viewModel.restore("gear-1")
        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.isLoggedIn)
        assertEquals(1, repository.templateCalls)
        assertEquals(0, repository.privateGearCalls)
        assertEquals("周末轻徒步清单", state.templates.single().title)
        assertTrue(state.gears.isEmpty())
        assertEquals(EMPTY_STATS, state.stats)
    }

    @Test
    fun loggedInGearListNetworkFailureUpdatesErrorWithoutCrashing() = runTest {
        val repository = FakeGearRepository(failStats = true)
        val viewModel = GearListViewModel(repository)

        viewModel.refresh(isLoggedIn = true)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
    }

    @Test
    fun loadIfNeededKeepsGearListDuringBackgroundFailure() = runTest {
        val repository = FakeGearRepository()
        val viewModel = GearListViewModel(repository)

        viewModel.loadIfNeeded(isLoggedIn = true)
        advanceUntilIdle()
        repository.failStats = true
        viewModel.loadIfNeeded(isLoggedIn = true)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertTrue(state.hasLoaded)
        assertFalse(state.loading)
        assertFalse(state.refreshing)
        assertEquals(listOf("登山包"), state.gears.map { it.name })
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
    }

    private class FakeGearRepository(
        var failStats: Boolean = false,
    ) : GearRepositoryContract {
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
            if (failStats) throw UnknownHostException("api.stellartrail.example")
            return EMPTY_STATS
        }

        override suspend fun list(request: ListGearsRequest): ListGearsResponse {
            privateGearCalls += 1
            return ListGearsResponse(
                listOf(
                    GearSummary(
                        id = "gear-1",
                        category = GearCategory.BACKPACK_SYSTEM,
                        categoryLabel = "背负与收纳",
                        name = "登山包",
                        status = GearStatus.AVAILABLE,
                        statusLabel = "可用",
                        createdAt = "2026-05-01T00:00:00Z",
                        updatedAt = "2026-05-01T00:00:00Z",
                    ),
                ),
                nextCursor = "next",
            )
        }

        override suspend fun get(id: String): GearItem = error("unused")
        override suspend fun create(request: CreateGearRequest): GearItem = error("unused")
        override suspend fun update(id: String, request: UpdateGearRequest): GearItem = error("unused")
        override suspend fun archive(id: String) {
            privateGearCalls += 1
        }
        override suspend fun delete(id: String) {
            privateGearCalls += 1
        }
        override suspend fun undelete(id: String): GearItem {
            privateGearCalls += 1
            error("unused")
        }
        override suspend fun restore(id: String): GearItem {
            privateGearCalls += 1
            error("unused")
        }
    }
}
