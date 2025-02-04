// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.client.Event
import kotlin.math.max
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.scan

/** Emit the last value emitted in the given timeframe (in milliseconds) every [timeframeMillis] */
fun <T> Flow<T>.lastValuePerTimeframe(timeframeMillis: Long): Flow<T> = flow {
    var lastValue: T? = null
    this@lastValuePerTimeframe.onEach { lastValue = it }.launchIn(CoroutineScope(Dispatchers.IO))
    while (true) {
        val lv = lastValue
        if (lv != null) {
            emit(lv)
        }
        delay(timeframeMillis)
    }
}

/** Emit a list of all values emitted in the given timeframe (in milliseconds) every [timeframeMillis] */
fun <T> Flow<T>.aggregateToListPerTimeframe(timeframeMillis: Long): Flow<List<T>> = flow {
    val buffer = mutableListOf<T>()
    this@aggregateToListPerTimeframe.onEach { buffer.add(it) }
        .launchIn(CoroutineScope(Dispatchers.IO))
    while (true) {
        emit(buffer.toList())
        buffer.clear()
        delay(timeframeMillis)
    }
}

/** Accumulate all events in the flow into a list up to a maximum of length of [limit] */
fun <E> Flow<E>.accumulateEvents(limit: Int = 1000) =
    this.scan(initial = listOf<E>()) { prev, next -> prev.plus(next).takeLast(limit) }

/** Convert a flow of numbers to an indexed series of value index pairs, limited to [seriesSize]
 * @param secondsPerDatapoint How often data is emitted from the input flow. For example, if this is
 * 2, the series will look like this:
 * (2, 1), (4, 2), (6, 3), ...
 */
fun Flow<Number>.toTimestampedSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Float>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to next.toFloat()).takeLast(seriesSize)
    }

/** Calculate the average message duration of the given flow of Lists of [Event.SysSendmsg]
 * for each list. If the list is empty, the average is 0. */
fun Flow<List<Event.SysSendmsg>>.averageMessageDuration() =
    this.map {
        if (it.isNotEmpty()) {
            val avg = it.map { it.duration.inWholeNanoseconds }.average()
            if (avg != 0.0) avg.nanosToMillis() else avg
        } else 0.0
    }

/** Calculate the average of bytes written to each file descriptor in the given flow of Lists of [Event.VfsWrite] */
fun Flow<List<Event.VfsWrite>>.averagesPerFd() =
    this.map {
        it.groupBy { it.fp }
            .map { it.key to it.value.map { it.bytesWritten.toLong() }.sum().toDouble() }
    }

/** Calculate the number of active JNI references based on the given flow of [Event.JniReferences]
 * This is a count that will be incremented on a Add*Ref and decremented on a Delete*Ref.
 * */
fun Flow<Event.JniReferences>.toReferenceCount() =
    this.scan(0 to 0) { prev, next ->
            when (next.jniMethodName) {
                Event.JniReferences.JniMethodName.AddLocalRef -> prev.first + 1 to prev.second
                Event.JniReferences.JniMethodName.DeleteLocalRef -> prev.first - 1 to prev.second
                Event.JniReferences.JniMethodName.AddGlobalRef -> prev.first to prev.second + 1
                Event.JniReferences.JniMethodName.DeleteGlobalRef -> prev.first to prev.second - 1
                else -> prev
            }
        }
        .map { it.first + it.second }

/** Count the number of file descriptors that are currently open based on the given flow of [Event.SysFdTracking].
 * This is a count that will be incremented on a SysFdTracking.Created and decremented on a SysFdTracking.Destroyed.
 * */
fun Flow<Event.SysFdTracking>.countFileDescriptors() =
    this.scan(0 to 0) { prev, next ->
            when (next.fdAction) {
                Event.SysFdTracking.SysFdAction.Created -> prev.first + 1 to prev.second
                Event.SysFdTracking.SysFdAction.Destroyed -> prev.first - 1 to prev.second
                else -> prev
            }
        }
        .map { it.first + it.second }

/** Convert a flow of [Event.Gc] to a a flow of a pair of used heap size and total heap size for
 * each GC event. This derivation from the GC events is how Perfetto does it. */
fun Flow<Event.Gc>.toMultiMemoryGraphData() =
    this.map { it.numBytesAllocated to max(it.targetFootprint, it.numBytesAllocated) }

/** Like [toTimestampedSeries], but for MultiGraphData.
 * @see toTimestampedSeries */
fun Flow<Pair<ULong, ULong>>.toTimestampedMultiSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Pair<Float, Float>>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to (next.first.toFloat() to next.second.toFloat())).takeLast(seriesSize)
    }

/** Like [toTimestampedMultiSeries], but for indexes instead of timestamps.
 * @see toTimestampedMultiSeries */
fun Flow<Pair<ULong, ULong>>.toCountedMultiSeries(seriesSize: Int) =
    this.scan(listOf<Pair<Float, Pair<Float, Float>>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + 1
        prev.plus(idx to (next.first.toFloat() to next.second.toFloat())).takeLast(seriesSize)
    }

/** Sort the given flow of lists of [Pair] by the second element (y) and clip the result to the given [limit] */
fun Flow<List<Pair<String, Double>>>.sortAndClip(limit: Int) =
    this.map { it.sortedBy { (fd, size) -> size }.reversed().take(limit) }.conflate()

/** Check if the given list is the default series data */
fun List<Pair<Float, Float>>.isDefaultSeries(): Boolean {
    return this == DEFAULT_TIMESERIES_DATA
}
