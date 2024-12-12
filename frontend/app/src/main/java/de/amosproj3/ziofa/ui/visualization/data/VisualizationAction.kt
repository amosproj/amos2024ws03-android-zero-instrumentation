// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

/** Interactions with the UI lead to these actions that are processed by the view model. */
sealed class VisualizationAction {

    /**
     * An option in a dropdown has changed, which one is determined by the [DropdownOption]
     * subclass.
     */
    data class OptionChanged(val option: DropdownOption) : VisualizationAction()

    /** The visualization display mode has changed. */
    data class ModeChanged(val newMode: VisualizationDisplayMode) : VisualizationAction()

    /** The overlay has been activated or deactivated by the user. */
    data class OverlayStatusChanged(val newState: Boolean) : VisualizationAction()

    /** The overlays settings were changed by the user. */
    data class OverlaySettingsChanged(val newSettings: OverlaySettings) : VisualizationAction()
}
