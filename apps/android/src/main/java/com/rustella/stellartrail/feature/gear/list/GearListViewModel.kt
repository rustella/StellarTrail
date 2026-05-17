package com.rustella.stellartrail.feature.gear.list

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearSort
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.feature.home.EMPTY_STATS
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearListUiState(
    val tab: GearTab = GearTab.AVAILABLE,
    val selectedCategory: GearCategory? = null,
    val selectedStatus: GearStatus? = null,
    val sort: GearSort = GearSort.CREATED_AT_DESC,
    val query: String = "",
    val categories: GearCategoriesResponse = GearCategoriesResponse(emptyList()),
    val stats: GearStatsResponse = EMPTY_STATS,
    val gears: List<GearSummary> = emptyList(),
    val nextCursor: String? = null,
    val loading: Boolean = false,
    val loadingMore: Boolean = false,
    val error: String? = null,
)

class GearListViewModel(private val repository: GearRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(GearListUiState())
    val state: StateFlow<GearListUiState> = _state.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, gears = emptyList(), nextCursor = null) }
            try {
                val request = buildRequest(cursor = null)
                val categories = async { repository.listCategories(request.tab) }
                val stats = async { repository.stats(request.tab) }
                val list = async { repository.list(request) }
                val response = list.await()
                _state.update {
                    it.copy(
                        categories = categories.await(),
                        stats = stats.await(),
                        gears = response.items,
                        nextCursor = response.nextCursor,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun loadMore() {
        val cursor = _state.value.nextCursor ?: return
        if (_state.value.loadingMore || _state.value.loading) return
        viewModelScope.launch {
            _state.update { it.copy(loadingMore = true, error = null) }
            try {
                val response = repository.list(buildRequest(cursor))
                _state.update { it.copy(gears = it.gears + response.items, nextCursor = response.nextCursor) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loadingMore = false) }
            }
        }
    }

    fun setTab(tab: GearTab) {
        _state.update { it.copy(tab = tab, selectedCategory = null, selectedStatus = null) }
        refresh()
    }

    fun setCategory(category: GearCategory?) {
        _state.update { it.copy(selectedCategory = category) }
        refresh()
    }

    fun setStatus(status: GearStatus?) {
        _state.update { it.copy(selectedStatus = status) }
        refresh()
    }

    fun setSort(sort: GearSort) {
        _state.update { it.copy(sort = sort) }
        refresh()
    }

    fun updateQuery(value: String) = _state.update { it.copy(query = value) }

    fun submitSearch() = refresh()

    fun archive(id: String) {
        viewModelScope.launch {
            try {
                repository.archive(id)
                refresh()
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            }
        }
    }

    fun restore(id: String) {
        viewModelScope.launch {
            try {
                repository.restore(id)
                refresh()
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            }
        }
    }

    private fun buildRequest(cursor: String?): ListGearsRequest {
        val current = _state.value
        return ListGearsRequest(
            tab = current.tab,
            category = current.selectedCategory,
            status = current.selectedStatus,
            query = current.query.trim().takeIf { it.isNotEmpty() },
            sort = current.sort,
            limit = 20,
            cursor = cursor,
        )
    }
}
