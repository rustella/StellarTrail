package com.rustella.stellartrail.feature.trips

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.trip.TripEditConflictException
import com.rustella.stellartrail.data.trip.TripRepositoryContract
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.TripConflictResponse
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripSectionKey
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripTimeBucket
import com.rustella.stellartrail.domain.trip.TripType
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import com.rustella.stellartrail.domain.trip.emptyFieldVersions
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.time.LocalDate

data class TripListUiState(
    val isLoggedIn: Boolean = false,
    val trips: List<TripSummary> = emptyList(),
    val highlight: TripHomeHighlightItem? = null,
    val nextCursor: String? = null,
    val loading: Boolean = false,
    val loadingMore: Boolean = false,
    val mutatingId: String? = null,
    val error: String? = null,
)

class TripListViewModel(private val repository: TripRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(TripListUiState())
    val state: StateFlow<TripListUiState> = _state.asStateFlow()

    fun refresh(isLoggedIn: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isLoggedIn = isLoggedIn, loading = true, error = null, trips = emptyList(), nextCursor = null) }
            if (!isLoggedIn) {
                _state.update { it.copy(loading = false) }
                return@launch
            }
            try {
                val today = LocalDate.now().toString()
                val highlight = repository.homeHighlight(today).item
                val response = repository.list(ListTripsRequest(limit = 20))
                _state.update { it.copy(highlight = highlight, trips = response.items, nextCursor = response.nextCursor) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun loadMore() {
        val cursor = _state.value.nextCursor ?: return
        if (_state.value.loading || _state.value.loadingMore || !_state.value.isLoggedIn) return
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

data class TripDetailUiState(
    val detail: TripDetail? = null,
    val selectedSection: TripSectionKey = TripSectionKey.MEMBERS,
    val invitationToken: String? = null,
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
            _state.update { it.copy(loading = true, error = null, conflict = null) }
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
