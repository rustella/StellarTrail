package com.rustella.stellartrail.core.theme

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

enum class ThemeMode {
    LIGHT,
    DARK,
    SYSTEM,
}

interface ThemeRepository {
    val theme: StateFlow<ThemeMode>
    fun setTheme(theme: ThemeMode)
}

class AndroidThemeRepository(context: Context) : ThemeRepository {
    private val preferences = context.getSharedPreferences("stellartrail_theme", Context.MODE_PRIVATE)
    private val _theme = MutableStateFlow(load())
    override val theme: StateFlow<ThemeMode> = _theme.asStateFlow()

    override fun setTheme(theme: ThemeMode) {
        preferences.edit().putString(KEY_THEME, theme.name).apply()
        _theme.value = theme
    }

    private fun load(): ThemeMode = preferences.getString(KEY_THEME, ThemeMode.SYSTEM.name)
        ?.let { runCatching { ThemeMode.valueOf(it) }.getOrNull() }
        ?: ThemeMode.SYSTEM

    private companion object {
        const val KEY_THEME = "theme"
    }
}

class InMemoryThemeRepository(initial: ThemeMode = ThemeMode.SYSTEM) : ThemeRepository {
    private val _theme = MutableStateFlow(initial)
    override val theme: StateFlow<ThemeMode> = _theme.asStateFlow()
    override fun setTheme(theme: ThemeMode) {
        _theme.value = theme
    }
}
