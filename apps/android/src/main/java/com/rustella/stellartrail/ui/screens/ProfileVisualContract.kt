package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.core.theme.ThemeMode

data class ProfileHelpItem(
    val icon: String,
    val title: String,
    val description: String,
    val action: ProfileHelpAction,
)

enum class ProfileHelpAction {
    CachedKnots,
    Feedback,
    Roadmap,
    VersionInfo,
    About,
}

object ProfileVisualContract {
    const val maxPrimaryCardCount = 2
    const val accountSettingsEntryLabel = "查看账号资料与户外资料"
    const val nightModeTitle = "黑夜模式"
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
        ProfileHelpItem("绳", "绳结离线缓存", "缓存后离线也能查询绳结详情和动图。", ProfileHelpAction.CachedKnots),
        ProfileHelpItem("馈", "意见反馈", "告诉我们遇到的问题或想要的功能。", ProfileHelpAction.Feedback),
        ProfileHelpItem("图", "产品路线图", "查看功能计划，投票或订阅你关心的方向。", ProfileHelpAction.Roadmap),
        ProfileHelpItem("版", "版本信息", "查看当前版本与更新说明。", ProfileHelpAction.VersionInfo),
        ProfileHelpItem("星", "关于寻径星野", "为户外爱好者准备的出行、装备与技能工具。", ProfileHelpAction.About),
    )

    fun helpDialog(action: ProfileHelpAction): Pair<String, String> = when (action) {
        ProfileHelpAction.CachedKnots -> "绳结离线缓存" to "离线缓存会保存常用绳结内容，方便在没有网络时继续查看。"
        ProfileHelpAction.Feedback -> "意见反馈" to "可以告诉我们遇到的问题，或留下你希望改进的功能。"
        ProfileHelpAction.VersionInfo -> "版本信息" to "版本更新会在这里展示。"
        ProfileHelpAction.About -> "关于寻径星野" to "寻径星野为户外爱好者准备装备、行程与技能工具，帮助出发前更从容。"
        ProfileHelpAction.Roadmap -> "产品路线图" to ""
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
            nightModeDescription(ThemeMode.DARK),
            nightModeDescription(ThemeMode.LIGHT),
            debugDefaultEndpointLabel,
            debugCustomEndpointLabel,
        ) +
            helpItems.flatMap { listOf(it.title, it.description) } +
            ProfileHelpAction.entries.flatMap { action ->
                val (title, body) = helpDialog(action)
                listOf(title, body)
            }

    private fun sameEndpoint(left: String, right: String): Boolean =
        left.trim().trimEnd('/') == right.trim().trimEnd('/')
}
