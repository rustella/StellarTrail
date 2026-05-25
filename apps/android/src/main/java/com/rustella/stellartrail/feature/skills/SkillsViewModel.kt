package com.rustella.stellartrail.feature.skills

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope

data class SkillsUiState(
    val categories: List<SkillCategorySummary> = emptyList(),
    val knots: List<KnotSummary> = emptyList(),
    val selectedKnot: KnotDetail? = null,
    val nextOffset: Int? = null,
    val loading: Boolean = false,
    val loadingMore: Boolean = false,
    val error: String? = null,
)

class SkillsViewModel(private val repository: SkillRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(SkillsUiState())
    val state: StateFlow<SkillsUiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                supervisorScope {
                    val categories = async { repository.listSkills(SkillLocale.ZH_CN) }
                    val knots = async { repository.listKnots(SkillLocale.ZH_CN, ListKnotsRequest(limit = 20)) }
                    val categoryItems = categories.await().items
                    val knotResponse = knots.await()
                    _state.update {
                        it.copy(
                            categories = categoryItems,
                            knots = knotResponse.items,
                            nextOffset = knotResponse.page.nextOffset,
                        )
                    }
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun loadMoreKnots() {
        val offset = _state.value.nextOffset ?: return
        if (_state.value.loadingMore) return
        viewModelScope.launch {
            _state.update { it.copy(loadingMore = true, error = null) }
            try {
                val response = repository.listKnots(SkillLocale.ZH_CN, ListKnotsRequest(offset = offset, limit = 20))
                _state.update { it.copy(knots = it.knots + response.items, nextOffset = response.page.nextOffset) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loadingMore = false) }
            }
        }
    }

    fun openKnot(id: String) {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                _state.update { it.copy(selectedKnot = repository.knotDetail(id, SkillLocale.ZH_CN)) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun closeKnot() {
        _state.update { it.copy(selectedKnot = null) }
    }
}
