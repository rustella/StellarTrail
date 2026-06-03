package com.rustella.stellartrail.feature.skills.detail

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class SkillDetailUiState(
    val detail: KnotDetail? = null,
    val loading: Boolean = false,
    val error: String? = null,
    val isFavorited: Boolean = false,
    val favoriteLoading: Boolean = false,
    val favoritedAt: String? = null,
    val actionError: String? = null,
)

class SkillDetailViewModel(
    private val repository: SkillRepositoryContract,
    private val id: String,
) : ViewModel() {
    private val _state = MutableStateFlow(SkillDetailUiState())
    val state: StateFlow<SkillDetailUiState> = _state.asStateFlow()

    fun load(isLoggedIn: Boolean = true) {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null, actionError = null) }
            try {
                val detail = repository.knotDetail(id, SkillLocale.ZH_CN)
                val favoriteStatus = if (isLoggedIn) {
                    runCatching { repository.getFavoriteKnotStatus(id) }.getOrNull()
                } else {
                    null
                }
                _state.update {
                    it.copy(
                        detail = detail,
                        isFavorited = favoriteStatus?.isFavorited ?: false,
                        favoritedAt = favoriteStatus?.favoritedAt,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun toggleFavorite() {
        val current = _state.value
        val detail = current.detail ?: return
        if (current.favoriteLoading) return
        viewModelScope.launch {
            val previousFavorited = _state.value.isFavorited
            val previousFavoritedAt = _state.value.favoritedAt
            _state.update { it.copy(favoriteLoading = true, actionError = null) }
            try {
                val status = if (previousFavorited) {
                    repository.unfavoriteKnot(detail.id)
                } else {
                    repository.favoriteKnot(detail.id)
                }
                _state.update {
                    it.copy(
                        isFavorited = status.isFavorited,
                        favoritedAt = status.favoritedAt,
                        favoriteLoading = false,
                    )
                }
            } catch (throwable: Throwable) {
                _state.update {
                    it.copy(
                        isFavorited = previousFavorited,
                        favoritedAt = previousFavoritedAt,
                        favoriteLoading = false,
                        actionError = throwable.userMessage(),
                    )
                }
            }
        }
    }

    fun resolveMediaUrl(pathOrUrl: String): String = repository.resolveMediaUrl(pathOrUrl)
}
