// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.composables.MetricDropdown
import de.amosproj3.ziofa.ui.visualization.composables.VicoTimeSeries
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import org.koin.androidx.compose.koinViewModel

/** Screen for visualizing data. */
@Composable
fun VisualizationScreen(
    modifier: Modifier = Modifier,
    viewModel: VisualizationViewModel = koinViewModel(),
) {
    Box(modifier = modifier.fillMaxSize()) {
        val seriesData by remember { viewModel.graphedData }.collectAsState()
        val chartMetadata by remember { viewModel.chartMetadata }.collectAsState()
        val selectionData by remember { viewModel.selectionData }.collectAsState()

        Column(verticalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxSize()) {
            MetricSelection(selectionData)
            VisualizationTitle(chartMetadata.visualizationTitle)
            VicoTimeSeries(seriesData = seriesData, chartMetadata = chartMetadata)
        }
    }
}

@Composable
fun VisualizationTitle(title: String) {
    Row(
        horizontalArrangement = Arrangement.Center,
        modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp),
    ) {
        Text(title)
    }
}

@Composable
fun MetricSelection(selectionData: SelectionData) {
    Row(Modifier.fillMaxWidth()) {
        MetricDropdown(
            selectionData.packageOptions.map { it.packageName to it.logo },
            "Select a package",
            modifier = Modifier.weight(1f).padding(end = 0.dp),
        )
        MetricDropdown(
            selectionData.metricOptions.map { it.displayName to null },
            "Select a metric",
            modifier = Modifier.weight(1f).padding(end = 0.dp),
        )
        MetricDropdown(
            selectionData.timeframeOptions.map { "${it.amount} ${it.unit}" to null },
            "Select an interval",
            modifier = Modifier.weight(1f).padding(end = 0.dp),
        )
    }
}
