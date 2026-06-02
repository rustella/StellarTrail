package com.rustella.stellartrail.feature.profile

import com.rustella.stellartrail.core.network.InMemoryOfflineHttpCacheStore
import com.rustella.stellartrail.data.skills.KnotCacheStatus
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsRequest
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillLocale
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
class ProfileCacheViewModelTest {
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
    fun enterSelectionModeStartsWithEmptySelection() = runTest {
        val repository = FakeSkillRepository()
        val viewModel = createViewModel(repository)

        viewModel.enterSelectionMode()

        val state = viewModel.state.value
        assertTrue(state.selectionMode)
        assertEquals(emptySet<ProfileCacheKind>(), state.selectedCacheKinds)
    }

    @Test
    fun selectAllAndInvertSelectionUpdateSelectedKinds() = runTest {
        val repository = FakeSkillRepository()
        val viewModel = createViewModel(repository)

        viewModel.enterSelectionMode()
        viewModel.selectAllCacheKinds()
        assertEquals(setOf(ProfileCacheKind.Knots, ProfileCacheKind.VisitedData), viewModel.state.value.selectedCacheKinds)

        viewModel.invertCacheSelection()
        assertEquals(emptySet<ProfileCacheKind>(), viewModel.state.value.selectedCacheKinds)

        viewModel.invertCacheSelection()
        assertEquals(setOf(ProfileCacheKind.Knots, ProfileCacheKind.VisitedData), viewModel.state.value.selectedCacheKinds)
    }

    @Test
    fun emptySelectionDoesNotCallRepository() = runTest {
        val repository = FakeSkillRepository(KnotCacheStatus(cachedKnotCount = 2, lastUpdatedAtMillis = 1000L))
        val viewModel = createViewModel(repository)

        viewModel.enterSelectionMode()
        viewModel.cacheSelectedCaches()
        viewModel.deleteSelectedCaches()
        advanceUntilIdle()

        assertEquals(0, repository.cacheAllCalls)
        assertEquals(0, repository.clearCalls)
        assertEquals(null, viewModel.state.value.message)
    }

    @Test
    fun cacheSelectedKnotsUpdatesStatusAndMessage() = runTest {
        val repository = FakeSkillRepository()
        val viewModel = createViewModel(repository)

        viewModel.enterSelectionMode()
        viewModel.toggleCacheKind(ProfileCacheKind.Knots)
        viewModel.cacheSelectedCaches()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(1, repository.cacheAllCalls)
        assertEquals(2, state.status.cachedKnotCount)
        assertEquals("已缓存选中内容，包含 2 个绳结。", state.message)
        assertFalse(state.cachingSelected)
    }

    @Test
    fun deleteSelectedKnotsUpdatesStatusAndMessage() = runTest {
        val repository = FakeSkillRepository(KnotCacheStatus(cachedKnotCount = 2, lastUpdatedAtMillis = 1000L))
        val viewModel = createViewModel(repository)

        viewModel.enterSelectionMode()
        viewModel.toggleCacheKind(ProfileCacheKind.Knots)
        viewModel.deleteSelectedCaches()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(1, repository.clearCalls)
        assertEquals(0, state.status.cachedKnotCount)
        assertEquals("已删除选中缓存。", state.message)
        assertFalse(state.deletingSelected)
    }

    @Test
    fun cacheKnotsUpdatesItemStatusAndMessage() = runTest {
        val repository = FakeSkillRepository()
        val viewModel = createViewModel(repository)

        viewModel.cacheKnots()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(1, repository.cacheAllCalls)
        assertEquals(2, state.status.cachedKnotCount)
        assertEquals("已缓存 2 个绳结。", state.message)
        assertFalse(state.cachingKnots)
    }

    @Test
    fun clearKnotCacheUpdatesItemStatusAndMessage() = runTest {
        val repository = FakeSkillRepository(KnotCacheStatus(cachedKnotCount = 2, lastUpdatedAtMillis = 1000L))
        val viewModel = createViewModel(repository)

        viewModel.clearKnotCache()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(1, repository.clearCalls)
        assertEquals(0, state.status.cachedKnotCount)
        assertEquals("绳结缓存已清空。", state.message)
        assertFalse(state.clearingKnots)
    }

    @Test
    fun clearVisitedDataCacheUpdatesStatusAndMessage() = runTest {
        val repository = FakeSkillRepository()
        val offlineCacheStore = InMemoryOfflineHttpCacheStore()
        offlineCacheStore.write("key", """{"status":"ok"}""")
        val viewModel = createViewModel(repository, offlineCacheStore)

        viewModel.clearVisitedDataCache()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(0, state.offlineStatus.cachedResponseCount)
        assertEquals("已访问数据缓存已清空。", state.message)
        assertFalse(state.clearingVisitedData)
    }

    @Test
    fun deleteSelectedVisitedDataClearsOfflineCache() = runTest {
        val repository = FakeSkillRepository()
        val offlineCacheStore = InMemoryOfflineHttpCacheStore()
        offlineCacheStore.write("key", """{"status":"ok"}""")
        val viewModel = createViewModel(repository, offlineCacheStore)

        viewModel.enterSelectionMode()
        viewModel.toggleCacheKind(ProfileCacheKind.VisitedData)
        viewModel.deleteSelectedCaches()
        advanceUntilIdle()

        val state = viewModel.state.value
        assertEquals(0, repository.clearCalls)
        assertEquals(0, state.offlineStatus.cachedResponseCount)
        assertEquals("已删除选中缓存。", state.message)
    }

    private fun createViewModel(
        repository: FakeSkillRepository,
        offlineCacheStore: InMemoryOfflineHttpCacheStore = InMemoryOfflineHttpCacheStore(),
    ): ProfileCacheViewModel = ProfileCacheViewModel(repository, offlineCacheStore)

    private class FakeSkillRepository(
        initialStatus: KnotCacheStatus = KnotCacheStatus(),
    ) : SkillRepositoryContract {
        private val mutableStatus = MutableStateFlow(initialStatus)
        override val knotCacheStatus: StateFlow<KnotCacheStatus> = mutableStatus
        var cacheAllCalls = 0
        var clearCalls = 0

        override suspend fun cacheAllKnots(locale: SkillLocale): KnotCacheStatus {
            cacheAllCalls += 1
            mutableStatus.value = KnotCacheStatus(cachedKnotCount = 2, lastUpdatedAtMillis = 2000L)
            return mutableStatus.value
        }

        override suspend fun clearKnotCache(): KnotCacheStatus {
            clearCalls += 1
            mutableStatus.value = KnotCacheStatus()
            return mutableStatus.value
        }

        override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = error("unused")
        override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse = error("unused")
        override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail = error("unused")
        override suspend fun listFavoriteSkills(locale: SkillLocale, request: ListFavoriteSkillsRequest): ListFavoriteSkillsResponse = error("unused")
        override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
    }
}
