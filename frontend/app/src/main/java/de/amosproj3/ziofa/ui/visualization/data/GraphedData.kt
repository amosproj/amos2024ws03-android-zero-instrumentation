// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import de.amosproj3.ziofa.api.WriteEvent

sealed class GraphedData {
    data class TimeSeriesData(val data: List<Pair<Float, Float>>) : GraphedData()

    data class HistogramData(val data: List<Pair<ULong, ULong>>) : GraphedData()

    data class EventListData(val data: List<WriteEvent>) : GraphedData()

    data object EMPTY : GraphedData()
}
