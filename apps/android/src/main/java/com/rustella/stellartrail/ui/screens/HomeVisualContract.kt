package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.ui.common.HeroStar
import com.rustella.stellartrail.ui.common.HeroVisualContract

enum class HomeActionTarget { Gears, NewGear, Trips, Skills, Profile, Login }

data class HomeQuickAction(
    val icon: String,
    val title: String,
    val body: String,
    val target: HomeActionTarget,
) {
    companion object {
        fun defaults(isLoggedIn: Boolean): List<HomeQuickAction> = listOf(
            HomeQuickAction(
                icon = "🎒",
                title = "装备库",
                body = "出行清单与我的装备",
                target = HomeActionTarget.Gears,
            ),
            HomeQuickAction(
                icon = "行",
                title = "我的行程",
                body = "单人准备与组队协作",
                target = HomeActionTarget.Trips,
            ),
            HomeQuickAction(
                icon = "结",
                title = "户外技能",
                body = "绳结、天气、急救知识",
                target = HomeActionTarget.Skills,
            ),
            HomeQuickAction(
                icon = "⚙",
                title = "个人设置",
                body = "账号与显示偏好",
                target = HomeActionTarget.Profile,
            ),
        )
    }
}

data class HomeOverviewStat(val label: String, val value: String)

data class HomeGearOverview(
    val eyebrow: String,
    val title: String,
    val promptTitle: String?,
    val promptBody: String?,
    val stats: List<HomeOverviewStat>,
) {
    companion object {
        fun from(stats: GearStatsResponse, isLoggedIn: Boolean): HomeGearOverview = HomeGearOverview(
            eyebrow = "GEAR READY",
            title = "装备概览",
            promptTitle = if (isLoggedIn) null else "可以先查看出行清单",
            promptBody = if (isLoggedIn) null else "登录后再管理自己的装备、重量和估值。",
            stats = listOf(
                HomeOverviewStat("可用装备", stats.currentCount.toString()),
                HomeOverviewStat("历史装备", stats.archivedCount.toString()),
                HomeOverviewStat("总重量", formatWeight(stats.totalWeightG)),
            ),
        )
    }
}

object HomeHeroVisualContract {
    const val contentPaddingDp = HeroVisualContract.contentPaddingDp
    const val actionRowTopGapDp = HeroVisualContract.actionRowTopGapDp
    const val actionBottomSafeGapDp = HeroVisualContract.actionBottomSafeGapDp
    const val followingSectionGapDp = HeroVisualContract.followingSectionGapDp

    val nightStars: List<HeroStar> = HeroVisualContract.nightStars
}
