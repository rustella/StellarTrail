package com.rustella.stellartrail.feature.trips

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.trip.TripEditConflictException
import com.rustella.stellartrail.data.trip.TripRepositoryContract
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.MapAnnotationRequest
import com.rustella.stellartrail.domain.trip.TripConflictResponse
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import com.rustella.stellartrail.domain.trip.TripMapStateResponse
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripSectionKey
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripsMapOverviewResponse
import com.rustella.stellartrail.domain.trip.TripTimeBucket
import com.rustella.stellartrail.domain.trip.TripType
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import com.rustella.stellartrail.domain.trip.emptyFieldVersions
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.coroutines.launch
import java.time.LocalDate

data class TripsOverviewMapUiState(
    val loading: Boolean = false,
    val data: TripsMapOverviewResponse? = null,
    val error: String? = null,
)

data class TripListUiState(
    val isLoggedIn: Boolean = false,
    val hasLoaded: Boolean = false,
    val trips: List<TripSummary> = emptyList(),
    val highlight: TripHomeHighlightItem? = null,
    val overviewMap: TripsOverviewMapUiState = TripsOverviewMapUiState(),
    val nextCursor: String? = null,
    val loading: Boolean = false,
    val refreshing: Boolean = false,
    val loadingMore: Boolean = false,
    val mutatingId: String? = null,
    val error: String? = null,
)

class TripListViewModel(private val repository: TripRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(TripListUiState())
    val state: StateFlow<TripListUiState> = _state.asStateFlow()

    fun loadIfNeeded(isLoggedIn: Boolean) {
        val current = _state.value
        refresh(isLoggedIn = isLoggedIn, preserveContent = current.hasLoaded && current.isLoggedIn == isLoggedIn)
    }

    fun refresh(isLoggedIn: Boolean) {
        refresh(isLoggedIn = isLoggedIn, preserveContent = false)
    }

    private fun refresh(isLoggedIn: Boolean, preserveContent: Boolean) {
        viewModelScope.launch {
            val keepContent = preserveContent && _state.value.hasLoaded && _state.value.isLoggedIn == isLoggedIn
            _state.update {
                val authChanged = it.isLoggedIn != isLoggedIn
                it.copy(
                    isLoggedIn = isLoggedIn,
                    loading = !keepContent,
                    refreshing = keepContent,
                    error = null,
                    trips = if (keepContent && !authChanged) it.trips else emptyList(),
                    highlight = if (keepContent && !authChanged) it.highlight else null,
                    nextCursor = if (keepContent && !authChanged) it.nextCursor else null,
                    overviewMap = if (keepContent && !authChanged) {
                        it.overviewMap.copy(loading = isLoggedIn, error = null)
                    } else {
                        TripsOverviewMapUiState(loading = isLoggedIn)
                    },
                )
            }
            if (!isLoggedIn) {
                _state.update {
                    it.copy(
                        loading = false,
                        refreshing = false,
                        overviewMap = TripsOverviewMapUiState(),
                        hasLoaded = true,
                    )
                }
                return@launch
            }
            try {
                val today = LocalDate.now().toString()
                val highlight = repository.homeHighlight(today).item
                val response = repository.list(ListTripsRequest(limit = 20))
                _state.update {
                    it.copy(
                        highlight = highlight,
                        trips = response.items,
                        nextCursor = response.nextCursor,
                        hasLoaded = true,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false, refreshing = false) }
            }
            refreshOverviewMap()
        }
    }

    private suspend fun refreshOverviewMap() {
        if (!_state.value.isLoggedIn) return
        _state.update { it.copy(overviewMap = it.overviewMap.copy(loading = true, error = null)) }
        try {
            val overview = repository.tripsMapOverview()
            _state.update { it.copy(overviewMap = TripsOverviewMapUiState(data = overview)) }
        } catch (throwable: Throwable) {
            _state.update {
                it.copy(
                    overviewMap = it.overviewMap.copy(
                        loading = false,
                        error = throwable.userMessage(),
                    ),
                )
            }
        }
    }

    fun loadMore() {
        val cursor = _state.value.nextCursor ?: return
        if (_state.value.loading || _state.value.refreshing || _state.value.loadingMore || !_state.value.isLoggedIn) return
        viewModelScope.launch {
            _state.update { it.copy(loadingMore = true, error = null) }
            try {
                val response = repository.list(ListTripsRequest(limit = 20, cursor = cursor))
                _state.update { it.copy(trips = it.trips + response.items, nextCursor = response.nextCursor) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loadingMore = false) }
            }
        }
    }

    fun deleteTrip(id: String) {
        if (!_state.value.isLoggedIn) return
        viewModelScope.launch {
            _state.update { it.copy(mutatingId = id, error = null) }
            try {
                repository.delete(id)
                refresh(true)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutatingId = null) }
            }
        }
    }

    fun convertToExperience(id: String) {
        if (!_state.value.isLoggedIn) return
        viewModelScope.launch {
            _state.update { it.copy(mutatingId = id, error = null) }
            try {
                repository.convertToOutdoorExperience(id)
                refresh(true)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutatingId = null) }
            }
        }
    }
}

