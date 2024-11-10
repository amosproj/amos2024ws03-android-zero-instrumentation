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
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp

@Composable
fun HomeScreen(
    toVisualize: () -> Unit,
    toConfiguration: () -> Unit,
    toAbout: () -> Unit,
    modifier: Modifier,
) {
    Box(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .padding(horizontal = 50.dp, vertical = 40.dp)
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            MenuOptions(Modifier.weight(1f), toVisualize, toConfiguration, toAbout)
        }
    }
}

@Composable
fun MenuOptions(
    modifier: Modifier = Modifier,
    toVisualize: () -> Unit,
    toConfiguration: () -> Unit,
    toAbout: () -> Unit,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        MenuOptionWithIcon(
            text = "Visualize",
            emoji = "\uD83D\uDCCA",
            modifier = Modifier.weight(1f).padding(horizontal = 10.dp),
            onClick = toVisualize,
        )
        MenuOptionWithIcon(
            text = "Configure",
            emoji = "⚙\uFE0F",
            modifier = Modifier.weight(1f).padding(horizontal = 10.dp),
            onClick = toConfiguration,
        )
        MenuOptionWithIcon(
            text = "About",
            emoji = "ℹ\uFE0F",
            modifier = Modifier.weight(1f).padding(horizontal = 10.dp),
            onClick = toAbout,
        )
    }
}

@Composable
fun MenuOptionWithIcon(
    text: String,
    emoji: String,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    val modifierForCards = modifier.aspectRatio(1f).clickable { onClick() }.focusable()

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
