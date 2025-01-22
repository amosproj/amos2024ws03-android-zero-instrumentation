// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.selection

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ToggleAutoscrollFab(
    autoScrollActive: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ExtendedFloatingActionButton(
        modifier = modifier.padding(25.dp),
        onClick = onClick,
        icon = {
            Icon(
                imageVector =
                    if (autoScrollActive) Icons.AutoMirrored.Filled.List
                    else Icons.Filled.PlayArrow,
                contentDescription = "",
            )
        },
        text = { Text(text = if (autoScrollActive) "Disable autoscroll" else "Enable autoscroll") },
    )
}
