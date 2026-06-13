package com.rustella.stellartrail.feature.trails

import com.rustella.stellartrail.data.trail.TrailRepositoryContract
import com.rustella.stellartrail.data.trip.TripRepositoryContract
import com.rustella.stellartrail.domain.trip.CreateTripInvitationResponse
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ImportTripPackingListRequest
import com.rustella.stellartrail.domain.trip.ListTrailsResponse
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.ListTripsResponse
import com.rustella.stellartrail.domain.trip.MapAnnotation
import com.rustella.stellartrail.domain.trip.MapAnnotationRequest
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.MapTrailLink
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import com.rustella.stellartrail.domain.trip.Trail
import com.rustella.stellartrail.domain.trip.TrailSourceFormat
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripHomeHighlightResponse
import com.rustella.stellartrail.domain.trip.TripMapStateResponse
import com.rustella.stellartrail.domain.trip.TripsMapOverviewResponse
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class TrailLibraryViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun filterAndSortStateCanBeChangedAndCleared() = runTest {
        val viewModel = TrailLibraryViewModel(FakeTrailRepository(), FakeTripRepository())

        viewModel.load()
        advanceUntilIdle()
        viewModel.setFormatFilter(TrailSourceFormat.KML)
        viewModel.setSort(TrailLibrarySort.UpdatedAsc)

        assertEquals(TrailSourceFormat.KML, viewModel.state.value.formatFilter)
        assertEquals(TrailLibrarySort.UpdatedAsc, viewModel.state.value.sort)

        viewModel.clearFilters()

        assertNull(viewModel.state.value.formatFilter)
        assertEquals(TrailLibrarySort.UpdatedDesc, viewModel.state.value.sort)
    }

    @Test
    fun uploadResetsFiltersAndKeepsNewTrailFirst() = runTest {
        val trailRepository = FakeTrailRepository()
        val viewModel = TrailLibraryViewModel(trailRepository, FakeTripRepository())

        viewModel.load()
        advanceUntilIdle()
        viewModel.setFormatFilter(TrailSourceFormat.KML)
        viewModel.setSort(TrailLibrarySort.UpdatedAsc)
        viewModel.uploadTrailFile("new-trail.gpx", "application/gpx+xml", byteArrayOf(1, 2, 3))
        advanceUntilIdle()

        assertNull(viewModel.state.value.formatFilter)
        assertEquals(TrailLibrarySort.UpdatedDesc, viewModel.state.value.sort)
        assertEquals("trail-new", viewModel.state.value.trails.first().id)
        assertEquals("轨迹已保存到轨迹库", viewModel.state.value.notice)
    }
}

private class FakeTrailRepository : TrailRepositoryContract {
    private val existing = listOf(
        trail("trail-old", TrailSourceFormat.GPX, "2026-05-01T00:00:00Z"),
        trail("trail-kml", TrailSourceFormat.KML, "2026-05-02T00:00:00Z"),
    )

    override suspend fun list(): ListTrailsResponse = ListTrailsResponse(existing.map { it.toSummary() })
    override suspend fun get(id: String): Trail = existing.first { it.id == id }
    override suspend fun upload(bytes: ByteArray, filename: String, contentType: String?): Trail =
        trail("trail-new", TrailSourceFormat.GPX, "2026-06-01T00:00:00Z")

    override suspend fun update(id: String, displayName: String?, description: String?): Trail = error("unused")
    override suspend fun delete(id: String) = error("unused")
    override suspend fun linkOutdoorExperienceTrail(experienceId: String, trailId: String): MapTrailLink = error("unused")
}

private class FakeTripRepository : TripRepositoryContract {
    override suspend fun mapConfig(): MapConfigResponse = MapConfigResponse(
        provider = "maptiler",
        publicKey = "pk.test",
        enabled = true,
    )

