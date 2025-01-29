// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Star
import androidx.compose.ui.graphics.vector.ImageVector

enum class VisualizationDisplayMode(val displayName: String, val icon: ImageVector) {
    CHART("Chart", Icons.Filled.PlayArrow),
    EVENTS("Event list", Icons.AutoMirrored.Filled.List),
    OVERLAY("Overlay", Icons.Filled.Star),
}
