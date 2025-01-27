// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.chart

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import co.yml.charts.axis.AxisData
import co.yml.charts.common.model.Point
import co.yml.charts.ui.linechart.LineChart
import co.yml.charts.ui.linechart.model.GridLines
import co.yml.charts.ui.linechart.model.IntersectionPoint
import co.yml.charts.ui.linechart.model.Line
import co.yml.charts.ui.linechart.model.LineChartData
import co.yml.charts.ui.linechart.model.LinePlotData
import co.yml.charts.ui.linechart.model.LineStyle
import co.yml.charts.ui.linechart.model.SelectionHighlightPoint
import co.yml.charts.ui.linechart.model.ShadowUnderLine
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.utils.bytesToHumanReadableSize
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList

private const val Y_SIZE_OVERLAY = 3
private const val Y_SIZE = 10
private val Y_STEP_SIZE_OVERLAY = 50.dp
private val Y_STEP_SIZE = 100.dp
private const val LINE_WIDTH_OVERLAY = 2f
private const val LINE_WIDTH = 4f
private val POINT_SIZE_OVERLAY = 2.dp
private val POINT_SIZE = 4.dp
private const val SERIES_SIZE_OVERLAY = 6

// Vico does not support drawing multiple lines, so we use YCharts for multiple lines
@Composable
fun YChartsMultiTimeSeries(
    seriesData: ImmutableList<Pair<Float, Pair<Float, Float>>>,
    chartMetadata: ChartMetadata,
    modifier: Modifier = Modifier,
    overlayMode: Boolean = false,
) {
    Column(modifier.padding(20.dp), verticalArrangement = Arrangement.Center) {
        if (seriesData.isNotEmpty()) {
            Box(Modifier.fillMaxSize()) {
                Chart(
                    seriesData = seriesData.clip(overlayMode),
                    chartMetadata = chartMetadata,
                    overlayMode = overlayMode,
                )
                Legend(Modifier.align(Alignment.TopEnd))
            }
        } else WaitingForDataHint()
    }
}

@Composable
private fun Legend(modifier: Modifier = Modifier) {
    Row(modifier.padding(end = 50.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(
            "◼",
            color = MaterialTheme.colorScheme.secondary,
            modifier = Modifier.padding(horizontal = 5.dp),
        )
        Text("Used heap size")
        Spacer(Modifier.width(20.dp))
        Text(
            "◼",
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(horizontal = 5.dp),
        )
        Text("Total heap size")
    }
}

@Composable
private fun Chart(
    seriesData: ImmutableList<Pair<Float, Pair<Float, Float>>>,
    chartMetadata: ChartMetadata,
    overlayMode: Boolean,
) {
    val pointsData1: List<Point> = seriesData.map { Point(it.first, it.second.first) }
    val pointsData2: List<Point> = seriesData.map { Point(it.first, it.second.second) }
    val ySteps = if (overlayMode) Y_SIZE_OVERLAY else Y_SIZE
    val xLabels = seriesData.map { it.first.toString() }

    val xAxisData = buildXAxis(xLabels = xLabels, overlayMode = overlayMode)

    val yAxisData =
        buildYAxis(
            yMin = pointsData1.plus(pointsData2).minOf { it.y },
            yMax = pointsData2.plus(pointsData2).maxOf { it.y },
            ySteps = ySteps,
        )

    val lineChartData =
        LineChartData(
            linePlotData = createLineChartData(pointsData1, pointsData2, overlayMode),
            xAxisData = xAxisData,
            yAxisData = yAxisData,
            gridLines = GridLines(),
            backgroundColor = Color.White,
        )

    LineChart(modifier = Modifier, lineChartData = lineChartData)
}

private fun buildYAxis(yMin: Float, yMax: Float, ySteps: Int) =
    AxisData.Builder()
        .steps(ySteps)
        .labelAndAxisLinePadding(20.dp)
        .labelData { i ->
            val yScale = (yMax - yMin) / ySteps
            ((i * yScale) + yMin).toDouble().bytesToHumanReadableSize()
        }
        .build()

private fun buildXAxis(xLabels: List<String>, overlayMode: Boolean) =
    AxisData.Builder()
        .axisStepSize(if (overlayMode) Y_STEP_SIZE_OVERLAY else Y_STEP_SIZE)
        .steps(xLabels.size-1)
        .labelData { i -> xLabels.getOrNull(i) ?: "" }
        .labelAndAxisLinePadding(20.dp)
        .build()

@Composable
private fun createLineChartData(
    line1Points: List<Point>,
    line2Points: List<Point>,
    overlayMode: Boolean,
) =
    LinePlotData(
        lines =
            listOf(
                Line(
                    dataPoints = line2Points,
                    LineStyle(
                        width = if (overlayMode) LINE_WIDTH_OVERLAY else LINE_WIDTH,
                        color = MaterialTheme.colorScheme.primary,
                    ),
                    IntersectionPoint(),
                    SelectionHighlightPoint(),
                    ShadowUnderLine(color = MaterialTheme.colorScheme.primary),
                ),
                Line(
                    dataPoints = line1Points,
                    LineStyle(
                        width = if (overlayMode) LINE_WIDTH_OVERLAY else LINE_WIDTH,
                        color = MaterialTheme.colorScheme.secondary,
                    ),
                    IntersectionPoint(radius = if (overlayMode) POINT_SIZE_OVERLAY else POINT_SIZE),
                    SelectionHighlightPoint(),
                    ShadowUnderLine(color = MaterialTheme.colorScheme.secondary),
                ),
            )
    )

private fun ImmutableList<Pair<Float, Pair<Float, Float>>>.clip(overlayMode: Boolean) =
    if (overlayMode) this.takeLast(SERIES_SIZE_OVERLAY).toImmutableList() else this
