// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp

data class MenuOptionData(val title: String, val logoEmoji: String, val onClick: () -> Unit)

/** Static home screen for navigation */
@Composable
@Preview(device = Devices.AUTOMOTIVE_1024p)
fun HomeScreen(
    modifier: Modifier = Modifier,
    toVisualize: () -> Unit = {},
    toConfiguration: () -> Unit = {},
    toAbout: () -> Unit = {},
    toProcesses: () -> Unit = {},
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
                        MenuOptionData(title = "Visualize", "\uD83D\uDCCA", toVisualize),
                        MenuOptionData(title = "Configuration", "⚙\uFE0F", toConfiguration),
                        MenuOptionData(title = "Processes", "\uD83D\uDD0E", toProcesses),
                        MenuOptionData(title = "About", "ℹ\uFE0F", toAbout),
                    )
            )
        }
    }
}

@Composable
fun MenuOptions(modifier: Modifier = Modifier, menuOptions: List<MenuOptionData>) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        menuOptions.forEach {
            MenuCardWithIcon(
                text = it.title,
                emoji = it.logoEmoji,
                onClick = it.onClick,
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Composable
fun MenuCardWithIcon(
    text: String,
    emoji: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    val modifierForCards =
        modifier.aspectRatio(1f).clickable { onClick() }.focusable().padding(horizontal = 10.dp)

    Card(
        modifier = modifierForCards,
        elevation = CardDefaults.cardElevation(defaultElevation = 6.dp),
        colors = CardDefaults.elevatedCardColors(),
    ) {
        Column(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                emoji,
                fontSize = TextUnit(120f, TextUnitType.Sp),
                modifier = Modifier.padding(bottom = 20.dp),
            )
            Text(text, fontSize = TextUnit(40f, TextUnitType.Sp))
        }
    }
}
