package com.rustella.stellartrail.feature.trips

import com.rustella.stellartrail.data.trip.TripRepositoryContract
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
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import com.rustella.stellartrail.domain.trip.TripHomeHighlightResponse
import com.rustella.stellartrail.domain.trip.TripMapStateResponse
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripTimeBucket
import com.rustella.stellartrail.domain.trip.TripType
import com.rustella.stellartrail.domain.trip.TripsMapOverviewResponse
import com.rustella.stellartrail.domain.trip.TripsMapOverviewStats
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import java.net.UnknownHostException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.serialization.json.JsonObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class TripListViewModelTest {
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
    fun loadIfNeededKeepsTripsAndMapDuringBackgroundFailure() = runTest {
        val repository = FakeTripRepository()
        val viewModel = TripListViewModel(repository)

        viewModel.loadIfNeeded(isLoggedIn = true)
        advanceUntilIdle()
        repository.failList = true
        repository.failMap = true
        viewModel.loadIfNeeded(isLoggedIn = true)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertTrue(state.hasLoaded)
        assertFalse(state.loading)
        assertFalse(state.refreshing)
        assertEquals(listOf("武功山周末线"), state.trips.map { it.title })
        assertEquals(1, state.overviewMap.data?.stats?.tripCount)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.error)
        assertEquals("无法连接到 API，请检查网络或 API Base URL。", state.overviewMap.error)
    }

    @Test
    fun loginStateChangeClearsPrivateTripContent() = runTest {
        val viewModel = TripListViewModel(FakeTripRepository())

        viewModel.loadIfNeeded(isLoggedIn = true)
        advanceUntilIdle()
        viewModel.loadIfNeeded(isLoggedIn = false)
        advanceUntilIdle()

        val state = viewModel.state.value
        assertTrue(state.hasLoaded)
        assertFalse(state.isLoggedIn)
        assertTrue(state.trips.isEmpty())
        assertNull(state.highlight)
        assertNull(state.overviewMap.data)
    }
}

private class FakeTripRepository : TripRepositoryContract {
    var failList = false
    var failMap = false

    override suspend fun list(request: ListTripsRequest): ListTripsResponse {
        if (failList) throw UnknownHostException("api.stellartrail.example")
        return ListTripsResponse(items = listOf(sampleTrip()), nextCursor = "next")
    }

    override suspend fun homeHighlight(today: String): TripHomeHighlightResponse {
        if (failList) throw UnknownHostException("api.stellartrail.example")
        return TripHomeHighlightResponse(TripHomeHighlightItem(trip = sampleTrip()))
    }

    override suspend fun mapConfig(): MapConfigResponse = sampleMapConfig()

    override suspend fun tripsMapOverview(): TripsMapOverviewResponse {
        if (failMap) throw UnknownHostException("api.stellartrail.example")
        return TripsMapOverviewResponse(
            map = sampleMapConfig(),
            stats = TripsMapOverviewStats(tripCount = 1, trailCount = 1),
        )
    }

    override suspend fun create(request: CreateTripRequest): TripDetail = unused()
    override suspend fun get(id: String): TripDetail = unused()
    override suspend fun update(id: String, request: UpdateTripRequest): TripDetail = unused()
    override suspend fun delete(id: String) = Unit
    override suspend fun tripMap(id: String): TripMapStateResponse = unused()
    override suspend fun uploadTripTrail(id: String, bytes: ByteArray, filename: String, contentType: String?): MapTrailLink = unused()
    override suspend fun linkTripTrail(id: String, trailId: String): MapTrailLink = unused()
    override suspend fun unlinkTripTrail(id: String, trailId: String) = Unit
    override suspend fun createMapAnnotation(id: String, request: MapAnnotationRequest): MapAnnotation = unused()
    override suspend fun updateMapAnnotation(id: String, annotationId: String, request: JsonObject): MapAnnotation = unused()
    override suspend fun deleteMapAnnotation(id: String, annotationId: String) = Unit
    override suspend fun updateSections(id: String, request: UpdateTripSectionsRequest): TripDetail = unused()
    override suspend fun createInvitation(id: String): CreateTripInvitationResponse = unused()
    override suspend fun acceptInvitation(token: String): TripDetail = unused()
    override suspend fun convertToOutdoorExperience(id: String): OutdoorExperience = unused()
    override suspend fun updateMember(id: String, memberId: String, request: JsonObject): TripDetail = unused()
    override suspend fun removeMember(id: String, memberId: String): TripDetail = unused()
    override suspend fun importPackingList(id: String, request: ImportTripPackingListRequest): TripDetail = unused()
    override suspend fun createRecord(id: String, collectionPath: String, request: JsonObject): TripDetail = unused()
    override suspend fun updateRecord(id: String, collectionPath: String, recordId: String, request: JsonObject): TripDetail = unused()
    override suspend fun deleteRecord(id: String, collectionPath: String, recordId: String): TripDetail = unused()
    override suspend fun bindSharedGearDemandMyGear(id: String, itemId: String, request: JsonObject): TripDetail = unused()
    override suspend fun fillSharedGearDemandConcreteGear(id: String, itemId: String, request: JsonObject): TripDetail = unused()

    private fun sampleTrip(): TripSummary = TripSummary(
        id = "trip-1",
        ownerUserId = "user-1",
        tripType = TripType.SOLO,
        title = "武功山周末线",
        startDate = "2026-06-20",
        endDate = "2026-06-21",
        timeBucket = TripTimeBucket.UPCOMING,
        createdAt = "2026-05-01T00:00:00Z",
        updatedAt = "2026-05-01T00:00:00Z",
    )

    private fun sampleMapConfig(): MapConfigResponse = MapConfigResponse(
        provider = "maptiler",
        publicKey = "pk.test",
        enabled = true,
    )

    private fun <T> unused(): T = error("unused")
}
