package com.rustella.stellartrail.feature.skills

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.FavoriteKnotItem
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsRequest
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

enum class SkillsMode { Catalog, Knots, Favorites }

data class SkillsUiState(
    val mode: SkillsMode = SkillsMode.Catalog,
    val categories: List<SkillCategorySummary> = emptyList(),
    val knots: List<KnotSummary> = emptyList(),
    val favoriteKnots: List<FavoriteKnotItem> = emptyList(),
    val selectedKnot: KnotDetail? = null,
    val nextOffset: Int? = null,
    val favoriteNextOffset: Int? = null,
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
                val categoryItems = repository.listSkills(SkillLocale.ZH_CN).items
                _state.update {
                    it.copy(categories = categoryItems)
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun openCatalog() {
        _state.update { it.copy(mode = SkillsMode.Catalog, error = null) }
    }

    fun openKnots() {
        _state.update { it.copy(mode = SkillsMode.Knots, error = null) }
        if (_state.value.knots.isEmpty() && !_state.value.loading) {
            loadKnots()
        }
    }

    fun openFavorites() {
        _state.update { it.copy(mode = SkillsMode.Favorites, error = null) }
        if (_state.value.favoriteKnots.isEmpty() && !_state.value.loading) {
            loadFavoriteSkills()
        }
    }

    fun loadKnots() {
        if (_state.value.loading) return
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val response = repository.listKnots(SkillLocale.ZH_CN, ListKnotsRequest(limit = 20))
                _state.update { it.copy(knots = response.items, nextOffset = response.page.nextOffset) }
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

    fun loadFavoriteSkills() {
        if (_state.value.loading) return
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val response = repository.listFavoriteSkills(SkillLocale.ZH_CN, ListFavoriteSkillsRequest(limit = 20))
                _state.update {
                    it.copy(
                        favoriteKnots = response.items,
                        favoriteNextOffset = response.page.nextOffset,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun loadMoreFavoriteSkills() {
        val offset = _state.value.favoriteNextOffset ?: return
        if (_state.value.loadingMore) return
        viewModelScope.launch {
            _state.update { it.copy(loadingMore = true, error = null) }
            try {
                val response = repository.listFavoriteSkills(
                    SkillLocale.ZH_CN,
                    ListFavoriteSkillsRequest(offset = offset, limit = 20),
                )
                _state.update {
                    it.copy(
                        favoriteKnots = it.favoriteKnots + response.items,
                        favoriteNextOffset = response.page.nextOffset,
                    )
                }
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
