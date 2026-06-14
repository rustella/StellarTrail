package com.rustella.stellartrail.ui.screens

import com.maptiler.maptilersdk.events.MTEvent
import com.rustella.stellartrail.core.map.InMemoryMapStylePreferenceRepository
import com.rustella.stellartrail.core.location.ForegroundLocation
import com.rustella.stellartrail.core.location.ForegroundLocationTrackingState
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.MapStyleOption
import com.rustella.stellartrail.domain.trip.TrailBounds
import com.rustella.stellartrail.domain.trip.TrailSourceFormat
import com.rustella.stellartrail.domain.trip.TrailSummary
import com.rustella.stellartrail.domain.trip.TripOverviewMapTrail
import com.rustella.stellartrail.domain.trip.TripOverviewMapTrailSource
import com.rustella.stellartrail.domain.trip.TripsMapOverviewStats
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class TripMapComponentsTest {
    @Test
    fun tripsOverviewMapSummaryCoversTripAndLibraryTrails() {
        assertEquals(
            "2 个行程 · 5 条轨迹",
            tripsOverviewMapSummary(TripsMapOverviewStats(tripCount = 2, trailCount = 5)),
        )
        assertEquals(
            "3 条轨迹",
            tripsOverviewMapSummary(TripsMapOverviewStats(tripCount = 0, trailCount = 3)),
        )
    }

    @Test
    fun featureCollectionJsonAllowsEmptyTrailSet() {
        val json = Json.parseToJsonElement(featureCollectionJson(emptyList())) as JsonObject

        assertEquals("FeatureCollection", (json["type"] as JsonPrimitive).content)
        assertEquals(0, (json["features"] as JsonArray).size)
    }

    @Test
    fun featureCollectionJsonCombinesFeaturesForOneMapSource() {
        val feature = JsonObject(
            mapOf(
                "type" to JsonPrimitive("Feature"),
                "geometry" to JsonObject(
                    mapOf(
                        "type" to JsonPrimitive("LineString"),
                        "coordinates" to JsonArray(
                            listOf(
                                JsonArray(listOf(JsonPrimitive(114.15), JsonPrimitive(27.45))),
                                JsonArray(listOf(JsonPrimitive(114.18), JsonPrimitive(27.49))),
                            ),
                        ),
                    ),
                ),
            ),
        )

        val json = Json.parseToJsonElement(featureCollectionJson(listOf(feature))) as JsonObject

        assertEquals("FeatureCollection", (json["type"] as JsonPrimitive).content)
        assertEquals(1, (json["features"] as JsonArray).size)
    }

    @Test
    fun featureCollectionJsonLimitsRenderedLinePoints() {
        val coordinates = JsonArray(
            (0 until 20).map { index ->
                JsonArray(listOf(JsonPrimitive(114.0 + index), JsonPrimitive(22.0 + index)))
            },
        )
        val feature = JsonObject(
            mapOf(
                "type" to JsonPrimitive("Feature"),
                "geometry" to JsonObject(
                    mapOf(
                        "type" to JsonPrimitive("LineString"),
                        "coordinates" to coordinates,
                    ),
                ),
            ),
        )

        val json = Json.parseToJsonElement(featureCollectionJson(listOf(feature), maxRenderedPoints = 6)) as JsonObject
        val features = json["features"] as JsonArray
        val geometry = (features.single() as JsonObject)["geometry"] as JsonObject
        val limitedCoordinates = geometry["coordinates"] as JsonArray

        assertEquals(6, limitedCoordinates.size)
    }

    @Test
    fun overviewTrailBoundsFromGeojsonSupportsLineStringAndMultiLineString() {
        val lineBounds = overviewTrailBoundsFromGeojson(
            lineFeature(
                listOf(
                    114.15 to 27.45,
                    114.24 to 27.53,
                    114.18 to 27.49,
                ),
            ),
        )
        val multiBounds = overviewTrailBoundsFromGeojson(
            multiLineFeature(
                listOf(
                    listOf(114.10 to 27.42, 114.12 to 27.44),
                    listOf(114.30 to 27.55, 114.28 to 27.58),
                ),
            ),
        )

        assertEquals(TrailBounds(114.15, 27.45, 114.24, 27.53), lineBounds)
        assertEquals(TrailBounds(114.10, 27.42, 114.30, 27.58), multiBounds)
    }

    @Test
    fun overviewTrailBoundsFromGeojsonRejectsInvalidOrTooShortGeometry() {
        assertNull(overviewTrailBoundsFromGeojson(lineFeature(listOf(114.15 to 27.45))))
        assertNull(
            overviewTrailBoundsFromGeojson(
                JsonObject(
                    mapOf(
                        "type" to JsonPrimitive("Feature"),
                        "geometry" to JsonObject(mapOf("type" to JsonPrimitive("Point"))),
                    ),
                ),
            ),
        )
    }

    @Test
    fun overviewTrailListItemsIncludeTripAndLibrarySources() {
        val tripTrail = overviewTrail(
            id = "trip-trail",
            name = "行程轨迹",
            source = TripOverviewMapTrailSource.Trip,
            distanceM = 12000.0,
            feature = lineFeature(listOf(114.15 to 27.45, 114.18 to 27.49)),
        )
        val libraryTrail = overviewTrail(
            id = "library-trail",
            name = "自传轨迹",
            source = TripOverviewMapTrailSource.Library,
            distanceM = 8000.0,
            feature = lineFeature(listOf(114.20 to 27.50, 114.24 to 27.53)),
        )

        val items = overviewTrailListItems(listOf(tripTrail, libraryTrail))

        assertEquals(listOf("trip-trail", "library-trail"), items.map { it.trailId })
        assertEquals(listOf(TripOverviewMapTrailSource.Trip, TripOverviewMapTrailSource.Library), items.map { it.source })
        assertEquals("行程", overviewTrailSourceLabel(items.first().source))
        assertEquals("轨迹库", overviewTrailSourceLabel(items.last().source))
        assertEquals(TrailBounds(114.20, 27.50, 114.24, 27.53), items.last().bounds)
    }

    @Test
    fun nextOverviewTrailIndexHandlesEmptyUnselectedAndLoopingCases() {
        assertNull(nextOverviewTrailIndex(currentIndex = -1, itemCount = 0))
        assertEquals(0, nextOverviewTrailIndex(currentIndex = -1, itemCount = 3))
        assertEquals(2, nextOverviewTrailIndex(currentIndex = 1, itemCount = 3))
        assertEquals(0, nextOverviewTrailIndex(currentIndex = 2, itemCount = 3))
    }

    @Test
    fun mapStyleResolutionUsesBackendStylesAndFallsBackToDefault() {
        val map = mapConfigWithStyles()

        val styles = resolveMapStyleOptions(map)
        val selected = resolveSelectedMapStyle(map, selectedStyleId = "missing")

        assertEquals(listOf("outdoor", "streets", "satellite"), styles.map { it.id })
        assertEquals("outdoor", selected.id)
    }

    @Test
    fun mapStyleResolutionRequiresBackendStyleList() {
        val map = MapConfigResponse(
            provider = "maptiler",
            publicKey = "pk.test",
            enabled = true,
            styles = emptyList(),
            defaultStyleId = "",
        )

        val styles = resolveMapStyleOptions(map)

        assertTrue(styles.isEmpty())
    }

    @Test
    fun mapStyleResolutionUpgradesPublicHttpStyleUrls() {
        val map = MapConfigResponse(
            provider = "maptiler",
            publicKey = "pk.test",
            enabled = true,
            styles = listOf(
                MapStyleOption("outdoor", "户外", "http://api.stellartrail.cn/api/v1/map/styles/outdoor/style.json"),
            ),
            defaultStyleId = "outdoor",
        )

        val styles = resolveMapStyleOptions(map)

        assertEquals("https://api.stellartrail.cn/api/v1/map/styles/outdoor/style.json", styles.single().styleUrl)
    }

    @Test
    fun mapStyleResolutionKeepsLocalHttpStyleUrls() {
        assertEquals(
            "http://127.0.0.1:8080/api/v1/map/styles/outdoor/style.json",
            normalizeMapStyleUrl(" http://127.0.0.1:8080/api/v1/map/styles/outdoor/style.json "),
        )
        assertEquals(
            "http://10.37.112.178:8080/api/v1/map/styles/outdoor/style.json",
            normalizeMapStyleUrl("http://10.37.112.178:8080/api/v1/map/styles/outdoor/style.json"),
        )
    }

    @Test
    fun flatTrailMapPresentationKeepsExistingGestureDefaults() {
        val presentation = trailMapPresentation(terrain3dEnabled = false, zoomGesturesEnabled = false)

        assertFalse(presentation.terrainEnabled)
        assertNull(presentation.terrainExaggeration)
        assertNull(presentation.pitch)
        assertNull(presentation.bearing)
        assertFalse(presentation.pinchRotateEnabled)
        assertFalse(presentation.pitchGestureEnabled)

        val zoomPresentation = trailMapPresentation(terrain3dEnabled = false, zoomGesturesEnabled = true)
        assertTrue(zoomPresentation.pinchRotateEnabled)
        assertFalse(zoomPresentation.pitchGestureEnabled)
    }

    @Test
    fun terrainTrailMapPresentationEnables3dCameraTerrainAndGestures() {
        val presentation = trailMapPresentation(terrain3dEnabled = true, zoomGesturesEnabled = false)

        assertTrue(presentation.terrainEnabled)
        assertEquals(1.35, presentation.terrainExaggeration ?: -1.0, 0.0)
        assertEquals(60.0, presentation.pitch ?: -1.0, 0.0)
        assertEquals(-25.0, presentation.bearing ?: 0.0, 0.0)
        assertTrue(presentation.pinchRotateEnabled)
        assertTrue(presentation.pitchGestureEnabled)
    }

    @Test
    fun map3dGestureGuideDescribesTerrainMapControls() {
        val lines = map3dGestureGuideLines()

        assertTrue(lines.contains("单指拖动移动地图"))
        assertTrue(lines.contains("双指捏合缩放"))
        assertTrue(lines.contains("双指旋转方向"))
        assertTrue(lines.contains("双指上下拖动调整俯仰"))
        assertTrue(lines.contains("双击放大"))
    }

    @Test
    fun trailMapRenderIdentityChangesForStyleIdStyleUrlAndPresentation() {
        val flatPresentation = trailMapPresentation(terrain3dEnabled = false, zoomGesturesEnabled = true)
        val terrainPresentation = trailMapPresentation(terrain3dEnabled = true, zoomGesturesEnabled = true)
        val outdoor = MapStyleOption("outdoor", "户外", "https://api.example.test/api/v1/map/styles/outdoor/style.json")
        val sameUrlStreets = MapStyleOption("streets", "街道", outdoor.styleUrl)
        val differentUrlOutdoor = outdoor.copy(styleUrl = "https://api.example.test/api/v1/map/styles/outdoor-v2/style.json")

        val identity = trailMapRenderIdentity(outdoor, flatPresentation)

        assertFalse(identity == trailMapRenderIdentity(sameUrlStreets, flatPresentation))
        assertFalse(identity == trailMapRenderIdentity(differentUrlOutdoor, flatPresentation))
        assertFalse(identity == trailMapRenderIdentity(outdoor, terrainPresentation))
    }

    @Test
    fun selectedTrailRenderIdentityChangesForTrailFeatureAndPresentation() {
        val flatPresentation = trailMapPresentation(terrain3dEnabled = false, zoomGesturesEnabled = true)
        val terrainPresentation = trailMapPresentation(terrain3dEnabled = true, zoomGesturesEnabled = true)
        val featureA = featureCollectionJson(listOf(lineFeature(listOf(114.15 to 27.45, 114.18 to 27.49))))
        val featureB = featureCollectionJson(listOf(lineFeature(listOf(114.20 to 27.50, 114.24 to 27.53))))

        val selectedA = overviewSelectedTrailRenderIdentity("trail-a", featureA, flatPresentation)

        assertNull(overviewSelectedTrailRenderIdentity(null, featureA, flatPresentation))
        assertFalse(selectedA == overviewSelectedTrailRenderIdentity("trail-b", featureA, flatPresentation))
        assertFalse(selectedA == overviewSelectedTrailRenderIdentity("trail-a", featureB, flatPresentation))
        assertFalse(selectedA == overviewSelectedTrailRenderIdentity("trail-a", featureA, terrainPresentation))
    }

    @Test
    fun overviewMapFocusPaddingLeavesMoreRoomFor3dAndSidebar() {
        val flatPadding = overviewMapFocusPadding(terrain3dEnabled = false)
        val terrainPadding = overviewMapFocusPadding(terrain3dEnabled = true)

        assertTrue(flatPadding.right > flatPadding.left)
        assertTrue(terrainPadding.right > terrainPadding.left)
        assertTrue(terrainPadding.top > flatPadding.top)
        assertTrue(terrainPadding.bottom > flatPadding.bottom)
    }

    @Test
    fun inMemoryMapStylePreferenceStoresSelectedStyleId() {
        val repository = InMemoryMapStylePreferenceRepository()

        repository.selectStyle(" streets ")

        assertEquals("streets", repository.selectedStyleId.value)
        repository.selectStyle(" ")
        assertEquals("streets", repository.selectedStyleId.value)
    }

    @Test
    fun currentLocationMarkerRendersOnlyWhileFollowingWithLocation() {
        val location = ForegroundLocation(longitude = 114.0579, latitude = 22.5431, accuracyMeters = 5f)

        assertFalse(shouldRenderCurrentLocationMarker(ForegroundLocationTrackingState.Idle, location))
        assertTrue(shouldRenderCurrentLocationMarker(ForegroundLocationTrackingState.Starting, location))
        assertFalse(shouldRenderCurrentLocationMarker(ForegroundLocationTrackingState.Starting, null))
        assertFalse(shouldRenderCurrentLocationMarker(ForegroundLocationTrackingState.Following, null))
        assertTrue(shouldRenderCurrentLocationMarker(ForegroundLocationTrackingState.Following, location))
    }

    @Test
    fun locationTrackingHandoffKeepsActiveStateAndLastLocationTogether() {
        val location = ForegroundLocation(longitude = 114.0579, latitude = 22.5431, accuracyMeters = 5f)

        val handoff = LocationTrackingHandoff()
            .withActive(true)
            .withLocation(location)

        assertTrue(handoff.active)
        assertEquals(location, handoff.lastLocation)
        assertEquals(LocationTrackingHandoff(), handoff.withActive(false))
    }

    @Test
    fun locationTrackingHandoffFallsBackToInitialLocationOnResume() {
        val initialLocation = ForegroundLocation(longitude = 114.0579, latitude = 22.5431, accuracyMeters = 5f)
        val activeWithoutFreshFix = LocationTrackingHandoff(active = true)

        val resumed = activeWithoutFreshFix.withFallbackLocation(initialLocation)

        assertTrue(resumed.active)
        assertEquals(initialLocation, resumed.lastLocation)
        assertEquals(LocationTrackingHandoff(), LocationTrackingHandoff().withFallbackLocation(initialLocation))
    }

    @Test
    fun locationTrackingAutoStartRequiresTokenEnabledMapAndInactiveState() {
        assertTrue(shouldAutoStartLocationTracking(1, locationTrackingEnabled = true, ForegroundLocationTrackingState.Idle))
        assertFalse(shouldAutoStartLocationTracking(0, locationTrackingEnabled = true, ForegroundLocationTrackingState.Idle))
        assertFalse(shouldAutoStartLocationTracking(1, locationTrackingEnabled = false, ForegroundLocationTrackingState.Idle))
        assertFalse(shouldAutoStartLocationTracking(1, locationTrackingEnabled = true, ForegroundLocationTrackingState.Starting))
        assertFalse(shouldAutoStartLocationTracking(1, locationTrackingEnabled = true, ForegroundLocationTrackingState.Following))
    }

    @Test
    fun transferredLocationMarkerIsKeptUntilAutoStartBecomesActive() {
        val location = ForegroundLocation(longitude = 114.0579, latitude = 22.5431, accuracyMeters = 5f)

        assertTrue(
            shouldKeepTransferredLocationMarker(
                autoStartKey = 1,
                locationTrackingEnabled = true,
                state = ForegroundLocationTrackingState.Idle,
                location = location,
            ),
        )
        assertFalse(
            shouldKeepTransferredLocationMarker(
                autoStartKey = 0,
                locationTrackingEnabled = true,
                state = ForegroundLocationTrackingState.Idle,
                location = location,
            ),
        )
        assertFalse(
            shouldKeepTransferredLocationMarker(
                autoStartKey = 1,
                locationTrackingEnabled = true,
                state = ForegroundLocationTrackingState.Following,
                location = location,
            ),
        )
        assertFalse(
            shouldKeepTransferredLocationMarker(
                autoStartKey = 1,
                locationTrackingEnabled = true,
                state = ForegroundLocationTrackingState.Idle,
                location = null,
            ),
        )
    }

    @Test
    fun locationTrackingAutoStartKeyOnlyAdvancesWhenTransferIsNeeded() {
        assertEquals(3, nextLocationTrackingAutoStartKey(currentKey = 2, shouldAutoStart = true))
        assertEquals(0, nextLocationTrackingAutoStartKey(currentKey = 2, shouldAutoStart = false))
    }

    @Test
    fun currentLocationMarkerVisualSpecHasVisibleConcentricLayers() {
        val spec = currentLocationMarkerVisualSpec()

        assertEquals(40, spec.sizePx)
        assertTrue(spec.outerRadiusPx > spec.strokeRadiusPx)
        assertTrue(spec.strokeRadiusPx > spec.innerRadiusPx)
        assertEquals(0xFFFFFFFF.toInt(), spec.strokeColor)
        assertEquals(0xFF0B7CFF.toInt(), spec.innerColor)
    }

    @Test
    fun mapInteractionsDoNotStopLocationTracking() {
        assertTrue(shouldStopLocationTracking(LocationTrackingStopReason.UserButton))
        assertTrue(shouldStopLocationTracking(LocationTrackingStopReason.AppBackgrounded))
        assertTrue(shouldStopLocationTracking(LocationTrackingStopReason.MapNotVisible))
        assertFalse(shouldStopLocationTracking(LocationTrackingStopReason.MapControlZoom))
        assertFalse(shouldStopLocationTracking(LocationTrackingStopReason.MapCanvasGesture))
        assertFalse(shouldStopLocationTracking(LocationTrackingStopReason.StyleSwitch))
    }

    @Test
    fun trailLayerIsEnsuredAfterMapStyleReadinessEvents() {
        assertTrue(shouldEnsureTrailLayerOnEvent(MTEvent.ON_READY))
        assertTrue(shouldEnsureTrailLayerOnEvent(MTEvent.ON_LOAD))
        assertFalse(shouldEnsureTrailLayerOnEvent(MTEvent.ON_IDLE))
    }

    @Test
    fun locationCameraMovesOnlyForInitialLocate() {
        assertEquals(
            LocationCameraMode.InitialLocate,
            locationCameraModeForTrackingState(
                ForegroundLocationTrackingState.Starting,
                firstLocationMode = LocationCameraMode.InitialLocate,
            ),
        )
        assertEquals(
            LocationCameraMode.TransferredTracking,
            locationCameraModeForTrackingState(
                ForegroundLocationTrackingState.Starting,
                firstLocationMode = LocationCameraMode.TransferredTracking,
            ),
        )
        assertEquals(
            LocationCameraMode.MarkerOnlyUpdate,
            locationCameraModeForTrackingState(
                ForegroundLocationTrackingState.Following,
                firstLocationMode = LocationCameraMode.InitialLocate,
            ),
        )
        assertTrue(shouldMoveLocationCamera(LocationCameraMode.InitialLocate))
        assertFalse(shouldMoveLocationCamera(LocationCameraMode.MarkerOnlyUpdate))
        assertFalse(shouldMoveLocationCamera(LocationCameraMode.TransferredTracking))
        assertTrue(shouldResetLocationZoom(LocationCameraMode.InitialLocate))
        assertFalse(shouldResetLocationZoom(LocationCameraMode.MarkerOnlyUpdate))
        assertFalse(shouldResetLocationZoom(LocationCameraMode.TransferredTracking))
    }

    @Test
    fun mapCameraSnapshotAcceptsValidCameraAndRejectsInvalidValues() {
        val snapshot = mapCameraSnapshotOrNull(centerLng = 114.0579, centerLat = 22.5431, zoom = 12.5)

        assertEquals(MapCameraSnapshot(centerLng = 114.0579, centerLat = 22.5431, zoom = 12.5), snapshot)
        assertNull(mapCameraSnapshotOrNull(centerLng = 181.0, centerLat = 22.5431, zoom = 12.5))
        assertNull(mapCameraSnapshotOrNull(centerLng = 114.0579, centerLat = -91.0, zoom = 12.5))
        assertNull(mapCameraSnapshotOrNull(centerLng = 114.0579, centerLat = 22.5431, zoom = Double.NaN))
    }

    @Test
    fun initialMapCameraSourcePrefersSnapshotWhenAvailable() {
        val snapshot = MapCameraSnapshot(centerLng = 114.0579, centerLat = 22.5431, zoom = 12.5)

        assertEquals(InitialMapCameraSource.Snapshot, initialMapCameraSource(snapshot))
        assertEquals(InitialMapCameraSource.BoundsOrDefault, initialMapCameraSource(null))
    }

    private fun mapConfigWithStyles() = MapConfigResponse(
        provider = "maptiler",
        publicKey = "pk.test",
        enabled = true,
        styles = listOf(
            MapStyleOption("outdoor", "户外", "https://api.example.test/api/v1/map/styles/outdoor/style.json"),
            MapStyleOption("streets", "街道", "https://api.example.test/api/v1/map/styles/streets/style.json"),
            MapStyleOption("satellite", "卫星", "https://api.example.test/api/v1/map/styles/satellite/style.json"),
        ),
        defaultStyleId = "outdoor",
    )

    private fun lineFeature(points: List<Pair<Double, Double>>): JsonObject = JsonObject(
        mapOf(
            "type" to JsonPrimitive("Feature"),
            "geometry" to JsonObject(
                mapOf(
                    "type" to JsonPrimitive("LineString"),
                    "coordinates" to JsonArray(points.map { (lng, lat) -> coordinate(lng, lat) }),
                ),
            ),
        ),
    )

    private fun multiLineFeature(lines: List<List<Pair<Double, Double>>>): JsonObject = JsonObject(
        mapOf(
            "type" to JsonPrimitive("Feature"),
            "geometry" to JsonObject(
                mapOf(
                    "type" to JsonPrimitive("MultiLineString"),
                    "coordinates" to JsonArray(lines.map { line ->
                        JsonArray(line.map { (lng, lat) -> coordinate(lng, lat) })
                    }),
                ),
            ),
        ),
    )

    private fun coordinate(lng: Double, lat: Double): JsonArray =
        JsonArray(listOf(JsonPrimitive(lng), JsonPrimitive(lat)))

    private fun overviewTrail(
        id: String,
        name: String,
        source: TripOverviewMapTrailSource,
        distanceM: Double,
        feature: JsonObject,
    ): TripOverviewMapTrail = TripOverviewMapTrail(
        source = source,
        tripId = if (source == TripOverviewMapTrailSource.Trip) "trip-1" else null,
        tripTitle = if (source == TripOverviewMapTrailSource.Trip) "端午武功山" else null,
        trailId = id,
        linkedByUserId = if (source == TripOverviewMapTrailSource.Trip) "user-1" else null,
        role = if (source == TripOverviewMapTrailSource.Trip) "route" else null,
        sortOrder = if (source == TripOverviewMapTrailSource.Trip) 0 else null,
        createdAt = if (source == TripOverviewMapTrailSource.Trip) "2026-05-01T00:00:00Z" else null,
        updatedAt = if (source == TripOverviewMapTrailSource.Trip) "2026-05-01T00:00:00Z" else null,
        trail = TrailSummary(
            id = id,
            ownerUserId = "user-1",
            displayName = name,
            sourceFormat = TrailSourceFormat.GPX,
            originalFilename = "$id.gpx",
            contentType = "application/gpx+xml",
            sizeBytes = 128,
            sha256Hex = "$id-sha",
            distanceM = distanceM,
            ascentM = 100.0,
            descentM = 80.0,
            pointCount = 2,
            createdAt = "2026-05-01T00:00:00Z",
            updatedAt = "2026-05-01T00:00:00Z",
        ),
        simplifiedGeojson = feature,
    )
}
