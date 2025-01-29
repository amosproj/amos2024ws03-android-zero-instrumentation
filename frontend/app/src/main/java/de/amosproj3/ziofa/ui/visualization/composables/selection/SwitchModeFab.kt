// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.selection

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.data.VisualizationDisplayMode

private const val ANIMATION_DURATION_MS = 500

/*Inspired by https://medium.com/@developerchunk/custom-material-3-expandable-floating-action-button-in-jetpack-compose-a29518c4c008*/
@Composable
fun SwitchModeFab(
    onDisplayModeSelected: (VisualizationDisplayMode) -> Unit,
    modifier: Modifier = Modifier,
) {
    var menuOpen by remember { mutableStateOf(false) }

    val interactionSource = remember { MutableInteractionSource() }

    Card(modifier = modifier, elevation = CardDefaults.elevatedCardElevation(4.dp)) {

        // parent layout
        Column {
            AnimatedVisibility(
                visible = menuOpen,
                enter = expandVertically(tween(ANIMATION_DURATION_MS)) + fadeIn(),
                exit = shrinkVertically(tween(ANIMATION_DURATION_MS)) + fadeOut(),
            ) {
                MenuItems(
                    onItemClicked = {
                        onDisplayModeSelected(it)
                        menuOpen = false
                    },
                    interactionSource,
                )
            }

            MenuToggle(onClick = { menuOpen = !menuOpen }, interactionSource)
        }
    }
}

@Composable
private fun MenuToggle(onClick: () -> Unit, interactionSource: MutableInteractionSource) {
    Card(
        modifier =
            Modifier.clickable(
                interactionSource = interactionSource,
                indication = null,
                onClick = onClick,
            ),
        colors = CardDefaults.cardColors(MaterialTheme.colorScheme.primary),
    ) {
        Row(modifier = Modifier.padding(vertical = 20.dp, horizontal = 30.dp)) {
            Icon(imageVector = Icons.AutoMirrored.Filled.List, contentDescription = "")
            Row {
                Spacer(modifier = Modifier.width(20.dp))
                Text(text = "Select mode")
            }
        }
    }
}

@Composable
private fun MenuItems(
    onItemClicked: (VisualizationDisplayMode) -> Unit,
    interactionSource: MutableInteractionSource,
) {
    Column(modifier = Modifier.padding(vertical = 20.dp, horizontal = 30.dp)) {
        listOf(
                VisualizationDisplayMode.CHART,
                VisualizationDisplayMode.EVENTS,
                VisualizationDisplayMode.OVERLAY,
            )
            .forEach { item ->
                Row(
                    modifier =
                        Modifier.padding(vertical = 10.dp)
                            .clickable(
                                interactionSource = interactionSource,
                                indication = null,
                                onClick = { onItemClicked(item) },
                            )
                ) {
                    Icon(imageVector = item.icon, contentDescription = "")
                    Spacer(modifier = Modifier.width(15.dp))
                    Text(text = item.displayName)
                }
            }
    }
}
