// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.ConfigurationAccess
import de.amosproj3.ziofa.api.ConfigurationUpdate
import de.amosproj3.ziofa.api.DataStreamProvider
import de.amosproj3.ziofa.api.WriteEvent
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_CHART_METADATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_GRAPHED_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_SELECTION_DATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_TIMEFRAME_OPTIONS
import de.amosproj3.ziofa.ui.visualization.utils.DisplayModes
import de.amosproj3.ziofa.ui.visualization.utils.OPTION_SEND_MESSAGE_EVENTS
import de.amosproj3.ziofa.ui.visualization.utils.OPTION_VFS_WRITE
import de.amosproj3.ziofa.ui.visualization.utils.TIME_SERIES_SIZE
import de.amosproj3.ziofa.ui.visualization.utils.sortAndClip
import de.amosproj3.ziofa.ui.visualization.utils.toAveragedDurationOverTimeframe
import de.amosproj3.ziofa.ui.visualization.utils.toBucketedData
import kotlin.time.toDuration
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import timber.log.Timber

class VisualizationViewModel(
    private val configurationManager: ConfigurationAccess,
    private val dataStreamProvider: DataStreamProvider,
) : ViewModel() {

    private val _displayMode = MutableStateFlow(DisplayModes.CHART)

    /** Selection data from which both the dropdowns and the displayed data is derived */
    private val _selectionData = MutableStateFlow(DEFAULT_SELECTION_DATA)

    /** Selection data for displaying the dropdowns */
    val selectionData =
        _selectionData.stateIn(
            viewModelScope,
            SharingStarted.Eagerly,
            initialValue = DEFAULT_SELECTION_DATA,
        )

    /** Chart metadata derived from selection */
    val chartMetadata =
        _selectionData
            .combine(_displayMode) { a, b -> a to b }
            .map { (selection, mode) ->
                if (
                    mode == DisplayModes.CHART &&
                        selection.selectedMetric is DropdownOption.MetricOption &&
                        selection.selectedTimeframe is DropdownOption.TimeframeOption
                ) {
                    when (selection.selectedMetric.metricName) {
                        OPTION_VFS_WRITE.metricName ->
                            VisualizationMetaData(
                                "Top file descriptors",
                                "File Descriptor Name",
                                "Sum of bytes written",
                            )

                        OPTION_SEND_MESSAGE_EVENTS.metricName ->
                            VisualizationMetaData(
                                "Average duration of messages",
                                "Seconds since start",
                                "Average duration in ms",
                            )

                        else -> {
                            Timber.e("needs metadata!")
                            DEFAULT_CHART_METADATA
                        }
                    }
                } else DEFAULT_CHART_METADATA
            }
            .stateIn(viewModelScope, SharingStarted.Eagerly, initialValue = DEFAULT_CHART_METADATA)

    /**
     * Based on the selection, switch to a new flow everytime it changes. FlatMapLatest will cancel
     * the last flow for us. If the selection is incomplete, show the default.
     */
    @OptIn(ExperimentalCoroutinesApi::class)
    val graphedData =
        _selectionData
            .combine(_displayMode) { a, b -> a to b }
            .flatMapLatest { (selection, mode) ->
                if (
                    selection.selectedMetric != null &&
                        selection.selectedMetric is DropdownOption.MetricOption &&
                        selection.selectedTimeframe != null &&
                        selection.selectedTimeframe is DropdownOption.TimeframeOption
                ) {
                    if (mode == DisplayModes.CHART) {
                        getGraphedDataForSelection(
                            selectedMetric = selection.selectedMetric,
                            selectedTimeframe = selection.selectedTimeframe,
                        )
                    } else {
                        getEventsForSelection(
                            selectedMetric = selection.selectedMetric,
                            selectedTimeframe = selection.selectedTimeframe,
                        )
                    }
                } else {
                    flowOf(DEFAULT_GRAPHED_DATA)
                }
            }
            .stateIn(viewModelScope, started = SharingStarted.Eagerly, DEFAULT_GRAPHED_DATA)

    private suspend fun getGraphedDataForSelection(
        selectedMetric: DropdownOption.MetricOption,
        selectedTimeframe: DropdownOption.TimeframeOption,
    ): Flow<GraphedData> {
        return when (selectedMetric.metricName) {
            OPTION_VFS_WRITE.metricName ->
                dataStreamProvider
                    .vfsWriteEvents(pids = listOf()) // TODO filter PIDS
                    .toBucketedData(
                        selectedTimeframe.amount
                            .toDuration(selectedTimeframe.unit)
                            .inWholeMilliseconds
                            .toULong()
                    )
                    .sortAndClip(10)
                    .map { GraphedData.HistogramData(it) }

            OPTION_SEND_MESSAGE_EVENTS.metricName ->
                dataStreamProvider
                    .sendMessageEvents(pids = listOf()) // TODO filter PIDS
                    .toAveragedDurationOverTimeframe(
                        TIME_SERIES_SIZE,
                        selectedTimeframe.amount
                            .toDuration(selectedTimeframe.unit)
                            .inWholeMilliseconds,
                    )
                    .map { GraphedData.TimeSeriesData(it) }

            else -> flowOf<GraphedData>(DEFAULT_GRAPHED_DATA)
        }
    }

    private suspend fun getEventsForSelection(
        selectedMetric: DropdownOption.MetricOption,
        selectedTimeframe: DropdownOption.TimeframeOption,
    ): Flow<GraphedData> {
        return when (selectedMetric.metricName) {
            OPTION_VFS_WRITE.metricName ->
                dataStreamProvider
                    .vfsWriteEvents(pids = listOf())
                    .scan(initial = listOf<WriteEvent.VfsWriteEvent>()) { prev, next ->
                        prev.plus(next)
                    }
                    .map { GraphedData.EventListData(it) }

            OPTION_SEND_MESSAGE_EVENTS.metricName ->
                dataStreamProvider
                    .sendMessageEvents(pids = listOf())
                    .scan(initial = listOf<WriteEvent.SendMessageEvent>()) { prev, next ->
                        prev.plus(next)
                    }
                    .map { GraphedData.EventListData(it) }

            else -> flowOf<GraphedData>(DEFAULT_GRAPHED_DATA)
        }
    }

    /** Called when a filter is selected */
    fun filterSelected(filterOption: DropdownOption) {
        when (filterOption) {
            is DropdownOption.Global -> {
                displayGlobalOptions()
            }

            is DropdownOption.Process -> {
                displayGlobalOptions() // everything is per process or global for now
            }

            is DropdownOption.AppOption -> {
                displayGlobalOptions() // everything is per process or global for now
            }

            else -> Timber.e("Invalid option in filter list!!! $filterOption")
        }
        _selectionData.update { prev -> prev.copy(selectedFilter = filterOption) }
    }

    /** Called when a metric is selected */
    fun metricSelected(metricOption: DropdownOption) {
        if (metricOption is DropdownOption.MetricOption) {
            _selectionData.update {
                it.copy(
                    selectedMetric = metricOption,
                    timeframeOptions = DEFAULT_TIMEFRAME_OPTIONS, // display timeframe options
                )
            }
        } else {
            Timber.e("Invalid option in metric list!!! $metricOption")
        }
    }

    /** Called when a timeframe is selected */
    fun timeframeSelected(timeframeOption: DropdownOption) {
        if (timeframeOption is DropdownOption.TimeframeOption) {
            _selectionData.update { it.copy(selectedTimeframe = timeframeOption) }
        } else {
            Timber.e("Invalid option in timeframe list!!! $timeframeOption")
        }
    }

    fun switchMode() {
        _displayMode.update { prev ->
            if (prev == DisplayModes.CHART) DisplayModes.EVENTS else DisplayModes.CHART
        }
    }

    /** Display the dropdown with the global options based on [ConfigurationUpdate] */
    private fun displayGlobalOptions() {
        val configurationUpdate = configurationManager.configuration.value
        if (configurationUpdate is ConfigurationUpdate.Valid) {
            // TODO This needs to filter for global entries only
            _selectionData.update { prev ->
                prev.copy(
                    metricOptions =
                        configurationUpdate.configuration.let {
                            val displayedOptions = mutableListOf<DropdownOption.MetricOption>()
                            if (it.vfsWrite != null) {
                                displayedOptions.add(OPTION_VFS_WRITE)
                            }
                            if (it.sysSendmsg != null) {
                                displayedOptions.add(OPTION_SEND_MESSAGE_EVENTS)
                            }
                            displayedOptions.toList()
                        }
                )
            }
        }
    }
}
