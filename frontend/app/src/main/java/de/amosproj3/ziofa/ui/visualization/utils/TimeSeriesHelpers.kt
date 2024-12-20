// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.api.events.BackendEvent
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import kotlin.time.toDuration
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.buffer
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.sample
import kotlinx.coroutines.flow.scan

fun Flow<UInt>.countPerTimeframe(timeframeOption: DropdownOption.TimeframeOption): Flow<UInt> =
    flow {
        var previousCount: UInt = 0u
        this@countPerTimeframe.collect { currentCount ->
            val countedLastTimeframe = currentCount - previousCount
            emit(countedLastTimeframe)
            previousCount = currentCount
            delay(timeframeOption.amount.toDuration(timeframeOption.unit))
        }
    }

fun Flow<UInt>.toTimestampedSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Float>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to next.toFloat()).takeLast(seriesSize)
    }

fun Flow<BackendEvent>.toAveragedDurationOverTimeframe(
    seriesSize: Int,
    millisTimeframeDuration: Long,
) =
    this.buffer().windowed(millisTimeframeDuration).scan(listOf<Pair<Float, Float>>()) { prev, next
        ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + 1
        prev.plus(idx to next.toFloat()).takeLast(seriesSize)
    }

fun Flow<BackendEvent>.windowed(windowMillis: Long): Flow<Double> = flow {
    val buffer = mutableListOf<ULong>()
    var windowStart = System.currentTimeMillis()

    this@windowed.collect { value ->
        val now = System.currentTimeMillis()
        buffer.add(value.durationOrSize)

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

fun Flow<BackendEvent>.toBucketedData(millisTimeframeDuration: ULong) = flow {
    val collectedEvents = mutableMapOf<ULong, MutableList<BackendEvent>>()
    this@toBucketedData.collect {

        // Remove old
        val currentTime = System.currentTimeMillis()
        collectedEvents.entries.forEach { (_, vfsWriteEventsList) ->
            vfsWriteEventsList.removeAll {
                currentTime.toULong() - it.startTimestamp > millisTimeframeDuration
            }
        }
        collectedEvents.entries.removeAll { (_, vfsWriteEventsList) ->
            vfsWriteEventsList.isEmpty()
        }

        // Add new
        val key = it.fileDescriptor
        val currentBucketEntries = collectedEvents.getOrElse(key) { mutableListOf() }
        currentBucketEntries.add(it)
        collectedEvents[key] = currentBucketEntries

        // Emit update
        emit(
            collectedEvents.entries.map { (fileDescriptor, writeEventsList) ->
                fileDescriptor to writeEventsList.sumOf { event -> event.durationOrSize }
            }
        )
    }
}

@OptIn(FlowPreview::class)
fun Flow<List<Pair<ULong, ULong>>>.sortAndClip(limit: Int) =
    this.map { it.sortedBy { (fd, size) -> size }.reversed().take(limit) }.conflate().sample(2500)

fun DropdownOption.TimeframeOption.toSeconds(): Float {
    return this.amount.toDuration(this.unit).inWholeMilliseconds / 1000.0f
}

fun Flow<BackendEvent>.accumulateEvents() =
    this.scan(initial = listOf<BackendEvent>()) { prev, next -> prev.plus(next) }

fun List<Pair<Float, Float>>.isDefaultSeries(): Boolean {
    return this == DEFAULT_TIMESERIES_DATA
}
