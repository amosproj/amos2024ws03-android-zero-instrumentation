// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation.composables

import android.net.Uri
import androidx.compose.runtime.Composable
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState
import de.amosproj3.ziofa.ui.Routes

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
                    screenName = "Configuration for ${Uri.decode(displayName)}",
                    onBack = { navController.popBackStack() },
                )
            }

            Routes.Symbols.name -> {
                ZiofaTopBar(
                    screenName = "Add uprobes for ${Uri.decode(displayName)}",
                    onBack = { navController.popBackStack() },
                )
            }

            else -> {
                ZiofaTopBar(screenName = currentRoute, onBack = { navController.popBackStack() })
            }
        }
    }
}
