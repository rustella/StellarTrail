package com.rustella.stellartrail.ui.screens

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class ProfileVisualContractTest {
    @Test
    fun profileHomeUsesWechatCompactAccountAndSettingsCards() {
        assertEquals(2, ProfileVisualContract.maxPrimaryCardCount)
        assertEquals("查看账号资料与户外资料", ProfileVisualContract.accountSettingsEntryLabel)
    }

    @Test
    fun profileHomeHelpRowsMatchMiniProgramSettingsList() {
        val items = ProfileVisualContract.helpItems

        assertEquals(
            listOf("绳结离线缓存", "意见反馈", "产品路线图", "版本信息", "关于寻径星野"),
            items.map { it.title },
        )
        assertEquals(
            listOf(
                ProfileHelpAction.CachedKnots,
                ProfileHelpAction.Feedback,
                ProfileHelpAction.Roadmap,
                ProfileHelpAction.VersionInfo,
                ProfileHelpAction.About,
            ),
            items.map { it.action },
        )
        assertFalse(items.any { it.title == "我的工具" || it.title == "本地调试地址" })
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

        assertEquals("默认接口", defaultSummary)
        assertEquals("自定义调试地址", customSummary)
        assertFalse(defaultSummary.contains("http"))
        assertFalse(customSummary.contains("10.0.2.2"))
    }
}
