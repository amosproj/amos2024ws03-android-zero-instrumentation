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
            screenWithDefaultAnimations(Routes.Home.name) {
                HomeScreen(
                    toVisualize = { navController.navigate(Routes.Visualize.name) },
                    toConfiguration = { navController.navigate(Routes.Configuration.name) },
                    toAbout = { navController.navigate(Routes.About.name) },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            screenWithDefaultAnimations(Routes.Reset.name) {
                ResetScreen(
                    Modifier.padding(innerPadding),
                    afterResetConfirmed = { navController.popBackStack() },
                )
            }
            screenWithDefaultAnimations(Routes.Configuration.name) {
                ConfigurationMenu(
                    Modifier.padding(innerPadding),
                    toProcesses = { navController.navigate(Routes.Processes.name) },
                    toGlobalConfiguration = { navController.navigate(GLOBAL_CONFIGURATION_ROUTE) },
                    toReset = { navController.navigate(Routes.Reset.name) },
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
                    onClickEdit = {
                        navController.navigate(it.toConfigurationScreenRouteForComponent())
                    },
                )
            }
            parameterizedScreen(
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
            ) {
                ConfigurationScreen(
                    Modifier.padding(innerPadding),
                    onBack = { navController.popBackStack() },
                    pids = it.arguments?.getString("pids")?.deserializePIDs()?.validPIDsOrNull(),
                    onAddUprobeSelected = {
                        navController.navigate(it.arguments.copyToSymbolsRoute())
                    },
                )
            }

            parameterizedScreen(
                "${Routes.Symbols.name}?displayName={displayName}?pids={pids}",
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
            ) {
                SymbolsScreen(
                    modifier = Modifier.padding(innerPadding),
                    onSymbolsSubmitted = { navController.popBackStack() },
                    pids =
                        it.arguments?.getString("pids")?.deserializePIDs()?.validPIDsOrNull()
                            ?: listOf(),
                )
            }
        }
    }
}
