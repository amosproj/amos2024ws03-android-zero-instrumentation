// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.overlay

import kotlinx.coroutines.flow.Flow

interface OverlayController {
    val overlayState: Flow<OverlayState>

    fun dispatchAction(action: OverlayAction)
}
