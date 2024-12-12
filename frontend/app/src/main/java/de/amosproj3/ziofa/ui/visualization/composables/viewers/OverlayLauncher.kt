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
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.visualization.composables.overlay.OverlayPositionSelector
import de.amosproj3.ziofa.ui.visualization.composables.overlay.OverlaySizeSlider
import de.amosproj3.ziofa.ui.visualization.composables.overlay.TrafficLightIndicator
import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import kotlinx.collections.immutable.toImmutableList

@Composable
fun OverlayLauncher(
    overlaySettings: OverlaySettings,
    overlayEnabled: Boolean,
    overlayStatusChanged: (Boolean) -> Unit,
    overlaySettingsChanged: (OverlaySettings) -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceEvenly,
        modifier = Modifier.fillMaxSize(),
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
                "Overlay Settings",
                fontSize = TextUnit(30f, TextUnitType.Sp),
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
            OverlaySizeSlider(
                overlaySizePct = overlaySettings.pctOfScreen,
                onOverlaySizeChanged = {
                    overlaySettingsChanged(overlaySettings.copy(pctOfScreen = it))
                },
            )

            Spacer(Modifier.height(20.dp))
            Button(
                onClick = { overlayStatusChanged(!overlayEnabled) },
                content = {
                    Text(
                        text = if (overlayEnabled) "Disable overlay" else "Enable overlay",
                        fontSize = TextUnit(20f, TextUnitType.Sp),
                    )
                },
            )
        }
    }
}
