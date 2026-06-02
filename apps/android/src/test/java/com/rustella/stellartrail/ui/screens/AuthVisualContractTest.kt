package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.feature.auth.AuthMode
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class AuthVisualContractTest {
    @Test
    fun loginTabsOnlyContainPrimaryLoginMethods() {
        assertEquals(listOf(AuthMode.LOGIN, AuthMode.PHONE_CODE, AuthMode.EMAIL_CODE), AuthVisualContract.loginTabModes)
        assertEquals("账号登录", AuthVisualContract.loginTabLabels.getValue(AuthMode.LOGIN))
        assertEquals("手机登录", AuthVisualContract.loginTabLabels.getValue(AuthMode.PHONE_CODE))
        assertEquals("邮箱登录", AuthVisualContract.loginTabLabels.getValue(AuthMode.EMAIL_CODE))
        assertEquals("手机号注册", AuthVisualContract.phoneRegisterMethod)
        assertEquals("邮箱注册", AuthVisualContract.emailRegisterMethod)
        assertEquals("手机号找回", AuthVisualContract.phoneResetMethod)
        assertEquals("邮箱找回", AuthVisualContract.emailResetMethod)
        assertFalse(AuthVisualContract.loginTabLabels.containsKey(AuthMode.REGISTER))
        assertFalse(AuthVisualContract.loginTabLabels.containsKey(AuthMode.RESET_PASSWORD))
    }

    @Test
    fun secondaryActionsAreLowEmphasisLinksNotPrimaryTabs() {
        assertEquals("注册账号", AuthVisualContract.registerLink)
        assertEquals("忘记密码？", AuthVisualContract.forgotPasswordLink)
        assertEquals("返回登录", AuthVisualContract.backToLogin)
        assertEquals("获取验证码", AuthVisualContract.sendCodeAction)
    }
}
