package com.rustella.stellartrail.feature.atlas.list

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.domain.atlas.GearAtlasPublicItem
import com.rustella.stellartrail.domain.atlas.GearAtlasSort
import com.rustella.stellartrail.domain.atlas.ListGearAtlasRequest
import com.rustella.stellartrail.domain.gear.GearCategory
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearAtlasListUiState(
    val selectedCategory: GearCategory? = null,
    val query: String = "",
    val items: List<GearAtlasPublicItem> = emptyList(),
    val nextCursor: String? = null,
    val loading: Boolean = false,
    val loadingMore: Boolean = false,
    val error: String? = null,
)

class GearAtlasListViewModel(private val repository: GearAtlasRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(GearAtlasListUiState())
    val state: StateFlow<GearAtlasListUiState> = _state.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, loadingMore = false, error = null, items = emptyList(), nextCursor = null) }
            try {
                val response = repository.list(buildRequest(cursor = null))
                _state.update { it.copy(items = response.items, nextCursor = response.nextCursor) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun loadMore() {
        val cursor = _state.value.nextCursor ?: return
        if (_state.value.loading || _state.value.loadingMore) return
        viewModelScope.launch {
            _state.update { it.copy(loadingMore = true, error = null) }
            try {
                val response = repository.list(buildRequest(cursor))
                _state.update { it.copy(items = it.items + response.items, nextCursor = response.nextCursor) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loadingMore = false) }
            }
        }
    }

    fun updateQuery(value: String) = _state.update { it.copy(query = value) }

    fun submitSearch() = refresh()

    fun clearSearch() {
        _state.update { it.copy(query = "") }
        refresh()
    }

    fun selectCategory(category: GearCategory?) {
        _state.update { it.copy(selectedCategory = category) }
        refresh()
    }

    private fun buildRequest(cursor: String?): ListGearAtlasRequest {
        val current = _state.value
        return ListGearAtlasRequest(
            category = current.selectedCategory,
            query = current.query.trim().takeIf { it.isNotEmpty() },
            sort = GearAtlasSort.APPROVED_AT_DESC,
            limit = 20,
            cursor = cursor,
        )
    }
}
