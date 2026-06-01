package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.gear.GearStatsResponse
import org.junit.Assert.assertEquals
import org.junit.Test

class HomeVisualContractTest {
    @Test
    fun quickActionsMatchWechatDayModeHomeGrid() {
        val actions = HomeQuickAction.defaults(isLoggedIn = false)

        assertEquals(listOf("装备库", "我的行程", "户外技能", "个人设置"), actions.map { it.title })
        assertEquals(listOf(HomeActionTarget.Gears, HomeActionTarget.Trips, HomeActionTarget.Skills, HomeActionTarget.Profile), actions.map { it.target })
        assertEquals(4, actions.size)
    }

    @Test
    fun quickActionsKeepTripEntryWhenLoggedIn() {
        val actions = HomeQuickAction.defaults(isLoggedIn = true)

        assertEquals(HomeActionTarget.Trips, actions.single { it.title == "我的行程" }.target)
    }

    @Test
    fun gearOverviewUsesThreeCompactStats() {
        val stats = GearStatsResponse(
            currentCount = 6,
            archivedCount = 2,
            totalValueCents = 129900,
            totalWeightG = 3250,
        )

        val overview = HomeGearOverview.from(stats = stats, isLoggedIn = true)

        assertEquals("装备准备", overview.eyebrow)
        assertEquals("装备概览", overview.title)
        assertEquals(listOf("装备数量", "总重量", "装备估值"), overview.stats.map { it.label })
        assertEquals(listOf("当前库存", "已记录装备重量", "按 CNY 购入价汇总"), overview.stats.map { it.hint })
        assertEquals(listOf("6", "3.25 kg", "¥1299"), overview.stats.map { it.value })
    }
    @Test
    fun heroLayoutReservesSpaceBelowActionButtons() {
        assertEquals(20, HomeHeroVisualContract.contentPaddingDp)
        assertEquals(12, HomeHeroVisualContract.actionRowTopGapDp)
        assertEquals(20, HomeHeroVisualContract.actionBottomSafeGapDp)
        assertEquals(16, HomeHeroVisualContract.followingSectionGapDp)
    }

    @Test
    fun darkHeroUsesSparseStarDecoration() {
        val stars = HomeHeroVisualContract.nightStars

        assertEquals(7, stars.size)
        assertEquals(true, stars.any { it.accent })
        assertEquals(true, stars.all { it.xPercent in 0.55f..0.95f && it.yPercent in 0.08f..0.58f })
    }

}