    override suspend fun list(request: ListTripsRequest): ListTripsResponse = error("unused")
    override suspend fun homeHighlight(today: String): TripHomeHighlightResponse = error("unused")
    override suspend fun tripsMapOverview(): TripsMapOverviewResponse = error("unused")
    override suspend fun create(request: CreateTripRequest): TripDetail = error("unused")
    override suspend fun get(id: String): TripDetail = error("unused")
    override suspend fun update(id: String, request: UpdateTripRequest): TripDetail = error("unused")
    override suspend fun delete(id: String) = error("unused")
    override suspend fun tripMap(id: String): TripMapStateResponse = error("unused")
    override suspend fun uploadTripTrail(id: String, bytes: ByteArray, filename: String, contentType: String?): MapTrailLink = error("unused")
    override suspend fun linkTripTrail(id: String, trailId: String): MapTrailLink = error("unused")
    override suspend fun unlinkTripTrail(id: String, trailId: String) = error("unused")
    override suspend fun createMapAnnotation(id: String, request: MapAnnotationRequest): MapAnnotation = error("unused")
    override suspend fun updateMapAnnotation(id: String, annotationId: String, request: JsonObject): MapAnnotation = error("unused")
    override suspend fun deleteMapAnnotation(id: String, annotationId: String) = error("unused")
    override suspend fun updateSections(id: String, request: UpdateTripSectionsRequest): TripDetail = error("unused")
    override suspend fun createInvitation(id: String): CreateTripInvitationResponse = error("unused")
    override suspend fun acceptInvitation(token: String): TripDetail = error("unused")
    override suspend fun convertToOutdoorExperience(id: String): OutdoorExperience = error("unused")
    override suspend fun updateMember(id: String, memberId: String, request: JsonObject): TripDetail = error("unused")
    override suspend fun removeMember(id: String, memberId: String): TripDetail = error("unused")
    override suspend fun importPackingList(id: String, request: ImportTripPackingListRequest): TripDetail = error("unused")
    override suspend fun createRecord(id: String, collectionPath: String, request: JsonObject): TripDetail = error("unused")
    override suspend fun updateRecord(id: String, collectionPath: String, recordId: String, request: JsonObject): TripDetail = error("unused")
    override suspend fun deleteRecord(id: String, collectionPath: String, recordId: String): TripDetail = error("unused")
    override suspend fun bindSharedGearDemandMyGear(id: String, itemId: String, request: JsonObject): TripDetail = error("unused")
    override suspend fun fillSharedGearDemandConcreteGear(id: String, itemId: String, request: JsonObject): TripDetail = error("unused")
}

private fun trail(id: String, format: TrailSourceFormat, updatedAt: String): Trail = Trail(
    id = id,
    ownerUserId = "user-1",
    displayName = id,
    sourceFormat = format,
    originalFilename = "$id.${format.name.lowercase()}",
    contentType = "application/gpx+xml",
    sizeBytes = 128,
    sha256Hex = "sha-$id",
    bucket = "trails",
    objectKey = "trails/user-1/$id.gpx",
    simplifiedGeojson = buildJsonObject {},
    distanceM = 4500.0,
    ascentM = 300.0,
    descentM = 200.0,
    startElevationM = 80.0,
    endElevationM = 380.0,
    pointCount = 7,
    createdAt = "2026-05-01T00:00:00Z",
    updatedAt = updatedAt,
)

private fun Trail.toSummary() = com.rustella.stellartrail.domain.trip.TrailSummary(
    id = id,
    ownerUserId = ownerUserId,
    displayName = displayName,
    description = description,
    sourceFormat = sourceFormat,
    originalFilename = originalFilename,
    contentType = contentType,
    sizeBytes = sizeBytes,
    sha256Hex = sha256Hex,
    bounds = bounds,
    distanceM = distanceM,
    ascentM = ascentM,
    descentM = descentM,
    minElevationM = minElevationM,
    maxElevationM = maxElevationM,
    startElevationM = startElevationM,
    endElevationM = endElevationM,
    startTime = startTime,
    endTime = endTime,
    pointCount = pointCount,
    createdAt = createdAt,
    updatedAt = updatedAt,
)
