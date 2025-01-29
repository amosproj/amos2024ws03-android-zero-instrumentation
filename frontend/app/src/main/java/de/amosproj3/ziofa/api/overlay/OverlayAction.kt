// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.overlay

import de.amosproj3.ziofa.ui.visualization.data.OverlaySettings
import de.amosproj3.ziofa.ui.visualization.data.SelectionData

sealed class OverlayAction {
    data class ChangeSettings(val newSettings: OverlaySettings) : OverlayAction()

    data class Enable(val selectionData: SelectionData) : OverlayAction()

    data object Disable : OverlayAction()
}
