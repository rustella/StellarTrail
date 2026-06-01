package com.rustella.stellartrail.feature.home

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.data.trip.TripRepositoryContract
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.GearTemplate
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import java.time.LocalDate

data class HomeUiState(
    val isLoggedIn: Boolean = false,
    val loading: Boolean = false,
    val error: String? = null,
    val stats: GearStatsResponse = EMPTY_STATS,
    val recentGears: List<GearSummary> = emptyList(),
    val templates: List<GearTemplate> = emptyList(),
    val skills: List<SkillCategorySummary> = emptyList(),
    val featuredKnots: List<KnotSummary> = emptyList(),
    val tripHighlight: TripHomeHighlightItem? = null,
)

class HomeViewModel(
    private val gearRepository: GearRepositoryContract,
    private val skillRepository: SkillRepositoryContract,
    private val tripRepository: TripRepositoryContract? = null,
) : ViewModel() {
    private val _state = MutableStateFlow(HomeUiState())
    val state: StateFlow<HomeUiState> = _state.asStateFlow()

    fun load(isLoggedIn: Boolean = true) {
        viewModelScope.launch {
            _state.update { it.copy(isLoggedIn = isLoggedIn, loading = true, error = null) }
            try {
                supervisorScope {
                    if (isLoggedIn) {
                        val stats = async { gearRepository.stats(GearTab.AVAILABLE) }
                        val tripHighlight = async { tripRepository?.homeHighlight(LocalDate.now().toString())?.item }
                        val featuredKnots = async { skillRepository.listKnots(request = ListKnotsRequest(limit = 3)).items }
                        val statsValue = stats.await()
                        val tripHighlightValue = tripHighlight.await()
                        val featuredKnotItems = featuredKnots.await()
                        _state.update {
                            it.copy(
                                stats = statsValue,
                                recentGears = emptyList(),
                                templates = emptyList(),
                                skills = emptyList(),
                                featuredKnots = featuredKnotItems,
                                tripHighlight = tripHighlightValue,
                            )
                        }
                    } else {
                        _state.update {
                            it.copy(
                                stats = EMPTY_STATS,
                                recentGears = emptyList(),
                                templates = emptyList(),
                                skills = emptyList(),
                                featuredKnots = emptyList(),
                                tripHighlight = null,
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
