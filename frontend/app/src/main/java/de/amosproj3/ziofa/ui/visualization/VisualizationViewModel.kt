// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.BackendConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationUpdate
import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationScreenState
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_SELECTION_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_TIMEFRAME_OPTIONS
import de.amosproj3.ziofa.ui.visualization.utils.VisualizationDisplayMode
import de.amosproj3.ziofa.ui.visualization.utils.getChartMetadata
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.toBucketedHistogram
import de.amosproj3.ziofa.ui.visualization.utils.toEventList
import de.amosproj3.ziofa.ui.visualization.utils.toMovingAverage
import de.amosproj3.ziofa.ui.visualization.utils.toUIOptions
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import timber.log.Timber

class VisualizationViewModel(
    backendConfigurationAccess: BackendConfigurationAccess,
    runningComponentsAccess: RunningComponentsAccess,
    dataStreamProviderFactory: (CoroutineScope) -> DataStreamProvider,
) : ViewModel() {

    private val dataStreamProvider = dataStreamProviderFactory(viewModelScope)

    // Mutable state
    private val selectedComponent = MutableStateFlow<DropdownOption>(DropdownOption.Global)
    private val selectedMetric = MutableStateFlow<DropdownOption.MetricOption?>(null)
    private val selectedTimeframe = MutableStateFlow<DropdownOption.TimeframeOption?>(null)
    private val displayMode = MutableStateFlow(VisualizationDisplayMode.EVENTS)

    // Derived selection data
    // This needs to be a StateFlow or else we will update the graphed data with every process list
    // update and get flickering on the UI
    private val selectionData =
        combine(
                selectedComponent,
                selectedMetric,
                selectedTimeframe,
                runningComponentsAccess.runningComponentsList,
                backendConfigurationAccess.backendConfiguration,
            ) { activeComponent, activeMetric, activeTimeframe, runningComponents, backendConfig ->
                if (backendConfig !is ConfigurationUpdate.Valid)
                    return@combine DEFAULT_SELECTION_DATA
                val configuredComponents =
                    runningComponents.filter { runningComponent ->
                        backendConfig.toUIOptionsForPids(runningComponent.pids).any { it.active }
                    }
                SelectionData(
                    selectedComponent = activeComponent,
                    selectedMetric = activeMetric,
                    selectedTimeframe = activeTimeframe,
                    componentOptions = configuredComponents.toUIOptions(),
                    metricOptions =
                        backendConfig.getActiveMetricsForPids(
                            pids = activeComponent.getPIDsOrNull()
                        ),
                    timeframeOptions = if (activeMetric != null) DEFAULT_TIMEFRAME_OPTIONS else null,
                )
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, DEFAULT_SELECTION_DATA)

    /**
     * Based on the [selectionData] and [displayMode], switch to a new flow everytime it changes.
     * FlatMapLatest will cancel the last flow for us. If the selection is incomplete, show the
     * default.
     */
    @OptIn(ExperimentalCoroutinesApi::class)
    val visualizationScreenState =
        combine(selectionData, displayMode) { a, b -> a to b }
            .flatMapLatest { (selection, mode) ->
                Timber.i("Data flow changed!")
                if (
                    selection.selectedMetric != null &&
                        selection.selectedMetric is DropdownOption.MetricOption &&
                        selection.selectedTimeframe != null &&
                        selection.selectedTimeframe is DropdownOption.TimeframeOption
                ) {
                    getDisplayedData(
                            selectedComponent = selection.selectedComponent,
                            selectedMetric = selection.selectedMetric,
                            selectedTimeframe = selection.selectedTimeframe,
                            visualizationMetaData = selection.selectedMetric.getChartMetadata(),
                            mode = mode,
                        )
                        .onStart { Timber.i("Subscribed to displayed data") }
                        .onCompletion { Timber.i("Displayed data completed") }
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
    fun componentSelected(componentOption: DropdownOption) {
        Timber.i("filterSelected()")
        selectedComponent.value = componentOption
    }

    /** Called when a metric is selected */
    fun metricSelected(metricOption: DropdownOption) {
        Timber.i("metricSelected()")
        if (metricOption is DropdownOption.MetricOption) {
            selectedMetric.value = metricOption
        } else {
            throw IllegalArgumentException("Wrong usage of this method")
        }
    }

    /** Called when a timeframe is selected */
    fun timeframeSelected(timeframeOption: DropdownOption) {
        Timber.i("timeframeSelected()")
        if (timeframeOption is DropdownOption.TimeframeOption) {
            selectedTimeframe.value = timeframeOption
        } else {
            throw IllegalArgumentException("Wrong usage of this method")
        }
    }

    fun switchMode() {
        displayMode.update { prev ->
            when (prev) {
                VisualizationDisplayMode.EVENTS -> VisualizationDisplayMode.CHART
                VisualizationDisplayMode.CHART -> VisualizationDisplayMode.EVENTS
            }
        }
    }

    /** This needs improvement. Creates [Flow] of [GraphedData] based on the selection. */
    private suspend fun getDisplayedData(
        selectedComponent: DropdownOption,
        selectedMetric: DropdownOption.MetricOption,
        selectedTimeframe: DropdownOption.TimeframeOption,
        mode: VisualizationDisplayMode,
        visualizationMetaData: VisualizationMetaData,
    ): Flow<GraphedData> {

        val pids = selectedComponent.getPIDsOrNull()
        val metric = selectedMetric.backendFeature

        return if (mode == VisualizationDisplayMode.CHART) {
            when (metric) {
                is BackendFeatureOptions.VfsWriteOption ->
                    dataStreamProvider
                        .vfsWriteEvents(pids = pids)
                        .toBucketedHistogram(visualizationMetaData, selectedTimeframe)

                is BackendFeatureOptions.SendMessageOption ->
                    dataStreamProvider
                        .sendMessageEvents(pids = pids)
                        .toMovingAverage(visualizationMetaData, selectedTimeframe)

                else -> throw NotImplementedError("NOT IMPLEMENTED YET")
            }
        } else {
            when (metric) {
                is BackendFeatureOptions.VfsWriteOption ->
                    dataStreamProvider.vfsWriteEvents(pids = pids).toEventList()

                is BackendFeatureOptions.SendMessageOption ->
                    dataStreamProvider.sendMessageEvents(pids = pids).toEventList()

                else -> throw NotImplementedError("NOT IMPLEMENTED YET")
            }
        }
    }

    private fun ConfigurationUpdate.Valid.getActiveMetricsForPids(pids: List<UInt>?) =
        this.toUIOptionsForPids(pids).filter { it.active }.map { DropdownOption.MetricOption(it) }
}
