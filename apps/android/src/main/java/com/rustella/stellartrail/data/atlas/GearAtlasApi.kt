package com.rustella.stellartrail.data.atlas

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.atlas.CreateGearAtlasSubmissionRequest
import com.rustella.stellartrail.domain.atlas.GearAtlasPublicItem
import com.rustella.stellartrail.domain.atlas.GearAtlasSubmission
import com.rustella.stellartrail.domain.atlas.ListGearAtlasRequest
import com.rustella.stellartrail.domain.atlas.ListGearAtlasResponse
import com.rustella.stellartrail.domain.atlas.ListGearAtlasSubmissionsRequest
import com.rustella.stellartrail.domain.atlas.ListGearAtlasSubmissionsResponse
import com.rustella.stellartrail.domain.atlas.toQueryMap

class GearAtlasApi(private val apiClient: ApiClient) {
    suspend fun list(request: ListGearAtlasRequest): ListGearAtlasResponse =
        apiClient.get("/gear-atlas", query = request.toQueryMap())

    suspend fun get(id: String): GearAtlasPublicItem =
        apiClient.get("/gear-atlas/$id")

    suspend fun createSubmission(request: CreateGearAtlasSubmissionRequest): GearAtlasSubmission =
        apiClient.post("/me/gear-atlas-submissions", request)

    suspend fun createSubmissionFromGear(id: String): GearAtlasSubmission =
        apiClient.post("/me/gears/$id/atlas-submission", EmptyRequest)

    suspend fun listMySubmissions(request: ListGearAtlasSubmissionsRequest): ListGearAtlasSubmissionsResponse =
        apiClient.get(
            "/me/gear-atlas-submissions",
            query = mapOf(
                "limit" to request.limit.toString(),
                "cursor" to request.cursor,
            ),
        )

    @kotlinx.serialization.Serializable
    private object EmptyRequest
}
