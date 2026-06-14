package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.trip.TrailPoint
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.abs

class Trail3dTrackModelTest {
    @Test
    fun trackModelConvertsLngLatToLocalMeters() {
        val model = buildTrail3dTrackModel(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = 100.0),
                TrailPoint(lng = 114.001, lat = 22.0, elevationM = 120.0),
                TrailPoint(lng = 114.001, lat = 22.001, elevationM = 130.0),
            ),
        )

        assertTrue(model.hasEnoughData)
        assertTrue(abs(model.points[1].eastM - model.points[0].eastM) in 100.0..110.0)
        assertTrue(abs(model.points[2].northM - model.points[1].northM) in 110.0..112.0)
    }

    @Test
    fun trackModelAccumulatesDistanceAndKeepsElevationReadings() {
        val model = buildTrail3dTrackModel(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = 100.0),
                TrailPoint(lng = 114.001, lat = 22.0, elevationM = 120.0),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = 110.0),
            ),
        )

        assertTrue(model.hasEnoughData)
        assertEquals(3, model.points.size)
        assertEquals(100.0, model.minElevationM ?: -1.0, 0.0)
        assertEquals(120.0, model.maxElevationM ?: -1.0, 0.0)
        assertTrue(model.distanceM > 200.0)
        assertEquals(110.0, model.points.last().elevationM, 0.0)
    }

    @Test
    fun trackModelSkipsInvalidCoordinatesAndMissingElevation() {
        val model = buildTrail3dTrackModel(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = null),
                TrailPoint(lng = 114.001, lat = 22.0, elevationM = 120.0),
                TrailPoint(lng = 999.0, lat = 22.0, elevationM = 130.0),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = null),
            ),
        )

        assertFalse(model.hasEnoughData)
        assertEquals(1, model.points.size)
        assertEquals(1, model.points.single().pointIndex)
    }

    @Test
    fun trackModelDownsamplingKeepsEndpointsWithinLimit() {
        val points = (0 until 100).map { index ->
            TrailPoint(lng = 114.0 + index * 0.001, lat = 22.0, elevationM = 100.0 + index)
        }

        val model = buildTrail3dTrackModel(points, maxRenderedPoints = 12)

        assertTrue(model.hasEnoughData)
        assertTrue(model.points.size <= 12)
        assertEquals(0, model.points.first().pointIndex)
        assertEquals(99, model.points.last().pointIndex)
    }

    @Test
    fun trackProjectionProducesFinitePointsInsideViewport() {
        val model = buildTrail3dTrackModel(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = 100.0),
                TrailPoint(lng = 114.001, lat = 22.001, elevationM = 180.0),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = 140.0),
            ),
        )

        val projection = projectTrail3dScene(model, resetTrail3dCamera(), viewportWidthPx = 480f, viewportHeightPx = 320f)

        assertNotNull(projection)
        projection ?: return
        assertEquals(model.points.size, projection.trackPoints.size)
        projection.trackPoints.forEach { point ->
            assertTrue(point.x.isFinite())
            assertTrue(point.y.isFinite())
            assertTrue(point.x in 0f..480f)
            assertTrue(point.y in 0f..320f)
        }
        assertTrue(projection.gridLines.isNotEmpty())
        assertEquals(model.points.first().pointIndex, projection.trackPoints.first().source?.pointIndex)
    }

    @Test
    fun trackCameraClampsPitchAndZoomAndCanReset() {
        val camera = updateTrail3dCamera(
            Trail3dCamera(yawDegrees = 10.0, pitchDegrees = 55.0, zoom = 1.0),
            yawDeltaDegrees = 20.0,
            pitchDeltaDegrees = 100.0,
            zoomMultiplier = 10.0,
        )

        assertEquals(30.0, camera.yawDegrees, 0.0)
        assertEquals(TRAIL_3D_MAX_PITCH_DEGREES, camera.pitchDegrees, 0.0)
        assertEquals(TRAIL_3D_MAX_ZOOM, camera.zoom, 0.0)
        assertEquals(Trail3dCamera(), resetTrail3dCamera())
    }
}
