// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import kotlin.time.DurationUnit
import timber.log.Timber

enum class VisualizationDisplayMode {
    CHART,
    EVENTS,
}

val DEFAULT_TIMEFRAME_OPTIONS =
    listOf(
        DropdownOption.TimeframeOption(500, DurationUnit.MILLISECONDS),
        DropdownOption.TimeframeOption(1, DurationUnit.SECONDS),
        DropdownOption.TimeframeOption(2, DurationUnit.SECONDS),
        DropdownOption.TimeframeOption(5, DurationUnit.SECONDS),
        DropdownOption.TimeframeOption(10, DurationUnit.SECONDS),
        DropdownOption.TimeframeOption(20, DurationUnit.SECONDS),
        DropdownOption.TimeframeOption(30, DurationUnit.SECONDS),
    )

val DEFAULT_TIMESERIES_DATA = listOf(-1f to -1f) // TODO replace with reasonable defaults
val DEFAULT_GRAPHED_DATA = GraphedData.EMPTY // TODO replace with reasonable defaults

val DEFAULT_SELECTION_DATA =
    SelectionData(
        componentOptions = listOf(DropdownOption.Global),
        metricOptions = null,
        timeframeOptions = null,
        selectedComponent = DropdownOption.Global,
        selectedMetric = null,
        selectedTimeframe = null,
    )

val DEFAULT_CHART_METADATA = // TODO replace with reasonable defaults
    VisualizationMetaData(xLabel = "x", yLabel = "y")

fun DropdownOption.MetricOption.getChartMetadata(): VisualizationMetaData {
    return when (this.backendFeature) {
        is BackendFeatureOptions.VfsWriteOption ->
            VisualizationMetaData("Top file descriptors", "File Descriptor Name")

        is BackendFeatureOptions.SendMessageOption ->
            VisualizationMetaData("Average duration of messages", "Seconds since start")

        else -> {
            Timber.e("needs metadata!")
            DEFAULT_CHART_METADATA
        }
    }
}
