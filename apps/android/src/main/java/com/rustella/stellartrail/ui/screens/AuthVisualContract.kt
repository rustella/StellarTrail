package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.feature.auth.AuthMode

object AuthVisualContract {
    const val heroEyebrow = "寻径星野"
    const val heroTitle = "山野出发前的准备台"
    const val heroSubtitle = "装备准备、户外技能与路线知识，登录后同步保存你的出行进度。"

    const val loginSectionTitle = "选择登录方式"
    const val passwordTab = "密码登录"
    const val emailCodeTab = "验证码登录"
    const val passwordPrimaryAction = "登录"
    const val emailCodePrimaryAction = "验证码登录"
    const val registerLink = "注册账号"
    const val forgotPasswordLink = "忘记密码？"
    const val backToLogin = "返回登录"
    const val sendCodeAction = "获取验证码"

    val loginTabModes = listOf(AuthMode.LOGIN, AuthMode.EMAIL_CODE)
    val loginTabLabels = mapOf(
        AuthMode.LOGIN to passwordTab,
        AuthMode.EMAIL_CODE to emailCodeTab,
    )
}
