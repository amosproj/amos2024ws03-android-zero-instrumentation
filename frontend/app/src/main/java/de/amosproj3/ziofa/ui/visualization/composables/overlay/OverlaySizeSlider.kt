// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.overlay

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
fun OverlaySizeSlider(
    overlaySizePct: Float,
    onOverlaySizeChanged: (Float) -> Unit,
    modifier: Modifier = Modifier,
) {
    var overlaySize by remember { mutableFloatStateOf(overlaySizePct) }

    Column(modifier = modifier.padding(horizontal = 10.dp)) {
        Text(
            "Overlay Size",
            fontWeight = FontWeight.Bold,
        )

        Spacer(Modifier.height(10.dp))

        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Slider(
                value = overlaySize,
                onValueChange = { overlaySize = it },
                valueRange = 0f..0.99f,
                onValueChangeFinished = { onOverlaySizeChanged(overlaySize) },
            )
            Text(text = "${(overlaySize * 100).toInt()}% of screen")
        }
    }
}
