// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.overlay

import kotlinx.coroutines.flow.Flow

/** Bridge between the frontend of the application and the overlay.*/
interface OverlayController {
    /** The current state of the overlay */
    val overlayState: Flow<OverlayState>

    /** Performs an action on the current [OverlayState].
     * @param action action to perform.
     * */
    fun performAction(action: OverlayAction)
}
