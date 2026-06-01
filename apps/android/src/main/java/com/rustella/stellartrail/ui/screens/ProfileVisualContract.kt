package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.core.theme.ThemeMode

data class ProfileHelpItem(
    val icon: String,
    val title: String,
    val description: String,
    val action: ProfileHelpAction,
)

enum class ProfileHelpAction {
    Cache,
    Feedback,
    AboutHub,
}

data class ProfileAboutItem(
    val icon: String,
    val title: String,
    val description: String,
    val action: ProfileAboutAction,
)

data class ProfileAboutIntroItem(
    val icon: String,
    val title: String,
    val description: String,
)

enum class ProfileAboutAction {
    Roadmap,
    VersionInfo,
}

object ProfileVisualContract {
    const val maxPrimaryCardCount = 2
    const val accountSettingsEntryLabel = "查看账号资料与户外资料"
    const val nightModeTitle = "黑夜模式"
    const val themeLightIcon = "☀"
    const val themeDarkIcon = "☾"
    const val debugDefaultEndpointLabel = "默认连接"
    const val debugCustomEndpointLabel = "自定义连接"
    val blockedUserFacingFragments = listOf(
        "Android 端",
        "对齐小程序",
        "开发调试",
        "调试验证码",
        "调试文本",
        "后续接入",
    )

    val helpItems = listOf(
        ProfileHelpItem("存", "缓存", "管理可离线查看的内容。", ProfileHelpAction.Cache),
        ProfileHelpItem("馈", "意见反馈", "告诉我们遇到的问题或想要的功能。", ProfileHelpAction.Feedback),
        ProfileHelpItem("关", "关于", "版本、路线图与寻径星野介绍。", ProfileHelpAction.AboutHub),
    )

    const val aboutTitle = "关于"
    const val aboutBrandEyebrow = "寻径星野"
    const val aboutBrandTitle = "关于寻径星野"
    const val aboutBrandDescription = "把每次出发前的准备，整理得更安心。"

    val aboutIntroItems = listOf(
        ProfileAboutIntroItem(
            "🧭",
            "出发准备",
            "寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。",
        ),
        ProfileAboutIntroItem(
            "🎒",
            "山野陪伴",
            "它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。",
        ),
        ProfileAboutIntroItem(
            "✨",
            "作者的话",
            "这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。寻径星野会永久免费，无广告，不做打扰用户的商业化设计。",
        ),
    )

    val aboutItems = listOf(
        ProfileAboutItem("图", "产品路线图", "查看功能计划，投票或订阅你关心的方向。", ProfileAboutAction.Roadmap),
        ProfileAboutItem("版", "版本信息", "查看当前版本与更新说明。", ProfileAboutAction.VersionInfo),
    )

    fun helpDialog(action: ProfileHelpAction): Pair<String, String> = when (action) {
        ProfileHelpAction.Cache -> "缓存" to "当前支持常用绳结内容，更多类型会统一在这里管理。"
        ProfileHelpAction.Feedback -> "意见反馈" to "可以告诉我们遇到的问题，或留下你希望改进的功能。"
        ProfileHelpAction.AboutHub -> aboutBrandTitle to aboutBrandDescription
    }

    fun aboutDialog(action: ProfileAboutAction): Pair<String, String> = when (action) {
        ProfileAboutAction.VersionInfo -> "版本信息" to "版本更新会在这里展示。"
        ProfileAboutAction.Roadmap -> "产品路线图" to ""
    }

    fun nightModeDescription(theme: ThemeMode): String = when (theme) {
        ThemeMode.DARK -> "已开启深色界面。"
        ThemeMode.LIGHT -> "当前使用浅色界面。"
        ThemeMode.SYSTEM -> "跟随系统外观。"
    }

    fun debugEndpointSummary(currentBaseUrl: String, defaultBaseUrl: String): String =
        if (sameEndpoint(currentBaseUrl, defaultBaseUrl)) debugDefaultEndpointLabel else debugCustomEndpointLabel

    fun userFacingCopySamples(): List<String> =
        listOf(
            accountSettingsEntryLabel,
            nightModeTitle,
            themeLightIcon,
            themeDarkIcon,
            nightModeDescription(ThemeMode.DARK),
            nightModeDescription(ThemeMode.LIGHT),
            aboutTitle,
            aboutBrandEyebrow,
            aboutBrandTitle,
            aboutBrandDescription,
            debugDefaultEndpointLabel,
            debugCustomEndpointLabel,
        ) +
            helpItems.flatMap { listOf(it.title, it.description) } +
            aboutIntroItems.flatMap { listOf(it.icon, it.title, it.description) } +
            aboutItems.flatMap { listOf(it.title, it.description) } +
            ProfileHelpAction.entries.flatMap { action ->
                val (title, body) = helpDialog(action)
                listOf(title, body)
            } +
            ProfileAboutAction.entries.flatMap { action ->
                val (title, body) = aboutDialog(action)
                listOf(title, body)
            }

    private fun sameEndpoint(left: String, right: String): Boolean =
        left.trim().trimEnd('/') == right.trim().trimEnd('/')
}
