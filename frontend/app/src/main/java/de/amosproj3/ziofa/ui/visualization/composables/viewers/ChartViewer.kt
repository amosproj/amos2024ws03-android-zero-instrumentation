// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.viewers

import androidx.compose.runtime.Composable
import de.amosproj3.ziofa.ui.visualization.composables.chart.VicoBar
import de.amosproj3.ziofa.ui.visualization.composables.chart.VicoTimeSeries
import de.amosproj3.ziofa.ui.visualization.data.GraphedData

@Composable
fun ChartViewer(data: GraphedData) {
    when (data) {
        is GraphedData.TimeSeriesData ->
            VicoTimeSeries(seriesData = data.seriesData, chartMetadata = data.metaData)

        is GraphedData.HistogramData ->
            VicoBar(seriesData = data.seriesData, chartMetadata = data.metaData)

        GraphedData.EMPTY -> {}
        else -> TODO() // crash because we havent implemented the visualization type yet
    }
}
