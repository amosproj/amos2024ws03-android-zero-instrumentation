// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

/** State of the visualization screen, partitioned by different visualization modes. */
sealed class VisualizationScreenState {

    /** Valid states display the [selectionData] (the selectors). */
    sealed class Valid(open val selectionData: SelectionData) : VisualizationScreenState() {

        /** Screen displays data as chart. */
        data class ChartView(
            val graphedData: GraphedData,
            override val selectionData: SelectionData,
        ) : Valid(selectionData)

        /** Screen displays data as event list. */
        data class EventListView(
            val graphedData: GraphedData.EventListData,
            override val selectionData: SelectionData,
            val eventListMetadata: EventListMetadata,
        ) : Valid(selectionData)

        /** Screen displays the overlay launcher. */
        data class OverlayView(
            override val selectionData: SelectionData,
            val overlaySettings: OverlaySettings,
            val overlayEnabled: Boolean,
        ) : Valid(selectionData)
    }

    /** States that are not errors, but are waiting for a valid selection. */
    sealed class Incomplete(open val selectionData: SelectionData) : VisualizationScreenState() {

        /** Screen is waiting for a valid selection to be made. */
        data class WaitingForMetricSelection(
            override val selectionData: SelectionData,
            val displayMode: VisualizationDisplayMode,
        ) : Incomplete(selectionData)

        /** The selection is technically valid, but there is visualization for the feature. */
        data class NoVisualizationExists(
            override val selectionData: SelectionData,
            val displayMode: VisualizationDisplayMode,
        ) : Incomplete(selectionData)
    }

    /** An error has occured and an error screen should be displayed */
    data class Invalid(val error: String) : VisualizationScreenState()

    /** Some background processing is happening and a loading spinner should be displayed. */
    data object Loading : VisualizationScreenState()
}
