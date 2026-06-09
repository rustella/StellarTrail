package com.rustella.stellartrail.ui.screens

import com.rustella.stellartrail.core.map.InMemoryMapStylePreferenceRepository
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.MapStyleOption
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
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
