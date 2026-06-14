package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.domain.trip.TrailPoint
import kotlin.math.PI
import kotlin.math.asin
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.roundToInt
import kotlin.math.sin
import kotlin.math.sqrt

internal const val TRAIL_3D_TRACK_MAX_POINTS = 240
internal const val TRAIL_3D_DEFAULT_YAW_DEGREES = -35.0
internal const val TRAIL_3D_DEFAULT_PITCH_DEGREES = 55.0
internal const val TRAIL_3D_DEFAULT_ZOOM = 1.0
internal const val TRAIL_3D_MIN_YAW_DEGREES = -180.0
internal const val TRAIL_3D_MAX_YAW_DEGREES = 180.0
internal const val TRAIL_3D_MIN_PITCH_DEGREES = 20.0
internal const val TRAIL_3D_MAX_PITCH_DEGREES = 78.0
internal const val TRAIL_3D_MIN_ZOOM = 0.65
internal const val TRAIL_3D_MAX_ZOOM = 2.4
internal const val TRAIL_3D_DOUBLE_TAP_ZOOM_MULTIPLIER = 1.35
internal const val TRAIL_3D_ROTATION_GESTURE_YAW_FACTOR = 0.8
internal const val TRAIL_3D_PITCH_GESTURE_PX_FACTOR = 0.22
internal const val TRAIL_3D_MAX_VERTICAL_EXAGGERATION = 3.0

internal data class Trail3dTrackPoint(
    val eastM: Double,
    val northM: Double,
    val elevationM: Double,
    val distanceM: Double,
    val lng: Double,
    val lat: Double,
    val pointIndex: Int,
)

internal data class Trail3dTrackModel(
    val points: List<Trail3dTrackPoint>,
    val minElevationM: Double?,
    val maxElevationM: Double?,
    val distanceM: Double,
    val eastMinM: Double,
    val eastMaxM: Double,
    val northMinM: Double,
    val northMaxM: Double,
    val horizontalSpanM: Double,
    val hasEnoughData: Boolean,
)

internal data class Trail3dCamera(
    val yawDegrees: Double = TRAIL_3D_DEFAULT_YAW_DEGREES,
    val pitchDegrees: Double = TRAIL_3D_DEFAULT_PITCH_DEGREES,
    val zoom: Double = TRAIL_3D_DEFAULT_ZOOM,
    val panXPx: Double = 0.0,
    val panYPx: Double = 0.0,
)

internal data class Trail3dProjectedPoint(
    val x: Float,
    val y: Float,
    val depth: Double,
    val source: Trail3dTrackPoint? = null,
)

internal data class Trail3dProjectedLine(
    val start: Trail3dProjectedPoint,
    val end: Trail3dProjectedPoint,
) {
    val depth: Double = (start.depth + end.depth) / 2.0
}

internal data class Trail3dProjection(
    val trackPoints: List<Trail3dProjectedPoint>,
    val shadowPoints: List<Trail3dProjectedPoint>,
    val groundOutline: List<Trail3dProjectedPoint>,
    val gridLines: List<Trail3dProjectedLine>,
)

private data class Raw3dPoint(
    val x: Double,
    val y: Double,
    val depth: Double,
    val source: Trail3dTrackPoint? = null,
)

private data class Trail3dPositionedPoint(
    val pointIndex: Int,
    val point: TrailPoint,
    val distanceM: Double,
)

