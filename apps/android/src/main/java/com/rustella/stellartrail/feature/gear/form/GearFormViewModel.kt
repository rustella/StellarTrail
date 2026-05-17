package com.rustella.stellartrail.feature.gear.form

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.rustella.stellartrail.core.network.userMessage
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearFormState
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.toCreateRequest
import com.rustella.stellartrail.domain.gear.toFormState
import com.rustella.stellartrail.domain.gear.toUpdateRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GearFormUiState(
    val id: String? = null,
    val form: GearFormState = GearFormState(),
    val loading: Boolean = false,
    val submitting: Boolean = false,
    val savedId: String? = null,
    val error: String? = null,
)

class GearFormViewModel(
    private val repository: GearRepositoryContract,
    private val id: String? = null,
) : ViewModel() {
    private val _state = MutableStateFlow(GearFormUiState(id = id))
    val state: StateFlow<GearFormUiState> = _state.asStateFlow()

    fun loadForEdit() {
        val gearId = id ?: return
        viewModelScope.launch {
            _state.update { it.copy(loading = true, error = null) }
            try {
                val gear = repository.get(gearId)
                _state.update { it.copy(form = gear.toFormState()) }
            } catch (throwable: Throwable) {
                _state.update { it.copy(error = throwable.userMessage()) }
            } finally {
                _state.update { it.copy(loading = false) }
            }
        }
    }

    fun updateName(value: String) = updateForm { copy(name = value) }
    fun updateCategory(value: GearCategory) = updateForm { copy(category = value) }
    fun updateStatus(value: GearStatus) = updateForm { copy(status = value) }
    fun updateBrand(value: String) = updateForm { copy(brand = value) }
    fun updateModel(value: String) = updateForm { copy(model = value) }
    fun updateColor(value: String) = updateForm { copy(color = value) }
    fun updateMaterial(value: String) = updateForm { copy(material = value) }
    fun updateCapacity(value: String) = updateForm { copy(capacity = value) }
    fun updateSize(value: String) = updateForm { copy(size = value) }
    fun updateDescription(value: String) = updateForm { copy(description = value) }
    fun updateWeight(value: String) = updateForm { copy(weightG = value) }
    fun updateWarmth(value: String) = updateForm { copy(warmthIndex = value) }
    fun updateWaterproof(value: String) = updateForm { copy(waterproofIndex = value) }
    fun updatePurchaseDate(value: String) = updateForm { copy(purchaseDate = value) }
    fun updatePrice(value: String) = updateForm { copy(purchasePrice = value) }
    fun updateWarrantyDate(value: String) = updateForm { copy(expiryOrWarrantyDate = value) }
    fun updatePurchaseLocation(value: String) = updateForm { copy(purchaseLocation = value) }
    fun updateStorageLocation(value: String) = updateForm { copy(storageLocation = value) }
    fun updateTags(value: String) = updateForm { copy(tags = value) }
    fun updateShareEnabled(value: Boolean) = updateForm { copy(shareEnabled = value) }
    fun updateNotes(value: String) = updateForm { copy(notes = value) }

    fun submit() {
        if (_state.value.submitting) return
        viewModelScope.launch {
            _state.update { it.copy(submitting = true, error = null, savedId = null) }
            try {
                val current = _state.value
                val item = if (current.id == null) {
                    repository.create(current.form.toCreateRequest())
                } else {
                    repository.update(current.id, current.form.toUpdateRequest())
                }
                _state.update { it.copy(savedId = item.id) }
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
