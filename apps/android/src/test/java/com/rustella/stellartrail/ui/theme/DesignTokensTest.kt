package com.rustella.stellartrail.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import org.junit.Assert.assertEquals
import org.junit.Test

class DesignTokensTest {
    @Test
    fun lightPaletteMatchesWechatMiniProgramTokens() {
        assertColor(0xFFF8FAFC, StellarTrailDesignColors.Light.pageBackground)
        assertColor(0xFFFFFFFF, StellarTrailDesignColors.Light.surface)
        assertColor(0xFF0F172A, StellarTrailDesignColors.Light.textPrimary)
        assertColor(0xFF64748B, StellarTrailDesignColors.Light.textMuted)
        assertColor(0xFF0F766E, StellarTrailDesignColors.Light.brand)
        assertColor(0xFFCCFBF1, StellarTrailDesignColors.Light.brandSoft)
    }

    @Test
    fun heroGradientUsesWechatMiniProgramStops() {
        assertColor(0xFF0F172A, StellarTrailDesignColors.Light.heroStart)
        assertColor(0xFF0F766E, StellarTrailDesignColors.Light.heroEnd)
    }

    private fun assertColor(expectedArgb: Long, actual: Color) {
        assertEquals(expectedArgb.toInt(), actual.toArgb())
    }
}
