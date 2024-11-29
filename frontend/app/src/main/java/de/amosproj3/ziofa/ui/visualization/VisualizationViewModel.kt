// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.BackendConfigurationAccess
import de.amosproj3.ziofa.api.ConfigurationUpdate
import de.amosproj3.ziofa.api.DataStreamProvider
import de.amosproj3.ziofa.api.RunningComponentsAccess
import de.amosproj3.ziofa.ui.shared.AccessedFromUI
import de.amosproj3.ziofa.ui.shared.HISTOGRAM_BUCKETS
import de.amosproj3.ziofa.ui.shared.TIME_SERIES_SIZE
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationScreenState
import de.amosproj3.ziofa.ui.visualization.utils.BackendFeature
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_GRAPHED_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_SELECTION_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_TIMEFRAME_OPTIONS
import de.amosproj3.ziofa.ui.visualization.utils.VisualizationDisplayMode
import de.amosproj3.ziofa.ui.visualization.utils.accumulateEvents
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.sortAndClip
import de.amosproj3.ziofa.ui.visualization.utils.toAveragedDurationOverTimeframe
import de.amosproj3.ziofa.ui.visualization.utils.toBucketedData
import de.amosproj3.ziofa.ui.visualization.utils.toFilterOptions
import kotlin.time.toDuration
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update

class VisualizationViewModel(
    private val backendConfigurationAccess: BackendConfigurationAccess,
    private val dataStreamProvider: DataStreamProvider,
    private val runningComponentsAccess: RunningComponentsAccess,
) : ViewModel() {

    // Mutable state
    private val selectedFilter = MutableStateFlow<DropdownOption>(DropdownOption.Global)
    private val selectedMetric = MutableStateFlow<DropdownOption.MetricOption?>(null)
    private val selectedTimeframe = MutableStateFlow<DropdownOption.TimeframeOption?>(null)
    private val displayMode = MutableStateFlow(VisualizationDisplayMode.EVENTS)

    // Derived selection data
    private val selectionData =
        combine(
            selectedFilter,
            selectedMetric,
            selectedTimeframe,
            runningComponentsAccess.runningComponentsList,
            backendConfigurationAccess.backendConfiguration,
        ) { filter, metric, timeframe, runningComponents, backendConfig ->
            if (backendConfig !is ConfigurationUpdate.Valid) return@combine DEFAULT_SELECTION_DATA
            val configuredComponents =
                runningComponents.filter { runningComponent ->
                    backendConfig.toUIOptionsForPids(runningComponent.pids).any { it.active }
                }
            SelectionData(
                selectedFilter = filter,
                selectedMetric = metric,
                selectedTimeframe = timeframe,
                filterOptions = configuredComponents.toFilterOptions(),
                metricOptions =
                    backendConfig
                        .toUIOptionsForPids(filter.getPIDsOrNull())
                        .filter { it.active }
                        .map { DropdownOption.MetricOption(it.featureName) },
                timeframeOptions = metric?.let { DEFAULT_TIMEFRAME_OPTIONS },
            )
        }

    /**
     * Based on the [selectionData] and [displayMode], switch to a new flow everytime it changes.
     * FlatMapLatest will cancel the last flow for us. If the selection is incomplete, show the
     * default.
     */
    @OptIn(ExperimentalCoroutinesApi::class)
    @AccessedFromUI
    val visualizationScreenState =
        combine(selectionData, displayMode) { a, b -> a to b }
            .flatMapLatest { (selection, mode) ->
                if (
                    selection.selectedMetric != null &&
                        selection.selectedMetric is DropdownOption.MetricOption &&
                        selection.selectedTimeframe != null &&
                        selection.selectedTimeframe is DropdownOption.TimeframeOption
                ) {
                    getDisplayedData(
                            selectedFilter = selection.selectedFilter,
                            selectedMetric = selection.selectedMetric,
                            selectedTimeframe = selection.selectedTimeframe,
                            visualizationMetaData =
                                BackendFeature.getChartMetadata(selection.selectedMetric),
                            mode = mode,
                        )
                        .map { VisualizationScreenState.MetricSelectionValid(it, selection, mode) }
                } else {
                    flowOf(VisualizationScreenState.WaitingForMetricSelection(selection, mode))
                }
            }
            .stateIn(
                viewModelScope,
                started = SharingStarted.Eagerly,
                VisualizationScreenState.Loading,
            )

    /** Called when a filter is selected */
    @AccessedFromUI
    fun filterSelected(filterOption: DropdownOption) {
        selectedFilter.value = filterOption
    }

    /** Called when a metric is selected */
    @AccessedFromUI
    fun metricSelected(metricOption: DropdownOption) {
        if (metricOption is DropdownOption.MetricOption) {
            selectedMetric.value = metricOption
        } else {
            throw IllegalArgumentException("Wrong usage of this method")
        }
    }

    /** Called when a timeframe is selected */
    @AccessedFromUI
    fun timeframeSelected(timeframeOption: DropdownOption) {
        if (timeframeOption is DropdownOption.TimeframeOption) {
            selectedTimeframe.value = timeframeOption
        } else {
            throw IllegalArgumentException("Wrong usage of this method")
        }
    }

    @AccessedFromUI
    fun switchMode() {
        displayMode.update { prev ->
            if (prev == VisualizationDisplayMode.CHART) VisualizationDisplayMode.EVENTS
            else VisualizationDisplayMode.CHART
        }
    }

    /** This needs improvement. Creates [Flow] of [GraphedData] based on the selection. */
    private suspend fun getDisplayedData(
        selectedFilter: DropdownOption,
        selectedMetric: DropdownOption.MetricOption,
        selectedTimeframe: DropdownOption.TimeframeOption,
        mode: VisualizationDisplayMode,
        visualizationMetaData: VisualizationMetaData,
    ): Flow<GraphedData> {

        val pids = selectedFilter.getPIDsOrNull()
        return when (selectedMetric) {
            BackendFeature.VFS_WRITE ->
                dataStreamProvider.vfsWriteEvents(pids = pids).let { events ->
                    if (mode == VisualizationDisplayMode.CHART) {
                        events
                            .toBucketedData(
                                selectedTimeframe.amount
                                    .toDuration(selectedTimeframe.unit)
                                    .inWholeMilliseconds
                                    .toULong()
                            )
                            .sortAndClip(HISTOGRAM_BUCKETS)
                            .map { GraphedData.HistogramData(it, visualizationMetaData) }
                    } else {
                        events.accumulateEvents().map { GraphedData.EventListData(it) }
                    }
                }

            BackendFeature.SEND_MESSAGE ->
                dataStreamProvider.sendMessageEvents(pids = pids).let {
                    if (mode == VisualizationDisplayMode.CHART) {
                        it.toAveragedDurationOverTimeframe(
                                TIME_SERIES_SIZE,
                                selectedTimeframe.amount
                                    .toDuration(selectedTimeframe.unit)
                                    .inWholeMilliseconds,
                            )
                            .map { GraphedData.TimeSeriesData(it, visualizationMetaData) }
                    } else {
                        it.accumulateEvents().map { GraphedData.EventListData(it) }
                    }
                }

            else -> flowOf<GraphedData>(DEFAULT_GRAPHED_DATA)
        }
    }
}
