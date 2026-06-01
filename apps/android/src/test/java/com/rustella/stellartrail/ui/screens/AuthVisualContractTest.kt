package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.feature.auth.AuthMode
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class AuthVisualContractTest {
    @Test
    fun loginTabsOnlyContainPrimaryLoginMethods() {
        assertEquals(listOf(AuthMode.LOGIN, AuthMode.EMAIL_CODE), AuthVisualContract.loginTabModes)
        assertEquals("密码登录", AuthVisualContract.loginTabLabels.getValue(AuthMode.LOGIN))
        assertEquals("验证码登录", AuthVisualContract.loginTabLabels.getValue(AuthMode.EMAIL_CODE))
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
