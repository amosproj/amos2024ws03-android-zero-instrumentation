// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import androidx.compose.ui.graphics.Color
import de.amosproj3.ziofa.client.Event
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.SelectionData
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.data.EventListMetadata
import kotlin.time.DurationUnit
import timber.log.Timber

@Suppress("MagicNumber") // these are constants already
val DEFAULT_TIMEFRAME_OPTIONS =
    listOf(
        DropdownOption.Timeframe(500, DurationUnit.MILLISECONDS),
        DropdownOption.Timeframe(1, DurationUnit.SECONDS),
        DropdownOption.Timeframe(2, DurationUnit.SECONDS),
        DropdownOption.Timeframe(5, DurationUnit.SECONDS),
        DropdownOption.Timeframe(10, DurationUnit.SECONDS),
        DropdownOption.Timeframe(20, DurationUnit.SECONDS),
        DropdownOption.Timeframe(30, DurationUnit.SECONDS),
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
    ChartMetadata(xLabel = "x", yLabel = "y")

val DEFAULT_EVENT_LIST_METADATA = EventListMetadata("unknown", "unknown", "unknown", "unknown")

fun DropdownOption.Metric.getChartMetadata(): ChartMetadata {
    return when (this.backendFeature) {
        is BackendFeatureOptions.VfsWriteOption ->
            ChartMetadata("Top file descriptors", "File Descriptor Name")

        is BackendFeatureOptions.SendMessageOption ->
            ChartMetadata("Average duration of messages", "Seconds since start")

        else -> {
            Timber.e("needs metadata!")
            DEFAULT_CHART_METADATA
        }
    }
}

fun DropdownOption.Metric.getEventListMetadata(): EventListMetadata {
    return when (this.backendFeature) {
        is BackendFeatureOptions.VfsWriteOption ->
            EventListMetadata(
                label1 = "Process ID",
                label2 = "File Descriptor",
                label3 = "Event time since Boot in s",
                label4 = "Size in byte"
            )

        is BackendFeatureOptions.SendMessageOption ->
            EventListMetadata(
                label1 = "Process ID",
                label2 = "File Descriptor",
                label3 = "Event time since Boot in s",
                label4 = "Duration in ms"
            )

        is BackendFeatureOptions.JniReferencesOption -> EventListMetadata(
            label1 = "Process ID",
            label2 = "Thread ID",
            label3 = "Event time since Boot in s",
            label4 = "JNI Method Name"
        )

        else -> {
            Timber.e("needs metadata!")
            DEFAULT_EVENT_LIST_METADATA
        }
    }
}

const val LIGHT_PURPLE = 0xffa485e0
val VICO_LINE_COLOR = Color(LIGHT_PURPLE)
