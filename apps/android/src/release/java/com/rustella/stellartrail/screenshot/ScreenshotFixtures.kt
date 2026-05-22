package com.rustella.stellartrail.screenshot

import android.content.Context
import android.content.Intent
import com.rustella.stellartrail.di.AppContainer

object ScreenshotFixtures {
    data class FixtureLaunch(
        val container: AppContainer,
        val startDestination: String,
    )

    fun createContainer(context: Context, intent: Intent): FixtureLaunch? = null
    fun startDestination(intent: Intent): String = "home"
}
