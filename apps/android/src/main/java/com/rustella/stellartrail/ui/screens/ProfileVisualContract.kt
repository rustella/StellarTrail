package com.rustella.stellartrail.ui.screens

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
    const val debugDefaultEndpointLabel = "默认接口"
    const val debugCustomEndpointLabel = "自定义调试地址"

    val helpItems = listOf(
        ProfileHelpItem("绳", "绳结离线缓存", "缓存后离线也能查询绳结详情和动图。", ProfileHelpAction.CachedKnots),
        ProfileHelpItem("馈", "意见反馈", "告诉我们遇到的问题或想要的功能。", ProfileHelpAction.Feedback),
        ProfileHelpItem("图", "产品路线图", "查看后续功能计划，投票或订阅你关心的方向。", ProfileHelpAction.Roadmap),
        ProfileHelpItem("版", "版本信息", "点击查看版本更新", ProfileHelpAction.VersionInfo),
        ProfileHelpItem("星", "关于寻径星野", "为户外爱好者准备的出行、装备与技能工具。", ProfileHelpAction.About),
    )

    fun debugEndpointSummary(currentBaseUrl: String, defaultBaseUrl: String): String =
        if (sameEndpoint(currentBaseUrl, defaultBaseUrl)) debugDefaultEndpointLabel else debugCustomEndpointLabel

    private fun sameEndpoint(left: String, right: String): Boolean =
        left.trim().trimEnd('/') == right.trim().trimEnd('/')
}
