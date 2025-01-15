// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationState
import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationDisplayMode
import de.amosproj3.ziofa.ui.visualization.data.VisualizationScreenState
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_SELECTION_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_TIMEFRAME_OPTIONS
import de.amosproj3.ziofa.ui.visualization.utils.getActiveMetricsForPids
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.isValidSelection
import de.amosproj3.ziofa.ui.visualization.utils.toUIOptions
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import timber.log.Timber

class VisualizationViewModel(
    configurationAccess: ConfigurationAccess,
    runningComponentsAccess: RunningComponentsAccess,
    dataStreamProviderFactory: (CoroutineScope) -> DataStreamProvider,
) : ViewModel() {

    private val dataStreamProvider = dataStreamProviderFactory(viewModelScope)

    // Mutable state
    private val selectedComponent = MutableStateFlow<DropdownOption>(DropdownOption.Global)
    private val selectedMetric = MutableStateFlow<DropdownOption.Metric?>(null)
    private val selectedTimeframe = MutableStateFlow<DropdownOption.Timeframe?>(null)
    private val displayMode = MutableStateFlow(VisualizationDisplayMode.EVENTS)

    /**
     * Combine the mutable state with current configuration and running components to create a
     * dropdown options selecting filters. This needs to be a StateFlow or else we will update the
     * graphed data with every process list update and get flickering on the UI
     */
    private val selectionData =
        combine(
                selectedComponent,
                selectedMetric,
                selectedTimeframe,
                runningComponentsAccess.runningComponentsList,
                configurationAccess.configurationState,
            ) { activeComponent, activeMetric, activeTimeframe, runningComponents, configState ->
                val config =
                    when (configState) {
                        is ConfigurationState.Synchronized -> configState.configuration
                        is ConfigurationState.Different -> configState.backendConfiguration
                        else -> return@combine DEFAULT_SELECTION_DATA
                    }
                val configuredComponents =
                    runningComponents.filter { runningComponent ->
                        config.toUIOptionsForPids(runningComponent.pids).any { it.active }
                    }
                SelectionData(
                    selectedComponent = activeComponent,
                    selectedMetric = activeMetric,
                    selectedTimeframe = activeTimeframe,
                    componentOptions = configuredComponents.toUIOptions(),
                    metricOptions =
                        config.getActiveMetricsForPids(pids = activeComponent.getPIDsOrNull()),
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
                if (isValidSelection(selection.selectedMetric, selection.selectedTimeframe)) {
                    when (mode) {
                        VisualizationDisplayMode.CHART ->
                            dataStreamProvider
                                .getChartData(
                                    selectedComponent = selection.selectedComponent,
                                    selectedMetric = selection.selectedMetric,
                                    selectedTimeframe = selection.selectedTimeframe,
                                    chartMetadata = selection.selectedMetric.getChartMetadata(),
                                )
                                ?.map { VisualizationScreenState.Valid.ChartView(it, selection) }

                        VisualizationDisplayMode.EVENTS ->
                            dataStreamProvider
                                .getEventListData(
                                    selectedComponent = selection.selectedComponent,
                                    selectedMetric = selection.selectedMetric,
                                )
                                ?.map {
                                    VisualizationScreenState.Valid.EventListView(
                                        graphedData = it,
                                        selectionData = selection,
                                        eventListMetadata =
                                            selection.selectedMetric.getEventListMetadata(),
                                    )
                                }
                    }
                        ?: flowOf(
                            VisualizationScreenState.Incomplete.NoVisualizationExists(
                                selection,
                                mode,
                            )
                        )
                } else {
                    flowOf(
                        VisualizationScreenState.Incomplete.WaitingForMetricSelection(
                            selection,
                            mode,
                        )
                    )
                }
            }
            .stateIn(
                viewModelScope,
                started = SharingStarted.Eagerly,
                VisualizationScreenState.Loading,
            )

    /**
     * Called when the selection of a dropdown changes. The change should be reflected in the
     * displayed data.
     */
    fun optionSelected(option: DropdownOption) {
        when (option) {
            is DropdownOption.App,
            is DropdownOption.Process -> selectedComponent.value = option

            is DropdownOption.Metric -> selectedMetric.value = option
            is DropdownOption.Timeframe -> selectedTimeframe.value = option
            else -> throw NotImplementedError("unsupported selection")
        }
    }

    /** Set the active display mode. */
    fun switchMode(newMode: VisualizationDisplayMode) {
        displayMode.value = newMode
    }
}
