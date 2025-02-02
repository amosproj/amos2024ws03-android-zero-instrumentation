// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import androidx.compose.ui.graphics.Color
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.EventListMetadata
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import kotlin.time.DurationUnit
import kotlinx.collections.immutable.persistentListOf

/** The options displayed on the time*/
@Suppress("MagicNumber") // these are constants already
val DEFAULT_TIMEFRAME_OPTIONS =
    persistentListOf(
        DropdownOption.Timeframe(1, DurationUnit.SECONDS),
        DropdownOption.Timeframe(2, DurationUnit.SECONDS),
        DropdownOption.Timeframe(5, DurationUnit.SECONDS),
        DropdownOption.Timeframe(10, DurationUnit.SECONDS),
        DropdownOption.Timeframe(20, DurationUnit.SECONDS),
        DropdownOption.Timeframe(30, DurationUnit.SECONDS),
    )

/** The time series data to show if there is no data yet*/
val DEFAULT_TIMESERIES_DATA = persistentListOf(-1f to -1f)

/** Graphed data to show if there is no data yet */
val DEFAULT_GRAPHED_DATA = GraphedData.EMPTY

/** Selection data before there is any selection made */
val DEFAULT_SELECTION_DATA =
    SelectionData(
        componentOptions = persistentListOf(),
        metricOptions = null,
        timeframeOptions = null,
        selectedComponent = null,
        selectedMetric = null,
        selectedTimeframe = null,
    )

/** Chart metadata to show if there are no labels mapped for a graph */
val DEFAULT_CHART_METADATA =
    ChartMetadata(xLabel = "x", yLabel = "y")

/** Event list labels to show if there are no labels mapped for a graph */
val DEFAULT_EVENT_LIST_METADATA = EventListMetadata("unknown", "unknown", "unknown", "unknown")

/** RGB Hex value for light purple */
const val LIGHT_PURPLE = 0xffa485e0

/** RGB Hex value for light yellow */
const val LIGHT_YELLOW = 0xFFFFED

/** The color of the primary line in the visualization */
val VICO_LINE_COLOR = Color(LIGHT_PURPLE)
