// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.client.Event
import de.amosproj3.ziofa.ui.shared.HISTOGRAM_BUCKETS
import de.amosproj3.ziofa.ui.shared.TIME_SERIES_SIZE
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import de.amosproj3.ziofa.ui.visualization.data.VisualizationMetaData
import kotlin.time.toDuration
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

fun Flow<Event.VfsWrite>.toBucketedHistogram(
    visualizationMetaData: VisualizationMetaData,
    timeframe: DropdownOption.Timeframe,
) =
    this.toBucketedData(timeframe.amount.toDuration(timeframe.unit).inWholeMilliseconds.toULong())
        .sortAndClip(HISTOGRAM_BUCKETS)
        .map { GraphedData.HistogramData(it, visualizationMetaData) }

fun Flow<Event.SysSendmsg>.toMovingAverage(
    visualizationMetaData: VisualizationMetaData,
    timeframe: DropdownOption.Timeframe,
) =
    this.toAveragedDurationOverTimeframe(
            TIME_SERIES_SIZE,
            timeframe.amount.toDuration(timeframe.unit).inWholeMilliseconds,
        )
        .map { GraphedData.TimeSeriesData(it, visualizationMetaData) }

fun Flow<Event>.toEventList() = this.accumulateEvents().map { GraphedData.EventListData(it) }
