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
    val cachingAll: Boolean = false,
    val deletingAll: Boolean = false,
    val cachingKnots: Boolean = false,
    val clearingKnots: Boolean = false,
    val message: String? = null,
    val error: String? = null,
)

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

    fun cacheAllContent() {
        if (_state.value.isBusy()) return
        viewModelScope.launch {
            _state.update { it.copy(cachingAll = true, message = null, error = null) }
            runCatching { skillRepository.cacheAllKnots(SkillLocale.ZH_CN) }
                .onSuccess { status ->
                    _state.update {
                        it.copy(
                            status = status,
                            message = "已缓存所有可缓存内容，包含 ${status.cachedKnotCount} 个绳结。",
                        )
                    }
                }
                .onFailure { throwable ->
                    _state.update { it.copy(error = throwable.userMessage()) }
                }
            _state.update { it.copy(cachingAll = false) }
        }
    }

    fun deleteAllCaches() {
        if (_state.value.isBusy()) return
        viewModelScope.launch {
            _state.update { it.copy(deletingAll = true, message = null, error = null) }
            runCatching { skillRepository.clearKnotCache() }
                .onSuccess { status ->
                    _state.update {
                        it.copy(
                            status = status,
                            message = "已删除所有缓存。",
                        )
                    }
                }
                .onFailure { throwable ->
                    _state.update { it.copy(error = throwable.userMessage()) }
                }
            _state.update { it.copy(deletingAll = false) }
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

    private fun ProfileCacheUiState.isBusy(): Boolean = cachingAll || deletingAll || cachingKnots || clearingKnots
}
