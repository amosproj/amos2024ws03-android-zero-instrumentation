// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui

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
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import de.amosproj3.ziofa.ui.about.AboutScreen
import de.amosproj3.ziofa.ui.configuration.ConfigurationScreen
import de.amosproj3.ziofa.ui.navigation.HomeScreen
import de.amosproj3.ziofa.ui.navigation.composables.ZiofaTopBar
import de.amosproj3.ziofa.ui.processes.ProcessesScreen
import de.amosproj3.ziofa.ui.visualization.VisualizationScreen

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
                    toProcesses = { navController.navigate(Routes.Processes.name) },
                    modifier = Modifier.padding(innerPadding),
                )
            }
            composable(
                Routes.Configuration.name,
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                ConfigurationScreen(
                    Modifier.padding(innerPadding),
                    onBack = { navController.backToHome() },
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
                enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
                exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
            ) {
                ProcessesScreen(Modifier.padding(innerPadding))
            }
        }
    }
}

/** Top bar with a back button on all screens except for the home screen. */
@Composable
fun DynamicTopBar(navController: NavController) {
    navController.currentBackStackEntryAsState().value?.destination?.route?.let { currentRoute ->
        when (currentRoute) {
            Routes.Home.name -> {
                ZiofaTopBar(
                    screenName = "Zero Instrumentation Observability for Android",
                    showBackButton = false,
                )
            }

            else -> {
                ZiofaTopBar(screenName = currentRoute, onBack = { navController.backToHome() })
            }
        }
    }
}

fun NavController.backToHome() {
    this.navigate(Routes.Home.name) { popUpTo(Routes.Home.name) { inclusive = false } }
}
