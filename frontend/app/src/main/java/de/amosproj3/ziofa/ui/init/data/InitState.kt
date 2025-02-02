// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.init.data

/** State of a sanity check operation to check if the backend is alive.*/
sealed class InitState {

    /** The sanity check is pending. */
    data object Initializing : InitState()

    /** The sanity check was successful. */
    data object Initialized : InitState()

    /** The sanity check failed, maybe the backend is not running? */
    data class Error(val error: String) : InitState()
}
