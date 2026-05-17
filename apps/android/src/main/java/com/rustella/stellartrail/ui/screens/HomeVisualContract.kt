package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.formatWeight

enum class HomeActionTarget { Gears, NewGear, Skills, Profile, Login }

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
                icon = "＋",
                title = "添加装备",
                body = "登录后快速记录装备",
                target = if (isLoggedIn) HomeActionTarget.NewGear else HomeActionTarget.Login,
            ),
            HomeQuickAction(
                icon = "🪢",
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
