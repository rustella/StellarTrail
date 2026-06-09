package com.rustella.stellartrail.data.trip

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.trip.CreateTripInvitationResponse
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ImportTripPackingListRequest
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.ListTripsResponse
import com.rustella.stellartrail.domain.trip.MapAnnotation
import com.rustella.stellartrail.domain.trip.MapAnnotationRequest
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.MapTrailLink
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import com.rustella.stellartrail.domain.trip.TripMapStateResponse
import com.rustella.stellartrail.domain.trip.TripsMapOverviewResponse
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripHomeHighlightResponse
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import com.rustella.stellartrail.domain.trip.apiValue
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject

class TripApi(private val apiClient: ApiClient) {
    suspend fun list(request: ListTripsRequest): ListTripsResponse = apiClient.get(
        "/me/trips",
        query = mapOf(
            "limit" to request.limit.toString(),
            "cursor" to request.cursor,
            "bucket" to request.bucket?.apiValue(),
            "trip_type" to request.tripType?.apiValue(),
        ),
    )

    suspend fun homeHighlight(today: String): TripHomeHighlightResponse = apiClient.get(
        "/me/trips/home-highlight",
        query = mapOf("today" to today),
    )

    suspend fun mapConfig(): MapConfigResponse = apiClient.get("/me/map/config")

    suspend fun tripsMapOverview(): TripsMapOverviewResponse = apiClient.get("/me/trips/map-overview")

    suspend fun create(request: CreateTripRequest): TripDetail = apiClient.post("/me/trips", request)

    suspend fun get(id: String): TripDetail = apiClient.get("/me/trips/$id")

    suspend fun update(id: String, request: UpdateTripRequest): TripDetail = apiClient.patch("/me/trips/$id", request)

    suspend fun tripMap(id: String): TripMapStateResponse = apiClient.get("/me/trips/$id/map")

    suspend fun uploadTripTrail(id: String, bytes: ByteArray, filename: String, contentType: String?): MapTrailLink =
        apiClient.uploadFile("/me/trips/$id/trails", bytes, filename, contentType)

    suspend fun linkTripTrail(id: String, trailId: String): MapTrailLink =
        apiClient.post("/me/trips/$id/trail-links", buildJsonObject { put("trail_id", JsonPrimitive(trailId)) })

    suspend fun unlinkTripTrail(id: String, trailId: String) {
        apiClient.delete("/me/trips/$id/trail-links/$trailId")
    }

    suspend fun createMapAnnotation(id: String, request: MapAnnotationRequest): MapAnnotation =
        apiClient.post("/me/trips/$id/map-annotations", request)

    suspend fun updateMapAnnotation(id: String, annotationId: String, request: JsonObject): MapAnnotation =
        apiClient.patch("/me/trips/$id/map-annotations/$annotationId", request)

    suspend fun deleteMapAnnotation(id: String, annotationId: String) = apiClient.delete("/me/trips/$id/map-annotations/$annotationId")

    suspend fun delete(id: String) {
        apiClient.delete("/me/trips/$id")
    }

    suspend fun updateSections(id: String, request: UpdateTripSectionsRequest): TripDetail =
        apiClient.patch("/me/trips/$id/sections", request)

    suspend fun createInvitation(id: String): CreateTripInvitationResponse =
        apiClient.post("/me/trips/$id/invitations", emptyRequest())

    suspend fun acceptInvitation(token: String): TripDetail =
        apiClient.post("/me/trip-invitations/$token/accept", emptyRequest())

    suspend fun convertToOutdoorExperience(id: String): OutdoorExperience =
        apiClient.post("/me/trips/$id/convert-to-outdoor-experience", emptyRequest())

    suspend fun updateMember(id: String, memberId: String, request: JsonObject): TripDetail =
        apiClient.patch("/me/trips/$id/members/$memberId", request)

    suspend fun removeMember(id: String, memberId: String): TripDetail =
        apiClient.deleteReturning("/me/trips/$id/members/$memberId")

    suspend fun importPackingList(id: String, request: ImportTripPackingListRequest): TripDetail =
        apiClient.post("/me/trips/$id/personal-gear/import-packing-list", request)

    suspend fun createRecord(id: String, collectionPath: String, request: JsonObject): TripDetail =
        apiClient.post("/me/trips/$id/$collectionPath", request)

    suspend fun updateRecord(id: String, collectionPath: String, recordId: String, request: JsonObject): TripDetail =
        apiClient.patch("/me/trips/$id/$collectionPath/$recordId", request)

    suspend fun deleteRecord(id: String, collectionPath: String, recordId: String): TripDetail =
        apiClient.deleteReturning("/me/trips/$id/$collectionPath/$recordId")

    suspend fun bindSharedGearDemandMyGear(id: String, itemId: String, request: JsonObject): TripDetail =
        apiClient.post("/me/trips/$id/shared-gear-demands/$itemId/bind-my-gear", request)

    suspend fun fillSharedGearDemandConcreteGear(id: String, itemId: String, request: JsonObject): TripDetail =
        apiClient.post("/me/trips/$id/shared-gear-demands/$itemId/fill-concrete-gear", request)

    private fun emptyRequest(): JsonObject = buildJsonObject { }
}
