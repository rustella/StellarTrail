package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.feature.auth.AuthMode

object AuthVisualContract {
    const val heroEyebrow = "寻径星野"
    const val heroTitle = "山野出发前的准备台"
    const val heroSubtitle = "装备准备、户外技能与路线知识，登录后同步保存你的出行进度。"

    const val loginSectionTitle = "选择登录方式"
    const val passwordTab = "账号登录"
    const val verificationCodeTab = "验证码登录"
    const val passwordPrimaryAction = "登录"
    const val verificationCodePrimaryAction = "验证码登录"
    const val phoneRegisterMethod = "手机号注册"
    const val emailRegisterMethod = "邮箱注册"
    const val phoneResetMethod = "手机号找回"
    const val emailResetMethod = "邮箱找回"
    const val registerLink = "注册账号"
    const val forgotPasswordLink = "忘记密码？"
    const val backToLogin = "返回登录"
    const val sendCodeAction = "获取验证码"

    val loginTabModes = listOf(AuthMode.LOGIN, AuthMode.VERIFICATION_CODE)
    val loginTabLabels = mapOf(
        AuthMode.LOGIN to passwordTab,
        AuthMode.VERIFICATION_CODE to verificationCodeTab,
    )
}
