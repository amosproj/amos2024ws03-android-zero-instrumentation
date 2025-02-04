// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.StateFlow

/**
 * Provides access to the [ConfigurationState] and allows modifying it.
 *
 * @property configurationState State of the synchronization between backend and frontend
 *   configuration.
 */
interface ConfigurationAccess {
    val configurationState: StateFlow<ConfigurationState>

    /**
     * Performs an action on the current [ConfigurationState].
     *
     * @param action action to perform.
     */
    suspend fun performAction(action: ConfigurationAction)
}
