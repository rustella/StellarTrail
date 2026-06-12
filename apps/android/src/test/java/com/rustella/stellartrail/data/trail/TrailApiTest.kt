package com.rustella.stellartrail.data.trail

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class TrailApiTest {
    @Test
    fun trailLibraryEndpointsUseExpectedPaths() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TrailApi(testClient { request ->
            requests += request
            respondJson(
                when (request.url.encodedPath) {
                    "/api/v1/me/trails" -> if (request.method == HttpMethod.Get) """{"items":[]}""" else trailJson
                    "/api/v1/me/trails/trail-1" -> trailJson
                    else -> error("unexpected ${request.method.value} ${request.url.encodedPath}")
                },
            )
        })

        api.list()
        val uploaded = api.upload(byteArrayOf(1, 2, 3), "wugongshan.gpx", "application/gpx+xml")
        val detail = api.get("trail-1")
        api.update("trail-1", "新名称", null)
        api.delete("trail-1")

        assertEquals(320.0, uploaded.startElevationM ?: -1.0, 0.0)
        assertEquals(1180.0, detail.endElevationM ?: -1.0, 0.0)
        assertEquals(
            listOf(
                HttpMethod.Get,
                HttpMethod.Post,
                HttpMethod.Get,
                HttpMethod.Patch,
                HttpMethod.Delete,
            ),
            requests.map { it.method },
        )
        assertEquals(
            listOf(
                "/api/v1/me/trails",
                "/api/v1/me/trails",
                "/api/v1/me/trails/trail-1",
                "/api/v1/me/trails/trail-1",
                "/api/v1/me/trails/trail-1",
            ),
            requests.map { it.url.encodedPath },
        )
    }

    @Test
    fun outdoorExperienceTrailLinkUsesMapContextPath() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TrailApi(testClient { request ->
            requests += request
            respondJson(mapTrailLinkJson)
        })

        val link = api.linkOutdoorExperienceTrail("experience-1", "trail-1")

        assertEquals("trail-1", link.trailId)
        assertEquals(HttpMethod.Post, requests.single().method)
        assertEquals("/api/v1/me/outdoor-experiences/experience-1/trail-links", requests.single().url.encodedPath)
    }

    private fun testClient(handler: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData): ApiClient {
        val engine = MockEngine { request -> handler(request) }
        return ApiClient(
            configProvider = { AppConfig(baseUrl = "https://api.example.test", requestSignature = null) },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )
    }

    private fun MockRequestHandleScope.respondJson(content: String) = respond(
        content = content,
        headers = headersOf(HttpHeaders.ContentType, "application/json"),
    )
}

private val trailJson = """
{
  "id": "trail-1",
  "owner_user_id": "user-1",
  "display_name": "武功山轨迹",
  "source_format": "gpx",
  "original_filename": "wugongshan.gpx",
  "content_type": "application/gpx+xml",
  "size_bytes": 128,
  "sha256_hex": "fixture",
  "bucket": "trails",
  "object_key": "trails/user-1/trail-1.gpx",
  "normalized_points": [],
  "simplified_geojson": {
    "type": "Feature",
    "geometry": {"type": "LineString", "coordinates": [[114.15,27.45],[114.18,27.49]]}
  },
  "distance_m": 12000.0,
  "ascent_m": 900.0,
  "descent_m": 850.0,
  "start_elevation_m": 320.0,
  "end_elevation_m": 1180.0,
  "point_count": 2,
  "created_at": "2026-05-01T00:00:00Z",
  "updated_at": "2026-05-01T00:00:00Z"
}
""".trimIndent()

private val mapTrailLinkJson = """
{
  "trail_id": "trail-1",
  "linked_by_user_id": "user-1",
  "role": "route",
  "sort_order": 0,
  "created_at": "2026-05-01T00:00:00Z",
  "updated_at": "2026-05-01T00:00:00Z",
  "trail": {
    "id": "trail-1",
    "owner_user_id": "user-1",
    "display_name": "武功山轨迹",
    "source_format": "gpx",
    "original_filename": "wugongshan.gpx",
    "content_type": "application/gpx+xml",
    "size_bytes": 128,
    "sha256_hex": "fixture",
    "distance_m": 12000.0,
    "ascent_m": 900.0,
    "descent_m": 850.0,
    "start_elevation_m": 320.0,
    "end_elevation_m": 1180.0,
    "point_count": 2,
    "created_at": "2026-05-01T00:00:00Z",
    "updated_at": "2026-05-01T00:00:00Z"
  },
  "simplified_geojson": {
    "type": "Feature",
    "geometry": {"type": "LineString", "coordinates": [[114.15,27.45],[114.18,27.49]]}
  }
}
""".trimIndent()
