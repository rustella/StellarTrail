package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.ui.common.HeroStar
import com.rustella.stellartrail.ui.common.HeroVisualContract
import java.util.Locale

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

data class HomeOverviewStat(val label: String, val value: String, val hint: String)

data class HomeGearOverview(
    val eyebrow: String,
    val title: String,
    val promptTitle: String?,
    val promptBody: String?,
    val stats: List<HomeOverviewStat>,
) {
    companion object {
        fun from(stats: GearStatsResponse, isLoggedIn: Boolean): HomeGearOverview = HomeGearOverview(
            eyebrow = "装备准备",
            title = "装备概览",
            promptTitle = if (isLoggedIn) null else "登录后管理装备",
            promptBody = if (isLoggedIn) null else "保存自己的装备、重量和估值，出发前整理清单更安心。",
            stats = listOf(
                HomeOverviewStat("装备数量", stats.currentCount.toString(), "当前库存"),
                HomeOverviewStat("总重量", formatHomeWeight(stats.totalWeightG), "已记录装备重量"),
                HomeOverviewStat("装备估值", formatPrice(stats.totalValueCents), "按 CNY 购入价汇总"),
            ),
        )
    }
}

fun formatHomeWeight(grams: Int): String {
    if (grams <= 0) return "0kg"
    if (grams < 1000) return "${grams}g"
    return String.format(Locale.US, "%.2f kg", grams / 1000.0)
}

object HomeHeroVisualContract {
    const val contentPaddingDp = HeroVisualContract.contentPaddingDp
    const val actionRowTopGapDp = HeroVisualContract.actionRowTopGapDp
    const val actionBottomSafeGapDp = HeroVisualContract.actionBottomSafeGapDp
    const val followingSectionGapDp = HeroVisualContract.followingSectionGapDp

    val nightStars: List<HeroStar> = HeroVisualContract.nightStars
}
