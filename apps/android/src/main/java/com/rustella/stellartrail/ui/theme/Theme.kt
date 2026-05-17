package com.rustella.stellartrail.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import com.rustella.stellartrail.core.theme.ThemeMode

private val LightPalette = StellarTrailDesignColors.Light
private val DarkPalette = StellarTrailDesignColors.Dark

private val LightColors = lightColorScheme(
    primary = LightPalette.brand,
    onPrimary = LightPalette.brandText,
    primaryContainer = LightPalette.brandSoft,
    onPrimaryContainer = LightPalette.brandSoftText,
    secondary = LightPalette.infoText,
    onSecondary = Color.White,
    secondaryContainer = LightPalette.infoBackground,
    onSecondaryContainer = LightPalette.infoText,
    tertiary = TrailTertiary,
    background = LightPalette.pageBackground,
    onBackground = LightPalette.textPrimary,
    surface = LightPalette.surface,
    onSurface = LightPalette.textPrimary,
    surfaceVariant = LightPalette.controlBackground,
    onSurfaceVariant = LightPalette.textMuted,
    outline = LightPalette.border,
    outlineVariant = LightPalette.softBorder,
    error = LightPalette.dangerText,
    errorContainer = LightPalette.dangerBackground,
    onErrorContainer = Color(0xFFB91C1C),
)

private val DarkColors = darkColorScheme(
    primary = DarkPalette.brand,
    onPrimary = DarkPalette.brandText,
    primaryContainer = DarkPalette.brandSoft,
    onPrimaryContainer = DarkPalette.brandSoftText,
    secondary = DarkPalette.infoText,
    onSecondary = Color(0xFF0F172A),
    secondaryContainer = DarkPalette.infoBackground,
    onSecondaryContainer = DarkPalette.infoText,
    tertiary = Color(0xFFFBBF24),
    background = DarkPalette.pageBackground,
    onBackground = DarkPalette.textPrimary,
    surface = DarkPalette.surface,
    onSurface = DarkPalette.textPrimary,
    surfaceVariant = DarkPalette.controlBackground,
    onSurfaceVariant = DarkPalette.textMuted,
    outline = DarkPalette.border,
    outlineVariant = DarkPalette.softBorder,
    error = DarkPalette.dangerText,
    errorContainer = DarkPalette.dangerBackground,
    onErrorContainer = DarkPalette.dangerText,
)

@Composable
fun StellarTrailTheme(
    themeMode: ThemeMode,
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit,
) {
    val darkTheme = when (themeMode) {
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
        ThemeMode.SYSTEM -> isSystemInDarkTheme()
    }
    val colorScheme = if (dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val context = LocalContext.current
        if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
    } else if (darkTheme) {
        DarkColors
    } else {
        LightColors
    }
    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content,
    )
}
