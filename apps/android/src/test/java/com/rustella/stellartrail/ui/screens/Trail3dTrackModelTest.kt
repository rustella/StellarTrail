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
    fun trackCameraSettersClampPitchAndNormalizeYaw() {
        val camera = Trail3dCamera(yawDegrees = 0.0, pitchDegrees = 55.0, zoom = 1.0)

        assertEquals(-170.0, setTrail3dCameraYaw(camera, 190.0).yawDegrees, 0.0)
        assertEquals(170.0, setTrail3dCameraYaw(camera, -190.0).yawDegrees, 0.0)
        assertEquals(TRAIL_3D_DEFAULT_YAW_DEGREES, setTrail3dCameraYaw(camera, Double.NaN).yawDegrees, 0.0)
        assertEquals(TRAIL_3D_MIN_PITCH_DEGREES, setTrail3dCameraPitch(camera, 0.0).pitchDegrees, 0.0)
        assertEquals(TRAIL_3D_MAX_PITCH_DEGREES, setTrail3dCameraPitch(camera, 90.0).pitchDegrees, 0.0)
    }

    @Test
    fun trackCameraUpdateNormalizesYawAndClampsPitchAndZoom() {
        val camera = updateTrail3dCamera(
            Trail3dCamera(yawDegrees = 170.0, pitchDegrees = 55.0, zoom = 1.0),
            yawDeltaDegrees = 30.0,
            pitchDeltaDegrees = 100.0,
            zoomMultiplier = 10.0,
        )

        assertEquals(-160.0, camera.yawDegrees, 0.0)
        assertEquals(TRAIL_3D_MAX_PITCH_DEGREES, camera.pitchDegrees, 0.0)
        assertEquals(TRAIL_3D_MAX_ZOOM, camera.zoom, 0.0)
    }

    @Test
    fun trackCameraZoomMultiplierStaysWithinLimit() {
        assertEquals(
            TRAIL_3D_MAX_ZOOM,
            zoomTrail3dCamera(
                Trail3dCamera(zoom = TRAIL_3D_MAX_ZOOM),
                10.0,
            ).zoom,
            0.0,
        )
        assertEquals(
            TRAIL_3D_MIN_ZOOM,
            zoomTrail3dCamera(
                Trail3dCamera(zoom = TRAIL_3D_MIN_ZOOM),
                0.1,
            ).zoom,
            0.0,
        )
    }

    @Test
    fun trackCameraDoubleTapZoomDoesNotExceedMaxZoom() {
        val camera = generateSequence(resetTrail3dCamera()) { current ->
            zoomTrail3dCamera(current, TRAIL_3D_DOUBLE_TAP_ZOOM_MULTIPLIER)
        }.drop(12).first()

        assertEquals(TRAIL_3D_MAX_ZOOM, camera.zoom, 0.0)
    }

    @Test
    fun trackCameraMapGestureUsesRotationForYawAndVerticalPanForPitch() {
        val camera = Trail3dCamera(yawDegrees = 0.0, pitchDegrees = 55.0, zoom = 1.0)

        val rotated = updateTrail3dCameraFromMapGesture(
            camera,
            rotationDeltaDegrees = 15.0,
            pitchPanDeltaYPx = 0.0,
            zoomMultiplier = 1.0,
        )
        val pitched = updateTrail3dCameraFromMapGesture(
            camera,
            rotationDeltaDegrees = 0.0,
            pitchPanDeltaYPx = -20.0,
            zoomMultiplier = 1.0,
        )
        val zoomed = updateTrail3dCameraFromMapGesture(
            Trail3dCamera(zoom = TRAIL_3D_MAX_ZOOM),
            rotationDeltaDegrees = 0.0,
            pitchPanDeltaYPx = 0.0,
            zoomMultiplier = 10.0,
        )

        assertEquals(15.0 * TRAIL_3D_ROTATION_GESTURE_YAW_FACTOR, rotated.yawDegrees, 0.0)
        assertEquals(55.0, rotated.pitchDegrees, 0.0)
        assertEquals(0.0, pitched.yawDegrees, 0.0)
        assertEquals(55.0 + 20.0 * TRAIL_3D_PITCH_GESTURE_PX_FACTOR, pitched.pitchDegrees, 0.0)
        assertEquals(TRAIL_3D_MAX_ZOOM, zoomed.zoom, 0.0)
    }

    @Test
    fun trackCameraPanMovesProjectionAndCanReset() {
        val model = buildTrail3dTrackModel(
            listOf(
                TrailPoint(lng = 114.0, lat = 22.0, elevationM = 100.0),
                TrailPoint(lng = 114.001, lat = 22.001, elevationM = 180.0),
                TrailPoint(lng = 114.002, lat = 22.0, elevationM = 140.0),
            ),
        )
        val baseProjection = projectTrail3dScene(model, resetTrail3dCamera(), viewportWidthPx = 480f, viewportHeightPx = 320f)
        val pannedCamera = panTrail3dCamera(resetTrail3dCamera(), panDeltaXPx = 24.0, panDeltaYPx = -18.0)
        val pannedProjection = projectTrail3dScene(model, pannedCamera, viewportWidthPx = 480f, viewportHeightPx = 320f)

        assertEquals(24.0, pannedCamera.panXPx, 0.0)
        assertEquals(-18.0, pannedCamera.panYPx, 0.0)
        assertNotNull(baseProjection)
        assertNotNull(pannedProjection)
        baseProjection ?: return
        pannedProjection ?: return
        assertEquals(baseProjection.trackPoints.first().x.toDouble() + 24.0, pannedProjection.trackPoints.first().x.toDouble(), 0.01)
        assertEquals(baseProjection.trackPoints.first().y.toDouble() - 18.0, pannedProjection.trackPoints.first().y.toDouble(), 0.01)
        assertEquals(0.0, resetTrail3dCamera().panXPx, 0.0)
        assertEquals(0.0, resetTrail3dCamera().panYPx, 0.0)
    }

    @Test
    fun trackCameraCanReset() {
        assertEquals(Trail3dCamera(), resetTrail3dCamera())
    }

    @Test
    fun trackGestureGuideDescribesMapAlignedControls() {
        val lines = trail3dGestureGuideLines()

        assertTrue(lines.contains("单指拖动移动模型"))
        assertTrue(lines.contains("双指捏合缩放"))
        assertTrue(lines.contains("双指旋转方向"))
        assertTrue(lines.contains("双指上下拖动调整俯仰"))
        assertTrue(lines.contains("双击放大"))
        assertTrue(lines.contains("点按轨迹查看点位"))
    }
}
