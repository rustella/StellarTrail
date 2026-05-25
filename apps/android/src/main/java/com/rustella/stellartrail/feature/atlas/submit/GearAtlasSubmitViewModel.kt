package com.rustella.stellartrail.feature.atlas.submit

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearFormState
import com.rustella.stellartrail.domain.gear.toAtlasSubmissionRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearAtlasSubmitUiState(
    val isLoggedIn: Boolean = false,
    val form: GearFormState = GearFormState(),
    val submitting: Boolean = false,
    val submitted: Boolean = false,
    val error: String? = null,
)

class GearAtlasSubmitViewModel(private val repository: GearAtlasRepositoryContract) : ViewModel() {
    private val _state = MutableStateFlow(GearAtlasSubmitUiState())
    val state: StateFlow<GearAtlasSubmitUiState> = _state.asStateFlow()

    fun setLoggedIn(isLoggedIn: Boolean) {
        _state.update { it.copy(isLoggedIn = isLoggedIn) }
    }

    fun updateName(value: String) = updateForm { copy(name = value) }
    fun updateCategory(value: GearCategory) = updateForm { copy(category = value) }
    fun updateBrand(value: String) = updateForm { copy(brand = value) }
    fun updateModel(value: String) = updateForm { copy(model = value) }
    fun updateDescription(value: String) = updateForm { copy(description = value) }
    fun updateWeight(value: String) = updateForm { copy(weightG = value) }
    fun updateOfficialPrice(value: String) = updateForm { copy(officialPrice = value) }
    fun updateColor(value: String) = updateForm { copy(color = value) }
    fun updateMaterial(value: String) = updateForm { copy(material = value) }
    fun updateCapacity(value: String) = updateForm { copy(capacity = value) }
    fun updateSize(value: String) = updateForm { copy(size = value) }

    fun submit() {
        if (_state.value.submitting || !_state.value.isLoggedIn) return
        viewModelScope.launch {
            _state.update { it.copy(submitting = true, submitted = false, error = null) }
            try {
                repository.createSubmission(_state.value.form.toAtlasSubmissionRequest())
                _state.update { it.copy(submitted = true) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(submitting = false) }
            }
        }
    }

    private fun updateForm(mutator: GearFormState.() -> GearFormState) {
        _state.update { it.copy(form = it.form.mutator(), error = null) }
    }
}
