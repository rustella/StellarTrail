package com.rustella.stellartrail.ui.common

data class HeroStar(
    val xPercent: Float,
    val yPercent: Float,
    val radiusDp: Float,
    val alpha: Float,
    val accent: Boolean = false,
)

object HeroVisualContract {
    const val contentPaddingDp = 20
    const val actionRowTopGapDp = 12
    const val actionBottomSafeGapDp = 20
    const val followingSectionGapDp = 16

    val nightStars: List<HeroStar> = listOf(
        HeroStar(xPercent = 0.62f, yPercent = 0.18f, radiusDp = 1.7f, alpha = 0.86f),
        HeroStar(xPercent = 0.73f, yPercent = 0.11f, radiusDp = 1.1f, alpha = 0.62f),
        HeroStar(xPercent = 0.80f, yPercent = 0.26f, radiusDp = 1.4f, alpha = 0.78f),
        HeroStar(xPercent = 0.90f, yPercent = 0.14f, radiusDp = 1.2f, alpha = 0.55f),
        HeroStar(xPercent = 0.87f, yPercent = 0.39f, radiusDp = 1.5f, alpha = 0.76f, accent = true),
        HeroStar(xPercent = 0.70f, yPercent = 0.48f, radiusDp = 1.0f, alpha = 0.48f),
        HeroStar(xPercent = 0.93f, yPercent = 0.55f, radiusDp = 1.2f, alpha = 0.46f),
    )
}
