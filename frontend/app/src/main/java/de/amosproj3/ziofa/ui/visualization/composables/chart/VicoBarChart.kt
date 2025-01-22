// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.chart

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.patrykandpatrick.vico.compose.cartesian.CartesianChartHost
import com.patrykandpatrick.vico.compose.cartesian.axis.rememberBottom
import com.patrykandpatrick.vico.compose.cartesian.axis.rememberStart
import com.patrykandpatrick.vico.compose.cartesian.cartesianLayerPadding
import com.patrykandpatrick.vico.compose.cartesian.layer.rememberColumnCartesianLayer
import com.patrykandpatrick.vico.compose.cartesian.rememberCartesianChart
import com.patrykandpatrick.vico.compose.cartesian.rememberVicoZoomState
import com.patrykandpatrick.vico.compose.common.component.rememberLineComponent
import com.patrykandpatrick.vico.compose.common.component.rememberShapeComponent
import com.patrykandpatrick.vico.compose.common.component.rememberTextComponent
import com.patrykandpatrick.vico.compose.common.component.shapeComponent
import com.patrykandpatrick.vico.compose.common.dimensions
import com.patrykandpatrick.vico.compose.common.fill
import com.patrykandpatrick.vico.core.cartesian.axis.HorizontalAxis
import com.patrykandpatrick.vico.core.cartesian.axis.VerticalAxis
import com.patrykandpatrick.vico.core.cartesian.data.CartesianChartModelProducer
import com.patrykandpatrick.vico.core.cartesian.data.columnSeries
import com.patrykandpatrick.vico.core.cartesian.layer.ColumnCartesianLayer
import com.patrykandpatrick.vico.core.common.shape.CorneredShape
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.utils.VICO_LINE_COLOR
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import timber.log.Timber

@Composable
fun VicoBar(
    chartMetadata: ChartMetadata,
    seriesData: ImmutableList<Pair<ULong, ULong>>,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier.padding(10.dp).fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        val modelProducer = remember { CartesianChartModelProducer() }
        if (seriesData.isNotEmpty()) {
            Timber.e("bar data $seriesData")
            modelProducer.SeriesUpdate(seriesData.map { it.second.toInt() })
            modelProducer.TimeSeriesChart(
                chartMetadata = chartMetadata,
                xLabels = seriesData.map { it.first.toString() }.toImmutableList(),
            )
        }
    }
}

@Composable
private fun CartesianChartModelProducer.TimeSeriesChart(
    chartMetadata: ChartMetadata,
    xLabels: ImmutableList<String>,
    modifier: Modifier = Modifier,
) {
    CartesianChartHost(
        chart =
            rememberCartesianChart(
                rememberColumnCartesianLayer(
                    ColumnCartesianLayer.ColumnProvider.series(
                        xLabels.map { _ ->
                            rememberLineComponent(
                                fill = fill(VICO_LINE_COLOR),
                                shape =
                                    CorneredShape.rounded(
                                        bottomLeftPercent = 40,
                                        bottomRightPercent = 40,
                                    ),
                            )
                        }
                    )
                ),
                startAxis =
                    VerticalAxis.rememberStart(
                        label = rememberTextComponent(),
                        titleComponent =
                            rememberTextComponent(
                                color = Color.White,
                                margins = dimensions(end = 4.dp),
                                padding = dimensions(8.dp, 2.dp),
                                background =
                                    rememberShapeComponent(
                                        fill = fill(MaterialTheme.colorScheme.secondary),
                                        shape = CorneredShape.Pill,
                                    ),
                            ),
                        title = chartMetadata.yLabel,
                    ),
                bottomAxis =
                    HorizontalAxis.rememberBottom(
                        titleComponent =
                            rememberTextComponent(
                                color = Color.White,
                                margins = dimensions(top = 4.dp),
                                padding = dimensions(8.dp, 2.dp),
                                background =
                                    shapeComponent(
                                        fill = fill(MaterialTheme.colorScheme.primary),
                                        shape = CorneredShape.Pill,
                                    ),
                            ),
                        title = chartMetadata.xLabel,
                        label = rememberTextComponent(),
                        guideline = null,
                        itemPlacer = remember { HorizontalAxis.ItemPlacer.segmented() },
                        valueFormatter = { _, value, _ ->
                            xLabels.getOrNull(value.toInt()) ?: "UNKNOWN"
                        },
                    ),
                layerPadding = cartesianLayerPadding(scalableStart = 16.dp, scalableEnd = 16.dp),
            ),
        modelProducer = this@TimeSeriesChart,
        modifier = modifier,
        zoomState = rememberVicoZoomState(zoomEnabled = false),
    )
}

@Composable
private fun CartesianChartModelProducer.SeriesUpdate(update: List<Int>) {
    LaunchedEffect(update) {
        withContext(Dispatchers.Default) {
            this@SeriesUpdate.runTransaction {
                this.extras {}
                Timber.e(update.toString())
                columnSeries { series(x = update.indices.toList().reversed(), y = update) }
            }
        }
    }
}
