// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.overlay

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata

@Composable
fun OverlayAxisLegend(modifier: Modifier = Modifier, chartMetadata: ChartMetadata) {
    Row(modifier = modifier, horizontalArrangement = Arrangement.SpaceEvenly) {
        Row(horizontalArrangement = Arrangement.Center, modifier = Modifier.weight(1f)) {
            Text("X-Axis: ", fontWeight = FontWeight.Bold)
            Text(chartMetadata.xLabel)
        }
        Row(horizontalArrangement = Arrangement.Center, modifier = Modifier.weight(1f)) {
            Text("Y-Axis: ", fontWeight = FontWeight.Bold)
            Text(chartMetadata.yLabel)
        }
    }
}
