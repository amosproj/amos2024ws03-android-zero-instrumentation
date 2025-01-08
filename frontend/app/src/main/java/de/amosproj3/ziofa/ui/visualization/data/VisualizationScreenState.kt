// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

sealed class VisualizationScreenState {
    data class ChartView(val graphedData: GraphedData, val selectionData: SelectionData) :
        VisualizationScreenState()

    data class EventListView(
        val graphedData: GraphedData.EventListData,
        val selectionData: SelectionData,
        val eventListMetadata: EventListMetadata,
    ) : VisualizationScreenState()

    data class WaitingForMetricSelection(
        val selectionData: SelectionData,
        val displayMode: VisualizationDisplayMode,
    ) : VisualizationScreenState()

    data class Invalid(val error: String) : VisualizationScreenState()

    data object Loading : VisualizationScreenState()
}
