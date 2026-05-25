package com.rustella.stellartrail.feature.atlas.detail

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.domain.atlas.GearAtlasPublicItem
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearAtlasDetailUiState(
    val item: GearAtlasPublicItem? = null,
    val loading: Boolean = false,
    val error: String? = null,
)

class GearAtlasDetailViewModel(
    private val repository: GearAtlasRepositoryContract,
    private val id: String,
) : ViewModel() {
    private val _state = MutableStateFlow(GearAtlasDetailUiState())
    val state: StateFlow<GearAtlasDetailUiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                _state.update { it.copy(item = repository.get(id)) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }
}
