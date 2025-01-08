// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.navigation.composables.MenuOptions
import de.amosproj3.ziofa.ui.navigation.data.Emoji
import de.amosproj3.ziofa.ui.navigation.data.MenuOptionData

/** Static home screen for navigation */
@Composable
@Preview(device = Devices.AUTOMOTIVE_1024p)
fun HomeScreen(
    modifier: Modifier = Modifier,
    toVisualize: () -> Unit = {},
    toConfiguration: () -> Unit = {},
    toAbout: () -> Unit = {},
) {
    Box(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .padding(horizontal = 50.dp, vertical = 40.dp)
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            MenuOptions(
                menuOptions =
                    listOf(
                        MenuOptionData(title = "Visualize", Emoji.Chart.unicode, toVisualize),
                        MenuOptionData(
                            title = "Configuration",
                            Emoji.Gear.unicode,
                            toConfiguration,
                        ),
                        MenuOptionData(title = "About", Emoji.Info.unicode, toAbout),
                    )
            )
        }
    }
}
