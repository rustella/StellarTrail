package com.rustella.stellartrail

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.screenshot.ScreenshotFixtures
import com.rustella.stellartrail.ui.StellarTrailApp
import com.rustella.stellartrail.ui.theme.StellarTrailTheme
import com.rustella.stellartrail.ui.theme.shouldUseLightSystemBars

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val fixture = ScreenshotFixtures.createContainer(this, intent)
        val appContainer = fixture?.container ?: (application as StellarTrailApplication).container
        val startDestination = fixture?.startDestination ?: ScreenshotFixtures.startDestination(intent)
        setContent {
            StellarTrailRoot(appContainer, startDestination)
        }
    }
}

@Composable
fun StellarTrailRoot(container: AppContainer, startDestination: String = "home") {
    val themeMode by container.themeRepository.theme.collectAsStateWithLifecycle()
    StellarTrailTheme(themeMode = themeMode) {
        SyncSystemBars(themeMode)
        StellarTrailApp(container = container, startDestination = startDestination)
    }
}

@Composable
private fun SyncSystemBars(themeMode: ThemeMode) {
    val view = LocalView.current
    val lightSystemBars = shouldUseLightSystemBars(themeMode, isSystemInDarkTheme())
    DisposableEffect(view, lightSystemBars) {
        val activity = view.context.findActivity()
        if (activity != null) {
            val controller = WindowInsetsControllerCompat(activity.window, view)
            controller.isAppearanceLightStatusBars = lightSystemBars
            controller.isAppearanceLightNavigationBars = lightSystemBars
        }
        onDispose { }
    }
}

private tailrec fun Context.findActivity(): Activity? = when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.findActivity()
    else -> null
}
