// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.api.events.BackendEvent
import de.amosproj3.ziofa.ui.shared.HISTOGRAM_BUCKETS
import de.amosproj3.ziofa.ui.shared.TIME_SERIES_SIZE
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import kotlin.time.toDuration
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

fun Flow<BackendEvent.VfsWriteEvent>.toBucketedHistogram(
    visualizationMetaData: VisualizationMetaData,
    timeframe: DropdownOption.TimeframeOption,
) =
    this.toBucketedData(timeframe.amount.toDuration(timeframe.unit).inWholeMilliseconds.toULong())
        .sortAndClip(HISTOGRAM_BUCKETS)
        .map { GraphedData.HistogramData(it, visualizationMetaData) }

fun Flow<BackendEvent.SendMessageEvent>.toMovingAverage(
    visualizationMetaData: VisualizationMetaData,
    timeframe: DropdownOption.TimeframeOption,
) =
    this.toAveragedDurationOverTimeframe(
            TIME_SERIES_SIZE,
            timeframe.amount.toDuration(timeframe.unit).inWholeMilliseconds,
        )
        .map { GraphedData.TimeSeriesData(it, visualizationMetaData) }

fun Flow<BackendEvent>.toEventList() = this.accumulateEvents().map { GraphedData.EventListData(it) }
