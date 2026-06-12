package com.rustella.stellartrail.core.map

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

interface MapStylePreferenceRepository {
    val selectedStyleId: StateFlow<String?>

    fun selectStyle(styleId: String)
}

class AndroidMapStylePreferenceRepository(context: Context) : MapStylePreferenceRepository {
    private val preferences = context.applicationContext.getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE)
    private val selectedStyle = MutableStateFlow(preferences.getString(KEY_SELECTED_STYLE_ID, null))

    override val selectedStyleId: StateFlow<String?> = selectedStyle.asStateFlow()

    override fun selectStyle(styleId: String) {
        val normalized = styleId.trim().takeIf { it.isNotEmpty() } ?: return
        preferences.edit().putString(KEY_SELECTED_STYLE_ID, normalized).apply()
        selectedStyle.value = normalized
    }

    private companion object {
        const val PREFERENCES_NAME = "stellartrail_map"
        const val KEY_SELECTED_STYLE_ID = "selected_style_id"
    }
}

class InMemoryMapStylePreferenceRepository(initialStyleId: String? = null) : MapStylePreferenceRepository {
    private val selectedStyle = MutableStateFlow(initialStyleId)

    override val selectedStyleId: StateFlow<String?> = selectedStyle.asStateFlow()

    override fun selectStyle(styleId: String) {
        selectedStyle.value = styleId.trim().takeIf { it.isNotEmpty() } ?: selectedStyle.value
    }
}
