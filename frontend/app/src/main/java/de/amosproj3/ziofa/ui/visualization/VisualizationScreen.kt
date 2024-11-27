// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.composables.EventList
import de.amosproj3.ziofa.ui.visualization.composables.MetricDropdown
import de.amosproj3.ziofa.ui.visualization.composables.SwitchModeFab
import de.amosproj3.ziofa.ui.visualization.composables.VicoBar
import de.amosproj3.ziofa.ui.visualization.composables.VicoTimeSeries
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_CHART_METADATA
import org.koin.androidx.compose.koinViewModel

/** Screen for visualizing data. */
@Composable
fun VisualizationScreen(
    modifier: Modifier = Modifier,
    viewModel: VisualizationViewModel = koinViewModel(),
) {
    Box(modifier = modifier.fillMaxSize()) {
        val graphedData by remember { viewModel.graphedData }.collectAsState()
        val chartMetadata by remember { viewModel.chartMetadata }.collectAsState()
        val selectionData by remember { viewModel.selectionData }.collectAsState()

        Column(verticalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxSize()) {
            MetricSelection(
                selectionData = selectionData,
                filterSelected = { viewModel.filterSelected(it) },
                metricSelected = { viewModel.metricSelected(it) },
                timeframeSelected = { viewModel.timeframeSelected(it) },
            )
            if (
                chartMetadata != DEFAULT_CHART_METADATA && graphedData !is GraphedData.EventListData
            ) {
                VisualizationTitle(chartMetadata.visualizationTitle)
            }

            when (val data = graphedData) {
                is GraphedData.TimeSeriesData ->
                    VicoTimeSeries(seriesData = data.data, chartMetadata = chartMetadata)

                is GraphedData.HistogramData ->
                    VicoBar(seriesData = data.data, chartMetadata = chartMetadata)

                is GraphedData.EventListData -> EventList(data.data)

                GraphedData.EMPTY -> {}
            }
        }

        SwitchModeFab(Modifier.align(Alignment.BottomEnd), onClick = { viewModel.switchMode() })
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
fun MetricSelection(
    selectionData: SelectionData,
    filterSelected: (DropdownOption) -> Unit,
    metricSelected: (DropdownOption) -> Unit,
    timeframeSelected: (DropdownOption) -> Unit,
) {
    Row(Modifier.fillMaxWidth()) {
        MetricDropdown(
            selectionData.filterOptions,
            "Select a package",
            modifier = Modifier.weight(1f).padding(end = 0.dp),
            optionSelected = { filterSelected(it) },
        )
        selectionData.metricOptions
            ?.takeIf { it.isNotEmpty() }
            ?.let { metricOptions ->
                MetricDropdown(
                    metricOptions,
                    "Select a metric",
                    modifier = Modifier.weight(1f).padding(end = 0.dp),
                    optionSelected = { metricSelected(it) },
                )
            } ?: Spacer(Modifier.weight(1f))
        selectionData.timeframeOptions
            ?.takeIf { it.isNotEmpty() }
            ?.let { timeframeOptions ->
                MetricDropdown(
                    timeframeOptions,
                    "Select an interval for aggregation",
                    modifier = Modifier.weight(1f).padding(end = 0.dp),
                    optionSelected = { timeframeSelected(it) },
                )
            } ?: Spacer(Modifier.weight(1f))
    }
}
