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
    val caching: Boolean = false,
    val clearing: Boolean = false,
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

    fun cacheAllKnots() {
        if (_state.value.caching || _state.value.clearing) return
        viewModelScope.launch {
            _state.update { it.copy(caching = true, message = null, error = null) }
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
            _state.update { it.copy(caching = false) }
        }
    }

    fun clearCache() {
        if (_state.value.caching || _state.value.clearing) return
        viewModelScope.launch {
            _state.update { it.copy(clearing = true, message = null, error = null) }
            runCatching { skillRepository.clearKnotCache() }
                .onSuccess { status ->
                    _state.update {
                        it.copy(
                            status = status,
                            message = "缓存已清空。",
                        )
                    }
                }
                .onFailure { throwable ->
                    _state.update { it.copy(error = throwable.userMessage()) }
                }
            _state.update { it.copy(clearing = false) }
        }
    }
}
