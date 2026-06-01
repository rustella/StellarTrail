package com.rustella.stellartrail.ui.screens

data class SkillCatalogCategory(
    val id: String,
    val icon: String,
    val title: String,
    val subtitle: String,
    val summary: String,
    val actionText: String,
)

object SkillsVisualContract {
    const val heroEyebrow = "寻径星野技能库"
    const val heroTitle = "户外技能"
    const val heroSubtitle = "绳结、扎营、打包、天气和急救知识，出发前随时复习。"
    const val favoriteTitle = "收藏清单"
    const val favoriteDescription = "快速找到已经收藏的户外技能"
    const val favoriteAction = "查看 >"

    val catalogCategories = listOf(
        SkillCatalogCategory(
            id = "knots",
            icon = "🪢",
            title = "绳结",
            subtitle = "Knots",
            summary = "常用露营、钓鱼、连接和固定绳结，按场景快速复习。",
            actionText = "查看绳结列表",
        ),
    )
}
