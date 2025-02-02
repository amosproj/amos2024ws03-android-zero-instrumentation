// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.overlay

import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import de.amosproj3.ziofa.ui.visualization.data.SelectionData

/** Possible interaction with the overlay state*/
sealed class OverlayAction {

    /** Change the settings of the overlay, like size, position, etc. */
    data class ChangeSettings(val newSettings: OverlaySettings) : OverlayAction()

    /** Enable the overlay for the given [selectionData] */
    data class Enable(val selectionData: SelectionData) : OverlayAction()

    /** Disable the overlay */
    data object Disable : OverlayAction()
}
