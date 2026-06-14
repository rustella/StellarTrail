package com.rustella.stellartrail.feature.profile

import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.profile.AppContentPage
import com.rustella.stellartrail.domain.profile.AppContentPageSection
import com.rustella.stellartrail.domain.profile.ClientVersion
import com.rustella.stellartrail.domain.profile.ClientVersionReleaseNoteSection
import com.rustella.stellartrail.domain.profile.ListClientVersionsResponse
import com.rustella.stellartrail.domain.profile.ListOutdoorExperiencesResponse
import com.rustella.stellartrail.domain.profile.ListRoadmapResponse
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.profile.OutdoorProfileResponse
import com.rustella.stellartrail.domain.profile.ProfileUserResponse
import com.rustella.stellartrail.domain.profile.RoadmapItem
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import java.io.IOException
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
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ProfileAboutViewModelTest {
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
    fun loadShowsDatabaseProfileAboutContent() = runTest {
        val viewModel = ProfileAboutViewModel(FakeProfileRepository(profileAboutContent = profileAboutContent()))

        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertNull(state.error)
        assertEquals("关于寻径星野", state.content?.title)
        assertEquals("数据库文案", state.content?.subtitle)
        assertEquals("来自数据库", state.content?.sections?.single()?.body)
    }

    @Test
    fun loadFailureDoesNotExposeLocalAboutFallback() = runTest {
        val viewModel = ProfileAboutViewModel(FakeProfileRepository(failProfileAboutContent = true))

        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertNotNull(state.error)
        assertNull(state.content)
    }

    @Test
    fun incompleteDatabaseContentDoesNotRenderAboutCardContent() = runTest {
        val viewModel = ProfileAboutViewModel(
            FakeProfileRepository(
                profileAboutContent = profileAboutContent(
                    sections = listOf(AppContentPageSection(icon = "", title = "出发准备", body = "来自数据库")),
                ),
            ),
        )

        advanceUntilIdle()

        val state = viewModel.state.value
        assertFalse(state.loading)
        assertEquals("关于内容暂不可用", state.error)
        assertNull(state.content)
    }

    @Test
    fun loadVersionInfoShowsDatabaseAndroidVersion() = runTest {
        val viewModel = ProfileAboutViewModel(
            FakeProfileRepository(
                profileAboutContent = profileAboutContent(),
                clientVersions = ListClientVersionsResponse(items = listOf(androidClientVersion())),
            ),
        )
        advanceUntilIdle()

        viewModel.loadVersionInfo()
        advanceUntilIdle()

        val versionInfo = viewModel.state.value.versionInfo
        assertFalse(versionInfo.loading)
        assertNull(versionInfo.error)
        assertEquals("0.0.1", versionInfo.versions.single().version)
        assertEquals("Android 0.0.1 初始版本", versionInfo.versions.single().title)
        assertEquals("主要功能", versionInfo.versions.single().sections.single().title)
        assertEquals(
            "关于页与版本信息改为读取数据库，便于后续按 Android 端独立维护。",
            versionInfo.versions.single().sections.single().items.last(),
        )
    }

    @Test
    fun loadVersionInfoFailureDoesNotExposeLocalVersionFallback() = runTest {
        val viewModel = ProfileAboutViewModel(
            FakeProfileRepository(
                profileAboutContent = profileAboutContent(),
                failClientVersions = true,
            ),
        )
        advanceUntilIdle()

        viewModel.loadVersionInfo()
        advanceUntilIdle()

        val versionInfo = viewModel.state.value.versionInfo
        assertFalse(versionInfo.loading)
        assertNotNull(versionInfo.error)
        assertEquals(emptyList<ProfileVersionUi>(), versionInfo.versions)
    }

    private fun profileAboutContent(
        sections: List<AppContentPageSection> = listOf(
            AppContentPageSection(icon = "星", title = "出发准备", body = "来自数据库"),
        ),
    ): AppContentPage =
        AppContentPage(
            pageKey = "profile_about",
            clientKey = "android",
            locale = "zh-CN",
            eyebrow = "寻径星野",
            title = "关于寻径星野",
            subtitle = "数据库文案",
            sections = sections,
            buttonText = "知道了",
            updatedAt = "2026-06-15T00:00:00Z",
        )

    private fun androidClientVersion(): ClientVersion =
        ClientVersion(
            id = "android-0-0-1",
            clientKey = "android",
            version = "0.0.1",
            title = "Android 0.0.1 初始版本",
            releaseNotes = emptyList(),
            releaseNoteSections = listOf(
                ClientVersionReleaseNoteSection(
                    key = "feature",
                    title = "Feature",
                    items = listOf(
                        "补齐账号登录、我的页面与资料入口，支持基础账号和户外资料管理。",
                        "上线装备库与装备图鉴，方便记录个人装备并查看公共装备信息。",
                        "支持户外技能与绳结内容浏览，常用内容可离线缓存。",
                        "支持行程规划、轨迹导入、轨迹库和地图预览，把出发前资料整理到手机端。",
                        "关于页与版本信息改为读取数据库，便于后续按 Android 端独立维护。",
                    ),
                ),
            ),
            status = "published",
            publishedAt = "2026-06-15T00:00:00Z",
            createdAt = "2026-06-15T00:00:00Z",
            updatedAt = "2026-06-15T00:00:00Z",
        )

    private class FakeProfileRepository(
        private val profileAboutContent: AppContentPage? = null,
        private val failProfileAboutContent: Boolean = false,
        private val clientVersions: ListClientVersionsResponse = ListClientVersionsResponse(),
        private val failClientVersions: Boolean = false,
    ) : ProfileRepositoryContract {
        override suspend fun currentProfile(): ProfileUserResponse =
            ProfileUserResponse(LoginUser(id = "user-1", username = "trail_user"))

        override suspend fun profileAboutContent(): AppContentPage {
            if (failProfileAboutContent) throw IOException("offline")
            return profileAboutContent ?: AppContentPage(
                pageKey = "profile_about",
                clientKey = "android",
                locale = "zh-CN",
                eyebrow = "寻径星野",
                title = "关于寻径星野",
                subtitle = "数据库文案",
                sections = listOf(AppContentPageSection(icon = "星", title = "出发准备", body = "来自数据库")),
                buttonText = "知道了",
                updatedAt = "2026-06-15T00:00:00Z",
            )
        }

        override suspend fun listAndroidClientVersions(): ListClientVersionsResponse {
            if (failClientVersions) throw IOException("offline")
            return clientVersions
        }

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
