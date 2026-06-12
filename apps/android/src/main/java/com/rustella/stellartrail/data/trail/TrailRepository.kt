package com.rustella.stellartrail.data.trail

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.trip.ListTrailsResponse
import com.rustella.stellartrail.domain.trip.MapTrailLink
import com.rustella.stellartrail.domain.trip.Trail
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

class TrailApi(private val apiClient: ApiClient) {
    suspend fun list(): ListTrailsResponse = apiClient.get("/me/trails")

    suspend fun get(id: String): Trail = apiClient.get("/me/trails/$id")

    suspend fun upload(bytes: ByteArray, filename: String, contentType: String?): Trail =
        apiClient.uploadFile("/me/trails", bytes, filename, contentType)

    suspend fun update(id: String, displayName: String?, description: String?): Trail =
        apiClient.patch(
            "/me/trails/$id",
            buildJsonObject {
                displayName?.let { put("display_name", it) }
                description?.let { put("description", JsonPrimitive(it)) }
            },
        )

    suspend fun delete(id: String) {
        apiClient.delete("/me/trails/$id")
    }

    suspend fun linkOutdoorExperienceTrail(experienceId: String, trailId: String): MapTrailLink =
        apiClient.post(
            "/me/outdoor-experiences/$experienceId/trail-links",
            buildJsonObject { put("trail_id", JsonPrimitive(trailId)) },
        )
}

interface TrailRepositoryContract {
    suspend fun list(): ListTrailsResponse
    suspend fun get(id: String): Trail
    suspend fun upload(bytes: ByteArray, filename: String, contentType: String? = null): Trail
    suspend fun update(id: String, displayName: String? = null, description: String? = null): Trail
    suspend fun delete(id: String)
    suspend fun linkOutdoorExperienceTrail(experienceId: String, trailId: String): MapTrailLink
}

class TrailRepository(private val api: TrailApi) : TrailRepositoryContract {
    override suspend fun list(): ListTrailsResponse = api.list()
    override suspend fun get(id: String): Trail = api.get(id)
    override suspend fun upload(bytes: ByteArray, filename: String, contentType: String?): Trail =
        api.upload(bytes, filename, contentType)

    override suspend fun update(id: String, displayName: String?, description: String?): Trail =
        api.update(id, displayName, description)

    override suspend fun delete(id: String) = api.delete(id)
    override suspend fun linkOutdoorExperienceTrail(experienceId: String, trailId: String): MapTrailLink =
        api.linkOutdoorExperienceTrail(experienceId, trailId)
}
