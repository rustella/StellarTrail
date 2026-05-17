package com.rustella.stellartrail.feature.home

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.GearSort
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class HomeUiState(
    val loading: Boolean = false,
    val error: String? = null,
    val stats: GearStatsResponse = EMPTY_STATS,
    val recentGears: List<GearSummary> = emptyList(),
    val skills: List<SkillCategorySummary> = emptyList(),
)

class HomeViewModel(
    private val gearRepository: GearRepositoryContract,
    private val skillRepository: SkillRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(HomeUiState())
    val state: StateFlow<HomeUiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val stats = async { gearRepository.stats(GearTab.AVAILABLE) }
                val gears = async {
                    gearRepository.list(
                        ListGearsRequest(
                            tab = GearTab.AVAILABLE,
                            limit = 2,
                            sort = GearSort.CREATED_AT_DESC,
                        ),
                    )
                }
                val skills = async { skillRepository.listSkills() }
                _state.update {
                    it.copy(
                        stats = stats.await(),
                        recentGears = gears.await().items,
                        skills = skills.await().items.take(3),
                    )
                }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }
}

val EMPTY_STATS = GearStatsResponse(
    currentCount = 0,
    archivedCount = 0,
    totalValueCents = 0,
    totalWeightG = 0,
)