internal fun buildTrail3dTrackModel(
    points: List<TrailPoint>,
    maxRenderedPoints: Int = TRAIL_3D_TRACK_MAX_POINTS,
): Trail3dTrackModel {
    val positionedPoints = points.positionedValidCoordinatePoints()
    if (positionedPoints.isEmpty()) return emptyTrail3dTrackModel()

    val centerLat = positionedPoints.map { it.point.lat }.average()
    val centerLng = positionedPoints.map { it.point.lng }.average()
    val centerLatRadians = centerLat.toRadians()
    val allEastMeters = positionedPoints.map { positioned ->
        (positioned.point.lng - centerLng).toRadians() * EARTH_RADIUS_M * cos(centerLatRadians)
    }
    val allNorthMeters = positionedPoints.map { positioned ->
        (positioned.point.lat - centerLat).toRadians() * EARTH_RADIUS_M
    }
    val trackPoints = positionedPoints.mapNotNull { positioned ->
        val point = positioned.point
        val elevationM = point.elevationM.takeIf { it?.isFinite() == true }
            ?: positioned.interpolateElevationM(positionedPoints)
            ?: return@mapNotNull null
        Trail3dTrackPoint(
            eastM = (point.lng - centerLng).toRadians() * EARTH_RADIUS_M * cos(centerLatRadians),
            northM = (point.lat - centerLat).toRadians() * EARTH_RADIUS_M,
            elevationM = elevationM,
            distanceM = positioned.distanceM,
            lng = point.lng,
            lat = point.lat,
            pointIndex = positioned.pointIndex,
        )
    }
    if (trackPoints.isEmpty()) return emptyTrail3dTrackModel()
    val sampledPoints = evenlySampleTrackPoints(trackPoints, maxRenderedPoints)
    val eastMin = allEastMeters.minOrNull() ?: 0.0
    val eastMax = allEastMeters.maxOrNull() ?: 0.0
    val northMin = allNorthMeters.minOrNull() ?: 0.0
    val northMax = allNorthMeters.maxOrNull() ?: 0.0
    return Trail3dTrackModel(
        points = sampledPoints,
        minElevationM = trackPoints.minOfOrNull { it.elevationM },
        maxElevationM = trackPoints.maxOfOrNull { it.elevationM },
        distanceM = positionedPoints.last().distanceM,
        eastMinM = eastMin,
        eastMaxM = eastMax,
        northMinM = northMin,
        northMaxM = northMax,
        horizontalSpanM = max(eastMax - eastMin, northMax - northMin).coerceAtLeast(1.0),
        hasEnoughData = trackPoints.size >= 2 && sampledPoints.size >= 2,
    )
}

internal fun evenlySampleTrackPoints(
    points: List<Trail3dTrackPoint>,
    maxRenderedPoints: Int,
): List<Trail3dTrackPoint> {
    if (maxRenderedPoints <= 0) return emptyList()
    if (points.size <= maxRenderedPoints) return points
    if (maxRenderedPoints == 1) return listOf(points.first())
    val lastIndex = points.lastIndex
    val indices = LinkedHashSet<Int>()
    indices += 0
    indices += lastIndex
    if (maxRenderedPoints >= 4) {
        indices += points.indices.minBy { points[it].elevationM }
        indices += points.indices.maxBy { points[it].elevationM }
    }
    (0 until maxRenderedPoints).forEach { index ->
        if (indices.size < maxRenderedPoints) {
            indices += ((index * lastIndex).toDouble() / (maxRenderedPoints - 1)).roundToInt().coerceIn(0, lastIndex)
        }
    }
    return indices.sorted().map(points::get)
}

internal fun resetTrail3dCamera(): Trail3dCamera = Trail3dCamera()

internal fun setTrail3dCameraYaw(camera: Trail3dCamera, yawDegrees: Double): Trail3dCamera =
    camera.copy(yawDegrees = normalizeTrail3dYawDegrees(yawDegrees))

internal fun setTrail3dCameraPitch(camera: Trail3dCamera, pitchDegrees: Double): Trail3dCamera =
    camera.copy(
        pitchDegrees = pitchDegrees.finiteOrDefault(TRAIL_3D_DEFAULT_PITCH_DEGREES).coerceIn(
            TRAIL_3D_MIN_PITCH_DEGREES,
            TRAIL_3D_MAX_PITCH_DEGREES,
        ),
    )

internal fun setTrail3dCameraZoom(camera: Trail3dCamera, zoom: Double): Trail3dCamera =
    camera.copy(
        zoom = zoom.finiteOrDefault(TRAIL_3D_DEFAULT_ZOOM).coerceIn(
            TRAIL_3D_MIN_ZOOM,
            TRAIL_3D_MAX_ZOOM,
        ),
    )

