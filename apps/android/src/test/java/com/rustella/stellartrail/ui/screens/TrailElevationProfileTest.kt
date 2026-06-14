package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.trip.TrailPoint
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TrailElevationProfileTest {
    @Test
    fun profileAccumulatesDistanceAndKeepsElevationPoints() {
        val profile = buildTrailElevationProfile(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = 100.0),
                TrailPoint(lng = 114.001, lat = 22.0, elevationM = 120.0),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = 110.0),
            ),
        )

        assertTrue(profile.hasEnoughData)
        assertEquals(3, profile.points.size)
        assertEquals(100.0, profile.minElevationM ?: -1.0, 0.0)
        assertEquals(120.0, profile.maxElevationM ?: -1.0, 0.0)
        assertTrue(profile.points.last().distanceM > profile.points[1].distanceM)
    }

    @Test
    fun profileSkipsMissingElevationButKeepsDistanceAxis() {
        val profile = buildTrailElevationProfile(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = 100.0),
                TrailPoint(lng = 114.001, lat = 22.0, elevationM = null),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = 130.0),
            ),
        )

        assertTrue(profile.hasEnoughData)
        assertEquals(2, profile.points.size)
        assertEquals(0.0, profile.points.first().distanceM, 0.0)
        assertTrue(profile.points.last().distanceM > 180.0)
    }

    @Test
    fun profileRequiresAtLeastTwoElevationPoints() {
        val profile = buildTrailElevationProfile(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = null),
                TrailPoint(lng = 114.001, lat = 22.0, elevationM = 120.0),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = null),
            ),
        )

        assertFalse(profile.hasEnoughData)
        assertEquals(1, profile.points.size)
    }

    @Test
    fun profileDownsamplingKeepsEndpointsWithinLimit() {
        val points = (0 until 100).map { index ->
            TrailPoint(lng = 114.0 + index * 0.001, lat = 22.0, elevationM = 100.0 + index)
        }

        val profile = buildTrailElevationProfile(points, maxRenderedPoints = 12)

        assertTrue(profile.hasEnoughData)
        assertTrue(profile.points.size <= 12)
        assertEquals(0, profile.points.first().pointIndex)
        assertEquals(99, profile.points.last().pointIndex)
    }
}
