package com.rustella.stellartrail.feature.gear.detail

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.domain.gear.GearItem
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearDetailUiState(
    val item: GearItem? = null,
    val loading: Boolean = false,
    val error: String? = null,
)

class GearDetailViewModel(
    private val repository: GearRepositoryContract,
    private val id: String,
) : ViewModel() {
    private val _state = MutableStateFlow(GearDetailUiState())
    val state: StateFlow<GearDetailUiState> = _state.asStateFlow()

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
}
