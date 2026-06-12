package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.core.map.InMemoryMapStylePreferenceRepository
import com.rustella.stellartrail.core.location.ForegroundLocation
import com.rustella.stellartrail.core.location.ForegroundLocationTrackingState
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.MapStyleOption
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
    fun mapStyleResolutionUsesBackendStylesAndFallsBackToDefault() {
        val map = mapConfigWithStyles()

        val styles = resolveMapStyleOptions(map)
        val selected = resolveSelectedMapStyle(map, selectedStyleId = "missing")

        assertEquals(listOf("outdoor", "streets", "satellite"), styles.map { it.id })
        assertEquals("outdoor", selected.id)
    }

    @Test
    fun mapStyleResolutionKeepsLegacySingleStyleConfigRenderable() {
        val map = MapConfigResponse(
            provider = "maptiler",
            styleUrl = "https://maps.example.test/custom.json",
            publicKey = "pk.test",
            enabled = true,
            styles = emptyList(),
            defaultStyleId = "",
        )

        val styles = resolveMapStyleOptions(map)
        val selected = resolveSelectedMapStyle(map, selectedStyleId = "streets")

        assertEquals(1, styles.size)
        assertEquals("outdoor", selected.id)
        assertEquals("https://maps.example.test/custom.json", selected.styleUrl)
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
        styleUrl = "https://api.maptiler.com/maps/outdoor-v2/style.json",
        publicKey = "pk.test",
        enabled = true,
        styles = listOf(
            MapStyleOption("outdoor", "户外", "https://api.maptiler.com/maps/outdoor-v2/style.json"),
            MapStyleOption("streets", "街道", "https://api.maptiler.com/maps/streets-v2/style.json"),
            MapStyleOption("satellite", "卫星", "https://api.maptiler.com/maps/satellite/style.json"),
        ),
        defaultStyleId = "outdoor",
    )
}
