package com.rustella.stellartrail.data.profile

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.profile.ListOutdoorExperiencesResponse
import com.rustella.stellartrail.domain.profile.ListRoadmapResponse
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.profile.OutdoorProfileResponse
import com.rustella.stellartrail.domain.profile.ProfileUserResponse
import com.rustella.stellartrail.domain.profile.RoadmapItem
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject

class ProfileApi(private val apiClient: ApiClient) {
    suspend fun currentProfile(): ProfileUserResponse =
        apiClient.get<ProfileUserResponse>("/me/profile").withResolvedAvatar()

    suspend fun outdoorProfile(): OutdoorProfileResponse = apiClient.get("/me/profile/outdoor")

    suspend fun updateOutdoorProfile(request: JsonObject): OutdoorProfileResponse =
        apiClient.patch("/me/profile/outdoor", request)

    suspend fun listOutdoorExperiences(): ListOutdoorExperiencesResponse =
        apiClient.get("/me/outdoor-experiences")

    suspend fun createOutdoorExperience(request: OutdoorExperienceRequest): OutdoorExperience =
        apiClient.post("/me/outdoor-experiences", request)

    suspend fun updateOutdoorExperience(id: String, request: OutdoorExperienceRequest): OutdoorExperience =
        apiClient.patch("/me/outdoor-experiences/$id", request)

    suspend fun deleteOutdoorExperience(id: String) {
        apiClient.delete("/me/outdoor-experiences/$id")
    }

    suspend fun listRoadmap(isLoggedIn: Boolean, status: RoadmapStatusFilter): ListRoadmapResponse {
        val path = if (isLoggedIn) "/me/roadmap" else "/roadmap"
        return apiClient.get(
            path = path,
            query = mapOf(
                "client_key" to "android",
                "status" to status.apiValue,
                "limit" to "50",
            ),
        )
    }

    suspend fun voteRoadmapItem(id: String): RoadmapItem =
        apiClient.put("/me/roadmap/$id/vote", emptyRequest())

    suspend fun unvoteRoadmapItem(id: String): RoadmapItem =
        apiClient.deleteReturning("/me/roadmap/$id/vote")

    suspend fun subscribeRoadmapItem(id: String): RoadmapItem =
        apiClient.put("/me/roadmap/$id/subscription", emptyRequest())

    suspend fun unsubscribeRoadmapItem(id: String): RoadmapItem =
        apiClient.deleteReturning("/me/roadmap/$id/subscription")

    private fun emptyRequest(): JsonObject = buildJsonObject { }

    private fun ProfileUserResponse.withResolvedAvatar(): ProfileUserResponse {
        val avatarUrl = user.avatarUrl?.trim()?.takeIf { it.isNotEmpty() }
        return copy(user = user.copy(avatarUrl = avatarUrl?.let(apiClient::resolveAssetUrl)))
    }
}

interface ProfileRepositoryContract {
    suspend fun currentProfile(): ProfileUserResponse
    suspend fun outdoorProfile(): OutdoorProfileResponse
    suspend fun updateOutdoorProfile(request: JsonObject): OutdoorProfileResponse
    suspend fun listOutdoorExperiences(): ListOutdoorExperiencesResponse
    suspend fun createOutdoorExperience(request: OutdoorExperienceRequest): OutdoorExperience
    suspend fun updateOutdoorExperience(id: String, request: OutdoorExperienceRequest): OutdoorExperience
    suspend fun deleteOutdoorExperience(id: String)
    suspend fun listRoadmap(isLoggedIn: Boolean, status: RoadmapStatusFilter): ListRoadmapResponse
    suspend fun voteRoadmapItem(id: String): RoadmapItem
    suspend fun unvoteRoadmapItem(id: String): RoadmapItem
    suspend fun subscribeRoadmapItem(id: String): RoadmapItem
    suspend fun unsubscribeRoadmapItem(id: String): RoadmapItem
}

class ProfileRepository(private val api: ProfileApi) : ProfileRepositoryContract {
    override suspend fun currentProfile(): ProfileUserResponse = api.currentProfile()
    override suspend fun outdoorProfile(): OutdoorProfileResponse = api.outdoorProfile()
    override suspend fun updateOutdoorProfile(request: JsonObject): OutdoorProfileResponse = api.updateOutdoorProfile(request)
    override suspend fun listOutdoorExperiences(): ListOutdoorExperiencesResponse = api.listOutdoorExperiences()
    override suspend fun createOutdoorExperience(request: OutdoorExperienceRequest): OutdoorExperience =
        api.createOutdoorExperience(request)
    override suspend fun updateOutdoorExperience(id: String, request: OutdoorExperienceRequest): OutdoorExperience =
        api.updateOutdoorExperience(id, request)
    override suspend fun deleteOutdoorExperience(id: String) = api.deleteOutdoorExperience(id)
    override suspend fun listRoadmap(isLoggedIn: Boolean, status: RoadmapStatusFilter): ListRoadmapResponse =
        api.listRoadmap(isLoggedIn, status)
    override suspend fun voteRoadmapItem(id: String): RoadmapItem = api.voteRoadmapItem(id)
    override suspend fun unvoteRoadmapItem(id: String): RoadmapItem = api.unvoteRoadmapItem(id)
    override suspend fun subscribeRoadmapItem(id: String): RoadmapItem = api.subscribeRoadmapItem(id)
    override suspend fun unsubscribeRoadmapItem(id: String): RoadmapItem = api.unsubscribeRoadmapItem(id)
}
