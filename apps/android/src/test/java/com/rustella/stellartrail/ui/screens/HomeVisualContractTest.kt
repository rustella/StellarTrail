package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.gear.GearStatsResponse
import org.junit.Assert.assertEquals
import org.junit.Test

class HomeVisualContractTest {
    @Test
    fun quickActionsMatchWechatDayModeHomeGrid() {
        val actions = HomeQuickAction.defaults(isLoggedIn = false)

        assertEquals(listOf("装备库", "添加装备", "户外技能", "个人设置"), actions.map { it.title })
        assertEquals(listOf(HomeActionTarget.Gears, HomeActionTarget.Login, HomeActionTarget.Skills, HomeActionTarget.Profile), actions.map { it.target })
        assertEquals(4, actions.size)
    }

    @Test
    fun quickActionsOpenCreateGearWhenLoggedIn() {
        val actions = HomeQuickAction.defaults(isLoggedIn = true)

        assertEquals(HomeActionTarget.NewGear, actions.single { it.title == "添加装备" }.target)
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

        assertEquals("GEAR READY", overview.eyebrow)
        assertEquals("装备概览", overview.title)
        assertEquals(listOf("可用装备", "历史装备", "总重量"), overview.stats.map { it.label })
        assertEquals(listOf("6", "2", "3250g"), overview.stats.map { it.value })
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