internal fun zoomTrail3dCamera(
    camera: Trail3dCamera,
    zoomMultiplier: Double,
): Trail3dCamera = setTrail3dCameraZoom(
    camera = camera,
    zoom = camera.zoom * zoomMultiplier.finiteOrDefault(1.0),
)

internal fun panTrail3dCamera(
    camera: Trail3dCamera,
    panDeltaXPx: Double,
    panDeltaYPx: Double,
): Trail3dCamera = camera.copy(
    panXPx = camera.panXPx + panDeltaXPx.finiteOrDefault(0.0),
    panYPx = camera.panYPx + panDeltaYPx.finiteOrDefault(0.0),
)

internal fun updateTrail3dCamera(
    camera: Trail3dCamera,
    yawDeltaDegrees: Double = 0.0,
    pitchDeltaDegrees: Double = 0.0,
    zoomMultiplier: Double = 1.0,
    panDeltaXPx: Double = 0.0,
    panDeltaYPx: Double = 0.0,
): Trail3dCamera = camera
    .let { setTrail3dCameraYaw(it, it.yawDegrees + yawDeltaDegrees) }
    .let { setTrail3dCameraPitch(it, it.pitchDegrees + pitchDeltaDegrees) }
    .let { zoomTrail3dCamera(it, zoomMultiplier) }
    .let { panTrail3dCamera(it, panDeltaXPx, panDeltaYPx) }

internal fun updateTrail3dCameraFromMapGesture(
    camera: Trail3dCamera,
    rotationDeltaDegrees: Double = 0.0,
    pitchPanDeltaYPx: Double = 0.0,
    zoomMultiplier: Double = 1.0,
): Trail3dCamera = updateTrail3dCamera(
    camera = camera,
    yawDeltaDegrees = rotationDeltaDegrees.finiteOrDefault(0.0) * TRAIL_3D_ROTATION_GESTURE_YAW_FACTOR,
    pitchDeltaDegrees = -pitchPanDeltaYPx.finiteOrDefault(0.0) * TRAIL_3D_PITCH_GESTURE_PX_FACTOR,
    zoomMultiplier = zoomMultiplier,
)

internal fun normalizeTrail3dYawDegrees(yawDegrees: Double): Double {
    if (!yawDegrees.isFinite()) return TRAIL_3D_DEFAULT_YAW_DEGREES
    var normalized = yawDegrees % 360.0
    if (normalized > TRAIL_3D_MAX_YAW_DEGREES) normalized -= 360.0
    if (normalized < TRAIL_3D_MIN_YAW_DEGREES) normalized += 360.0
    return normalized
}

