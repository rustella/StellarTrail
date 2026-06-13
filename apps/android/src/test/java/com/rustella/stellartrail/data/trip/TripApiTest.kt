package com.rustella.stellartrail.data.trip

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripMapStateResponse
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripSectionKey
import com.rustella.stellartrail.domain.trip.TripsMapOverviewResponse
import com.rustella.stellartrail.domain.trip.TripType
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
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import org.junit.Assert.assertEquals
import org.junit.Test

class TripApiTest {
    @Test
    fun listTripsUsesBackendTripPathAndQuery() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TripApi(testClient { request ->
            requests += request
            respondJson("""{"items":[],"next_cursor":null}""")
        })

        api.list(ListTripsRequest(limit = 20, cursor = "cursor-1"))

        val request = requests.single()
        assertEquals(HttpMethod.Get, request.method)
        assertEquals("/api/v1/me/trips", request.url.encodedPath)
        assertEquals("limit=20&cursor=cursor-1", request.url.encodedQuery)
    }

    @Test
    fun invitationAcceptanceUsesCurrentBackendPath() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TripApi(testClient { request ->
            requests += request
            respondJson(tripDetailJson)
        })

        api.acceptInvitation("11111111-2222-3333-4444-555555555555")

        assertEquals(HttpMethod.Post, requests.single().method)
        assertEquals(
            "/api/v1/me/trip-invitations/11111111-2222-3333-4444-555555555555/accept",
            requests.single().url.encodedPath,
        )
    }

    @Test
    fun tripDetailDecodesEveryCollaborativeSection() {
        val detail = ApiClient.defaultJson.decodeFromString<TripDetail>(tripDetailJson)

        assertEquals(TripType.TEAM, detail.trip.tripType)
        assertEquals(TripSectionKey.GOALS, detail.sections.last())
        assertEquals("队长", detail.members.single().profile.roleLabel)
        assertEquals("炉头", detail.sharedGearDemands.single().slotName)
        assertEquals("第一天", detail.itineraryDays.single().title)
        assertEquals("雷雨", detail.safetyRisks.single().riskType)
        assertEquals("完成穿越", detail.goals.single().content)
    }

    @Test
    fun genericTripRecordPathsMatchSectionResources() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TripApi(testClient { request ->
            requests += request
            respondJson(tripDetailJson)
        })
        val payload: JsonObject = buildJsonObject { put("name", "新路线段") }

        api.createRecord("trip-1", TripRecordKind.RouteSegment.collectionPath, payload)
        api.updateRecord("trip-1", TripRecordKind.RouteSegment.collectionPath, "segment-1", payload)
        api.deleteRecord("trip-1", TripRecordKind.RouteSegment.collectionPath, "segment-1")

        assertEquals(
            listOf(
                "/api/v1/me/trips/trip-1/route-segments",
                "/api/v1/me/trips/trip-1/route-segments/segment-1",
                "/api/v1/me/trips/trip-1/route-segments/segment-1",
            ),
            requests.map { it.url.encodedPath },
        )
        assertEquals(listOf(HttpMethod.Post, HttpMethod.Patch, HttpMethod.Delete), requests.map { it.method })
    }

    @Test
    fun mapConfigUsesPublicMapConfigPathAndHostedStyles() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TripApi(testClient { request ->
            requests += request
            respondJson(mapConfigJson)
        })

        val config = api.mapConfig()

        assertEquals(HttpMethod.Get, requests.single().method)
        assertEquals("/api/v1/map/config", requests.single().url.encodedPath)
        assertEquals(
            "https://api.example.test/api/v1/map/styles/outdoor/style.json",
            config.styles.first().styleUrl,
        )
    }

    @Test
    fun tripMapEndpointsUseSingleMapRequests() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TripApi(testClient { request ->
            requests += request
            respondJson(
                when (request.url.encodedPath) {
                    "/api/v1/me/trips/map-overview" -> tripsMapOverviewJson
                    "/api/v1/me/trips/trip-1/map" -> tripMapStateJson
                    else -> error("unexpected ${request.url.encodedPath}")
                },
            )
        })

        val overview = api.tripsMapOverview()
        val tripMap = api.tripMap("trip-1")

        assertEquals(1, overview.trails.size)
        assertEquals(1, tripMap.trails.size)
        assertEquals(
            listOf("/api/v1/me/trips/map-overview", "/api/v1/me/trips/trip-1/map"),
            requests.map { it.url.encodedPath },
        )
        assertEquals(listOf(HttpMethod.Get, HttpMethod.Get), requests.map { it.method })
    }

    @Test
    fun multipartTripTrailUploadUsesTripTrailPath() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = TripApi(testClient { request ->
            requests += request
            respondJson(mapTrailLinkJson)
        })

        val link = api.uploadTripTrail(
            id = "trip-1",
            bytes = byteArrayOf(1, 2, 3),
            filename = "china-wugongshan.gpx",
            contentType = "application/gpx+xml",
        )

        val request = requests.single()
        assertEquals("trail-1", link.trailId)
        assertEquals(HttpMethod.Post, request.method)
        assertEquals("/api/v1/me/trips/trip-1/trails", request.url.encodedPath)
        assertEquals("multipart", request.body.contentType?.contentType)
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

