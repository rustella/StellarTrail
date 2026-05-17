package com.rustella.stellartrail.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class DesignTokensTest {
    @Test
    fun lightPaletteMatchesWechatMiniProgramTokens() {
        assertColor(0xFFF8FAFC, StellarTrailDesignColors.Light.pageBackground)
        assertColor(0xFFFFFFFF, StellarTrailDesignColors.Light.surface)
        assertColor(0xFF0F172A, StellarTrailDesignColors.Light.textPrimary)
        assertColor(0xFF64748B, StellarTrailDesignColors.Light.textMuted)
        assertColor(0xFF334155, StellarTrailDesignColors.Light.headingMuted)
        assertColor(0xFF0F766E, StellarTrailDesignColors.Light.accent)
        assertColor(0xFF0F766E, StellarTrailDesignColors.Light.brand)
        assertColor(0xFFCCFBF1, StellarTrailDesignColors.Light.brandSoft)
        assertColor(0xFFE2E8F0, StellarTrailDesignColors.Light.softControlBackground)
        assertColor(0xFF475569, StellarTrailDesignColors.Light.softControlText)
        assertColor(0xFFECFDF5, StellarTrailDesignColors.Light.chipBackground)
    }

    @Test
    fun darkPaletteMatchesWechatMiniProgramTokens() {
        assertColor(0xFF07051A, StellarTrailDesignColors.Dark.pageBackground)
        assertColor(0xE6181234, StellarTrailDesignColors.Dark.surface)
        assertColor(0xFF17112F, StellarTrailDesignColors.Dark.surfaceStrong)
        assertColor(0xFF120D2C, StellarTrailDesignColors.Dark.controlBackground)
        assertColor(0xFF3D2D63, StellarTrailDesignColors.Dark.border)
        assertColor(0xFF332555, StellarTrailDesignColors.Dark.softBorder)
        assertColor(0xFFF6F1FF, StellarTrailDesignColors.Dark.textPrimary)
        assertColor(0xFFC7B9F4, StellarTrailDesignColors.Dark.textMuted)
        assertColor(0xFFDDD6FE, StellarTrailDesignColors.Dark.headingMuted)
        assertColor(0xFFE879F9, StellarTrailDesignColors.Dark.accent)
        assertColor(0xFFA78BFA, StellarTrailDesignColors.Dark.brand)
        assertColor(0xFF2A1F4F, StellarTrailDesignColors.Dark.brandSoft)
        assertColor(0xFF2A1F4F, StellarTrailDesignColors.Dark.chipBackground)
        assertColor(0xFFFDE68A, StellarTrailDesignColors.Dark.warningText)
        assertColor(0xFF3B2A11, StellarTrailDesignColors.Dark.warningBackground)
    }

    @Test
    fun mobileChromeTokensMatchWechatMiniProgram() {
        assertColor(0xFFF8FAFC, StellarTrailDesignColors.Light.topBarBackground)
        assertColor(0xFFFFFFFF, StellarTrailDesignColors.Light.footerBackground)
        assertColor(0xFF12082E, StellarTrailDesignColors.Dark.topBarBackground)
        assertColor(0xEF0E0A22, StellarTrailDesignColors.Dark.footerBackground)
    }

    @Test
    fun lightHeroUsesWechatDayModeIllustrationColors() {
        assertNotEquals(
            "Light mode hero should not keep the old dark navy start color.",
            0xFF0F172A.toInt(),
            StellarTrailDesignColors.Light.heroStart.toArgb(),
        )
        assertColor(0xFFFFF7ED, StellarTrailDesignColors.Light.heroStart)
        assertColor(0xFFECFEFF, StellarTrailDesignColors.Light.heroMid)
        assertColor(0xFFEEF2FF, StellarTrailDesignColors.Light.heroEnd)
        assertColor(0xFFD8F1F6, StellarTrailDesignColors.Light.heroHill)
        assertColor(0xFFFBBF24, StellarTrailDesignColors.Light.heroSun)
    }

    @Test
    fun darkHeroKeepsHighContrastOutdoorGradient() {
        assertColor(0xFF12082E, StellarTrailDesignColors.Dark.heroStart)
        assertColor(0xFF1A274D, StellarTrailDesignColors.Dark.heroMid)
        assertColor(0xFF0F766E, StellarTrailDesignColors.Dark.heroEnd)
        assertColor(0xFF1F3F4A, StellarTrailDesignColors.Dark.heroHill)
        assertColor(0xFFFDE68A, StellarTrailDesignColors.Dark.heroSun)
    }

    private fun assertColor(expectedArgb: Long, actual: Color) {
        assertEquals(expectedArgb.toInt(), actual.toArgb())
    }
}
