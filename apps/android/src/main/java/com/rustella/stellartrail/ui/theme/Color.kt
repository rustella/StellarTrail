package com.rustella.stellartrail.ui.theme

import androidx.compose.ui.graphics.Color

data class StellarTrailPalette(
    val pageBackground: Color,
    val surface: Color,
    val surfaceStrong: Color,
    val controlBackground: Color,
    val border: Color,
    val softBorder: Color,
    val textPrimary: Color,
    val textMuted: Color,
    val brand: Color,
    val brandText: Color,
    val brandSoft: Color,
    val brandSoftText: Color,
    val heroStart: Color,
    val heroEnd: Color,
    val successText: Color,
    val successBackground: Color,
    val warningText: Color,
    val warningBackground: Color,
    val dangerText: Color,
    val dangerBackground: Color,
    val infoText: Color,
    val infoBackground: Color,
)

object StellarTrailDesignColors {
    val Light = StellarTrailPalette(
        pageBackground = Color(0xFFF8FAFC),
        surface = Color(0xFFFFFFFF),
        surfaceStrong = Color(0xFFFFFFFF),
        controlBackground = Color(0xFFF1F5F9),
        border = Color(0xFFE2E8F0),
        softBorder = Color(0xFFF1F5F9),
        textPrimary = Color(0xFF0F172A),
        textMuted = Color(0xFF64748B),
        brand = Color(0xFF0F766E),
        brandText = Color(0xFFFFFFFF),
        brandSoft = Color(0xFFCCFBF1),
        brandSoftText = Color(0xFF0F766E),
        heroStart = Color(0xFF0F172A),
        heroEnd = Color(0xFF0F766E),
        successText = Color(0xFF047857),
        successBackground = Color(0xFFD1FAE5),
        warningText = Color(0xFFB45309),
        warningBackground = Color(0xFFFEF3C7),
        dangerText = Color(0xFFDC2626),
        dangerBackground = Color(0xFFFFF1F2),
        infoText = Color(0xFF2563EB),
        infoBackground = Color(0xFFEFF6FF),
    )

    val Dark = StellarTrailPalette(
        pageBackground = Color(0xFF07051A),
        surface = Color(0xE6171234),
        surfaceStrong = Color(0xFF17112F),
        controlBackground = Color(0xFF120D2C),
        border = Color(0xFF3D2D63),
        softBorder = Color(0xFF332555),
        textPrimary = Color(0xFFF6F1FF),
        textMuted = Color(0xFFC7B9F4),
        brand = Color(0xFFA78BFA),
        brandText = Color(0xFF12071F),
        brandSoft = Color(0xFF2A1F4F),
        brandSoftText = Color(0xFFEDE7FF),
        heroStart = Color(0xFF12082E),
        heroEnd = Color(0xFF0F766E),
        successText = Color(0xFFBBF7D0),
        successBackground = Color(0xFF123522),
        warningText = Color(0xFFFDE68A),
        warningBackground = Color(0xFF3B2A11),
        dangerText = Color(0xFFFECDD3),
        dangerBackground = Color(0xFF3B1520),
        infoText = Color(0xFFBFDBFE),
        infoBackground = Color(0xFF1A274D),
    )
}

val TrailPrimary = StellarTrailDesignColors.Light.brand
val TrailPrimaryDark = StellarTrailDesignColors.Dark.brand
val TrailSecondary = StellarTrailDesignColors.Light.infoText
val TrailSecondaryDark = StellarTrailDesignColors.Dark.infoText
val TrailTertiary = Color(0xFFF97316)
val TrailSurface = StellarTrailDesignColors.Light.pageBackground
val TrailSurfaceDark = StellarTrailDesignColors.Dark.pageBackground
