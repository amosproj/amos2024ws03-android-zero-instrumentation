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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.visualization.composables.EventList
import de.amosproj3.ziofa.ui.visualization.composables.MetricDropdown
import de.amosproj3.ziofa.ui.visualization.composables.SwitchModeFab
import de.amosproj3.ziofa.ui.visualization.composables.VicoBar
import de.amosproj3.ziofa.ui.visualization.composables.VicoTimeSeries
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationScreenState
import org.koin.androidx.compose.koinViewModel
import timber.log.Timber

/** Screen for visualizing data. */
@Composable
fun VisualizationScreen(
    modifier: Modifier = Modifier,
    viewModel: VisualizationViewModel = koinViewModel(),
) {
    Box(modifier = modifier.fillMaxSize()) {
        val visualizationScreenState by
            remember { viewModel.visualizationScreenState }.collectAsState()
        val state = visualizationScreenState
        Timber.i("Updating UI based on $state")

        Column(verticalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxSize()) {
            when (state) {
                is VisualizationScreenState.MetricSelectionValid -> {
                    MetricSelection(
                        selectionData = state.selectionData,
                        filterSelected = { viewModel.componentSelected(it) },
                        metricSelected = { viewModel.metricSelected(it) },
                        timeframeSelected = { viewModel.timeframeSelected(it) },
                    )
                    DataViewer(state.graphedData)
                }

                is VisualizationScreenState.WaitingForMetricSelection -> {
                    MetricSelection(
                        selectionData = state.selectionData,
                        filterSelected = { viewModel.componentSelected(it) },
                        metricSelected = { viewModel.metricSelected(it) },
                        timeframeSelected = { viewModel.timeframeSelected(it) },
                    )
                    SelectMetricPrompt()
                }

                is VisualizationScreenState.Invalid -> ErrorScreen(state.error)
                VisualizationScreenState.Loading -> CircularProgressIndicator()
            }
        }

        if (state is VisualizationScreenState.MetricSelectionValid)
            SwitchModeFab(
                Modifier.align(Alignment.BottomEnd),
                onClick = { viewModel.switchMode() },
                activeDisplayMode = state.displayMode,
            )
    }
}

@Composable
fun SelectMetricPrompt() {
    Box(Modifier.fillMaxSize()) {
        Text(
            "Please make a selection!",
            Modifier.align(Alignment.Center),
            fontWeight = FontWeight.Bold,
        )
    }
}

@Composable
fun DataViewer(data: GraphedData) {
    when (data) {
        is GraphedData.TimeSeriesData ->
            VicoTimeSeries(seriesData = data.seriesData, chartMetadata = data.metaData)

        is GraphedData.HistogramData ->
            VicoBar(seriesData = data.seriesData, chartMetadata = data.metaData)

        is GraphedData.EventListData -> EventList(data.eventData)

        GraphedData.EMPTY -> {}
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
            selectionData.componentOptions,
            "Select a package",
            modifier = Modifier.weight(1f).padding(end = 0.dp),
            optionSelected = { filterSelected(it) },
            selectedOption = selectionData.selectedComponent.displayName,
        )
        selectionData.metricOptions
            ?.takeIf { it.isNotEmpty() }
            ?.let { metricOptions ->
                MetricDropdown(
                    metricOptions,
                    "Select a metric",
                    modifier = Modifier.weight(1f).padding(end = 0.dp),
                    optionSelected = { metricSelected(it) },
                    selectedOption = selectionData.selectedMetric?.displayName ?: "Please select...",
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
                    selectionData.selectedTimeframe?.displayName ?: "Please select...",
                )
            } ?: Spacer(Modifier.weight(1f))
    }
}
