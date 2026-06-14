package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.feature.profile.ProfileCacheKind

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

data class ProfileCacheItem(
    val kind: ProfileCacheKind,
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

    val dialogHelpActions = setOf(ProfileHelpAction.Feedback)

    const val cacheTitle = "缓存"
    const val cacheDescription = "管理可离线查看的内容。"
    const val cacheSectionTitle = "可离线内容"
    const val cacheSelectAction = "选择缓存项"
    const val cacheSelectAllAction = "全选"
    const val cacheInvertSelectionAction = "反选"
    const val cacheDoneAction = "完成"
    const val cacheSelectedAction = "缓存选中项"
    const val deleteSelectedAction = "删除选中缓存"
    const val cacheKnotsAction = "缓存绳结"
    const val cacheClearKnotsAction = "清空绳结"
    const val autoCacheAction = "自动缓存"
    const val cacheClearVisitedDataAction = "清空数据"
    val cacheItems = listOf(
        ProfileCacheItem(ProfileCacheKind.Knots, "绳", "绳结缓存", "常用绳结列表和详情可离线查看。"),
        ProfileCacheItem(ProfileCacheKind.VisitedData, "数", "已访问数据", "在线浏览过的页面数据可离线查看。"),
    )

    fun knotCacheStatusLabel(count: Int): String = if (count > 0) "已缓存 $count 个" else "未缓存"
    fun visitedDataCacheStatusLabel(count: Int): String = if (count > 0) "已缓存 $count 条" else "未缓存"

    const val aboutTitle = "更多信息"

    val aboutItems = listOf(
        ProfileAboutItem("图", "产品路线图", "查看功能计划，投票或订阅你关心的方向。", ProfileAboutAction.Roadmap),
        ProfileAboutItem("版", "版本信息", "查看当前版本与更新说明。", ProfileAboutAction.VersionInfo),
    )

    fun helpDialog(action: ProfileHelpAction): Pair<String, String> = when (action) {
        ProfileHelpAction.Feedback -> "意见反馈" to "可以告诉我们遇到的问题，或留下你希望改进的功能。"
        ProfileHelpAction.Cache,
        ProfileHelpAction.AboutHub -> error("No dialog copy is defined for $action")
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
            cacheTitle,
            cacheDescription,
            cacheSectionTitle,
            cacheSelectAction,
            cacheSelectAllAction,
            cacheInvertSelectionAction,
            cacheDoneAction,
            cacheSelectedAction,
            deleteSelectedAction,
            cacheKnotsAction,
            cacheClearKnotsAction,
            autoCacheAction,
            cacheClearVisitedDataAction,
            debugDefaultEndpointLabel,
            debugCustomEndpointLabel,
        ) +
            helpItems.flatMap { listOf(it.title, it.description) } +
            listOf(knotCacheStatusLabel(0), knotCacheStatusLabel(3), visitedDataCacheStatusLabel(0), visitedDataCacheStatusLabel(3)) +
            cacheItems.flatMap { listOf(it.icon, it.title, it.description) } +
            aboutItems.flatMap { listOf(it.title, it.description) } +
            dialogHelpActions.flatMap { action ->
                val (title, body) = helpDialog(action)
                listOf(title, body)
            }

    private fun sameEndpoint(left: String, right: String): Boolean =
        left.trim().trimEnd('/') == right.trim().trimEnd('/')
}
