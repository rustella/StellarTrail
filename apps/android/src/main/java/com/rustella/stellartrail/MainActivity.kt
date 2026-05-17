package com.rustella.stellartrail

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.ui.StellarTrailApp
import com.rustella.stellartrail.ui.theme.StellarTrailTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val appContainer = (application as StellarTrailApplication).container
        setContent {
            StellarTrailRoot(appContainer)
        }
    }
}

@Composable
fun StellarTrailRoot(container: AppContainer) {
    val themeMode by container.themeRepository.theme.collectAsStateWithLifecycle()
    StellarTrailTheme(themeMode = themeMode) {
        StellarTrailApp(container = container)
    }
}
