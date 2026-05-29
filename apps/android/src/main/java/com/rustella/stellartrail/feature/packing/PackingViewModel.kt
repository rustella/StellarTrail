package com.rustella.stellartrail.feature.packing

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.packing.PackingRepositoryContract
import com.rustella.stellartrail.domain.packing.CreateGearPackingListRequest
import com.rustella.stellartrail.domain.packing.GearPackingListDetail
import com.rustella.stellartrail.domain.packing.GearPackingListSummary
import com.rustella.stellartrail.domain.packing.ListGearPackingListsRequest
import com.rustella.stellartrail.domain.packing.UpdateGearPackingItemRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class PackingUiState(
    val isLoggedIn: Boolean = false,
    val lists: List<GearPackingListSummary> = emptyList(),
    val selectedDetail: GearPackingListDetail? = null,
    val nextCursor: String? = null,
    val loading: Boolean = false,
    val mutating: Boolean = false,
    val error: String? = null,
)

class PackingViewModel(private val repository: PackingRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(PackingUiState())
    val state: StateFlow<PackingUiState> = _state.asStateFlow()

    fun refresh(isLoggedIn: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isLoggedIn = isLoggedIn, loading = true, error = null, selectedDetail = null) }
            if (!isLoggedIn) {
                _state.update { it.copy(loading = false, lists = emptyList()) }
                return@launch
            }
            try {
                val response = repository.list(ListGearPackingListsRequest())
                _state.update { it.copy(lists = response.items, nextCursor = response.nextCursor) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun open(id: String) {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                _state.update { it.copy(selectedDetail = repository.get(id)) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun closeDetail() = _state.update { it.copy(selectedDetail = null) }

    fun createDefault() {
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null) }
            try {
                repository.create(CreateGearPackingListRequest(title = "新的打包清单", description = "出发前逐项确认装备。"))
                refresh(true)
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutating = false) }
            }
        }
    }

    fun toggleItem(itemId: String, packedQuantity: Int) {
        val detail = _state.value.selectedDetail ?: return
        viewModelScope.launch {
            _state.update { it.copy(mutating = true, error = null) }
            try {
                _state.update {
                    it.copy(selectedDetail = repository.updateItem(detail.id, itemId, UpdateGearPackingItemRequest(packedQuantity)))
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(mutating = false) }
            }
        }
    }
}
