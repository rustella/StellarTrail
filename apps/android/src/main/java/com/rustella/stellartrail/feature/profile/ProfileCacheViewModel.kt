package com.rustella.stellartrail.feature.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.skills.KnotCacheStatus
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class ProfileCacheUiState(
    val status: KnotCacheStatus = KnotCacheStatus(),
    val selectionMode: Boolean = false,
    val selectedCacheKinds: Set<ProfileCacheKind> = emptySet(),
    val cachingSelected: Boolean = false,
    val deletingSelected: Boolean = false,
    val cachingKnots: Boolean = false,
    val clearingKnots: Boolean = false,
    val message: String? = null,
    val error: String? = null,
)

enum class ProfileCacheKind {
    Knots,
}

class ProfileCacheViewModel(
    private val skillRepository: SkillRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(ProfileCacheUiState(status = skillRepository.knotCacheStatus.value))
    val state: StateFlow<ProfileCacheUiState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            skillRepository.knotCacheStatus.collect { status ->
                _state.update { it.copy(status = status) }
            }
        }
    }

    fun enterSelectionMode() {
        if (_state.value.isBusy()) return
        _state.update { it.copy(selectionMode = true, selectedCacheKinds = emptySet(), message = null, error = null) }
    }

    fun exitSelectionMode() {
        if (_state.value.isBusy()) return
        _state.update { it.copy(selectionMode = false, selectedCacheKinds = emptySet()) }
    }

    fun toggleCacheKind(kind: ProfileCacheKind) {
        if (_state.value.isBusy()) return
        _state.update { state ->
            val selected = if (kind in state.selectedCacheKinds) {
                state.selectedCacheKinds - kind
            } else {
                state.selectedCacheKinds + kind
            }
            state.copy(selectedCacheKinds = selected, message = null, error = null)
        }
    }

    fun selectAllCacheKinds() {
        if (_state.value.isBusy()) return
        _state.update { it.copy(selectedCacheKinds = ProfileCacheKind.entries.toSet(), message = null, error = null) }
    }

    fun invertCacheSelection() {
        if (_state.value.isBusy()) return
        _state.update { state ->
            state.copy(
                selectedCacheKinds = ProfileCacheKind.entries.toSet() - state.selectedCacheKinds,
                message = null,
                error = null,
            )
        }
    }

    fun cacheSelectedCaches() {
        val selected = _state.value.selectedCacheKinds
        if (_state.value.isBusy() || selected.isEmpty()) return
        viewModelScope.launch {
            _state.update { it.copy(cachingSelected = true, message = null, error = null) }
            runCatching {
                var status = _state.value.status
                if (ProfileCacheKind.Knots in selected) {
                    status = skillRepository.cacheAllKnots(SkillLocale.ZH_CN)
                }
                status
            }.onSuccess { status ->
                _state.update {
                    it.copy(
                        status = status,
                        message = "已缓存选中内容，包含 ${status.cachedKnotCount} 个绳结。",
                    )
                }
            }.onFailure { throwable ->
                _state.update { it.copy(error = throwable.userMessage()) }
            }
            _state.update { it.copy(cachingSelected = false) }
        }
    }

    fun deleteSelectedCaches() {
        val selected = _state.value.selectedCacheKinds
        if (_state.value.isBusy() || selected.isEmpty()) return
        viewModelScope.launch {
            _state.update { it.copy(deletingSelected = true, message = null, error = null) }
            runCatching {
                var status = _state.value.status
                if (ProfileCacheKind.Knots in selected) {
                    status = skillRepository.clearKnotCache()
                }
                status
            }.onSuccess { status ->
                _state.update {
                    it.copy(
                        status = status,
                        message = "已删除选中缓存。",
                    )
                }
            }.onFailure { throwable ->
                _state.update { it.copy(error = throwable.userMessage()) }
            }
            _state.update { it.copy(deletingSelected = false) }
        }
    }

    fun cacheKnots() {
        if (_state.value.isBusy()) return
        viewModelScope.launch {
            _state.update { it.copy(cachingKnots = true, message = null, error = null) }
            runCatching { skillRepository.cacheAllKnots(SkillLocale.ZH_CN) }
                .onSuccess { status ->
                    _state.update {
                        it.copy(
                            status = status,
                            message = "已缓存 ${status.cachedKnotCount} 个绳结。",
                        )
                    }
                }
                .onFailure { throwable ->
                    _state.update { it.copy(error = throwable.userMessage()) }
                }
            _state.update { it.copy(cachingKnots = false) }
        }
    }

    fun clearKnotCache() {
        if (_state.value.isBusy()) return
        viewModelScope.launch {
            _state.update { it.copy(clearingKnots = true, message = null, error = null) }
            runCatching { skillRepository.clearKnotCache() }
                .onSuccess { status ->
                    _state.update {
                        it.copy(
                            status = status,
                            message = "绳结缓存已清空。",
                        )
                    }
                }
                .onFailure { throwable ->
                    _state.update { it.copy(error = throwable.userMessage()) }
                }
            _state.update { it.copy(clearingKnots = false) }
        }
    }

    private fun ProfileCacheUiState.isBusy(): Boolean = cachingSelected || deletingSelected || cachingKnots || clearingKnots
}
