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
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.navigation.composables.MenuOptionData
import de.amosproj3.ziofa.ui.navigation.composables.MenuOptions
import de.amosproj3.ziofa.ui.navigation.data.Emoji

@Composable
fun ConfigurationMenu(
    modifier: Modifier = Modifier,
    toPresets: () -> Unit,
    toProcesses: () -> Unit,
    toGlobalConfiguration: () -> Unit,
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
                        MenuOptionData(
                            title = "Presets",
                            logoEmoji = Emoji.Bookmarks.unicode,
                            onClick = toPresets,
                        ),
                        MenuOptionData(
                            title = "Global",
                            logoEmoji = Emoji.Globe.unicode,
                            onClick = toGlobalConfiguration,
                        ),
                        MenuOptionData(
                            title = "Per Process",
                            logoEmoji = Emoji.MagnifyingGlass.unicode,
                            onClick = toProcesses,
                        ),
                    )
            )
        }
    }
}
