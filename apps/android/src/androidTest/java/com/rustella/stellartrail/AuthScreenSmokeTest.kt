package com.rustella.stellartrail

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AuthScreenSmokeTest {
    @get:Rule
    val composeRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun unauthenticatedUserSeesAuthScreen() {
        composeRule.onNodeWithText("寻径星野").assertIsDisplayed()
        composeRule.onNodeWithText("登录").assertIsDisplayed()
        composeRule.onNodeWithText("注册").assertIsDisplayed()
    }
}
