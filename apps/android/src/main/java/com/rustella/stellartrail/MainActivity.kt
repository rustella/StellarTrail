package com.rustella.stellartrail

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.screenshot.ScreenshotFixtures
import com.rustella.stellartrail.ui.StellarTrailApp
import com.rustella.stellartrail.ui.theme.StellarTrailTheme

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
        StellarTrailApp(container = container, startDestination = startDestination)
    }
}
