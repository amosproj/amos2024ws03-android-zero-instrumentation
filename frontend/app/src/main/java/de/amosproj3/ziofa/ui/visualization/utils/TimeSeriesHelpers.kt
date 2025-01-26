// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import android.os.SystemClock
import androidx.compose.ui.text.intl.Locale
import de.amosproj3.ziofa.client.Event
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import kotlin.math.max
import kotlin.time.toDuration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.buffer
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.scan
import kotlinx.coroutines.withTimeout

fun Flow<Number>.toTimestampedSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Float>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to next.toFloat()).takeLast(seriesSize)
    }

fun Flow<Event.SysSendmsg>.toAveragedDurationOverTimeframe(
    seriesSize: Int,
    millisTimeframeDuration: Long,
) =
    this.buffer().windowed(millisTimeframeDuration).scan(listOf<Pair<Float, Float>>()) { prev, next
        ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + 1
        prev.plus(idx to next.toFloat()).takeLast(seriesSize)
    }

fun Flow<Event>.collect1SecondFrames() = flow {
    val list = mutableListOf<Event>()
    while (true) {
        withTimeout(1000) { this@collect1SecondFrames.collect { list.add(it) } }
        list.clear()
        emit(list.toList())
    }
}

fun <T> Flow<T>.emitLastValueOfTimeframe(timeframeMillis: Long): Flow<T> = flow {
    var lastValue: T? = null
    this@emitLastValueOfTimeframe.onEach { lastValue = it }.launchIn(CoroutineScope(Dispatchers.IO))
    while (true) {
        val lv = lastValue
        if (lv != null) {
            emit(lv)
        }
        delay(timeframeMillis)
    }
}

fun <T> Flow<T>.emitTimeframeLists(timeframeMillis: Long): Flow<List<T>> = flow {
    val buffer = mutableListOf<T>()
    this@emitTimeframeLists.onEach { buffer.add(it) }.launchIn(CoroutineScope(Dispatchers.IO))
    while (true) {
        emit(buffer.toList())
        delay(timeframeMillis)
    }
}

fun Flow<List<Event.VfsWrite>>.averagesPerFd() =
    this.map {
        it.groupBy { it.fp }
            .map { it.key to it.value.map { it.bytesWritten.toLong() }.sum().toDouble() }
    }

fun DropdownOption.Timeframe.toMillis() = this.amount.toDuration(this.unit).inWholeMilliseconds

fun Flow<Event.SysSendmsg>.windowed(windowMillis: Long): Flow<Double> = flow {
    val buffer = mutableListOf<ULong>()
    var windowStart = System.currentTimeMillis()

    this@windowed.collect { value ->
        val now = System.currentTimeMillis()
        buffer.add(value.durationNanoSecs)

        if (now - windowStart >= windowMillis) {
            val average = buffer.map { it.toFloat() }.average()
            emit(average)
            buffer.clear() // Clear the buffer for the next window
            windowStart = now // Reset the window start time
        }
    }

    // Emit remaining values if any after the flow completes
    if (buffer.isNotEmpty()) {
        val average = buffer.map { it.toFloat() }.average()
        emit(average)
    }
}

fun Flow<Event.VfsWrite>.toBucketedData(nanosTimeframeDuration: ULong) = flow {
    val collectedEvents = mutableMapOf<ULong, MutableList<Event.VfsWrite>>()
    this@toBucketedData.collect {

        // Remove old
        val currentTime = SystemClock.elapsedRealtimeNanos()
        collectedEvents.entries.forEach { (_, vfsWriteEventsList) ->
            vfsWriteEventsList.removeAll {
                currentTime.toULong() - it.beginTimeStamp > nanosTimeframeDuration
            }
        }
        collectedEvents.entries.removeAll { (_, vfsWriteEventsList) ->
            vfsWriteEventsList.isEmpty()
        }

        // Add new
        val key = it.fp
        val currentBucketEntries = collectedEvents.getOrElse(key) { mutableListOf() }
        currentBucketEntries.add(it)
        collectedEvents[key] = currentBucketEntries

        // Emit update
        emit(
            collectedEvents.entries.map { (fileDescriptor, writeEventsList) ->
                fileDescriptor to writeEventsList.sumOf { event -> event.bytesWritten }
            }
        )
    }
}

fun Flow<Event.JniReferences>.toReferenceCount() =
    this.scan(0 to 0) { prev, next ->
            when (next.jniMethodName) {
                Event.JniReferences.JniMethodName.AddLocalRef -> prev.first + 1 to prev.second
                Event.JniReferences.JniMethodName.DeleteLocalRef -> prev.first - 1 to prev.second
                Event.JniReferences.JniMethodName.AddGlobalRef -> prev.first to prev.second + 1
                Event.JniReferences.JniMethodName.DeleteGlobalRef -> prev.first to prev.second - 1
                null -> prev
            }
        }
        .map { it.first + it.second }

fun Flow<Event.SysFdTracking>.countFileDescriptors() =
    this.scan(0 to 0) { prev, next ->
            when (next.fdAction) {
                Event.SysFdTracking.SysFdAction.Created -> prev.first + 1 to prev.second
                Event.SysFdTracking.SysFdAction.Destroyed -> prev.first - 1 to prev.second
                null -> prev
            }
        }
        .map { it.first + it.second }

@OptIn(FlowPreview::class)
fun Flow<Event.Gc>.toMultiMemoryGraphData(intervalMillis: Long) =
    this.map { it.numBytesAllocated to max(it.targetFootprint, it.numBytesAllocated) }

fun Flow<Pair<ULong, ULong>>.toTimestampedMultiSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Pair<Float, Float>>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to (next.first.toFloat() to next.second.toFloat())).takeLast(seriesSize)
    }

fun Flow<List<Pair<ULong, Double>>>.sortAndClip(limit: Int) =
    this.map { it.sortedBy { (fd, size) -> size }.reversed().take(limit) }.conflate()

@Suppress("MagicNumber") // unit conversion
fun ULong.nanosToSeconds(): String {
    val locale = Locale.current.platformLocale
    return String.format(locale, "%.4f", this.toDouble() / 1_000_000_000)
}

fun <E> Flow<E>.accumulateEvents() =
    this.scan(initial = listOf<E>()) { prev, next -> prev.plus(next) }

fun List<Pair<Float, Float>>.isDefaultSeries(): Boolean {
    return this == DEFAULT_TIMESERIES_DATA
}
