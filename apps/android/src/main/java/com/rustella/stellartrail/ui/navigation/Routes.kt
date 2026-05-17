package com.rustella.stellartrail.ui.navigation

import androidx.annotation.StringRes
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.School
import androidx.compose.ui.graphics.vector.ImageVector
import com.rustella.stellartrail.R

object AppRoutes {
    const val AUTH = "auth"
    const val HOME = "home"
    const val GEARS = "gears"
    const val GEAR_DETAIL = "gear/{id}"
    const val GEAR_NEW = "gear/new"
    const val GEAR_EDIT = "gear/{id}/edit"
    const val SKILLS = "skills"
    const val PROFILE = "profile"

    fun gearDetail(id: String): String = "gear/$id"
    fun gearEdit(id: String): String = "gear/$id/edit"
}

data class TopLevelDestination(
    val route: String,
    @StringRes val labelRes: Int,
    val icon: ImageVector,
)

val topLevelDestinations = listOf(
    TopLevelDestination(AppRoutes.HOME, R.string.nav_home, Icons.Filled.Home),
    TopLevelDestination(AppRoutes.GEARS, R.string.nav_gears, Icons.AutoMirrored.Filled.List),
    TopLevelDestination(AppRoutes.SKILLS, R.string.nav_skills, Icons.Filled.School),
    TopLevelDestination(AppRoutes.PROFILE, R.string.nav_profile, Icons.Filled.Person),
)
