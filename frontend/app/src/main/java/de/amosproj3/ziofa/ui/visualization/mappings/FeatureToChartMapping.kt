// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.mappings

import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.shared.HISTOGRAM_BUCKETS
import de.amosproj3.ziofa.ui.shared.TIME_SERIES_SIZE_VICO
import de.amosproj3.ziofa.ui.shared.TIME_SERIES_SIZE_YCHARTS
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_CHART_METADATA
import de.amosproj3.ziofa.ui.visualization.utils.aggregateToListPerTimeframe
import de.amosproj3.ziofa.ui.visualization.utils.averageMessageDuration
import de.amosproj3.ziofa.ui.visualization.utils.averagesPerFd
import de.amosproj3.ziofa.ui.visualization.utils.countFileDescriptors
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.lastValuePerTimeframe
import de.amosproj3.ziofa.ui.visualization.utils.sortAndClip
import de.amosproj3.ziofa.ui.visualization.utils.toCountedMultiSeries
import de.amosproj3.ziofa.ui.visualization.utils.toMillis
import de.amosproj3.ziofa.ui.visualization.utils.toMultiMemoryGraphData
import de.amosproj3.ziofa.ui.visualization.utils.toReferenceCount
import de.amosproj3.ziofa.ui.visualization.utils.toSeconds
import de.amosproj3.ziofa.ui.visualization.utils.toTimestampedSeries
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import timber.log.Timber

/*
 * When adding new features, the chart visualizations have to be configured in this file.
 */

/** Configures the metadata for charts for [BackendFeatureOptions]. */
fun DropdownOption.Metric.getChartMetadata(): ChartMetadata {
    return when (this.backendFeature) {
        is BackendFeatureOptions.VfsWriteOption ->
            ChartMetadata(yLabel = "Sum of bytes written", xLabel = "File Descriptor Name")

        is BackendFeatureOptions.SendMessageOption ->
            ChartMetadata(yLabel = "Average duration", xLabel = "Seconds since start")

        is BackendFeatureOptions.JniReferencesOption ->
            ChartMetadata(yLabel = "# Indirect References", xLabel = "Seconds since start")

        is BackendFeatureOptions.OpenFileDescriptors ->
            ChartMetadata(yLabel = "# Open File Descriptors", xLabel = "Seconds since start")

        is BackendFeatureOptions.GcOption ->
            ChartMetadata(yLabel = "Size of total heap / used heap", xLabel = "n-th GC invocation")

        else -> {
            Timber.e("needs metadata!")
            DEFAULT_CHART_METADATA
        }
    }
}

/**
 * Create a [Flow] of [GraphedData] for the given selection. The events should already be aggregated
 * in the flow. New features have to be mapped to their events here.
 */
fun DataStreamProvider.getChartData(
    selectedComponent: DropdownOption,
    selectedMetric: DropdownOption.Metric,
    selectedTimeframe: DropdownOption.Timeframe,
    chartMetadata: ChartMetadata,
): Flow<GraphedData>? {

    val pids = selectedComponent.getPIDsOrNull()
    val metric = selectedMetric.backendFeature

    return when (metric) {
        is BackendFeatureOptions.VfsWriteOption ->
            this.vfsWriteEvents(pids = pids)
                .aggregateToListPerTimeframe(selectedTimeframe.toMillis())
                .averagesPerFd()
                .sortAndClip(HISTOGRAM_BUCKETS)
                .map { GraphedData.HistogramData(it.toImmutableList(), chartMetadata) }

        is BackendFeatureOptions.SendMessageOption ->
            this.sendMessageEvents(pids = pids)
                .aggregateToListPerTimeframe(selectedTimeframe.toMillis())
                .averageMessageDuration()
                .toTimestampedSeries(TIME_SERIES_SIZE_VICO, selectedTimeframe.toSeconds().toFloat())
                .map { GraphedData.TimeSeriesData(it.toImmutableList(), chartMetadata) }

        is BackendFeatureOptions.JniReferencesOption ->
            this.jniReferenceEvents(pids = pids)
                .toReferenceCount()
                .lastValuePerTimeframe(selectedTimeframe.toMillis())
                .toTimestampedSeries(TIME_SERIES_SIZE_VICO, selectedTimeframe.toSeconds().toFloat())
                .map { GraphedData.TimeSeriesData(it.toImmutableList(), chartMetadata) }

        is BackendFeatureOptions.GcOption ->
            this.gcEvents(pids = pids)
                .toMultiMemoryGraphData()
                .toCountedMultiSeries(TIME_SERIES_SIZE_YCHARTS)
                .map {
                    GraphedData.MultiTimeSeriesData(it.toImmutableList(), DEFAULT_CHART_METADATA)
                }

        is BackendFeatureOptions.OpenFileDescriptors ->
            this.fileDescriptorTrackingEvents(pids = pids)
                .countFileDescriptors()
                .lastValuePerTimeframe(selectedTimeframe.toMillis())
                .toTimestampedSeries(TIME_SERIES_SIZE_VICO, selectedTimeframe.amount.toFloat())
                .map {
                    GraphedData.TimeSeriesData(
                        seriesData = it.toImmutableList(),
                        metaData = chartMetadata,
                    )
                }

        else -> null
    }
}
