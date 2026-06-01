package com.rustella.stellartrail.data.profile

import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.profile.ListOutdoorExperiencesResponse
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.profile.OutdoorProfileResponse
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import org.junit.Assert.assertEquals
import org.junit.Test

class ProfileApiTest {
    @Test
    fun profileOutdoorExperienceAndRoadmapPathsMatchBackend() = runTest {
        val requests = mutableListOf<HttpRequestData>()
        val api = ProfileApi(testClient { request ->
            requests += request
            when (request.url.encodedPath) {
                "/api/v1/me/profile/outdoor" -> respondJson(outdoorProfileJson)
                "/api/v1/me/outdoor-experiences" -> {
                    if (request.method == HttpMethod.Get) respondJson(outdoorExperiencesJson) else respondJson(outdoorExperienceJson)
                }
                "/api/v1/me/outdoor-experiences/experience-1" -> {
                    if (request.method == HttpMethod.Delete) respond("", HttpStatusCode.NoContent) else respondJson(outdoorExperienceJson)
                }
                "/api/v1/roadmap" -> respondJson(roadmapJson)
                "/api/v1/me/roadmap" -> respondJson(roadmapJson)
                "/api/v1/me/roadmap/roadmap-1/vote" -> respondJson(roadmapItemJson)
                "/api/v1/me/roadmap/roadmap-1/subscription" -> respondJson(roadmapItemJson)
                else -> error("unexpected request ${request.method.value} ${request.url}")
            }
        })
        val outdoorPatch: JsonObject = buildJsonObject { put("outdoor_id", "星星") }
        val experienceRequest = OutdoorExperienceRequest(title = "罗浮山三天两夜重装")

        api.outdoorProfile()
        api.updateOutdoorProfile(outdoorPatch)
        api.listOutdoorExperiences()
        api.createOutdoorExperience(experienceRequest)
        api.updateOutdoorExperience("experience-1", experienceRequest)
        api.deleteOutdoorExperience("experience-1")
        api.listRoadmap(false, RoadmapStatusFilter.All)
        api.listRoadmap(true, RoadmapStatusFilter.Building)
        api.voteRoadmapItem("roadmap-1")
        api.unvoteRoadmapItem("roadmap-1")
        api.subscribeRoadmapItem("roadmap-1")
        api.unsubscribeRoadmapItem("roadmap-1")

        assertEquals(
            listOf(
                HttpMethod.Get,
                HttpMethod.Patch,
                HttpMethod.Get,
                HttpMethod.Post,
                HttpMethod.Patch,
                HttpMethod.Delete,
                HttpMethod.Get,
                HttpMethod.Get,
                HttpMethod.Put,
                HttpMethod.Delete,
                HttpMethod.Put,
                HttpMethod.Delete,
            ),
            requests.map { it.method },
        )
        assertEquals(
            listOf(
                "/api/v1/me/profile/outdoor",
                "/api/v1/me/profile/outdoor",
                "/api/v1/me/outdoor-experiences",
                "/api/v1/me/outdoor-experiences",
                "/api/v1/me/outdoor-experiences/experience-1",
                "/api/v1/me/outdoor-experiences/experience-1",
                "/api/v1/roadmap",
                "/api/v1/me/roadmap",
                "/api/v1/me/roadmap/roadmap-1/vote",
                "/api/v1/me/roadmap/roadmap-1/vote",
                "/api/v1/me/roadmap/roadmap-1/subscription",
                "/api/v1/me/roadmap/roadmap-1/subscription",
            ),
            requests.map { it.url.encodedPath },
        )
        assertEquals("client_key=android&limit=50", requests[6].url.encodedQuery)
        assertEquals("client_key=android&status=building&limit=50", requests[7].url.encodedQuery)
    }

    @Test
    fun profileJsonDecodesMiniProgramParityModels() {
        val profile = ApiClient.defaultJson.decodeFromString<OutdoorProfileResponse>(outdoorProfileJson).profile
        val experiences = ApiClient.defaultJson.decodeFromString<ListOutdoorExperiencesResponse>(outdoorExperiencesJson).items

        assertEquals("星星", profile.outdoorId)
        assertEquals("家属", profile.emergencyContactRelationship)
        assertEquals("罗浮山三天两夜重装", experiences.single().title)
        assertEquals("轻量雨衣够用。", experiences.single().gearSummary)
    }

    private fun testClient(handler: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData): ApiClient {
        val engine = MockEngine { request -> handler(request) }
        return ApiClient(
            configProvider = { AppConfig("https://api.example.test") },
            httpClient = HttpClient(engine) { install(ContentNegotiation) { json(ApiClient.defaultJson) } },
        )
    }

    private fun MockRequestHandleScope.respondJson(content: String, status: HttpStatusCode = HttpStatusCode.OK) = respond(
        content = content,
        status = status,
        headers = headersOf(HttpHeaders.ContentType, "application/json"),
    )
}

private val outdoorProfileJson = """
{
  "profile": {
    "user_id": "user-1",
    "outdoor_id": "星星",
    "real_name": "星野",
    "gender": "女",
    "birth_date": "1996-03-21",
    "height_cm": 168,
    "phone": "13800000000",
    "emergency_contact": "家人",
    "emergency_contact_relationship": "家属",
    "emergency_phone": "13900000000",
    "blood_type": "O",
    "medical_history": "无",
    "allergy_history": "无",
    "medical_response_note": "随身携带常用药。",
    "diet_preference": "不吃辛辣",
    "insurance_policy_no": "TEST-001",
    "insurance_company_phone": "4000000000",
    "created_at": "2026-05-01T00:00:00Z",
    "updated_at": "2026-05-20T00:00:00Z"
  }
}
""".trimIndent()

private val outdoorExperienceJson = """
{
  "id": "experience-1",
  "user_id": "user-1",
  "source_trip_id": null,
  "trip_type": "solo",
  "title": "罗浮山三天两夜重装",
  "start_date": "2026-05-01",
  "end_date": "2026-05-03",
  "day_count": 3,
  "companion_count": 2,
  "route_summary": "罗浮山环线。",
  "gear_summary": "轻量雨衣够用。",
  "food_summary": "早餐偏少。",
  "budget_summary": "人均约 120。",
  "notes": "第二天下午注意补水。",
  "created_at": "2026-05-04T00:00:00Z",
  "updated_at": "2026-05-04T00:00:00Z"
}
""".trimIndent()

private val outdoorExperiencesJson = """{"items":[$outdoorExperienceJson]}"""

private val roadmapItemJson = """
{
  "id": "roadmap-1",
  "client_key": "android",
  "title": "Android 行程协作完善",
  "summary": "补齐协作功能。",
  "details": "完善移动端体验。",
  "category": "routes",
  "status": "building",
  "priority": 1,
  "sort_order": 10,
  "is_published": true,
  "vote_count": 36,
  "subscription_count": 12,
  "is_voted": true,
  "is_subscribed": false,
  "published_at": "2026-05-01T00:00:00Z",
  "created_at": "2026-05-01T00:00:00Z",
  "updated_at": "2026-05-20T00:00:00Z"
}
""".trimIndent()

private val roadmapJson = """{"items":[$roadmapItemJson],"next_cursor":null}"""
