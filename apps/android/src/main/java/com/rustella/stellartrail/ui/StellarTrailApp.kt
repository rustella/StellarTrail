package com.rustella.stellartrail.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
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
import com.rustella.stellartrail.feature.atlas.detail.GearAtlasDetailViewModel
import com.rustella.stellartrail.feature.atlas.list.GearAtlasListViewModel
import com.rustella.stellartrail.feature.atlas.submit.GearAtlasSubmitViewModel
import com.rustella.stellartrail.feature.auth.AuthMode
import com.rustella.stellartrail.feature.auth.AuthViewModel
import com.rustella.stellartrail.feature.gear.detail.GearDetailViewModel
import com.rustella.stellartrail.feature.gear.form.GearFormViewModel
import com.rustella.stellartrail.feature.gear.list.GearListViewModel
import com.rustella.stellartrail.feature.home.HomeViewModel
import com.rustella.stellartrail.feature.packing.PackingViewModel
import com.rustella.stellartrail.feature.profile.OutdoorExperiencesViewModel
import com.rustella.stellartrail.feature.profile.OutdoorProfileViewModel
import com.rustella.stellartrail.feature.profile.ProfileSettingsViewModel
import com.rustella.stellartrail.feature.profile.ProfileViewModel
import com.rustella.stellartrail.feature.profile.RoadmapViewModel
import com.rustella.stellartrail.feature.skills.detail.SkillDetailViewModel
import com.rustella.stellartrail.feature.skills.SkillsViewModel
import com.rustella.stellartrail.feature.trips.TripDetailViewModel
import com.rustella.stellartrail.feature.trips.TripFormViewModel
import com.rustella.stellartrail.feature.trips.TripJoinViewModel
import com.rustella.stellartrail.feature.trips.TripListViewModel
import com.rustella.stellartrail.ui.common.currentTrailPalette
import com.rustella.stellartrail.ui.common.viewModelFactory
import com.rustella.stellartrail.ui.navigation.AppRoutes
import com.rustella.stellartrail.ui.navigation.TopLevelDestination
import com.rustella.stellartrail.ui.navigation.topLevelDestinations
import com.rustella.stellartrail.ui.screens.AuthScreen
import com.rustella.stellartrail.ui.screens.GearAtlasDetailScreen
import com.rustella.stellartrail.ui.screens.GearAtlasListScreen
import com.rustella.stellartrail.ui.screens.GearAtlasSubmitScreen
import com.rustella.stellartrail.ui.screens.GearDetailScreen
import com.rustella.stellartrail.ui.screens.GearFormScreen
import com.rustella.stellartrail.ui.screens.GearListScreen
import com.rustella.stellartrail.ui.screens.HomeScreen
import com.rustella.stellartrail.ui.screens.LoginRequiredScreen
import com.rustella.stellartrail.ui.screens.OutdoorExperiencesScreen
import com.rustella.stellartrail.ui.screens.OutdoorProfileScreen
import com.rustella.stellartrail.ui.screens.PackingListsScreen
import com.rustella.stellartrail.ui.screens.ProfileAboutScreen
import com.rustella.stellartrail.ui.screens.ProfileSettingsScreen
import com.rustella.stellartrail.ui.screens.RoadmapScreen
import com.rustella.stellartrail.ui.screens.TripDetailScreen
import com.rustella.stellartrail.ui.screens.TripFormScreen
import com.rustella.stellartrail.ui.screens.TripJoinScreen
import com.rustella.stellartrail.ui.screens.TripsScreen
import com.rustella.stellartrail.ui.screens.ProfileScreen
import com.rustella.stellartrail.ui.screens.SkillDetailScreen
import com.rustella.stellartrail.ui.screens.SkillsScreen
import com.rustella.stellartrail.domain.trip.TripType

@Composable
fun StellarTrailApp(
    container: AppContainer,
    modifier: Modifier = Modifier,
    startDestination: String = AppRoutes.HOME,
) {
    val session by container.sessionStore.session.collectAsStateWithLifecycle()
    val navController = rememberNavController()
    AuthenticatedApp(
        container = container,
        isLoggedIn = session != null,
        navController = navController,
        startDestination = startDestination,
        modifier = modifier,
    )
}

