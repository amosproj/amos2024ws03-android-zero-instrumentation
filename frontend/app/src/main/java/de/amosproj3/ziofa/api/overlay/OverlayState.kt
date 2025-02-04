// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.overlay

import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import de.amosproj3.ziofa.ui.visualization.data.SelectionData

/** State of the overlay */
sealed class OverlayState(open val overlaySettings: OverlaySettings) {

    /** The overlay is disabled, but we require the so we can change them while it is disabled */
    data class Disabled(override val overlaySettings: OverlaySettings) :
        OverlayState(overlaySettings)

    /** The overlay is enabled for the given [selectionData] */
    data class Enabled(
        override val overlaySettings: OverlaySettings,
        val selectionData: SelectionData? = null,
    ) : OverlayState(overlaySettings)
}
