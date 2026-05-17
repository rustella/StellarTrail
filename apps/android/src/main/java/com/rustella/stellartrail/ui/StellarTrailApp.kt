package com.rustella.stellartrail.ui

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.ui.unit.dp
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.feature.auth.AuthViewModel
import com.rustella.stellartrail.feature.gear.detail.GearDetailViewModel
import com.rustella.stellartrail.feature.gear.form.GearFormViewModel
import com.rustella.stellartrail.feature.gear.list.GearListViewModel
import com.rustella.stellartrail.feature.home.HomeViewModel
import com.rustella.stellartrail.feature.profile.ProfileViewModel
import com.rustella.stellartrail.feature.skills.SkillsViewModel
import com.rustella.stellartrail.ui.common.viewModelFactory
import com.rustella.stellartrail.ui.navigation.AppRoutes
import com.rustella.stellartrail.ui.navigation.topLevelDestinations
import com.rustella.stellartrail.ui.screens.AuthScreen
import com.rustella.stellartrail.ui.screens.GearDetailScreen
import com.rustella.stellartrail.ui.screens.GearFormScreen
import com.rustella.stellartrail.ui.screens.GearListScreen
import com.rustella.stellartrail.ui.screens.HomeScreen
import com.rustella.stellartrail.ui.screens.ProfileScreen
import com.rustella.stellartrail.ui.screens.SkillsScreen

@Composable
fun StellarTrailApp(container: AppContainer, modifier: Modifier = Modifier) {
    val session by container.sessionStore.session.collectAsStateWithLifecycle()
    if (session == null) {
        val authViewModel: AuthViewModel = viewModel(
            factory = viewModelFactory { AuthViewModel(container.authRepository) },
        )
        AuthScreen(viewModel = authViewModel, modifier = modifier)
    } else {
        val navController = rememberNavController()
        AuthenticatedApp(container = container, navController = navController, modifier = modifier)
    }
}

@Composable
private fun AuthenticatedApp(
    container: AppContainer,
    navController: NavHostController,
    modifier: Modifier = Modifier,
) {
    val currentBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = currentBackStackEntry?.destination?.route
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        bottomBar = {
            if (currentRoute in topLevelDestinations.map { it.route }) {
                NavigationBar(
                    containerColor = MaterialTheme.colorScheme.surface,
                    tonalElevation = 8.dp,
                ) {
                    topLevelDestinations.forEach { destination ->
                        NavigationBarItem(
                            selected = currentRoute == destination.route,
                            onClick = {
                                navController.navigate(destination.route) {
                                    popUpTo(navController.graph.findStartDestination().id) { saveState = true }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            },
                            icon = { Icon(destination.icon, contentDescription = null) },
                            label = { Text(text = androidx.compose.ui.res.stringResource(destination.labelRes)) },
                        )
                    }
                }
            }
        },
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = AppRoutes.HOME,
            modifier = Modifier.padding(innerPadding),
        ) {
            composable(AppRoutes.HOME) {
                val viewModel: HomeViewModel = viewModel(factory = viewModelFactory {
                    HomeViewModel(container.gearRepository, container.skillRepository)
                })
                LaunchedEffect(Unit) { viewModel.load() }
                HomeScreen(
                    viewModel = viewModel,
                    onOpenGears = { navController.navigate(AppRoutes.GEARS) },
                    onOpenSkills = { navController.navigate(AppRoutes.SKILLS) },
                    onOpenGear = { navController.navigate(AppRoutes.gearDetail(it)) },
                )
            }
            composable(AppRoutes.GEARS) {
                val viewModel: GearListViewModel = viewModel(factory = viewModelFactory {
                    GearListViewModel(container.gearRepository)
                })
                LaunchedEffect(Unit) { viewModel.refresh() }
                GearListScreen(
                    viewModel = viewModel,
                    onOpenGear = { navController.navigate(AppRoutes.gearDetail(it)) },
                    onCreateGear = { navController.navigate(AppRoutes.GEAR_NEW) },
                )
            }
            composable(
                AppRoutes.GEAR_DETAIL,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                val viewModel: GearDetailViewModel = viewModel(
                    key = "gear-detail-$id",
                    factory = viewModelFactory { GearDetailViewModel(container.gearRepository, id) },
                )
                LaunchedEffect(id) { viewModel.load() }
                GearDetailScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onEdit = { navController.navigate(AppRoutes.gearEdit(id)) },
                    onClosed = { navController.popBackStack() },
                )
            }
            composable(AppRoutes.GEAR_NEW) {
                val viewModel: GearFormViewModel = viewModel(factory = viewModelFactory {
                    GearFormViewModel(container.gearRepository)
                })
                GearFormScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onSaved = { id ->
                        navController.navigate(AppRoutes.gearDetail(id)) {
                            popUpTo(AppRoutes.GEARS)
                        }
                    },
                )
            }
            composable(
                AppRoutes.GEAR_EDIT,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                val viewModel: GearFormViewModel = viewModel(
                    key = "gear-edit-$id",
                    factory = viewModelFactory { GearFormViewModel(container.gearRepository, id) },
                )
                LaunchedEffect(id) { viewModel.loadForEdit() }
                GearFormScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onSaved = { savedId -> navController.navigate(AppRoutes.gearDetail(savedId)) { popUpTo(AppRoutes.GEARS) } },
                )
            }
            composable(AppRoutes.SKILLS) {
                val viewModel: SkillsViewModel = viewModel(factory = viewModelFactory {
                    SkillsViewModel(container.skillRepository)
                })
                LaunchedEffect(Unit) { viewModel.load() }
                SkillsScreen(viewModel = viewModel)
            }
            composable(AppRoutes.PROFILE) {
                val viewModel: ProfileViewModel = viewModel(factory = viewModelFactory {
                    ProfileViewModel(container.authRepository, container.themeRepository, container.configStore)
                })
                ProfileScreen(viewModel = viewModel)
            }
        }
    }
}
