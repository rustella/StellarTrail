package com.rustella.stellartrail.data.trip

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripSectionKey
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

    private fun testClient(handler: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData): ApiClient {
        val engine = MockEngine { request -> handler(request) }
        return ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )
    }

    private fun MockRequestHandleScope.respondJson(content: String) = respond(
        content = content,
        headers = headersOf(HttpHeaders.ContentType, "application/json"),
    )
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
