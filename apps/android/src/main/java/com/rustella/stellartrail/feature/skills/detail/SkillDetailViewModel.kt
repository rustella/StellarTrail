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
)

class SkillDetailViewModel(
    private val repository: SkillRepositoryContract,
    private val id: String,
) : ViewModel() {
    private val _state = MutableStateFlow(SkillDetailUiState())
    val state: StateFlow<SkillDetailUiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                _state.update { it.copy(detail = repository.knotDetail(id, SkillLocale.ZH_CN)) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }
}
