// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import de.amosproj3.ziofa.api.events.BackendEvent

sealed class GraphedData {
    data class TimeSeriesData(
        val seriesData: List<Pair<Float, Float>>,
        val metaData: VisualizationMetaData,
    ) : GraphedData()

    data class HistogramData(
        val seriesData: List<Pair<ULong, ULong>>,
        val metaData: VisualizationMetaData,
    ) : GraphedData()

    data class EventListData(val eventData: List<BackendEvent>) : GraphedData()

    data object EMPTY : GraphedData()
}
