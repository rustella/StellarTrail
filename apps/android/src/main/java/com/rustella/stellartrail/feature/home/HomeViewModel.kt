package com.rustella.stellartrail.feature.home

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.domain.gear.GearSort
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.GearTemplate
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope

data class HomeUiState(
    val isLoggedIn: Boolean = false,
    val loading: Boolean = false,
    val error: String? = null,
    val stats: GearStatsResponse = EMPTY_STATS,
    val recentGears: List<GearSummary> = emptyList(),
    val templates: List<GearTemplate> = emptyList(),
    val skills: List<SkillCategorySummary> = emptyList(),
)

class HomeViewModel(
    private val gearRepository: GearRepositoryContract,
    private val skillRepository: SkillRepositoryContract,
) : ViewModel() {
    private val _state = MutableStateFlow(HomeUiState())
    val state: StateFlow<HomeUiState> = _state.asStateFlow()

    fun load(isLoggedIn: Boolean = true) {
        viewModelScope.launch {
            _state.update { it.copy(isLoggedIn = isLoggedIn, loading = true, error = null) }
            try {
                supervisorScope {
                    val templates = async { gearRepository.listTemplates() }
                    val skills = async { skillRepository.listSkills() }
                    if (isLoggedIn) {
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
                        val templateItems = templates.await().items.take(2)
                        val skillItems = skills.await().items.take(3)
                        val statsValue = stats.await()
                        val recentGearItems = gears.await().items
                        _state.update {
                            it.copy(
                                stats = statsValue,
                                recentGears = recentGearItems,
                                templates = templateItems,
                                skills = skillItems,
                            )
                        }
                    } else {
                        val templateItems = templates.await().items.take(2)
                        val skillItems = skills.await().items.take(3)
                        _state.update {
                            it.copy(
                                stats = EMPTY_STATS,
                                recentGears = emptyList(),
                                templates = templateItems,
                                skills = skillItems,
                            )
                        }
                    }
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
