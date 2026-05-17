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
        assertColor(0xFF120A2A, StellarTrailDesignColors.Dark.pageBackground)
        assertColor(0xFF1A1035, StellarTrailDesignColors.Dark.surface)
        assertColor(0xFF211343, StellarTrailDesignColors.Dark.surfaceStrong)
        assertColor(0xFF130B2A, StellarTrailDesignColors.Dark.controlBackground)
        assertColor(0xFF3A2465, StellarTrailDesignColors.Dark.border)
        assertColor(0xFF2D1F4D, StellarTrailDesignColors.Dark.softBorder)
        assertColor(0xFFF4EFFF, StellarTrailDesignColors.Dark.textPrimary)
        assertColor(0xFFC7B7EB, StellarTrailDesignColors.Dark.textMuted)
        assertColor(0xFFEDE7FF, StellarTrailDesignColors.Dark.headingMuted)
        assertColor(0xFFC16CFF, StellarTrailDesignColors.Dark.accent)
        assertColor(0xFFA985FF, StellarTrailDesignColors.Dark.brand)
        assertColor(0xFF2B1854, StellarTrailDesignColors.Dark.brandSoft)
        assertColor(0xFF2B1854, StellarTrailDesignColors.Dark.chipBackground)
        assertColor(0xFFFFD66B, StellarTrailDesignColors.Dark.warningText)
        assertColor(0xFF4B3608, StellarTrailDesignColors.Dark.warningBackground)
    }

    @Test
    fun mobileChromeTokensMatchWechatMiniProgram() {
        assertColor(0xFFF8FAFC, StellarTrailDesignColors.Light.topBarBackground)
        assertColor(0xFFFFFFFF, StellarTrailDesignColors.Light.footerBackground)
        assertColor(0xFF120A2A, StellarTrailDesignColors.Dark.topBarBackground)
        assertColor(0xFF100823, StellarTrailDesignColors.Dark.footerBackground)
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
    fun darkHeroUsesWechatNightModeStarCardColors() {
        assertColor(0xFF4C2C9A, StellarTrailDesignColors.Dark.heroStart)
        assertColor(0xFF3156B8, StellarTrailDesignColors.Dark.heroMid)
        assertColor(0xFF167C7B, StellarTrailDesignColors.Dark.heroEnd)
        assertColor(0xFF283978, StellarTrailDesignColors.Dark.heroHill)
        assertColor(0xFFFFCC4D, StellarTrailDesignColors.Dark.heroSun)
        assertColor(0xFFEFEAFF, StellarTrailDesignColors.Dark.heroStar)
        assertColor(0xFFFFCC4D, StellarTrailDesignColors.Dark.heroStarAccent)
    }

    private fun assertColor(expectedArgb: Long, actual: Color) {
        assertEquals(expectedArgb.toInt(), actual.toArgb())
    }
}
