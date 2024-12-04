// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.utils.VisualizationDisplayMode

@Composable
fun SwitchModeFab(
    modifier: Modifier = Modifier,
    onClick: () -> Unit = {},
    activeDisplayMode: VisualizationDisplayMode,
) {
    ExtendedFloatingActionButton(
        modifier = modifier.padding(end = 25.dp, bottom = 25.dp),
        onClick = onClick,
        icon = { Icon(imageVector = Icons.AutoMirrored.Filled.List, contentDescription = "") },
        text = {
            when (activeDisplayMode) {
                VisualizationDisplayMode.CHART -> "Switch to event mode"
                VisualizationDisplayMode.EVENTS -> "Switch to chart mode"
            }.let { Text(it) }
        },
    )
}
