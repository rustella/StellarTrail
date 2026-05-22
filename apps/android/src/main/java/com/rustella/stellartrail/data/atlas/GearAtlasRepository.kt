package com.rustella.stellartrail.data.atlas

import com.rustella.stellartrail.domain.atlas.CreateGearAtlasSubmissionRequest
import com.rustella.stellartrail.domain.atlas.GearAtlasPublicItem
import com.rustella.stellartrail.domain.atlas.GearAtlasSubmission
import com.rustella.stellartrail.domain.atlas.ListGearAtlasRequest
import com.rustella.stellartrail.domain.atlas.ListGearAtlasResponse
import com.rustella.stellartrail.domain.atlas.ListGearAtlasSubmissionsRequest
import com.rustella.stellartrail.domain.atlas.ListGearAtlasSubmissionsResponse

interface GearAtlasRepositoryContract {
    suspend fun list(request: ListGearAtlasRequest = ListGearAtlasRequest()): ListGearAtlasResponse
    suspend fun get(id: String): GearAtlasPublicItem
    suspend fun createSubmission(request: CreateGearAtlasSubmissionRequest): GearAtlasSubmission
    suspend fun createSubmissionFromGear(id: String): GearAtlasSubmission
    suspend fun listMySubmissions(request: ListGearAtlasSubmissionsRequest = ListGearAtlasSubmissionsRequest()): ListGearAtlasSubmissionsResponse
}

class GearAtlasRepository(private val api: GearAtlasApi) : GearAtlasRepositoryContract {
    override suspend fun list(request: ListGearAtlasRequest): ListGearAtlasResponse = api.list(request)
    override suspend fun get(id: String): GearAtlasPublicItem = api.get(id)
    override suspend fun createSubmission(request: CreateGearAtlasSubmissionRequest): GearAtlasSubmission = api.createSubmission(request)
    override suspend fun createSubmissionFromGear(id: String): GearAtlasSubmission = api.createSubmissionFromGear(id)
    override suspend fun listMySubmissions(request: ListGearAtlasSubmissionsRequest): ListGearAtlasSubmissionsResponse =
        api.listMySubmissions(request)
}