data class TripFormUiState(
    val isEdit: Boolean = false,
    val tripType: TripType = TripType.SOLO,
    val title: String = "",
    val startDate: String = "",
    val endDate: String = "",
    val description: String = "",
    val baseVersions: com.rustella.stellartrail.domain.trip.FieldVersions = emptyFieldVersions(),
    val loading: Boolean = false,
    val saving: Boolean = false,
    val error: String? = null,
)

class TripFormViewModel(
    private val repository: TripRepositoryContract,
    private val tripId: String? = null,
    tripType: TripType = TripType.SOLO,
) : ViewModel() {
    private val _state = MutableStateFlow(TripFormUiState(isEdit = tripId != null, tripType = tripType))
    val state: StateFlow<TripFormUiState> = _state.asStateFlow()

    fun load() {
        val id = tripId ?: return
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val detail = repository.get(id)
                _state.update {
                    it.copy(
                        tripType = detail.trip.tripType,
                        title = detail.trip.displayName,
                        startDate = detail.trip.startDate.orEmpty(),
                        endDate = detail.trip.endDate.orEmpty(),
                        description = detail.trip.description.orEmpty(),
                        baseVersions = detail.trip.fieldVersions,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun updateTitle(value: String) = _state.update { it.copy(title = value) }
    fun updateStartDate(value: String) = _state.update { it.copy(startDate = value) }
    fun updateEndDate(value: String) = _state.update { it.copy(endDate = value) }
    fun updateDescription(value: String) = _state.update { it.copy(description = value) }

    fun save(onSaved: (String) -> Unit) {
        val current = _state.value
        if (current.title.isBlank()) {
            _state.update { it.copy(error = "请输入行程名称") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(saving = true, error = null) }
            try {
                val detail = if (tripId == null) {
                    repository.create(
                        CreateTripRequest(
                            tripType = current.tripType,
                            title = current.title.trim(),
                            description = current.description.trim().takeIf { it.isNotEmpty() },
                            startDate = current.startDate.trim().takeIf { it.isNotEmpty() },
                            endDate = current.endDate.trim().takeIf { it.isNotEmpty() },
                        ),
                    )
                } else {
                    repository.update(
                        tripId,
                        UpdateTripRequest(
                            title = current.title.trim(),
                            description = current.description.trim().takeIf { it.isNotEmpty() },
                            startDate = current.startDate.trim().takeIf { it.isNotEmpty() },
                            endDate = current.endDate.trim().takeIf { it.isNotEmpty() },
                            baseFieldVersions = current.baseVersions,
                        ),
                    )
                }
                onSaved(detail.trip.id)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(saving = false) }
            }
        }
    }
}

data class TripJoinUiState(
    val token: String = "",
    val loading: Boolean = false,
    val error: String? = null,
)

class TripJoinViewModel(private val repository: TripRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(TripJoinUiState())
    val state: StateFlow<TripJoinUiState> = _state.asStateFlow()

    fun updateToken(value: String) = _state.update { it.copy(token = value) }

    fun accept(onAccepted: (String) -> Unit) {
        val token = extractInvitationToken(_state.value.token)
        if (token.isBlank()) {
            _state.update { it.copy(error = "请输入邀请口令") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val detail = repository.acceptInvitation(token)
                onAccepted(detail.trip.id)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    private fun extractInvitationToken(value: String): String =
        INVITATION_TOKEN_PATTERN.find(value)?.value ?: value.trim()
}

data class TripMapUiState(
    val loading: Boolean = false,
    val data: TripMapStateResponse? = null,
    val error: String? = null,
    val mutating: Boolean = false,
)

data class TripDetailUiState(
    val detail: TripDetail? = null,
    val selectedSection: TripSectionKey = TripSectionKey.MEMBERS,
    val invitationToken: String? = null,
    val map: TripMapUiState = TripMapUiState(),
    val loading: Boolean = false,
    val mutating: Boolean = false,
    val error: String? = null,
    val conflict: TripConflictResponse? = null,
)

class TripDetailViewModel(
    private val repository: TripRepositoryContract,
    private val tripId: String,
) : ViewModel() {
    private val _state = MutableStateFlow(TripDetailUiState())
    val state: StateFlow<TripDetailUiState> = _state.asStateFlow()
    private var retryWithForce: (suspend () -> TripDetail)? = null

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, conflict = null, map = it.map.copy(error = null)) }
            try {
                val detail = repository.get(tripId)
                val selected = _state.value.selectedSection.takeIf { it in detail.visibleSections() }
                    ?: detail.visibleSections().first()
                _state.update { it.copy(detail = detail, selectedSection = selected) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
            refreshMapState()
        }
    }

    fun refreshMap() {
        viewModelScope.launch { refreshMapState() }
    }

    private suspend fun refreshMapState() {
        if (_state.value.detail == null) return
        _state.update { it.copy(map = it.map.copy(loading = true, error = null)) }
        try {
            val mapState = repository.tripMap(tripId)
            _state.update { it.copy(map = TripMapUiState(data = mapState)) }
        } catch (throwable: Throwable) {
            _state.update { it.copy(map = TripMapUiState(error = throwable.userMessage())) }
        }
    }

    fun selectSection(section: TripSectionKey) = _state.update { it.copy(selectedSection = section) }

    fun toggleSection(section: TripSectionKey) {
        val detail = _state.value.detail ?: return
        val sections = detail.visibleSections().toMutableList()
        if (section in sections) sections.remove(section) else sections.add(section)
        mutate(block = {
            repository.updateSections(
                tripId,
                UpdateTripSectionsRequest(sections.distinct(), baseFieldVersions = detail.trip.fieldVersions),
            )
        })
    }

    fun createInvitation() = mutate(block = { repository.createInvitation(tripId).let { _state.update { state -> state.copy(invitationToken = it.invitation.token) }; _state.value.detail ?: repository.get(tripId) } })

    fun deleteTrip(onDeleted: () -> Unit) {
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null) }
            try {
                repository.delete(tripId)
                onDeleted()
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutating = false) }
            }
        }
    }

    fun convertToExperience() {
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null) }
            try {
                repository.convertToOutdoorExperience(tripId)
                load()
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutating = false) }
            }
        }
    }

    fun uploadTrailFile(filename: String, contentType: String?, bytes: ByteArray) {
        if (bytes.isEmpty()) {
            _state.update { it.copy(map = it.map.copy(error = "轨迹文件为空")) }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(map = it.map.copy(mutating = true, error = null)) }
            try {
                repository.uploadTripTrail(tripId, bytes, filename, contentType)
                refreshMapState()
            } catch (throwable: Throwable) {
                _state.update { it.copy(map = it.map.copy(error = throwable.userMessage())) }
            } finally {
                _state.update { it.copy(map = it.map.copy(mutating = false)) }
            }
        }
    }

    fun unlinkTrail(trailId: String) {
        viewModelScope.launch {
            _state.update { it.copy(map = it.map.copy(mutating = true, error = null)) }
            try {
                repository.unlinkTripTrail(tripId, trailId)
                refreshMapState()
            } catch (throwable: Throwable) {
                _state.update { it.copy(map = it.map.copy(error = throwable.userMessage())) }
            } finally {
                _state.update { it.copy(map = it.map.copy(mutating = false)) }
            }
        }
    }

    fun createMapAnnotation(lng: Double, lat: Double, title: String, note: String) {
        viewModelScope.launch {
            _state.update { it.copy(map = it.map.copy(mutating = true, error = null)) }
            try {
                repository.createMapAnnotation(
                    tripId,
                    MapAnnotationRequest(
                        lng = lng,
                        lat = lat,
                        title = title.trim().takeIf { it.isNotEmpty() },
                        note = note.trim().takeIf { it.isNotEmpty() },
                    ),
                )
                refreshMapState()
            } catch (throwable: Throwable) {
                _state.update { it.copy(map = it.map.copy(error = throwable.userMessage())) }
            } finally {
                _state.update { it.copy(map = it.map.copy(mutating = false)) }
            }
        }
    }

    fun updateMapAnnotation(annotationId: String, title: String, note: String) {
        viewModelScope.launch {
            _state.update { it.copy(map = it.map.copy(mutating = true, error = null)) }
            try {
                repository.updateMapAnnotation(
                    tripId,
                    annotationId,
                    buildJsonObject {
                        put("title", title.trim().takeIf { it.isNotEmpty() })
                        put("note", note.trim().takeIf { it.isNotEmpty() })
                    },
                )
                refreshMapState()
            } catch (throwable: Throwable) {
                _state.update { it.copy(map = it.map.copy(error = throwable.userMessage())) }
            } finally {
                _state.update { it.copy(map = it.map.copy(mutating = false)) }
            }
        }
    }

    fun deleteMapAnnotation(annotationId: String) {
        viewModelScope.launch {
            _state.update { it.copy(map = it.map.copy(mutating = true, error = null)) }
            try {
                repository.deleteMapAnnotation(tripId, annotationId)
                refreshMapState()
            } catch (throwable: Throwable) {
                _state.update { it.copy(map = it.map.copy(error = throwable.userMessage())) }
            } finally {
                _state.update { it.copy(map = it.map.copy(mutating = false)) }
            }
        }
    }


    fun addRecord(kind: TripRecordKind) {
        val detail = _state.value.detail ?: return
        if (kind == TripRecordKind.FoodMeal && detail.itineraryDays.isEmpty()) {
            _state.update { it.copy(error = "请先在行程安排里新增一天，再编辑食品计划。") }
            return
        }
        mutate(block = { repository.createRecord(tripId, kind.collectionPath, TripPayloads.defaultCreate(kind, detail)) })
    }

    fun updateRecord(kind: TripRecordKind, id: String, versions: com.rustella.stellartrail.domain.trip.FieldVersions = emptyFieldVersions()) {
        val patch = TripPayloads.defaultPatch(kind, versions)
        mutate(
            block = { repository.updateRecord(tripId, kind.collectionPath, id, patch) },
            forceBlock = { conflict ->
                repository.updateRecord(
                    tripId,
                    kind.collectionPath,
                    id,
                    TripPayloads.forcePatch(conflict.conflicts.map { it.field }, patch),
                )
            },
        )
    }

    fun deleteRecord(kind: TripRecordKind, id: String) = mutate(block = {
        repository.deleteRecord(tripId, kind.collectionPath, id)
    })

    fun updateMember(memberId: String, displayName: String, versions: com.rustella.stellartrail.domain.trip.FieldVersions) {
        val patch = TripPayloads.memberPatch(displayName, versions)
        mutate(
            block = { repository.updateMember(tripId, memberId, patch) },
            forceBlock = { conflict -> repository.updateMember(tripId, memberId, TripPayloads.forcePatch(conflict.conflicts.map { it.field }, patch)) },
        )
    }

    fun removeMember(memberId: String) = mutate(block = { repository.removeMember(tripId, memberId) })

    fun forceConflictOverwrite() {
        val retry = retryWithForce ?: return
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null) }
            try {
                val detail = retry()
                retryWithForce = null
                _state.update { it.copy(detail = detail, conflict = null) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutating = false) }
            }
        }
    }

    fun clearConflict() {
        retryWithForce = null
        _state.update { it.copy(conflict = null) }
    }

    private fun mutate(
        block: suspend () -> TripDetail,
        forceBlock: (suspend (TripConflictResponse) -> TripDetail)? = null,
    ) {
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, conflict = null) }
            try {
                val detail = block()
                retryWithForce = null
                _state.update { it.copy(detail = detail) }
            } catch (conflict: TripEditConflictException) {
                retryWithForce = forceBlock?.let { { it(conflict.response) } }
                _state.update { it.copy(conflict = conflict.response) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutating = false) }
            }
        }
    }
}

fun TripDetail.visibleSections(): List<TripSectionKey> =
    (sections.ifEmpty { trip.enabledSections }.ifEmpty { TripPayloads.defaultSections }).distinct()

private val INVITATION_TOKEN_PATTERN =
    Regex("[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", RegexOption.IGNORE_CASE)
