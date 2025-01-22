// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.overlay

import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import de.amosproj3.ziofa.ui.visualization.data.SelectionData

sealed class OverlayState(open val overlaySettings: OverlaySettings) {
    data class Disabled(override val overlaySettings: OverlaySettings) :
        OverlayState(overlaySettings)

    data class Enabled(
        override val overlaySettings: OverlaySettings,
        val selectionData: SelectionData? = null,
    ) : OverlayState(overlaySettings)
}
