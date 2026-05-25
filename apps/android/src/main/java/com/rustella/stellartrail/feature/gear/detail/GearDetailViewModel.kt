package com.rustella.stellartrail.feature.gear.detail

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.domain.atlas.GearAtlasStatus
import com.rustella.stellartrail.domain.gear.GearItem
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearDetailUiState(
    val item: GearItem? = null,
    val atlasStatus: GearAtlasStatus? = null,
    val atlasMessage: String? = null,
    val submittingAtlas: Boolean = false,
    val loading: Boolean = false,
    val error: String? = null,
)

class GearDetailViewModel(
    private val repository: GearRepositoryContract,
    private val atlasRepository: GearAtlasRepositoryContract,
    private val id: String,
) : ViewModel() {
    private val _state = MutableStateFlow(GearDetailUiState())
    val state: StateFlow<GearDetailUiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val item = repository.get(id)
                val submission = runCatching {
                    atlasRepository.listMySubmissions().items.firstOrNull { it.sourceUserGearId == id || it.id == id }
                }.getOrNull()
                _state.update {
                    it.copy(
                        item = item,
                        atlasStatus = submission?.status,
                        atlasMessage = submission?.rejectionReason,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun archive(onDone: () -> Unit = {}) {
        viewModelScope.launch {
            try {
                repository.archive(id)
                onDone()
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            }
        }
    }

    fun restore() {
        viewModelScope.launch {
            try {
                _state.update { it.copy(item = repository.restore(id)) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            }
        }
    }

    fun submitToAtlas() {
        if (_state.value.submittingAtlas) return
        viewModelScope.launch {
            _state.update { it.copy(submittingAtlas = true, error = null) }
            try {
                val submission = atlasRepository.createSubmissionFromGear(id)
                _state.update {
                    it.copy(
                        atlasStatus = submission.status,
                        atlasMessage = submission.rejectionReason,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(submittingAtlas = false) }
            }
        }
    }
}
