// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui

import android.net.Uri
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import de.amosproj3.ziofa.ui.about.AboutScreen
import de.amosproj3.ziofa.ui.configuration.ConfigurationScreen
import de.amosproj3.ziofa.ui.init.InitScreen
import de.amosproj3.ziofa.ui.navigation.ConfigurationMenu
import de.amosproj3.ziofa.ui.navigation.HomeScreen
import de.amosproj3.ziofa.ui.navigation.composables.DynamicTopBar
import de.amosproj3.ziofa.ui.navigation.utils.copyToSymbolsRoute
import de.amosproj3.ziofa.ui.navigation.utils.parameterizedScreen
import de.amosproj3.ziofa.ui.navigation.utils.screenWithDefaultAnimations
import de.amosproj3.ziofa.ui.navigation.utils.toConfigurationScreenRouteForComponent
import de.amosproj3.ziofa.ui.processes.ProcessesScreen
import de.amosproj3.ziofa.ui.reset.ResetScreen
import de.amosproj3.ziofa.ui.shared.deserializePIDs
import de.amosproj3.ziofa.ui.shared.validPIDsOrNull
import de.amosproj3.ziofa.ui.symbols.SymbolsScreen
import de.amosproj3.ziofa.ui.visualization.VisualizationScreen
import kotlinx.collections.immutable.toImmutableList

val GLOBAL_CONFIGURATION_ROUTE =
    "${Routes.IndividualConfiguration.name}?displayName=${Uri.encode("all processes")}?pids=-1"

val PIDS_ARG =
    navArgument("pids") {
        type = NavType.StringType
        nullable = true
    }
val DISPLAY_NAME_ARG =
    navArgument("displayName") {
        type = NavType.StringType
        nullable = true
    }

/** Main application composable. All calls to [NavController] should happen here. */
@Suppress("ModifierMissing, LongMethod") // Top level composable
@Composable
fun ZIOFAApp() {
    val navController = rememberNavController()

    Scaffold(modifier = Modifier.fillMaxSize(), topBar = { DynamicTopBar(navController) }) {
        innerPadding ->
        NavHost(
            navController,
            modifier = Modifier.fillMaxSize(),
            startDestination = Routes.Init.name,
        ) {
            screenWithDefaultAnimations(Routes.Init.name) {
                InitScreen(
                    onInitFinished = { navController.navigate(Routes.Home.name) },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            screenWithDefaultAnimations(Routes.Home.name) {
                HomeScreen(
                    toVisualize = { navController.navigate(Routes.Visualize.name) },
                    toConfiguration = { navController.navigate(Routes.Processes.name) },
                    toReset = { navController.navigate(Routes.Reset.name) },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            screenWithDefaultAnimations(Routes.Reset.name) {
                ResetScreen(
                    afterResetConfirmed = { navController.popBackStack() },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            screenWithDefaultAnimations(Routes.Configuration.name) {
                ConfigurationMenu(
                    toProcesses = { navController.navigate(Routes.Processes.name) },
                    toGlobalConfiguration = { navController.navigate(GLOBAL_CONFIGURATION_ROUTE) },
                    toReset = { navController.navigate(Routes.Reset.name) },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            screenWithDefaultAnimations(Routes.Visualize.name) {
                VisualizationScreen(Modifier.padding(innerPadding))
            }
            screenWithDefaultAnimations(Routes.About.name) {
                AboutScreen(Modifier.padding(innerPadding))
            }
            screenWithDefaultAnimations(Routes.Processes.name) {
                ProcessesScreen(
                    Modifier.padding(innerPadding),
                    onClickEdit = { component ->
                        navController.navigate(component.toConfigurationScreenRouteForComponent())
                    },
                )
            }

            parameterizedScreen(
                "${Routes.IndividualConfiguration.name}?displayName={displayName}?pids={pids}",
                arguments = listOf(DISPLAY_NAME_ARG, PIDS_ARG),
            ) {
                ConfigurationScreen(
                    Modifier.padding(innerPadding),
                    pids =
                        it.arguments
                            ?.getString("pids")
                            ?.deserializePIDs()
                            ?.validPIDsOrNull()
                            ?.toImmutableList(),
                    onAddUprobeSelected = {
                        navController.navigate(it.arguments.copyToSymbolsRoute())
                    },
                )
            }

            parameterizedScreen(
                "${Routes.Symbols.name}?displayName={displayName}?pids={pids}",
                arguments = listOf(DISPLAY_NAME_ARG, PIDS_ARG),
            ) {
                SymbolsScreen(
                    modifier = Modifier.padding(innerPadding),
                    onSymbolsSubmitted = { navController.popBackStack() },
                    pids =
                        it.arguments
                            ?.getString("pids")
                            ?.deserializePIDs()
                            ?.validPIDsOrNull()
                            .orEmpty(),
                )
            }
        }
    }
}
