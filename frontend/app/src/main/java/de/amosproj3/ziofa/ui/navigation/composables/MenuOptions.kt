// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation.composables

import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.navigation.data.MenuOptionData

private const val CARD_EMOJI_SIZE = 120f
private const val CARD_TITLE_TEXT_SIZE = 40f

@Composable
fun MenuOptions(menuOptions: List<MenuOptionData>, modifier: Modifier = Modifier) {
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
                fontSize = TextUnit(CARD_EMOJI_SIZE, TextUnitType.Sp),
                modifier = Modifier.padding(bottom = 20.dp),
            )
            Text(text, fontSize = TextUnit(CARD_TITLE_TEXT_SIZE, TextUnitType.Sp))
        }
    }
}
