// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import android.util.Log
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.ui.visualization.data.MetricOption
import de.amosproj3.ziofa.ui.visualization.data.PackageOption
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.TimeframeOption
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

// TODO add loadProgram from the backend and filter stream for respective metric
class VisualizationViewModel(private val clientFactory: ClientFactory) : ViewModel() {
    private val defaultGraphedData = listOf(0 to 0) // TODO replace with reasonable defaults
    private val _graphedData = MutableStateFlow(defaultGraphedData)
    val graphedData =
        _graphedData.stateIn(viewModelScope, started = SharingStarted.Eagerly, defaultGraphedData)

    private val defaultMetadata = // TODO replace with reasonable defaults
        VisualizationMetaData(
            visualizationTitle = "Packages per second",
            xLabel = "Seconds since start",
            yLabel = "#Packages",
        )

    val chartMetadata =
        flowOf(defaultMetadata)
            .stateIn(viewModelScope, SharingStarted.Eagerly, initialValue = defaultMetadata)

    private val defaultSelectionData = // TODO replace with reasonable defaults
        SelectionData(
            packageOptions =
                listOf(PackageOption("de.example.app", "Example app", logo = Icons.Filled.Info)),
            metricOptions = listOf(MetricOption("Packages per X")),
            timeframeOptions = listOf(TimeframeOption(1, TimeUnit.SECONDS)),
        )
    val selectionData =
        flowOf(defaultSelectionData)
            .stateIn(viewModelScope, SharingStarted.Eagerly, initialValue = defaultSelectionData)

    init {
        start()
    }

    /**
     * Connect to the backend and start updating the [_graphedData]. //TODO move connecting to
     * initialization (before home screen)
     */
    private fun start() {
        viewModelScope.launch {
            try {
                val counterFlow =
                    clientFactory
                        .connect(viewModelScope, "http://[::1]:50051")
                        .also {
                            // TODO: separate try catch because we have no good error handling yet
                            // the load, attach and startCollecting method return an error
                            // for AlreadyLoaded, AlreadyAttached, AlreadyCollecting which
                            // is typed on the rust side but not yet exported.
                            // In this case we just ignore all errors as we do not care
                            // about whether the daemon is already doing stuff as it is
                            // not managed by the apps lifecycle.
                            // If the counter does not work, it will error later.
                            try {
                                it.load()
                                // default wifi interface on android, now configurable
                                it.attach("wlan0")
                                it.startCollecting()
                            } catch (e: Exception) {
                                Log.e("Counter Error", e.stackTraceToString())
                            }
                        }
                        .serverCount
                        .stateIn(this, SharingStarted.Eagerly, 0u)

                packagesPerSecond(counterFlow).toIndexedTimeSeries(20).collect {
                    _graphedData.value = it
                }
            } catch (e: Exception) {
                Log.e("Counter Error", e.stackTraceToString())
            }
        }
    }

    /**
     * Obtain a flow that emits the amount of packages in the last second every second based on a
     * monotonically increasing [counterSource]. TODO generalize or replace
     */
    private fun packagesPerSecond(counterSource: StateFlow<UInt>) = flow {
        var previousCount = counterSource.value
        while (true) {
            delay(1000)
            val currentCount = counterSource.value
            val packagesLastSecond = currentCount - previousCount
            emit(packagesLastSecond)
            previousCount = currentCount
        }
    }

    /** Emit a indexed time series where the length of the list will never surpass [seriesSize]. */
    private fun Flow<UInt>.toIndexedTimeSeries(seriesSize: Int) =
        this.scan(listOf<Pair<Int, Int>>()) { prev, next ->
            val idx = ((prev.lastOrNull()?.first) ?: 0) + 1
            prev.plus(idx to next.toInt()).takeLast(seriesSize)
        }
}
