// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.viewers

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.composables.overlay.OverlayAxisLegend
import de.amosproj3.ziofa.ui.visualization.composables.overlay.OverlayPositionSelector
import de.amosproj3.ziofa.ui.visualization.composables.overlay.OverlaySizeSlider
import de.amosproj3.ziofa.ui.visualization.composables.overlay.TrafficLightIndicator
import de.amosproj3.ziofa.ui.visualization.data.ChartMetadata
import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import kotlinx.collections.immutable.toImmutableList

private const val OVERLAY_TITLE_SIZE = 30f
private const val OVERLAY_BUTTON_SIZE = 20f

@Suppress("MagicNumber") // does not improve readability
@Composable
fun OverlayLauncher(
    overlaySettings: OverlaySettings,
    overlayEnabled: Boolean,
    overlayStatusChanged: (Boolean) -> Unit,
    overlaySettingsChanged: (OverlaySettings) -> Unit,
    chartMetadata: ChartMetadata,
    modifier: Modifier = Modifier,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceEvenly,
        modifier = modifier.fillMaxSize(),
    ) {
        Column(
            modifier = Modifier.weight(0.3f),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            TrafficLightIndicator(overlayEnabled)
        }
        Column(
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.weight(1f),
        ) {
            Text(
                "Overlay Launcher",
                fontSize = TextUnit(OVERLAY_TITLE_SIZE, TextUnitType.Sp),
                fontWeight = FontWeight.Bold,
            )

            OverlayPositionSelector(
                selectedPosition = overlaySettings.selectedPosition,
                options = overlaySettings.availablePositions.toImmutableList(),
                onPositionSelected = {
                    overlaySettingsChanged(overlaySettings.copy(selectedPosition = it))
                },
            )

            Spacer(Modifier.height(10.dp))
            HorizontalDivider(
                Modifier.height(3.dp).padding(horizontal = 10.dp),
                color = Color.LightGray,
            )
            Spacer(Modifier.height(10.dp))

            OverlaySizeSlider(
                overlaySizePct = overlaySettings.pctOfScreen,
                onOverlaySizeChanged = {
                    overlaySettingsChanged(overlaySettings.copy(pctOfScreen = it))
                },
            )

            Spacer(Modifier.height(10.dp))
            HorizontalDivider(
                Modifier.height(3.dp).padding(horizontal = 100.dp),
                color = Color.LightGray,
            )

            Spacer(Modifier.height(10.dp))
            OverlayAxisLegend(chartMetadata = chartMetadata)
            Spacer(Modifier.height(10.dp))

            HorizontalDivider(
                Modifier.height(3.dp).padding(horizontal = 100.dp),
                color = Color.LightGray,
            )

            Spacer(Modifier.height(10.dp))

            Button(
                onClick = { overlayStatusChanged(!overlayEnabled) },
                content = {
                    Text(
                        text = if (overlayEnabled) "Disable overlay" else "Enable overlay",
                        fontSize = TextUnit(OVERLAY_BUTTON_SIZE, TextUnitType.Sp),
                    )
                },
            )
        }
    }
}
