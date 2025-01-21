// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Warning
import androidx.compose.runtime.Composable
import de.amosproj3.ziofa.ui.visualization.composables.CenteredInfoText

@Composable
fun VisualizationNonExistentPrompt() {
    CenteredInfoText(
        text =
            "There is no visualization configured for this feature. \n" +
                "Please switch to a different mode.",
        icon = Icons.Filled.Warning,
    )
}
