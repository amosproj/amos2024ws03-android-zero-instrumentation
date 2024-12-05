// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui

import android.net.Uri
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.ui.about.AboutScreen
import de.amosproj3.ziofa.ui.configuration.ConfigurationScreen
import de.amosproj3.ziofa.ui.navigation.ConfigurationMenu
import de.amosproj3.ziofa.ui.navigation.HomeScreen
import de.amosproj3.ziofa.ui.navigation.composables.ZiofaTopBar
import de.amosproj3.ziofa.ui.processes.ProcessesScreen
import de.amosproj3.ziofa.ui.shared.deserializePIDs
import de.amosproj3.ziofa.ui.shared.getDisplayName
import de.amosproj3.ziofa.ui.shared.serializePIDs
import de.amosproj3.ziofa.ui.shared.validPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.VisualizationScreen

val GLOBAL_CONFIGURATION_ROUTE =
    "${Routes.IndividualConfiguration.name}?displayName=${Uri.encode("all processes")}?pids=-1"

/** Main application composable. All calls to [NavController] should happen here. */
@Composable
fun ZIOFAApp() {
    val navController = rememberNavController()

    Scaffold(modifier = Modifier.fillMaxSize(), topBar = { DynamicTopBar(navController) }) {
        innerPadding ->
        NavHost(
            navController,
            modifier = Modifier.fillMaxSize(),
            startDestination = Routes.Home.name,
        ) {
            composable(Routes.Home.name) {
                HomeScreen(
                    toVisualize = { navController.navigate(Routes.Visualize.name) },
                    toConfiguration = { navController.navigate(Routes.Configuration.name) },
                    toAbout = { navController.navigate(Routes.About.name) },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            composable(
                Routes.Configuration.name,
                popEnterTransition = { fadeIn() },
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                ConfigurationMenu(
                    Modifier.padding(innerPadding),
                    toPresets = { /*TODO*/ },
                    toProcesses = { navController.navigate(Routes.Processes.name) },
                    toGlobalConfiguration = { navController.navigate(GLOBAL_CONFIGURATION_ROUTE) },
                )
            }
            composable(
                "${Routes.IndividualConfiguration.name}?displayName={displayName}?pids={pids}",
                arguments =
                    listOf(
                        navArgument("displayName") {
                            type = NavType.StringType
                            nullable = true
                        },
                        navArgument("pids") {
                            type = NavType.StringType
                            nullable = true
                        },
                    ),
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                ConfigurationScreen(
                    Modifier.padding(innerPadding),
                    onBack = { navController.popBackStack() },
                    pids = it.arguments?.getString("pids")?.deserializePIDs()?.validPIDsOrNull(),
                )
            }
            composable(
                Routes.Visualize.name,
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                VisualizationScreen(Modifier.padding(innerPadding))
            }
            composable(
                Routes.About.name,
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                AboutScreen(Modifier.padding(innerPadding))
            }

            composable(
                Routes.Processes.name,
                popEnterTransition = { fadeIn() },
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                ProcessesScreen(
                    Modifier.padding(innerPadding),
                    onClickEdit = {
                        navController.navigate(it.toConfigurationScreenRouteForProcess())
                    },
                )
            }
        }
    }
}

/** Top bar with a back button on all screens except for the home screen. */
@Composable
fun DynamicTopBar(navController: NavController) {
    val backStackEntry = navController.currentBackStackEntryAsState().value
    val route = backStackEntry?.destination?.route?.split("?")?.getOrNull(0)
    val displayName = backStackEntry?.arguments?.getString("displayName")
    route?.let { currentRoute ->
        when (currentRoute) {
            Routes.Home.name -> {
                ZiofaTopBar(
                    screenName = "Zero Instrumentation Observability for Android",
                    showBackButton = false,
                )
            }

            Routes.IndividualConfiguration.name -> {
                ZiofaTopBar(
                    screenName = "Configuration for $displayName",
                    onBack = { navController.popBackStack() },
                )
            }

            else -> {
                ZiofaTopBar(screenName = currentRoute, onBack = { navController.popBackStack() })
            }
        }
    }
}

fun RunningComponent.toConfigurationScreenRouteForProcess(): String {
    val displayNameParam = Uri.encode(this.getDisplayName())
    val pidsParam = Uri.encode(this.serializePIDs())
    return "${Routes.IndividualConfiguration.name}?displayName=$displayNameParam?pids=$pidsParam"
}
