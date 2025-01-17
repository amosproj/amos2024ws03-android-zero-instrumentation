// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import VisualizationNonExistentPrompt
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.visualization.composables.EventListViewer
import de.amosproj3.ziofa.ui.visualization.composables.MetricSelection
import de.amosproj3.ziofa.ui.visualization.composables.SelectMetricPrompt
import de.amosproj3.ziofa.ui.visualization.composables.SwitchModeFab
import de.amosproj3.ziofa.ui.visualization.composables.ToggleAutoscrollFab
import de.amosproj3.ziofa.ui.visualization.composables.VicoBar
import de.amosproj3.ziofa.ui.visualization.composables.VicoTimeSeries
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
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
        var autoScrollActive by remember { mutableStateOf(true) }

        val state = visualizationScreenState
        Timber.i("Updating UI based on $state")

        Column(verticalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxSize()) {
            when (state) {
                is VisualizationScreenState.Valid -> {
                    MetricSelection(
                        selectionData = state.selectionData,
                        optionSelected = { viewModel.optionSelected(it) },
                    )

                    when (state) {
                        is VisualizationScreenState.Valid.ChartView ->
                            ChartViewer(state.graphedData)

                        is VisualizationScreenState.Valid.EventListView ->
                            EventListViewer(
                                eventListData = state.graphedData,
                                eventListMetadata = state.eventListMetadata,
                                autoScrollActive = autoScrollActive,
                            )
                    }
                }

                is VisualizationScreenState.Incomplete -> {
                    MetricSelection(
                        selectionData = state.selectionData,
                        optionSelected = { viewModel.optionSelected(it) },
                    )

                    // Show a prompt to explain the incomplete selection
                    when (state) {
                        is VisualizationScreenState.Incomplete.WaitingForMetricSelection ->
                            SelectMetricPrompt()

                        is VisualizationScreenState.Incomplete.NoVisualizationExists ->
                            VisualizationNonExistentPrompt()
                    }
                }

                is VisualizationScreenState.Invalid -> ErrorScreen(state.error)
                VisualizationScreenState.Loading -> CircularProgressIndicator()
            }
        }

        // Show autoscroll toggle on event list view
        if (state is VisualizationScreenState.Valid.EventListView) {
            ToggleAutoscrollFab(
                autoScrollActive = autoScrollActive,
                onClick = { autoScrollActive = !autoScrollActive },
                modifier = Modifier.align(Alignment.BottomStart),
            )
        }

        // Show switch mode fab on incomplete or valid states
        if (
            state is VisualizationScreenState.Valid || state is VisualizationScreenState.Incomplete
        ) {
            SwitchModeFab(
                modifier = Modifier.align(Alignment.BottomEnd).padding(20.dp),
                onDisplayModeSelected = { viewModel.switchMode(it) },
            )
        }
    }
}

@Composable
fun ChartViewer(data: GraphedData) {
    when (data) {
        is GraphedData.TimeSeriesData ->
            VicoTimeSeries(seriesData = data.seriesData, chartMetadata = data.metaData)

        is GraphedData.HistogramData ->
            VicoBar(seriesData = data.seriesData, chartMetadata = data.metaData)

        GraphedData.EMPTY -> {}
        else -> TODO() // crash because we havent implemented the visualization type yet
    }
}
