// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

import de.amosproj3.ziofa.ui.visualization.utils.VisualizationDisplayMode

sealed class VisualizationScreenState {
    data class MetricSelectionValid(
        val graphedData: GraphedData,
        val selectionData: SelectionData,
        val displayMode: VisualizationDisplayMode,
    ) : VisualizationScreenState()

    data class WaitingForMetricSelection(
        val selectionData: SelectionData,
        val displayMode: VisualizationDisplayMode,
    ) : VisualizationScreenState()

    data class Invalid(val error: String) : VisualizationScreenState()

    data object Loading : VisualizationScreenState()
}
