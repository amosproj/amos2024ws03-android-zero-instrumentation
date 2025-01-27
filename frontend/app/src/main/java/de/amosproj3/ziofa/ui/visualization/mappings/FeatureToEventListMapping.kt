package de.amosproj3.ziofa.ui.visualization.mappings

import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.EventListEntry
import de.amosproj3.ziofa.ui.visualization.data.EventListMetadata
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.utils.DEFAULT_EVENT_LIST_METADATA
import de.amosproj3.ziofa.ui.visualization.utils.accumulateEvents
import de.amosproj3.ziofa.ui.visualization.utils.bytesToHumanReadableSize
import de.amosproj3.ziofa.ui.visualization.utils.getPIDsOrNull
import de.amosproj3.ziofa.ui.visualization.utils.nanosToSecondsStr
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import timber.log.Timber

/*
 * When adding new features, the event visualizations have to be configured in this file.
 */

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

        is BackendFeatureOptions.OpenFileDescriptors ->
            EventListMetadata(
                label1 = "PID",
                label2 = "TID",
                label3 = "Event time since Boot in s",
                label4 = "Event type",
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
            this.vfsWriteEvents(pids = pids).map {
                EventListEntry(
                    col1 = "${it.pid}",
                    col2 = "${it.fp}",
                    col3 = it.beginTimeStamp.nanosToSecondsStr(),
                    col4 = "${it.bytesWritten}",
                )
            }

        is BackendFeatureOptions.SendMessageOption ->
            this.sendMessageEvents(pids = pids).map {
                EventListEntry(
                    col1 = "${it.pid}",
                    col2 = "${it.fd}",
                    col3 = it.beginTimeStamp.nanosToSecondsStr(),
                    col4 = it.durationNanoSecs.nanosToSecondsStr(),
                )
            }

        is BackendFeatureOptions.JniReferencesOption ->
            this.jniReferenceEvents(pids = pids).map {
                EventListEntry(
                    col1 = "${it.pid}",
                    col2 = "${it.tid}",
                    col3 = "${it.beginTimeStamp}",
                    col4 = it.jniMethodName!!.name, // TODO why is this nullable??
                )
            }

        is BackendFeatureOptions.SigquitOption ->
            this.sigquitEvents(pids = pids).map {
                EventListEntry(
                    col1 = "${it.pid}",
                    col2 = "${it.tid}",
                    col3 = "${it.targetPid}",
                    col4 = it.timeStamp.nanosToSecondsStr(),
                )
            }

        is BackendFeatureOptions.GcOption ->
            this.gcEvents(pids = pids).map {
                Int
                EventListEntry(
                    col1 = "${it.pid}",
                    col2 = it.targetFootprint.bytesToHumanReadableSize(),
                    col3 = it.numBytesAllocated.bytesToHumanReadableSize(),
                    col4 = it.freedBytes.bytesToHumanReadableSize(),
                )
            }

        is BackendFeatureOptions.OpenFileDescriptors ->
            this.fileDescriptorTrackingEvents(pids = pids).map {
                EventListEntry(
                    col1 = "${it.pid}",
                    col2 = "${it.tid}",
                    col3 = it.timeStamp.nanosToSecondsStr(),
                    col4 = it.fdAction?.name.toString(),
                )
            }

        else -> null
    }?.let { it.accumulateEvents().map { GraphedData.EventListData(it.toImmutableList()) } }
}
