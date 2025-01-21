// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization

import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.EventListEntry
import de.amosproj3.ziofa.ui.visualization.data.EventListMetadata
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_CHART_METADATA
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_EVENT_LIST_METADATA
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.nanosToSeconds
import de.amosproj3.ziofa.ui.visualization.utils.toBucketedHistogram
import de.amosproj3.ziofa.ui.visualization.utils.toCombinedReferenceCount
import de.amosproj3.ziofa.ui.visualization.utils.toEventList
import de.amosproj3.ziofa.ui.visualization.utils.toMovingAverage
import de.amosproj3.ziofa.ui.visualization.utils.toMultiMemoryGraphData
import de.amosproj3.ziofa.ui.visualization.utils.toTimestampedMultiSeries
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import timber.log.Timber
import kotlin.time.toDuration

/*
 * When adding new features, the event visualizations have to be configured in this file.
 */

/** Configures the metadata for charts for [BackendFeatureOptions]. */
fun DropdownOption.Metric.getChartMetadata(): ChartMetadata {
    return when (this.backendFeature) {
        is BackendFeatureOptions.VfsWriteOption ->
            ChartMetadata(yLabel = "Top file descriptors", xLabel = "File Descriptor Name")

        is BackendFeatureOptions.SendMessageOption ->
            ChartMetadata(yLabel = "Average duration of messages", xLabel = "Seconds since start")

        is BackendFeatureOptions.JniReferencesOption ->
            ChartMetadata(
                yLabel = "Number of JNI Indirect References",
                xLabel = "Events since start",
            )

        else -> {
            Timber.e("needs metadata!")
            DEFAULT_CHART_METADATA
        }
    }
}

/** Configures the headers for [BackendFeatureOptions]. */
fun DropdownOption.Metric.getEventListMetadata(): EventListMetadata {
    return when (this.backendFeature) {
        is BackendFeatureOptions.VfsWriteOption ->
            EventListMetadata(
                label1 = "Process ID",
                label2 = "File Descriptor",
                label3 = "Event time since Boot in s",
                label4 = "Size in byte",
            )

        is BackendFeatureOptions.SendMessageOption ->
            EventListMetadata(
                label1 = "Process ID",
                label2 = "File Descriptor",
                label3 = "Event time since Boot in s",
                label4 = "Duration in seconds",
            )

        is BackendFeatureOptions.JniReferencesOption ->
            EventListMetadata(
                label1 = "Process ID",
                label2 = "Thread ID",
                label3 = "Event time since Boot in s",
                label4 = "JNI Method Name",
            )

        is BackendFeatureOptions.SigquitOption ->
            EventListMetadata(
                label1 = "Origin PID",
                label2 = "Origin TID",
                label3 = "Target PID",
                label4 = "Timestamp",
            )

        is BackendFeatureOptions.GcOption ->
            EventListMetadata(
                label1 = "PID",
                label2 = "TargetFootprint",
                label3 = "Allocated bytes",
                label4 = "Freed bytes",
            )

        else -> {
            Timber.e("needs metadata!")
            DEFAULT_EVENT_LIST_METADATA
        }
    }
}

/**
 * Create a [Flow] of [GraphedData.EventListData] for the given selection. New features have to be
 * mapped to their events here.
 */
fun DataStreamProvider.getEventListData(
    selectedComponent: DropdownOption,
    selectedMetric: DropdownOption.Metric,
): Flow<GraphedData.EventListData>? {
    val pids = selectedComponent.getPIDsOrNull()
    val metric = selectedMetric.backendFeature
    return when (metric) {
        is BackendFeatureOptions.VfsWriteOption ->
            this.vfsWriteEvents(pids = pids)
                .map {
                    EventListEntry(
                        col1 = "${it.fp}",
                        col2 = "${it.pid}",
                        col3 = it.beginTimeStamp.nanosToSeconds(),
                        col4 = "${it.bytesWritten}",
                    )
                }
                .toEventList()

        is BackendFeatureOptions.SendMessageOption ->
            this.sendMessageEvents(pids = pids)
                .map {
                    EventListEntry(
                        col1 = "${it.pid}",
                        col2 = "${it.fd}",
                        col3 = it.beginTimeStamp.nanosToSeconds(),
                        col4 = it.durationNanoSecs.nanosToSeconds(),
                    )
                }
                .toEventList()

        is BackendFeatureOptions.JniReferencesOption ->
            this.jniReferenceEvents(pids = pids)
                .map {
                    EventListEntry(
                        col1 = "${it.pid}",
                        col2 = "${it.tid}",
                        col3 = "${it.beginTimeStamp}",
                        col4 = it.jniMethodName!!.name, // TODO why is this nullable??
                    )
                }
                .toEventList()

        is BackendFeatureOptions.SigquitOption ->
            this.sigquitEvents(pids = pids)
                .map {
                    EventListEntry(
                        col1 = "${it.pid}",
                        col2 = "${it.tid}",
                        col3 = "${it.targetPid}",
                        col4 = it.timeStamp.nanosToSeconds(),
                    )
                }
                .toEventList()

        is BackendFeatureOptions.GcOption ->
            this.gcEvents(pids = pids)
                .map {
                    EventListEntry(
                        col1 = "${it.pid}",
                        col2 = "${it.targetFootprint}",
                        col3 = "${it.numBytesAllocated}",
                        col4 = "${it.freedBytes}"
                    )
                }
                .toEventList()

        else -> null
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
            this.vfsWriteEvents(pids = pids).toBucketedHistogram(chartMetadata, selectedTimeframe)

        is BackendFeatureOptions.SendMessageOption ->
            this.sendMessageEvents(pids = pids)
                .toMovingAverage(chartMetadata, selectedTimeframe)
                .map {
                    it.copy(
                        seriesData =
                        it.seriesData
                            .map { pair -> pair.copy(second = pair.second / 1_000_000) }
                            .toImmutableList()
                    )
                }

        is BackendFeatureOptions.JniReferencesOption ->
            this.jniReferenceEvents(pids = pids)
                .toCombinedReferenceCount(chartMetadata, selectedTimeframe)


        is BackendFeatureOptions.GcOption ->
            this.gcEvents(pids = pids)
                .toMultiMemoryGraphData(selectedTimeframe.amount.toDuration(selectedTimeframe.unit).inWholeMilliseconds)
                .toTimestampedMultiSeries(
                    10,
                    selectedTimeframe.amount.toDuration(selectedTimeframe.unit).inWholeSeconds.toFloat()
                )
                .map { GraphedData.MultiTimeSeriesData(it.toImmutableList(), DEFAULT_CHART_METADATA) }

        else -> null
    }
}
