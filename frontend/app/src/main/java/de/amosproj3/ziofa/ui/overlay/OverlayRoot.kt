// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.overlay

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.composables.chart.VicoTimeSeries
import de.amosproj3.ziofa.ui.visualization.composables.chart.YChartsMultiTimeSeries
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import org.koin.androidx.compose.koinViewModel
import timber.log.Timber

/** Root composable rendered into the overlay.
 * Depending on the overlayData and overlaySettings, we render the graphed data.
 * Currently, the overlay reuses the existing graphs that are also display in the app,
 * with the additional overlayMode=true flag on the graphs.
 * This flag will adjust the length of the data, as well as colors to improve contrast,
 * which is important as the background is transparent.
 */
@Composable
fun OverlayRoot(viewModel: OverlayViewModel = koinViewModel(), modifier: Modifier = Modifier) {

    val overlayData by remember(viewModel) { viewModel.overlayData }.collectAsState()
    val overlaySettings by remember(viewModel) { viewModel.overlaySettings }.collectAsState()

    val data = overlayData // Snapshot
    val configuration = remember { LocalConfiguration }.current
    val screenWidth = remember { configuration.screenWidthDp.dp }
    val screenHeight = remember { configuration.screenHeightDp.dp }
    Timber.i("$overlayData update in composable")

    Box(
        modifier =
            modifier.size(
                width = screenWidth * overlaySettings.pctOfScreen,
                height = screenHeight * overlaySettings.pctOfScreen,
            )
    ) {
        when (data) {
            is GraphedData.TimeSeriesData -> TimeSeriesOverlay(data)
            is GraphedData.MultiTimeSeriesData -> MultiTimeSeriesOverlay(data)
            else -> Unsupported(data::class.simpleName.toString())
        }
    }
}

@Composable
fun Unsupported(visualizationType: String, modifier: Modifier = Modifier) {
    Text("Overlay mode unsupported for visualization type $visualizationType", modifier = modifier)
}

@Composable
fun TimeSeriesOverlay(data: GraphedData.TimeSeriesData, modifier: Modifier = Modifier) {
    if (data.seriesData.isNotEmpty()) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = modifier) {
            Text("ZIOFA OVERLAY", color = Color.Red)
            VicoTimeSeries(seriesData = data.seriesData, data.metaData, overlayMode = true)
        }
    }
}

@Composable
fun MultiTimeSeriesOverlay(data: GraphedData.MultiTimeSeriesData, modifier: Modifier = Modifier) {
    if (data.seriesData.isNotEmpty()) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = modifier) {
            Text("ZIOFA OVERLAY", color = Color.Red)
            YChartsMultiTimeSeries(seriesData = data.seriesData, overlayMode = true)
        }
    }
}
