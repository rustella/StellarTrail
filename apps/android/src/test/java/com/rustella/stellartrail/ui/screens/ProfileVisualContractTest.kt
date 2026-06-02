package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.core.theme.ThemeMode
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class ProfileVisualContractTest {
    @Test
    fun profileHomeKeepsCompactAccountAndSettingsCards() {
        assertEquals(2, ProfileVisualContract.maxPrimaryCardCount)
        assertEquals("查看账号资料与户外资料", ProfileVisualContract.accountSettingsEntryLabel)
    }

    @Test
    fun profileHomeHelpRowsStayCompactAndUserFacing() {
        val items = ProfileVisualContract.helpItems

        assertEquals(
            listOf("缓存", "意见反馈", "关于"),
            items.map { it.title },
        )
        assertEquals(setOf(ProfileHelpAction.Feedback), ProfileVisualContract.dialogHelpActions)
        assertFalse(ProfileVisualContract.dialogHelpActions.contains(ProfileHelpAction.Cache))
        assertEquals(
            listOf(
                ProfileHelpAction.Cache,
                ProfileHelpAction.Feedback,
                ProfileHelpAction.AboutHub,
            ),
            items.map { it.action },
        )
        assertFalse(items.any { it.title in listOf("我的工具", "本地调试地址", "绳结离线缓存", "产品路线图", "版本信息", "关于寻径星野") })
    }

    @Test
    fun cachePageHasDedicatedContentInsteadOfDialogCopy() {
        assertEquals("缓存", ProfileVisualContract.cacheTitle)
        assertEquals("管理可离线查看的内容。", ProfileVisualContract.cacheDescription)
        assertEquals("全部缓存操作", ProfileVisualContract.cacheActionTitle)
        assertEquals("缓存所有内容", ProfileVisualContract.cacheAllContentAction)
        assertEquals("删除所有缓存", ProfileVisualContract.cacheDeleteAllAction)
        assertEquals("可离线内容", ProfileVisualContract.cacheSectionTitle)
        assertEquals("缓存绳结", ProfileVisualContract.cacheKnotsAction)
        assertEquals("清空绳结", ProfileVisualContract.cacheClearKnotsAction)
        assertEquals(
            listOf("绳结缓存"),
            ProfileVisualContract.cacheItems.map { it.title },
        )
        assertEquals("未缓存", ProfileVisualContract.knotCacheStatusLabel(0))
        assertEquals("已缓存 3 个", ProfileVisualContract.knotCacheStatusLabel(3))
    }

    @Test
    fun aboutPageCollectsRoadmapVersionAndBrandIntro() {
        assertEquals("更多信息", ProfileVisualContract.aboutTitle)
        assertEquals("寻径星野", ProfileVisualContract.aboutBrandEyebrow)
        assertEquals("关于寻径星野", ProfileVisualContract.aboutBrandTitle)
        assertEquals("把每次出发前的准备，整理得更安心。", ProfileVisualContract.aboutBrandDescription)
        assertEquals(
            listOf("出发准备", "山野陪伴", "作者的话"),
            ProfileVisualContract.aboutIntroItems.map { it.title },
        )
        assertEquals(
            listOf("🧭", "🎒", "✨"),
            ProfileVisualContract.aboutIntroItems.map { it.icon },
        )
        assertEquals(
            listOf("产品路线图", "版本信息"),
            ProfileVisualContract.aboutItems.map { it.title },
        )
        assertEquals(
            listOf(ProfileAboutAction.Roadmap, ProfileAboutAction.VersionInfo),
            ProfileVisualContract.aboutItems.map { it.action },
        )
    }

    @Test
    fun nightModeHasExplicitUserFacingLabelAndStateCopy() {
        assertEquals("黑夜模式", ProfileVisualContract.nightModeTitle)
        assertEquals("☀", ProfileVisualContract.themeLightIcon)
        assertEquals("☾", ProfileVisualContract.themeDarkIcon)
        assertEquals("已开启深色界面。", ProfileVisualContract.nightModeDescription(ThemeMode.DARK))
        assertEquals("当前使用浅色界面。", ProfileVisualContract.nightModeDescription(ThemeMode.LIGHT))
        assertEquals("跟随系统外观。", ProfileVisualContract.nightModeDescription(ThemeMode.SYSTEM))
    }

    @Test
    fun debugEndpointSummaryDoesNotExposeUrlText() {
        val defaultSummary = ProfileVisualContract.debugEndpointSummary(
            currentBaseUrl = "https://api.example.invalid",
            defaultBaseUrl = "https://api.example.invalid/",
        )
        val customSummary = ProfileVisualContract.debugEndpointSummary(
            currentBaseUrl = "http://10.0.2.2:8080",
            defaultBaseUrl = "https://api.example.invalid",
        )

        assertEquals("默认连接", defaultSummary)
        assertEquals("自定义连接", customSummary)
        assertFalse(defaultSummary.contains("http"))
        assertFalse(customSummary.contains("10.0.2.2"))
    }

    @Test
    fun profileUserFacingCopyAvoidsImplementationLanguage() {
        val copy = ProfileVisualContract.userFacingCopySamples().joinToString("\n")

        ProfileVisualContract.blockedUserFacingFragments.forEach { fragment ->
            assertFalse("Unexpected user-facing implementation copy: $fragment", copy.contains(fragment))
        }
    }
}
