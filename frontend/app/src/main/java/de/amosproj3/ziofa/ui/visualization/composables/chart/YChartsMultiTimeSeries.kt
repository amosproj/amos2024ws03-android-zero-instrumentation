package de.amosproj3.ziofa.ui.visualization.composables.chart

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import co.yml.charts.axis.AxisData
import co.yml.charts.common.extensions.formatToSinglePrecision
import co.yml.charts.common.model.Point
import co.yml.charts.ui.linechart.LineChart
import co.yml.charts.ui.linechart.model.GridLines
import co.yml.charts.ui.linechart.model.IntersectionPoint
import co.yml.charts.ui.linechart.model.Line
import co.yml.charts.ui.linechart.model.LineChartData
import co.yml.charts.ui.linechart.model.LinePlotData
import co.yml.charts.ui.linechart.model.LineStyle
import co.yml.charts.ui.linechart.model.SelectionHighlightPoint
import co.yml.charts.ui.linechart.model.SelectionHighlightPopUp
import co.yml.charts.ui.linechart.model.ShadowUnderLine
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import timber.log.Timber

@Composable
fun YChartsMultiTimeSeries(
    seriesData: ImmutableList<Pair<Float, Pair<Float, Float>>>,
    chartMetadata: ChartMetadata,
    overlayMode: Boolean = false,
    modifier: Modifier = Modifier,
) {
    Timber.i("$seriesData")
    Column(
        modifier.padding(10.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        if (seriesData.isNotEmpty()) {
            Chart(seriesData.takeLast(6).toImmutableList(), chartMetadata, overlayMode)
        }
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
    val ySteps = if(overlayMode) 3 else 10
    val xAxisData = AxisData.Builder()
        .axisStepSize(if (overlayMode) 50.dp else 100.dp)
        .steps(seriesData.size-1)
        .labelData { i -> seriesData.getOrNull(i)?.first.toString() }
        .labelAndAxisLinePadding(20.dp)
        .build()

    val yAxisData = AxisData.Builder()
        .steps(ySteps)
        .labelAndAxisLinePadding(20.dp)
        .labelData { i ->
            val yMin = pointsData1.plus(pointsData2).minOf { it.y }
            val yMax = pointsData2.plus(pointsData2).maxOf { it.y }
            val yScale = (yMax - yMin) / ySteps
            bytesToHumanReadableSize(((i * yScale) + yMin).toDouble())
        }
        .build()

    val lineChartData = LineChartData(
        linePlotData = LinePlotData(
            lines = listOf(
                Line(
                    dataPoints = pointsData2,
                    LineStyle(width = if(overlayMode) 2f else 4f),
                    IntersectionPoint(),
                    SelectionHighlightPoint(),
                    ShadowUnderLine(),
                    SelectionHighlightPopUp()
                ),
                Line(
                    dataPoints = pointsData1,
                    LineStyle(width = if(overlayMode) 2f else 4f),
                    IntersectionPoint(radius = if(overlayMode) 2.dp else 6.dp),
                    SelectionHighlightPoint(),
                    ShadowUnderLine(),
                    SelectionHighlightPopUp()
                ),
            )
        ),
        xAxisData = xAxisData,
        yAxisData = yAxisData,
        gridLines = GridLines(),
        backgroundColor = Color.White
    )

    LineChart(
        modifier = Modifier
            .padding(10.dp),
        lineChartData = lineChartData
    )

}

fun bytesToHumanReadableSize(bytes: Double) = when {
    bytes >= 1 shl 30 -> "%.1f GB".format(bytes / (1 shl 30))
    bytes >= 1 shl 20 -> "%.1f MB".format(bytes / (1 shl 20))
    bytes >= 1 shl 10 -> "%.0f kB".format(bytes / (1 shl 10))
    else -> "$bytes bytes"
}