internal fun projectTrail3dScene(
    model: Trail3dTrackModel,
    camera: Trail3dCamera,
    viewportWidthPx: Float,
    viewportHeightPx: Float,
): Trail3dProjection? {
    if (!model.hasEnoughData || viewportWidthPx <= 0f || viewportHeightPx <= 0f) return null
    val clampedCamera = updateTrail3dCamera(camera)
    val ground = model.groundExtents()
    val groundElevation = model.minElevationM ?: 0.0
    val positions = mutableListOf<Raw3dPoint>()
    val trackRaw = model.points.map { point ->
        rawProject(
            eastM = point.eastM,
            northM = point.northM,
            elevationM = point.elevationM,
            model = model,
            groundElevationM = groundElevation,
            camera = clampedCamera,
            source = point,
        ).also(positions::add)
    }
    val shadowRaw = model.points.map { point ->
        rawProject(
            eastM = point.eastM,
            northM = point.northM,
            elevationM = groundElevation,
            model = model,
            groundElevationM = groundElevation,
            camera = clampedCamera,
            source = point,
        ).also(positions::add)
    }
    val groundRaw = listOf(
        rawProject(ground.eastMinM, ground.northMinM, groundElevation, model, groundElevation, clampedCamera),
        rawProject(ground.eastMaxM, ground.northMinM, groundElevation, model, groundElevation, clampedCamera),
        rawProject(ground.eastMaxM, ground.northMaxM, groundElevation, model, groundElevation, clampedCamera),
        rawProject(ground.eastMinM, ground.northMaxM, groundElevation, model, groundElevation, clampedCamera),
    ).onEach(positions::add)
    val gridRaw = buildGridRawLines(ground, groundElevation, model, clampedCamera)
    gridRaw.forEach { line ->
        positions += line.start
        positions += line.end
    }

    val rawMinX = positions.minOf { it.x }
    val rawMaxX = positions.maxOf { it.x }
    val rawMinY = positions.minOf { it.y }
    val rawMaxY = positions.maxOf { it.y }
    val paddingPx = min(viewportWidthPx, viewportHeightPx) * 0.08f
    val fitWidth = (viewportWidthPx - paddingPx * 2).coerceAtLeast(1f).toDouble()
    val fitHeight = (viewportHeightPx - paddingPx * 2).coerceAtLeast(1f).toDouble()
    val rawWidth = (rawMaxX - rawMinX).coerceAtLeast(1.0)
    val rawHeight = (rawMaxY - rawMinY).coerceAtLeast(1.0)
    val scale = min(fitWidth / rawWidth, fitHeight / rawHeight) * clampedCamera.zoom
    val centerRawX = (rawMinX + rawMaxX) / 2.0
    val centerRawY = (rawMinY + rawMaxY) / 2.0
    fun toScreen(raw: Raw3dPoint): Trail3dProjectedPoint = Trail3dProjectedPoint(
        x = (viewportWidthPx / 2.0 + (raw.x - centerRawX) * scale + clampedCamera.panXPx).toFloat(),
        y = (viewportHeightPx / 2.0 + (raw.y - centerRawY) * scale + clampedCamera.panYPx).toFloat(),
        depth = raw.depth,
        source = raw.source,
    )

    return Trail3dProjection(
        trackPoints = trackRaw.map(::toScreen),
        shadowPoints = shadowRaw.map(::toScreen),
        groundOutline = groundRaw.map(::toScreen),
        gridLines = gridRaw.map { line -> Trail3dProjectedLine(toScreen(line.start), toScreen(line.end)) },
    )
}

private fun rawProject(
    eastM: Double,
    northM: Double,
    elevationM: Double,
    model: Trail3dTrackModel,
    groundElevationM: Double,
    camera: Trail3dCamera,
    source: Trail3dTrackPoint? = null,
): Raw3dPoint {
    val yaw = camera.yawDegrees.toRadians()
    val pitch = camera.pitchDegrees.toRadians()
    val x = eastM
    val z = northM
    val y = (elevationM - groundElevationM) * model.verticalDisplayScale()
    val yawX = x * cos(yaw) - z * sin(yaw)
    val yawZ = x * sin(yaw) + z * cos(yaw)
    return Raw3dPoint(
        x = yawX,
        y = yawZ * sin(pitch) - y * cos(pitch),
        depth = yawZ * cos(pitch) + y * sin(pitch),
        source = source,
    )
}

private fun buildGridRawLines(
    ground: Trail3dGroundExtents,
    elevationM: Double,
    model: Trail3dTrackModel,
    camera: Trail3dCamera,
): List<Raw3dLine> {
    val lines = mutableListOf<Raw3dLine>()
    val divisions = 4
    (0..divisions).forEach { index ->
        val ratio = index.toDouble() / divisions
        val east = ground.eastMinM + (ground.eastMaxM - ground.eastMinM) * ratio
        val north = ground.northMinM + (ground.northMaxM - ground.northMinM) * ratio
        lines += Raw3dLine(
            rawProject(east, ground.northMinM, elevationM, model, elevationM, camera),
            rawProject(east, ground.northMaxM, elevationM, model, elevationM, camera),
        )
        lines += Raw3dLine(
            rawProject(ground.eastMinM, north, elevationM, model, elevationM, camera),
            rawProject(ground.eastMaxM, north, elevationM, model, elevationM, camera),
        )
    }
    return lines
}

private data class Raw3dLine(val start: Raw3dPoint, val end: Raw3dPoint)

private data class Trail3dGroundExtents(
    val eastMinM: Double,
    val eastMaxM: Double,
    val northMinM: Double,
    val northMaxM: Double,
)

