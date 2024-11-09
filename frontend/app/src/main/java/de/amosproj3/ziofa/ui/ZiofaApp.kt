// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import de.amosproj3.ziofa.ui.about.AboutScreen
import de.amosproj3.ziofa.ui.configuration.ConfigurationScreen
import de.amosproj3.ziofa.ui.navigation.ZiofaNavBarItem
import de.amosproj3.ziofa.ui.navigation.ZiofaNavigationBar
import de.amosproj3.ziofa.ui.navigation.ZiofaTopBar
import de.amosproj3.ziofa.ui.visualization.VisualizationScreen

/** Main application composable */
@Composable
fun ZIOFAApp() {
    val navController = rememberNavController()

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = { ZiofaTopBar() },
        bottomBar = {
            ZiofaNavigationBar(
                navBarItems =
                    listOf(
                        ZiofaNavBarItem(text = "Configuration", icon = Icons.Filled.Settings) {
                            navController.navigate(Routes.Configuration.name)
                        },
                        ZiofaNavBarItem(text = "Visualize", icon = Icons.Filled.PlayArrow) {
                            navController.navigate(Routes.Visualize.name)
                        },
                        ZiofaNavBarItem(text = "About", icon = Icons.Filled.Info) {
                            navController.navigate(Routes.About.name)
                        },
                    )
            )
        },
    ) { innerPadding ->
        NavHost(
            navController,
            modifier = Modifier.padding(innerPadding).fillMaxSize(),
            startDestination = Routes.Configuration.name,
        ) {
            composable(Routes.Configuration.name) { ConfigurationScreen() }
            composable(Routes.Visualize.name) { VisualizationScreen() }
            composable(Routes.About.name) { AboutScreen() }
        }
    }
}
