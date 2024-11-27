// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import kotlin.time.DurationUnit

enum class DisplayModes {
    CHART,
    EVENTS,
}

/** The maximum number of datapoints to show on screen */
const val TIME_SERIES_SIZE = 20

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
        filterOptions = listOf(DropdownOption.Global),
        metricOptions = null,
        timeframeOptions = null,
        selectedFilter = DropdownOption.Global,
        selectedMetric = null,
        selectedTimeframe = null,
    )

val DEFAULT_CHART_METADATA = // TODO replace with reasonable defaults
    VisualizationMetaData(visualizationTitle = "", xLabel = "x", yLabel = "y")

val OPTION_VFS_WRITE = DropdownOption.MetricOption("VFS Write")
val OPTION_SEND_MESSAGE_EVENTS = DropdownOption.MetricOption("Send Message Events Write")
