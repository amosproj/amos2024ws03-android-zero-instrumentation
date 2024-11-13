// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.about

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

/**
 * Screen containing information about the project. Might delete later if we need space for another
 * screen in the tab bar.
 */
@Composable
fun AboutScreen(modifier: Modifier = Modifier) {
    Box(modifier = modifier) { Text("This is the about screen") }
}
