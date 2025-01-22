// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables.overlay

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

/** Fancy status indicator */
@Composable
fun TrafficLightIndicator(active: Boolean, modifier: Modifier = Modifier) {
    val lightShapeModifier = Modifier.size(150.dp).clip(CircleShape).padding(10.dp)
    Column(modifier.background(Color.DarkGray).clip(CircleShape)) {
        Box(modifier = lightShapeModifier.background(if (active) Color.Green else Color.LightGray))
        Box(modifier = lightShapeModifier.background(Color.LightGray))
        Box(modifier = lightShapeModifier.background(if (!active) Color.Red else Color.LightGray))
    }
}
