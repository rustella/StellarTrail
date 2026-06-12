package com.rustella.stellartrail.feature.trails

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.core.trail.PendingTrailImport
import com.rustella.stellartrail.core.trail.PendingTrailImportStore
import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import com.rustella.stellartrail.data.trail.TrailRepositoryContract
import com.rustella.stellartrail.data.trip.TripRepositoryContract
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.Trail
import com.rustella.stellartrail.domain.trip.TrailSourceFormat
import com.rustella.stellartrail.domain.trip.TrailSummary
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripType
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class TrailLibraryUiState(
    val loading: Boolean = false,
    val trails: List<TrailSummary> = emptyList(),
    val formatFilter: TrailSourceFormat? = null,
    val sort: TrailLibrarySort = TrailLibrarySort.UpdatedDesc,
    val mapConfig: MapConfigResponse? = null,
    val preview: Trail? = null,
    val selectedTrailIds: Set<String> = emptySet(),
    val mutating: Boolean = false,
    val error: String? = null,
    val notice: String? = null,
)

enum class TrailLibrarySort {
    UpdatedDesc,
    UpdatedAsc,
}

class TrailLibraryViewModel(
    private val trailRepository: TrailRepositoryContract,
    private val tripRepository: TripRepositoryContract,
    private val tripIdForSelection: String? = null,
) : ViewModel() {
    private val _state = MutableStateFlow(TrailLibraryUiState())
    val state: StateFlow<TrailLibraryUiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, notice = null) }
            runCatching { trailRepository.list() to tripRepository.mapConfig() }.onSuccess { (response, mapConfig) ->
                _state.update { it.copy(loading = false, trails = response.items, mapConfig = mapConfig, error = null) }
            }.onFailure { error ->
                _state.update { it.copy(loading = false, error = error.userMessage()) }
            }
        }
    }

    fun uploadTrailFile(filename: String, contentType: String?, bytes: ByteArray) {
        if (bytes.isEmpty()) {
            _state.update { it.copy(error = "轨迹文件为空") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching { trailRepository.upload(bytes, filename, contentType) }.onSuccess { trail ->
                val summary = trail.toSummary()
                _state.update { state ->
                    state.copy(
                        mutating = false,
                        trails = listOf(summary) + state.trails.filterNot { it.id == summary.id },
                        formatFilter = null,
                        sort = TrailLibrarySort.UpdatedDesc,
                        preview = trail,
                        notice = "轨迹已保存到轨迹库",
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun previewTrail(id: String) {
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null) }
            runCatching { trailRepository.get(id) }.onSuccess { trail ->
                _state.update { it.copy(mutating = false, preview = trail) }
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun dismissPreview() = _state.update { it.copy(preview = null) }

    fun setFormatFilter(format: TrailSourceFormat?) {
        _state.update { it.copy(formatFilter = format, notice = null, error = null) }
    }

    fun setSort(sort: TrailLibrarySort) {
        _state.update { it.copy(sort = sort, notice = null, error = null) }
    }

    fun clearFilters() {
        _state.update {
            it.copy(formatFilter = null, sort = TrailLibrarySort.UpdatedDesc, notice = null, error = null)
        }
    }

    fun toggleTrailSelection(id: String) {
        if (tripIdForSelection == null) return
        _state.update { state ->
            val next = state.selectedTrailIds.toMutableSet()
            if (!next.add(id)) next.remove(id)
            state.copy(selectedTrailIds = next, notice = null, error = null)
        }
    }

    fun linkSelectedToTrip(onLinked: () -> Unit) {
        val tripId = tripIdForSelection ?: return
        val ids = _state.value.selectedTrailIds
        if (ids.isEmpty()) {
            _state.update { it.copy(error = "请选择要添加到行程的轨迹") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching {
                ids.forEach { tripRepository.linkTripTrail(tripId, it) }
            }.onSuccess {
                _state.update { it.copy(mutating = false, selectedTrailIds = emptySet(), notice = "已添加到行程") }
                onLinked()
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun renameTrail(id: String, displayName: String) {
        val name = displayName.trim()
        if (name.isBlank()) {
            _state.update { it.copy(error = "请输入轨迹名称") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching { trailRepository.update(id, displayName = name) }.onSuccess { trail ->
                val summary = trail.toSummary()
                _state.update { state ->
                    state.copy(
                        mutating = false,
                        trails = state.trails.map { if (it.id == id) summary else it },
                        preview = state.preview?.let { if (it.id == id) trail else it },
                        notice = "轨迹名称已更新",
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun deleteTrail(id: String) {
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching { trailRepository.delete(id) }.onSuccess {
                _state.update { state ->
                    state.copy(
                        mutating = false,
                        trails = state.trails.filterNot { it.id == id },
                        selectedTrailIds = state.selectedTrailIds - id,
                        preview = state.preview?.takeIf { it.id != id },
                        notice = "轨迹已删除",
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }
}

enum class TrailImportMode {
    Actions,
    ExistingTrip,
    NewTrip,
    OutdoorExperience,
}

data class TrailImportUiState(
    val pending: PendingTrailImport? = null,
    val uploadedTrail: Trail? = null,
    val mode: TrailImportMode = TrailImportMode.Actions,
    val trips: List<TripSummary> = emptyList(),
    val newTripTitle: String = "",
    val newTripType: TripType = TripType.SOLO,
    val outdoorExperienceTitle: String = "",
    val loading: Boolean = false,
    val mutating: Boolean = false,
    val error: String? = null,
    val notice: String? = null,
)

class TrailImportViewModel(
    private val importId: String,
    private val pendingStore: PendingTrailImportStore,
    private val trailRepository: TrailRepositoryContract,
    private val tripRepository: TripRepositoryContract,
    private val profileRepository: ProfileRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(TrailImportUiState())
    val state: StateFlow<TrailImportUiState> = _state.asStateFlow()

    init {
        val pending = pendingStore.get(importId)
        _state.value = TrailImportUiState(
            pending = pending,
            newTripTitle = pending?.filename?.defaultTitle().orEmpty(),
            outdoorExperienceTitle = pending?.filename?.defaultTitle().orEmpty(),
            error = if (pending == null) "导入文件已失效，请重新选择轨迹文件" else null,
        )
    }

    fun uploadToLibrary() {
        val pending = _state.value.pending ?: return
        val bytes = pendingStore.readBytes(importId)
        if (bytes == null || bytes.isEmpty()) {
            _state.update { it.copy(error = "导入文件已失效，请重新选择轨迹文件") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching { trailRepository.upload(bytes, pending.filename, pending.contentType) }.onSuccess { trail ->
                _state.update {
                    it.copy(
                        uploadedTrail = trail,
                        mutating = false,
                        newTripTitle = it.newTripTitle.ifBlank { trail.displayName },
                        outdoorExperienceTitle = it.outdoorExperienceTitle.ifBlank { trail.displayName },
                        notice = "轨迹已保存到轨迹库",
                    )
                }
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun showActions() = _state.update { it.copy(mode = TrailImportMode.Actions, error = null) }

    fun showExistingTrip() {
        _state.update { it.copy(mode = TrailImportMode.ExistingTrip, error = null) }
        loadTrips()
    }

    fun showNewTrip() = _state.update { it.copy(mode = TrailImportMode.NewTrip, error = null) }

    fun showOutdoorExperience() = _state.update { it.copy(mode = TrailImportMode.OutdoorExperience, error = null) }

    fun updateNewTripTitle(value: String) = _state.update { it.copy(newTripTitle = value) }

    fun updateNewTripType(value: TripType) = _state.update { it.copy(newTripType = value) }

    fun updateOutdoorExperienceTitle(value: String) = _state.update { it.copy(outdoorExperienceTitle = value) }

    fun saveOnly(onSaved: () -> Unit) {
        if (_state.value.uploadedTrail == null) {
            uploadToLibrary()
            return
        }
        pendingStore.clear(importId)
        onSaved()
    }

    fun linkToExistingTrip(tripId: String, onLinked: (String) -> Unit) {
        val trail = _state.value.uploadedTrail ?: return
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching { tripRepository.linkTripTrail(tripId, trail.id) }.onSuccess {
                pendingStore.clear(importId)
                _state.update { it.copy(mutating = false, notice = "轨迹已添加到行程") }
                onLinked(tripId)
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun createTripAndLink(onLinked: (String) -> Unit) {
        val trail = _state.value.uploadedTrail ?: return
        val title = _state.value.newTripTitle.trim()
        if (title.isBlank()) {
            _state.update { it.copy(error = "请输入行程名称") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching {
                val detail = tripRepository.create(CreateTripRequest(tripType = _state.value.newTripType, title = title))
                tripRepository.linkTripTrail(detail.trip.id, trail.id)
                detail.trip.id
            }.onSuccess { tripId ->
                pendingStore.clear(importId)
                _state.update { it.copy(mutating = false, notice = "行程已创建并添加轨迹") }
                onLinked(tripId)
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    fun createOutdoorExperienceAndLink(onLinked: () -> Unit) {
        val trail = _state.value.uploadedTrail ?: return
        val title = _state.value.outdoorExperienceTitle.trim()
        if (title.isBlank()) {
            _state.update { it.copy(error = "请输入户外经历标题") }
            return
        }
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null, notice = null) }
            runCatching {
                val experience = profileRepository.createOutdoorExperience(
                    OutdoorExperienceRequest(title = title, routeSummary = trail.displayName),
                )
                trailRepository.linkOutdoorExperienceTrail(experience.id, trail.id)
            }.onSuccess {
                pendingStore.clear(importId)
                _state.update { it.copy(mutating = false, notice = "户外经历已记录") }
                onLinked()
            }.onFailure { error ->
                _state.update { it.copy(mutating = false, error = error.userMessage()) }
            }
        }
    }

    private fun loadTrips() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            runCatching { tripRepository.list(ListTripsRequest(limit = 50)) }.onSuccess { response ->
                _state.update { it.copy(loading = false, trips = response.items) }
            }.onFailure { error ->
                _state.update { it.copy(loading = false, error = error.userMessage()) }
            }
        }
    }
}

private fun String.defaultTitle(): String = substringBeforeLast('.').ifBlank { "导入轨迹" }

private fun Trail.toSummary(): TrailSummary = TrailSummary(
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