private fun Trail3dTrackModel.groundExtents(): Trail3dGroundExtents {
    val padding = horizontalSpanM * 0.12
    val eastPadding = if (eastMaxM - eastMinM < 1.0) padding.coerceAtLeast(25.0) else padding
    val northPadding = if (northMaxM - northMinM < 1.0) padding.coerceAtLeast(25.0) else padding
    return Trail3dGroundExtents(
        eastMinM = eastMinM - eastPadding,
        eastMaxM = eastMaxM + eastPadding,
        northMinM = northMinM - northPadding,
        northMaxM = northMaxM + northPadding,
    )
}

private fun Trail3dTrackModel.verticalDisplayScale(): Double {
    val elevationSpan = ((maxElevationM ?: 0.0) - (minElevationM ?: 0.0)).coerceAtLeast(1.0)
    val targetScale = (horizontalSpanM * TRAIL_3D_TARGET_VERTICAL_RATIO / elevationSpan).coerceAtLeast(1.0)
    return targetScale.coerceAtMost(TRAIL_3D_MAX_VERTICAL_EXAGGERATION)
}

private fun emptyTrail3dTrackModel(): Trail3dTrackModel = Trail3dTrackModel(
    points = emptyList(),
    minElevationM = null,
    maxElevationM = null,
    distanceM = 0.0,
    eastMinM = 0.0,
    eastMaxM = 0.0,
    northMinM = 0.0,
    northMaxM = 0.0,
    horizontalSpanM = 1.0,
    hasEnoughData = false,
)

private fun TrailPoint.hasValidCoordinate(): Boolean =
    lng.isFinite() && lat.isFinite() && lng in -180.0..180.0 && lat in -90.0..90.0

private fun List<TrailPoint>.positionedValidCoordinatePoints(): List<Trail3dPositionedPoint> {
    var distanceM = 0.0
    var previousPoint: TrailPoint? = null
    return withIndex()
        .filter { (_, point) -> point.hasValidCoordinate() }
        .map { (index, point) ->
            previousPoint?.let { previous -> distanceM += haversineDistanceM(previous, point) }
            previousPoint = point
            Trail3dPositionedPoint(
                pointIndex = index,
                point = point,
                distanceM = distanceM,
            )
        }
}

private fun Trail3dPositionedPoint.interpolateElevationM(points: List<Trail3dPositionedPoint>): Double? {
    val currentIndex = points.indexOfFirst { it.pointIndex == pointIndex }
    if (currentIndex < 0) return null
    val previous = points.take(currentIndex)
        .lastOrNull { it.point.elevationM?.isFinite() == true }
    val next = points.drop(currentIndex + 1)
        .firstOrNull { it.point.elevationM?.isFinite() == true }
    val previousElevation = previous?.point?.elevationM ?: return null
    val nextElevation = next?.point?.elevationM ?: return null
    val distanceSpan = next.distanceM - previous.distanceM
    if (distanceSpan <= 0.0001) return (previousElevation + nextElevation) / 2.0
    val ratio = ((distanceM - previous.distanceM) / distanceSpan).coerceIn(0.0, 1.0)
    return previousElevation + (nextElevation - previousElevation) * ratio
}

private fun haversineDistanceM(a: TrailPoint, b: TrailPoint): Double {
    val lat1 = a.lat.toRadians()
    val lat2 = b.lat.toRadians()
    val deltaLat = (b.lat - a.lat).toRadians()
    val deltaLng = (b.lng - a.lng).toRadians()
    val h = sin(deltaLat / 2).pow(2.0) + cos(lat1) * cos(lat2) * sin(deltaLng / 2).pow(2.0)
    return 2.0 * EARTH_RADIUS_M * asin(sqrt(h.coerceIn(0.0, 1.0)))
}

private fun Double.finiteOrDefault(default: Double): Double = if (isFinite()) this else default

private fun Double.toRadians(): Double = this * PI / 180.0

private const val EARTH_RADIUS_M = 6_371_000.0
private const val TRAIL_3D_TARGET_VERTICAL_RATIO = 0.22
