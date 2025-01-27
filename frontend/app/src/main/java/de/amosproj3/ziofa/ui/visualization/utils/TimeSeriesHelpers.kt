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

fun <E> Flow<E>.accumulateEvents() =
    this.scan(initial = listOf<E>()) { prev, next -> prev.plus(next) }

fun Flow<Number>.toTimestampedSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Float>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to next.toFloat()).takeLast(seriesSize)
    }

fun Flow<List<Event.SysSendmsg>>.averageMessageDuration() =
    this.map {
        if(it.isNotEmpty()){
            val avg = it.map {
                it.durationNanoSecs.toLong()
            }.average()
            if(avg!=0.0) avg.nanosToMillis() else avg
        } else 0.0
    }

fun Flow<List<Event.VfsWrite>>.averagesPerFd() =
    this.map {
        it.groupBy { it.fp }
            .map { it.key to it.value.map { it.bytesWritten.toLong() }.sum().toDouble() }
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

fun Flow<Event.Gc>.toMultiMemoryGraphData() =
    this.map { it.numBytesAllocated to max(it.targetFootprint, it.numBytesAllocated) }

fun Flow<Pair<ULong, ULong>>.toTimestampedMultiSeries(seriesSize: Int, secondsPerDatapoint: Float) =
    this.scan(listOf<Pair<Float, Pair<Float, Float>>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + secondsPerDatapoint
        prev.plus(idx to (next.first.toFloat() to next.second.toFloat())).takeLast(seriesSize)
    }

fun Flow<Pair<ULong, ULong>>.toCountedMultiSeries(seriesSize: Int) =
    this.scan(listOf<Pair<Float, Pair<Float, Float>>>()) { prev, next ->
        val idx = (prev.lastOrNull()?.first ?: 0.0f) + 1
        prev.plus(idx to (next.first.toFloat() to next.second.toFloat())).takeLast(seriesSize)
    }

fun Flow<List<Pair<ULong, Double>>>.sortAndClip(limit: Int) =
    this.map { it.sortedBy { (fd, size) -> size }.reversed().take(limit) }.conflate()

fun List<Pair<Float, Float>>.isDefaultSeries(): Boolean {
    return this == DEFAULT_TIMESERIES_DATA
}
