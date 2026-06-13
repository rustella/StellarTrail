package com.rustella.stellartrail

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.core.trail.TrailImportIntentResult
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.screenshot.ScreenshotFixtures
import com.rustella.stellartrail.ui.StellarTrailApp
import com.rustella.stellartrail.ui.navigation.AppRoutes
import com.rustella.stellartrail.ui.theme.StellarTrailTheme
import com.rustella.stellartrail.ui.theme.shouldUseLightSystemBars

class MainActivity : ComponentActivity() {
    private lateinit var appContainer: AppContainer
    private var pendingTrailImportRoute by mutableStateOf<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val fixture = ScreenshotFixtures.createContainer(this, intent)
        appContainer = fixture?.container ?: (application as StellarTrailApplication).container
        val startDestination = fixture?.startDestination ?: ScreenshotFixtures.startDestination(intent)
        if (fixture == null) handleTrailImportIntent(intent)
        setContent {
            StellarTrailRoot(
                container = appContainer,
                startDestination = startDestination,
                pendingTrailImportRoute = pendingTrailImportRoute,
                onPendingTrailImportConsumed = { pendingTrailImportRoute = null },
            )
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        if (::appContainer.isInitialized) handleTrailImportIntent(intent)
    }

    private fun handleTrailImportIntent(intent: Intent) {
        when (val result = appContainer.pendingTrailImportStore.createFromIntent(intent)) {
            is TrailImportIntentResult.Created -> pendingTrailImportRoute = AppRoutes.trailImport(result.pending.id)
            TrailImportIntentResult.Unsupported -> {
                Toast.makeText(this, R.string.unsupported_trail_import, Toast.LENGTH_SHORT).show()
            }
            TrailImportIntentResult.Ignored -> Unit
        }
    }
}

@Composable
fun StellarTrailRoot(
    container: AppContainer,
    startDestination: String = "home",
    pendingTrailImportRoute: String? = null,
    onPendingTrailImportConsumed: () -> Unit = {},
) {
    val themeMode by container.themeRepository.theme.collectAsStateWithLifecycle()
    StellarTrailTheme(themeMode = themeMode) {
        SyncSystemBars(themeMode)
        StellarTrailApp(
            container = container,
            startDestination = startDestination,
            pendingTrailImportRoute = pendingTrailImportRoute,
            onPendingTrailImportConsumed = onPendingTrailImportConsumed,
        )
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
