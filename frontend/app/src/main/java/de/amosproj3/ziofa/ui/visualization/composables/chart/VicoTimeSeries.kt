// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.chart

import android.graphics.Typeface
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import com.patrykandpatrick.vico.compose.cartesian.CartesianChartHost
import com.patrykandpatrick.vico.compose.cartesian.axis.rememberBottom
import com.patrykandpatrick.vico.compose.cartesian.axis.rememberStart
import com.patrykandpatrick.vico.compose.cartesian.cartesianLayerPadding
import com.patrykandpatrick.vico.compose.cartesian.layer.rememberLine
import com.patrykandpatrick.vico.compose.cartesian.layer.rememberLineCartesianLayer
import com.patrykandpatrick.vico.compose.cartesian.rememberCartesianChart
import com.patrykandpatrick.vico.compose.cartesian.rememberVicoZoomState
import com.patrykandpatrick.vico.compose.common.component.rememberShapeComponent
import com.patrykandpatrick.vico.compose.common.component.rememberTextComponent
import com.patrykandpatrick.vico.compose.common.component.shapeComponent
import com.patrykandpatrick.vico.compose.common.dimensions
import com.patrykandpatrick.vico.compose.common.fill
import com.patrykandpatrick.vico.core.cartesian.axis.HorizontalAxis
import com.patrykandpatrick.vico.core.cartesian.axis.VerticalAxis
import com.patrykandpatrick.vico.core.cartesian.data.CartesianChartModelProducer
import com.patrykandpatrick.vico.core.cartesian.data.lineSeries
import com.patrykandpatrick.vico.core.cartesian.layer.LineCartesianLayer
import com.patrykandpatrick.vico.core.common.shape.CorneredShape
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.utils.VICO_LINE_COLOR
import de.amosproj3.ziofa.ui.visualization.utils.isDefaultSeries
import kotlinx.collections.immutable.ImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import timber.log.Timber

@Composable
fun VicoTimeSeries(
    seriesData: ImmutableList<Pair<Float, Float>>,
    chartMetadata: ChartMetadata,
    modifier: Modifier = Modifier,
    overlayMode: Boolean = false,
) {
    Timber.i("$seriesData")
    Column(modifier.padding(10.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        val modelProducer = remember { CartesianChartModelProducer() }
        if (seriesData.isNotEmpty() && !seriesData.isDefaultSeries()) {
            modelProducer.SeriesUpdate(seriesData)
            modelProducer.TimeSeriesChart(chartMetadata, overlayMode)
        } else WaitingForDataHint()
    }
}

@Composable
private fun CartesianChartModelProducer.TimeSeriesChart(
    chartMetadata: ChartMetadata,
    overlayMode: Boolean,
    modifier: Modifier = Modifier,
) {
    val mainColor = remember { if (overlayMode) Color.Red else Color.Black }
    val typeface = remember { if (overlayMode) Typeface.DEFAULT_BOLD else Typeface.DEFAULT }
    val xLabel = remember { chartMetadata.xLabel }
    val yLabel = remember { chartMetadata.yLabel }

    CartesianChartHost(
        chart =
        rememberCartesianChart(
            rememberLineCartesianLayer(
                LineCartesianLayer.LineProvider.series(
                    LineCartesianLayer.rememberLine(
                        remember {
                            LineCartesianLayer.LineFill.single(
                                fill(if (overlayMode) mainColor else VICO_LINE_COLOR)
                            )
                        }
                    )
                )
            ),
            startAxis =
            VerticalAxis.rememberStart(
                label = rememberTextComponent(color = mainColor, typeface = typeface),
                titleComponent =
                if (overlayMode) null else rememberTextComponent(
                    textSize = TextUnit(15f, TextUnitType.Sp),
                    color = Color.White,
                    margins = dimensions(end = 4.dp),
                    padding = dimensions(8.dp, 2.dp),
                    background =
                    rememberShapeComponent(
                        fill = fill(MaterialTheme.colorScheme.secondary),
                        shape = CorneredShape.Pill,
                    ),
                ),
                title = if (overlayMode) null else yLabel,
            ),
            bottomAxis =
            HorizontalAxis.rememberBottom(
                label = rememberTextComponent(color = mainColor, typeface = typeface),
                titleComponent =
                if (overlayMode) null else rememberTextComponent(
                    textSize = TextUnit(15f, TextUnitType.Sp),
                    color = Color.White,
                    margins = dimensions(top = 4.dp),
                    padding = dimensions(8.dp, 2.dp),
                    background =
                    shapeComponent(
                        fill = fill(MaterialTheme.colorScheme.primary),
                        shape = CorneredShape.Pill,
                    ),
                ),
                title = if (overlayMode) null else xLabel,
                guideline = null,
                itemPlacer = remember { HorizontalAxis.ItemPlacer.segmented() },
            ),
            layerPadding = cartesianLayerPadding(scalableStart = 16.dp, scalableEnd = 16.dp),
        ),
        modelProducer = this@TimeSeriesChart,
        modifier = modifier,
        zoomState = rememberVicoZoomState(zoomEnabled = false),
    )
}

@Composable
private fun CartesianChartModelProducer.SeriesUpdate(update: List<Pair<Float, Float>>) {
    LaunchedEffect(update) {
        withContext(Dispatchers.Default) {
            this@SeriesUpdate.runTransaction {
                lineSeries { series(update.map { (x, _) -> x }, update.map { (_, y) -> y }) }
            }
        }
    }
}