@Composable
private fun AuthenticatedApp(
    container: AppContainer,
    isLoggedIn: Boolean,
    navController: NavHostController,
    startDestination: String,
    modifier: Modifier = Modifier,
) {
    val currentBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = currentBackStackEntry?.destination?.route
    LaunchedEffect(isLoggedIn, currentRoute) {
        if (isLoggedIn && currentRoute in listOf(AppRoutes.AUTH, AppRoutes.AUTH_REGISTER)) {
            navController.navigate(AppRoutes.HOME) {
                popUpTo(AppRoutes.AUTH) { inclusive = true }
                launchSingleTop = true
            }
        }
    }
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            if (currentRoute in topLevelDestinations.map { it.route }) {
                MiniProgramTopBar(title = miniProgramTopBarTitle(currentRoute))
            }
        },
        bottomBar = {
            if (currentRoute in topLevelDestinations.map { it.route }) {
                val palette = currentTrailPalette()
                NavigationBar(
                    containerColor = palette.footerBackground,
                    tonalElevation = 0.dp,
                ) {
                    topLevelDestinations.forEach { destination ->
                        MiniProgramBottomNavItem(
                            destination = destination,
                            selected = currentRoute == destination.route,
                            onClick = {
                                navController.navigate(destination.route) {
                                    popUpTo(navController.graph.findStartDestination().id) { saveState = true }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            },
                        )
                    }
                }
            }
        },
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = startDestination,
            modifier = Modifier.padding(innerPadding),
        ) {
            composable(AppRoutes.AUTH) {
                val authViewModel: AuthViewModel = viewModel(
                    factory = viewModelFactory { AuthViewModel(container.authRepository) },
                )
                AuthScreen(viewModel = authViewModel)
            }
            composable(AppRoutes.AUTH_REGISTER) {
                val authViewModel: AuthViewModel = viewModel(
                    factory = viewModelFactory { AuthViewModel(container.authRepository, AuthMode.REGISTER) },
                )
                AuthScreen(viewModel = authViewModel)
            }
            composable(AppRoutes.HOME) {
                val viewModel: HomeViewModel = viewModel(factory = viewModelFactory {
                    HomeViewModel(container.gearRepository, container.skillRepository, container.tripRepository)
                })
                LaunchedEffect(isLoggedIn) { viewModel.load(isLoggedIn) }
                HomeScreen(
                    viewModel = viewModel,
                    onOpenGears = { navController.navigate(AppRoutes.GEARS) },
                    onCreateGear = {
                        if (isLoggedIn) navController.navigate(AppRoutes.GEAR_NEW) else navController.navigate(AppRoutes.AUTH)
                    },
                    onOpenSkills = { navController.navigate(AppRoutes.SKILLS) },
                    onOpenTrips = { navController.navigate(AppRoutes.TRIPS) },
                    onOpenTrip = { id ->
                        if (isLoggedIn) navController.navigate(AppRoutes.tripDetail(id)) else navController.navigate(AppRoutes.AUTH)
                    },
                    onOpenProfile = { navController.navigate(AppRoutes.PROFILE) },
                    onOpenGear = { id ->
                        if (isLoggedIn) navController.navigate(AppRoutes.gearDetail(id)) else navController.navigate(AppRoutes.AUTH)
                    },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                )
            }
            composable(AppRoutes.GEARS) {
                val viewModel: GearListViewModel = viewModel(factory = viewModelFactory {
                    GearListViewModel(container.gearRepository)
                })
                LaunchedEffect(isLoggedIn) { viewModel.refresh(isLoggedIn) }
                GearListScreen(
                    viewModel = viewModel,
                    onOpenGear = { id ->
                        if (isLoggedIn) navController.navigate(AppRoutes.gearDetail(id)) else navController.navigate(AppRoutes.AUTH)
                    },
                    onCreateGear = {
                        if (isLoggedIn) navController.navigate(AppRoutes.GEAR_NEW) else navController.navigate(AppRoutes.AUTH)
                    },
                    onOpenAtlas = { navController.navigate(AppRoutes.GEAR_ATLAS) },
                    onOpenPackingLists = {
                        if (isLoggedIn) navController.navigate(AppRoutes.PACKING_LISTS) else navController.navigate(AppRoutes.AUTH)
                    },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                )
            }
            composable(AppRoutes.PACKING_LISTS) {
                val viewModel: PackingViewModel = viewModel(factory = viewModelFactory {
                    PackingViewModel(container.packingRepository)
                })
                PackingListsScreen(
                    viewModel = viewModel,
                    isLoggedIn = isLoggedIn,
                    onBack = { navController.popBackStack() },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                )
            }
            composable(AppRoutes.GEAR_ATLAS) {
                val viewModel: GearAtlasListViewModel = viewModel(factory = viewModelFactory {
                    GearAtlasListViewModel(container.gearAtlasRepository)
                })
                LaunchedEffect(Unit) { viewModel.refresh() }
                GearAtlasListScreen(
                    viewModel = viewModel,
                    onOpenItem = { id -> navController.navigate(AppRoutes.gearAtlasDetail(id)) },
                    onSubmit = {
                        if (isLoggedIn) navController.navigate(AppRoutes.GEAR_ATLAS_SUBMIT) else navController.navigate(AppRoutes.AUTH)
                    },
                )
            }
            composable(
                AppRoutes.GEAR_ATLAS_DETAIL,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                val viewModel: GearAtlasDetailViewModel = viewModel(
                    key = "gear-atlas-detail-$id",
                    factory = viewModelFactory { GearAtlasDetailViewModel(container.gearAtlasRepository, id) },
                )
                LaunchedEffect(id) { viewModel.load() }
                GearAtlasDetailScreen(viewModel = viewModel, onBack = { navController.popBackStack() })
            }
            composable(AppRoutes.GEAR_ATLAS_SUBMIT) {
                val viewModel: GearAtlasSubmitViewModel = viewModel(factory = viewModelFactory {
                    GearAtlasSubmitViewModel(container.gearAtlasRepository)
                })
                GearAtlasSubmitScreen(
                    viewModel = viewModel,
                    isLoggedIn = isLoggedIn,
                    onBack = { navController.popBackStack() },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                    onSubmitted = {
                        navController.navigate(AppRoutes.GEAR_ATLAS) {
                            popUpTo(AppRoutes.GEAR_ATLAS)
                            launchSingleTop = true
                        }
                    },
                )
            }
            composable(
                AppRoutes.GEAR_DETAIL,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                if (!isLoggedIn) {
                    LoginRequiredScreen(
                        title = "登录后查看装备详情",
                        body = "这是你的个人装备记录，登录后可继续查看和编辑。",
                        onLogin = { navController.navigate(AppRoutes.AUTH) },
                    )
                    return@composable
                }
                val viewModel: GearDetailViewModel = viewModel(
                    key = "gear-detail-$id",
                    factory = viewModelFactory { GearDetailViewModel(container.gearRepository, container.gearAtlasRepository, id) },
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
                if (!isLoggedIn) {
                    LoginRequiredScreen(
                        title = "登录后添加装备",
                        body = "添加或修改装备会保存到你的个人清单，请先登录。",
                        onLogin = { navController.navigate(AppRoutes.AUTH) },
                    )
                    return@composable
                }
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
                if (!isLoggedIn) {
                    LoginRequiredScreen(
                        title = "登录后编辑装备",
                        body = "添加或修改装备会保存到你的个人清单，请先登录。",
                        onLogin = { navController.navigate(AppRoutes.AUTH) },
                    )
                    return@composable
                }
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
                SkillsScreen(viewModel = viewModel, onOpenKnot = { id -> navController.navigate(AppRoutes.skillDetail(id)) })
            }
            composable(AppRoutes.TRIPS) {
                val viewModel: TripListViewModel = viewModel(factory = viewModelFactory {
                    TripListViewModel(container.tripRepository)
                })
                TripsScreen(
                    viewModel = viewModel,
                    isLoggedIn = isLoggedIn,
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                    onCreateTrip = { type -> navController.navigate(AppRoutes.tripNew(type.name.lowercase())) },
                    onJoinTrip = {
                        if (isLoggedIn) navController.navigate(AppRoutes.TRIP_JOIN) else navController.navigate(AppRoutes.AUTH)
                    },
                    onOpenTrip = { id ->
                        if (isLoggedIn) navController.navigate(AppRoutes.tripDetail(id)) else navController.navigate(AppRoutes.AUTH)
                    },
                )
            }
            composable(
                AppRoutes.TRIP_NEW,
                arguments = listOf(navArgument("type") { type = NavType.StringType }),
            ) { backStackEntry ->
                if (!isLoggedIn) {
                    LoginRequiredScreen(
                        title = "登录后制作行程",
                        body = "行程计划和协作分工会保存到账号中，请先登录。",
                        onLogin = { navController.navigate(AppRoutes.AUTH) },
                    )
                    return@composable
                }
                val typeValue = backStackEntry.arguments?.getString("type")
                val tripType = if (typeValue == "team") TripType.TEAM else TripType.SOLO
                val viewModel: TripFormViewModel = viewModel(
                    key = "trip-new-$typeValue",
                    factory = viewModelFactory { TripFormViewModel(container.tripRepository, tripType = tripType) },
                )
                TripFormScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onSaved = { id ->
                        navController.navigate(AppRoutes.tripDetail(id)) {
                            popUpTo(AppRoutes.TRIPS)
                        }
                    },
                )
            }
            composable(
                AppRoutes.TRIP_EDIT,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                val viewModel: TripFormViewModel = viewModel(
                    key = "trip-edit-$id",
                    factory = viewModelFactory { TripFormViewModel(container.tripRepository, tripId = id) },
                )
                TripFormScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onSaved = { savedId ->
                        navController.navigate(AppRoutes.tripDetail(savedId)) {
                            popUpTo(AppRoutes.TRIPS)
                        }
                    },
                )
            }
            composable(AppRoutes.TRIP_JOIN) {
                if (!isLoggedIn) {
                    LoginRequiredScreen(
                        title = "登录后加入行程",
                        body = "加入多人行程需要绑定到你的账号。",
                        onLogin = { navController.navigate(AppRoutes.AUTH) },
                    )
                    return@composable
                }
                val viewModel: TripJoinViewModel = viewModel(factory = viewModelFactory {
                    TripJoinViewModel(container.tripRepository)
                })
                TripJoinScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onAccepted = { id ->
                        navController.navigate(AppRoutes.tripDetail(id)) {
                            popUpTo(AppRoutes.TRIPS)
                        }
                    },
                )
            }
            composable(
                AppRoutes.TRIP_DETAIL,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                if (!isLoggedIn) {
                    LoginRequiredScreen(
                        title = "登录后查看行程",
                        body = "行程计划和装备分工会保存到账号中，请先登录。",
                        onLogin = { navController.navigate(AppRoutes.AUTH) },
                    )
                    return@composable
                }
                val viewModel: TripDetailViewModel = viewModel(
                    key = "trip-detail-$id",
                    factory = viewModelFactory { TripDetailViewModel(container.tripRepository, id) },
                )
                TripDetailScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onEdit = { tripId -> navController.navigate(AppRoutes.tripEdit(tripId)) },
                    onDeleted = {
                        navController.navigate(AppRoutes.TRIPS) {
                            popUpTo(AppRoutes.TRIPS)
                            launchSingleTop = true
                        }
                    },
                )
            }
            composable(
                AppRoutes.SKILL_DETAIL,
                arguments = listOf(navArgument("id") { type = NavType.StringType }),
            ) { backStackEntry ->
                val id = requireNotNull(backStackEntry.arguments?.getString("id"))
                val viewModel: SkillDetailViewModel = viewModel(
                    key = "skill-detail-$id",
                    factory = viewModelFactory { SkillDetailViewModel(container.skillRepository, id) },
                )
                LaunchedEffect(id) { viewModel.load() }
                SkillDetailScreen(viewModel = viewModel, onBack = { navController.popBackStack() })
            }
            composable(AppRoutes.PROFILE) {
                val viewModel: ProfileViewModel = viewModel(factory = viewModelFactory {
                    ProfileViewModel(container.authRepository, container.themeRepository, container.configStore)
                })
                ProfileScreen(
                    viewModel = viewModel,
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                    onOpenAbout = { navController.navigate(AppRoutes.PROFILE_ABOUT) },
                    onOpenSettings = { navController.navigate(AppRoutes.PROFILE_SETTINGS) },
                )
            }
            composable(AppRoutes.PROFILE_ABOUT) {
                ProfileAboutScreen(
                    onBack = { navController.popBackStack() },
                    onOpenRoadmap = { navController.navigate(AppRoutes.PROFILE_ROADMAP) },
                )
            }
            composable(AppRoutes.PROFILE_ROADMAP) {
                val viewModel: RoadmapViewModel = viewModel(factory = viewModelFactory {
                    RoadmapViewModel(container.profileRepository, container.authRepository)
                })
                RoadmapScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                )
            }
            composable(AppRoutes.PROFILE_OUTDOOR) {
                val viewModel: OutdoorProfileViewModel = viewModel(factory = viewModelFactory {
                    OutdoorProfileViewModel(container.profileRepository, container.authRepository)
                })
                OutdoorProfileScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                )
            }
            composable(AppRoutes.PROFILE_OUTDOOR_EXPERIENCES) {
                val viewModel: OutdoorExperiencesViewModel = viewModel(factory = viewModelFactory {
                    OutdoorExperiencesViewModel(container.profileRepository, container.authRepository)
                })
                OutdoorExperiencesScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                )
            }
            composable(AppRoutes.PROFILE_SETTINGS) {
                val viewModel: ProfileSettingsViewModel = viewModel(factory = viewModelFactory {
                    ProfileSettingsViewModel(container.authRepository, container.themeRepository, container.configStore)
                })
                ProfileSettingsScreen(
                    viewModel = viewModel,
                    onBack = { navController.popBackStack() },
                    onLogin = { navController.navigate(AppRoutes.AUTH) },
                    onOpenOutdoorProfile = { navController.navigate(AppRoutes.PROFILE_OUTDOOR) },
                    onOpenOutdoorExperiences = { navController.navigate(AppRoutes.PROFILE_OUTDOOR_EXPERIENCES) },
                )
            }
        }
    }
}

@Composable
private fun RowScope.MiniProgramBottomNavItem(
    destination: TopLevelDestination,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val palette = currentTrailPalette()
    val selectedColor = MaterialTheme.colorScheme.primary
    val unselectedColor = palette.textMuted
    Column(
        modifier = Modifier
            .weight(1f)
            .padding(horizontal = 5.dp, vertical = 6.dp)
            .clip(RoundedCornerShape(20.dp))
            .background(if (selected) palette.softControlBackground else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Icon(
            imageVector = destination.icon,
            contentDescription = null,
            modifier = Modifier.size(25.dp),
            tint = if (selected) selectedColor else unselectedColor,
        )
        Text(
            text = stringResource(destination.labelRes),
            color = if (selected) selectedColor else unselectedColor,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
        )
    }
}

private fun miniProgramTopBarTitle(route: String?): String = when (route) {
    AppRoutes.SKILLS -> "户外技能"
    else -> "寻径星野"
}

@Composable
private fun MiniProgramTopBar(title: String) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(84.dp)
            .background(Color(0xFF0F172A))
            .statusBarsPadding(),
        contentAlignment = Alignment.Center,
    ) {
        Text(title, color = Color.White, fontWeight = FontWeight.ExtraBold)
    }
}
