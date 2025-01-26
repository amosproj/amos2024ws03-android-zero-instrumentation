// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationState
import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.api.overlay.OverlayAction
import de.amosproj3.ziofa.api.overlay.OverlayController
import de.amosproj3.ziofa.api.overlay.OverlayState
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationAction
import de.amosproj3.ziofa.ui.visualization.data.VisualizationDisplayMode
import de.amosproj3.ziofa.ui.visualization.data.VisualizationScreenState
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_SELECTION_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_TIMEFRAME_OPTIONS
import de.amosproj3.ziofa.ui.visualization.utils.getActiveMetricsForPids
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.isValidSelection
import de.amosproj3.ziofa.ui.visualization.utils.toUIOptions
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import timber.log.Timber

class VisualizationViewModel(
    configurationAccess: ConfigurationAccess,
    runningComponentsAccess: RunningComponentsAccess,
    dataStreamProviderFactory: (CoroutineScope) -> DataStreamProvider,
    private val overlayController: OverlayController,
) : ViewModel() {

    data class VisualizationSettings(
        val selection: SelectionData,
        val mode: VisualizationDisplayMode,
        val overlaySettings: OverlaySettings,
        val overlayEnabled: Boolean,
    )

    private val dataStreamProvider = dataStreamProviderFactory(viewModelScope)

    // Mutable state
    private val selectedComponent = MutableStateFlow<DropdownOption?>(null)
    private val selectedMetric = MutableStateFlow<DropdownOption.Metric?>(null)
    private val selectedTimeframe = MutableStateFlow<DropdownOption.Timeframe?>(null)
    private val displayMode = MutableStateFlow(VisualizationDisplayMode.EVENTS)
    private val overlaySettings =
        overlayController.overlayState
            .map { it.overlaySettings }
            .stateIn(viewModelScope, SharingStarted.Eagerly, OverlaySettings())
    private val overlayEnabled =
        overlayController.overlayState
            .map { it is OverlayState.Enabled }
            .stateIn(viewModelScope, SharingStarted.Eagerly, false)

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
                val availableMetricsForComponent =
                    activeComponent?.let {
                        config.getActiveMetricsForPids(pids = it.getPIDsOrNull())
                    }
                val componentsDropdownOptions =
                    configuredComponents
                        .toUIOptions()
                        .plus(
                            activeComponent?.let { listOf(it) } ?: listOf()
                        ) // prevent the selected component from disappearing if process ends
                        .toSet() // convert to set to remove the duplicate if it is already in
                        // the list
                        .toImmutableList()
                SelectionData(
                    selectedComponent = activeComponent,
                    selectedMetric = activeMetric,
                    selectedTimeframe = activeTimeframe,
                    componentOptions = componentsDropdownOptions,
                    metricOptions = availableMetricsForComponent,
                    timeframeOptions = if (activeMetric != null) DEFAULT_TIMEFRAME_OPTIONS else null,
                )
            }
            .onEach {
                // Select the first available option automatically
                if (it.selectedComponent == null && it.componentOptions.isNotEmpty())
                    selectedComponent.value = it.componentOptions.first()
            }
            .onEach { Timber.i("updated selection data $it") }
            .stateIn(viewModelScope, SharingStarted.Lazily, DEFAULT_SELECTION_DATA)

    /**
     * Based on the [selectionData] and [displayMode], switch to a new flow everytime it changes.
     * FlatMapLatest will cancel the last flow for us. If the selection is incomplete, show the
     * default.
     */
    @OptIn(ExperimentalCoroutinesApi::class)
    val visualizationScreenState =
        combine(selectionData, displayMode, overlaySettings, overlayEnabled) { a, b, c, d ->
                VisualizationSettings(a, b, c, d)
            }
            .flatMapLatest {
                val selection = it.selection
                val selectedComponent = it.selection.selectedComponent
                val selectedMetric = it.selection.selectedMetric
                val selectedTimeframe = it.selection.selectedTimeframe
                val mode = it.mode

                Timber.i("Data flow changed!")
                if (isValidSelection(selectedComponent, selectedMetric, selectedTimeframe)) {
                    when (mode) {
                        VisualizationDisplayMode.CHART ->
                            dataStreamProvider
                                .getChartData(
                                    selectedComponent = selectedComponent,
                                    selectedMetric = selectedMetric,
                                    selectedTimeframe = selectedTimeframe,
                                    chartMetadata = selectedMetric.getChartMetadata(),
                                )
                                .toChartViewOrNull(selection)

                        VisualizationDisplayMode.EVENTS ->
                            dataStreamProvider
                                .getEventListData(
                                    selectedComponent = selectedComponent,
                                    selectedMetric = selectedMetric,
                                )
                                .toEventListViewOrNull(selection)

                        VisualizationDisplayMode.OVERLAY -> it.overlayLauncher()
                    } ?: it.noVisualizationExists()
                } else {
                    it.waitingForMetricSelection()
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
    private fun updateSelectionState(option: DropdownOption) {
        when (option) {
            is DropdownOption.App,
            is DropdownOption.Process -> selectedComponent.value = option

            is DropdownOption.Metric -> selectedMetric.value = option
            is DropdownOption.Timeframe -> selectedTimeframe.value = option
            else -> throw NotImplementedError("unsupported selection")
        }
    }

    private fun updateOverlayState(shouldBeActive: Boolean) {
        viewModelScope.launch {
            if (shouldBeActive) {
                overlayController.dispatchAction(OverlayAction.Enable(selectionData.value))
            } else {
                overlayController.dispatchAction(OverlayAction.Disable)
            }
        }
    }

    private fun changeOverlaySettings(newSettings: OverlaySettings) {
        viewModelScope.launch {
            overlayController.dispatchAction(OverlayAction.ChangeSettings(newSettings))
        }
    }

    fun processAction(action: VisualizationAction) {
        when (action) {
            is VisualizationAction.OptionChanged -> updateSelectionState(action.option)
            is VisualizationAction.ModeChanged -> this.displayMode.value = action.newMode
            is VisualizationAction.OverlayStatusChanged -> updateOverlayState(action.newState)
            is VisualizationAction.OverlaySettingsChanged ->
                changeOverlaySettings(action.newSettings)
        }
    }

    private fun VisualizationSettings.waitingForMetricSelection() =
        flowOf(
            VisualizationScreenState.Incomplete.WaitingForMetricSelection(this.selection, this.mode)
        )

    private fun VisualizationSettings.noVisualizationExists() =
        flowOf(VisualizationScreenState.Incomplete.NoVisualizationExists(this.selection, this.mode))

    private fun VisualizationSettings.overlayLauncher() =
        flowOf(
            VisualizationScreenState.Valid.OverlayView(
                this.selection,
                this.overlaySettings,
                this.overlayEnabled,
            )
        )

    private fun Flow<GraphedData>?.toChartViewOrNull(selection: SelectionData) =
        this?.map { VisualizationScreenState.Valid.ChartView(it, selection) }

    private fun Flow<GraphedData.EventListData>?.toEventListViewOrNull(
        selection: SelectionData
    ): Flow<VisualizationScreenState.Valid.EventListView>? {
        require(selection.selectedMetric is DropdownOption.Metric)
        return this?.map {
            VisualizationScreenState.Valid.EventListView(
                graphedData = it,
                selectionData = selection,
                eventListMetadata = selection.selectedMetric.getEventListMetadata(),
            )
        }
    }
}
