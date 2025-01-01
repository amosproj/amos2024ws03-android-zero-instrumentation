// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

data class EventListEntry(val col1: String, val col2: String, val col3: String, val col4: String)

sealed class GraphedData {
    data class TimeSeriesData(
        val seriesData: List<Pair<Float, Float>>,
        val metaData: ChartMetadata,
    ) : GraphedData()

    data class HistogramData(
        val seriesData: List<Pair<ULong, ULong>>,
        val metaData: ChartMetadata,
    ) : GraphedData()

    data class EventListData(val eventData: List<EventListEntry>) : GraphedData()

    data object EMPTY : GraphedData()
}
