// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation.utils

import android.net.Uri
import android.os.Bundle
import androidx.compose.animation.AnimatedContentScope
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.runtime.Composable
import androidx.navigation.NamedNavArgument
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.ui.Routes
import de.amosproj3.ziofa.ui.shared.getDisplayName
import de.amosproj3.ziofa.ui.shared.serializePIDs

fun NavGraphBuilder.screenWithDefaultAnimations(
    route: String,
    content: @Composable AnimatedContentScope.(NavBackStackEntry) -> Unit,
) {
    composable(
        route,
        popEnterTransition = { fadeIn() },
        enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
        exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
        content = content,
    )
}

fun NavGraphBuilder.parameterizedScreen(
    route: String,
    arguments: List<NamedNavArgument>,
    content: @Composable AnimatedContentScope.(NavBackStackEntry) -> Unit,
) {
    composable(
        route,
        arguments = arguments,
        popEnterTransition = { fadeIn() },
        enterTransition = { slideInHorizontally(initialOffsetX = { it }) + fadeIn() },
        exitTransition = { slideOutHorizontally(targetOffsetX = { it }) + fadeOut() },
        content = content,
    )
}

fun RunningComponent.toConfigurationScreenRouteForComponent(): String {
    val displayNameParam = Uri.encode(this.getDisplayName())
    val pidsParam = Uri.encode(this.serializePIDs())
    return "${Routes.IndividualConfiguration.name}?displayName=$displayNameParam?pids=$pidsParam"
}

/** Pass the parameters of the opened configuration to the symbols screen. */
fun Bundle?.copyToSymbolsRoute(): String {
    val displayName = this?.getString("displayName")
    val pids = this?.getString("pids")
    return "${Routes.Symbols.name}?displayName=$displayName?pids=$pids"
}