class TripMapDtoTest {
    @Test
    fun decodesTripMapAndOverviewResponses() {
        val tripMap = ApiClient.defaultJson.decodeFromString<TripMapStateResponse>(tripMapStateJson)
        val overview = ApiClient.defaultJson.decodeFromString<TripsMapOverviewResponse>(tripsMapOverviewJson)

        assertEquals(true, tripMap.map.enabled)
        assertEquals("outdoor", tripMap.map.defaultStyleId)
        assertEquals(listOf("outdoor", "streets", "satellite"), tripMap.map.styles.map { it.id })
        assertEquals(listOf("https://api.maptiler.com"), tripMap.map.styles.first().requestOrigins)
        assertEquals("trail-1", tripMap.trails.single().trailId)
        assertEquals("trip-1", overview.trails.single().tripId)
        assertEquals(2, overview.stats.renderedPointCount)
    }
}

private val tripDetailJson = """
{
  "trip": {
    "id": "trip-1",
    "owner_user_id": "user-1",
    "trip_type": "team",
    "title": "端午武功山",
    "enabled_sections": ["members", "personal_gear", "itinerary", "shared_gear", "food_plan", "medical_kit", "safety_plan", "rescue_info", "budget", "goals"],
    "day_count": 1,
    "itinerary_day_count": 1,
    "time_bucket": "upcoming",
    "member_count": 1,
    "readiness": {"missing_count": 0, "missing_labels": [], "completion_percent": 80},
    "field_versions": {},
    "is_deleted": false,
    "created_at": "2026-05-01T00:00:00Z",
    "updated_at": "2026-05-01T00:00:00Z"
  },
  "sections": ["members", "personal_gear", "itinerary", "shared_gear", "food_plan", "medical_kit", "safety_plan", "rescue_info", "budget", "goals"],
  "my_member_id": "member-1",
  "members": [{"id": "member-1", "trip_id": "trip-1", "user_id": "user-1", "is_owner": true, "profile": {"display_name": "星野", "role_label": "队长"}, "field_versions": {}}],
  "personal_gear": [{"id": "gear-1", "member_id": "member-1", "category": "other_gear", "category_label": "其他", "name": "头灯", "planned_quantity": 1, "packed_quantity": 1, "field_versions": {}}],
  "shared_gear_demands": [{"id": "shared-1", "category": "kitchen_system", "category_label": "餐厨系统", "name": "炉头", "responsible_member_id": "member-1", "slot_name": "炉头", "planned_quantity": 1, "packed_quantity": 0, "field_versions": {}}],
  "shared_gear_demand_templates": [],
  "itinerary_days": [{"id": "day-1", "day_index": 1, "title": "第一天", "estimate_minutes": 300, "time_slots": [], "field_versions": {}}],
  "route_segments": [{"id": "segment-1", "name": "上山", "distance_km": 5.5, "ascent_m": 800, "descent_m": 0, "formula_estimate_minutes": 210, "final_estimate_minutes": 210, "field_versions": {}}],
  "food_meals": [{"id": "meal-1", "itinerary_day_id": "day-1", "meal_key": "dinner", "skipped": false, "items": [], "field_versions": {}}],
  "food_supplies": [],
  "medical_items": [{"id": "medical-1", "name": "绷带", "required_quantity": 1, "packed_quantity": 0, "field_versions": {}}],
  "segment_assignments": [],
  "safety_risks": [{"id": "risk-1", "risk_type": "雷雨", "field_versions": {}}],
  "rescue_contacts": [{"id": "rescue-1", "organization": "景区救援", "field_versions": {}}],
  "budget_items": [{"id": "budget-1", "name": "包车", "quantity": 1, "linked_shared_gear_deleted": false, "field_versions": {}}],
  "goals": [{"id": "goal-1", "scope": "team", "content": "完成穿越", "field_versions": {}}],
  "weight_summaries": [],
  "member_gear_views": []
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
    "bounds": {"min_lng": 114.15, "min_lat": 27.45, "max_lng": 114.18, "max_lat": 27.49},
    "distance_m": 12000.0,
    "ascent_m": 900.0,
    "descent_m": 850.0,
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

private val mapConfigJson = """
{
  "provider": "maptiler",
  "public_key": "pk.test",
  "coordinate_system": "WGS84",
  "enabled": true,
  "styles": [
    {"id": "outdoor", "label": "户外", "style_url": "https://api.example.test/api/v1/map/styles/outdoor/style.json", "request_origins": ["https://api.maptiler.com"]},
    {"id": "streets", "label": "街道", "style_url": "https://api.example.test/api/v1/map/styles/streets/style.json", "request_origins": ["https://api.maptiler.com"]},
    {"id": "satellite", "label": "卫星", "style_url": "https://api.example.test/api/v1/map/styles/satellite/style.json", "request_origins": ["https://api.maptiler.com"]}
  ],
  "default_style_id": "outdoor"
}
""".trimIndent()

private val tripMapStateJson = """
{
  "map": {
    "provider": "maptiler",
    "public_key": "pk.test",
    "coordinate_system": "WGS84",
    "enabled": true,
    "styles": [
      {"id": "outdoor", "label": "户外", "style_url": "https://api.example.test/api/v1/map/styles/outdoor/style.json", "request_origins": ["https://api.maptiler.com"]},
      {"id": "streets", "label": "街道", "style_url": "https://api.example.test/api/v1/map/styles/streets/style.json", "request_origins": ["https://api.maptiler.com"]},
      {"id": "satellite", "label": "卫星", "style_url": "https://api.example.test/api/v1/map/styles/satellite/style.json", "request_origins": ["https://api.maptiler.com"]}
    ],
    "default_style_id": "outdoor"
  },
  "trails": [$mapTrailLinkJson],
  "annotations": []
}
""".trimIndent()

private val tripsMapOverviewJson = """
{
  "map": {
    "provider": "maptiler",
    "public_key": "pk.test",
    "coordinate_system": "WGS84",
    "enabled": true,
    "styles": [
      {"id": "outdoor", "label": "户外", "style_url": "https://api.example.test/api/v1/map/styles/outdoor/style.json", "request_origins": ["https://api.maptiler.com"]},
      {"id": "streets", "label": "街道", "style_url": "https://api.example.test/api/v1/map/styles/streets/style.json", "request_origins": ["https://api.maptiler.com"]},
      {"id": "satellite", "label": "卫星", "style_url": "https://api.example.test/api/v1/map/styles/satellite/style.json", "request_origins": ["https://api.maptiler.com"]}
    ],
    "default_style_id": "outdoor"
  },
  "trails": [{
    "trip_id": "trip-1",
    "trip_title": "端午武功山",
    "trip_start_date": "2026-06-01",
    "trip_end_date": "2026-06-02",
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
      "point_count": 2,
      "created_at": "2026-05-01T00:00:00Z",
      "updated_at": "2026-05-01T00:00:00Z"
    },
    "simplified_geojson": {"type":"Feature","geometry":{"type":"LineString","coordinates":[[114.15,27.45],[114.18,27.49]]}}
  }],
  "bounds": {"min_lng": 114.15, "min_lat": 27.45, "max_lng": 114.18, "max_lat": 27.49},
  "stats": {"trip_count": 1, "trail_count": 1, "rendered_point_count": 2, "total_distance_m": 12000.0, "total_ascent_m": 900.0, "total_descent_m": 850.0},
  "truncated": false
}
""".trimIndent()
