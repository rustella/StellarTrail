package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.trip.TrailPoint
import kotlin.math.PI
import kotlin.math.asin
import kotlin.math.cos
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.math.sqrt

internal const val TRAIL_ELEVATION_PROFILE_MAX_POINTS = 240

internal data class TrailElevationProfilePoint(
    val distanceM: Double,
    val elevationM: Double,
    val lng: Double,
    val lat: Double,
    val pointIndex: Int,
)

internal data class TrailElevationProfile(
    val points: List<TrailElevationProfilePoint>,
    val minElevationM: Double?,
    val maxElevationM: Double?,
    val distanceM: Double,
    val hasEnoughData: Boolean,
)

internal fun buildTrailElevationProfile(
    points: List<TrailPoint>,
    maxRenderedPoints: Int = TRAIL_ELEVATION_PROFILE_MAX_POINTS,
): TrailElevationProfile {
    var distanceM = 0.0
    var previousValidPoint: TrailPoint? = null
    val profilePoints = mutableListOf<TrailElevationProfilePoint>()

    points.forEachIndexed { index, point ->
        if (!point.hasValidCoordinate()) return@forEachIndexed
        previousValidPoint?.let { previous -> distanceM += haversineDistanceM(previous, point) }
        previousValidPoint = point
        val elevation = point.elevationM?.takeIf { it.isFinite() } ?: return@forEachIndexed
        profilePoints += TrailElevationProfilePoint(
            distanceM = distanceM,
            elevationM = elevation,
            lng = point.lng,
            lat = point.lat,
            pointIndex = index,
        )
    }

    val sampledPoints = evenlySampleProfilePoints(profilePoints, maxRenderedPoints)
    return TrailElevationProfile(
        points = sampledPoints,
        minElevationM = profilePoints.minOfOrNull { it.elevationM },
        maxElevationM = profilePoints.maxOfOrNull { it.elevationM },
        distanceM = distanceM,
        hasEnoughData = profilePoints.size >= 2 && sampledPoints.size >= 2,
    )
}

internal fun evenlySampleProfilePoints(
    points: List<TrailElevationProfilePoint>,
    maxRenderedPoints: Int,
): List<TrailElevationProfilePoint> {
    if (maxRenderedPoints <= 0) return emptyList()
    if (points.size <= maxRenderedPoints) return points
    if (maxRenderedPoints == 1) return listOf(points.first())
    val targetCount = maxRenderedPoints
    val lastIndex = points.lastIndex
    val indices = LinkedHashSet<Int>()
    (0 until targetCount).forEach { index ->
        indices += ((index * lastIndex).toDouble() / (targetCount - 1)).roundToInt().coerceIn(0, lastIndex)
    }
    indices += lastIndex
    return indices.sorted().map(points::get)
}

private fun TrailPoint.hasValidCoordinate(): Boolean =
    lng.isFinite() && lat.isFinite() && lng in -180.0..180.0 && lat in -90.0..90.0

private fun haversineDistanceM(a: TrailPoint, b: TrailPoint): Double {
    val lat1 = a.lat.toRadians()
    val lat2 = b.lat.toRadians()
    val deltaLat = (b.lat - a.lat).toRadians()
    val deltaLng = (b.lng - a.lng).toRadians()
    val h = sin(deltaLat / 2).pow(2.0) + cos(lat1) * cos(lat2) * sin(deltaLng / 2).pow(2.0)
    return 2.0 * EARTH_RADIUS_M * asin(sqrt(h.coerceIn(0.0, 1.0)))
}

private fun Double.toRadians(): Double = this * PI / 180.0

private const val EARTH_RADIUS_M = 6_371_000.0
