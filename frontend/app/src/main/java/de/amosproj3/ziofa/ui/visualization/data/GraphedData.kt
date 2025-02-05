// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import kotlinx.collections.immutable.ImmutableList

data class EventListEntry(val col1: String, val col2: String, val col3: String, val col4: String)

sealed class GraphedData {
    data class TimeSeriesData(
        val seriesData: ImmutableList<Pair<Float, Float>>,
        val metaData: ChartMetadata,
    ) : GraphedData()

    data class MultiTimeSeriesData(
        val seriesData: ImmutableList<Pair<Float, Pair<Float, Float>>>,
        val metaData: ChartMetadata,
    ) : GraphedData()

    data class HistogramData(
        val seriesData: ImmutableList<Pair<String, Double>>,
        val metaData: ChartMetadata,
    ) : GraphedData()

    data class EventListData(val eventData: ImmutableList<EventListEntry>) : GraphedData()

    data object EMPTY : GraphedData()
}
