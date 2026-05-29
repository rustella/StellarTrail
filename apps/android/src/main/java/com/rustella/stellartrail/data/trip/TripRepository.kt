package com.rustella.stellartrail.data.trip

import com.rustella.stellartrail.core.network.ApiException
import com.rustella.stellartrail.domain.trip.CreateTripInvitationResponse
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ImportTripPackingListRequest
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.ListTripsResponse
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import com.rustella.stellartrail.domain.trip.TripConflictResponse
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripHomeHighlightResponse
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject

class TripEditConflictException(
    val response: TripConflictResponse,
    cause: Throwable,
) : RuntimeException(response.message.ifBlank { "行程内容已被其他成员更新" }, cause)

interface TripRepositoryContract {
    suspend fun list(request: ListTripsRequest = ListTripsRequest()): ListTripsResponse
    suspend fun homeHighlight(today: String): TripHomeHighlightResponse
    suspend fun create(request: CreateTripRequest): TripDetail
    suspend fun get(id: String): TripDetail
    suspend fun update(id: String, request: UpdateTripRequest): TripDetail
    suspend fun delete(id: String)
    suspend fun updateSections(id: String, request: UpdateTripSectionsRequest): TripDetail
    suspend fun createInvitation(id: String): CreateTripInvitationResponse
    suspend fun acceptInvitation(token: String): TripDetail
    suspend fun convertToOutdoorExperience(id: String): OutdoorExperience
    suspend fun updateMember(id: String, memberId: String, request: JsonObject): TripDetail
    suspend fun removeMember(id: String, memberId: String): TripDetail
    suspend fun importPackingList(id: String, request: ImportTripPackingListRequest): TripDetail
    suspend fun createRecord(id: String, collectionPath: String, request: JsonObject): TripDetail
    suspend fun updateRecord(id: String, collectionPath: String, recordId: String, request: JsonObject): TripDetail
    suspend fun deleteRecord(id: String, collectionPath: String, recordId: String): TripDetail
    suspend fun bindSharedGearDemandMyGear(id: String, itemId: String, request: JsonObject): TripDetail
    suspend fun fillSharedGearDemandConcreteGear(id: String, itemId: String, request: JsonObject): TripDetail
}

class TripRepository(
    private val api: TripApi,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        explicitNulls = false
        encodeDefaults = false
    },
) : TripRepositoryContract {
    override suspend fun list(request: ListTripsRequest): ListTripsResponse = protectConflicts { api.list(request) }
    override suspend fun homeHighlight(today: String): TripHomeHighlightResponse = protectConflicts { api.homeHighlight(today) }
    override suspend fun create(request: CreateTripRequest): TripDetail = protectConflicts { api.create(request) }
    override suspend fun get(id: String): TripDetail = protectConflicts { api.get(id) }
    override suspend fun update(id: String, request: UpdateTripRequest): TripDetail = protectConflicts { api.update(id, request) }
    override suspend fun delete(id: String) = protectConflicts { api.delete(id) }
    override suspend fun updateSections(id: String, request: UpdateTripSectionsRequest): TripDetail =
        protectConflicts { api.updateSections(id, request) }
    override suspend fun createInvitation(id: String): CreateTripInvitationResponse = protectConflicts { api.createInvitation(id) }
    override suspend fun acceptInvitation(token: String): TripDetail = protectConflicts { api.acceptInvitation(token) }
    override suspend fun convertToOutdoorExperience(id: String): OutdoorExperience = protectConflicts {
        api.convertToOutdoorExperience(id)
    }
    override suspend fun updateMember(id: String, memberId: String, request: JsonObject): TripDetail =
        protectConflicts { api.updateMember(id, memberId, request) }
    override suspend fun removeMember(id: String, memberId: String): TripDetail = protectConflicts {
        api.removeMember(id, memberId)
    }
    override suspend fun importPackingList(id: String, request: ImportTripPackingListRequest): TripDetail =
        protectConflicts { api.importPackingList(id, request) }
    override suspend fun createRecord(id: String, collectionPath: String, request: JsonObject): TripDetail =
        protectConflicts { api.createRecord(id, collectionPath, request) }
    override suspend fun updateRecord(id: String, collectionPath: String, recordId: String, request: JsonObject): TripDetail =
        protectConflicts { api.updateRecord(id, collectionPath, recordId, request) }
    override suspend fun deleteRecord(id: String, collectionPath: String, recordId: String): TripDetail =
        protectConflicts { api.deleteRecord(id, collectionPath, recordId) }
    override suspend fun bindSharedGearDemandMyGear(id: String, itemId: String, request: JsonObject): TripDetail =
        protectConflicts { api.bindSharedGearDemandMyGear(id, itemId, request) }
    override suspend fun fillSharedGearDemandConcreteGear(id: String, itemId: String, request: JsonObject): TripDetail =
        protectConflicts { api.fillSharedGearDemandConcreteGear(id, itemId, request) }

    private inline fun <T> protectConflicts(block: () -> T): T = try {
        block()
    } catch (error: ApiException) {
        if (error.errorCode == "edit_conflict") {
            val conflict = runCatching { json.decodeFromString<TripConflictResponse>(error.rawBody) }.getOrNull()
            if (conflict != null) throw TripEditConflictException(conflict, error)
        }
        throw error
    }
}
